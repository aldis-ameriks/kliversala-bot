use std::env;

use kliversala_bot::{dynamo_db, posts, process_posts, telegram};

#[tokio::test]
async fn process_posts_success() {
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
    delete_posts(&dynamo_client, &posts).await;
    delete_messages(&telegram_client, &posts).await;
}

async fn delete_posts(client: &dynamo_db::DynamoClient, posts: &[posts::Post]) {
    for post in posts {
        client.delete_post(&post.id).await.unwrap();
    }
    let posts = client.scan_posts().await.unwrap();
    assert_eq!(0, posts.len());
}

async fn delete_messages(client: &telegram::TelegramClient, posts: &[posts::Post]) {
    for post in posts {
        client
            .delete_message(&post.message_id.as_ref().unwrap())
            .await
            .expect("Failed to delete message");
        for image_id in &post.image_ids {
            client
                .delete_message(&image_id)
                .await
                .expect("Failed to delete image");
        }
    }
}
