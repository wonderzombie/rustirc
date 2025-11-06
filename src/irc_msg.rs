use std::{borrow::Cow, time::SystemTime};

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Ping {
        token: Option<Cow<'a, str>>,
    },
    Join {
        nick: Cow<'a, str>,
        channel: Cow<'a, str>,
    },
    Privmsg {
        nick: Cow<'a, str>,
        channel: Cow<'a, str>,
        message: Cow<'a, str>,
    },
    // Part / Numeric / Unknown can be added later with Cow<'a, str> as well
}

impl<'a> Command<'a> {
    fn from_parts(nick: Cow<'a, str>, parts: &CmdParts<'a>, trailing: &'a str) -> Option<Command<'a>> {
        match parts.command {
            "PING" => {
                // Prefer trailing token if present, else first arg; allow None
                let token = if !trailing.is_empty() {
                    Some(Cow::from(trailing))
                } else {
                    parts.args.first().map(|s| Cow::from(*s))
                };
                Some(Command::Ping { token })
            }
            "PRIVMSG" => {
                let channel = *parts.args.first()?;
                Some(Command::Privmsg {
                    nick,
                    channel: Cow::from(channel),
                    message: Cow::from(trailing),
                })
            }
            "JOIN" => {
                let channel = *parts.args.first()?;
                Some(Command::Join {
                    nick,
                    channel: Cow::from(channel),
                })
            }
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MsgMeta {
    pub raw: String,
    pub ts: SystemTime,
}

#[derive(Debug, PartialEq)]
pub struct Msg<'a> {
    pub meta: MsgMeta,
    pub source: Option<Cow<'a, str>>, // entire prefix if present
    pub command: Command<'a>,
}

#[derive(Debug)]
struct CmdParts<'a> {
    source: Option<&'a str>,
    command: &'a str,
    args: Vec<&'a str>,
}

impl<'a> Msg<'a> {
    pub fn parse(line: &'a str, now: SystemTime) -> Option<Msg<'a>> {
        let meta = MsgMeta {
            raw: line.to_string(),
            ts: now,
        };

        let (before, trailing) = split_irc(line)?;
        let cmd_parts = Self::parse_command_tokens(before)?;

        let nick = Self::source_to_nick(cmd_parts.source);
        let source = cmd_parts.source.map(Cow::from);

        let command = Command::from_parts(nick, &cmd_parts, trailing)?;
        Some(Msg { meta, source, command })
    }

    fn source_to_nick<'b>(source: Option<&'b str>) -> Cow<'b, str> {
        match source.and_then(|s| s.split('!').next()) {
            Some(n) => Cow::from(n),
            None => Cow::from(""),
        }
    }

    fn parse_command_tokens<'b>(before: &'b str) -> Option<CmdParts<'b>> {
        let mut it = before.split_ascii_whitespace();
        let first = it.next()?;
        let (source, command) = if first.starts_with(':') {
            (Some(first.trim_start_matches(':')), it.next()?)
        } else {
            (None, first)
        };
        let args = it.collect();
        Some(CmdParts { source, command, args })
    }
}

fn split_irc(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.splitn(2, " :");
    let before = parts.next()?;
    let trailing = parts.next().unwrap_or_default();
    Some((before, trailing))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn from_parts_privmsg() {
        let got = Command::from_parts(
            Cow::from("nickname"),
            &CmdParts {
                source: Some("nickname!+username@host"),
                command: "PRIVMSG",
                args: vec!["#channel"],
            },
            "chat chat chat",
        )
        .unwrap();

        assert_eq!(
            Command::Privmsg {
                nick: "nickname".into(),
                channel: "#channel".into(),
                message: "chat chat chat".into(),
            },
            got
        );
    }

    #[test]
    fn parse_privmsg() {
        let raw = ":nick!username@host PRIVMSG #channel :chat chat chat";
        let now = SystemTime::now();
        let got = Msg::parse(raw, now).unwrap();

        assert_eq!(
            Msg {
                meta: MsgMeta { raw: String::from(raw), ts: now },
                command: Command::Privmsg {
                    nick: "nick".into(),
                    channel: "#channel".into(),
                    message: "chat chat chat".into(),
                },
                source: Some("nick!username@host".into()),
            },
            got
        );
    }

    #[test]
    fn parse_ping() {
        let raw = "PING foo.example.com";
        let now = SystemTime::now();
        let got = Msg::parse(raw, now).unwrap();

        assert_eq!(
            Msg {
                meta: MsgMeta { raw: String::from(raw), ts: now },
                command: Command::Ping { token: Some("foo.example.com".into()) },
                source: None,
            },
            got
        )
    }
}
