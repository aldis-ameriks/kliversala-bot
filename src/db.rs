use std::collections::HashMap;
use std::env;
use std::error::Error;

use rusoto_core::{Region, RusotoError};
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, GetItemError, GetItemInput, PutItemError, PutItemInput};

use crate::posts::Post;

pub struct Client {
    client: DynamoDbClient,
    table_name: String,
}

impl Client {
    pub fn new() -> Client {
        let table_name = env::var("TABLE_NAME").expect("Missing TABLE_NAME env var");
        let client = DynamoDbClient::new(Region::EuWest1);
        Client { client, table_name }
    }

    pub fn get_post(&self, id: String) -> Result<Option<String>, RusotoError<GetItemError>> {
        let mut query_key: HashMap<String, AttributeValue> = HashMap::new();
        query_key.insert(String::from("id"), AttributeValue { s: Some(id.clone()), ..Default::default() });
        let get_item_input = GetItemInput { table_name: self.table_name.clone(), key: query_key, ..GetItemInput::default() };
        match self.client.get_item(get_item_input).sync() {
            Ok(output) => return match output.item {
                Some(item) => {
                    println!("get_item: OK: {:#?}", item);
                    Ok(Some(id.clone()))
                }
                None => {
                    println!("get_item: item {} not found", id);
                    Ok(None)
                }
            },
            Err(error) => {
                println!("get_item: Error: {:?}", error);
                Err(error)
            }
        }
    }

    pub fn put_post(&self, post: &Post) -> Result<(), RusotoError<PutItemError>> {
        let mut query_key: HashMap<String, AttributeValue> = HashMap::new();
        query_key.insert(String::from("id"), AttributeValue { s: Some(String::from(&post.id)), ..Default::default() });
        query_key.insert(String::from("text"), AttributeValue { s: Some(String::from(&post.text)), ..Default::default() });
        let put_item_input = PutItemInput { table_name: self.table_name.clone(), item: query_key, ..PutItemInput::default() };

        match self.client.put_item(put_item_input).sync() {
            Ok(_) => {
                println!("put_item: OK");
                Ok(())
            }
            Err(error) => {
                println!("put_item: Error: {:?}", error);
                Err(error)
            }
        }
    }
}
