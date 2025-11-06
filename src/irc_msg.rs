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
    Numeric {
        code: u16,
        args: Vec<String>,
        trailing: Option<String>,
    },
    // Part / Numeric / Unknown can be added later with String as well
}

impl Command {
    fn build_from_parts(nick: String, parts: &CmdParts<'_>) -> Option<Command> {
        match parts.command {
            "PING" => Some(Command::Ping {
                token: parts.trailing_or_first().map(str::to_owned),
            }),

            "PRIVMSG" => Some(Command::Privmsg {
                nick,
                channel: parts.first_arg()?.to_owned(),
                message: parts.trailing.unwrap_or_default().to_owned(),
            }),

            "JOIN" => Some(Command::Join {
                nick,
                channel: parts.first_arg()?.to_owned(),
            }),

            _ if parts.is_numeric() => Some(Command::Numeric {
                code: parts.code()?,
                args: parts.args.iter().map(|s| (*s).to_owned()).collect(),
                trailing: parts.trailing_or_first().map(str::to_owned),
            }),
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
    trailing: Option<&'a str>,
}

impl<'a> CmdParts<'a> {
    fn is_numeric(&self) -> bool {
        self.command.len() == 3 && self.command.as_bytes().iter().all(u8::is_ascii_digit)
    }

    fn code(&self) -> Option<u16> {
        (self.is_numeric())
            .then(|| self.command.parse().ok())
            .flatten()
    }

    fn first_arg(&self) -> Option<&'_ str> {
        self.args.first().copied()
    }

    fn trailing_or_first<'t>(&'t self) -> Option<&'t str> {
        self.trailing.or_else(|| self.first_arg())
    }
}

impl Msg {
    pub fn parse(line: &str, now: SystemTime) -> Option<Msg> {
        let meta = MsgMeta {
            raw: line.to_owned(),
            ts: now,
        };

        let parts = Self::tokenize_line(line)?;
        let nick = Self::source_to_nick(parts.source);
        let source = parts.source.map(|s| s.to_owned());

        let command = Command::build_from_parts(nick, &parts)?;
        Some(Msg {
            meta,
            source,
            command,
        })
    }

    fn source_to_nick(source: Option<&str>) -> String {
        source
            .and_then(|s| s.split('!').next())
            .unwrap_or("")
            .to_owned()
    }

    fn tokenize_line<'a>(line: &'_ str) -> Option<CmdParts<'_>> {
        let (before, trailing) = split_irc(line)?;
        let mut it = before.split_ascii_whitespace();

        let first = it.next()?;
        let (source, command) = if first.starts_with(':') {
            (Some(first.trim_start_matches(':')), it.next()?)
        } else {
            (None, first)
        };

        let args = it.collect();
        Some(CmdParts {
            source,
            command,
            args,
            trailing,
        })
    }
}

fn split_irc(line: &str) -> Option<(&str, Option<&str>)> {
    let mut parts = line.splitn(2, " :");
    let before = parts.next()?;
    let trailing = parts.next();
    Some((before, trailing))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_parts_privmsg() {
        let got = Command::build_from_parts(
            "nickname".to_owned(),
            &CmdParts {
                source: Some("nickname!+username@host"),
                command: "PRIVMSG",
                args: vec!["#channel"],
                trailing: Some("chat chat chat"),
            },
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
                meta: MsgMeta {
                    raw: String::from(raw),
                    ts: now
                },
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
                meta: MsgMeta {
                    raw: String::from(raw),
                    ts: now
                },
                command: Command::Ping {
                    token: Some("foo.example.com".into())
                },
                source: None,
            },
            got
        )
    }
}
