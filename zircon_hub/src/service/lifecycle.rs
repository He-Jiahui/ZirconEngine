use super::{
    cloud::BlobStore, config::ServiceConfig, error::ServiceError, http, identity, storage,
};
use std::{future::IntoFuture, sync::Arc};

mod supervisor;
use supervisor::{StartupSupervisor, SHUTDOWN_TIMEOUT};

pub async fn run() -> Result<(), ServiceError> {
    let mut supervisor = StartupSupervisor::install_ctrl_c().await?;
    let config_path = std::env::args_os()
        .nth(1)
        .ok_or(ServiceError::Configuration)?;
    let config_path = std::path::PathBuf::from(config_path);
    let config = supervisor
        .wait(tokio::task::spawn_blocking(move || {
            ServiceConfig::load(&config_path)
        }))
        .await?;

    let database_path = config.database.clone();
    let storage = supervisor
        .wait(tokio::task::spawn_blocking(move || {
            storage::Database::open(database_path)
        }))
        .await?;
    supervisor.register_storage(storage.clone());
    if let Some(deadline) = supervisor.deadline() {
        return finish_cancelled_startup(&storage, deadline).await;
    }

    let cloud_config = config.cloud.clone();
    let cloud_storage = storage.clone();
    let cloud = match supervisor
        .wait(tokio::spawn(async move {
            cloud_storage
                .execute(move |connection| {
                    let store = BlobStore::load(&cloud_config)?;
                    store.recover(connection)?;
                    Ok(Arc::new(store))
                })
                .await
        }))
        .await
    {
        Ok(cloud) => cloud,
        Err(error) => return finish_startup_error(&storage, &supervisor, error).await,
    };

    let identity_config = config.clone();
    let identity = match supervisor
        .wait(tokio::spawn(async move {
            identity::OidcVerifier::discover(identity_config).await
        }))
        .await
    {
        Ok(identity) => identity,
        Err(error) => return finish_startup_error(&storage, &supervisor, error).await,
    };

    let bind = config.bind;
    let listener = match supervisor
        .wait(tokio::spawn(async move {
            tokio::net::TcpListener::bind(bind)
                .await
                .map_err(|_| ServiceError::Configuration)
        }))
        .await
    {
        Ok(listener) => listener,
        Err(error) => return finish_startup_error(&storage, &supervisor, error).await,
    };

    let app = http::router(identity, storage.clone(), config.catalog_policy_file, cloud);
    let (stop_server, server_stopping) = tokio::sync::oneshot::channel();
    let server = axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = server_stopping.await;
        })
        .into_future();
    tokio::pin!(server);
    tokio::select! {
        result = &mut server => {
            let deadline = supervisor.deadline().unwrap_or_else(|| tokio::time::Instant::now() + SHUTDOWN_TIMEOUT);
            storage.stop_admission();
            if storage.shutdown(deadline).await.is_err() {
                report_incomplete_shutdown(&storage);
                return Err(ServiceError::OutcomeUnknown);
            }
            result.map_err(|_| ServiceError::Storage)
        }
        deadline = supervisor.shutdown_requested() => {
            let deadline = deadline?;
            storage.stop_admission();
            let _ = stop_server.send(());
            let (server_result, storage_result) = tokio::join!(
                tokio::time::timeout_at(deadline, &mut server),
                storage.shutdown(deadline),
            );
            if server_result.is_err() || storage_result.is_err() {
                report_incomplete_shutdown(&storage);
                return Err(ServiceError::OutcomeUnknown);
            }
            if matches!(server_result, Ok(Err(_))) {
                return Err(ServiceError::Storage);
            }
            Ok(())
        }
    }
}

async fn finish_startup_error(
    storage: &storage::Database,
    supervisor: &StartupSupervisor,
    error: ServiceError,
) -> Result<(), ServiceError> {
    let cancelled = supervisor.deadline();
    let deadline = cancelled.unwrap_or_else(|| tokio::time::Instant::now() + SHUTDOWN_TIMEOUT);
    storage.stop_admission();
    if storage.shutdown(deadline).await.is_err() {
        report_incomplete_shutdown(storage);
        return Err(ServiceError::OutcomeUnknown);
    }
    if cancelled.is_some() {
        Err(ServiceError::OutcomeUnknown)
    } else {
        Err(error)
    }
}

async fn finish_cancelled_startup(
    storage: &storage::Database,
    deadline: tokio::time::Instant,
) -> Result<(), ServiceError> {
    storage.stop_admission();
    if storage.shutdown(deadline).await.is_err() {
        report_incomplete_shutdown(storage);
    }
    Err(ServiceError::OutcomeUnknown)
}

fn report_incomplete_shutdown(storage: &storage::Database) {
    let census = storage.job_census();
    eprintln!(
        "Hub service shutdown incomplete: active_database_jobs={}, finished_database_jobs={}, failed_database_jobs={}",
        census.active, census.finished, census.failed
    );
}
