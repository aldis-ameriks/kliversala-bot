use kliversala_bot::{dynamo_db, process_posts, telegram};
use std::env;

#[tokio::test]
async fn process_posts_success() {
    env::set_var("TG_CHAT_ID", "900963193");
    env::set_var("TABLE_NAME", "posts-dev");
    let dynamo_client = dynamo_db::DynamoClient::new(String::from("posts-dev"));
    delete_all_posts(&dynamo_client).await;

    process_posts().await.unwrap();

    let existing_posts = dynamo_client.scan_posts().await.unwrap();
    assert_eq!(19, existing_posts.len());
    delete_all_posts(&dynamo_client).await;

    let telegram_client = telegram::TelegramClient::new(
        env::var("TG_TOKEN").unwrap(),
        String::from("900963193"),
    );
    for post in existing_posts {
        telegram_client
            .delete_message(&post.message_id.unwrap())
            .await
            .expect("Failed to delete message");
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
