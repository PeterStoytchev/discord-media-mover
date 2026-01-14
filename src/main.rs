use discord_media_mover::discord_handler::Handler;
use std::env;

use dotenvy::dotenv;
use serenity::{Client, all::GatewayIntents};

#[tokio::main]
async fn main() {
    dotenv().ok();

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
    };

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client!");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}")
    }
}
