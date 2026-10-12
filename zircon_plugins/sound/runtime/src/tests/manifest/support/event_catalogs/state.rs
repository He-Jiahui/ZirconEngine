#[path = "../../../support/state/event_catalogs_fields.rs"]
mod fields;
#[path = "../../../support/state/event_catalogs_finalize.rs"]
mod finalize;
#[path = "../../../support/state/event_catalogs_storage.rs"]
mod storage;
#[path = "../../../support/state/event_catalogs_table.rs"]
mod table;

pub(super) use storage::EventCatalogParserState;
