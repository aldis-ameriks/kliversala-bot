use std::env;
use std::error::Error;
use std::thread;
use std::time::Duration;

use log::{debug, error, info};
use rusqlite::{Connection, NO_PARAMS};
use scraper::{Html, Selector};
use serde::{Serialize};
use html2md::parse_html;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    init_database()?;

    loop {
        match process_posts() {
            Ok(()) => info!("successfully processed posts"),
            Err(e) => error!("error occurred in post processor: {}", e),
        }
        thread::sleep(Duration::from_secs(3600));
    }
}

fn init_database() -> rusqlite::Result<()> {
    let conn = open_database_connection()?;

    debug!("initializing database");
    conn.execute(
        "
            CREATE TABLE IF NOT EXISTS posts
            (
                id      varchar(24) PRIMARY KEY NOT NULL,
                text    text,
                sent_at timestamptz DEFAULT CURRENT_TIMESTAMP
            );
         ",
        NO_PARAMS,
    )?;
    debug!("database initialized");
    Ok(())
}

fn open_database_connection() -> rusqlite::Result<Connection> {
    Connection::open("kliversala.db")
}

fn process_posts() -> Result<(), Box<dyn Error>> {
    let conn = open_database_connection()?;
    let posts = fetch_posts("https://mobile.facebook.com/kantineKliversala/posts/")?;

    for post in posts {
        debug!("processing post_id: {}", post.id);

        let mut stmt = conn.prepare("SELECT * FROM posts WHERE id = ?;")?;

        let found_posts = stmt.query_map(&[&post.id], |row| {
            Ok(Post {
                id: row.get(0)?,
                text: row.get(1)?,
            })
        })?;

        let mut posts_count = 0;
        for found_post in found_posts {
            match found_post {
                Ok(_) => posts_count += 1,
                Err(e) => error!("post errored: {:#?}", e),
            }
        }

        debug!("posts_count: {}", posts_count);

        if posts_count == 0 {
            info!("sending new post {}", post.id);
            conn.execute(
                "INSERT INTO posts (id, text) values (?, ?)",
                &[&post.id, &post.text],
            )?;

            match send_message( post.text.clone()) {
                Err(e) => error!("failed to send message {}", e),
                Ok(_) => continue,
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct Post {
    id: String,
    text: String,
}

fn fetch_posts(url: &str) -> Result<Vec<Post>, Box<dyn Error>> {
    let resp = reqwest::blocking::Client::builder()
        .build()?
        .get(url)
        .header("user-agent", "rusty")
        .send()?;

    assert!(resp.status().is_success());

    let res_text = resp.text()?;
    let document = Html::parse_document(&res_text);
    let selector =
        Selector::parse("#recent > div:first-of-type > div:first-of-type > div:first-of-type").unwrap();
    let inner_text_selector =
        Selector::parse("div:nth-child(1) > div:nth-child(2) > span").unwrap();

    let mut result: Vec<Post> = vec![];
    for element in document.select(&selector) {
        let data_attribute = element.value().attr("data-ft").unwrap();
        let data_attribute: serde_json::Value = serde_json::from_str(data_attribute)?;
        let post_id = &data_attribute["mf_story_key"];
        debug!("post_id: {}", post_id);

        let mut inner_texts: Vec<String> = vec![];
        let inner_text_elements = element.select(&inner_text_selector);

        for inner_text_element in inner_text_elements {
            let inner_text = inner_text_element.inner_html();
            debug!("{:#?}", inner_text);
            inner_texts.push(inner_text);
        }
        let inner_text = inner_texts.concat();

        let inner_text = parse_html(&inner_text);

        debug!("parsed html into markdown: {}", inner_text);

        let post = Post {
            id: format!("{}", post_id).replace("\"", ""),
            text: inner_text.replace("\\-", "-"),
        };
        result.push(post)
    }

    Ok(result)
}

#[derive(Debug, Serialize)]
struct Message {
    chat_id: String,
    text: String,
    disable_notification: bool,
}

fn send_message(text: String) -> Result<(), Box<dyn Error>> {
    let token = env::var("TG_TOKEN")?;
    let message = Message {
        chat_id: String::from("@kliversala"),
        text: String::from(text),
        disable_notification: true,
    };

    info!("sending message: {:#?}", message);

    let _resp = reqwest::blocking::Client::builder().build()?.post(
        &format!("https://api.telegram.org/bot{}/sendMessage", token),
    ).json(&message).send()?;

    Ok(())
}
