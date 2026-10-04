use super::{
    cloud,
    operations::{
        CloudCommitCleanupRecord, OperationIdentity, OperationJournal, OperationPayload,
        OperationStatus, OperationSummary,
    },
    service::project_mutation_result,
    AccountBroker, AccountError, AccountView, ServiceRequest,
};
use serde_json::Value;

impl AccountBroker {
    /// The account identity and its journal revision are observed under one state guard.
    pub async fn recovery_snapshot(
        &self,
    ) -> (
        AccountView,
        Result<(Vec<OperationSummary>, String), AccountError>,
    ) {
        let state = self.state.lock().await;
        let view = state.view.clone();
        let Some(authenticated) = state
            .authenticated
            .as_ref()
            .filter(|_| view.status == "signed-in")
        else {
            return (view, Ok((Vec::new(), "0".into())));
        };
        let identity = OperationIdentity::new(&self.config, &authenticated.subject);
        let records = match self.operation_journal() {
            Ok(journal) => tokio::task::spawn_blocking(move || journal.snapshot(&identity))
                .await
                .map_err(|_| AccountError::OperationStore)
                .and_then(|records| records),
            Err(error) => Err(error),
        };
        (view, records)
    }

    /// Returns the exact authenticated journal view used to authorize local Cloud cleanup.
    ///
    /// The public account snapshot intentionally keeps its legacy DTO. This internal view adds
    /// CloudCommit project identity and raw status so a terminal result for one project cannot
    /// authorize deletion of another upload snapshot.
    pub(crate) async fn cloud_cleanup_snapshot(
        &self,
    ) -> (
        AccountView,
        Result<(Vec<OperationSummary>, Vec<CloudCommitCleanupRecord>, String), AccountError>,
    ) {
        let state = self.state.lock().await;
        let view = state.view.clone();
        let Some(authenticated) = state
            .authenticated
            .as_ref()
            .filter(|_| view.status == "signed-in")
        else {
            return (view, Ok((Vec::new(), Vec::new(), "0".into())));
        };
        let identity = OperationIdentity::new(&self.config, &authenticated.subject);
        let records = match self.operation_journal() {
            Ok(journal) => {
                tokio::task::spawn_blocking(move || journal.cloud_cleanup_snapshot(&identity))
                    .await
                    .map_err(|_| AccountError::OperationStore)
                    .and_then(|records| records)
            }
            Err(error) => Err(error),
        };
        (view, records)
    }

    pub async fn retry_operation(
        &self,
        expected_generation: &str,
        operation_id: &str,
    ) -> Result<Value, AccountError> {
        // This permit spans lookup, durable admission, dispatch and completion.
        let permit = self.mutation.try_lock().map_err(|_| AccountError::Busy)?;
        let identity = self.operation_identity(expected_generation).await?;
        let journal = self.operation_journal()?;
        let id = operation_id.to_owned();
        let record = tokio::task::spawn_blocking(move || journal.get(&identity, &id))
            .await
            .map_err(|_| AccountError::OperationStore)??;
        if matches!(&record.payload, OperationPayload::InstallPackage { .. }) {
            return self
                .install_with_permit(
                    expected_generation,
                    record.operation_id,
                    record.payload,
                    true,
                    &permit,
                )
                .await;
        }
        let request = match record.payload {
            OperationPayload::InstallPackage { .. } => return Err(AccountError::ServiceFailure),
            OperationPayload::CatalogLicense {
                organization,
                expected_policy_revision,
                package_id,
                revision,
                license_id,
            } => ServiceRequest::CatalogLicense {
                organization,
                operation_id: record.operation_id,
                expected_policy_revision,
                package_id,
                revision,
                license_id,
            },
            OperationPayload::CreateOrganization { name } => ServiceRequest::CreateOrganization {
                operation_id: record.operation_id,
                name,
            },
            OperationPayload::Mutate {
                organization,
                expected_policy_revision,
                mutation,
            } => ServiceRequest::Mutate {
                organization,
                operation_id: record.operation_id,
                expected_policy_revision,
                mutation: serde_json::to_value(mutation)
                    .map_err(|_| AccountError::ServiceFailure)?,
            },
            OperationPayload::CloudCommit {
                organization,
                project,
                base_revision,
                manifest,
            } => ServiceRequest::CloudCommit {
                organization,
                project,
                operation_id: record.operation_id,
                base_revision,
                manifest: serde_json::to_value(manifest)
                    .map_err(|_| AccountError::ServiceFailure)?,
            },
        };
        self.service_request_with_permit(expected_generation, request, &permit)
            .await
    }

