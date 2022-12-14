use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Poll, Waker};
use std::time::Duration;

use parking_lot::Mutex;

struct SharedState<T> {
    done_reason: Option<ContextDoneReason<T>>,
    wakers: Vec<Waker>,
}

#[derive(Clone)]
pub enum ContextDoneReason<T> {
    Cancelled(T),
    Timeout,
}

#[derive(Clone)]
pub struct Context<T> {
    shared_state: Arc<Mutex<SharedState<T>>>,
}

impl<T> Context<T>
where
    T: Clone + Sync + Send + 'static,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            shared_state: Arc::new(Mutex::new(SharedState {
                done_reason: None,
                wakers: vec![],
            })),
        }
    }

    pub fn with_timeout(duration: Duration) -> Self {
        let this = Self::new();
        let this_clone = this.clone();
        tokio::spawn(async move {
            let clone1 = this_clone.clone();
            let clone2 = this_clone;
            tokio::select! {
                _ = tokio::time::sleep(duration) => {
                    clone1.cancel0(ContextDoneReason::Timeout);
                }
                _ = clone2 => {}
            }
        });
        this
    }

    pub fn with_parent(parent: &Self) -> Self {
        let this = Self::new();
        let this_clone = this.clone();
        let parent = parent.clone();
        tokio::spawn(async move {
            let clone1 = this_clone.clone();
            let clone2 = this_clone;
            tokio::select! {
                reason = parent => {
                    clone1.cancel0(reason);
                }
                _ = clone2 => {}
            }
        });
        this
    }

    pub fn with_parent_and_timeout(parent: &Self, duration: Duration) -> Self {
        let this = Self::new();
        let this_clone = this.clone();
        let parent = parent.clone();
        tokio::spawn(async move {
            let clone1 = this_clone.clone();
            let clone2 = this_clone.clone();
            let clone3 = this_clone;
            tokio::select! {
                _ = tokio::time::sleep(duration) => {
                    clone1.cancel0(ContextDoneReason::Timeout);
                }
                reason = parent => {
                    clone2.cancel0(reason)
                }
                _ = clone3 => {}
            }
        });
        this
    }

    #[inline]
    pub fn id(&self) -> usize {
        &*self.shared_state as *const _ as usize
    }

    pub fn cancel(self, reason: T) {
        self.cancel0(ContextDoneReason::Cancelled(reason))
    }

    fn cancel0(self, reason: ContextDoneReason<T>) {
        let mut this = self.shared_state.lock();
        if this.done_reason.is_some() {
            return;
        }

        this.done_reason.replace(reason);

        let mut wakers = vec![];
        mem::swap(&mut this.wakers, &mut wakers);
        for waker in wakers {
            waker.wake();
        }
    }
}
impl<T> Default for Context<T>
where
    T: Clone + Sync + Send + 'static,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Future for Context<T>
where
    T: Clone,
{
    type Output = ContextDoneReason<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut this = self.shared_state.lock();
        if let Some(x) = &this.done_reason {
            return Poll::Ready(x.clone());
        }
        this.wakers.push(cx.waker().clone());
        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::{Context, ContextDoneReason};

    #[tokio::test]
    async fn test1() {
        let ctx: Context<u32> = Context::new();
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            ctx_clone.cancel(42);
        });
        match ctx.await {
            ContextDoneReason::Cancelled(x) => {
                assert_eq!(x, 42);
            }
            ContextDoneReason::Timeout => {
                unreachable!();
            }
        }
    }

    #[tokio::test]
    async fn test2() {
        let ctx: Context<u32> = Context::with_timeout(std::time::Duration::from_micros(1));
        match ctx.await {
            ContextDoneReason::Cancelled(_) => {
                unreachable!();
            }
            ContextDoneReason::Timeout => {}
        }
    }

    #[tokio::test]
    async fn test3() {
        let p_ctx: Context<u32> = Context::new();
        let ctx = Context::with_parent(&p_ctx);
        tokio::spawn(async move {
            p_ctx.cancel(42);
        });
        match ctx.await {
            ContextDoneReason::Cancelled(x) => {
                assert_eq!(x, 42);
            }
            ContextDoneReason::Timeout => {
                unreachable!();
            }
        }
    }

    #[tokio::test]
    async fn test4() {
        let p_ctx: Context<u32> = Context::with_timeout(std::time::Duration::from_micros(1));
        let ctx = Context::with_parent(&p_ctx);
        match ctx.await {
            ContextDoneReason::Cancelled(_) => {
                unreachable!();
            }
            ContextDoneReason::Timeout => {}
        }
    }

    #[tokio::test]
    async fn test5() {
        let p_ctx: Context<u32> = Context::new();
        let ctx = Context::with_parent_and_timeout(&p_ctx, std::time::Duration::from_micros(1));
        match ctx.await {
            ContextDoneReason::Cancelled(_) => {
                unreachable!();
            }
            ContextDoneReason::Timeout => {}
        }
    }

    #[tokio::test]
    async fn test6() {
        let pp_ctx: Context<u32> = Context::new();
        let p_ctx = Context::with_parent(&pp_ctx);
        let ctx = Context::with_parent(&p_ctx);
        tokio::spawn(async move {
            pp_ctx.cancel(42);
        });
        match ctx.await {
            ContextDoneReason::Cancelled(x) => {
                assert_eq!(x, 42);
            }
            ContextDoneReason::Timeout => {
                unreachable!();
            }
        }
    }
}
