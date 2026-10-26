//! AI semantics over supplied source snapshots, without document or runtime mutation.

#[path = "menu.rs"]
mod menu;
#[path = "menu_types.rs"]
mod menu_types;
#[path = "rename.rs"]
mod rename;
#[path = "scan.rs"]
mod scan;
#[path = "schema_edit.rs"]
mod schema_edit;
#[path = "types.rs"]
mod types;

pub use menu::AiBlackboardMenuAdapter;
pub use menu_types::{
    AiBlackboardKeyMenuRow, AiBlackboardMenuCandidate, AiBlackboardMenuCommand,
    AiBlackboardMenuDisabledReason, AiBlackboardMenuError, AiBlackboardMenuRequest,
    AiBlackboardMenuRow, AiBlackboardReferenceKnowledge,
};
pub use schema_edit::AiBlackboardSchemaEditCandidate;
pub use types::{
    AiBlackboardDependencies, AiBlackboardKeyReference, AiBlackboardKeyUsage,
    AiBlackboardProviderSnapshot, AiBlackboardReferenceBlocker, AiBlackboardReferencePlan,
    AiBlackboardRenameCandidate, AiBlackboardRenameError, AiBlackboardSchemaInput,
    AiBlackboardSourceVersion, AiBlackboardTreeAssociation, AiBlackboardTreeInput,
    AiBlackboardTreeRewrite,
};
