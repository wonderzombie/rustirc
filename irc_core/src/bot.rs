use crate::client::BotClient;
use crate::handler::{Handler, State};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Bot {
    handlers: Vec<Box<dyn Handler>>,
    state: Arc<Mutex<State>>,
    client: BotClient,
}

pub struct BotBuilder {
    handlers: Vec<Box<dyn Handler>>,
}

impl BotBuilder {
    pub fn new() -> Self {
        Self { handlers: vec![] }
    }

    pub fn with_handler<H: Handler + 'static>(mut self, h: H) -> Self {
        self.handlers.push(Box::new(h));
        self
    }

    pub fn build(self, client: BotClient) -> Bot {
        Bot {
            handlers: self.handlers,
            state: Arc::new(Mutex::new(State::default())),
            client,
        }
    }
}

impl Bot {
    pub async fn run(self) -> anyhow::Result<()> {
        let ctx = crate::handler::Context {
            client: self.client.clone(),
            state: self.state.clone(),
        };

        while let Some(msg) = self.client.recv().await? {
            for h in &self.handlers {
                use std::ops::ControlFlow;
                let flow = h.handle(&ctx, &msg).await;
                if matches!(flow, ControlFlow::Break(())) {
                    break;
                }
            }
        }

        Ok(())
    }
}
