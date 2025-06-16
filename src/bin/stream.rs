use mini_redis::client;
use tokio::time::{Duration, sleep};
use tokio_stream::StreamExt;

async fn publish() -> mini_redis::Result<()> {
    let mut connect = client::connect("127.0.0.1:6379").await?;

    connect.publish("numbers", "one".into()).await?;
    connect.publish("numbers", "two".into()).await?;
    connect.publish("numbers", "three".into()).await?;
    connect.publish("numbers", "1".into()).await?;
    connect.publish("numbers", "2".into()).await?;
    connect.publish("numbers", "3".into()).await?;
    connect.publish("numbers", "10".into()).await?;

    Ok(())
}

async fn subscribe() -> mini_redis::Result<()> {
    let connect = client::connect("127.0.0.1:6379").await?;
    let subscriber = connect.subscribe(vec!["numbers".to_string()]).await?;

    let messages = subscriber
        .into_stream()
        .filter(|msg| matches!(msg, Ok(msg) if msg.content.len() == 1))
        .take(3)
        .map(|msg| msg.unwrap().content);

    tokio::pin!(messages);

    while let Some(msg) = messages.next().await {
        println!("I got some published messages on the numbers channel {msg:?}")
    }
    Ok(())
}

#[tokio::main]
async fn main() -> mini_redis::Result<()> {
    tokio::spawn(async {
        sleep(Duration::from_millis(1)).await;
        publish().await.unwrap();
    });

    subscribe().await?;
    println!("Done");
    Ok(())
}
