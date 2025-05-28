use mini_redis::{Connection, Frame, Result};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket).await.unwrap();
        });
    }
}

async fn process(socket: TcpStream) -> Result<()> {
    use mini_redis::{Command, Frame};
    use std::collections::HashMap;
    let mut connection = Connection::new(socket);
    // so with the connection we want to have a db (hashmap)
    // we want to be able to read frames that store cmd from mini redis and we want
    // to essentially read set or get and then set or get from hashmap
    let mut db = HashMap::new();

    while let Some(frame) = connection.read_frame().await? {
        let response = match Command::from_frame(frame)? {
            Command::Set(cmd) => {
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Command::Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            _ => panic!("not implemented"),
        };
        connection.write_frame(&response).await?;
    }

    Ok(())
}
