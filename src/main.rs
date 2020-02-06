use std::env;
use std::error::Error;

use lambda_runtime::{error::HandlerError, lambda, Context};
use log::Level;
use log::{error, info};
use serde_json::Value;
use tokio::runtime::Runtime;

use dynamo_db::DynamoClient;
use posts::fetch_posts;
use telegram::TelegramClient;

mod dynamo_db;
mod posts;
mod telegram;

fn main() {
    simple_logger::init_with_level(Level::Info).expect("Failed to init logger");
    lambda!(handler);
}

fn handler(event: Value, _: Context) -> Result<Value, HandlerError> {
    let mut rt = Runtime::new().unwrap();
    rt.block_on(async {
        match process_posts().await {
            Ok(()) => info!("successfully processed posts"),
            Err(e) => error!("error occurred while processing posts: {}", e),
        }
    });
    Ok(event)
}

async fn process_posts() -> Result<(), Box<dyn Error>> {
    let token = env::var("TG_TOKEN").expect("Missing TG_TOKEN env var");
    let chat_id = env::var("TG_CHAT_ID").expect("Missing TG_CHAT_ID env var");
    let table_name = env::var("TABLE_NAME").expect("Missing TABLE_NAME env var");

    let dynamo_client = DynamoClient::new(table_name);
    let telegram_client = TelegramClient::new(token, chat_id);

    let posts = fetch_posts().await?;
    info!("found {} posts", posts.len());

    for post in posts {
        if let None = dynamo_client.get_post(&post.id)? {
            info!("sending notification for post: {:?}", post);
            telegram_client.send_message(&post.text).await?;
            dynamo_client.put_post(&post)?;
            for image in post.images {
                telegram_client.send_image(&image).await?;
            }
        } else {
            info!("post is already sent: {}", &post.id);
        }
    }

    Ok(())
}
