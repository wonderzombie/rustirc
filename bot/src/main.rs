use irc_core;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    irc_core::start().await
}
