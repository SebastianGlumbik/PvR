#![allow(unused)]
use crate::messages::{ClientToServerMsg, ServerToClientMsg};
use crate::reader::MessageReader;
use crate::writer::MessageWriter;
use mio::net::TcpStream;
use mio::Token;
use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone)]
struct Wrapper(Rc<TcpStream>);

impl Read for Wrapper {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.as_ref().read(buf)
    }
}

impl Write for Wrapper {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.as_ref().write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.as_ref().flush()
    }
}

pub struct Client {
    username: Option<String>,
    writer: MessageWriter<ServerToClientMsg, Wrapper>,
    reader: MessageReader<ClientToServerMsg, Wrapper>,
    logged_in: std::time::Instant,
}

impl Client {
    pub fn new(stream: TcpStream) -> Client {
        let stream = Wrapper(Rc::new(stream));
        Client {
            username: None,
            writer: MessageWriter::new(stream.clone()),
            reader: MessageReader::new(stream),
            logged_in: std::time::Instant::now(),
        }
    }

    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    pub fn stream(&self) -> Rc<TcpStream> {
        self.writer.inner().0.clone()
    }

    /// Time since connection was established
    pub fn active(&self) -> Duration {
        self.logged_in.elapsed()
    }

    pub fn send_message(&mut self, message: ServerToClientMsg) -> anyhow::Result<()> {
        self.writer.send(message)
    }

    pub fn read_message(&mut self) -> Option<std::io::Result<ClientToServerMsg>> {
        self.reader.recv()
    }

    pub fn set_username(&mut self, username: String) {
        self.username = Some(username);
    }

    pub fn disconnect(mut self, message: Option<ServerToClientMsg>) {
        if let Some(message) = message {
            self.send_message(message).unwrap_or_default();
        }
        self.stream()
            .shutdown(std::net::Shutdown::Both)
            .unwrap_or_default();
    }
}

pub struct Clients {
    usernames: HashMap<String, Token>,
    data: HashMap<Token, Client>,
}

impl Clients {
    pub fn new(capacity: usize) -> Self {
        Self {
            usernames: HashMap::with_capacity(capacity),
            data: HashMap::with_capacity(capacity),
        }
    }

    pub fn exists(&self, username: &str) -> bool {
        self.usernames.contains_key(username)
    }

    pub fn insert(&mut self, token: Token, client: Client) -> Option<Client> {
        if let Some(username) = client.username() {
            if self.usernames.contains_key(username) {
                return Some(client);
            }
            self.usernames.insert(username.to_string(), token);
        }

        self.data.insert(token, client)
    }

    pub fn get_mut(&mut self, username: &str) -> Option<&mut Client> {
        self.usernames
            .get(username)
            .and_then(|token| self.data.get_mut(token))
    }

    pub fn remove(&mut self, token: &Token) -> Option<Client> {
        if let Some(client) = self.data.remove(token) {
            if let Some(username) = client.username() {
                self.usernames.remove(username);
            }
            Some(client)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn unnamed(&self) -> impl Iterator<Item=(&Token, &Client)> {
        self.data
            .iter()
            .filter(|(_, client)| client.username().is_none())
    }

    pub fn named(&mut self) -> impl Iterator<Item=(&Token, &mut Client)> {
        self.data
            .iter_mut()
            .filter(|(_, client)| client.username().is_some())
    }

    pub fn drain(&mut self) -> impl Iterator<Item=(Token, Client)> + use < '_ > {
        self.usernames
            .drain()
            .map(|(_, token)| (token, self.data.remove(&token).unwrap()))
    }

    pub fn get_usernames_list(&self) -> Vec<String> {
        self.usernames.keys().cloned().collect()
    }
}

pub fn handle_client(mut client: Client, clients: &mut Clients) -> Option<Client> {
    if client.username().is_none() {
        match client.read_message() {
            Some(Ok(ClientToServerMsg::Join { name })) => {
                if clients.exists(&name) {
                    client.disconnect(Some(ServerToClientMsg::Error(
                        "Username already taken".to_string(),
                    )));
                    return None;
                }

                client.set_username(name);
                client.send_message(ServerToClientMsg::Welcome).ok()?;
            }
            Some(Err(error)) if error.kind() == ErrorKind::WouldBlock => return Some(client),
            _ => {
                client.disconnect(Some(ServerToClientMsg::Error(
                    "Unexpected message received".to_string(),
                )));
                return None;
            }
        };
    }

    loop {
        match client.read_message() {
            Some(Ok(message)) => match message {
                ClientToServerMsg::Join { .. } => {
                    client.disconnect(Some(ServerToClientMsg::Error(
                        "Unexpected message received".to_string(),
                    )));
                    break None;
                }
                ClientToServerMsg::Ping => {
                    client.send_message(ServerToClientMsg::Pong).ok()?;
                }
                ClientToServerMsg::ListUsers => {
                    let mut users = clients.get_usernames_list();
                    users.push(client.username().unwrap().to_string());
                    users.sort();
                    client
                        .send_message(ServerToClientMsg::UserList { users })
                        .ok()?;
                }
                ClientToServerMsg::SendDM { to, message } => {
                    let from = client.username().unwrap().to_string();

                    if to == from {
                        client
                            .send_message(ServerToClientMsg::Error(
                                "Cannot send a DM to yourself".to_string(),
                            ))
                            .ok()?;
                    } else {
                        match clients.get_mut(&to) {
                            Some(to) => {
                                to.send_message(ServerToClientMsg::Message { from, message })
                                    .ok()?;
                            }
                            None => {
                                client
                                    .send_message(ServerToClientMsg::Error(format!(
                                        "User {} does not exist",
                                        to
                                    )))
                                    .ok()?;
                            }
                        }
                    }
                }
                ClientToServerMsg::Broadcast { message } => {
                    let from = client.username().unwrap().to_string();
                    for (_, to) in clients.named() {
                        to.send_message(ServerToClientMsg::Message {
                            from: from.clone(),
                            message: message.clone(),
                        })
                            .ok()?;
                    }
                }
            },
            Some(Err(error)) if error.kind() == ErrorKind::WouldBlock => break Some(client),
            _ => {
                client.disconnect(None);
                break None;
            }
        }
    }
}
