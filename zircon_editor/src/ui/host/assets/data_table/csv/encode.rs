use zircon_runtime::asset::DataAsset;
use zircon_runtime_interface::reflect::{ReflectTypeInfo, ReflectedValue};

use super::super::{raw_source, DataTableCellError, DataTableSourceAdapter};
use super::{records, DataTableCsvCodec, DataTableCsvError};

impl DataTableCsvCodec {
    /// Exports actual source rows, schema field names/order and the explicit key column.
    /// Serializable hidden/read-only columns remain data. Display labels are not import keys.
    /// Numeric source tokens never pass through a floating-point display formatter.
    pub fn encode(
        asset: &DataAsset,
        row_type: Option<&ReflectTypeInfo>,
        key_field: &str,
    ) -> Result<String, DataTableCsvError> {
        let source = DataTableSourceAdapter::read(asset, row_type, key_field)?;
        let raw_rows = raw_source::rows(&asset.text)?;
        let mut output = String::new();
        let mut cells = vec![key_field.to_owned()];
        cells.extend(source.columns.iter().map(|field| field.name.clone()));
        records::write(&mut output, &cells);
        for (row, raw) in source.rows.iter().zip(raw_rows.iter()) {
            let mut cells = vec![row.source_key.clone()];
            for (field, cell) in source.columns.iter().zip(row.cells.iter()) {
                cells.push(match &cell.value {
                    ReflectedValue::String(text) => text.clone(),
                    _ => raw
                        .0
                        .iter()
                        .find(|(name, _)| name == &field.name)
                        .ok_or(DataTableCellError::SourceInvariant)?
                        .1
                        .get()
                        .to_owned(),
                });
            }
            records::write(&mut output, &cells);
        }
        Ok(output)
    }
}
