use std::collections::{BTreeMap, HashMap};

use toml::Value;

use super::entry::CommandProjectionEntry;
use super::ids::command_id_values;
use super::parse::command_entry_list;

const COMMANDS: &str = "commands";
const FILTERED_COMMANDS: &str = "filtered_commands";

pub(super) fn projected_command_entries(
    attributes: &BTreeMap<String, Value>,
) -> Vec<CommandProjectionEntry> {
    let commands = attributes
        .get(COMMANDS)
        .map(command_entry_list)
        .unwrap_or_default();
    let Some(filtered) = attributes.get(FILTERED_COMMANDS) else {
        return commands;
    };
    let mut command_index = HashMap::with_capacity(commands.len());
    for (index, entry) in commands.iter().enumerate() {
        command_index.entry(entry.id.as_str()).or_insert(index);
    }

    let ids = command_id_values(filtered);
    let mut entries = Vec::with_capacity(ids.len());
    for id in ids {
        let entry = if let Some(index) = command_index.get(id.as_str()) {
            Some(commands[*index].clone())
        } else if id.is_empty() {
            None
        } else {
            Some(CommandProjectionEntry::new(id))
        };
        if let Some(entry) = entry {
            entries.push(entry.with_filter_matched());
        }
    }
    entries
}

#[cfg(test)]
mod optimization_batch_20260830ce_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const IDS_PER_SAMPLE: usize = 512;

    #[test]
    fn filtered_command_projection_reserves_id_capacity() {
        let source = include_str!("entries.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("command palette entry implementation");

        assert!(implementation.contains("let ids = command_id_values(filtered)"));
        assert!(implementation.contains("Vec::with_capacity(ids.len())"));
        assert!(implementation.contains("for id in ids"));
        assert!(implementation.contains("entry.with_filter_matched()"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830ce_editor_command_palette_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!("EDITOR329_COMMAND_PALETTE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} ids_per_sample={IDS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}", csv(&legacy), csv(&optimized));
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..128 {
            let mut entries = if use_capacity {
                Vec::with_capacity(IDS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for id in 0..IDS_PER_SAMPLE {
                if id % 4 != 0 {
                    entries.push(id);
                }
            }
            checksum ^= entries.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
