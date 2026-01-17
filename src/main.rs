use discord_media_mover::discord_handler::Handler;
use std::time::Duration;
use std::{env, str::FromStr};

use dotenvy::dotenv;
use serenity::{Client, all::GatewayIntents};

use humantime::parse_duration;

use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let crate_name = env!("CARGO_CRATE_NAME");

    let llevel = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let cllevel = env::var("CRATE_LOG_LEVEL").unwrap_or_else(|_| "warn".to_string());
    let filter_directives = format!("{},{}={}", cllevel, crate_name, llevel);

    let filter = EnvFilter::new(filter_directives);

    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_env_filter(filter)
        .init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::all();

    let handler = Handler {
        dest_channel_id: match env::var("DEST_CHANNEL_ID") {
            Ok(val) => match val.parse::<u64>() {
                Ok(num) => num,
                Err(_) => {
                    panic!("Invalid DEST_CHANNEL_ID provided!");
                }
            },
            Err(_) => {
                panic!("DEST_CHANNEL_ID env variable not provided!");
            }
        },
        gif_keep_duration: match env::var("GIF_KEEP_TIME") {
            Ok(val) => match parse_duration(&val) {
                Ok(time) => time,
                Err(_) => Duration::from_secs(10),
            },
            Err(_) => Duration::from_secs(10),
        },
        banned_domains: match env::var("BANNED_DOMAINS") {
            Ok(val) => val
                .split(";")
                .map(|domain| String::from_str(domain).unwrap())
                .collect(),
            Err(_) => vec![
                String::from_str("klipy.com").unwrap(),
                String::from_str("tenor.com").unwrap(),
            ],
        },
        banned_formats: match env::var("BANNED_FORMATS") {
            Ok(val) => val
                .split(";")
                .map(|domain| String::from_str(domain).unwrap())
                .collect(),
            Err(_) => vec![
                String::from_str("image/gif").unwrap(),
                String::from_str("image/avif").unwrap(),
            ],
        },
    };

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client!");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}")
    }
}
