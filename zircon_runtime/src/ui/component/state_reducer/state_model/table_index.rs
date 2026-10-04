use std::cmp::Ordering;
use std::collections::HashMap;

use zircon_runtime_interface::ui::component::UiValue;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TableFieldKind {
    Numeric,
    Text,
}

#[derive(Clone, Debug)]
struct TableField {
    index: usize,
    kind: TableFieldKind,
}

/// A generation-tagged row permutation produced by the table model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableSortResult {
    pub generation: u64,
    pub column: String,
    pub descending: bool,
    pub permutation: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct TableIndex {
    generation: u64,
    fields: HashMap<String, TableField>,
    accepted_generation: u64,
}

impl TableIndex {
    pub fn compile(columns: Option<&UiValue>, rows: Option<&UiValue>, generation: u64) -> Self {
        let mut fields = HashMap::new();
        if let Some(UiValue::Array(columns)) = columns {
            for (index, column) in columns.iter().enumerate() {
                let Some(map) = (match column {
                    UiValue::Map(values) => Some(values),
                    _ => None,
                }) else {
                    continue;
                };
                let Some(field) = ["field", "id", "key", "name"]
                    .into_iter()
                    .find_map(|key| map.get(key).and_then(string_value))
                else {
                    continue;
                };
                let kind = infer_field_kind(rows, field);
                fields
                    .entry(field.to_string())
                    .or_insert(TableField { index, kind });
            }
        }
        if fields.is_empty() {
            if let Some(UiValue::Array(rows)) = rows {
                if let Some(UiValue::Map(row)) = rows.first() {
                    for (index, (field, value)) in row.iter().enumerate() {
                        let kind = if matches!(value, UiValue::Int(_) | UiValue::Float(_)) {
                            TableFieldKind::Numeric
                        } else {
                            TableFieldKind::Text
                        };
                        fields.insert(field.clone(), TableField { index, kind });
                    }
                }
            }
        }
        Self {
            generation,
            fields,
            accepted_generation: generation,
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn set_generation(&mut self, generation: u64) {
        self.generation = generation;
        self.accepted_generation = generation;
    }

    pub fn field_index(&self, field: &str) -> Option<usize> {
        self.fields.get(field).map(|field| field.index)
    }

    pub fn sort_permutation(
        &self,
        rows: &UiValue,
        column: &str,
        direction: &str,
    ) -> TableSortResult {
        let mut permutation = match rows {
            UiValue::Array(rows) => (0..rows.len()).collect::<Vec<_>>(),
            _ => Vec::new(),
        };
        let descending = matches!(direction, "desc" | "descending");
        let kind = self
            .fields
            .get(column)
            .map(|field| field.kind)
            .unwrap_or(TableFieldKind::Text);
        if let UiValue::Array(rows) = rows {
            permutation.sort_by(|left, right| {
                let value_ordering = compare_rows(&rows[*left], &rows[*right], column, kind);
                let ordering = if value_ordering == Ordering::Equal {
                    left.cmp(right)
                } else {
                    value_ordering
                };
                if descending {
                    if value_ordering == Ordering::Equal {
                        ordering
                    } else {
                        ordering.reverse()
                    }
                } else {
                    ordering
                }
            });
        }
        TableSortResult {
            generation: self.generation,
            column: column.to_string(),
            descending,
            permutation,
        }
    }

    /// Accept only a result produced for the current model generation.
    pub fn accept_sort_result(
        &self,
        result: TableSortResult,
        current_generation: u64,
    ) -> Option<TableSortResult> {
        (result.generation == self.generation
            && current_generation == self.generation
            && self.accepted_generation == result.generation)
            .then_some(result)
    }
}

fn infer_field_kind(rows: Option<&UiValue>, field: &str) -> TableFieldKind {
    let Some(UiValue::Array(rows)) = rows else {
        return TableFieldKind::Text;
    };
    if rows.iter().any(|row| {
        row_field(row, field)
            .is_some_and(|value| matches!(value, UiValue::Int(_) | UiValue::Float(_)))
    }) {
        TableFieldKind::Numeric
    } else {
        TableFieldKind::Text
    }
}

fn compare_rows(left: &UiValue, right: &UiValue, column: &str, kind: TableFieldKind) -> Ordering {
    let left = row_field(left, column);
    let right = row_field(right, column);
    if kind == TableFieldKind::Numeric {
        if let (Some(left), Some(right)) = (
            left.and_then(UiValue::as_f64),
            right.and_then(UiValue::as_f64),
        ) {
            return left.partial_cmp(&right).unwrap_or(Ordering::Equal);
        }
    }
    match (left.and_then(string_value), right.and_then(string_value)) {
        (Some(left), Some(right)) => left.cmp(right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn row_field<'a>(row: &'a UiValue, column: &str) -> Option<&'a UiValue> {
    match row {
        UiValue::Map(values) => values.get(column),
        _ => None,
    }
}

fn string_value(value: &UiValue) -> Option<&str> {
    match value {
        UiValue::String(value)
        | UiValue::Color(value)
        | UiValue::AssetRef(value)
        | UiValue::InstanceRef(value)
        | UiValue::Enum(value) => Some(value),
        _ => None,
    }
}
