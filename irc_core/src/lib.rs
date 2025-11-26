pub mod bot;
pub mod client;
pub mod handler;
pub mod irc_msg;

use std::borrow::Cow;
use std::fmt::Debug;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpStream;

use anyhow::Context as _;
use tracing::info;

use crate::client::Client;

pub async fn connect<S, N, U>(server: S, nick: N, user: U) -> anyhow::Result<Client>
where
    S: Send + Debug + Into<Cow<'static, str>>,
    N: Send + Debug + Into<Cow<'static, str>>,
    U: Send + Debug + Into<Cow<'static, str>>,
{
    let server: Cow<'static, str> = server.into();
    let nick: Cow<'static, str> = nick.into();
    let user: Cow<'static, str> = user.into();

    info!("Connecting to IRC server {} as {}", server.as_ref(), nick.as_ref());

    let stream = TcpStream::connect(server.as_ref())
        .await
        .with_context(|| format!("failed to connect to server {}", server.as_ref()))?;

    let (read_half, mut write_half) = stream.into_split();

    // Channels between socket tasks and BotClient.
    // Outgoing: app → socket
    let (outgoing_tx, mut outgoing_rx) = tokio::sync::mpsc::channel::<String>(100);
    // Incoming: socket → app
    let (incoming_tx, incoming_rx) = tokio::sync::mpsc::channel::<String>(100);

    let nick_ = nick.as_ref().to_string();

    // Writer task: drains outgoing_rx and writes to the TCP socket.
    tokio::spawn(async move {
        // IRC registration first.
        // `BotClient::send` already appends CRLF; here we write raw lines.
        // Registration
        if let Err(e) = write_half
            .write_all(format!("NICK {}\r\n", nick_).as_bytes())
            .await
        {
            error!("failed to write NICK: {e:?}");
            return;
        }
        if let Err(e) = write_half
            .write_all(format!("USER {user} 0 * :{user}\r\n").as_bytes())
            .await
        {
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

    let client = Client {
        tx: outgoing_tx,
        rx: Arc::new(tokio::sync::Mutex::new(incoming_rx)),
        nick: nick.into_owned(),
    };

    Ok(client)
}
