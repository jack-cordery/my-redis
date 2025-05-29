use bytes::Bytes;
use mini_redis::{Connection, Result};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;
type ShardedDb = Arc<Vec<Mutex<HashMap<String, Vec<u8>>>>>;

fn new_sharded_db(num_shards: usize) -> ShardedDb {
    let mut db = Vec::with_capacity(num_shards);
    for _ in 0..num_shards {
        db.push(Mutex::new(HashMap::new()));
    }
    Arc::new(db)
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let db: ShardedDb = new_sharded_db(4);
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let dbc = db.clone();
        println!("Accepted");
        tokio::spawn(async move {
            sharded_process(socket, dbc).await.unwrap();
        });
    }
}

async fn sharded_process(socket: TcpStream, db: ShardedDb) -> Result<()> {
    use mini_redis::{Command, Frame};
    let mut connection = Connection::new(socket);
    // so with the connection we want to have a db (hashmap)
    // we want to be able to read frames that store cmd from mini redis and we want
    // to essentially read set or get and then set or get from hashmap

    while let Some(frame) = connection.read_frame().await? {
        let response = match Command::from_frame(frame)? {
            Command::Set(cmd) => {
                let mut state = DefaultHasher::new();
                cmd.key().hash(&mut state);
                let shard_index: usize = (state.finish() % (db.len() as u64)) as usize;
                let shard = db[shard_index].lock();
                shard
                    .unwrap()
                    .insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Command::Get(cmd) => {
                let mut state = DefaultHasher::new();
                cmd.key().hash(&mut state);
                let shard_index: usize = (state.finish() % (db.len() as u64)) as usize;
                let shard = db[shard_index].lock();
                if let Some(value) = shard.unwrap().get(cmd.key()) {
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
