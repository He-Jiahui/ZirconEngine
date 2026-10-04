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

/// 返回仍持有实例的外部服务所阻挡的首个卸载项及其依赖者。
/// 一至五项走定长匹配；更长清单才构建通用索引。
// TODO: [CR-R02-core_handles-0001] services 是 HashMap；多个依赖者会按迭代次序进入错误 Vec，需确认诊断是否要求跨运行稳定排序。
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
    let mut blocked_dependents: Option<Vec<&RegistryName>> = None;

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

// 只保留卸载顺序最靠前项的全部依赖者；发现更早项时丢弃旧候选。
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

// 扫描期间借用注册键；最终只复制阻断项及其依赖者，供调用方释放服务表锁后返回错误。
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
#[path = "tests/blocked_unload_optimization_batch_fo_tests.rs"]
mod optimization_batch_fo_tests;
