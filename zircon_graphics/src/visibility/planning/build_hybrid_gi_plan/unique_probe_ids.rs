use std::collections::BTreeSet;

pub(super) fn unique_probe_ids(probes: impl IntoIterator<Item = u32>, budget: usize) -> Vec<u32> {
    if budget == 0 {
        return Vec::new();
    }

    let mut seen = BTreeSet::new();
    let mut unique = Vec::new();
    for probe_id in probes {
        if seen.insert(probe_id) {
            unique.push(probe_id);
            if unique.len() == budget {
                break;
            }
        }
    }
    unique
}
