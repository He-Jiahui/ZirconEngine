use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{
    binding::UiBindingUpdateReport, component::UiValue, event_ui::UiNodeId, tree::UiTreeError,
};

use crate::ui::surface::{UiPropertyMutationRequest, UiPropertyMutationStatus, UiSurface};

use super::columns;

fn replace_table_map_value(values: &mut BTreeMap<String, UiValue>, key: &str, value: UiValue) {
    if let Some(existing) = values.get_mut(key) {
        *existing = value;
    } else {
        values.insert(key.to_owned(), value);
    }
}

impl UiSurface {
    pub(super) fn apply_table_column_widths_mutation(
        &mut self,
        owner_id: UiNodeId,
        field: &str,
        width: f64,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let mut widths = self
            .template_metadata(owner_id)
            .ok()
            .and_then(|metadata| metadata.attributes.get("column_widths"))
            .map(UiValue::from_toml)
            .and_then(|value| match value {
                UiValue::Map(values) => Some(values),
                _ => None,
            })
            .unwrap_or_default();
        replace_table_map_value(&mut widths, field, UiValue::Float(width));
        self.apply_table_mutation(
            owner_id,
            "column_widths",
            UiValue::Map(widths),
            binding_reports,
        )
    }

    pub(super) fn apply_table_columns_width_mutation(
        &mut self,
        owner_id: UiNodeId,
        field: &str,
        width: f64,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let Some(mut columns) = self
            .template_metadata(owner_id)
            .ok()
            .and_then(|metadata| metadata.attributes.get("columns"))
            .map(UiValue::from_toml)
            .and_then(|value| match value {
                UiValue::Array(columns) => Some(columns),
                _ => None,
            })
        else {
            return Ok(false);
        };

        let mut found = false;
        for column in &mut columns {
            let UiValue::Map(values) = column else {
                continue;
            };
            if columns::table_column_matches(values, field) {
                replace_table_map_value(values, "width", UiValue::Float(width));
                found = true;
                break;
            }
        }
        if !found {
            return Ok(false);
        }

        self.apply_table_mutation(
            owner_id,
            "columns",
            UiValue::Array(columns),
            binding_reports,
        )
    }

    pub(super) fn apply_table_sort_model_mutation(
        &mut self,
        owner_id: UiNodeId,
        field: &str,
        direction: &str,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        self.apply_table_mutation(
            owner_id,
            "sortModel",
            UiValue::Array(vec![UiValue::Map(BTreeMap::from([
                ("field".to_string(), UiValue::String(field.to_string())),
                ("sort".to_string(), UiValue::String(direction.to_string())),
            ]))]),
            binding_reports,
        )
    }

    pub(super) fn apply_table_columns_sort_direction_mutation(
        &mut self,
        owner_id: UiNodeId,
        field: &str,
        direction: &str,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let Some(mut columns) = self
            .template_metadata(owner_id)
            .ok()
            .and_then(|metadata| metadata.attributes.get("columns"))
            .map(UiValue::from_toml)
            .and_then(|value| match value {
                UiValue::Array(columns) => Some(columns),
                _ => None,
            })
        else {
            return Ok(false);
        };

        let mut found = false;
        for column in &mut columns {
            let UiValue::Map(values) = column else {
                continue;
            };
            let next_direction = if columns::table_column_matches(values, field) {
                found = true;
                direction
            } else {
                "none"
            };
            replace_table_map_value(
                values,
                "sortDirection",
                UiValue::String(next_direction.to_string()),
            );
        }
        if !found {
            return Ok(false);
        }

        self.apply_table_mutation(
            owner_id,
            "columns",
            UiValue::Array(columns),
            binding_reports,
        )
    }

    pub(super) fn apply_table_rows_sort_mutation(
        &mut self,
        owner_id: UiNodeId,
        field: &str,
        direction: &str,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let Some(mut rows) = self
            .template_metadata(owner_id)
            .ok()
            .and_then(|metadata| metadata.attributes.get("rows"))
            .map(UiValue::from_toml)
            .and_then(|value| match value {
                UiValue::Array(rows) => Some(rows),
                _ => None,
            })
        else {
            return Ok(false);
        };

        rows.sort_by(|left, right| columns::compare_table_row_value(left, right, field));
        if direction == "desc" {
            rows.reverse();
        }
        self.apply_table_mutation(owner_id, "rows", UiValue::Array(rows), binding_reports)
    }

