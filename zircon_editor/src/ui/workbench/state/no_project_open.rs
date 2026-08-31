use super::EditorStateOperationError;

pub(super) fn no_project_open() -> EditorStateOperationError {
    EditorStateOperationError::NoProjectOpen
}
