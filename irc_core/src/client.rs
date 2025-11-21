use anyhow::Context;
use std::sync::Arc;
use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

use crate::irc_msg::Msg;

#[derive(Clone)]
pub struct BotClient {
    pub(crate) tx: Sender<String>,
    pub(crate) rx: Arc<Mutex<Receiver<String>>>,

    pub nick: String,
}

impl BotClient {
    pub async fn send(&self, line: &str) -> anyhow::Result<()> {
        self.tx.send(format!("{}\r\n", line)).await?;
        Ok(())
    }

    pub async fn recv(&self) -> anyhow::Result<Option<Msg>> {
        let mut rx = self.rx.lock().await;

        match rx.recv().await {
            Some(line) => {
                let msg = Msg::parse(&line, chrono::Local::now())
                    .with_context(|| format!("failed to parse IRC line: {}", line))?;
                anyhow::Ok(Some(msg))
            }
            None => Ok(None),
        }
    }

    pub async fn privmsg(&self, target: &str, msg: &str) -> anyhow::Result<()> {
        let line = format!("PRIVMSG {} :{}", target, msg);
        self.send(&line).await
    }

    pub async fn join(&self, channel: &str) -> anyhow::Result<()> {
        let line = format!("JOIN {}", channel);
        self.send(&line).await
    }

    pub async fn pong(&self, token: Option<&str>) -> anyhow::Result<()> {
        let line = match token {
            Some(t) => format!("PONG :{}", t),
            None => "PONG".to_string(),
        };
        self.send(&line).await
    }

    pub async fn names(&self, channel: &str) -> anyhow::Result<()> {
        let line = format!("NAMES {}", channel);
        self.send(&line).await
    }
}
