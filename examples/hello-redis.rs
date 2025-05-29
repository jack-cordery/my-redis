use mini_redis::{Result, client};

#[tokio::main]
async fn main() -> Result<()> {
    let mut connection = client::connect("127.0.0.1:6379").await?;

    connection.set("hello", "world".into()).await?;

    let result = connection.get("hello").await?;

    println!("I got some results yo: {:?}", result);

    let mut connection_another = client::connect("127.0.0.1:6379").await?;
    let result = connection_another.get("hello").await?;
    println!(
        "I also got your result from a shared and sharded state yo: {:?}",
        result
    );

    Ok(())
}
