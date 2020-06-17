use std::env;

use log::Level;

use annasdarzs_bot::{dynamo_db::DynamoClient, process_posts, sources, telegram::client::TelegramClient};
use std::thread::sleep;
use std::time::Duration;

#[tokio::test]
async fn process_posts_success() {
    simple_logger::init_with_level(Level::Info).expect("Failed to init logger");
    let token = env::var("TG_TOKEN").expect("Missing TG_TOKEN env var");
    let chat_id = env::var("TG_CHAT_ID").expect("Missing TG_CHAT_ID env var");
    let table_name = env::var("TABLE_NAME").expect("Missing TABLE_NAME env var");

    let dynamo_client = DynamoClient::new(table_name);
    let telegram_client = TelegramClient::new(token, chat_id);

    let posts = dynamo_client.scan_posts().await.unwrap();
    delete_posts(&dynamo_client, &posts).await;

    process_posts().await.unwrap();
    let posts = dynamo_client.scan_posts().await.unwrap();
    assert_eq!(19, posts.len());

    // Running second time skips posts that are already sent
    process_posts().await.unwrap();
    let posts = dynamo_client.scan_posts().await.unwrap();
    assert_eq!(19, posts.len());

    delete_posts(&dynamo_client, &posts).await;
    delete_messages(&telegram_client, &posts).await;
}

async fn delete_posts(client: &DynamoClient, posts: &[sources::Post]) {
    for post in posts {
        client.delete_post(&post.id).await.unwrap();
        sleep(Duration::from_millis(200)); // Throttling to avoid hitting limits
    }
    let posts = client.scan_posts().await.unwrap();
    assert_eq!(0, posts.len());
}

async fn delete_messages(client: &TelegramClient, posts: &[sources::Post]) {
    for post in posts {
        client
            .delete_message(&post.tg_id.as_ref().unwrap())
            .await
            .expect("Failed to delete message");
        for sources::Image { url: _url, tg_id } in &post.images {
            client
                .delete_message(&tg_id.as_ref().unwrap())
                .await
                .expect("Failed to delete image");
        }
    }
}
