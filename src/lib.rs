mod irc_msg;
use std::time::SystemTime;

use irc_msg::Msg;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

pub async fn start() -> anyhow::Result<()> {
    loop {
        match TcpStream::connect("irc.libera.chat:6667").await {
            Ok(stream) => {
                println!("=== connected!");
                if let Err(e) = run_irc(stream, "rusty_bot", "rustling 0 * :Rusty Bot").await {
                    println!("=== error {e:?}");
                }
            }
            Err(e) => println!("=== connection error {:?}", e),
        }

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

async fn run_irc(stream: TcpStream, nick: &str, user: &str) -> anyhow::Result<()> {
    let (rx, mut tx) = stream.into_split();
    let mut lines = BufReader::new(rx).lines();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim_end();

        if let Some(msg) = Msg::parse(line, SystemTime::now()) {
            println!("{:#?}", msg);

        }
    }
    Ok(())
}
