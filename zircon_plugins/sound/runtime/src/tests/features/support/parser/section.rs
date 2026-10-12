#[path = "section/kind.rs"]
mod kind;
#[path = "../../../support/parser/section/table_header.rs"]
mod table_header;

pub(super) use self::kind::OptionalFeatureSection;
