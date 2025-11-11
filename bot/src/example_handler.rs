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

            irc_msg::Command::Numeric { code, .. } => {
                // End of MOTD, join a channel
                match code {
                    376 | 422 => {
                        println!("=== Joined server");
                        let _ = ctx.client.join("#el_rb_test376").await;
                        return ControlFlow::Break(());
                    }
                    _ => return ControlFlow::Continue(()),
                }
            }

            irc_msg::Command::Privmsg {
                ref channel,
                ref message,
                ..
            } => {
                println!("=== PRIVMSG {} :{}", channel, message);

                if message == "!hello" {
                    let _ = ctx.client.privmsg(channel, "Hello!").await;
                }
            }

            _ => {}
        }

        ControlFlow::Continue(())
    }
}
