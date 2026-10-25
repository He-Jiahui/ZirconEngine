//! CSV text exchange for an already admitted JSON table and its explicit row schema.
//!
//! No file I/O, schema association, source defaults or document/write admission lives here.
//! CSV is an exchange format; the original DataAsset retains its JSON format and URI.

#[path = "decode.rs"]
mod decode;
#[path = "encode.rs"]
mod encode;
#[path = "error.rs"]
mod error;
#[path = "records.rs"]
mod records;

pub use error::DataTableCsvError;

pub struct DataTableCsvCodec;
