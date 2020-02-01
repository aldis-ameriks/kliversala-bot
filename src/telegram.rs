use std::env;
use std::error::Error;

use reqwest::blocking::{Client, Response};
use serde::Serialize;

#[derive(Serialize)]
struct Message<'a> {
    chat_id: &'a str,
    text: &'a str,
    disable_notification: bool,
}

pub fn send_message(text: &str) -> Result<(), Box<dyn Error>> {
    let token = env::var("TG_TOKEN")?;
    let message = Message {
        chat_id: "900963193", // TODO: Move chat_id inside env vars
        text,
        disable_notification: true,
    };

    let url = &format!("https://api.telegram.org/bot{}/sendMessage", token);
    let resp: Response = Client::builder().build()?.post(url).json(&message).send()?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(resp.text()?.into())
    }
}
