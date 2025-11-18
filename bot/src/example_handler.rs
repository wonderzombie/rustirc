use std::ops::ControlFlow;

use crate::irc_core::{handler, irc_msg};

pub struct ExampleHandler;

#[async_trait::async_trait]
impl handler::Handler for ExampleHandler {
    async fn handle(&self, ctx: &handler::Context, msg: &irc_msg::Msg) -> ControlFlow<()> {
        match msg.command {
            irc_msg::Command::Ping { ref token } => {
                let _ = ctx.client.pong(token.as_deref()).await;
                return ControlFlow::Break(());
            }

            irc_msg::Command::Privmsg {
                ref channel,
                ref message,
                ..
            } => {
                if message.starts_with("!test") {
                    let nick = msg.nick().unwrap_or("someone".into());
                    let _ = ctx.client.privmsg(channel, &format!("hi {}", nick)).await;
                }
            }

            _ => {}
        }

        ControlFlow::Continue(())
    }
}
