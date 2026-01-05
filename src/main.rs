use std::env;

use dotenvy::dotenv;
use futures::stream::{self, StreamExt};
use serenity::{
    Client,
    all::{
        Attachment, ChannelId, CreateAttachment, CreateMessage, EditMessage, Embed, EventHandler,
        GatewayIntents, Mentionable, Message, MessageId,
    },
    async_trait,
    prelude::Context,
};

struct Handler;

const DEST_CHANNEL_ID: ChannelId = ChannelId::new(829861206777004042);

async fn generate_attachements(
    attachments: Vec<Attachment>,
    msg_id: MessageId,
    channel_id: ChannelId,
) -> Vec<CreateAttachment> {
    let filtered_attachements = attachments
        .into_iter()
        .filter(|attachment| attachment.content_type.as_ref().unwrap() == "image/gif");

    let gifs: Vec<CreateAttachment> = stream::iter(filtered_attachements)
        .map(async |attachment| {
            println!(
                "Message: {}, Channel: {}, Attachement ID: {}",
                msg_id, channel_id, attachment.id
            );

            let data = attachment.download().await.unwrap();
            CreateAttachment::bytes(data, attachment.filename.clone())
        })
        .buffer_unordered(5)
        .collect()
        .await;

    return gifs;
}

fn generate_embeds(embeds: Vec<Embed>) -> Option<Vec<String>> {
    let mut gif_embeds = embeds
        .into_iter()
        .filter(|embed| {
            embed.kind.as_ref().unwrap() == "gifv" || embed.kind.as_ref().unwrap() == "image" //the right side of the || is a placeholder, to be augmented by introspection logic (check if the image is really a gif)
        })
        .peekable();

    if gif_embeds.peek().is_none() {
        return None;
    }

    let gif_embeds: Vec<String> = gif_embeds.map(|embed| embed.url.unwrap()).collect();

    Some(gif_embeds)
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if (msg.attachments.len() == 0 && msg.embeds.len() == 0)
            || msg.channel_id == DEST_CHANNEL_ID
            || msg.author.bot
        {
            return;
        }

        println!("{:?}", msg.embeds);

        let embeds = generate_embeds(msg.embeds.clone());

        if embeds.is_none() && msg.attachments.len() == 0 {
            return;
        }

        let gifs = generate_attachements(msg.attachments.clone(), msg.id, msg.channel_id).await;

        let gif_message = match embeds.clone() {
            Some(_) => CreateMessage::new().content("New gif!"),
            None => CreateMessage::new(),
        };

        let mut gif_message = DEST_CHANNEL_ID
            .send_files(ctx.http.clone(), gifs, gif_message)
            .await
            .unwrap();

        let new_message = CreateMessage::new().content(format!(
            "{}\nGif(s) rerouted to {}. Original message sent by {}",
            match embeds.clone() {
                None => msg.content.clone(),
                Some(vals) => vals
                    .iter()
                    .fold(msg.content.clone(), |acc, word| acc.replace(word, ""))
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .join(" "),
            },
            gif_message.link(),
            msg.clone().author.mention()
        ));

        let new_message = msg
            .clone()
            .channel_id
            .send_message(&ctx, new_message)
            .await
            .unwrap();

        gif_message
            .edit(
                &ctx,
                match embeds {
                    Some(val) => EditMessage::new().content(format!(
                        "Gif from: {}\n{}",
                        new_message.clone().link(),
                        val.join("\n")
                    )),
                    None => EditMessage::new()
                        .content(format!("Gif from: {}", new_message.clone().link())),
                },
            )
            .await
            .unwrap();

        msg.clone().delete(&ctx).await.unwrap();
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
