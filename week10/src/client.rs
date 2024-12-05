#![allow(unused)]
use crate::messages::ClientToServerMsg;
use crate::messages::ServerToClientMsg;
use crate::reader::MessageReader;
use crate::writer::MessageWriter;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::Sender;

pub struct Client {
    writer: MessageWriter<ServerToClientMsg, OwnedWriteHalf>,
    reader: MessageReader<ClientToServerMsg, OwnedReadHalf>,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        let (read, write) = stream.into_split();
        let writer = MessageWriter::<ServerToClientMsg, OwnedWriteHalf>::new(write);
        let reader = MessageReader::<ClientToServerMsg, OwnedReadHalf>::new(read);

        Self { writer, reader }
    }

    pub async fn send_message(&mut self, message: ServerToClientMsg) -> anyhow::Result<()> {
        self.writer.send(message).await
    }

    pub async fn read_message(&mut self) -> Option<std::io::Result<ClientToServerMsg>> {
        self.reader.recv().await
    }

    pub async fn disconnect(mut self, message: Option<ServerToClientMsg>) {
        if let Some(message) = message {
            self.send_message(message).await.unwrap_or_default();
        }
        self.writer
            .into_inner()
            .shutdown()
            .await
            .unwrap_or_default();
    }
}

pub struct Clients {
    clients: HashMap<String, Sender<ServerToClientMsg>>,
}

impl Clients {
    pub fn new(capacity: usize) -> Self {
        Self {
            clients: HashMap::with_capacity(capacity),
        }
    }

    pub fn add_client(&mut self, username: String, client: Sender<ServerToClientMsg>) -> bool {
        if self.clients.contains_key(&username) {
            return true;
        }

        self.clients.insert(username, client).is_some()
    }

    pub fn remove_client(&mut self, username: &str) {
        self.clients.remove(username);
    }

    pub fn get_client(&self, username: &str) -> Option<Sender<ServerToClientMsg>> {
        self.clients.get(username).cloned()
    }

    pub fn get_all_clients(&self) -> Vec<(String, Sender<ServerToClientMsg>)> {
        self.clients
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn get_usernames_list(&self) -> Vec<String> {
        self.clients.keys().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.clients.clear();
    }
}

pub async fn handle_client(mut client: Client, clients: Rc<RefCell<Clients>>) {
    let username = select! {
        message = client.read_message() => match message {
            Some(Ok(ClientToServerMsg::Join { name })) => name,
            _ => {
                client.disconnect(Some(ServerToClientMsg::Error("Unexpected message received".to_string()))).await;
                return;
            }
        },
        _ = tokio::time::sleep(Duration::from_secs(2)) => {
            client.disconnect(Some(ServerToClientMsg::Error("Timed out waiting for Join".to_string()))).await;
            return;
        }
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerToClientMsg>(1024);

    let result = { clients.borrow_mut().add_client(username.clone(), tx) };
    if result {
        client
            .disconnect(Some(ServerToClientMsg::Error(
                "Username already taken".to_string(),
            )))
            .await;
        return;
    }

    client
        .send_message(ServerToClientMsg::Welcome)
        .await
        .unwrap_or_default();

    let message = loop {
        select! {
            message = rx.recv() => match message {
                Some(message) => client.send_message(message).await.unwrap_or_default(),
                None => break None,
            },
            message = client.read_message() => match message {
                Some(Ok(message)) => match message {
                    ClientToServerMsg::Join{ .. } => break Some(ServerToClientMsg::Error("Unexpected message received".to_string())),
                    ClientToServerMsg::Ping => client.send_message(ServerToClientMsg::Pong).await.unwrap_or_default(),
                    ClientToServerMsg::ListUsers => {
                        let users = clients.borrow().get_usernames_list();
                        client.send_message(ServerToClientMsg::UserList{ users }).await.unwrap_or_default();
                    }
                    ClientToServerMsg::SendDM{to,message  } => {
                        if to == username {
                            client.send_message(ServerToClientMsg::Error(
                            "Cannot send a DM to yourself".to_string(),
                        )).await.unwrap_or_default();
                        continue;
                        }
                        let sender = clients.borrow().get_client(&to);
                        if let Some(sender) = sender {
                            sender.send(ServerToClientMsg::Message{ from: username.clone(), message }).await.unwrap_or_default();
                        } else {
                            client.send_message(ServerToClientMsg::Error(format!(
                                "User {} does not exist",
                                to
                            ))).await.unwrap_or_default();
                        }
                    }
                    ClientToServerMsg::Broadcast{  message } => {
                        let clients = clients.borrow().get_all_clients();
                        for (to, sender) in clients {
                            if to == username {
                                continue;
                            }
                            sender.send(ServerToClientMsg::Message{ from: username.clone(), message: message.clone() }).await.unwrap_or_default();
                        }
                    }
                },
                _ => break None,
            },
            _ = tokio::time::sleep(Duration::from_secs(3)) => {
                break Some(ServerToClientMsg::Error("Timeouted".to_string()))
            }
        }
    };

    {
        clients.borrow_mut().remove_client(&username)
    }
    client.disconnect(message).await;
}
