use futures::stream::{self, StreamExt};
use linkify::{LinkFinder, LinkKind};
use serenity::all::{Attachment, ChannelId, CreateAttachment, MessageId};
use tokio::process::Command;
use tracing::{Span, instrument};

#[instrument(fields(curl_content_type = tracing::field::Empty))]
pub async fn is_gif_via_curl(url: &str) -> bool {
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

            Span::current().record("curl_content_type", ct);
            ct.contains("image/gif")
        }
        Err(_) => false,
    }
}

#[instrument(fields(gif_attachements = tracing::field::Empty))]
pub async fn generate_attachements(
    attachments: Vec<Attachment>,
    msg_id: MessageId,
    channel_id: ChannelId,
) -> Vec<CreateAttachment> {
    let filtered_attachements = attachments
        .into_iter()
        .filter(|attachment| attachment.content_type.as_ref().unwrap() == "image/gif");

    let gifs: Vec<CreateAttachment> = stream::iter(filtered_attachements)
        .map(async |attachment| {
            let data = attachment.download().await.unwrap();
            CreateAttachment::bytes(data, &attachment.filename)
        })
        .buffer_unordered(5)
        .collect()
        .await;

    let names: Vec<&str> = gifs
        .iter()
        .map(|attachement| attachement.filename.as_str())
        .collect();

    Span::current().record("gif_attachements", tracing::field::debug(&names));
    return gifs;
}

#[instrument(fields(gif_links = tracing::field::Empty, links = tracing::field::Empty))]
pub async fn detect_link_embeds(content: &String, domains: &Vec<String>) -> Option<Vec<String>> {
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url]);

    let found_urls: Vec<String> = finder
        .links(&content)
        .map(|l| l.as_str().to_string())
        .collect();

    Span::current().record("links", tracing::field::debug(&found_urls));

    let gif_embeds: Vec<String> = stream::iter(found_urls.into_iter())
        .filter(|link| {
            let mut url = link.clone();
            url.make_ascii_lowercase();

            let domains = domains.clone();

            async move {
                for domain in domains {
                    if url.contains(domain.as_str()) {
                        return true;
                    }
                }

                if url.ends_with(".gif") {
                    return true;
                }

                let is_gif = is_gif_via_curl(&url).await;

                is_gif
            }
        })
        .collect()
        .await;

    Span::current().record("gif_links", tracing::field::debug(&gif_embeds));

    if gif_embeds.len() == 0 {
        return None;
    }

    Some(gif_embeds)
}
