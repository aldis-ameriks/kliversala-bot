use std::error::Error;

use async_trait::async_trait;

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
    type Source;

    fn new(url: &str) -> Self::Source;
    async fn fetch_posts(&self) -> Result<Vec<Post>, Box<dyn Error>>;
}
