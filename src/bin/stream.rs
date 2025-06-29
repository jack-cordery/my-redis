use mini_redis::client;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::time::Instant;
use tokio::time::{Duration, sleep};
use tokio_stream::{Stream, StreamExt};

struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        if Instant::now() >= self.when {
            Poll::Ready("done")
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

struct Interval {
    rem: usize,
    delay: Delay,
}

impl Interval {
    pub fn new() -> Self {
        Interval {
            rem: 3,
            delay: Delay {
                when: Instant::now(),
            },
        }
    }
}

impl Stream for Interval {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<()>> {
        if self.rem == 0 {
            return Poll::Ready(None);
        }

        match Pin::new(&mut self.delay).poll(cx) {
            Poll::Ready(_) => {
                self.rem -= 1;
                let when = self.delay.when + Duration::from_millis(10);
                self.delay = Delay { when };
                Poll::Ready(Some(()))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

async fn publish() -> mini_redis::Result<()> {
    let mut connect = client::connect("127.0.0.1:6379").await?;

    connect.publish("numbers", "one".into()).await?;
    connect.publish("numbers", "two".into()).await?;
    connect.publish("numbers", "three".into()).await?;
    connect.publish("numbers", "1".into()).await?;
    connect.publish("numbers", "2".into()).await?;
    connect.publish("numbers", "3".into()).await?;
    connect.publish("numbers", "10".into()).await?;

    Ok(())
}

async fn subscribe() -> mini_redis::Result<()> {
    let connect = client::connect("127.0.0.1:6379").await?;
    let subscriber = connect.subscribe(vec!["numbers".to_string()]).await?;

    let messages = subscriber
        .into_stream()
        .filter(|msg| matches!(msg, Ok(msg) if msg.content.len() == 1))
        .take(3)
        .map(|msg| msg.unwrap().content);

    tokio::pin!(messages);

    while let Some(msg) = messages.next().await {
        println!("I got some published messages on the numbers channel {msg:?}")
    }
    Ok(())
}

#[tokio::main]
async fn main() -> mini_redis::Result<()> {
    tokio::spawn(async {
        sleep(Duration::from_millis(1)).await;
        publish().await.unwrap();
    });

    subscribe().await?;
    println!("Done");

    let mut interval = Interval::new();
    while interval.next().await.is_some() {
        println!(
            "manual streamed delayed a few instances in time at {:?}",
            Instant::now()
        )
    }
    Ok(())
}
