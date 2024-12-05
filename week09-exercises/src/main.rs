//! implement a TCP/IP client that will connect to a server and figure out a password to
//! open a VAULT.
//!
//! If you can guess the password to the vault by the end of the seminar, you'll be awarded 2 points
//! for Gryffindor.
//!
//! # Communication protocol
//! After you connect to the server on a given TCP/IP v4 address and port, the following should
//! happen:
//! 1) You send a string that identifies you. You have to choose a nickname.
//!
//! - You have to send the message within two seconds. If you don't, the server will disconnect you.
//! - If the nickname is not unique (someone else has the same nickname), you will be disconnected.
//! - The nickname cannot be longer than `15` (UTF-8) bytes.
//!
//! 2) The following communication happens in a lockstep. You send a string that contains your guess
//! of the password. The server then responds either with:
//! - "correct" if you have guessed the password correctly
//! - "incorrect" if your password guess was wrong
//! - a string containing an error if some error has happened
//!
//! # Spam protection
//! - You must not send a message more often than once every 0.1 milliseconds. If
//!   you do, you will receive a strike. After accumulating three strikes, you will be disconnected.
//! - You must not make more than 10 thousand password guesses. After 10k attempts, you will be
//!   disconnected.
//!
//! # Inactivity protection
//! You have to send a message at least once every five seconds, otherwise you will be disconnected.
//!
//! # Message encoding
//! The encoding is similar to last week, although this time, each message is a simple UTF-8 string,
//! there is no JSON involved. You can use the provided `MessageReader` and `MessageWriter` structs
//! to communicate with the server.
//!
//! Bonus point if you can crash the server :)

use std::net::{Shutdown, TcpStream};
use std::process::exit;
use std::time::Instant;

mod reader;
mod writer;

fn main() {
    let capacity = 9;
    let address = "";
    let Ok(stream) = TcpStream::connect(address) else {
        println!("Could not connect to the server");
        return;
    };
    println!("Connected to the server");
    let mut reader = reader::MessageReader::new(stream.try_clone().unwrap());
    let mut writer = writer::MessageWriter::new(stream.try_clone().unwrap());
    // Send nickname
    writer.write("nickname").unwrap_or_default();
    let mut password = String::with_capacity(capacity);

    'outer: for _ in 0..=capacity {
        let mut best = None;
        let mut best_time = 0;

        for char in ('a'..='z').chain('A'..='Z') {
            //std::thread::sleep(std::time::Duration::from_millis(1));
            password.push(char);
            let start = Instant::now();
            writer.write(password.as_str()).unwrap_or_default();
            let Some(Ok(answer)) = reader.read() else {
                eprintln!("Could not read the answer");
                exit(1);
            };
            let elapsed = start.elapsed().as_micros();
            println!("Password: {password}");
            println!("Answer: {answer}");
            println!("Time: {elapsed}");
            if answer == "correct" {
                break 'outer;
            }
            password.pop();
            if elapsed > best_time {
                best = Some(char);
                best_time = elapsed;
            }
        }
        password.push(best.unwrap());
    }

    stream.shutdown(Shutdown::Both).unwrap_or_default()
}
