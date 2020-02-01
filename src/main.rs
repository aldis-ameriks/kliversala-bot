use std::error::Error;

use lambda_runtime::{Context, error::HandlerError, lambda};
use log::{error, info};
use log::Level;
use serde_json::Value;

use posts::fetch_posts;
use telegram::send_message;

mod posts;
mod telegram;

//fn main() {
//    simple_logger::init_with_level(Level::Info).unwrap();
//
//    match process_posts() {
//        Ok(()) => info!("successfully processed posts"),
//        Err(e) => error!("error occurred in post processor: {}", e),
//    }
//}

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
        // TODO: Verify if post has already been sent

        if post.id == "2465890140339822" {
            info!("sending notification for post: {:#?}", post);
            match send_message(post.text.clone()) {
                Err(e) => error!("failed to send message {}", e),
                Ok(()) => {
                    // TODO: Save sent post in db
                    continue
                },
            }
        }
    }

    Ok(())
}
