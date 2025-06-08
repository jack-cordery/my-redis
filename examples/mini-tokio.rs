use futures::task;
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

struct MiniTokio {
    tasks: VecDeque<Tasks>,
}

type Tasks = Pin<Box<dyn Future<Output = ()> + Send>>;

impl MiniTokio {
    fn new() -> Self {
        MiniTokio {
            tasks: VecDeque::new(),
        }
    }

    /// Spawn a future onto the mini-tokio instance
    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.tasks.push_back(Box::pin(future))
    }

    fn run(&mut self) {
        let waker = task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        while let Some(mut task) = self.tasks.pop_front() {
            if task.as_mut().poll(&mut cx).is_pending() {
                self.tasks.push_back(task);
            }
        }
    }
}
struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
        if Instant::now() >= self.when {
            println!("Hello World!");
            Poll::Ready("done")
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

fn main() {
    let mut mini_tokio = MiniTokio::new();

    mini_tokio.spawn(async {
        let when = Instant::now() + Duration::from_secs(5);
        let future = Delay { when };
        let out = future.await;
        assert_eq!(out, "done");
    });

    mini_tokio.run();
}
