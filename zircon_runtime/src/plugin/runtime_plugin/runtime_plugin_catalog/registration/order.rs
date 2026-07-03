use std::collections::HashMap;

use crate::core::{sort_module_activation_order, ModuleDescriptor};
use crate::plugin::{RuntimePlugin, RuntimePluginDescriptor};

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn order_runtime_plugins<'a>(
    plugins: Vec<&'a dyn RuntimePlugin>,
) -> (Vec<&'a dyn RuntimePlugin>, Vec<String>) {
    let descriptors = plugins
        .iter()
        .map(|plugin| plugin.module_descriptor().clone())
        .collect::<Vec<_>>();

    match sort_module_activation_order(&descriptors) {
        Ok(module_names) => {
            let by_name = descriptors
                .iter()
                .enumerate()
                .map(|(index, descriptor)| (descriptor.name.clone(), index))
                .collect::<HashMap<_, _>>();
            let ordered = module_names
                .into_iter()
                .filter_map(|module_name| by_name.get(&module_name).map(|&index| plugins[index]))
                .collect::<Vec<_>>();
            (ordered, Vec::new())
        }
        Err(error) => (
            fallback_order_plugins(plugins),
            vec![format!(
                "runtime plugin module descriptor ordering failed: {error}"
            )],
        ),
    }
}

pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn order_runtime_plugin_descriptors(
    descriptors: Vec<RuntimePluginDescriptor>,
) -> (Vec<RuntimePluginDescriptor>, Vec<String>) {
    let module_descriptors = descriptors
        .iter()
        .map(|descriptor| descriptor.module_descriptor().clone())
        .collect::<Vec<_>>();

    match sort_module_activation_order(&module_descriptors) {
        Ok(module_names) => {
            let mut by_name = descriptors
                .into_iter()
                .map(|descriptor| (descriptor.module_descriptor().name.clone(), descriptor))
                .collect::<HashMap<_, _>>();
            let ordered = module_names
                .into_iter()
                .filter_map(|module_name| by_name.remove(&module_name))
                .collect::<Vec<_>>();
            (ordered, Vec::new())
        }
        Err(error) => (
            fallback_order_descriptors(descriptors),
            vec![format!(
                "runtime plugin module descriptor ordering failed: {error}"
            )],
        ),
    }
}

fn fallback_order_plugins<'a>(
    mut plugins: Vec<&'a dyn RuntimePlugin>,
) -> Vec<&'a dyn RuntimePlugin> {
    plugins.sort_by(|left, right| {
        compare_module_descriptors(left.module_descriptor(), right.module_descriptor()).then_with(
            || {
                left.descriptor()
                    .package_id()
                    .cmp(right.descriptor().package_id())
            },
        )
    });
    plugins
}

fn fallback_order_descriptors(
    mut descriptors: Vec<RuntimePluginDescriptor>,
) -> Vec<RuntimePluginDescriptor> {
    descriptors.sort_by(|left, right| {
        compare_module_descriptors(left.module_descriptor(), right.module_descriptor())
            .then_with(|| left.package_id().cmp(right.package_id()))
    });
    descriptors
}

fn compare_module_descriptors(
    left: &ModuleDescriptor,
    right: &ModuleDescriptor,
) -> std::cmp::Ordering {
    left.init_level
        .cmp(&right.init_level)
        .then_with(|| left.name.cmp(&right.name))
}
