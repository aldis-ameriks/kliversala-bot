use std::error::Error;

use lambda_runtime::{Context, error::HandlerError, lambda};
use log::{error, info};
use log::Level;
use serde_json::Value;

use db::Client;
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
    let client = Client::new();
    let posts = fetch_posts("https://mobile.facebook.com/kantineKliversala/posts/")?;
    info!("found {} posts", posts.len());

    for post in posts {
        match client.get_post(&post.id)? {
            Some(_) => info!("post already sent: {}", &post.id),
            None => {
                info!("sending notification for post: {:#?}", post);
                match send_message(&post.text) {
                    Err(e) => error!("failed to send message {}", e),
                    Ok(()) => client.put_post(&post)?
                }
            }
        }
    }

    Ok(())
}
