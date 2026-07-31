use std::sync::Arc;

use crate::core::jobs::{EditorJobSpec, JobCategory};

use super::flight::ImportAdmission;
use super::job::{AssetImportJob, ImportLease};
use super::state::{ImportFinishAction, ImportGenerationKey, ImportReservation};
use super::{
    EditorAssetImportFlow, EditorAssetImportRequest, EditorAssetImportSubmitError,
    EditorAssetImportTicket,
};

impl EditorAssetImportFlow {
    pub fn submit(
        &self,
        request: EditorAssetImportRequest,
    ) -> Result<EditorAssetImportTicket, EditorAssetImportSubmitError> {
        loop {
            let generation = self
                .index
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .import_generation(request.uri())
                .ok_or_else(|| EditorAssetImportSubmitError::AssetNotIndexed {
                    uri: request.uri().clone(),
                })?;
            let key = ImportGenerationKey::new(
                generation.uuid(),
                Arc::clone(generation.uri()),
                Arc::clone(generation.source_digest()),
            );
            let reservation = self.state.reserve(key, request.reason(), self.limits)?;

            #[cfg(test)]
            if let Some(hook) = &self.before_generation_validate {
                hook();
            }

            match reservation {
                ImportReservation::Existing { flight } => {
                    let current = self
                        .index
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .is_current_import_generation(&generation);
                    if !current {
                        continue;
                    }
                    flight.add_reason(request.reason());
                    #[cfg(test)]
                    if let Some(hook) = &self.before_wait_admission {
                        hook();
                    }
                    match flight.wait_admission() {
                        ImportAdmission::Admitted(id) => {
                            return Ok(EditorAssetImportTicket { id, flight });
                        }
                        ImportAdmission::Revalidate => continue,
                        ImportAdmission::Rejected(error) => return Err(error),
                    }
                }
                ImportReservation::New {
                    key,
                    flight_identity,
                    mutex_group,
                    begin_uuid,
                    flight,
                } => {
                    let current = {
                        let mut index = self
                            .index
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        match begin_uuid {
                            Some(_) => index.begin_import_generation(&generation),
                            None => index.is_current_import_generation(&generation),
                        }
                    };
                    if !current {
                        let action = self.state.abort_unsubmitted(&key, flight_identity);
                        self.complete_index_transition(action);
                        flight.publish_admission(ImportAdmission::Revalidate);
                        continue;
                    }
                    if let Some(token) = begin_uuid {
                        if !self.state.mark_uuid_ready(token) {
                            let action = self.state.abort_unsubmitted(&key, flight_identity);
                            self.complete_index_transition(action);
                            flight.publish_admission(ImportAdmission::Revalidate);
                            continue;
                        }
                    }

                    #[cfg(test)]
                    if let Some(hook) = &self.before_job_submit {
                        hook();
                    }

                    let lease = ImportLease::new(
                        Arc::clone(&self.state),
                        Arc::clone(&self.index),
                        key,
                        flight_identity,
                        Arc::clone(&flight),
                        self.limits,
                    );
                    let label = format!("Import {}", request.uri());
                    let job =
                        AssetImportJob::new(request.clone(), Arc::clone(&self.backend), lease);
                    let ticket = match self.jobs.submit(
                        EditorJobSpec::new(label, JobCategory::Import)
                            .with_mutex_group(mutex_group),
                        job,
                    ) {
                        Ok(ticket) => ticket,
                        Err(error) => {
                            let error = EditorAssetImportSubmitError::Job(error);
                            flight.publish_admission(ImportAdmission::Rejected(error.clone()));
                            return Err(error);
                        }
                    };
                    let id = ticket.id();
                    flight.publish_admission(ImportAdmission::Admitted(id));
                    drop(ticket);
                    return Ok(EditorAssetImportTicket { id, flight });
                }
            }
        }
    }

    fn complete_index_transition(&self, action: ImportFinishAction) {
        if let ImportFinishAction::ClearUuid(token) = action {
            self.index
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clear_import(token.uuid());
            self.state.complete_uuid_clear(token);
        }
    }
}
