use std::ops::ControlFlow;

use crate::irc_core::{handler, irc_msg};

pub struct ExampleHandler;

#[async_trait::async_trait]
impl handler::Handler for ExampleHandler {
    async fn handle(&self, ctx: &handler::Context, msg: &irc_msg::Msg) -> ControlFlow<()> {
        if let irc_msg::Command::Privmsg {
                ref reply_to,
                ref message,
                ..
            } = msg.command
            && message.starts_with("!test") {
                let nick = msg.nick().unwrap_or("someone".into());
                let _ = ctx.client.privmsg(reply_to, &format!("hi {}", nick)).await;
            }

        ControlFlow::Continue(())
    }
}
