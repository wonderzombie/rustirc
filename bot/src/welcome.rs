use std::ops::ControlFlow;


use tracing::info;

use crate::irc_core::{handler, irc_msg};

pub struct WelcomeHandler;

#[async_trait::async_trait]
impl handler::Handler for WelcomeHandler {
    async fn handle(&self, ctx: &handler::Context, msg: &irc_msg::Msg) -> ControlFlow<()> {
        if let irc_msg::Command::Numeric { code, .. } = msg.command {
            // End of MOTD, join a channel
            match code {
                376 | 422 => {
                    let channels = ctx.with_state(|state| state.channels.clone()).await;
                    for channel in channels {
                        let _ = ctx.client.join(&channel).await;
                        info!("Joined channel {}", channel);
                    }

                    return ControlFlow::Break(());
                }
                _ => return ControlFlow::Continue(()),
            }
        }

        ControlFlow::Continue(())
    }
}
