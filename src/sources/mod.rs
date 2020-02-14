use std::error::Error;

use async_trait::async_trait;
use facebook::fetch_posts;

pub mod facebook;

#[derive(Debug)]
pub struct Post {
    pub id: String,
    pub tg_id: Option<String>,
    pub text: String,
    pub images: Vec<Image>,
}

#[derive(Debug)]
pub struct Image {
    pub url: String,
    pub tg_id: Option<String>,
}

#[async_trait]
pub trait PostSource {
    async fn fetch_posts(&self) -> Result<Vec<Post>, Box<dyn Error>>;
}

pub struct FacebookSource {
    url: String,
}

impl FacebookSource {
    pub fn new(url: &str) -> FacebookSource {
        FacebookSource {
            url: String::from(url),
        }
    }
}

#[async_trait]
impl PostSource for FacebookSource {
    async fn fetch_posts(&self) -> Result<Vec<Post>, Box<dyn Error>> {
        fetch_posts(&self.url).await
    }
}
