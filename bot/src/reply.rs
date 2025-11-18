use std::ops::ControlFlow;

use crate::irc_core::{handler, irc_msg};

pub struct ReplyHandler;

#[async_trait::async_trait]
impl handler::Handler for ReplyHandler {
    async fn handle(&self, ctx: &handler::Context, msg: &irc_msg::Msg) -> ControlFlow<()> {
        match msg.command {
            irc_msg::Command::Privmsg {
                ref channel,
                ref message,
                ..
            } => {
                if message
                    .to_ascii_lowercase()
                    .contains(ctx.client.nick.to_lowercase().as_str())
                {
                    let reply = format!("where is {0}, where is {0}", ctx.client.nick);
                    let _ = ctx.client.privmsg(channel, &reply).await;
                }
            }
            _ => {}
        }

        ControlFlow::Continue(())
    }
}
