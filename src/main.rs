use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

mod tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.command {
        Commands::Server { ip } => server(ip).await?,
        Commands::Client { connection, user } => client(connection, user).await?,
    }
    Ok(())
}

async fn server(ip: String) -> Result<(), Box<dyn std::error::Error>> {
    let (broadcast, _) = tokio::sync::broadcast::channel::<Message>(u8::MAX as usize);
    let listener = TcpListener::bind(&ip).await?;
    println!("Listening on {ip}");

    loop {
        let (socket, addr) = listener.accept().await?;
        let (mut socket_read, mut socket_write) = socket.into_split();
        println!("Connected: {addr}");

        let broadcast = broadcast.clone();

        let broadcast_r = broadcast.clone();
        // Read thread
        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                let n = match socket_read.read(&mut buf).await {
                    // socket closed
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {e}");
                        return;
                    }
                };

                let Ok(message) = serde_json::from_slice::<Message>(&buf[..n]) else {
                    eprintln!("Cannot deserialize message");
                    continue;
                };

                println!("Sending message from {addr}: {}", message.message);

                broadcast_r.send(message).unwrap();
            }
        });

        tokio::spawn(async move {
            let mut broadcast = broadcast.subscribe();
            loop {
                let Ok(msg) = broadcast.recv().await else {
                    continue;
                };

                let Ok(msg) = serde_json::to_vec(&msg) else {
                    eprintln!("Cannot deserialize message");
                    continue;
                };

                _ = socket_write.write(&msg).await;
            }
        });
    }
}

async fn client(ip: String, user: String) -> Result<(), Box<dyn std::error::Error>> {
    let (message_sender, message_receiver) = mpsc::channel::<Vec<u8>>(u8::MAX as usize);
    let (input_sender, mut input_receiver) = mpsc::channel::<Vec<u8>>(u8::MAX as usize);
    let (mut reader, mut writer) = TcpStream::connect(ip).await?.into_split();

    let app = App::new(message_receiver, input_sender);

    // Read thread
    tokio::spawn(async move {
        let mut buf = [0; 1024];
        loop {
            let n = match reader.read(&mut buf).await {
                Ok(0) => return,
                Ok(n) => n,
                Err(e) => {
                    eprintln!("failed to read from socket; err = {e}");
                    return;
                }
            };

            if (message_sender.send(buf[0..n].to_vec()).await).is_err() {
                eprintln!("Error sending message");
                return;
            }
        }
    });

    // Write thread
    tokio::spawn(async move {
        loop {
            // Input here
            let Some(msg) = input_receiver.recv().await else {
                continue;
            };

            let msg = Message::new(user.clone(), unsafe { String::from_utf8_unchecked(msg) });
            let Ok(msg) = serde_json::to_vec(&msg) else {
                continue;
            };

            if let Err(e) = writer.write(&msg).await {
                eprintln!("Error: {e}");
                return;
            }
        }
    });

    run_tui(app).unwrap();
    Ok(())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    user: String,
    message: String,
}

impl Message {
    fn new(user: String, msg: String) -> Self {
        Self { user, message: msg }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}] {}", self.user, self.message))
    }
}

use clap::{Parser, Subcommand};

use crate::tui::{App, run_tui};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Server {
        #[arg(index = 1)]
        ip: String,
    },
    Client {
        #[arg(long, short, default_value = "unknown")]
        user: String,
        #[arg(index = 1)]
        connection: String,
    },
}
