use std::error::Error;

use reqwest::blocking::{Client, Response};
use serde::Serialize;

pub struct TelegramClient {
    token: String,
    chat_id: String,
    domain: String,
}

#[derive(Serialize)]
struct Message<'a> {
    chat_id: &'a str,
    text: &'a str,
    disable_notification: bool,
}

impl TelegramClient {
    pub fn new(token: String, chat_id: String) -> TelegramClient {
        TelegramClient {
            token,
            chat_id,
            domain: String::from("https://api.telegram.org"),
        }
    }

    pub fn new_with(token: String, chat_id: String, domain: String) -> TelegramClient {
        TelegramClient {
            token,
            chat_id,
            domain,
        }
    }

    pub fn send_message(&self, text: &str) -> Result<(), Box<dyn Error>> {
        let message = Message {
            chat_id: &self.chat_id,
            text,
            disable_notification: true,
        };

        let url = format!("{}/bot{}/sendMessage", self.domain, self.token);
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

    pub fn send_image(&self, url: &str) -> Result<(), Box<dyn Error>> {
        let params = [("chat_id", self.chat_id.as_str()), ("photo", url)];
        let url = format!("{}/bot{}/sendPhoto", self.domain, self.token);
        let resp: Response = Client::builder().build()?.post(&url).form(&params).send()?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(resp.text()?.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use mockito;

    use super::*;

    const TOKEN: &str = "token";
    const CHAT_ID: &str = "123";

    #[test]
    fn send_message_success() {
        let url = &mockito::server_url();
        let _m = mockito::mock("POST", format!("/bot{}/sendMessage", TOKEN).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .create();

        let client = TelegramClient::new_with(
            String::from(TOKEN),
            String::from(CHAT_ID),
            String::from(url),
        );

        let result = client.send_message("test").unwrap();
        assert_eq!(result, ());
    }

    #[test]
    fn send_message_error() {
        let error = r#"{"ok":false,"error_code":400,"description":"Bad Request: chat not found"}"#;
        let url = &mockito::server_url();
        let _m = mockito::mock("POST", format!("/bot{}/sendMessage", TOKEN).as_str())
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(error)
            .create();

        let client = TelegramClient::new_with(
            String::from(TOKEN),
            String::from(CHAT_ID),
            String::from(url),
        );

        let result = client.send_message("test").unwrap_err();
        let result = format!("{}", result);
        assert_eq!(result, error);
    }

    #[test]
    fn send_image_success() {
        let url = &mockito::server_url();
        let _m = mockito::mock("POST", format!("/bot{}/sendPhoto", TOKEN).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .create();

        let client = TelegramClient::new_with(
            String::from(TOKEN),
            String::from(CHAT_ID),
            String::from(url),
        );

        let result = client.send_image("image url").unwrap();
        assert_eq!(result, ());
    }

    #[test]
    fn send_image_error() {
        let error = r#"{"ok":false,"error_code":400,"description":"Bad Request: chat not found"}"#;
        let url = &mockito::server_url();
        let _m = mockito::mock("POST", format!("/bot{}/sendPhoto", TOKEN).as_str())
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(error)
            .create();

        let client = TelegramClient::new_with(
            String::from(TOKEN),
            String::from(CHAT_ID),
            String::from(url),
        );

        let result = client.send_image("image url").unwrap_err();
        let result = format!("{}", result);
        assert_eq!(result, error);
    }
}
