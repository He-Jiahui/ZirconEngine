use std::collections::HashMap;

use super::super::super::descriptors::RegistryName;
use super::super::super::state::ServiceEntry;
use super::blocked_dependencies::{
    dependency_slice_contains_service, first_blocked_five_service_dependency,
    first_blocked_four_service_dependency, first_blocked_three_service_dependency,
    first_blocked_two_service_dependency, FiveServiceDependencyMatch, FourServiceDependencyMatch,
    ThreeServiceDependencyMatch, TwoServiceDependencyMatch,
};

const BLOCKED_DEPENDENT_INITIAL_CAPACITY: usize = 1;

pub(super) fn first_blocked_unload(
    services: &HashMap<RegistryName, ServiceEntry>,
    unload_order: &[RegistryName],
) -> Option<(String, Vec<String>)> {
    if let [service_name] = unload_order {
        return first_blocked_single_service_unload(services, service_name);
    }
    if let [first_service_name, second_service_name] = unload_order {
        return first_blocked_two_service_unload(services, first_service_name, second_service_name);
    }
    if let [first_service_name, second_service_name, third_service_name] = unload_order {
        return first_blocked_three_service_unload(
            services,
            first_service_name,
            second_service_name,
            third_service_name,
        );
    }
    if let [first_service_name, second_service_name, third_service_name, fourth_service_name] =
        unload_order
    {
        return first_blocked_four_service_unload(
            services,
            first_service_name,
            second_service_name,
            third_service_name,
            fourth_service_name,
        );
    }
    if let [first_service_name, second_service_name, third_service_name, fourth_service_name, fifth_service_name] =
        unload_order
    {
        return first_blocked_five_service_unload(
            services,
            first_service_name,
            second_service_name,
            third_service_name,
            fourth_service_name,
            fifth_service_name,
        );
    }

    let mut unload_indices: HashMap<&RegistryName, usize> =
        HashMap::with_capacity(unload_order.len());
    for (index, service_name) in unload_order.iter().enumerate() {
        unload_indices.insert(service_name, index);
    }
    let mut blocked_index = None;
    let mut blocked_dependents = None;

    for (dependent_name, entry) in services.iter() {
        if unload_indices.contains_key(dependent_name) || entry.instance.is_none() {
            continue;
        }

        for dependency in entry.dependencies.iter() {
            if let Some(index) = unload_indices.get(dependency).copied() {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    index,
                    dependent_name,
                );
            }
        }
    }

    match (blocked_index, blocked_dependents) {
        (Some(index), Some(dependents)) => {
            Some(owned_blocked_result(&unload_order[index], dependents))
        }
        _ => None,
    }
}

fn first_blocked_single_service_unload(
    services: &HashMap<RegistryName, ServiceEntry>,
    service_name: &RegistryName,
) -> Option<(String, Vec<String>)> {
    let mut blocked_dependents: Option<Vec<String>> = None;

    for (dependent_name, entry) in services.iter() {
        if dependent_name == service_name || entry.instance.is_none() {
            continue;
        }

        if dependency_slice_contains_service(entry.dependencies.as_ref(), service_name) {
            blocked_dependents
                .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                .push(dependent_name);
        }
    }

    let Some(dependents) = blocked_dependents else {
        return None;
    };
    Some(owned_blocked_result(service_name, dependents))
}

fn first_blocked_two_service_unload(
    services: &HashMap<RegistryName, ServiceEntry>,
    first_service_name: &RegistryName,
    second_service_name: &RegistryName,
) -> Option<(String, Vec<String>)> {
    let mut blocked_index = None;
    let mut blocked_dependents = None;

    for (dependent_name, entry) in services.iter() {
        if dependent_name == first_service_name
            || dependent_name == second_service_name
            || entry.instance.is_none()
        {
            continue;
        }

        match first_blocked_two_service_dependency(
            entry.dependencies.as_ref(),
            first_service_name,
            second_service_name,
        ) {
            Some(TwoServiceDependencyMatch::FirstService) => {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    0,
                    dependent_name,
                );
            }
            Some(TwoServiceDependencyMatch::SecondService) => {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    1,
                    dependent_name,
                );
            }
            None => {}
        }
    }

    blocked_exact_service_result(
        [first_service_name, second_service_name],
        blocked_index,
        blocked_dependents,
    )
}

