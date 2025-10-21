use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use crate::structs::Message;

pub async fn server(ip: String) -> Result<(), Box<dyn std::error::Error>> {
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
