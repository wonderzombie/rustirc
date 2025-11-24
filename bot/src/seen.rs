use chrono_humanize::HumanTime;
use std::ops::ControlFlow;

use irc_core::
    handler::{self, PrivmsgHandler}
;

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
        if message.to_lowercase().starts_with("!seen") {
            let parts: Vec<&str> = message.split_whitespace().collect();
            if parts.len() >= 2 {
                let target_nick = parts[1];

                let response = {
                    let state = ctx.state.lock().await;
                    format_seen_response(&state, target_nick)
                };

                let _ = ctx.client.privmsg(channel, &response).await;
            }
        }

        if !source.is_empty() {
            let now = chrono::Local::now();
            let mut state = ctx.state.lock().await;
            handler::State::update_seen(&mut state.seen, source, message, now);
        }

        ControlFlow::Continue(())
    }
}

fn format_seen_response(state: &handler::State, target_nick: &str) -> String {
    if let Some(info) = state.seen.get(target_nick) {
        let human_time = HumanTime::from(info.last_seen);
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
