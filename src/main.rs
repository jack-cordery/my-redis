use bytes::Bytes;
use mini_redis::{Connection, Frame, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let dbc = db.clone();
        println!("Accepted");
        tokio::spawn(async move {
            process(socket, dbc).await.unwrap();
        });
    }
}

async fn process(socket: TcpStream, db: Db) -> Result<()> {
    use mini_redis::{Command, Frame};
    let mut connection = Connection::new(socket);
    // so with the connection we want to have a db (hashmap)
    // we want to be able to read frames that store cmd from mini redis and we want
    // to essentially read set or get and then set or get from hashmap

    while let Some(frame) = connection.read_frame().await? {
        let response = match Command::from_frame(frame)? {
            Command::Set(cmd) => {
                db.lock()
                    .unwrap()
                    .insert(cmd.key().to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Command::Get(cmd) => {
                if let Some(value) = db.lock().unwrap().get(cmd.key()) {
                    Frame::Bulk(value.to_owned())
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
