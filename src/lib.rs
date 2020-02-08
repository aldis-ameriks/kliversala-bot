use std::env;
use std::error::Error;

use log::info;

use dynamo_db::DynamoClient;
use posts::fetch_posts;
use telegram::TelegramClient;

mod dynamo_db;
mod posts;
mod telegram;

pub async fn process_posts() -> Result<(), Box<dyn Error>> {
    let token = env::var("TG_TOKEN").expect("Missing TG_TOKEN env var");
    let chat_id = env::var("TG_CHAT_ID").expect("Missing TG_CHAT_ID env var");
    let table_name = env::var("TABLE_NAME").expect("Missing TABLE_NAME env var");

    let dynamo_client = DynamoClient::new(table_name);
    let telegram_client = TelegramClient::new(token, chat_id);

    let posts = fetch_posts("https://www.facebook.com/pg/kantineKliversala/posts/").await?;
    info!("found {} posts", posts.len());

    for post in posts {
        if let None = dynamo_client.get_post(&post.id).await? {
            info!("sending notification for post: {:?}", post);
            telegram_client.send_message(&post.text).await?;
            dynamo_client.put_post(&post).await?;
            for image in post.images {
                telegram_client.send_image(&image).await?;
            }
        } else {
            info!("post is already sent: {}", &post.id);
        }
    }

    Ok(())
}
