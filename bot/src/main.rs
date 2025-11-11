
use irc_core;
use irc_core::bot;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    nick: String,
    #[arg(short, long)]
    user: String,
    #[arg(short, long, default_value_t = String::from("irc.libera.chat:6667"))]
    server: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Hello, world!");
    let args = Args::parse();
    println!("=== Using nick: {}, user: {}, server: {}", args.nick, args.user, args.server);

    let client = irc_core::connect(args.server, args.nick, args.user).await?;
    let bot = bot::BotBuilder::new()
        // .with_handler(...) // Add handlers here
        .build(client);

    bot.run().await?;

    Ok(())
}
