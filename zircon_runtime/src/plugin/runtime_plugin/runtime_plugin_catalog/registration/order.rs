use std::collections::HashMap;

use crate::core::{sort_module_activation_order, CoreError};
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

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn order_runtime_plugin_registration_reports(
    registrations: &[RuntimePluginRegistrationReport],
) -> Result<Vec<&RuntimePluginRegistrationReport>, CoreError> {
    order_runtime_plugin_registration_report_refs(registrations.iter().collect())
}

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn order_runtime_plugin_registration_report_refs(
    registrations: Vec<&RuntimePluginRegistrationReport>,
) -> Result<Vec<&RuntimePluginRegistrationReport>, CoreError> {
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
        return Ok(registrations);
    }

    let module_names = sort_module_activation_order(&module_descriptors)?;
    let by_name = module_descriptors
        .iter()
        .enumerate()
        .map(|(module_index, descriptor)| (descriptor.name.as_str(), module_index))
        .collect::<HashMap<_, _>>();
    let mut ordered_registration_indices = Vec::with_capacity(registrations.len());
    let mut registration_seen = vec![false; registrations.len()];
    for module_name in module_names {
        let module_index = by_name[&module_name.as_str()];
        let registration_index = registration_indices[module_index];
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
        .map(|registration_index| registrations[registration_index])
        .collect())
}

#[cfg(test)]
mod tests {
    #[test]
    fn registration_order_uses_constant_time_seen_membership() {
        let source = include_str!("order.rs");
        let linear_membership = ["ordered_registration_indices", ".contains("].concat();
        assert!(!source.contains(&linear_membership));
    }
}
