use irc_core;
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
    irc_core::start(&args.server, &args.nick, &args.user).await
}
