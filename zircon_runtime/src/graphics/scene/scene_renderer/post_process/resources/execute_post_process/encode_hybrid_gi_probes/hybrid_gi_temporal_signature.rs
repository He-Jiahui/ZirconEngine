use std::collections::BTreeSet;

use crate::core::framework::render::RenderHybridGiExtract;

const TEMPORAL_SIGNATURE_BUCKETS: u32 = 255;
const PROBE_SIGNATURE_SEED: u32 = 0x9E37_79B9;
const PARENT_SIGNATURE_SEED: u32 = 0x85EB_CA77;
const NO_PARENT_SIGNATURE_SEED: u32 = 0xA5A5_5A5A;
const ANCESTOR_SIGNATURE_DEPTH_SEED: u32 = 0x27D4_EB2F;

pub(super) fn hybrid_gi_temporal_signature(
    extract: Option<&RenderHybridGiExtract>,
    probe_id: u32,
    parent_probe_id: Option<u32>,
) -> f32 {
    let parent_signature = parent_probe_id.unwrap_or(NO_PARENT_SIGNATURE_SEED);
    let mut mixed_signature = mix_signature_words(
        probe_id ^ PROBE_SIGNATURE_SEED,
        parent_signature ^ PARENT_SIGNATURE_SEED,
    );
    let mut next_parent_probe_id = parent_probe_id;
    let mut visited_probe_ids = BTreeSet::from([probe_id]);
    let mut depth = 0u32;
    while let Some(ancestor_probe_id) = next_parent_probe_id {
        if !visited_probe_ids.insert(ancestor_probe_id) {
            break;
        }
        depth += 1;
        mixed_signature = mix_signature_words(
            mixed_signature ^ ancestor_probe_id.rotate_left(depth % u32::BITS),
            ancestor_probe_id ^ ANCESTOR_SIGNATURE_DEPTH_SEED.wrapping_mul(depth),
        );
        next_parent_probe_id =
            extract.and_then(|extract| probe_parent_id(extract, ancestor_probe_id));
    }
    let signature_bucket = 1 + (mixed_signature % TEMPORAL_SIGNATURE_BUCKETS);
    signature_bucket as f32 / TEMPORAL_SIGNATURE_BUCKETS as f32
}

fn mix_signature_words(left: u32, right: u32) -> u32 {
    let mut mixed = left.wrapping_add(0x7FEB_352D).wrapping_mul(0x846C_A68B);
    mixed ^= right.rotate_left(16);
    mixed ^= mixed >> 15;
    mixed = mixed.wrapping_mul(0x2C1B_3C6D);
    mixed ^ (mixed >> 12)
}

fn probe_parent_id(extract: &RenderHybridGiExtract, probe_id: u32) -> Option<u32> {
    extract
        .probes
        .iter()
        .find(|probe| probe.probe_id == probe_id)
        .and_then(|probe| probe.parent_probe_id)
}
