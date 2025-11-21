use chrono_humanize::HumanTime;
use std::ops::ControlFlow;

use irc_core::{
    self,
    handler::{self, Handler},
    irc_msg,
};

pub struct SeenHandler;

#[async_trait::async_trait]
impl Handler for SeenHandler {
    async fn handle(&self, ctx: &handler::Context, msg: &irc_msg::Msg) -> ControlFlow<()> {

        if let irc_msg::Command::Privmsg {
            ref channel,
            ref message,
        } = msg.command
        {
            if message.to_lowercase().starts_with("!seen") {
                let parts: Vec<&str> = message.split_whitespace().collect();
                if parts.len() >= 2 {
                    let target_nick = parts[1];

                    let seen_time_opt = {
                        let state = ctx.state.lock().await;
                        state.seen.get(target_nick).cloned()
                    };

                    let response = if let Some(seen_time) = seen_time_opt {
                        let human_time = HumanTime::from(seen_time);
                        format!("{} was last seen {}", target_nick, human_time.to_string())
                    } else {
                        format!("I have not seen {}", target_nick)
                    };

                    let _ = ctx.client.privmsg(channel, &response).await;
                }
            }

            println!("Updating seen time for {}", msg.nick().unwrap_or_default());
            let mut state = ctx.state.lock().await;
            state.seen.insert(msg.nick().unwrap_or_default(), msg.meta.ts);
        }

        ControlFlow::Continue(())
    }
}
