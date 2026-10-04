use super::super::{error::ServiceError, storage::Database};
use std::{
    future::{poll_fn, Future},
    sync::{Arc, Mutex, MutexGuard},
    task::Poll,
    time::Duration,
};
use tokio::{
    sync::{oneshot, watch},
    task::JoinHandle,
    time::{timeout_at, Instant},
};

pub(super) const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(20);

pub(super) struct StartupSupervisor {
    deadline: watch::Receiver<Option<Instant>>,
    storage: Arc<Mutex<Option<Database>>>,
    owners: Vec<JoinHandle<()>>,
    signal_task: Option<JoinHandle<()>>,
}

impl StartupSupervisor {
    pub(super) async fn install_ctrl_c() -> Result<Self, ServiceError> {
        let (sender, deadline) = watch::channel(None);
        let (ready_sender, ready_receiver) = oneshot::channel();
        let storage = Arc::new(Mutex::new(None::<Database>));
        let signal_storage = storage.clone();
        let signal_task = tokio::spawn(async move {
            let signal = tokio::signal::ctrl_c();
            tokio::pin!(signal);
            let initial = poll_fn(|context| match signal.as_mut().poll(context) {
                Poll::Ready(result) => Poll::Ready(Some(result)),
                Poll::Pending => Poll::Ready(None),
            })
            .await;
            if initial.as_ref().is_some_and(Result::is_err) {
                let _ = ready_sender.send(Err(ServiceError::Storage));
                return;
            }
            let _ = ready_sender.send(Ok(()));
            let result = match initial {
                Some(result) => result,
                None => signal.await,
            };
            if result.is_ok() {
                let mut storage = lock_storage(&signal_storage);
                if let Some(database) = storage.as_ref() {
                    database.stop_admission();
                }
                sender.send_replace(Some(Instant::now() + SHUTDOWN_TIMEOUT));
                storage.take();
            }
        });
        ready_receiver.await.map_err(|_| ServiceError::Storage)??;
        Ok(Self {
            deadline,
            storage,
            owners: Vec::new(),
            signal_task: Some(signal_task),
        })
    }

    #[cfg(test)]
    pub(super) fn injected(timeout: Duration) -> (Self, ShutdownTrigger) {
        let (sender, deadline) = watch::channel(None);
        let storage = Arc::new(Mutex::new(None));
        (
            Self {
                deadline,
                storage: storage.clone(),
                owners: Vec::new(),
                signal_task: None,
            },
            ShutdownTrigger {
                sender,
                storage,
                timeout,
            },
        )
    }

    pub(super) fn register_storage(&self, storage: Database) {
        let mut registered = lock_storage(&self.storage);
        if self.deadline().is_some() {
            storage.stop_admission();
        }
        *registered = Some(storage);
    }

    pub(super) fn deadline(&self) -> Option<Instant> {
        *self.deadline.borrow()
    }

    pub(super) async fn wait<T: Send + 'static>(
        &mut self,
        operation: JoinHandle<Result<T, ServiceError>>,
    ) -> Result<T, ServiceError> {
        let (sender, receiver) = oneshot::channel();
        let owner = tokio::spawn(async move {
            let result = match operation.await {
                Ok(result) => result,
                Err(_) => Err(ServiceError::Storage),
            };
            let _ = sender.send(result);
        });
        self.owners.push(owner);
        tokio::pin!(receiver);
        if let Some(deadline) = self.deadline() {
            return finish_cancelled_startup(receiver, deadline).await;
        }
        tokio::select! {
            result = &mut receiver => {
                let result = result.map_err(|_| ServiceError::OutcomeUnknown)??;
                if self.deadline().is_some() {
                    Err(ServiceError::OutcomeUnknown)
                } else {
                    Ok(result)
                }
            },
            deadline = self.shutdown_requested() => {
                finish_cancelled_startup(receiver, deadline?).await
            }
        }
    }

    pub(super) async fn shutdown_requested(&mut self) -> Result<Instant, ServiceError> {
        loop {
            if let Some(deadline) = self.deadline() {
                return Ok(deadline);
            }
            self.deadline
                .changed()
                .await
                .map_err(|_| ServiceError::Storage)?;
        }
    }

    pub(super) fn active_owner_count(&self) -> usize {
        self.owners
            .iter()
            .filter(|owner| !owner.is_finished())
            .count()
    }
}

impl Drop for StartupSupervisor {
    fn drop(&mut self) {
        let active = self.active_owner_count();
        if active != 0 {
            eprintln!("Hub service startup shutdown incomplete: active_startup_owners={active}");
        }
        if let Some(task) = self.signal_task.take() {
            task.abort();
        }
    }
}

async fn finish_cancelled_startup<T: Send + 'static>(
    mut result: std::pin::Pin<&mut oneshot::Receiver<Result<T, ServiceError>>>,
    deadline: Instant,
) -> Result<T, ServiceError> {
    let _ = timeout_at(deadline, result.as_mut()).await;
    Err(ServiceError::OutcomeUnknown)
}

fn lock_storage(storage: &Mutex<Option<Database>>) -> MutexGuard<'_, Option<Database>> {
    storage
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
pub(super) struct ShutdownTrigger {
    sender: watch::Sender<Option<Instant>>,
    storage: Arc<Mutex<Option<Database>>>,
    timeout: Duration,
}

#[cfg(test)]
impl ShutdownTrigger {
    pub(super) fn request(&self) {
        let mut storage = lock_storage(&self.storage);
        if self.sender.borrow().is_none() {
            if let Some(database) = storage.as_ref() {
                database.stop_admission();
            }
            self.sender
                .send_replace(Some(Instant::now() + self.timeout));
            storage.take();
        }
    }
}

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
