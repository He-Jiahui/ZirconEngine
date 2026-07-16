use std::sync::Arc;

use crate::core::{LifecycleState, StartupMode};

use super::super::super::descriptors::{DependencySpec, RegistryName};
use super::super::super::state::{ServiceEntry, ServiceEntryFactory};

pub(super) fn service_entry(
    startup_mode: StartupMode,
    dependencies: &[DependencySpec],
    factory: ServiceEntryFactory,
) -> ServiceEntry {
    ServiceEntry {
        index: ServiceEntry::unassigned_index(),
        generation: ServiceEntry::initial_generation(),
        startup_mode,
        dependencies: dependency_names(dependencies),
        factory,
        lifecycle: LifecycleState::Registered,
        initialization_owner: None,
        instance: None,
    }
}

fn dependency_names(dependencies: &[DependencySpec]) -> Arc<[RegistryName]> {
    if dependencies.is_empty() {
        return Arc::default();
    }
    if let [dependency] = dependencies {
        return Arc::<[RegistryName]>::from([dependency.name.clone()]);
    }
    if let [first_dependency, second_dependency] = dependencies {
        return Arc::<[RegistryName]>::from([
            first_dependency.name.clone(),
            second_dependency.name.clone(),
        ]);
    }
    if let [first_dependency, second_dependency, third_dependency] = dependencies {
        return Arc::<[RegistryName]>::from([
            first_dependency.name.clone(),
            second_dependency.name.clone(),
            third_dependency.name.clone(),
        ]);
    }
    if let [first_dependency, second_dependency, third_dependency, fourth_dependency] = dependencies
    {
        return Arc::<[RegistryName]>::from([
            first_dependency.name.clone(),
            second_dependency.name.clone(),
            third_dependency.name.clone(),
            fourth_dependency.name.clone(),
        ]);
    }
    if let [first_dependency, second_dependency, third_dependency, fourth_dependency, fifth_dependency] =
        dependencies
    {
        return Arc::<[RegistryName]>::from([
            first_dependency.name.clone(),
            second_dependency.name.clone(),
            third_dependency.name.clone(),
            fourth_dependency.name.clone(),
            fifth_dependency.name.clone(),
        ]);
    }

    let mut names = Vec::with_capacity(dependencies.len());
    for dependency in dependencies {
        names.push(dependency.name.clone());
    }
    names.into()
}
