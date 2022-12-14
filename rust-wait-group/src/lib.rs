use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use parking_lot::Mutex;

#[derive(Default)]
struct SharedState {
    count: usize,
    waker: Option<Waker>,
}

#[derive(Default)]
pub struct WaitGroup {
    shared_state: Arc<Mutex<SharedState>>,
}
impl WaitGroup {
    pub fn new_token(&self) -> WaitGroupToken {
        let token = WaitGroupToken {
            shared_state: self.shared_state.clone(),
        };
        let mut shared_state = self.shared_state.lock();
        shared_state.count += 1;
        token
    }
}
impl Future for WaitGroup {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock();
        if shared_state.count == 0 {
            Poll::Ready(())
        } else {
            shared_state.waker.replace(cx.waker().clone());
            Poll::Pending
        }
    }
}

#[derive(Clone)]
pub struct WaitGroupToken {
    shared_state: Arc<Mutex<SharedState>>,
}
impl Drop for WaitGroupToken {
    fn drop(&mut self) {
        let mut shared_state = self.shared_state.lock();
        shared_state.count -= 1;
        if shared_state.count == 0 {
            if let Some(x) = shared_state.waker.take() {
                x.wake()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test() {
        const TASK_NUM: usize = 100;
        let count = Arc::new(AtomicUsize::new(0));
        let wg = super::WaitGroup::default();
        for _ in 0..TASK_NUM {
            let token = wg.new_token();
            let count = count.clone();
            tokio::spawn(async move {
                let _token = token;
                count.fetch_add(1, Ordering::AcqRel);
            });
        }
        wg.await;
        assert_eq!(count.load(Ordering::Acquire), TASK_NUM);
    }
}
