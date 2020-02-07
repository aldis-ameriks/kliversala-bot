use std::error::Error;
use std::{
    fs::File,
    io::{BufWriter, Write},
};

use html2md::parse_html;
use log::{debug, info};
use reqwest::Client;
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

pub async fn fetch_posts(url: &str) -> Result<Vec<Post>, Box<dyn Error>> {
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
            text: parsed_text
                .replace("\\-", "-")
                .replace("...", "")
                .replace(
                    format!("[See more](/kantineKliversala/posts/{})", post_id).as_str(),
                    "",
                )
                .replace(
                    format!("[See More](/kantineKliversala/posts/{})", post_id).as_str(),
                    "",
                ),
            images,
        };

        result.push(post);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use mockito::{mock, server_url, Matcher};

    use super::*;

    #[tokio::test]
    async fn fetch_posts_success() {
        let url = &server_url();
        let _m = mock("GET", "/pg/kantineKliversala/posts/")
            .with_status(200)
            .with_body_from_file("mock_response.html")
            .create();

        let result = fetch_posts(format!("{}/pg/kantineKliversala/posts/", url).as_str())
            .await
            .unwrap();
        assert_eq!(result.len(), 19);
        assert_eq!(result[0].id, "2471140943148075");
        assert_eq!(result[0].text, "Pusdienu piedāvājums 7. februārī.  \n Dienas piedāvājums pieejams 11:00-16:00\n\n Mazais pusdienu piedāvājums:  \n🍗v/g saldakābā mērcē vai 🥘makaroni \"Jūrnieku gaumē\", vai 🌽kuskuss ar dārzeņiem  \n🥒 dienas salāti  \n🍷 dzērveņu dzēriens   \n💸 3,90€\n\n Lielais pusdienu piedāvājums:  \n🍲frikadeļu zupa vai dārzeņu krēmzupa, vai 🍰 dienas deserts  \n🍗v/g saldskābā mērcē vai 🥘makaroni \"Jūrnieku gaumē\", vai 🌽kuskuss ar dārzeņiem  \n🥒 dienas salāti  \n🍷 dzērveņu dzēriens   \n💸 4,60€\n\n Labu apetīti!\n\n[Skatīt vairāk]()\n\n");
        let images: Vec<String> = Vec::new();
        assert_eq!(result[0].images, images);

        assert_eq!(result[5].id, "2465890140339822");
        assert_eq!(
            result[5].text,
            "Nāc un piedalies arī Tu, jau no 01.02.2020! 🥘🍴☕\n\n"
        );
        assert_eq!(result[5].images, vec![String::from("https://scontent.frix3-1.fna.fbcdn.net/v/t1.0-0/p526x296/84437983_2465890103673159_2752238738611372032_o.jpg?_nc_cat=106&_nc_ohc=YlgO1JJVbLQAX8aROMV&_nc_ht=scontent.frix3-1.fna&_nc_tp=6&oh=a9a1e00cf9bf5ce65254d36f7ef27590&oe=5EC77203")]);
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
    }
}
