//! implement a TCP/IP client that will connect to a server so that you can play a game
//!
//! You can send numbers that represent various *actions* to the server.
//! Actions control your *blob*. Use the provided code to
//! control the blob with your keyboard and shoot other players!
//! Be wary that the server is a bit moody, and it will periodically remap which numbers correspond
//! to which actions. You need to take this into account, otherwise the server will punish you.
//!
//! # Communication protocol
//! After you connect to the server on a given TCP/IP v4 address and port, the following should
//! happen:
//! 1) You send a [`ClientToServerMsg::Join`] message that identifies you.
//! You have to choose a nickname.
//!
//! - You have to send the message within two seconds. If you don't, the server will disconnect you.
//! - If the nickname is not unique (someone else has the same nickname), you will be disconnected.
//! - The nickname cannot be longer than `15` (UTF-8) bytes.
//!
//! 2) The server responds with a [`ServerToClientMsg::ActionMappingUpdate`] message, which maps
//! numbers to actions. The first message will always have the following mapping:
//! ```
//! 0 => MoveForward
//! 1 => MoveBackward
//! 2 => TurnLeft
//! 3 => TurnRight
//! 4 => Shield
//! 5 => Fire
//! 6..=10 => Invalid
//! ```
//!
//! Periodically, the server will decide to change the mapping of numbers to actions, and send you
//! the `ActionMappingUpdate` message again. You should read it and update your local mapping, so
//! that you send the correct actions to the server.
//!
//! Use either [`tokio::select`] or Tokio tasks to make sure that your code can concurrently handle
//! incoming server messages, events from the user's keyboard, and sending a heartbeat (see below).
//!
//! 3) Read key events from the keyboard using the provided code, and map some keyboard keys
//! to actions (and then actions to numbers). After an action is produced by the corresponding key,
//! send the [`ClientToServerMsg::PerformAction`] message to the server.
//!
//! If you send an invalid action, the server will freeze your blob for a few seconds, and increase
//! incoming damage by 100%.
//!
//! # Spam protection
//! You must not send a message more often than once every 0.1 milliseconds. If
//! you do, you will receive a strike. After accumulating three strikes, you will be disconnected.
//!
//! # Inactivity protection
//! You have to send the [`ClientToServerMsg::Heartbeat`] message at least once every five seconds,
//! otherwise you will be disconnected. You must not send it more often than once per second, though.
//!
//! # Message encoding
//! You can use the provided [`MessageReader`] and [`MessageWriter`] structs to communicate with the
//! server.
//!
//! Bonus point if you can crash the server :)

use crate::messages::{Action, ClientToServerMsg, ServerToClientMsg};
use crate::reader::MessageReader;
use crate::writer::MessageWriter;
use anyhow::anyhow;
use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use std::collections::HashMap;
use std::io::Write;
use std::time::Duration;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::{select, time};

mod messages;
mod reader;
mod writer;

/// You can use this macro for a bit nicer debugging output.
macro_rules! output {
    ($lit: literal) => {
        output!($lit,);
    };
    ($lit: literal, $($arg:tt),*) => {
        ::crossterm::terminal::disable_raw_mode().unwrap();
        println!($lit, $($arg),*);
        std::io::stdout().flush().unwrap();
        ::crossterm::terminal::enable_raw_mode().unwrap();
    };
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    // Enable raw mode so that input key events are not buffered
    crossterm::terminal::enable_raw_mode()?;
    let result = run().await;
    crossterm::terminal::disable_raw_mode()?;
    result
}

async fn run() -> anyhow::Result<()> {
    let client = TcpStream::connect("").await?;
    let (stream, sink) = client.into_split();

    let (mut rx, mut tx) = (
        MessageReader::<ServerToClientMsg, _>::new(stream),
        MessageWriter::<ClientToServerMsg, _>::new(sink),
    );

    tx.send(ClientToServerMsg::Join {
        name: "name".to_string(),
    })
        .await?;

    // Asynchronous stream of events from the keyboard
    // You can get a future with the next key using `keys.next()`
    let mut keys = EventStream::new();

    let mut action_mapping: HashMap<Action, u8> = HashMap::new();
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        select! {
            _ = interval.tick() => {
                tx.send(ClientToServerMsg::Heartbeat).await?;
                continue;
            }
            msg = rx.recv() => match msg {
                Some(Ok(result)) => match result {
                    ServerToClientMsg::ActionMappingUpdate(mapping) => {
                        output!("Received action mapping update: {:?}", mapping);
                        for (index, action) in mapping.into_iter().enumerate() {
                            action_mapping.insert(action, index as u8);
                        }
                    }
                    ServerToClientMsg::Error(e) => {
                        eprintln!("{e}");
                        break;
                    }
                },
                _ => break,
            },
            Some(Ok(Event::Key(event))) = keys.next() => {
                let action = match event.code {
                    KeyCode::Char('w') => action_mapping.get(&Action::MoveForward).unwrap(),
                    KeyCode::Char('s') => action_mapping.get(&Action::MoveBackward).unwrap(),
                    KeyCode::Char('a') => action_mapping.get(&Action::TurnLeft).unwrap(),
                    KeyCode::Char('d') => action_mapping.get(&Action::TurnRight).unwrap(),
                    KeyCode::Char('e') => action_mapping.get(&Action::Fire).unwrap(),
                    KeyCode::Char('q') => action_mapping.get(&Action::Shield).unwrap(),
                    _ => continue,
                };
                tx.send(ClientToServerMsg::PerformAction(*action)).await?;
            }
        }
    }
    Ok(())
}
