#[path = "../../../../support/options/parser/state/entry.rs"]
mod entry;
#[path = "../../../../support/options/parser/state/output.rs"]
mod output;
#[path = "../../../../support/options/parser/state/storage.rs"]
mod storage;
#[path = "../../../../support/options/parser/state/table.rs"]
mod table;

pub(super) use storage::OptionManifestParserState;
