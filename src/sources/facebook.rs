use std::error::Error;

use html2md::parse_html;
use log::{debug, info};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};

use async_trait::async_trait;

use crate::sources::{Image, Post, PostSource};

const POSTS_SELECTOR: &str = "#pagelet_timeline_main_column > div:first-of-type > div:nth-child(2) > div:first-of-type > div";
const IMAGE_CONTAINER_SELECTOR: &str = concat!(
"#pagelet_timeline_main_column > div:first-of-type > div:nth-child(2) > div:first-of-type > div",
" > div > div > div > div > div > div > div:not(:nth-child(1))"
);
const ID_SELECTOR: &str = r#"div[data-testid="story-subtitle"]"#;
const TEXT_SELECTOR: &str = r#"div[data-testid="post_message"] > *:first-child"#;
const IMAGE_SELECTOR: &str = "img";

pub struct FacebookSource {
    url: String,
}

#[async_trait]
impl PostSource for FacebookSource {
    type Source = FacebookSource;

    fn new(url: &str) -> FacebookSource {
        FacebookSource {
            url: String::from(url),
        }
    }
    async fn fetch_posts(&self) -> Result<Vec<Post>, Box<dyn Error>> {
        fetch_posts(&self.url).await
    }
}

async fn fetch_posts(url: &str) -> Result<Vec<Post>, Box<dyn Error>> {
    let resp = Client::new()
        .get(url)
        .header("user-agent", "rusty")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(resp.text().await?.into());
    }

    let res_text = resp.text().await?;
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

        info!("post_id: {}", post_id);
        let post_id: Vec<&str> = post_id.split(";").collect::<Vec<&str>>();
        if post_id.len() < 2 {
            continue;
        }
        let post_id = post_id[1];

        let mut text_parts: Vec<String> = Vec::new();

        for text in post.select(&text_selector) {
            text_parts.push(text.inner_html());
        }
        let text = text_parts.concat();
        let parsed_text = parse_html(&text);

        let mut images: Vec<Image> = Vec::new();
        for img_container in post.select(&image_container_selector) {
            for img_element in img_container.select(&image_selector) {
                let img_src = img_element.value().attr("src").unwrap();
                info!("img src: {}", img_src);
                let image = Image {
                    url: String::from(img_src),
                    tg_id: None,
                };
                images.push(image);
            }
        }

        debug!("parsed html into markdown: {}", parsed_text);

        let parsed_text = parsed_text
            .replace("\\-", "-")
            .replace("...", "")
            .replace("See More", "")
            .replace(
                format!("[See more](/PusdienotavaAnnasDarzs/posts/{})", post_id).as_str(),
                "",
            )
            .replace(
                format!("[See More](/PusdienotavaAnnasDarzs/posts/{})", post_id).as_str(),
                "",
            );

        let parsed_text = remove_markdown_links(&parsed_text);

        let post = Post {
            id: format!("{}", post_id).replace("\"", ""),
            text: parsed_text,
            images,
            tg_id: None,
        };

        result.push(post);
    }

    Ok(result)
}

fn remove_markdown_links(text: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\[(.*?)\]\(.*?\)").unwrap();
    }
    return String::from(RE.replace_all(text, "$1"));
}

#[cfg(test)]
mod tests {
    use mockito::{mock, server_url};

    use super::*;

