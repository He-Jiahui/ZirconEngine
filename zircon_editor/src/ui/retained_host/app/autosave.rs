use super::*;
use crate::core::recovery::{AutosaveDocumentRequest, AutosaveDocumentState};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn poll_editor_autosave(&mut self) {
        let project_root = self
            .startup_session
            .project
            .as_ref()
            .map(|project| project.root_path.clone());
        let poll = self
            .editor_manager
            .context()
            .autosave()
            .poll_project(project_root.as_deref());
        let completion = poll.completion();
        if completion.failed() != 0 {
            self.set_status_line(format!(
                "Autosave completed with {} failed document(s).",
                completion.failed()
            ));
        }
        if !poll.is_due() {
            return;
        }
        let Some(project_root) = project_root else {
            return;
        };
        match self
            .editor_manager
            .context()
            .autosave()
            .preflight_schedule(poll.now())
        {
            Ok(true) => {}
            Ok(false) => return,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        }
        let dirty_documents = match self.editor_manager.dirty_document_toolkits() {
            Ok(documents) => documents,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        let intents = match dirty_documents
            .into_iter()
            .map(|document| {
                self.editor_manager
                    .autosave_document_identity(document.document_id, &project_root)
                    .map(|identity| (document, identity))
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(intents) => intents,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        let documents = intents
            .iter()
            .map(|(_, identity)| {
                AutosaveDocumentState::from_dirty_projection(identity.document().clone(), true)
            })
            .collect::<Vec<_>>();
        if documents.is_empty() {
            return;
        }

        let editor_manager = Arc::clone(&self.editor_manager);
        let mut request_error = None;
        let result = self.editor_manager.context().autosave().schedule(
            poll.now(),
            &documents,
            |_| std::mem::size_of::<AutosaveDocumentRequest>().max(1),
            |autosave_document| {
                let (dirty, identity) = intents
                    .iter()
                    .find(|(_, identity)| identity.document() == autosave_document)?;
                match editor_manager.autosave_document_request(
                    dirty.document_id,
                    dirty.dirty_generation,
                    identity.clone(),
                ) {
                    Ok(request) => Some(request),
                    Err(error) => {
                        request_error = Some(error.to_string());
                        None
                    }
                }
            },
        );
        if let Some(error) = request_error {
            self.set_status_line(error);
        } else if let Err(error) = result {
            self.set_status_line(error.to_string());
        }
    }
}
