use std::ops::ControlFlow;

use crate::irc_core::{
    handler::{self, Handler},
    irc_msg::{self},
};

pub struct NamesHandler;

#[async_trait::async_trait]
impl Handler for NamesHandler {
    async fn handle(&self, ctx: &handler::Context, msg: &irc_msg::Msg) -> ControlFlow<()> {
        match msg.command {
            irc_msg::Command::Join { ref channel, .. } => {
                if let Some(nick) = msg.nick()
                    && nick != ctx.client.nick
                {
                    println!("=== {0} joined {1}", nick, channel);
                    ctx.with_state(|state| {
                        state.names.push(nick.to_string());
                    })
                    .await;
                }
            }

            irc_msg::Command::Part { ref channel, .. } => {
                if let Some(nick) = msg.nick() {
                    println!("=== {0} left {1}", nick, channel);
                    ctx.with_state(|state| {
                        state.names.retain(|n| n != &nick);
                    })
                    .await;
                }
            }

            irc_msg::Command::Numeric {
                code: 353,
                trailing: Some(ref new_names_list),
                ..
            } => {
                let new_names = &mut new_names_list
                    .split_ascii_whitespace()
                    .filter(|s| *s != ctx.client.nick)
                    .map(str::to_owned)
                    .collect();
                ctx.with_state(|state| {
                    state.names.append(new_names);
                })
                .await;
            }
            _ => (),
        }
        ControlFlow::Continue(())
    }
}
