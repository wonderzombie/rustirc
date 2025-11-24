mod example_handler;
mod reply;
mod rumors;
mod seen;
mod welcome;

use clap::Parser;
use irc_core::{self, bot};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    nick: String,
    #[arg(short, long)]
    user: String,
    #[arg(short, long, default_value_t = String::from("irc.libera.chat:6667"))]
    server: String,
    #[arg(short, long, default_value = "#el_rb_test376")]
    channels: Vec<String>,
    #[arg(short, long, default_value_t = String::from("rumors.db"))]
    db_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Hello, world!");
    let args = Args::parse();
    println!(
        "=== Using nick: {}, user: {}, server: {}, channels: {:?}",
        args.nick, args.user, args.server, args.channels
    );

    let state = irc_core::handler::State {
        channels: args.channels.clone(),
        ..Default::default()
    };

    let rumors_handler = rumors::RumorsHandler::new(&args.db_url, &args.nick).await?;

    let client = irc_core::connect(args.server, args.nick, args.user).await?;
    let bot = bot::BotBuilder::new_with_state(state)
        .with_handler(example_handler::ExampleHandler)
        .with_handler(welcome::WelcomeHandler)
        .with_handler(reply::ReplyHandler)
        .with_handler(seen::SeenHandler)
        .with_handler(rumors_handler)
        .build(client);

    bot.run().await?;

    Ok(())
}
