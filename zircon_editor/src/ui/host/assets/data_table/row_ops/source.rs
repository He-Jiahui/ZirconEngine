use std::ops::Range;

use serde_json::value::RawValue;
use zircon_runtime::asset::DataAsset;
use zircon_runtime_interface::reflect::ReflectTypeInfo;

use super::super::{
    cell_value, raw_source, DataTableCellError, DataTableRow, DataTableSource,
    DataTableSourceAdapter, DataTableSourceError,
};

pub(super) fn spans(text: &str) -> Result<Vec<Range<usize>>, DataTableCellError> {
    let rows: Vec<&RawValue> = serde_json::from_str(text)
        .map_err(|error| DataTableSourceError::Encoding(error.to_string()))?;
    let spans: Vec<_> = rows
        .into_iter()
        .map(|row| raw_source::span(text, row).ok_or(DataTableCellError::SourceInvariant))
        .collect::<Result<_, _>>()?;
    if spans.windows(2).any(|pair| pair[0].end >= pair[1].start) {
        return Err(DataTableCellError::SourceInvariant);
    }
    Ok(spans)
}

pub(super) fn verify(
    asset: &DataAsset,
    text: String,
    row_type: Option<&ReflectTypeInfo>,
    key_field: &str,
    source: &DataTableSource,
    expected_rows: &[&DataTableRow],
) -> Result<DataAsset, DataTableCellError> {
    let mut candidate = asset.clone();
    candidate.text = text;
    candidate.canonical_json = serde_json::from_str(&candidate.text)
        .map_err(|error| DataTableSourceError::Encoding(error.to_string()))?;
    let reopened = DataTableSourceAdapter::read(&candidate, row_type, key_field)?;
    if reopened.columns != source.columns
        || reopened.rows.len() != expected_rows.len()
        || reopened
            .rows
            .iter()
            .zip(expected_rows)
            .any(|(after, before)| !equal(after, before))
    {
        return Err(DataTableCellError::SourceInvariant);
    }
    Ok(candidate)
}

fn equal(left: &DataTableRow, right: &DataTableRow) -> bool {
    left.source_key == right.source_key
        && left.cells.len() == right.cells.len()
        && left.cells.iter().zip(&right.cells).all(|(left, right)| {
            left.field_id == right.field_id
                && left.field_name == right.field_name
                && cell_value::equal(&left.value, &right.value)
        })
}
