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
                let msg = Msg::parse(&line, std::time::SystemTime::now())
                    .with_context(|| format!("failed to parse IRC line: {}", line))?;
                anyhow::Ok(Some(msg))
            }
            None => Ok(None),
        }
    }
    }
}
