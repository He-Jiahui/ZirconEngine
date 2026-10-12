#[path = "state/entry.rs"]
mod entry;
#[path = "../../../support/modules/state/finalize.rs"]
mod finalize;
#[path = "state/storage.rs"]
mod storage;
#[path = "state/table.rs"]
mod table;

pub(super) use storage::ModuleContributionParserState;
