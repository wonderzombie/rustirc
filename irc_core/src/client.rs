use std::sync::Arc;
use tokio::sync::{Mutex, mpsc::{Receiver, Sender}};

use crate::irc_msg;

#[derive(Clone)]
pub struct BotClient {
    tx: Sender<String>,
    rx: Arc<Mutex<Receiver<irc_msg::Msg>>>,
}

impl BotClient {
    pub async fn send(&self, line: &str) -> anyhow::Result<()> {
        self.tx.send(format!("{}\r\n", line)).await?;
        Ok(())
    }

    pub async fn recv(&self) -> Option<crate::irc_msg::Msg> {
        let mut rx = self.rx.lock().await;
        rx.recv().await
    }
}
