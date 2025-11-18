use std::{collections::HashMap, ops::ControlFlow, sync::Arc, time::SystemTime};

use tokio::sync::Mutex;

use crate::{client::BotClient, irc_msg::Msg};

/// Shared mutable state for modules.
#[derive(Default)]
pub struct State {
    pub seen: HashMap<String, SystemTime>,
    pub channels: Vec<String>,
}

/// Read/write context passed to handlers.
pub struct Context {
    pub client: BotClient,
    pub state: Arc<Mutex<State>>,
}

#[async_trait::async_trait]
pub trait Handler: Send + Sync {
    /// Return ControlFlow::Break(()) to stop processing further handlers.
    async fn handle(&self, ctx: &Context, msg: &Msg) -> ControlFlow<()>;
}

pub struct HandlerFn<F>(pub F);
#[async_trait::async_trait]
impl<F, Fut> Handler for HandlerFn<F>
where
    F: Send + Sync + Fn(&Context, &Msg) -> Fut,
    Fut: std::future::Future<Output = ControlFlow<()>> + Send,
{
    async fn handle(&self, ctx: &Context, msg: &Msg) -> ControlFlow<()> {
        (self.0)(ctx, msg).await
    }
}
