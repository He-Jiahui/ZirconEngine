use std::collections::HashSet;

use serde_json::value::RawValue;
use zircon_runtime::asset::DataAsset;
use zircon_runtime_interface::reflect::ReflectTypeInfo;

use super::super::{
    cell_value, fields, raw_source, DataTableCellError, DataTableSource, DataTableSourceAdapter,
    DataTableSourceError,
};
use super::{records, DataTableCsvCodec, DataTableCsvError};

impl DataTableCsvCodec {
    /// Preflights every record before returning one complete JSON source candidate.
    ///
    /// All columns must be supplied by exact schema name, including the explicit key column;
    /// column order can differ. Missing values/defaults, UE FName sanitation, nested fields and
    /// import options that skip errors are not inferred. Row order follows the CSV. Unchanged
    /// hidden/read-only cells may round trip, but cannot be changed/initialized here.
    ///
    /// `None` preserves every original byte, URI, format and canonical-cache state. A changed
    /// table is completely re-encoded. The actual Document must retain its whole before/after
    /// inverse and authorize one current-revision history/CAS/save operation; matching cell
    /// values or this candidate alone cannot establish asset/schema association or reject ABA.
    pub fn prepare_reimport(
        asset: &DataAsset,
        row_type: Option<&ReflectTypeInfo>,
        key_field: &str,
        csv: &str,
    ) -> Result<Option<DataAsset>, DataTableCsvError> {
        let source = DataTableSourceAdapter::read(asset, row_type, key_field)?;
        let raw_rows = raw_source::rows(&asset.text)?;
        let records = records::read(csv)?;
        let header = records.first().ok_or(DataTableCsvError::MissingHeader)?;
        let mut seen = HashSet::new();
        for name in header {
            if !seen.insert(name.as_str()) {
                return Err(DataTableCsvError::Header(format!(
                    "duplicate column {name}"
                )));
            }
            if name != key_field && !source.columns.iter().any(|field| &field.name == name) {
                return Err(DataTableCsvError::Header(format!("unknown column {name}")));
            }
        }
        let key_index = header
            .iter()
            .position(|name| name == key_field)
            .ok_or_else(|| DataTableCsvError::Header(format!("missing key column {key_field}")))?;
        let columns = source
            .columns
            .iter()
            .map(|field| {
                header
                    .iter()
                    .position(|name| name == &field.name)
                    .ok_or_else(|| {
                        DataTableCsvError::Header(format!("missing column {}", field.name))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut text = String::from("[");
        for (row_index, row) in records.iter().skip(1).enumerate() {
            if row.len() != header.len() {
                return Err(DataTableCsvError::RowWidth {
                    row: row_index + 1,
                    expected: header.len(),
                    found: row.len(),
                });
            }
            let key = &row[key_index];
            if row_index != 0 {
                text.push(',');
            }
            text.push('{');
            text.push_str(&json_string(key_field)?);
            text.push(':');
            text.push_str(&json_string(key)?);
            let original = source
                .rows
                .iter()
                .enumerate()
                .find(|(_, row)| &row.source_key == key);
            for (column_index, (field, csv_index)) in
                source.columns.iter().zip(columns.iter()).enumerate()
            {
                let token = if field.value_type_path == "alloc::string::String" {
                    json_string(&row[*csv_index])?
                } else {
                    row[*csv_index].clone()
                };
                let cell = (|| {
                    let raw: &RawValue =
                        serde_json::from_str(&token).map_err(|error| error.to_string())?;
                    fields::value(raw, field)
                })()
                .map_err(|reason| DataTableCellError::InvalidValue {
                    row: key.clone(),
                    field_id: field.id,
                    reason,
                })?;
                let unchanged = original.is_some_and(|(_, old)| {
                    cell_value::equal(&old.cells[column_index].value, &cell)
                });
                if (!field.editable || !field.editor_visible) && !unchanged {
                    return Err(DataTableCellError::FieldNotEditable(field.id).into());
                }
                let encoded = if let Some((original_index, _)) = original.filter(|_| unchanged) {
                    // Preserve actual unchanged tokens, including legacy values outside current
                    // editor limits. Schema hints cannot turn a lossless round trip into an edit.
                    raw_rows[original_index]
                        .0
                        .iter()
                        .find(|(name, _)| name == &field.name)
                        .ok_or(DataTableCellError::SourceInvariant)?
                        .1
                        .get()
                        .to_owned()
                } else {
                    cell_value::encode(field, &cell).map_err(|reason| {
                        DataTableCellError::InvalidValue {
                            row: key.clone(),
                            field_id: field.id,
                            reason,
                        }
                    })?
                };
                text.push(',');
                text.push_str(&json_string(&field.name)?);
                text.push(':');
                text.push_str(&encoded);
            }
            text.push('}');
        }
        text.push(']');
        let mut candidate = asset.clone();
        candidate.canonical_json = serde_json::from_str(&text)
            .map_err(|error| DataTableSourceError::Encoding(error.to_string()))?;
        candidate.text = text;
        let reopened = DataTableSourceAdapter::read(&candidate, row_type, key_field)?;
        if same_rows(&source, &reopened) {
            return Ok(None);
        }
        Ok(Some(candidate))
    }
}

fn json_string(text: &str) -> Result<String, DataTableSourceError> {
    serde_json::to_string(text).map_err(|error| DataTableSourceError::Encoding(error.to_string()))
}

fn same_rows(left: &DataTableSource, right: &DataTableSource) -> bool {
    left.rows.len() == right.rows.len()
        && left
            .rows
            .iter()
            .zip(right.rows.iter())
            .all(|(left, right)| {
                left.source_key == right.source_key
                    && left.cells.len() == right.cells.len()
                    && left
                        .cells
                        .iter()
                        .zip(right.cells.iter())
                        .all(|(left, right)| {
                            left.field_id == right.field_id
                                && cell_value::equal(&left.value, &right.value)
                        })
            })
}
