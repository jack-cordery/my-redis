use tokio::fs::File;

use tokio::io::{self, AsyncWriteExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let content = b"bello\n".repeat(100_000_000);
    let reader: &[u8] = &content;

    let mut file = File::create("foo.txt").await?;

    file.write_all(reader).await?;

    println!("write completed");

    Ok(())
}
