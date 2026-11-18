use std::collections::HashMap;

use crate::core::{sort_module_activation_order, CoreError, ModuleDescriptor};
use crate::plugin::{
    PluginModuleKind, RuntimePlugin, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn order_runtime_plugins<'a>(
    plugins: Vec<&'a dyn RuntimePlugin>,
) -> Result<Vec<&'a dyn RuntimePlugin>, CoreError> {
    let descriptors = plugins
        .iter()
        .map(|plugin| plugin.module_descriptor().clone())
        .collect::<Vec<_>>();

    let module_names = sort_module_activation_order(&descriptors)?;
    let by_name = descriptors
        .iter()
        .enumerate()
        .map(|(index, descriptor)| (descriptor.name.clone(), index))
        .collect::<HashMap<_, _>>();
    Ok(module_names
        .into_iter()
        .filter_map(|module_name| by_name.get(&module_name).map(|&index| plugins[index]))
        .collect())
}

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn order_runtime_plugin_descriptors(
    descriptors: Vec<RuntimePluginDescriptor>,
) -> Result<Vec<RuntimePluginDescriptor>, CoreError> {
    let module_descriptors = descriptors
        .iter()
        .map(|descriptor| descriptor.module_descriptor().clone())
        .collect::<Vec<_>>();

    let module_names = sort_module_activation_order(&module_descriptors)?;
    let mut by_name = descriptors
        .into_iter()
        .map(|descriptor| (descriptor.module_descriptor().name.clone(), descriptor))
        .collect::<HashMap<_, _>>();
    Ok(module_names
        .into_iter()
        .filter_map(|module_name| by_name.remove(&module_name))
        .collect())
}

// Sort runtime manifest rows, then take each report once at its first row.
// Reports without runtime rows keep their original order at the end.
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn order_runtime_plugin_registration_reports<
    'report,
>(
    registrations: &'report [RuntimePluginRegistrationReport],
    host_descriptors: &[ModuleDescriptor],
) -> Result<Vec<&'report RuntimePluginRegistrationReport>, CoreError> {
    let mut registration_indices = Vec::new();
    let mut module_descriptors = Vec::new();
    for (registration_index, registration) in registrations.iter().enumerate() {
        for module in registration
            .package_manifest
            .modules
            .iter()
            .filter(|module| module.kind == PluginModuleKind::Runtime)
        {
            registration_indices.push(registration_index);
            module_descriptors.push(module.module_descriptor());
        }
    }
    if module_descriptors.is_empty() {
        return Ok(registrations.iter().collect());
    }

    let report_module_count = module_descriptors.len();
    append_reached_host_modules(&mut module_descriptors, host_descriptors)?;
    let module_names = sort_module_activation_order(&module_descriptors)?;
    let by_name = module_descriptors[..report_module_count]
        .iter()
        .zip(registration_indices)
        .map(|(descriptor, registration_index)| (descriptor.name.as_str(), registration_index))
        .collect::<HashMap<_, _>>();
    let mut ordered_registration_indices = Vec::with_capacity(registrations.len());
    let mut registration_seen = vec![false; registrations.len()];
    for module_name in module_names {
        let Some(&registration_index) = by_name.get(module_name.as_str()) else {
            continue; // A host descriptor is an ordering input, never a report contribution.
        };
        if !registration_seen[registration_index] {
            registration_seen[registration_index] = true;
            ordered_registration_indices.push(registration_index);
        }
    }
    let metadata_only_registration_indices = (0..registrations.len())
        .filter(|registration_index| !registration_seen[*registration_index])
        .collect::<Vec<_>>();
    ordered_registration_indices.extend(metadata_only_registration_indices);

    Ok(ordered_registration_indices
        .into_iter()
        .map(|registration_index| &registrations[registration_index])
        .collect())
}

fn append_reached_host_modules(
    module_descriptors: &mut Vec<ModuleDescriptor>,
    host_descriptors: &[ModuleDescriptor],
) -> Result<(), CoreError> {
    if host_descriptors.is_empty() {
        return Ok(());
    }
    let mut candidates = host_descriptors
        .iter()
        .map(|descriptor| (descriptor.name.as_str(), descriptor))
        .collect::<HashMap<_, _>>();
    // Ownership is independent of reachability. Reject in manifest row order,
    // without adding the unrelated host descriptors or their dependency edges.
    if let Some(collision) = module_descriptors
        .iter()
        .find(|descriptor| candidates.contains_key(descriptor.name.as_str()))
    {
        return Err(CoreError::DuplicateModule(collision.name.clone()));
    }
    let report_module_count = module_descriptors.len();
    let mut pending = module_descriptors.iter().collect::<Vec<_>>();
    let mut next = 0;
    // Visit only the host closure reached from report rows, in declaration order.
    // Leave all edges intact for the existing strict sorter (including cycles).
    while let Some(descriptor) = pending.get(next).copied() {
        pending.extend(
            descriptor
                .module_dependencies
                .iter()
                .filter_map(|dependency| candidates.remove(dependency.module_name.as_str())),
        );
        next += 1;
    }
    let reached = pending
        .into_iter()
        .skip(report_module_count)
        .cloned()
        .collect::<Vec<_>>();
    module_descriptors.extend(reached);
    Ok(())
}

#[cfg(test)]
#[path = "tests/order.rs"]
mod tests;
