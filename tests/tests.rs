use log::Level;
use std::env;

use kliversala_bot::{dynamo_db, sources, process_posts, telegram};

#[tokio::test]
async fn process_posts_success() {
    simple_logger::init_with_level(Level::Info).expect("Failed to init logger");
    let token = env::var("TG_TOKEN").expect("Missing TG_TOKEN env var");
    let chat_id = env::var("TG_CHAT_ID").expect("Missing TG_CHAT_ID env var");
    let table_name = env::var("TABLE_NAME").expect("Missing TABLE_NAME env var");

    let dynamo_client = dynamo_db::DynamoClient::new(table_name);
    let telegram_client = telegram::TelegramClient::new(token, chat_id);

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

async fn delete_posts(client: &dynamo_db::DynamoClient, posts: &[sources::Post]) {
    for post in posts {
        client.delete_post(&post.id).await.unwrap();
    }
    let posts = client.scan_posts().await.unwrap();
    assert_eq!(0, posts.len());
}

async fn delete_messages(client: &telegram::TelegramClient, posts: &[sources::Post]) {
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
