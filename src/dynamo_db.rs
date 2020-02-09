use std::collections::HashMap;

use log::{error, info};
use rusoto_core::{Region, RusotoError};
use rusoto_dynamodb::{
    AttributeValue, DynamoDb, DynamoDbClient, GetItemError, GetItemInput, PutItemError,
    PutItemInput,
};

use crate::posts::Post;

pub struct DynamoClient {
    client: DynamoDbClient,
    table_name: String,
}

impl DynamoClient {
    pub fn new(table_name: String) -> DynamoClient {
        let client = DynamoDbClient::new(Region::EuWest1);
        DynamoClient { client, table_name }
    }

    pub async fn get_post<'a>(
        &self,
        id: &'a str,
    ) -> Result<Option<&'a str>, RusotoError<GetItemError>> {
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
                Some(_) => {
                    info!("get_item: Ok(id: {})", id);
                    Ok(Some(id))
                }
                None => {
                    info!("get_item: item {} not found", id);
                    Ok(None)
                }
            },
            Err(error) => {
                error!("get_item: Error: {:?}", error);
                Err(error)
            }
        }
    }

    pub async fn put_post(&self, post: &Post) -> Result<(), RusotoError<PutItemError>> {
        let mut query_key: HashMap<String, AttributeValue> = HashMap::new();
        query_key.insert(
            String::from("id"),
            AttributeValue {
                s: Some(post.id.to_string()),
                ..Default::default()
            },
        );
        query_key.insert(
            String::from("text"),
            AttributeValue {
                s: Some(post.text.to_string()),
                ..Default::default()
            },
        );
        query_key.insert(
            String::from("message_id"),
            AttributeValue {
                s: Some(post.message_id.clone().unwrap()),
                ..Default::default()
            },
        );
        if post.images.len() > 0 {
            query_key.insert(
                String::from("images"),
                AttributeValue {
                    ss: Some(post.images.clone()),
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
                info!("put_item: OK(id: {})", post.id);
                Ok(())
            }
            Err(error) => {
                error!("put_item: Error: {:?}", error);
                Err(error)
            }
        }
    }
}
