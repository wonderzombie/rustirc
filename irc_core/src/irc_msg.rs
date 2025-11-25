use chrono::{DateTime, Local};

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Ping {
        token: Option<String>,
    },
    Join {
        channel: String,
        message: Option<String>,
    },
    Part {
        channel: String,
        message: Option<String>,
    },
    Privmsg {
        reply_to: String,
        message: String,
    },
    Notice {
        channel: String,
        message: String,
    },
    Numeric {
        code: u16,
        args: Vec<String>,
        trailing: Option<String>,
    },
    Other {},
}

impl Command {
    fn build_from_parts(parts: &CmdParts<'_>) -> Option<Command> {
        match parts.command {
            "PING" => Some(Command::Ping {
                token: parts.trailing_or_first().map(str::to_owned),
            }),

            "PRIVMSG" => Some(Command::Privmsg {
                reply_to: parts.first_arg()?.to_owned(),
                message: parts.trailing.unwrap_or_default().to_owned(),
            }),

            "JOIN" => Some(Command::Join {
                channel: parts.first_arg()?.to_owned(),
                message: parts.trailing.map(str::to_owned),
            }),

            "PART" => Some(Command::Part {
                channel: parts.first_arg()?.to_owned(),
                message: parts.trailing.map(str::to_owned),
            }),

            "NOTICE" => Some(Command::Notice {
                channel: parts.first_arg()?.to_owned(),
                message: parts.trailing.unwrap_or_default().to_owned(),
            }),

            _ if parts.is_numeric() => Some(Command::Numeric {
                code: parts.code()?,
                args: parts.args.iter().map(|s| (*s).to_owned()).collect(),
                trailing: parts.trailing_or_first().map(str::to_owned),
            }),

            _ => Some(Command::Other {}),
        }
    }
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

#[derive(Debug, PartialEq)]
pub struct MsgMeta {
    pub raw: String,
    pub ts: DateTime<Local>,
}

#[derive(Debug, PartialEq)]
pub struct Msg {
    pub meta: MsgMeta,
    pub source: Option<String>, // entire prefix if present
    pub command: Command,
}

impl Msg {
    pub fn nick(&self) -> Option<String> {
        let nick = Self::source_to_nick(self.source.as_deref());
        if nick.is_empty() { None } else { Some(nick) }
    }

    pub fn channel(&self) -> Option<String> {
        match &self.command {
            Command::Privmsg {
                reply_to: channel, ..
            } => Some(channel.into()),
            Command::Join { channel, .. } => Some(channel.into()),
            Command::Part { channel, .. } => Some(channel.into()),
            Command::Notice { channel, .. } => Some(channel.into()),
            _ => None,
        }
    }

    pub fn parse(line: &str, now: DateTime<Local>) -> Option<Msg> {
        let meta = MsgMeta {
            raw: line.to_owned(),
            ts: now,
        };

        let parts = Self::tokenize_line(line)?;
        let source = parts.source.map(|s| s.to_owned());

        let command = Command::build_from_parts(&parts)?;
        Some(Msg {
            meta,
            source,
            command,
        })
    }

    /// Extracts the nick from the IRC message source string.
    /// The expected format is "nick!username@host". If the source does not contain '!',
    /// the entire source string is returned as the nick. If the source is None, returns an empty string.
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
    use std::time::SystemTime;

    use super::*;

    static FAKE_NOW: SystemTime = SystemTime::UNIX_EPOCH;

    #[test]
    fn from_parts_privmsg() {
        let got = Command::build_from_parts(&CmdParts {
            source: Some("nickname!+username@host"),
            command: "PRIVMSG",
            args: vec!["#channel"],
            trailing: Some("chat chat chat"),
        })
        .unwrap();

        assert_eq!(
            Command::Privmsg {
                reply_to: "#channel".into(),
                message: "chat chat chat".into(),
            },
            got
        );
    }

    #[test]
    fn from_parts_notice() {
        let got = Command::build_from_parts(&CmdParts {
            source: Some("irc.example.com"),
            command: "NOTICE",
            args: vec!["*"],
            trailing: Some("*** Looking up your hostname..."),
        })
        .unwrap();
        assert_eq!(
            Command::Notice {
                channel: "*".into(),
                message: "*** Looking up your hostname...".into(),
            },
            got
        );
    }

