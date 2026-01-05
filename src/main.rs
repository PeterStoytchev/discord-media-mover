use std::env;

use dotenvy::dotenv;
use futures::stream::{self, StreamExt};
use serenity::{
    Client,
    all::{
        ChannelId, CreateAttachment, CreateMessage, EditMessage, EventHandler, GatewayIntents,
        Message,
    },
    async_trait,
    prelude::Context,
};

struct Handler;

const DEST_CHANNEL_ID: ChannelId = ChannelId::new(829861206777004042);

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.attachments.len() == 0 || msg.channel_id == DEST_CHANNEL_ID || msg.author.bot {
            return;
        }

        let filtered_attachements = msg
            .clone()
            .attachments
            .into_iter()
            .filter(|attachment| attachment.content_type.as_ref().unwrap() == "image/gif");

        let gifs: Vec<CreateAttachment> = stream::iter(filtered_attachements)
            .map(async |attachment| {
                println!(
                    "Message: {}, Channel: {}, Attachement ID: {}",
                    msg.id, msg.channel_id, attachment.id
                );

                let data = attachment.download().await.unwrap();
                CreateAttachment::bytes(data, attachment.filename.clone())
            })
            .buffer_unordered(5)
            .collect()
            .await;

        let new_message = CreateMessage::new().content(format!("Gif from: {}", msg.clone().link()));

        DEST_CHANNEL_ID
            .send_files(ctx.http.clone(), gifs, new_message)
            .await
            .unwrap();
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = GatewayIntents::all();

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client!");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}")
    }
}
