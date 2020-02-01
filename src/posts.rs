use std::error::Error;

use html2md::parse_html;
use log::{debug};
use scraper::{Html, Selector};

#[derive(Debug)]
pub struct Post {
    pub id: String,
    pub text: String,
}

pub fn fetch_posts(url: &str) -> Result<Vec<Post>, Box<dyn Error>> {
    let resp = reqwest::blocking::Client::builder()
        .build()?
        .get(url)
        .header("user-agent", "rusty")
        .send()?;

    // TODO: Avoid using panicky assert
    assert!(resp.status().is_success());

    let res_text = resp.text()?;
    let document = Html::parse_document(&res_text);
    // TODO: Avoid using panicky unwraps
    let selector =
        Selector::parse("#recent > div:first-of-type > div:first-of-type > div").unwrap();
    let inner_text_selector =
        Selector::parse("div:first-of-type > div:nth-child(2) > span").unwrap();

    let mut result: Vec<Post> = Vec::new();
    for element in document.select(&selector) {
        let data_attribute = element.value().attr("data-ft").unwrap();
        let data_attribute: serde_json::Value = serde_json::from_str(data_attribute)?;
        let post_id = &data_attribute["mf_story_key"];
        debug!("post_id: {}", post_id);

        let mut inner_texts: Vec<String> = Vec::new();
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
