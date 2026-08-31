use std::sync::Arc;

use zircon_runtime::asset::AssetUri;

use crate::core::jobs::{EditorJobSpec, JobCategory};

use super::diagnostics::EditorAssetImportFlightDiagnostics;
use super::flight::ImportAdmission;
use super::job::{AssetImportJob, ImportLease};
use super::lock::lock_editor_asset_index_recovering_poison;
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
        let mut remaining_generation_revalidations =
            self.limits.max_inline_generation_revalidations;
        loop {
            let generation = lock_editor_asset_index_recovering_poison(self.index.as_ref())
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
                    let current = lock_editor_asset_index_recovering_poison(self.index.as_ref())
                        .is_current_import_generation(&generation);
                    if !current {
                        Self::consume_generation_revalidation(
                            &mut remaining_generation_revalidations,
                            request.uri(),
                        )?;
                        continue;
                    }
                    flight.add_reason(request.reason());
                    match flight.try_admission() {
                        Some(ImportAdmission::Admitted(id)) => {
                            return Ok(EditorAssetImportTicket { id, flight });
                        }
                        Some(ImportAdmission::Revalidate) => {
                            Self::consume_generation_revalidation(
                                &mut remaining_generation_revalidations,
                                request.uri(),
                            )?;
                            continue;
                        }
                        Some(ImportAdmission::Rejected(error)) => return Err(error),
                        None => {
                            return Err(EditorAssetImportSubmitError::AdmissionPending {
                                uri: request.uri().clone(),
                            });
                        }
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
                        let mut index =
                            lock_editor_asset_index_recovering_poison(self.index.as_ref());
                        match begin_uuid {
                            Some(_) => index.begin_import_generation(&generation),
                            None => index.is_current_import_generation(&generation),
                        }
                    };
                    if !current {
                        let action = self.state.abort_unsubmitted(&key, flight_identity);
                        self.complete_index_transition(action);
                        flight.publish_admission(ImportAdmission::Revalidate);
                        Self::consume_generation_revalidation(
                            &mut remaining_generation_revalidations,
                            request.uri(),
                        )?;
                        continue;
                    }
                    if let Some(token) = begin_uuid {
                        if !self.state.mark_uuid_ready(token) {
                            let action = self.state.abort_unsubmitted(&key, flight_identity);
                            self.complete_index_transition(action);
                            flight.publish_admission(ImportAdmission::Revalidate);
                            return Err(
                                EditorAssetImportSubmitError::UuidLifecycleTransitionPending {
                                    uri: request.uri().clone(),
                                },
                            );
                        }
                    }

                    #[cfg(test)]
                    if let Some(hook) = &self.before_job_submit {
                        hook();
                    }

                    let diagnostics = Arc::new(EditorAssetImportFlightDiagnostics::new(
                        Arc::clone(flight.uri()),
                        self.diagnostics.clone(),
                    ));
                    let lease = ImportLease::new(
                        Arc::clone(&self.state),
                        Arc::clone(&self.index),
                        key,
                        flight_identity,
                        Arc::clone(&flight),
                        Arc::clone(&diagnostics),
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
                            diagnostics.reject_submission(&error.to_string());
                            let error = EditorAssetImportSubmitError::Job(error);
                            flight.publish_admission(ImportAdmission::Rejected(error.clone()));
                            return Err(error);
                        }
                    };
                    diagnostics.arm();
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
            lock_editor_asset_index_recovering_poison(self.index.as_ref())
                .clear_import(token.uuid());
            self.state.complete_uuid_clear(token);
        }
    }

    fn consume_generation_revalidation(
        remaining: &mut usize,
        uri: &AssetUri,
    ) -> Result<(), EditorAssetImportSubmitError> {
        let Some(next) = remaining.checked_sub(1) else {
            return Err(EditorAssetImportSubmitError::RegistryGenerationSuperseded {
                uri: uri.clone(),
            });
        };
        *remaining = next;
        Ok(())
    }
}
