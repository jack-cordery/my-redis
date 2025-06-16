use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (mut tx, mut rx) = mpsc::channel(128);
    let (mut tx2, mut rx2) = mpsc::channel(128);

    tokio::spawn(async move {
        let _ = tx.send(1).await;
        let _ = tx2.send(1).await;
    });

    tokio::select! {

            Some(msg) = rx.recv() => {
            println!("recevied from 1 -> {msg}")
            }
            Some(msg) = rx2.recv() => {
            println!("recevied from 2 -> {msg}")
            }
            else => {
            println!("nothing recevied")
        }


    }
}