fn first_blocked_three_service_unload(
    services: &HashMap<RegistryName, ServiceEntry>,
    first_service_name: &RegistryName,
    second_service_name: &RegistryName,
    third_service_name: &RegistryName,
) -> Option<(String, Vec<String>)> {
    let mut blocked_index = None;
    let mut blocked_dependents = None;

    for (dependent_name, entry) in services.iter() {
        if dependent_name == first_service_name
            || dependent_name == second_service_name
            || dependent_name == third_service_name
            || entry.instance.is_none()
        {
            continue;
        }

        match first_blocked_three_service_dependency(
            entry.dependencies.as_ref(),
            first_service_name,
            second_service_name,
            third_service_name,
        ) {
            Some(ThreeServiceDependencyMatch::FirstService) => {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    0,
                    dependent_name,
                );
            }
            Some(ThreeServiceDependencyMatch::SecondService) => {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    1,
                    dependent_name,
                );
            }
            Some(ThreeServiceDependencyMatch::ThirdService) => {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    2,
                    dependent_name,
                );
            }
            None => {}
        }
    }

    blocked_exact_service_result(
        [first_service_name, second_service_name, third_service_name],
        blocked_index,
        blocked_dependents,
    )
}

fn first_blocked_four_service_unload(
    services: &HashMap<RegistryName, ServiceEntry>,
    first_service_name: &RegistryName,
    second_service_name: &RegistryName,
    third_service_name: &RegistryName,
    fourth_service_name: &RegistryName,
) -> Option<(String, Vec<String>)> {
    let mut blocked_index = None;
    let mut blocked_dependents = None;

    for (dependent_name, entry) in services.iter() {
        if dependent_name == first_service_name
            || dependent_name == second_service_name
            || dependent_name == third_service_name
            || dependent_name == fourth_service_name
            || entry.instance.is_none()
        {
            continue;
        }

        match first_blocked_four_service_dependency(
            entry.dependencies.as_ref(),
            first_service_name,
            second_service_name,
            third_service_name,
            fourth_service_name,
        ) {
            Some(FourServiceDependencyMatch::FirstService) => {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    0,
                    dependent_name,
                );
            }
            Some(FourServiceDependencyMatch::SecondService) => {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    1,
                    dependent_name,
                );
            }
            Some(FourServiceDependencyMatch::ThirdService) => {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    2,
                    dependent_name,
                );
            }
            Some(FourServiceDependencyMatch::FourthService) => {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    3,
                    dependent_name,
                );
            }
            None => {}
        }
    }

    blocked_exact_service_result(
        [
            first_service_name,
            second_service_name,
            third_service_name,
            fourth_service_name,
        ],
        blocked_index,
        blocked_dependents,
    )
}

