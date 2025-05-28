use mini_redis::{Result, client};

#[tokio::main]
async fn main() -> Result<()> {
    let mut connection = client::connect("127.0.0.1:6379").await?;

    connection.set("hello", "world".into()).await?;

    let result = connection.get("hello").await?;

    println!("I got some results yo: {:?}", result);

    Ok(())
}
