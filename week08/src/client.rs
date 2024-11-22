use crate::messages::{ClientToServerMsg, ServerToClientMsg};
use crate::reader::MessageReader;
use crate::writer::MessageWriter;
use crate::SocketWrapper;
use std::collections::hash_map::Drain;
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub struct Client {
    stream: SocketWrapper,
    reader: Arc<Mutex<MessageReader<ClientToServerMsg, SocketWrapper>>>,
    writer: Arc<Mutex<MessageWriter<ServerToClientMsg, SocketWrapper>>>,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        let stream = SocketWrapper(Arc::new(stream));
        let reader = Arc::new(Mutex::new(
            MessageReader::<ClientToServerMsg, SocketWrapper>::new(stream.clone()),
        ));
        let writer = Arc::new(Mutex::new(
            MessageWriter::<ServerToClientMsg, SocketWrapper>::new(stream.clone()),
        ));

        Self {
            stream,
            reader,
            writer,
        }
    }

    pub fn send_message(&self, message: ServerToClientMsg) -> anyhow::Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.write(message)
    }

    pub fn read_message(&self) -> Option<anyhow::Result<ClientToServerMsg>> {
        let mut reader = self.reader.lock().unwrap();
        reader.read()
    }

    pub fn disconnect(self, message: Option<ServerToClientMsg>) {
        if let Some(message) = message {
            self.send_message(message).unwrap_or_default();
        }
        self.stream
            .0
            .shutdown(std::net::Shutdown::Both)
            .unwrap_or_default();
    }
}

impl Clone for Client {
    fn clone(&self) -> Self {
        let stream = self.stream.clone();
        let reader = self.reader.clone();
        let writer = self.writer.clone();
        Self {
            stream,
            reader,
            writer,
        }
    }
}

pub struct Clients {
    clients: HashMap<String, Client>,
}

impl Clients {
    pub fn new(capacity: usize) -> Self {
        Self {
            clients: HashMap::with_capacity(capacity),
        }
    }

    /// Return client back if username is already taken, otherwise return None
    pub fn add_client(&mut self, username: String, client: Client) -> Option<Client> {
        if self.clients.contains_key(&username) {
            return Some(client);
        }

        self.clients.insert(username, client)
    }

    pub fn remove_client(&mut self, username: &str) -> Option<Client> {
        self.clients.remove(username)
    }

    pub fn drain(&mut self) -> Drain<'_, String, Client> {
        self.clients.drain()
    }

    pub fn get_client(&self, username: &str) -> Option<Client> {
        self.clients.get(username).cloned()
    }

    pub fn iter(&self) -> impl Iterator<Item=(&String, &Client)> {
        self.clients.iter()
    }

    pub fn get_usernames_list(&self) -> Vec<String> {
        self.clients.keys().cloned().collect()
    }
}

/// Handles the client connection.
pub fn handle_client(client: Client, clients: Arc<Mutex<Clients>>) {
    let username = match client.read_message() {
        Some(Ok(ClientToServerMsg::Join { name })) => name,
        _ => {
            client.disconnect(Some(ServerToClientMsg::Error(
                "Unexpected message received".to_string(),
            )));
            return;
        }
    };

    {
        let mut clients = clients.lock().unwrap();
        if clients
            .add_client(username.clone(), client.clone())
            .is_some()
        {
            client.disconnect(Some(ServerToClientMsg::Error(
                "Username already taken".to_string(),
            )));
            return;
        }

        client
            .send_message(ServerToClientMsg::Welcome)
            .unwrap_or_default();
    }

    while let Some(Ok(message)) = client.read_message() {
        match message {
            ClientToServerMsg::Join { .. } => {
                let mut clients = clients.lock().unwrap();
                clients.remove_client(&username);
                client.disconnect(Some(ServerToClientMsg::Error(
                    "Unexpected message received".to_string(),
                )));
                return;
            }
            ClientToServerMsg::Ping => client
                .send_message(ServerToClientMsg::Pong)
                .unwrap_or_default(),
            ClientToServerMsg::ListUsers => {
                let clients = clients.lock().unwrap();
                let users = clients.get_usernames_list();
                drop(clients);
                client
                    .send_message(ServerToClientMsg::UserList { users })
                    .unwrap_or_default();
            }
            ClientToServerMsg::SendDM { to, message } => {
                if to == username {
                    client
                        .send_message(ServerToClientMsg::Error(
                            "Cannot send a DM to yourself".to_string(),
                        ))
                        .unwrap_or_default();
                    continue;
                }

                let clients = clients.lock().unwrap();
                let result = clients.get_client(&to);
                drop(clients);
                match result {
                    Some(to) => {
                        to.send_message(ServerToClientMsg::Message {
                            from: username.clone(),
                            message,
                        })
                            .unwrap_or_default();
                    }
                    None => {
                        client
                            .send_message(ServerToClientMsg::Error(format!(
                                "User {} does not exist",
                                to
                            )))
                            .unwrap_or_default();
                    }
                }
            }
            ClientToServerMsg::Broadcast { message } => {
                let clients = clients.lock().unwrap();
                for (to, client) in clients.iter() {
                    if &username != to {
                        client
                            .send_message(ServerToClientMsg::Message {
                                from: username.clone(),
                                message: message.clone(),
                            })
                            .unwrap_or_default();
                    }
                }
            }
        }
    }

    clients.lock().unwrap().remove_client(&username);
    client.disconnect(None);
}
