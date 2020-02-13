use std::collections::HashMap;

use log::{debug, error, info};
use rusoto_core::{Region, RusotoError};
use rusoto_dynamodb::{
    AttributeValue, DeleteItemError, DeleteItemInput, DynamoDb, DynamoDbClient, GetItemError,
    GetItemInput, PutItemError, PutItemInput, ScanError, ScanInput,
};

use crate::posts::{Image, Post};

pub struct DynamoClient {
    client: DynamoDbClient,
    table_name: String,
}

impl DynamoClient {
    pub fn new(table_name: String) -> DynamoClient {
        let client = DynamoDbClient::new(Region::EuWest1);
        DynamoClient { client, table_name }
    }

    pub async fn get_post<'a>(&self, id: &str) -> Result<Option<Post>, RusotoError<GetItemError>> {
        let mut query_key: HashMap<String, AttributeValue> = HashMap::new();
        query_key.insert(
            String::from("id"),
            AttributeValue {
                s: Some(id.to_string()),
                ..Default::default()
            },
        );
        let get_item_input = GetItemInput {
            table_name: self.table_name.clone(),
            key: query_key,
            ..GetItemInput::default()
        };
        match self.client.get_item(get_item_input).await {
            Ok(output) => match output.item {
                Some(entry) => {
                    info!("get_post: Ok(id: {})", id);
                    let post = build_post(entry);
                    Ok(Some(post))
                }
                None => {
                    info!("get_post: post {} not found", id);
                    Ok(None)
                }
            },
            Err(error) => {
                error!("get_post: Error: {:?}", error);
                Err(error)
            }
        }
    }

    pub async fn put_post(&self, post: &Post) -> Result<(), RusotoError<PutItemError>> {
        debug!("put_post: {:?}", post);

        let mut query_key: HashMap<String, AttributeValue> = HashMap::new();
        query_key.insert(
            String::from("id"),
            AttributeValue {
                s: Some(post.id.to_string()),
                ..Default::default()
            },
        );
        if &post.text != "" {
            query_key.insert(
                String::from("text"),
                AttributeValue {
                    s: Some(post.text.to_string()),
                    ..Default::default()
                },
            );
        }
        if let Some(tg_id) = &post.tg_id {
            if tg_id != "" {
                query_key.insert(
                    String::from("message_id"),
                    AttributeValue {
                        s: Some(tg_id.clone()),
                        ..Default::default()
                    },
                );
            }
        }
        if post.images.len() > 0 {
            let mut urls = vec![];
            let mut ids = vec![];

            for image in &post.images {
                urls.push(String::from(&image.url));
                if let Some(val) = &image.tg_id {
                    ids.push(val.into());
                }
            }

            query_key.insert(
                String::from("images"),
                AttributeValue {
                    ss: Some(urls.clone()),
                    ..Default::default()
                },
            );
            query_key.insert(
                String::from("image_ids"),
                AttributeValue {
                    ss: Some(ids.clone()),
                    ..Default::default()
                },
            );
        }
        let put_item_input = PutItemInput {
            table_name: self.table_name.clone(),
            item: query_key,
            ..PutItemInput::default()
        };

        match self.client.put_item(put_item_input).await {
            Ok(_) => {
                info!("put_post: Ok(id: {})", post.id);
                Ok(())
            }
            Err(error) => {
                error!("put_post: Error: {:?}", error);
                Err(error)
            }
        }
    }

    pub async fn scan_posts(&self) -> Result<Vec<Post>, RusotoError<ScanError>> {
        let scan_input = ScanInput {
            table_name: self.table_name.clone(),
            ..ScanInput::default()
        };

        match self.client.scan(scan_input).await {
            Ok(res) => {
                info!("scan: Ok(count: {:?})", res.count);
                match res.items {
                    Some(result) => {
                        let mut posts = vec![];
                        for entry in result {
                            debug!("{:?}", entry);
                            let post = build_post(entry);
                            posts.push(post);
                        }
                        Ok(posts)
                    }
                    None => Ok(vec![]),
                }
            }
            Err(error) => {
                error!("scan: Error: {:?}", error);
                Err(error)
            }
        }
    }

    pub async fn delete_post(&self, id: &str) -> Result<(), RusotoError<DeleteItemError>> {
        let mut query_key: HashMap<String, AttributeValue> = HashMap::new();
        query_key.insert(
            String::from("id"),
            AttributeValue {
                s: Some(id.to_string()),
                ..Default::default()
            },
        );
        let delete_item_input = DeleteItemInput {
            table_name: self.table_name.clone(),
            key: query_key,
            ..DeleteItemInput::default()
        };

        match self.client.delete_item(delete_item_input).await {
            Ok(_) => {
                info!("delete_post: Ok(id: {})", id);
                Ok(())
            }
            Err(error) => {
                error!("delete_post: Error: {:?}", error);
                Err(error)
            }
        }
    }
}

fn build_post(entry: std::collections::HashMap<String, AttributeValue>) -> Post {
    let image_urls = if let Some(val) = entry.get("images") {
        val.ss.as_ref().unwrap().clone()
    } else {
        vec![]
    };

    let image_ids = if let Some(val) = entry.get("image_ids") {
        val.ss.as_ref().unwrap().clone()
    } else {
        vec![]
    };

    let mut images = Vec::new();
    for (url, tg_id) in image_urls.iter().zip(image_ids.iter()) {
        let image = Image {
            url: url.into(),
            tg_id: Some(tg_id.into()),
        };
        images.push(image);
    }

    let text = if let Some(val) = entry.get("text") {
        val.s.as_ref().unwrap()
    } else {
        ""
    };

    let tg_id = if let Some(val) = entry.get("message_id") {
        Some(String::from(val.s.as_ref().unwrap()))
    } else {
        None
    };

    Post {
        id: String::from(entry.get("id").unwrap().s.as_ref().unwrap()),
        text: String::from(text),
        images,
        tg_id,
    }
}
