use std::ops::ControlFlow;

use crate::irc_core::{handler, irc_msg};


pub struct ExampleHandler;

#[async_trait::async_trait]
impl handler::Handler for ExampleHandler {
    async fn handle(
        &self,
        ctx: &handler::Context,
        msg: &irc_msg::Msg,
    ) -> ControlFlow<()> {
        use crate::irc_core::irc_msg::Command;

        if let Command::Privmsg { channel, message, .. } = &msg.command {
            if message == "!hello" {
                let reply = format!("Hello! You said: {}", message);
                if let Err(e) = ctx.client.privmsg(channel, &reply).await {
                    eprintln!("Failed to send PRIVMSG: {:?}", e);
                }
                return ControlFlow::Break(());
            }
        }

        ControlFlow::Continue(())
    }
}
