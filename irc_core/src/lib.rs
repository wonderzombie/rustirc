pub mod bot;
pub mod client;
pub mod handler;
pub mod irc_msg;

use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;

use anyhow::Context as _;

use crate::client::BotClient;

pub async fn connect(
    server: impl Into<String>,
    nick: impl Into<String>,
    user: impl Into<String>,
) -> anyhow::Result<BotClient> {
    let server = server.into();
    let nick = nick.into();
    let user = user.into();

    let stream = TcpStream::connect(&server)
        .await
        .with_context(|| format!("failed to connect to server {server}"))?;

    let (read_half, mut write_half) = stream.into_split();

    // Channels between socket tasks and BotClient.
    // Outgoing: app → socket
    let (outgoing_tx, mut outgoing_rx) = tokio::sync::mpsc::channel::<String>(100);
    // Incoming: socket → app
    let (incoming_tx, incoming_rx) = tokio::sync::mpsc::channel::<String>(100);

    let nick_ = nick.clone();

    // Writer task: drains outgoing_rx and writes to the TCP socket.
    tokio::spawn(async move {
        // IRC registration first.
        // `BotClient::send` already appends CRLF; here we write raw lines.
        // Registration
        if let Err(e) = write_half.write_all(format!("NICK {nick_}\r\n").as_bytes()).await {
            eprintln!("failed to write NICK: {e:?}");
            return;
        }
        if let Err(e) = write_half.write_all(format!("USER {user} 0 * :{user}\r\n").as_bytes()).await {
            eprintln!("failed to write USER: {e:?}");
            return;
        }

        while let Some(mut line) = outgoing_rx.recv().await {
            if !line.ends_with("\r\n") {
                line.push_str("\r\n");
            }
            if let Err(e) = write_half.write_all(line.as_bytes()).await {
                eprintln!("writer task error: {e:?}");
                break;
            }
            println!("==> {}", line.trim_end());
        }
    });

    // Reader task: reads lines from the TCP socket and forwards to incoming_tx.
    tokio::spawn(async move {
        let mut lines = BufReader::new(read_half).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            println!("<== {}", line.trim_end());
            if incoming_tx.send(line).await.is_err() {
                break; // receiver dropped; end the task
            }
        }
        // dropping incoming_tx closes the channel
    });

    let client = BotClient {
        tx: outgoing_tx,
        rx: Arc::new(tokio::sync::Mutex::new(incoming_rx)),
        nick: nick.clone(),
    };

    Ok(client)
}
