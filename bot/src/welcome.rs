use std::ops::ControlFlow;

use tracing::info;

use crate::irc_core::{handler, irc_msg};

pub struct WelcomeHandler;

#[async_trait::async_trait]
impl handler::Handler for WelcomeHandler {
    async fn handle(&self, ctx: &handler::Context, msg: &irc_msg::Msg) -> ControlFlow<()> {
        match msg.command {
            irc_msg::Command::Numeric { code, .. } => {
                // End of MOTD, join a channel
                match code {
                    376 | 422 => {
                        let state = ctx.state.lock().await;
                        for channel in &state.channels {
                            let _ = ctx.client.join(channel).await;
                            info!("Joined channel {}", channel);
                        }
                        return ControlFlow::Break(());
                    }
                    _ => return ControlFlow::Continue(()),
                }
            }
            _ => {}
        }

        ControlFlow::Continue(())
    }
}
