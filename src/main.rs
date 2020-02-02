use std::error::Error;

use lambda_runtime::{error::HandlerError, lambda, Context};
use log::Level;
use log::{error, info};
use serde_json::Value;

use db::Client;
use posts::fetch_posts;
use telegram::{send_image, send_message};

mod db;
mod posts;
mod telegram;

fn main() {
    simple_logger::init_with_level(Level::Info).expect("Failed to init logger");
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
    let posts = fetch_posts()?;
    info!("found {} posts", posts.len());

    for post in posts {
        if let None = client.get_post(&post.id)? {
            info!("sending notification for post: {:?}", post);
            send_message(&post.text)?;
            client.put_post(&post)?;
            for image in &post.images {
                send_image(&image)?;
            }
        } else {
            info!("post already sent: {}", &post.id)
        }
    }

    Ok(())
}
