#[path = "../state/fields.rs"]
mod fields;
#[path = "../state/finalize.rs"]
mod finalize;
#[path = "../state/storage.rs"]
mod storage;
#[path = "../state/table.rs"]
mod table;

pub(super) use storage::DependencyParserState;
