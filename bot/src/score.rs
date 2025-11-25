use std::{collections::HashMap, ops::ControlFlow};

use irc_core::handler::{Context, PrivmsgHandler};

pub struct ScoreHandler;

#[async_trait::async_trait]
impl PrivmsgHandler for ScoreHandler {
    async fn handle_privmsg(
        &self,
        ctx: &Context,
        _source: &str,
        channel: &str,
        message: &str,
    ) -> ControlFlow<()> {
        let in_channel = {
            let state = ctx.state.lock().await;
            state.channels.iter().any(|s| channel == s)
        };
        if !in_channel {
            return ControlFlow::Continue(());
        }

        let delta: Option<(&str, i32)> = parse_score_delta(message);

        if let Some((nick, d)) = delta {
            let mut state = ctx.state.lock().await;
            let scores = &mut state.scores;
            ScoreHandler::add_to_score(scores, nick, d);
            let _ = ctx.client.privmsg(channel, "{nick}'s score is now");
            return ControlFlow::Break(());
        }

        ControlFlow::Continue(())
    }
}

fn parse_score_delta(message: &str) -> Option<(&str, i32)> {
    let mut delta: Option<(&str, i32)> = None;

    for token in message.split_ascii_whitespace() {
        if let Some((nick, _)) = token.split_once("++")
            && nick != ""
        {
            delta = Some((nick, 1));
            break;
        } else if let Some((nick, _)) = token.split_once("--")
            && nick != ""
        {
            delta = Some((nick, -1));
            break;
        }
    }
    delta
}

impl ScoreHandler {
    pub fn add_to_score(scores: &mut HashMap<String, i32>, nick: &str, d: i32) -> i32 {
        return *scores
            .entry(nick.to_string())
            .and_modify(|it| *it += d)
            .or_insert_with(|| d);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_score_delta() {
        let delta = parse_score_delta("message++");
        assert_eq!(delta, Some(("message", 1)));
        let delta = parse_score_delta("message--");
        assert_eq!(delta, Some(("message", -1)));
    }

    #[test]
    fn test_parse_score_delta_invalid() {
        let delta = parse_score_delta("message+-");
        assert_eq!(delta, None);
        let delta = parse_score_delta("++");
        assert_eq!(delta, None);
        let delta = parse_score_delta("--message");
        assert_eq!(delta, None);
    }

    #[test]
    fn test_add_to_score() {
        let mut scores: HashMap<String, i32> = HashMap::new();
        scores.insert("botty".into(), 1);
        scores.insert("thumbkin".into(), -1);

        ScoreHandler::add_to_score(&mut scores, "botty", 1);
        let new_score = scores.get("botty").copied();
        assert_eq!(Some(2), new_score);

        ScoreHandler::add_to_score(&mut scores, "thumbkin", -1);
        let new_score = scores.get("thumbkin").copied();
        assert_eq!(Some(-2), new_score);

        ScoreHandler::add_to_score(&mut scores, "beelzebub", 1);
        let new_score = scores.get("beelzebub").copied();
        assert_eq!(Some(1), new_score);
    }
}
