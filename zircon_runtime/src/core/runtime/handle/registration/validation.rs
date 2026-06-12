use crate::core::CoreError;
use crate::core::ServiceKind;

use super::super::super::descriptors::{DependencySpec, RegistryName};

pub(super) fn is_canonical_module_name(name: &str) -> bool {
    !name.is_empty() && name.trim() == name
}

pub(super) fn validate_service_descriptor(
    owner_module: &str,
    kind: ServiceKind,
    name: &RegistryName,
    dependencies: &[DependencySpec],
) -> Result<(), CoreError> {
    let actual_owner = name.module_name();
    if actual_owner != owner_module {
        return Err(CoreError::ServiceOwnerMismatch {
            name: name.to_string(),
            expected: owner_module.to_owned(),
            actual: actual_owner.to_string(),
        });
    }
    let actual_kind = name.service_kind();
    if actual_kind != kind {
        return Err(CoreError::ServiceKindMismatch {
            name: name.to_string(),
            expected: kind,
            actual: actual_kind,
        });
    }
    validate_driver_dependencies(kind, name, dependencies)?;
    Ok(())
}

fn validate_driver_dependencies(
    kind: ServiceKind,
    name: &RegistryName,
    dependencies: &[DependencySpec],
) -> Result<(), CoreError> {
    if kind != ServiceKind::Driver {
        return Ok(());
    }
    if dependencies.is_empty() {
        return Ok(());
    }
    if let [dependency] = dependencies {
        return validate_driver_dependency_kind(kind, name, dependency);
    }
    if let [first_dependency, second_dependency] = dependencies {
        validate_driver_dependency_kind(kind, name, first_dependency)?;
        return validate_driver_dependency_kind(kind, name, second_dependency);
    }
    if let [first_dependency, second_dependency, third_dependency] = dependencies {
        validate_driver_dependency_kind(kind, name, first_dependency)?;
        validate_driver_dependency_kind(kind, name, second_dependency)?;
        return validate_driver_dependency_kind(kind, name, third_dependency);
    }
    if let [first_dependency, second_dependency, third_dependency, fourth_dependency] = dependencies
    {
        validate_driver_dependency_kind(kind, name, first_dependency)?;
        validate_driver_dependency_kind(kind, name, second_dependency)?;
        validate_driver_dependency_kind(kind, name, third_dependency)?;
        return validate_driver_dependency_kind(kind, name, fourth_dependency);
    }
    if let [first_dependency, second_dependency, third_dependency, fourth_dependency, fifth_dependency] =
        dependencies
    {
        validate_driver_dependency_kind(kind, name, first_dependency)?;
        validate_driver_dependency_kind(kind, name, second_dependency)?;
        validate_driver_dependency_kind(kind, name, third_dependency)?;
        validate_driver_dependency_kind(kind, name, fourth_dependency)?;
        return validate_driver_dependency_kind(kind, name, fifth_dependency);
    }
    for dependency in dependencies {
        validate_driver_dependency_kind(kind, name, dependency)?;
    }
    Ok(())
}

fn validate_driver_dependency_kind(
    kind: ServiceKind,
    name: &RegistryName,
    dependency: &DependencySpec,
) -> Result<(), CoreError> {
    let dependency_kind = dependency.name.service_kind();
    if dependency_kind == ServiceKind::Driver {
        return Ok(());
    }
    Err(CoreError::InvalidServiceDependencyKind {
        service: name.to_string(),
        service_kind: kind,
        dependency: dependency.name.to_string(),
        dependency_kind,
    })
}
