use super::error::ServiceError;
use rusqlite::{Connection, TransactionBehavior};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{sync::Semaphore, time::Instant};
mod jobs;
mod migration;
#[cfg(windows)]
mod path_guard;
pub mod receipt;

pub(super) use jobs::JobCensus;

#[derive(Clone)]
pub struct Database {
    connection: Arc<Mutex<OwnedConnection>>,
    slots: Arc<Semaphore>,
    jobs: Arc<jobs::JobRegistry>,
}

struct OwnedConnection {
    connection: Connection,
    #[cfg(windows)]
    _path_guard: Option<path_guard::PathGuard>,
}

impl Database {
    pub fn open(path: PathBuf) -> Result<Self, ServiceError> {
        #[cfg(windows)]
        {
            let guard =
                path_guard::PathGuard::open(&path).map_err(|_| ServiceError::Configuration)?;
            let connection = Connection::open_with_flags(
                &guard.path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                    | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )?;
            Self::from_owned_connection(OwnedConnection {
                connection,
                _path_guard: Some(guard),
            })
        }
        #[cfg(not(windows))]
        Self::from_connection(Connection::open(path)?)
    }

    #[cfg(any(test, not(windows)))]
    fn from_connection(connection: Connection) -> Result<Self, ServiceError> {
        Self::from_owned_connection(OwnedConnection {
            connection,
            #[cfg(windows)]
            _path_guard: None,
        })
    }

    fn from_owned_connection(mut owned: OwnedConnection) -> Result<Self, ServiceError> {
        let connection = &mut owned.connection;
        connection.busy_timeout(Duration::from_secs(2))?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        migration::migrate(connection)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "FULL")?;
        Ok(Self {
            connection: Arc::new(Mutex::new(owned)),
            slots: Arc::new(Semaphore::new(16)),
            jobs: jobs::JobRegistry::new(),
        })
    }

    pub async fn execute<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&mut Connection) -> Result<T, ServiceError> + Send + 'static,
    ) -> Result<T, ServiceError> {
        let permit = self
            .slots
            .clone()
            .try_acquire_owned()
            .map_err(|_| ServiceError::Capacity)?;
        let connection = self.connection.clone();
        let result = self.jobs.submit(move || {
            let _permit = permit;
            let mut connection = connection.lock().map_err(|_| ServiceError::Storage)?;
            operation(&mut connection.connection)
        })?;
        result.await.map_err(|_| ServiceError::OutcomeUnknown)?
    }

    pub(super) fn stop_admission(&self) -> JobCensus {
        self.jobs.stop_admission()
    }

    pub(super) fn job_census(&self) -> JobCensus {
        self.jobs.census()
    }

    pub(super) async fn shutdown(&self, deadline: Instant) -> Result<JobCensus, ServiceError> {
        self.jobs.shutdown(deadline).await
    }

    #[cfg(test)]
    pub fn memory() -> Self {
        Self::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }
}

#[cfg(test)]
mod tests;
