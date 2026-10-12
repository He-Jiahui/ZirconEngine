#[path = "state/entry.rs"]
mod entry;
#[path = "state/flush.rs"]
mod flush;
#[path = "../../../support/parser/state/section_line.rs"]
mod section_line;
#[path = "state/storage.rs"]
mod storage;
#[path = "../../../support/parser/state/transition.rs"]
mod transition;

pub(super) use self::storage::OptionalFeatureParserState;
