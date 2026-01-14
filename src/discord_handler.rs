use std::time::Duration;

use serenity::{
    all::{
        ChannelId, Context, CreateMessage, EditMessage, EventHandler, Mentionable, Message,
        ReactionType, Ready,
    },
    async_trait,
};
use tokio::time::sleep;
use tracing::info;

use crate::utils::{detect_link_embeds, generate_attachements};

pub struct Handler {
    pub dest_channel_id: u64,
    pub gif_keep_duration: Duration,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let dest_channel_id = ChannelId::new(self.dest_channel_id);

        if msg.channel_id == dest_channel_id || msg.author.bot {
            return;
        }

        let embeds = detect_link_embeds(&msg.content).await;

        if embeds.is_none() && msg.attachments.len() == 0 {
            return;
        }

        let gifs = generate_attachements(msg.attachments.clone(), msg.id, msg.channel_id).await;

        let gif_message = match &embeds {
            Some(_) => CreateMessage::new().content("New gif!"),
            None => CreateMessage::new(),
        };

        msg.react(&ctx.http, ReactionType::Unicode("⏱️".to_string()))
            .await
            .unwrap();

        let duration = self.gif_keep_duration;
        tokio::spawn(async move {
            sleep(duration).await;

            let mut gif_message = dest_channel_id
                .send_files(&ctx.http, gifs, gif_message)
                .await
                .unwrap();

            let new_message = CreateMessage::new().content(format!(
                "{}\nGif(s) rerouted to {}. Original message sent by {}",
                match &embeds {
                    None => msg.content.clone(),
                    Some(vals) => vals
                        .iter()
                        .fold(msg.content.clone(), |acc, word| acc.replace(word, ""))
                        .split_whitespace()
                        .collect::<Vec<&str>>()
                        .join(" "),
                },
                gif_message.link(),
                &msg.author.mention()
            ));

            let new_message = &msg
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
                            &new_message.link(),
                            val.join("\n")
                        )),
                        None => {
                            EditMessage::new().content(format!("Gif from: {}", &new_message.link()))
                        }
                    },
                )
                .await
                .unwrap();

            msg.delete(&ctx).await.unwrap();
        });
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("Bot {} ready!", ready.user.display_name())
    }
}
