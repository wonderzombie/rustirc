use std::ops::ControlFlow;

use crate::irc_core::{handler, irc_msg};

pub struct ReplyHandler;

#[async_trait::async_trait]
impl handler::Handler for ReplyHandler {
    async fn handle(&self, ctx: &handler::Context, msg: &irc_msg::Msg) -> ControlFlow<()> {
        if let irc_msg::Command::Privmsg {
            ref reply_to,
            ref message,
            ..
        } = msg.command
            && message
                .to_ascii_lowercase()
                .contains(ctx.client.nick.to_lowercase().as_str())
        {
            let reply = format!("where is {0}, where is {0}", ctx.client.nick);
            let _ = ctx.client.privmsg(reply_to, &reply).await;
        }

        ControlFlow::Continue(())
    }
}