    #[tokio::test]
    async fn fetch_posts_success() {
        let url = &server_url();
        let _m = mock("GET", "/pg/kantineKliversala/posts/")
            .with_status(200)
            .with_body_from_file("_mock_response")
            .create();

        let result = fetch_posts(format!("{}/pg/kantineKliversala/posts/", url).as_str())
            .await
            .unwrap();
        assert_eq!(result.len(), 19);
        assert_eq!(result[0].id, "2471140943148075");
        assert_eq!(result[0].text, "Pusdienu piedāvājums 7. februārī.  \n Dienas piedāvājums pieejams 11:00-16:00\n\n Mazais pusdienu piedāvājums:  \n🍗v/g saldakābā mērcē vai 🥘makaroni \"Jūrnieku gaumē\", vai 🌽kuskuss ar dārzeņiem  \n🥒 dienas salāti  \n🍷 dzērveņu dzēriens   \n💸 3,90€\n\n Lielais pusdienu piedāvājums:  \n🍲frikadeļu zupa vai dārzeņu krēmzupa, vai 🍰 dienas deserts  \n🍗v/g saldskābā mērcē vai 🥘makaroni \"Jūrnieku gaumē\", vai 🌽kuskuss ar dārzeņiem  \n🥒 dienas salāti  \n🍷 dzērveņu dzēriens   \n💸 4,60€\n\n Labu apetīti!\n\nSkatīt vairāk");
        assert_eq!(result[0].images.len(), 0);

        assert_eq!(result[5].id, "2465890140339822");
        assert_eq!(
            result[5].text,
            "Nāc un piedalies arī Tu, jau no 01.02.2020! 🥘🍴☕"
        );
        assert_eq!(result[5].images[0].url, String::from("https://scontent.frix3-1.fna.fbcdn.net/v/t1.0-0/p526x296/84437983_2465890103673159_2752238738611372032_o.jpg?_nc_cat=106&_nc_ohc=YlgO1JJVbLQAX8aROMV&_nc_ht=scontent.frix3-1.fna&_nc_tp=6&oh=a9a1e00cf9bf5ce65254d36f7ef27590&oe=5EC77203"));
        _m.assert();
    }

    #[tokio::test]
    async fn fetch_posts_empty_html() {
        let url = &server_url();
        let _m = mock("GET", "/pg/kantineKliversala/posts/")
            .with_status(200)
            .with_body("<html><body><div>empty</div></body></html>")
            .create();

        let result = fetch_posts(format!("{}/pg/kantineKliversala/posts/", url).as_str())
            .await
            .unwrap();
        assert_eq!(result.len(), 0);
        _m.assert();
    }

    #[tokio::test]
    async fn fetch_posts_corrupt_html() {
        let url = &server_url();
        let _m = mock("GET", "/pg/kantineKliversala/posts/")
            .with_status(200)
            .with_body("something")
            .create();

        let result = fetch_posts(format!("{}/pg/kantineKliversala/posts/", url).as_str())
            .await
            .unwrap();
        assert_eq!(result.len(), 0);
        _m.assert();
    }

    #[tokio::test]
    async fn fetch_posts_error() {
        let url = &server_url();
        let _m = mock("GET", "/pg/kantineKliversala/posts/")
            .with_status(400)
            .with_body("error")
            .create();

        let result = fetch_posts(format!("{}/pg/kantineKliversala/posts/", url).as_str())
            .await
            .unwrap_err();
        let result = format!("{}", result);
        assert_eq!(result, "error");
        _m.assert();
    }

    #[test]
    fn remove_markdown_links_single_works() {
        let test_string = r#"test [Skatīt vairāk](/kantineKliversala/posts/2457708144491355)"#;
        let result = r#"test Skatīt vairāk"#;
        assert_eq!(result, remove_markdown_links(test_string));
    }

    #[test]
    fn remove_markdown_links_multiple_works() {
        let test_string = r#"test [Skatīt vairāk](/kantineKliversala/posts/2457708144491355) [Skatīt vairāk](/kantineKliversala/posts/2457708144491355)"#;
        let result = r#"test Skatīt vairāk Skatīt vairāk"#;
        assert_eq!(result, remove_markdown_links(test_string));
    }

    #[test]
    fn remove_markdown_links_works_without_links() {
        let test_string = r#"test Skatīt vairāk"#;
        let result = r#"test Skatīt vairāk"#;
        assert_eq!(result, remove_markdown_links(test_string));
    }
}