fn first_blocked_five_service_unload(
    services: &HashMap<RegistryName, ServiceEntry>,
    first_service_name: &RegistryName,
    second_service_name: &RegistryName,
    third_service_name: &RegistryName,
    fourth_service_name: &RegistryName,
    fifth_service_name: &RegistryName,
) -> Option<(String, Vec<String>)> {
    let mut blocked_index = None;
    let mut blocked_dependents = None;

    for (dependent_name, entry) in services.iter() {
        if dependent_name == first_service_name
            || dependent_name == second_service_name
            || dependent_name == third_service_name
            || dependent_name == fourth_service_name
            || dependent_name == fifth_service_name
            || entry.instance.is_none()
        {
            continue;
        }

        match first_blocked_five_service_dependency(
            entry.dependencies.as_ref(),
            first_service_name,
            second_service_name,
            third_service_name,
            fourth_service_name,
            fifth_service_name,
        ) {
            Some(FiveServiceDependencyMatch::FirstService) => {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    0,
                    dependent_name,
                );
            }
            Some(FiveServiceDependencyMatch::SecondService) => {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    1,
                    dependent_name,
                );
            }
            Some(FiveServiceDependencyMatch::ThirdService) => {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    2,
                    dependent_name,
                );
            }
            Some(FiveServiceDependencyMatch::FourthService) => {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    3,
                    dependent_name,
                );
            }
            Some(FiveServiceDependencyMatch::FifthService) => {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    4,
                    dependent_name,
                );
            }
            None => {}
        }
    }

    blocked_exact_service_result(
        [
            first_service_name,
            second_service_name,
            third_service_name,
            fourth_service_name,
            fifth_service_name,
        ],
        blocked_index,
        blocked_dependents,
    )
}

fn blocked_exact_service_result<const SERVICE_COUNT: usize>(
    unload_order: [&RegistryName; SERVICE_COUNT],
    blocked_index: Option<usize>,
    blocked_dependents: Option<Vec<&RegistryName>>,
) -> Option<(String, Vec<String>)> {
    match (blocked_index, blocked_dependents) {
        (Some(index), Some(dependents)) => {
            Some(owned_blocked_result(unload_order[index], dependents))
        }
        _ => None,
    }
}

fn record_blocked_dependent<'a>(
    blocked_index: &mut Option<usize>,
    blocked_dependents: &mut Option<Vec<&'a RegistryName>>,
    index: usize,
    dependent_name: &'a RegistryName,
) {
    match *blocked_index {
        Some(current_index) if index > current_index => {}
        Some(current_index) if index == current_index => {
            blocked_dependents
                .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                .push(dependent_name);
        }
        _ => {
            *blocked_index = Some(index);
            let dependents = blocked_dependents
                .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY));
            dependents.clear();
            dependents.push(dependent_name);
        }
    }
}

fn owned_blocked_result(
    service_name: &RegistryName,
    dependents: Vec<&RegistryName>,
) -> (String, Vec<String>) {
    (
        service_name.to_string(),
        dependents
            .into_iter()
            .map(|dependent| dependent.as_str().to_owned())
            .collect(),
    )
}

