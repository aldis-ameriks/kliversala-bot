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
    let token = env::var("TG_TOKEN").expect("Missing TG_TOKEN env var");
    let chat_id = env::var("TG_CHAT_ID").expect("Missing TG_CHAT_ID env var");
    let message = Message {
        chat_id: chat_id.as_str(),
        text,
        disable_notification: true,
    };

    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);
    let resp: Response = Client::builder()
        .build()?
        .post(&url)
        .json(&message)
        .send()?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(resp.text()?.into())
    }
}

pub fn send_image(url: &str) -> Result<(), Box<dyn Error>> {
    let token = env::var("TG_TOKEN").expect("Missing TG_TOKEN env var");
    let chat_id = env::var("TG_CHAT_ID").expect("Missing TG_CHAT_ID env var");
    let params = [("chat_id", chat_id.as_str()), ("photo", url)];
    let url = format!("https://api.telegram.org/bot{}/sendPhoto", token);
    let resp: Response = Client::builder().build()?.post(&url).form(&params).send()?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(resp.text()?.into())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Missing TG_TOKEN env var")]
    fn send_message_missing_token() {
        send_message("message").unwrap();
    }

    #[test]
    #[should_panic(expected = "Missing TG_TOKEN env var")]
    fn send_image_missing_token() {
        send_image("image url").unwrap();
    }
}
