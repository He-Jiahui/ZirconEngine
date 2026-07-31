use std::collections::HashMap;

use super::super::error::{CoreError, CoreResult};
use super::ModuleDescriptor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Visited,
}

pub fn sort_module_activation_order(descriptors: &[ModuleDescriptor]) -> CoreResult<Vec<String>> {
    let mut by_name: HashMap<&str, usize> = HashMap::with_capacity(descriptors.len());
    for (index, descriptor) in descriptors.iter().enumerate() {
        if by_name.insert(descriptor.name.as_str(), index).is_some() {
            return Err(CoreError::DuplicateModule(descriptor.name.clone()));
        }
    }

    for descriptor in descriptors {
        for dependency in &descriptor.module_dependencies {
            let Some(&dependency_index) = by_name.get(dependency.module_name.as_str()) else {
                return Err(CoreError::MissingModuleDependency {
                    module: descriptor.name.clone(),
                    dependency: dependency.module_name.clone(),
                });
            };
            let dependency_descriptor = &descriptors[dependency_index];
            if dependency_descriptor.init_level > descriptor.init_level {
                return Err(CoreError::ModuleInitLevelViolation {
                    module: descriptor.name.clone(),
                    module_level: descriptor.init_level.as_str().to_owned(),
                    dependency: dependency_descriptor.name.clone(),
                    dependency_level: dependency_descriptor.init_level.as_str().to_owned(),
                });
            }
        }
    }

    let mut traversal: Vec<usize> = (0..descriptors.len()).collect();
    traversal.sort_by_key(|&index| (descriptors[index].init_level, index));

    let mut states = vec![None; descriptors.len()];
    let mut stack = Vec::new();
    let mut order = Vec::with_capacity(descriptors.len());
    for index in traversal {
        visit_module(
            index,
            descriptors,
            &by_name,
            &mut states,
            &mut stack,
            &mut order,
        )?;
    }
    Ok(order)
}

fn visit_module(
    index: usize,
    descriptors: &[ModuleDescriptor],
    by_name: &HashMap<&str, usize>,
    states: &mut [Option<VisitState>],
    stack: &mut Vec<usize>,
    order: &mut Vec<String>,
) -> CoreResult<()> {
    match states[index] {
        Some(VisitState::Visited) => return Ok(()),
        Some(VisitState::Visiting) => {
            let name = &descriptors[index].name;
            let mut path = match stack.iter().position(|&candidate| candidate == index) {
                Some(cycle_start) => stack[cycle_start..]
                    .iter()
                    .map(|&module_index| descriptors[module_index].name.clone())
                    .collect(),
                None => Vec::new(),
            };
            path.push(name.clone());
            return Err(CoreError::ModuleDependencyCycle { path });
        }
        None => {}
    }

    states[index] = Some(VisitState::Visiting);
    stack.push(index);
    for dependency in &descriptors[index].module_dependencies {
        let Some(&dependency_index) = by_name.get(dependency.module_name.as_str()) else {
            return Err(CoreError::MissingModuleDependency {
                module: descriptors[index].name.clone(),
                dependency: dependency.module_name.clone(),
            });
        };
        visit_module(dependency_index, descriptors, by_name, states, stack, order)?;
    }
    stack.pop();
    states[index] = Some(VisitState::Visited);
    order.push(descriptors[index].name.clone());
    Ok(())
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn activation_sort_borrows_names_and_stacks_indices() {
        let source = include_str!("module_order.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("module order implementation");

        assert!(implementation.contains("HashMap<&str, usize>"));
        assert!(implementation.contains("stack: &mut Vec<usize>"));
        assert!(implementation.contains("stack.push(index);"));
        assert!(!implementation.contains("descriptor.name.clone(), index"));
    }
}
