use kliversala_bot::{dynamo_db, process_posts, telegram};
use std::env;

#[tokio::test]
async fn process_posts_success() {
    let token = env::var("TG_TOKEN").expect("Missing TG_TOKEN env var");
    let chat_id = env::var("TG_CHAT_ID").expect("Missing TG_CHAT_ID env var");
    let table_name = env::var("TABLE_NAME").expect("Missing TABLE_NAME env var");

    let dynamo_client = dynamo_db::DynamoClient::new(table_name);
    delete_all_posts(&dynamo_client).await;

    process_posts().await.unwrap();

    let existing_posts = dynamo_client.scan_posts().await.unwrap();
    assert_eq!(19, existing_posts.len());
    delete_all_posts(&dynamo_client).await;

    let telegram_client = telegram::TelegramClient::new(token, chat_id);
    for post in existing_posts {
        telegram_client
            .delete_message(&post.message_id.as_ref().unwrap())
            .await
            .expect("Failed to delete message");
        for image_id in &post.image_ids {
            telegram_client
                .delete_message(&image_id)
                .await
                .expect("Failed to delete image");
        }
    }
}

async fn delete_all_posts(client: &dynamo_db::DynamoClient) {
    let existing_posts = client.scan_posts().await.unwrap();
    for post in existing_posts {
        client.delete_post(&post.id).await.unwrap();
    }
    let existing_posts = client.scan_posts().await.unwrap();
    assert_eq!(0, existing_posts.len());
}