    pub(super) fn apply_table_mutation(
        &mut self,
        owner_id: UiNodeId,
        property: impl Into<String>,
        value: UiValue,
        binding_reports: &mut Vec<UiBindingUpdateReport>,
    ) -> Result<bool, UiTreeError> {
        let report = self.mutate_property(UiPropertyMutationRequest::widget_behavior(
            owner_id, property, value,
        ))?;
        if matches!(report.status, UiPropertyMutationStatus::Accepted) {
            binding_reports.push(report.binding);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_ec_existing_table_map_keys_update_in_place() {
        let source = include_str!("mutation.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("table mutation production implementation");

        assert!(production.contains("if let Some(existing) = values.get_mut(key)"));
        assert!(production.contains("values.insert(key.to_owned(), value)"));
        assert_eq!(production.matches("replace_table_map_value(").count(), 4);
        assert!(!production.contains("widths.insert(field.to_string()"));
        assert!(!production.contains("values.insert(\"width\".to_string()"));
        assert!(!production.contains("\"sortDirection\".to_string()"));

        let mut values = BTreeMap::from([("width".to_owned(), UiValue::Float(24.0))]);
        replace_table_map_value(&mut values, "width", UiValue::Float(48.0));
        assert_eq!(values.len(), 1);
        assert_eq!(values.get("width"), Some(&UiValue::Float(48.0)));

        replace_table_map_value(&mut values, "sortDirection", UiValue::String("asc".into()));
        assert_eq!(values.len(), 2);
        assert_eq!(
            values.get("sortDirection"),
            Some(&UiValue::String("asc".into()))
        );
    }

    #[test]
    #[ignore = "release-only existing table map key benchmark"]
    fn optimization_batch_ec_existing_table_map_key_release_benchmark_evidence() {
        const SAMPLE_PAIRS: usize = 17;
        const COLUMN_COUNT: usize = 256;
        const UPDATE_PASSES_PER_SAMPLE: usize = 128;

        fn fixture() -> (Vec<String>, BTreeMap<String, UiValue>) {
            let fields = (0..COLUMN_COUNT)
                .map(|index| format!("column.{index:04}"))
                .collect::<Vec<_>>();
            let values = fields
                .iter()
                .cloned()
                .map(|field| (field, UiValue::Float(0.0)))
                .collect();
            (fields, values)
        }

        fn measure_legacy(fields: &[String], base: &BTreeMap<String, UiValue>) -> u128 {
            let mut values = base.clone();
            let started = Instant::now();
            for pass in 0..UPDATE_PASSES_PER_SAMPLE {
                for field in fields {
                    values.insert(
                        black_box(field.as_str()).to_owned(),
                        UiValue::Float(pass as f64),
                    );
                }
            }
            black_box(values);
            started.elapsed().as_nanos().max(1)
        }

        fn measure_optimized(fields: &[String], base: &BTreeMap<String, UiValue>) -> u128 {
            let mut values = base.clone();
            let started = Instant::now();
            for pass in 0..UPDATE_PASSES_PER_SAMPLE {
                for field in fields {
                    replace_table_map_value(
                        &mut values,
                        black_box(field.as_str()),
                        UiValue::Float(pass as f64),
                    );
                }
            }
            black_box(values);
            started.elapsed().as_nanos().max(1)
        }

        fn percentile(samples: &[u128], percentile: usize) -> u128 {
            let mut sorted = samples.to_vec();
            sorted.sort_unstable();
            let rank = (sorted.len() * percentile).div_ceil(100);
            sorted[rank.saturating_sub(1)]
        }

        fn raw(samples: &[u128]) -> String {
            samples
                .iter()
                .map(u128::to_string)
                .collect::<Vec<_>>()
                .join(",")
        }

        let (fields, base) = fixture();
        for _ in 0..4 {
            black_box(measure_legacy(&fields, &base));
            black_box(measure_optimized(&fields, &base));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure_legacy(&fields, &base));
                optimized_samples.push(measure_optimized(&fields, &base));
            } else {
                optimized_samples.push(measure_optimized(&fields, &base));
                legacy_samples.push(measure_legacy(&fields, &base));
            }
        }

        let legacy_p50_ns = percentile(&legacy_samples, 50);
        let optimized_p50_ns = percentile(&optimized_samples, 50);
        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        let key_updates_per_sample = COLUMN_COUNT * UPDATE_PASSES_PER_SAMPLE;

        println!(
            "RUNTIME437_EXISTING_TABLE_MAP_KEY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
column_count={COLUMN_COUNT} update_passes_per_sample={UPDATE_PASSES_PER_SAMPLE} \
key_updates_per_sample={key_updates_per_sample} pair_order=alternating_legacy_even \
legacy_first_pairs=9 optimized_first_pairs=8 \
legacy_key_allocations_per_sample={key_updates_per_sample} optimized_key_allocations_per_sample=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );

        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
            "borrowed existing table map keys must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }
}
