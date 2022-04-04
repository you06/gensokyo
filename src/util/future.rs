// Copyright 2018 TiKV Project Authors. Licensed under Apache-2.0.

use futures::channel::mpsc;
use futures::channel::oneshot as futures_oneshot;
use futures::future::{self, BoxFuture, Future, FutureExt, TryFutureExt};
use futures::stream::{Stream, StreamExt};
use futures::task::{self, ArcWake, Context, Poll};

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

/// Polls the provided future immediately. If the future is not ready,
/// it will register the waker. When the event is ready, the waker will
/// be notified, then the internal future is immediately polled in the
/// thread calling `wake()`.
pub fn poll_future_notify<T, F: Future<Output = T> + Send + 'static>(f: F) -> T {
    let f: BoxFuture<'static, T> = Box::pin(f);
    let poller = Arc::new(PollAtWake {
        f: UnsafeCell::new(Some(f)),
        state: AtomicU8::new(IDLE),
    });
    PollAtWake::poll(&poller).unwrap()
}

/// The future is not processed by any one.
const IDLE: u8 = 0;
/// The future is being polled by some thread.
const POLLING: u8 = 1;
/// The future is woken when being polled.
const NOTIFIED: u8 = 2;

/// A waker that will poll the future immediately when waking up.
pub struct PollAtWake<T> {
    f: UnsafeCell<Option<BoxFuture<'static, T>>>,
    state: AtomicU8,
}

impl<T> PollAtWake<T> {
    fn poll(arc_self: &Arc<PollAtWake<T>>) -> Option<T> {
        let mut state = arc_self.state.load(Ordering::Relaxed);
        loop {
            match state {
                IDLE => {
                    match arc_self.state.compare_exchange_weak(
                        IDLE,
                        POLLING,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    ) {
                        Ok(_) => break,
                        Err(s) => state = s,
                    }
                }
                POLLING => {
                    match arc_self.state.compare_exchange_weak(
                        POLLING,
                        NOTIFIED,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    ) {
                        // The polling thread should poll the future again.
                        Ok(_) => return None,
                        Err(s) => state = s,
                    }
                }
                NOTIFIED => {
                    // It will be polled again, so we don't need to do anything here.
                    return None;
                }
                _ => panic!("unexpected state {}", state),
            }
        }

        let f = unsafe { &mut *arc_self.f.get() };
        let fut = match f {
            Some(f) => f,
            None => {
                // It can't be `None` for the moment. But it's not a big mistake, just ignore.
                return None;
            }
        };

        let waker = task::waker_ref(arc_self);
        let cx = &mut Context::from_waker(&*waker);
        loop {
            match fut.as_mut().poll(cx) {
                // Likely pending
                Poll::Pending => (),
                Poll::Ready(t) => {
                    // We skip updating states here as all future wake should be ignored once
                    // a future is resolved.
                    f.take();
                    return Some(t);
                }
            }
            match arc_self
                .state
                .compare_exchange(POLLING, IDLE, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return None,
                Err(s) => {
                    if s == NOTIFIED {
                        // Only this thread can change the state from NOTIFIED, so it has to succeed.
                        match arc_self.state.compare_exchange(
                            NOTIFIED,
                            POLLING,
                            Ordering::SeqCst,
                            Ordering::SeqCst,
                        ) {
                            Ok(_) => continue,
                            Err(s) => panic!("unexpected state {}", s),
                        }
                    } else {
                        panic!("unexpcted state {}", s);
                    }
                }
            }
        }
    }
}

// `BoxFuture` is Send, so `PollAtWake` is Send and Sync.
unsafe impl<T> Send for PollAtWake<T> {}
unsafe impl<T> Sync for PollAtWake<T> {}

impl<T> ArcWake for PollAtWake<T> {
    #[inline]
    fn wake_by_ref(arc_self: &Arc<Self>) {
        PollAtWake::poll(arc_self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::task::Poll;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_in_place_wake() {
        let poll_times = Arc::new(AtomicUsize::new(0));
        let times = poll_times.clone();
        let f = futures::future::poll_fn(move |cx| {
            cx.waker().wake_by_ref();
            let last_time = times.fetch_add(1, Ordering::SeqCst);
            if last_time == 0 {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        });
        poll_future_notify(f);
        // The future will be woken twice, but only polled twice.
        // The sequence should be:
        // 1. future gets polled
        //   1.1 future gets woken
        //      1.1.1 future marks NOTIFIED
        //   1.2 future returns Poll::Pending
        // 2. future finishes polling, then re-poll
        //   2.1 future gets woken
        //     2.1.1 future marks NOTIFIED
        //   2.2 future returns Poll::Ready
        // 3. future gets ready, ignore NOTIFIED
        assert_eq!(poll_times.load(Ordering::SeqCst), 2);
    }
}
