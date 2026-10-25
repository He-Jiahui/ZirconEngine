use std::collections::HashSet;
use zircon_runtime::asset::{DataAsset, DataAssetFormat};
use zircon_runtime_interface::reflect::{ReflectFieldValue, ReflectTypeInfo};

use super::{fields, raw_source, DataTableRow, DataTableSource, DataTableSourceError};

pub struct DataTableSourceAdapter;

impl DataTableSourceAdapter {
    /// Reads actual JSON text, with a row type and key-field contract supplied by the owner.
    ///
    /// File name, generic Data kind, canonical JSON and reflected type shape do not establish
    /// semantic asset/schema admission. The caller must associate that type with this source.
    /// This function does no I/O, callbacks, mutation, registration or write admission.
    pub fn read(
        asset: &DataAsset,
        row_type: Option<&ReflectTypeInfo>,
        key_field: &str,
    ) -> Result<DataTableSource, DataTableSourceError> {
        if asset.format != DataAssetFormat::Json {
            return Err(DataTableSourceError::UnsupportedFormat(asset.format));
        }
        let row_type = row_type.ok_or(DataTableSourceError::MissingRowType)?;
        let columns = fields::columns(row_type, key_field)?;
        let raw_rows = raw_source::rows(&asset.text)?;
        let mut rows = Vec::with_capacity(raw_rows.len());
        let mut keys = HashSet::new();
        for (source_row, raw_row) in raw_rows.into_iter().enumerate() {
            let key = raw_row
                .0
                .iter()
                .find(|(name, _)| name == key_field)
                .and_then(|(_, raw)| serde_json::from_str::<String>(raw.get()).ok())
                .filter(|key| !key.is_empty())
                .ok_or_else(|| DataTableSourceError::InvalidRowKey {
                    source_row,
                    key_field: key_field.into(),
                })?;
            if !keys.insert(key.clone()) {
                return Err(DataTableSourceError::DuplicateRowKey(key));
            }
            for (name, _) in &raw_row.0 {
                if name != key_field && !columns.iter().any(|field| &field.name == name) {
                    return Err(DataTableSourceError::UnknownColumn {
                        row: key,
                        field: name.clone(),
                    });
                }
            }
            let mut cells = Vec::with_capacity(columns.len());
            for field in &columns {
                let raw = raw_row
                    .0
                    .iter()
                    .find(|(name, _)| name == &field.name)
                    .map(|(_, value)| value)
                    .ok_or_else(|| DataTableSourceError::MissingColumn {
                        row: key.clone(),
                        field: field.name.clone(),
                    })?;
                let value = fields::value(raw, field).map_err(|reason| {
                    DataTableSourceError::InvalidCell {
                        row: key.clone(),
                        field: field.name.clone(),
                        value_type_path: field.value_type_path.clone(),
                        reason,
                    }
                })?;
                cells.push(ReflectFieldValue::new(field.id, field.name.clone(), value));
            }
            rows.push(DataTableRow {
                source_key: key,
                cells,
            });
        }
        if !asset.canonical_json.is_null() {
            let canonical: serde_json::Value = serde_json::from_str(&asset.text)
                .map_err(|error| DataTableSourceError::Encoding(error.to_string()))?;
            if canonical != asset.canonical_json {
                return Err(DataTableSourceError::CanonicalMismatch);
            }
        }
        Ok(DataTableSource {
            key_field: key_field.into(),
            columns: columns.into_iter().cloned().collect(),
            rows,
        })
    }
}
