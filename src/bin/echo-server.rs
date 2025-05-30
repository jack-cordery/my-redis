use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8081").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            while process(&mut socket).await.is_ok() {
                continue;
            }
            println!("graceful shutdown");
        });
    }
}

// ok so we now want to understand how to manually do a read and write instead of using copy
// the good news is that we dont need to use split, although this implementation of a split
// in one thread is actually zero cost anyway
// what we will need to do is store the read in some kind of buffer that the writer can then
// access
async fn process(_socket: &mut TcpStream) -> io::Result<()> {
    let mut buf = vec![0; 1024];
    let n;
    match _socket.read(&mut buf).await {
        Ok(0) => {
            println!("socket returned 0 that means that a EOF was seen");
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "end of connection",
            ));
        }
        Ok(m) => {
            n = m;
            println!("socket read {n} bytes to the buffer");
        }
        Err(e) => {
            return Err(e);
        }
    };
    _socket.write_all(&buf[..n]).await?;
    Ok(())
}
