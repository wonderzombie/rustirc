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
            irc_msg::Command::Join { ref channel } => {
                if let Some(nick) = msg.nick()
                    && nick != ctx.client.nick
                {
                    println!("=== {0} joined {1}", nick, channel)
                }
            }

            irc_msg::Command::Part { ref channel } => {
                if let Some(nick) = msg.nick() {
                    println!("=== {0} left {1}", nick, channel)
                }
            }

            irc_msg::Command::Numeric {
                code: 353,
                trailing: Some(ref new_names_list),
                ..
            } => {
                let mut new_names: Vec<String> = new_names_list
                    .split_ascii_whitespace()
                    .filter(|s| *s != ctx.client.nick)
                    .map(str::to_owned)
                    .collect();
                let mut state = ctx.state.lock().await;
                state.names.append(&mut new_names);
            }
            _ => (),
        }
        ControlFlow::Continue(())
    }
}
