use std::collections::BTreeMap;

pub(in super::super) fn projected_dialog_value_text(
    component_role: &str,
    attributes: &BTreeMap<String, toml::Value>,
) -> Option<String> {
    if !matches!(component_role, "dialog" | "confirm-dialog" | "alert-dialog") {
        return None;
    }
    first_non_empty_attribute(attributes, &["message", "description", "body"]).map(str::to_owned)
}

fn first_non_empty_attribute<'a>(
    attributes: &'a BTreeMap<String, toml::Value>,
    names: &[&str],
) -> Option<&'a str> {
    names
        .iter()
        .filter_map(|name| attributes.get(*name))
        .filter_map(toml::Value::as_str)
        .find(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use super::{first_non_empty_attribute, projected_dialog_value_text};

    #[test]
    fn optimization_batch_dq_dialog_message_borrow_preserves_precedence() {
        let attributes = BTreeMap::from([
            (
                "description".to_string(),
                toml::Value::String("Description".to_string()),
            ),
            ("body".to_string(), toml::Value::String("Body".to_string())),
        ]);
        assert_eq!(
            projected_dialog_value_text("dialog", &attributes),
            Some("Description".to_string())
        );
        assert_eq!(projected_dialog_value_text("other", &attributes), None);
        assert_eq!(
            first_non_empty_attribute(&attributes, &["missing", "description"]),
            Some("Description")
        );
    }

    #[test]
    fn optimization_batch_dq_dialog_message_borrows_candidate_values() {
        let source = include_str!("message.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("dialog message production source");
        assert!(production.contains("toml::Value::as_str"));
        assert!(!production.contains("first_non_empty_string_attribute"));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dq_dialog_message_borrow_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const LOOKUPS_PER_SAMPLE: usize = 16_384;
        let attributes = BTreeMap::from([
            (
                "message".to_string(),
                toml::Value::String("Dialog message payload".to_string()),
            ),
            (
                "description".to_string(),
                toml::Value::String("Fallback description".to_string()),
            ),
        ]);

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_message_lookups(
                    &attributes,
                    LOOKUPS_PER_SAMPLE,
                    true,
                ));
                optimized_samples.push(measure_message_lookups(
                    &attributes,
                    LOOKUPS_PER_SAMPLE,
                    false,
                ));
            } else {
                optimized_samples.push(measure_message_lookups(
                    &attributes,
                    LOOKUPS_PER_SAMPLE,
                    false,
                ));
                legacy_samples.push(measure_message_lookups(
                    &attributes,
                    LOOKUPS_PER_SAMPLE,
                    true,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "EDITOR353_DIALOG_MESSAGE_BORROW_BENCH_V1 lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "dialog message borrow p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );

        fn measure_message_lookups(
            attributes: &BTreeMap<String, toml::Value>,
            lookup_count: usize,
            legacy: bool,
        ) -> u128 {
            let started_at = Instant::now();
            let mut checksum = 0_usize;
            for _ in 0..lookup_count {
                let message_length = if legacy {
                    super::super::super::attribute_values::first_non_empty_string_attribute(
                        attributes,
                        &["message", "description", "body"],
                    )
                    .map_or(0, |value| value.len())
                } else {
                    first_non_empty_attribute(attributes, &["message", "description", "body"])
                        .map_or(0, str::len)
                };
                checksum = checksum.wrapping_add(message_length);
            }
            black_box(checksum);
            started_at.elapsed().as_nanos()
        }

        fn p95(samples: &mut [u128]) -> u128 {
            samples.sort_unstable();
            samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
        }
    }
}
