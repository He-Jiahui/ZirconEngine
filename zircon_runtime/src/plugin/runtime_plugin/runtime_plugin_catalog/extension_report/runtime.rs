use crate::plugin::RuntimeExtensionRegistry;

use super::super::extension_merge::merge_runtime_extensions;
use super::super::registration::order::order_runtime_plugin_registration_reports;
use super::super::RuntimePluginRegistrationReport;
use super::RuntimeExtensionCatalogReport;

// 全量报告先验证模块依赖顺序；失败时 finalize 空 registry 并同时返回 fatal，成功后逐登记合并再 finalize。
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn runtime_extension_report(
    registrations: &[RuntimePluginRegistrationReport],
) -> RuntimeExtensionCatalogReport {
    let mut registry = RuntimeExtensionRegistry::default();
    let mut diagnostics = Vec::with_capacity(registrations.len());
    let mut fatal_diagnostics = Vec::with_capacity(registrations.len());
    let registrations = match order_runtime_plugin_registration_reports(registrations) {
        Ok(registrations) => registrations,
        Err(error) => {
            let diagnostic = format!("runtime plugin module descriptor ordering failed: {error}");
            diagnostics.push(diagnostic.clone());
            fatal_diagnostics.push(diagnostic);
            registry.finalize();
            return RuntimeExtensionCatalogReport {
                registry,
                diagnostics,
                fatal_diagnostics,
            };
        }
    };
    for registration in registrations {
        merge_runtime_extensions(
            registration,
            &mut registry,
            &mut diagnostics,
            &mut fatal_diagnostics,
        );
    }
    registry.finalize();
    RuntimeExtensionCatalogReport {
        registry,
        diagnostics,
        fatal_diagnostics,
    }
}

#[cfg(test)]
#[path = "tests/runtime_optimization_batch_20260830br_runtime_tests.rs"]
mod optimization_batch_20260830br_runtime_tests;
