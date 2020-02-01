use std::error::Error;

use lambda_runtime::{Context, error::HandlerError, lambda};
use log::{error, info};
use log::Level;
use serde_json::Value;

use db::{get_post, put_post};
use posts::fetch_posts;
use telegram::send_message;

mod posts;
mod telegram;
mod db;

fn main() {
    simple_logger::init_with_level(Level::Info).unwrap();
    lambda!(handler)
}

fn handler(event: Value, _: Context) -> Result<Value, HandlerError> {
    match process_posts() {
        Ok(()) => info!("successfully processed posts"),
        Err(e) => error!("error occurred in post processor: {}", e),
    }
    Ok(event)
}

fn process_posts() -> Result<(), Box<dyn Error>> {
    let posts = fetch_posts("https://mobile.facebook.com/kantineKliversala/posts/")?;
    info!("found {} posts", posts.len());

    for post in posts {
        match get_post(post.id.clone())? {
            Some(_) => {
                info!("post already sent: {}", &post.id);
                continue;
            }
            None => {
                info!("post is not already sent: {}", &post.id);
                info!("sending notification for post: {:#?}", post);
                match send_message(&post.text) {
                    Err(e) => error!("failed to send message {}", e),
                    Ok(()) => {
                        put_post(&post)?;
                        continue;
                    }
                }
            }
        }
    }

    Ok(())
}
