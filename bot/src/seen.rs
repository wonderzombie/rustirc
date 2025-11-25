use chrono_humanize;
use std::{collections::HashMap, ops::ControlFlow};

use irc_core::handler::{self, PrivmsgHandler};

pub struct SeenHandler;

#[async_trait::async_trait]
impl PrivmsgHandler for SeenHandler {
    async fn handle_privmsg(
        &self,
        ctx: &handler::Context,
        source: &str,
        channel: &str,
        message: &str,
    ) -> ControlFlow<()> {
        let in_channel = {
            let state = ctx.state.lock().await;
            state.channels.iter().any(|c| channel == c)
        };
        if !in_channel {
            return ControlFlow::Continue(());
        }

        let query = format!("{},", ctx.client.nick);
        if message.to_ascii_lowercase().starts_with(&query) {
            let parts: Vec<&str> = message.split_whitespace().collect();
            if parts.len() >= 3 && parts[1].to_lowercase() == "seen" {
                let target_nick = parts[2];

                let response = {
                    let state = ctx.state.lock().await;
                    format_seen_response(&state, target_nick)
                };

                let _ = ctx.client.privmsg(channel, &response).await;
            }
            return ControlFlow::Continue(());
        }

        if !source.is_empty() {
            let now = chrono::Local::now();
            let mut state = ctx.state.lock().await;
            update_seen(&mut state.seen, source, message, now);
        }

        ControlFlow::Continue(())
    }
}

fn format_seen_response(state: &handler::State, target_nick: &str) -> String {
    if let Some(info) = state.seen.get(target_nick) {
        let human_time = chrono_humanize::HumanTime::from(info.last_seen);
        format!(
            "{} was last seen {} saying: {}",
            target_nick,
            human_time.to_string(),
            info.message,
        )
    } else {
        format!("I have not seen {}", target_nick)
    }
}

fn update_seen(
    seen: &mut HashMap<String, handler::SeenInfo>,
    nick: &str,
    message: &str,
    now: chrono::DateTime<chrono::Local>,
) {
    seen.entry(nick.to_string())
        .and_modify(|info| {
            info.last_seen = now;
            info.message = message.to_string();
        })
        .or_insert_with(|| handler::SeenInfo {
            nick: nick.to_string(),
            last_seen: now,
            message: message.to_string(),
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use irc_core::handler::SeenInfo;
    use std::collections::HashMap;

    #[test]
    fn seen_response_when_user_not_seen() {
        let state = handler::State {
            seen: HashMap::new(),
            ..Default::default()
        };

        let resp = format_seen_response(&state, "alice");
        assert_eq!(resp, "I have not seen alice");
    }

    #[test]
    fn seen_response_when_user_seen() {
        let mut state = handler::State::default();
        state.seen.insert(
            "alice".to_string(),
            SeenInfo {
                nick: "alice".to_string(),
                last_seen: chrono::Local::now(),
                message: "hello world".to_string(),
            },
        );

        let resp = format_seen_response(&state, "alice");
        assert!(
            resp.contains("alice was last seen"),
            "response was: {resp:?}"
        );
        assert!(
            resp.contains("saying: hello world"),
            "response was: {resp:?}"
        );
    }
}
