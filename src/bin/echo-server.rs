use tokio::fs::File;
use tokio::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let content = b"bello".repeat(100_000_000);
    let mut reader: &[u8] = &content;

    let mut file = File::create("foo.txt").await?;

    let b = io::copy(&mut reader, &mut file).await?;

    println!("copy completed having copied {b} bytes");

    Ok(())
}
