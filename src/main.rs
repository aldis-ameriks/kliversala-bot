use lambda_runtime::{error::HandlerError, lambda, Context};
use log::Level;
use log::{error, info};
use serde_json::Value;
use tokio::runtime::Runtime;

use kliversala_bot::process_posts;

fn main() {
    simple_logger::init_with_level(Level::Info).expect("Failed to init logger");
    lambda!(|event: Value, _: Context| -> Result<Value, HandlerError> {
        let mut rt = Runtime::new().unwrap();
        rt.block_on(async {
            match process_posts().await {
                Ok(()) => info!("successfully processed posts"),
                Err(e) => error!("error occurred while processing posts: {}", e),
            }
        });
        Ok(event)
    });
}
