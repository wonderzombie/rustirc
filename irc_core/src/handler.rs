use chrono::{DateTime, Local};
use std::{collections::HashMap, ops::ControlFlow, sync::Arc};

use tokio::sync::Mutex;

use crate::{client::BotClient, irc_msg};

/// Information about when a user was last seen and what they said.
#[derive(Default, Clone)]
pub struct SeenInfo {
    pub nick: String,
    pub last_seen: DateTime<Local>,
    pub message: String,
}

/// Shared mutable state for modules.
#[derive(Default)]
pub struct State {
    pub seen: HashMap<String, SeenInfo>,
    pub scores: HashMap<String, i32>,
    pub channels: Vec<String>,
    pub names: Vec<String>,
}

impl State {
    pub fn update_seen(
        seen: &mut HashMap<String, SeenInfo>,
        nick: &str,
        message: &str,
        now: chrono::DateTime<chrono::Local>,
    ) {
        seen.entry(nick.to_string())
            .and_modify(|info| {
                info.last_seen = now;
                info.message = message.to_string();
            })
            .or_insert_with(|| SeenInfo {
                nick: nick.to_string(),
                last_seen: now,
                message: message.to_string(),
            });
    }
}

/// Read/write context passed to handlers.
pub struct Context {
    pub client: BotClient,
    pub state: Arc<Mutex<State>>,
}

#[async_trait::async_trait]
pub trait Handler: Send + Sync {
    /// Return ControlFlow::Break(()) to stop processing further handlers.
    async fn handle(&self, ctx: &Context, msg: &irc_msg::Msg) -> ControlFlow<()>;
}

pub struct HandlerFn<F>(pub F);
#[async_trait::async_trait]
impl<F, Fut> Handler for HandlerFn<F>
where
    F: Send + Sync + Fn(&Context, &irc_msg::Msg) -> Fut,
    Fut: std::future::Future<Output = ControlFlow<()>> + Send,
{
    async fn handle(&self, ctx: &Context, msg: &irc_msg::Msg) -> ControlFlow<()> {
        (self.0)(ctx, msg).await
    }
}

#[async_trait::async_trait]
pub trait PrivmsgHandler: Send + Sync {
    async fn handle_privmsg(
        &self,
        ctx: &Context,
        source: &str,
        channel: &str,
        message: &str,
    ) -> ControlFlow<()>;
}

#[async_trait::async_trait]
impl<T> Handler for T
where
    T: PrivmsgHandler + Send + Sync,
{
    async fn handle(&self, ctx: &Context, msg: &irc_msg::Msg) -> ControlFlow<()> {
        if let irc_msg::Command::Privmsg {
            ref channel,
            ref message,
        } = msg.command
            && let Some(ref source) = msg.source
        {
            self.handle_privmsg(ctx, source, channel, message).await
        } else {
            ControlFlow::Continue(())
        }
    }
}
