use tokio::io;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8081").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            process(&mut socket).await;
        });
    }
}

async fn process(socket: &mut TcpStream) {
    let (mut read_socket, mut write_socket) = socket.split();
    if io::copy(&mut read_socket, &mut write_socket).await.is_err() {
        eprintln!("failed to copy");
    };
}
