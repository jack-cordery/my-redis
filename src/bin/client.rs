use bytes::Bytes;
use mini_redis::{Result, client};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

type Responder<T> = oneshot::Sender<Result<T>>;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    // this spawns a thread that will connect to the redis server and
    // recieve messages from the client channels and send them to redis
    // now the problem is how do we report back
    // when i send a Set command i want to know if it was successful
    // when i send a Get command i want to receive the value
    // use Oneshot channel which will forward on the result
    // we will include it as part of the command and client will spin up the channel
    // and tag it int he command when they send it to the channel
    // the manager can then attach a response to that channel
    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::Get { key, resp } => {
                    let g = client.get(&key).await;
                    resp.send(g).unwrap();
                }
                Command::Set { key, val, resp } => {
                    let s = client.set(&key, val).await;
                    resp.send(s).unwrap();
                }
            }
        }
    });

    // this spawns a thread that pushes a command to the client channel for the manager to process
    // in this case it tells it to send the set command
    let t1 = tokio::spawn(async move {
        let (otx, orx) = oneshot::channel();
        tx.send(Command::Set {
            key: "hello".to_string(),
            val: Bytes::from("world"),
            resp: otx,
        })
        .await
        .unwrap();

        match orx.await {
            Ok(msg) => println!("got {:?}", msg.unwrap()),
            Err(_) => println!("Arrrgghhhhh"),
        }
    });

    // this sends a get command
    let t2 = tokio::spawn(async move {
        let (otx, orx) = oneshot::channel();
        tx2.send(Command::Get {
            key: "hello".to_string(),
            resp: otx,
        })
        .await
        .unwrap();
        match orx.await {
            Ok(msg) => println!("got {:?}", msg.unwrap()),
            Err(_) => println!("Arrrgghhhhh"),
        }
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
