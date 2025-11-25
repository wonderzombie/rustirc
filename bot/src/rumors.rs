use crate::irc_core::handler::{Context, PrivmsgHandler};
use sqlx::{Pool, Result as SqlxResult, Sqlite};

pub struct RumorsHandler {
    db_pool: Pool<Sqlite>,
    bot_name: String,
    canned_prefixes: Vec<&'static str>,
}

#[async_trait::async_trait]
impl PrivmsgHandler for RumorsHandler {
    async fn handle_privmsg(
        &self,
        ctx: &Context,
        source: &str,
        channel: &str,
        message: &str,
    ) -> std::ops::ControlFlow<()> {
        if let Some(stripped) = strip_bot_prefix(&self.bot_name, message) {
            if let Some(topic) = extract_topic(stripped) {
                if let Ok(Some(rumor)) = self.fetch_random_rumor_matching(topic).await {
                    let response = format!("{} {}", self.random_prefix(), rumor);
                    let _ = ctx.client.privmsg(channel, &response).await;
                }
            } else if !source.is_empty() && !stripped.is_empty() {
                // Store the rumor only if it has a source and a message
                let _ = self.store_rumor(source, channel, stripped).await;
                let _ = ctx.client.privmsg(channel, "Good to know!").await;
            }
            return std::ops::ControlFlow::Break(());
        }

        std::ops::ControlFlow::Continue(())
    }
}

impl RumorsHandler {
    pub async fn new(db_url: &str, bot_name: &str) -> SqlxResult<Self> {
        let pool = Pool::<Sqlite>::connect(db_url).await?;
        // TODO: load from schema.sql file
        sqlx::query(
            r#"
          CREATE TABLE IF NOT EXISTS rumors (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              nick TEXT NOT NULL,
              channel TEXT NOT NULL,
              message TEXT NOT NULL,
              ts DATETIME DEFAULT CURRENT_TIMESTAMP
          )"#,
        )
        .execute(&pool)
        .await?;

        // TODO: load from config or default to these
        const CANNED_PREFIXES: &[&'static str] = &[
            "rumor has it",
            "i heard that",
            "they say",
            "word on the street is",
            "people are saying",
        ];

        Ok(Self {
            db_pool: pool,
            bot_name: bot_name.to_string(),
            canned_prefixes: CANNED_PREFIXES.to_vec(),
        })
    }

    async fn store_rumor(&self, nick: &str, channel: &str, rumor: &str) -> SqlxResult<()> {
        let now = chrono::Local::now().timestamp();
        sqlx::query_scalar::<_, String>(
            r#"INSERT INTO rumors (nick, channel, message, ts) VALUES (?1, ?2, ?3, ?4)"#,
        )
        .bind(nick)
        .bind(channel)
        .bind(rumor)
        // TODO: investigate if we need to bind ts or if it auto-fills
        .bind(now)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn fetch_random_rumor_matching(&self, query: &str) -> SqlxResult<Option<String>> {
        let row = sqlx::query_scalar::<_, String>(
            r#"SELECT message FROM rumors WHERE message LIKE ?1 ORDER BY RANDOM() LIMIT 1"#,
        )
        .bind(format!("%{}%", query))
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(row)
    }

    fn random_prefix(&self) -> &str {
        use rand::seq::IndexedRandom;

        let mut rng = rand::rng();
        self.canned_prefixes
            .as_slice()
            .choose(&mut rng)
            .unwrap_or(&"i may be broken but i know for sure that")
    }
}

/// Strips the bot's name prefix from a message; returns None if bot_name is not present.
fn strip_bot_prefix<'a>(bot_name: &str, message: &'a str) -> Option<&'a str> {
    let lowered = message.to_lowercase();
    let bot_name_lower = bot_name.to_lowercase();

    if lowered.starts_with(&format!("{},", bot_name_lower))
        || lowered.starts_with(&format!("{}:", bot_name_lower))
    {
        Some(&message[bot_name.len() + 1..].trim())
    } else {
        None
    }
}

/// Returns the tail of a message after the first interrogative word, if any.
fn extract_query_tail<'a>(message: &'a str) -> Option<&'a str> {
    let interrogatives: [&'static str; 6] = ["who", "what", "when", "where", "why", "how"];
    let lowered = message.to_lowercase();
    let orig_words: Vec<&str> = message.split_whitespace().collect();
    let lowered_words: Vec<&str> = lowered.split_whitespace().collect();

    for (i, word) in lowered_words.iter().enumerate() {
        if interrogatives.contains(word) && i + 1 < orig_words.len() {
            return Some(&message[message.find(orig_words[i + 1]).unwrap_or(message.len())..]);
        }
    }

    None
}

/// Extracts a topic from a message ending with a question mark, or None if not found.
fn extract_topic<'a>(message: &'a str) -> Option<&'a str> {
    let trimmed = message.trim();
    if trimmed.ends_with("?") {
        trimmed.trim_end_matches("?").split_whitespace().next()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_bot_prefix() {
        let bot_name = "RumorBot";
        assert_eq!(
            strip_bot_prefix(bot_name, "RumorBot, tell me a rumor"),
            Some("tell me a rumor")
        );
        assert_eq!(
            strip_bot_prefix(bot_name, "RumorBot: what's the news?"),
            Some("what's the news?")
        );
        assert_eq!(strip_bot_prefix(bot_name, "Hello RumorBot"), None);
    }

    #[test]
    fn test_extract_query_tail() {
        assert_eq!(
            extract_query_tail("Who is the best coder?"),
            Some("is the best coder?")
        );
        assert_eq!(extract_query_tail("Tell me a rumor about Rust."), None);
    }

    #[test]
    fn test_extract_topic() {
        assert_eq!(extract_topic("rust???"), Some("rust"));
        assert_eq!(extract_topic("something interesting?"), Some("something"));
    }

    #[tokio::test]
    async fn test_new_rumors_handler() {
        let handler = RumorsHandler::new("sqlite::memory:", "RumorBot").await;
        assert!(handler.is_ok());
    }

    #[tokio::test]
    async fn test_store_rumor_fetch_rumor() {
        let handler = RumorsHandler::new("sqlite::memory:", "RumorBot")
            .await
            .expect("Failed to create RumorsHandler");

        handler
            .store_rumor("thumbkin", "#channel", "botty was written in rust")
            .await
            .expect("Failed to store rumor");

        let fetched = handler
            .fetch_random_rumor_matching("rust")
            .await
            .expect("Failed to fetch rumor");

        assert_eq!(fetched, Some("botty was written in rust".to_string()));
    }

    #[tokio::test]
    async fn test_random_prefix() {
        let handler = RumorsHandler {
            db_pool: Pool::<Sqlite>::connect_lazy("sqlite::memory:").unwrap(),
            bot_name: "RumorBot".to_string(),
            canned_prefixes: vec!["prefix1", "prefix2", "prefix3"],
        };

        let prefix = handler.random_prefix();
        assert!(handler.canned_prefixes.contains(&prefix));
    }
}
