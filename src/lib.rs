#[macro_use]
extern crate lazy_static;

use std::env;
use std::error::Error;

use log::{error, info};

use dynamo_db::DynamoClient;
use posts::fetch_posts;
use telegram::TelegramClient;

pub mod dynamo_db;
pub mod posts;
pub mod telegram;

pub async fn process_posts() -> Result<(), Box<dyn Error>> {
    let token = env::var("TG_TOKEN").expect("Missing TG_TOKEN env var");
    let chat_id = env::var("TG_CHAT_ID").expect("Missing TG_CHAT_ID env var");
    let table_name = env::var("TABLE_NAME").expect("Missing TABLE_NAME env var");

    let dynamo_client = DynamoClient::new(table_name);
    let telegram_client = TelegramClient::new(token, chat_id);

    let posts = fetch_posts("https://www.facebook.com/pg/kantineKliversala/posts/").await?;
    info!("found {} posts", posts.len());

    for mut post in posts {
        match dynamo_client.get_post(&post.id).await? {
            None => {
                info!("sending notification for post: {:?}", post);
                if &post.text != "" {
                    let message_id = telegram_client.send_message(&post.text).await?;
                    post.tg_id = Some(message_id);
                }
                for image in &mut post.images {
                    let image_id = telegram_client.send_image(&image.url).await?;
                    image.tg_id = Some(image_id);
                }
                dynamo_client.put_post(&post).await?;
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            Some(mut sent_post) => {
                info!("post is already sent: {}", &sent_post.id);
                let mut updated = false;
                if sent_post.text != post.text {
                    info!(
                        "post text has been updated from: {}, to: {}",
                        &sent_post.text, &post.text
                    );
                    if let Err(e) = telegram_client
                        .edit_message_text(&sent_post.tg_id.as_ref().unwrap(), &post.text)
                        .await
                    {
                        error!("Failed to update message text: {}", e);
                    };
                    updated = true;
                }

                for (sent_image, new_image) in
                    sent_post.images.iter_mut().zip(post.images.iter_mut())
                {
                    new_image.tg_id = sent_image.tg_id.clone();
                    if sent_image.url != new_image.url {
                        info!(
                            "image has been updated from: {:?}, to: {:?}",
                            sent_image, new_image
                        );
                        if let Err(e) = telegram_client
                            .edit_message_image(&sent_image.tg_id.as_ref().unwrap(), &new_image.url)
                            .await
                        {
                            error!("Failed to update image: {}", e);
                        };
                        updated = true;
                    }
                }

                if updated {
                    post.tg_id = sent_post.tg_id.clone();
                    dynamo_client.put_post(&post).await?;
                }
            }
        }
    }

    Ok(())
}
