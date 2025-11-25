use std::ops::ControlFlow;

use crate::irc_core::{handler, irc_msg};

pub struct PingHandler;

#[async_trait::async_trait]
impl handler::Handler for PingHandler {
    async fn handle(&self, ctx: &handler::Context, msg: &irc_msg::Msg) -> ControlFlow<()> {
        if let irc_msg::Command::Ping { ref token } = msg.command {
            let _ = ctx.client.pong(token.as_deref()).await;
            return ControlFlow::Break(());
        }

        ControlFlow::Continue(())
    }
}
