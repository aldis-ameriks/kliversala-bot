use std::collections::HashMap;
use std::error::Error;

use rusoto_core::Region;
use rusoto_dynamodb::{AttributeValue, DynamoDb, DynamoDbClient, GetItemInput, PutItemInput};

use crate::posts::Post;

// TODO: Read table name from env vars
static TABLE_NAME: &str = "posts-dev";

pub fn get_post(id: String) -> Result<Option<String>, Box<dyn Error>> {
    let client = DynamoDbClient::new(Region::EuWest1);
    let mut query_key: HashMap<String, AttributeValue> = HashMap::new();
    query_key.insert(String::from("id"), AttributeValue { s: Some(id.clone()), ..Default::default() });
    let get_item_input = GetItemInput { table_name: TABLE_NAME.to_string(), key: query_key, ..GetItemInput::default() };
    match client.get_item(get_item_input).sync() {
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
            Err(error.to_string().into())
        }
    }
}

pub fn put_post(post: &Post) -> Result<(), Box<dyn Error>> {
    let client = DynamoDbClient::new(Region::EuWest1);

    let mut query_key: HashMap<String, AttributeValue> = HashMap::new();
    query_key.insert(String::from("id"), AttributeValue { s: Some(String::from(&post.id)), ..Default::default() });
    query_key.insert(String::from("text"), AttributeValue { s: Some(String::from(&post.text)), ..Default::default() });
    let put_item_input = PutItemInput { table_name: TABLE_NAME.to_string(), item: query_key, ..PutItemInput::default() };

    match client.put_item(put_item_input).sync() {
        Ok(_) => {
            println!("put_item: OK");
            Ok(())
        }
        Err(error) => {
            println!("put_item: Error: {:?}", error);
            Err(error.to_string().into())
        }
    }
}
