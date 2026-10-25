use zircon_runtime_interface::reflect::{ReflectFieldId, ReflectedValue};

#[derive(Clone, Debug, PartialEq)]
pub struct DataTableCellEdit {
    /// Exact source key, independent of filtering/sorting and display row indices.
    pub row_key: String,
    pub field_id: ReflectFieldId,
    /// The actual typed value observed in the captured source. This is a cell precondition,
    /// not a substitute for the original document revision/publication guard or ABA protection.
    pub expected: ReflectedValue,
    pub value: ReflectedValue,
}
