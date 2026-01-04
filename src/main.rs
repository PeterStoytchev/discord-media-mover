use std::env;

use serenity::{
    Client,
    all::{ChannelId, CreateAttachment, CreateMessage, EventHandler, GatewayIntents, Message},
    async_trait,
    futures::future::join_all,
    prelude::Context,
};

struct Handler;

const DEST_CHANNEL_ID: ChannelId = ChannelId::new(829861206777004042);

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.attachments.len() == 0 || msg.channel_id == DEST_CHANNEL_ID {
            return;
        }

        let gifs_promises = msg
            .attachments
            .iter()
            .filter(|attachment| attachment.content_type.as_ref().unwrap() == "image/gif")
            .map(async |attachment| {
                println!(
                    "Message: {}, Channel: {}, Attachement ID: {}",
                    msg.id, msg.channel_id, attachment.id
                );

                let data = attachment.download().await.unwrap();
                CreateAttachment::bytes(data, attachment.filename.clone())
            });

        let gifs = join_all(gifs_promises).await;
        DEST_CHANNEL_ID
            .send_files(ctx.http.clone(), gifs, CreateMessage::new())
            .await
            .unwrap();
    }
}

#[tokio::main]
async fn main() {
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
