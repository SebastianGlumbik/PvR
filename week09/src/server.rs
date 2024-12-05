#![allow(unused)]
mod client;

use crate::messages::ServerToClientMsg;
use crate::ServerOpts;
use client::{handle_client, Client, Clients};
use mio::net::TcpListener;
use mio::unix::pipe::Receiver;
use mio::{Events, Interest, Poll, Token};
use std::io::{ErrorKind, Read, Write};
use std::os::fd::AsRawFd;
use std::rc::Rc;
use std::time::Duration;

const END: Token = Token(0);
const LISTENER: Token = Token(1);
const TIMEOUT_DURATION: Duration = Duration::from_secs(2);

pub fn server_loop(
    mut listener: TcpListener,
    mut end: Receiver,
    opts: ServerOpts,
) -> anyhow::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);
    poll.registry()
        .register(&mut end, END, Interest::READABLE)?;
    poll.registry()
        .register(&mut listener, LISTENER, Interest::READABLE)?;

    let mut clients = Clients::new(opts.max_clients);
    let mut tokens_to_disconnect = Vec::new();

    loop {
        let timeout = clients
            .unnamed()
            .map(|(_, client)| TIMEOUT_DURATION.saturating_sub(client.active()))
            .min();

        poll.poll(&mut events, timeout)?;
        for event in &events {
            match event.token() {
                END => {
                    for (.., client) in clients.drain() {
                        let mut stream = client.stream();
                        client.disconnect(None);
                        if let Some(stream) = Rc::get_mut(&mut stream) {
                            poll.registry().deregister(stream).unwrap_or_default();
                        }
                    }
                    return Ok(());
                }
                LISTENER => loop {
                    let mut connection = match listener.accept() {
                        Ok((connection, _)) => connection,
                        Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                        Err(error) => return Err(error.into()),
                    };

                    let token = Token(connection.as_raw_fd() as usize);
                    poll.registry().register(
                        &mut connection,
                        token,
                        Interest::READABLE.add(Interest::WRITABLE),
                    )?;

                    let client = Client::new(connection);

                    if clients.len() >= opts.max_clients {
                        let mut stream = client.stream();
                        client.disconnect(Some(ServerToClientMsg::Error(
                            "Server is full".to_string(),
                        )));
                        if let Some(stream) = Rc::get_mut(&mut stream) {
                            poll.registry().deregister(stream).unwrap_or_default();
                        }
                    } else {
                        clients.insert(token, client);
                    }
                },
                token => {
                    let Some(client) = clients.remove(&token) else {
                        // Can happen if the client was disconnected in the same loop iteration
                        eprintln!("unexpected token: {:?}", token);
                        continue;
                    };
                    let mut stream = client.stream();
                    if let Some(client) = handle_client(client, &mut clients) {
                        clients.insert(token, client);
                    } else if let Some(stream) = Rc::get_mut(&mut stream) {
                        poll.registry().deregister(stream).unwrap_or_default();
                    }
                }
            }
        }

        for (token, client) in clients.unnamed() {
            if client.active() >= TIMEOUT_DURATION {
                tokens_to_disconnect.push(*token);
            }
        }

        for token in tokens_to_disconnect.drain(..) {
            let Some(client) = clients.remove(&token) else {
                continue;
            };
            let mut stream = client.stream();
            client.disconnect(Some(ServerToClientMsg::Error(
                "Timed out waiting for Join".to_string(),
            )));
            if let Some(stream) = Rc::get_mut(&mut stream) {
                poll.registry().deregister(stream).unwrap_or_default();
            }
        }
    }
}
