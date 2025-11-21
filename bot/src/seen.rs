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
        let now = msg.meta.ts;

        if let irc_msg::Command::Privmsg {
            ref channel,
            ref message,
        } = msg.command
        {
            println!("SeenHandler received message: {}", message);
            if message.to_lowercase().starts_with("!seen") {
                let parts: Vec<&str> = message.split_whitespace().collect();
                if parts.len() >= 2 {
                    let target_nick = parts[1];
                    println!("Looking up seen time for {}", target_nick);

                    let seen_time_opt = {
                        let state = ctx.state.lock().await;
                        state.seen.get(target_nick).cloned()
                    };

                    let response = if let Some(seen_time) = seen_time_opt {
                        let seen_time = seen_time
                            .format("%Y-%m-%d %H:%M:%S %Z")
                            .to_string();
                        format!("{} was last seen at {:?}", target_nick, seen_time)
                    } else {
                        println!(
                            "No seen time found for {} in {:?}",
                            target_nick,
                            ctx.state.lock().await.seen
                        );
                        format!("I have not seen {}", target_nick)
                    };

                    let _ = ctx.client.privmsg(channel, &response).await;
                }
            }

            println!("Updating seen time for {}", msg.nick().unwrap_or_default());
            let mut state = ctx.state.lock().await;
            state.seen.insert(msg.nick().unwrap_or_default(), now);
        }

        ControlFlow::Continue(())
    }
}