#[cfg(test)]
mod optimization_batch_fo_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::ServiceKind;

    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const SCANS_PER_SAMPLE: usize = 2_048;
    const DEPENDENTS_PER_INDEX: usize = 16;

    #[test]
    fn optimization_batch_fo_runtime471_materializes_only_final_blocked_dependents() {
        let names = representative_dependents();
        let events = descending_blocked_events(&names);
        let service_name = registry_name("BlockedService");
        let mut blocked_index = None;
        let mut blocked_dependents = None;

        for &(index, dependent) in &events {
            record_blocked_dependent(
                &mut blocked_index,
                &mut blocked_dependents,
                index,
                dependent,
            );
        }

        assert_eq!(blocked_index, Some(0));
        let borrowed = blocked_dependents.unwrap();
        assert_eq!(borrowed.len(), DEPENDENTS_PER_INDEX);
        assert!(borrowed
            .iter()
            .zip(names.iter().skip(5 * DEPENDENTS_PER_INDEX))
            .all(|(left, right)| std::ptr::eq(*left, right)));

        let (blocked, dependents) = owned_blocked_result(&service_name, borrowed);
        assert_eq!(blocked, service_name.as_str());
        assert_eq!(dependents.len(), DEPENDENTS_PER_INDEX);
        assert_eq!(dependents[0], names[5 * DEPENDENTS_PER_INDEX].as_str());
    }

    #[test]
    #[ignore = "release performance gate"]
    fn optimization_batch_fo_runtime471_borrowed_blocked_dependents_benchmark() {
        let names = representative_dependents();
        let events = descending_blocked_events(&names);
        let service_name = registry_name("BlockedService");

        for _ in 0..4 {
            black_box(measure_legacy(&service_name, &events));
            black_box(measure_optimized(&service_name, &events));
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_samples.push(measure_legacy(&service_name, &events));
                optimized_samples.push(measure_optimized(&service_name, &events));
            } else {
                optimized_samples.push(measure_optimized(&service_name, &events));
                legacy_samples.push(measure_legacy(&service_name, &events));
            }
        }

        report_performance(&legacy_samples, &optimized_samples);
    }

    fn representative_dependents() -> Vec<RegistryName> {
        (0..6 * DEPENDENTS_PER_INDEX)
            .map(|index| registry_name(&format!("DependentService{index:03}")))
            .collect()
    }

    fn registry_name(service: &str) -> RegistryName {
        RegistryName::from_parts("Runtime.Optimization", ServiceKind::Manager, service)
    }

    fn descending_blocked_events(names: &[RegistryName]) -> Vec<(usize, &RegistryName)> {
        names
            .chunks_exact(DEPENDENTS_PER_INDEX)
            .enumerate()
            .flat_map(|(group, names)| names.iter().map(move |name| (5 - group, name)))
            .collect()
    }

    fn measure_legacy(service_name: &RegistryName, events: &[(usize, &RegistryName)]) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..SCANS_PER_SAMPLE {
            let mut blocked_index = None;
            let mut blocked_dependents = None;
            for &(index, dependent_name) in black_box(events) {
                legacy_record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    index,
                    dependent_name,
                );
            }
            let result = (
                service_name.to_string(),
                blocked_dependents.expect("blocked dependents"),
            );
            checksum = checksum
                .wrapping_add(result.0.len())
                .wrapping_add(result.1.iter().map(String::len).sum::<usize>());
            black_box(result);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(service_name: &RegistryName, events: &[(usize, &RegistryName)]) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_usize;
        for _ in 0..SCANS_PER_SAMPLE {
            let mut blocked_index = None;
            let mut blocked_dependents = None;
            for &(index, dependent_name) in black_box(events) {
                record_blocked_dependent(
                    &mut blocked_index,
                    &mut blocked_dependents,
                    index,
                    dependent_name,
                );
            }
            let result = owned_blocked_result(
                service_name,
                blocked_dependents.expect("blocked dependents"),
            );
            checksum = checksum
                .wrapping_add(result.0.len())
                .wrapping_add(result.1.iter().map(String::len).sum::<usize>());
            black_box(result);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn legacy_record_blocked_dependent(
        blocked_index: &mut Option<usize>,
        blocked_dependents: &mut Option<Vec<String>>,
        index: usize,
        dependent_name: &RegistryName,
    ) {
        match *blocked_index {
            Some(current_index) if index > current_index => {}
            Some(current_index) if index == current_index => blocked_dependents
                .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY))
                .push(dependent_name.as_str().to_owned()),
            _ => {
                *blocked_index = Some(index);
                let dependents = blocked_dependents
                    .get_or_insert_with(|| Vec::with_capacity(BLOCKED_DEPENDENT_INITIAL_CAPACITY));
                dependents.clear();
                dependents.push(dependent_name.as_str().to_owned());
            }
        }
    }

    fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
        let legacy_p95 = nearest_rank_p95(legacy_samples);
        let optimized_p95 = nearest_rank_p95(optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "RUNTIME471_BORROWED_BLOCKED_DEPENDENTS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} scans_per_sample={SCANS_PER_SAMPLE} events_per_scan={} legacy_name_copies_per_scan={} optimized_name_copies_per_scan={} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=40",
            6 * DEPENDENTS_PER_INDEX,
            6 * DEPENDENTS_PER_INDEX + 1,
            DEPENDENTS_PER_INDEX + 1,
            csv(legacy_samples),
            csv(optimized_samples),
        );
        assert!(
            optimized_p95 <= legacy_p95.saturating_mul(60) / 100,
            "borrowed blocked-dependent tracking must reduce P95 by at least 40%"
        );
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
