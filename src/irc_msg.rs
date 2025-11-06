use std::time::SystemTime;

#[derive(Debug, PartialEq)]
pub enum Command {
    Ping {
        token: Option<String>,
    },
    Join {
        nick: String,
        channel: String,
    },
    Privmsg {
        nick: String,
        channel: String,
        message: String,
    },
    // Part {
    //     nick: String,
    //     channel: String,
    //     message: Option<String>,
    // },
    // Numeric {
    //     code: u16,
    //     trailing: Option<String>,
    // },
}

impl Command {
    fn from_parts(nick: String, parts: CmdParts<'_>, trailing: &str) -> Option<Command> {
        // println!("nick {} parts {:#?} trailing {}", nick, parts, trailing);
        match parts.command {
            "PING" => {
                let token = *parts.args.first()?;
                Some(Command::Ping {
                    token: Some(token.to_owned()),
                })
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
    pub source: Option<String>,
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
            raw: line.to_string(),
            ts: now,
        };

        let (before, trailing) = split_irc(line)?;
        let cmd_parts = Self::parse_command_tokens(before)?;

        let nick = Self::source_to_nick(cmd_parts.source);
        let message = trailing.to_string();
        let source = cmd_parts.source.map(|s| s.to_string());
        let first_arg = cmd_parts.args.first();

        match cmd_parts.command {
            "PING" => Some(Msg {
                meta,
                source,
                command: Command::Ping {
                    token: cmd_parts.source.map(|s| String::from(s)),
                },
            }),
            "PRIVMSG" => Some(Msg {
                meta,
                source,
                command: Command::Privmsg {
                    nick: nick,
                    channel: first_arg
                        .expect("PRIVMSG requires a channel argument")
                        .to_string(),
                    message: message,
                },
            }),
            "JOIN" => Some(Msg {
                meta,
                source,
                command: Command::Join {
                    nick,
                    channel: first_arg
                        .expect("JOIN requires a channel argument")
                        .to_string(),
                },
            }),
            _ => None,
        }
    }

    fn source_to_nick(source: Option<&str>) -> String {
        source
            .and_then(|s| s.split("!").next())
            .unwrap_or_default()
            .to_string()
    }

    fn parse_command_tokens<'a>(before: &'a str) -> Option<CmdParts<'a>> {
        let mut tokens = before.split_ascii_whitespace();
        let maybe_source = tokens.next();
        let source = maybe_source.and_then(|w| w.strip_prefix(":"));
        let command = tokens.next()?;
        let args = tokens.collect();
        Some(CmdParts {
            source,
            command,
            args,
        })
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
            String::from("nickname"),
            CmdParts {
                source: Some("nickname!+username@host"),
                command: "PRIVMSG",
                args: vec![&"#channel"],
            },
            "chat chat chat",
        )
        .unwrap();

        assert_eq!(
            Command::Privmsg {
                nick: String::from("nickname"),
                channel: String::from("#channel"),
                message: String::from("chat chat chat")
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
                meta: MsgMeta {
                    raw: String::from(raw),
                    ts: now,
                },
                command: Command::Privmsg {
                    nick: String::from("nick"),
                    channel: String::from("#channel"),
                    message: String::from("chat chat chat"),
                },
                source: Some(String::from("nick!username@host")),
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
                meta: MsgMeta {
                    raw: String::from(raw),
                    ts: now
                },
                command: Command::Ping {
                    token: Some(String::from("foo.example.com"))
                },
                source: None,
            },
            got
        )
    }
}
