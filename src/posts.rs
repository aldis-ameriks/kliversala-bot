use std::error::Error;

use html2md::parse_html;
use log::{debug, info};
use reqwest::blocking::Client;
use scraper::{Html, Selector};

#[derive(Debug)]
pub struct Post {
    pub id: String,
    pub text: String,
    pub images: Vec<String>,
}

const POSTS_SELECTOR: &str = "#pagelet_timeline_main_column > div:first-of-type > div:nth-child(2) > div:first-of-type > div";
const IMAGE_CONTAINER_SELECTOR: &str = concat!(
    "#pagelet_timeline_main_column > div:first-of-type > div:nth-child(2) > div:first-of-type > div",
    " > div > div > div > div > div > div > div:not(:nth-child(1))"
);
const ID_SELECTOR: &str = r#"div[data-testid="story-subtitle"]"#;
const TEXT_SELECTOR: &str = r#"div[data-testid="post_message"]"#;
const IMAGE_SELECTOR: &str = "img";

pub fn fetch_posts() -> Result<Vec<Post>, Box<dyn Error>> {
    let url = "https://www.facebook.com/pg/kantineKliversala/posts/";
    let resp = Client::builder()
        .build()?
        .get(url)
        .header("user-agent", "rusty")
        .send()?;
    assert!(resp.status().is_success());
    let res_text = resp.text()?;
    let mut result: Vec<Post> = Vec::new();

    let document = Html::parse_document(&res_text);
    let posts_selector = Selector::parse(POSTS_SELECTOR).unwrap();
    let id_selector = Selector::parse(ID_SELECTOR).unwrap();
    let text_selector = Selector::parse(TEXT_SELECTOR).unwrap();
    let image_container_selector = Selector::parse(IMAGE_CONTAINER_SELECTOR).unwrap();
    let image_selector = Selector::parse(IMAGE_SELECTOR).unwrap();

    for post in document.select(&posts_selector) {
        let mut post_id = "";
        for id_element in post.select(&id_selector) {
            post_id = id_element.value().id().unwrap();
        }

        if post_id == "" {
            continue;
        }

        let post_id: &str = post_id.split(";").collect::<Vec<&str>>()[1];
        info!("post_id: {}", post_id);

        let mut text_parts: Vec<String> = Vec::new();

        for text in post.select(&text_selector) {
            text_parts.push(text.inner_html());
        }
        let text = text_parts.concat();
        let parsed_text = parse_html(&text);

        let mut images: Vec<String> = Vec::new();
        for img_container in post.select(&image_container_selector) {
            for img_element in img_container.select(&image_selector) {
                let img_src = img_element.value().attr("src").unwrap();
                info!("img src: {}", img_src);
                images.push(String::from(img_src));
            }
        }

        debug!("parsed html into markdown: {}", parsed_text);

        let post = Post {
            id: format!("{}", post_id).replace("\"", ""),
            text: parsed_text.replace("\\-", "-"),
            images,
        };

        result.push(post);
    }

    Ok(result)
}
