use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
};

use crate::{
    structs::Message,
    tui::{App, run_tui},
};

pub async fn client(ip: String, user: String) -> Result<(), Box<dyn std::error::Error>> {
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