    #[test]
    fn parse_privmsg() {
        let raw = ":nick!username@host PRIVMSG #channel :chat chat chat";
        let got = Msg::parse(raw, FAKE_NOW.into()).unwrap();

        assert_eq!(
            Msg {
                meta: MsgMeta {
                    raw: String::from(raw),
                    ts: FAKE_NOW.into(),
                },
                command: Command::Privmsg {
                    reply_to: "#channel".into(),
                    message: "chat chat chat".into(),
                },
                source: Some("nick!username@host".into()),
            },
            got
        );
    }

    #[test]
    fn parse_numeric_welcome() {
        let raw = ":irc.example.com 001 nickname :Welcome to IRC you cheeky nickname!user@host";
        let got = Msg::parse(raw, FAKE_NOW.into()).unwrap();

        assert_eq!(
            Msg {
                meta: MsgMeta {
                    raw: raw.into(),
                    ts: FAKE_NOW.into()
                },
                source: Some("irc.example.com".into()),
                command: Command::Numeric {
                    code: 001,
                    args: vec!["nickname".into()],
                    trailing: Some("Welcome to IRC you cheeky nickname!user@host".into())
                },
            },
            got
        );
    }

    #[test]
    fn parse_numeric_topic() {
        let raw = ":irc.example.com 332 nickname #channel  :This is the new topic";
        let got = Msg::parse(raw, FAKE_NOW.into()).unwrap();

        assert_eq!(
            Msg {
                meta: MsgMeta {
                    raw: raw.into(),
                    ts: FAKE_NOW.into(),
                },
                source: Some("irc.example.com".into()),
                command: Command::Numeric {
                    code: 332,
                    args: vec!["nickname".into(), "#channel".into()],
                    trailing: Some("This is the new topic".into())
                }
            },
            got
        );
    }

    #[test]
    fn parse_join() {
        let raw = ":nick!username@host JOIN #channel :hello world";
        let got = Msg::parse(raw, FAKE_NOW.into()).unwrap();

        assert_eq!(
            Msg {
                meta: MsgMeta {
                    raw: String::from(raw),
                    ts: FAKE_NOW.into(),
                },
                command: Command::Join {
                    channel: "#channel".into(),
                    message: Some("hello world".into()),
                },
                source: Some("nick!username@host".into()),
            },
            got
        );
        assert_eq!("nick", got.nick().unwrap());
        assert_eq!("#channel", got.channel().unwrap());
    }

    #[test]
    fn parse_ping() {
        let raw = "PING foo.example.com";
        let got = Msg::parse(raw, FAKE_NOW.into()).unwrap();

        assert_eq!(
            Msg {
                meta: MsgMeta {
                    raw: String::from(raw),
                    ts: FAKE_NOW.into(),
                },
                command: Command::Ping {
                    token: Some("foo.example.com".into())
                },
                source: None,
            },
            got
        )
    }

    #[test]
    fn parse_notice() {
        let raw = ":irc.example.com NOTICE * :*** Looking up your hostname...";
        let got = Msg::parse(raw, FAKE_NOW.into());

        assert!(!got.is_none());
    }

    #[test]
    fn msg_nick_extraction() {
        let msg = Msg {
            meta: MsgMeta {
                raw: String::new(),
                ts: FAKE_NOW.into(),
            },
            source: Some("nickname!username@host".into()),
            command: Command::Other {},
        };
        assert_eq!(msg.nick(), Some("nickname".into()));
    }

    #[test]
    fn msg_nick_extraction_no_prefix() {
        let msg = Msg {
            meta: MsgMeta {
                raw: String::new(),
                ts: FAKE_NOW.into(),
            },
            source: None,
            command: Command::Other {},
        };
        assert_eq!(msg.nick(), None);
    }

    #[test]
    fn msg_nick_extraction_no_nick() {
        let msg = Msg {
            meta: MsgMeta {
                raw: String::new(),
                ts: FAKE_NOW.into(),
            },
            source: Some("".into()),
            command: Command::Other {},
        };
        assert_eq!(msg.nick(), None);
    }

    #[test]
    fn msg_channel_extraction() {
        let msg = Msg {
            meta: MsgMeta {
                raw: String::new(),
                ts: FAKE_NOW.into(),
            },
            source: Some("nickname!username@host".into()),
            command: Command::Privmsg {
                reply_to: "#channel".into(),
                message: "hello".into(),
            },
        };
        assert_eq!(msg.channel(), Some("#channel".into()));
    }

    #[test]
    fn msg_channel_extraction_no_channel() {
        let msg = Msg {
            meta: MsgMeta {
                raw: String::new(),
                ts: FAKE_NOW.into(),
            },
            source: Some("nickname!username@host".into()),
            command: Command::Ping { token: None },
        };
        assert_eq!(msg.channel(), None);
    }
}
