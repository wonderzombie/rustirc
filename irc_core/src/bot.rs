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
    pub async fn run(mut self) -> anyhow::Result<()> {
        // ...
        Ok(())
    }
}
