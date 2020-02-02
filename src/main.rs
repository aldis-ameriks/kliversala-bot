use std::env;
use std::error::Error;

use lambda_runtime::{error::HandlerError, lambda, Context};
use log::Level;
use log::{error, info};
use serde_json::Value;

use db::Client;
use posts::fetch_posts;
use telegram::TelegramClient;

mod db;
mod posts;
mod telegram;

fn main() {
    simple_logger::init_with_level(Level::Info).expect("Failed to init logger");
    lambda!(handler)
    //    match process_posts() {
    //        Ok(()) => info!("successfully processed posts"),
    //        Err(e) => error!("error occurred while processing posts: {}", e),
    //    }
}

fn handler(event: Value, _: Context) -> Result<Value, HandlerError> {
    match process_posts() {
        Ok(()) => info!("successfully processed posts"),
        Err(e) => error!("error occurred while processing posts: {}", e),
    }
    Ok(event)
}

fn process_posts() -> Result<(), Box<dyn Error>> {
    let token = env::var("TG_TOKEN").expect("Missing TG_TOKEN env var");
    let chat_id = env::var("TG_CHAT_ID").expect("Missing TG_CHAT_ID env var");

    let client = Client::new();
    let telegram_client = TelegramClient::new(token, chat_id);

    let posts = fetch_posts()?;
    info!("found {} posts", posts.len());

    for post in posts {
        if let None = client.get_post(&post.id)? {
            info!("sending notification for post: {:?}", post);
            telegram_client.send_message(&post.text)?;
            client.put_post(&post)?;
            for image in post.images {
                telegram_client.send_image(&image)?;
            }
        } else {
            info!("post is already sent: {}", &post.id)
        }
    }

    Ok(())
}
