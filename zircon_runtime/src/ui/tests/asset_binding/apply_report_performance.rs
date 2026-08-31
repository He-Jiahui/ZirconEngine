use std::{hint::black_box, time::Instant};

use zircon_runtime_interface::ui::binding::{
    UiBindingDirtyDomain, UiBindingMutationReceipt, UiBindingUpdateStatus,
};

const SAMPLE_PAIRS: usize = 21;
const RECEIPTS_PER_SAMPLE: usize = 2_048;
const UPDATES_PER_RECEIPT: usize = 32;

#[test]
#[ignore = "release performance gate; run through the Runtime74 validator"]
fn authoritative_binding_apply_receipt_p95_beats_report_reconstruction() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut authoritative_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(reconstructed_receipt_sample());
            authoritative_samples.push(authoritative_receipt_sample());
        } else {
            authoritative_samples.push(authoritative_receipt_sample());
            legacy_samples.push(reconstructed_receipt_sample());
        }
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let authoritative_p95 = nearest_rank_p95(&authoritative_samples);
    let improvement_percent = legacy_p95
        .saturating_sub(authoritative_p95)
        .saturating_mul(100)
        / legacy_p95.max(1);
    println!(
        "PERF-RUNTIME74-AUTHORITATIVE-BINDING-APPLY sample_pairs={SAMPLE_PAIRS} receipts_per_sample={RECEIPTS_PER_SAMPLE} updates_per_receipt={UPDATES_PER_RECEIPT} pair_order=alternating_legacy_even legacy_first_pairs=11 authoritative_first_pairs=10 legacy_update_scans_per_receipt={UPDATES_PER_RECEIPT} authoritative_update_scans_per_receipt=0 legacy_p95_ns={legacy_p95} authoritative_p95_ns={authoritative_p95} improvement_percent={improvement_percent} improvement_threshold_percent=50 legacy_samples_ns={} authoritative_samples_ns={}",
        samples_csv(&legacy_samples),
        samples_csv(&authoritative_samples),
    );
    assert!(
        authoritative_p95.saturating_mul(2) <= legacy_p95,
        "authoritative apply receipt P95 must be at least 50% faster than reconstructing one from updates: legacy={legacy_p95}ns authoritative={authoritative_p95}ns"
    );
}

fn reconstructed_receipt_sample() -> u64 {
    let statuses = [
        UiBindingUpdateStatus::Applied,
        UiBindingUpdateStatus::Unchanged,
        UiBindingUpdateStatus::Applied,
        UiBindingUpdateStatus::Applied,
    ];
    let dirty = [
        UiBindingDirtyDomain::Layout,
        UiBindingDirtyDomain::HitTest,
        UiBindingDirtyDomain::Render,
        UiBindingDirtyDomain::Interaction,
    ];
    let started_at = Instant::now();
    for receipt_index in 0..RECEIPTS_PER_SAMPLE {
        let mut applied = 0usize;
        let mut unchanged = 0usize;
        let mut impact = Vec::with_capacity(dirty.len());
        for update_index in 0..UPDATES_PER_RECEIPT {
            match statuses[update_index % statuses.len()] {
                UiBindingUpdateStatus::Applied => applied += 1,
                UiBindingUpdateStatus::Unchanged => unchanged += 1,
                UiBindingUpdateStatus::Rejected => {}
            }
            let domain = dirty[update_index % dirty.len()];
            if !impact.contains(&domain) {
                impact.push(domain);
            }
        }
        let base_generation = receipt_index as u64;
        black_box(UiBindingMutationReceipt::committed(
            base_generation,
            base_generation.saturating_add(1),
            UPDATES_PER_RECEIPT,
            applied,
            unchanged,
            impact,
        ));
    }
    elapsed_nanos(started_at)
}

fn authoritative_receipt_sample() -> u64 {
    let impact = [
        UiBindingDirtyDomain::Layout,
        UiBindingDirtyDomain::HitTest,
        UiBindingDirtyDomain::Render,
        UiBindingDirtyDomain::Interaction,
    ];
    let started_at = Instant::now();
    for receipt_index in 0..RECEIPTS_PER_SAMPLE {
        let base_generation = receipt_index as u64;
        black_box(UiBindingMutationReceipt::committed(
            base_generation,
            base_generation.saturating_add(1),
            UPDATES_PER_RECEIPT,
            24,
            8,
            impact.to_vec(),
        ));
    }
    elapsed_nanos(started_at)
}

fn nearest_rank_p95(samples: &[u64]) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn elapsed_nanos(started_at: Instant) -> u64 {
    started_at.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn samples_csv(samples: &[u64]) -> String {
    samples
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
