//! Bounded completion fanout: payloads are shared and waking never holds the state lock.
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
type Outcome<T> = Result<T, Arc<str>>;
const MAX_WAITERS: usize = 64;
pub(crate) struct AsyncValue<T> {
    state: Mutex<State<T>>,
}
struct State<T> {
    result: Option<Outcome<T>>,
    waiters: Vec<Option<Waker>>,
}
impl<T: Clone> AsyncValue<T> {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State {
                result: None,
                waiters: Vec::new(),
            }),
        })
    }
    pub(crate) fn result(&self) -> Option<Outcome<T>> {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .result
            .clone()
    }
    pub(crate) fn complete(&self, result: Outcome<T>) {
        let wake = {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if state.result.is_some() {
                return;
            }
            state.result = Some(result);
            std::mem::take(&mut state.waiters)
        };
        for waker in wake.into_iter().flatten() {
            waker.wake();
        }
    }
    pub(crate) fn wait(self: &Arc<Self>) -> Result<ValueWait<T>, Arc<str>> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let slot = if state.result.is_some() {
            None
        } else {
            let slot = state
                .waiters
                .iter()
                .position(Option::is_none)
                .unwrap_or(state.waiters.len());
            if slot >= MAX_WAITERS {
                return Err("asset completion waiter limit reached".into());
            }
            if slot == state.waiters.len() {
                state.waiters.push(None);
            }
            // Reserve the slot before its first poll. An empty waker is not a free reservation.
            state.waiters[slot] = Some(Waker::noop().clone());
            Some(slot)
        };
        Ok(ValueWait {
            value: Arc::clone(self),
            slot,
        })
    }
    pub(crate) fn sender(self: &Arc<Self>) -> ValueSender<T> {
        ValueSender(Some(Arc::clone(self)))
    }
}
pub(crate) struct ValueWait<T> {
    value: Arc<AsyncValue<T>>,
    slot: Option<usize>,
}
impl<T: Clone> Future for ValueWait<T> {
    type Output = Outcome<T>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.value.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(result) = &state.result {
            return Poll::Ready(result.clone());
        }
        let slot = self.slot.expect("pending completion has a reserved waiter");
        let waker = &mut state.waiters[slot];
        if !waker.as_ref().is_some_and(|w| w.will_wake(cx.waker())) {
            *waker = Some(cx.waker().clone());
        }
        Poll::Pending
    }
}
impl<T> Drop for ValueWait<T> {
    fn drop(&mut self) {
        if let Some(slot) = self.slot {
            if let Some(waker) = self
                .value
                .state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .waiters
                .get_mut(slot)
            {
                *waker = None;
            }
        }
    }
}
pub(crate) struct ValueSender<T: Clone>(Option<Arc<AsyncValue<T>>>);
impl<T: Clone> ValueSender<T> {
    pub(crate) fn send(mut self, value: Outcome<T>) {
        if let Some(cell) = self.0.take() {
            cell.complete(value);
        }
    }
}
impl<T: Clone> Drop for ValueSender<T> {
    fn drop(&mut self) {
        if let Some(cell) = self.0.take() {
            cell.complete(Err("asset request cancelled before completion".into()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::task::Wake;
    #[test]
    fn bounded_waiters_share_completion_and_reuse_cancelled_slots() {
        let value = AsyncValue::new();
        let mut waiters = (0..MAX_WAITERS)
            .map(|_| value.wait().unwrap())
            .collect::<Vec<_>>();
        assert!(value.wait().is_err());
        drop(waiters.pop());
        let mut replacement = value.wait().unwrap();
        value.sender().send(Ok(7));
        let mut context = Context::from_waker(Waker::noop());
        for waiter in &mut waiters {
            assert_eq!(Pin::new(waiter).poll(&mut context), Poll::Ready(Ok(7)));
        }
        assert_eq!(
            Pin::new(&mut replacement).poll(&mut context),
            Poll::Ready(Ok(7))
        );
        assert!(value.wait().is_ok());
    }
    #[test]
    fn abandoned_sender_publishes_cancellation() {
        let value = AsyncValue::<()>::new();
        drop(value.sender());
        assert!(value.result().unwrap().unwrap_err().contains("cancelled"));
    }
    struct Reentrant {
        value: Arc<AsyncValue<()>>,
    }
    impl Wake for Reentrant {
        fn wake(self: Arc<Self>) {
            assert!(self.value.state.try_lock().is_ok());
        }
    }
    #[test]
    fn completion_wakes_without_holding_its_lock() {
        let value = AsyncValue::new();
        let mut waiter = value.wait().unwrap();
        let waker = Waker::from(Arc::new(Reentrant {
            value: value.clone(),
        }));
        assert!(Pin::new(&mut waiter)
            .poll(&mut Context::from_waker(&waker))
            .is_pending());
        value.complete(Ok(()));
    }
}
