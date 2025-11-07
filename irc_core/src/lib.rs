mod irc_msg;
use std::time::SystemTime;

use irc_msg::Msg;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

pub async fn start(server: &str, nick: &str, user: &str) -> anyhow::Result<()> {
    loop {
        match TcpStream::connect(server).await {
            Ok(stream) => {
                println!("=== connected!");
                if let Err(e) = run_irc(stream, nick, user).await {
                    println!("=== error {e:?}");
                }
            }
            Err(e) => println!("=== connection error {:?}", e),
        }

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

async fn run_irc(stream: TcpStream, nick: &str, user: &str) -> anyhow::Result<()> {
    println!("=== running IRC now");
    let (rx, mut tx) = stream.into_split();

    send(&mut tx, &format!("NICK {}", nick)).await?;
    send(&mut tx, &format!("USER {} 0 * :{}", user, user)).await?;

    println!("=== awaiting lines, et al");
    let mut lines = BufReader::new(rx).lines();
    while let Some(line) = lines.next_line().await? {
        let line = line.trim_end();

        if let Some(msg) = Msg::parse(line, SystemTime::now()) {
            println!("<<< {}", msg.meta.raw.trim_ascii_end());
        }
    }
    Ok(())
}

async fn send(tx: &mut tokio::net::tcp::OwnedWriteHalf, msg: &str) -> anyhow::Result<()> {
    let out = format!("{}\r\n", msg);
    println!(">>> {}", msg);
    tx.write_all(out.as_bytes()).await?;
    Ok(())
}