    pub async fn reconcile_operation(
        &self,
        expected_generation: &str,
        operation_id: &str,
    ) -> Result<Value, AccountError> {
        let permit = self.mutation.try_lock().map_err(|_| AccountError::Busy)?;
        let identity = self.operation_identity(expected_generation).await?;
        let journal = self.operation_journal()?;
        let record = tokio::task::spawn_blocking({
            let journal = journal.clone();
            let identity = identity.clone();
            let id = operation_id.to_owned();
            move || journal.get(&identity, &id)
        })
        .await
        .map_err(|_| AccountError::OperationStore)??;
        let response = if matches!(&record.payload, OperationPayload::InstallPackage { .. }) {
            match self
                .reconcile_install(expected_generation, &record.payload)
                .await?
            {
                Some(result) => {
                    let id = operation_id.to_owned();
                    tokio::task::spawn_blocking(move || {
                        journal.finish(&identity, &id, OperationStatus::Committed)
                    })
                    .await
                    .map_err(|_| AccountError::OperationStore)??;
                    return Ok(serde_json::json!({"status":"committed","result":result}));
                }
                None => serde_json::json!({"status":"unknown"}),
            }
        } else {
            self.service_request_with_permit(
                expected_generation,
                ServiceRequest::Receipt {
                    operation_id: operation_id.into(),
                },
                &permit,
            )
            .await?
        };
        let (status, response) = project_operation_receipt(&record.payload, &response)?;
        let id = operation_id.to_owned();
        tokio::task::spawn_blocking(move || journal.finish(&identity, &id, status))
            .await
            .map_err(|_| AccountError::OperationStore)??;
        Ok(response)
    }

    pub async fn acknowledge_operation(
        &self,
        expected_generation: &str,
        operation_id: &str,
    ) -> Result<(), AccountError> {
        let _mutation = self.mutation.try_lock().map_err(|_| AccountError::Busy)?;
        let state = self.state.lock().await;
        if state.view.generation != expected_generation {
            return Err(AccountError::Cancelled);
        }
        let authenticated = state
            .authenticated
            .as_ref()
            .ok_or(AccountError::SessionExpired)?;
        let identity = OperationIdentity::new(&self.config, &authenticated.subject);
        let journal = self.operation_journal()?;
        let id = operation_id.to_owned();
        tokio::task::spawn_blocking(move || journal.acknowledge(&identity, &id))
            .await
            .map_err(|_| AccountError::OperationStore)?
    }

    pub(super) async fn operation_identity(
        &self,
        expected_generation: &str,
    ) -> Result<OperationIdentity, AccountError> {
        let state = self.state.lock().await;
        if state.view.generation != expected_generation {
            return Err(AccountError::Cancelled);
        }
        let authenticated = state
            .authenticated
            .as_ref()
            .ok_or(AccountError::SessionExpired)?;
        Ok(OperationIdentity::new(&self.config, &authenticated.subject))
    }

    pub(super) fn operation_journal(&self) -> Result<OperationJournal, AccountError> {
        Ok(OperationJournal::new(self.config.journal_path()?))
    }
}

fn project_operation_receipt(
    payload: &OperationPayload,
    response: &Value,
) -> Result<(OperationStatus, Value), AccountError> {
    match response.get("status").and_then(Value::as_str) {
        Some("committed") => {
            let result = response
                .get("result")
                .and_then(|value| project_mutation_result(payload, value))
                .ok_or(AccountError::OutcomeUnknown)?;
            let conflict = matches!(payload, OperationPayload::CloudCommit { .. })
                && cloud::commit::is_conflict(&result);
            Ok((
                if conflict {
                    OperationStatus::Conflict
                } else {
                    OperationStatus::Committed
                },
                serde_json::json!({"status":if conflict {"conflict"} else {"committed"},"result":result}),
            ))
        }
        Some("unknown") => Ok((
            OperationStatus::Unknown,
            serde_json::json!({"status":"unknown"}),
        )),
        _ => Err(AccountError::OutcomeUnknown),
    }
}

#[cfg(test)]
#[path = "tests/recovery.rs"]
mod tests;
