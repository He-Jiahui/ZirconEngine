#[path = "state/fields.rs"]
mod fields;
#[path = "../../statuses/state/finalize.rs"]
mod finalize;
#[path = "../../statuses/state/storage.rs"]
mod storage;
#[path = "state/table.rs"]
mod table;

pub(super) use storage::CapabilityStatusParserState;
