use std::time::SystemTime;

#[derive(Debug, PartialEq)]
pub enum Command {
    Ping { token: Option<String> },
    Join { nick: String, channel: String },
    Privmsg { nick: String, channel: String, message: String },
    // Part / Numeric / Unknown can be added later with String as well
}

impl Command {
    fn from_parts(nick: String, parts: &CmdParts<'_>, trailing: &str) -> Option<Command> {
        match parts.command {
            "PING" => {
                // Prefer trailing token if present, else first arg; allow None
                let token = if !trailing.is_empty() {
                    Some(trailing.to_owned())
                } else {
                    parts.args.first().map(|s| (*s).to_owned())
                };
                Some(Command::Ping { token })
            }
            "PRIVMSG" => {
                let channel = *parts.args.first()?;
                Some(Command::Privmsg {
                    nick,
                    channel: channel.to_owned(),
                    message: trailing.to_owned(),
                })
            }
            "JOIN" => {
                let channel = *parts.args.first()?;
                Some(Command::Join {
                    nick,
                    channel: channel.to_owned(),
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
pub struct Msg {
    pub meta: MsgMeta,
    pub source: Option<String>, // entire prefix if present
    pub command: Command,
}

#[derive(Debug)]
struct CmdParts<'a> {
    source: Option<&'a str>,
    command: &'a str,
    args: Vec<&'a str>,
}

impl Msg {
    pub fn parse(line: &str, now: SystemTime) -> Option<Msg> {
        let meta = MsgMeta {
            raw: line.to_owned(),
            ts: now,
        };

        let (before, trailing) = split_irc(line)?;
        let cmd_parts = Self::parse_command_tokens(before)?;

        let nick = Self::source_to_nick(cmd_parts.source);
        let source = cmd_parts.source.map(|s| s.to_owned());

        let command = Command::from_parts(nick, &cmd_parts, trailing)?;
        Some(Msg { meta, source, command })
    }

    fn source_to_nick(source: Option<&str>) -> String {
        source
            .and_then(|s| s.split('!').next())
            .unwrap_or("")
            .to_owned()
    }

    fn parse_command_tokens(before: &str) -> Option<CmdParts<'_>> {
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

    #[test]
    fn from_parts_privmsg() {
        let got = Command::from_parts(
            "nickname".to_owned(),
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
