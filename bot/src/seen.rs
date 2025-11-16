use std::ops::ControlFlow;

use irc_core::{self, handler::{self, Handler}, irc_msg};
use crate::botty;

pub struct SeenHandler;

impl Handler for SeenHandler {
    async fn handle(&self, ctx: &handler::Context, msg: &irc_msg::Msg) -> ControlFlow<()> {
        match msg.command {
            irc_msg::Command::Privmsg { .. } => {
                if let Some(source) = &msg.source {
                    let nick = irc_msg::Msg::source_to_nick(Some(source));
                    let now = msg.meta.ts;

                    let mut state = ctx.state.lock().await;
                    state.seen.insert(nick, now);
                }
            }
            _ => {}
        }

        ControlFlow::Continue(())
    }
}
