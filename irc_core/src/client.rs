use anyhow::Context;
use std::{borrow::Cow, sync::Arc};
use tokio::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

use crate::irc_msg::Msg;

#[derive(Clone)]
pub struct Client {
    pub(crate) tx: Sender<String>,
    pub(crate) rx: Arc<Mutex<Receiver<String>>>,

    pub nick: String,
}

impl Client {
    pub async fn send<'a>(&self, line: impl Into<Cow<'a, str>>) -> anyhow::Result<()> {
        let owned = line.into().to_owned();
        self.tx.send(format!("{}\r\n", owned)).await?;
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
