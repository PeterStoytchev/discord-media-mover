use std::{env, time::Duration};
use tokio::{process::Command, time::sleep};

use dotenvy::dotenv;
use futures::stream::{self, StreamExt};
use linkify::{LinkFinder, LinkKind};
use serenity::{
    Client,
    all::{
        Attachment, ChannelId, CreateAttachment, CreateMessage, EditMessage, EventHandler,
        GatewayIntents, Mentionable, Message, MessageId,
    },
    async_trait,
    prelude::Context,
};

struct Handler {
    dest_channel_id: u64,
}

async fn is_gif_via_curl(url: &str) -> bool {
    let output = Command::new("curl")
        .arg("-I")
        .arg("-L")
        .arg("-A")
        .arg("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .arg(url)
        .output().await;

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);

            let lower_out = stdout.to_lowercase();
            let ct = lower_out
                .lines()
                .filter(|line| line.starts_with("content-type"))
                .next()
                .unwrap();

            println!("{}", ct);
            ct.contains("image/gif")
        }
        Err(_) => false,
    }
}

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

async fn detect_link_embeds(content: String) -> Option<Vec<String>> {
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url]);

    let found_urls: Vec<String> = finder
        .links(&content)
        .map(|l| l.as_str().to_string())
        .collect();

    let gif_embeds: Vec<String> = stream::iter(found_urls.into_iter())
        .filter(|link| {
            let url = link.clone();

            async move {
                if url.clone().to_lowercase().contains("tenor.com") {
                    println!("Added! Tenor link detected.");
                    return true;
                }

                if url.clone().split(".").last().unwrap() == "gif" {
                    println!(".gif detected");
                    return true;
                }

                let is_gif = is_gif_via_curl(&url).await;

                is_gif
            }
        })
        .collect()
        .await;

    if gif_embeds.len() == 0 {
        return None;
    }

    Some(gif_embeds)
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let dest_channel_id = ChannelId::new(self.dest_channel_id);

        if msg.channel_id == dest_channel_id || msg.author.bot {
            return;
        }

        let embeds = detect_link_embeds(msg.content.clone()).await;

        if embeds.is_none() && msg.attachments.len() == 0 {
            return;
        }

        let gifs = generate_attachements(msg.attachments.clone(), msg.id, msg.channel_id).await;

        let gif_message = match embeds.clone() {
            Some(_) => CreateMessage::new().content("New gif!"),
            None => CreateMessage::new(),
        };

        tokio::spawn(async move {
            sleep(Duration::from_secs(10)).await;

            let mut gif_message = dest_channel_id
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
        });
    }
}

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
