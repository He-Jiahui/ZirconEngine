use zircon_runtime_interface::reflect::{ReflectFieldInfo, ReflectFieldValue};

/// A read-only projection of one source and a caller-supplied row type.
///
/// This carries no asset/schema registration, document revision, write admission or history.
/// Column metadata retains its original stable field IDs and declaration order. Its `editable`
/// flag alone never enables a write command. The original DataAsset remains the source owner.
#[derive(Clone, Debug, PartialEq)]
pub struct DataTableSource {
    pub key_field: String,
    pub columns: Vec<ReflectFieldInfo>,
    pub rows: Vec<DataTableRow>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataTableRow {
    /// Exact source key, including case and Unicode; never a filtered/sorted row index.
    /// UE FName sanitation/case rules require a separate, explicit import contract.
    pub source_key: String,
    pub cells: Vec<ReflectFieldValue>,
}
