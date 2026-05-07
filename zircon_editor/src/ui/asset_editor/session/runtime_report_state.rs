use zircon_runtime::ui::template::{
    collect_document_localization_report, collect_document_resource_dependencies,
    validate_document_action_policy,
};
use zircon_runtime_interface::ui::template::{
    UiActionHostPolicy, UiActionPolicyDiagnostic, UiActionPolicyReport, UiActionSideEffectClass,
    UiLocalizationReport, UiResourceCollectionReport, UiResourceDependency, UiResourceDiagnostic,
};

use super::ui_asset_editor_session::UiAssetEditorSession;

pub(super) const DEFAULT_LOCALE_PREVIEW: &str = "authoring-fallback";
const LOCALE_PREVIEW_OPTIONS: [&str; 3] = [DEFAULT_LOCALE_PREVIEW, "en-US", "zh-CN"];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct UiAssetRuntimeReportProjection {
    pub(super) action_policy_items: Vec<String>,
    pub(super) capability_explanation_items: Vec<String>,
    pub(super) host_enforcement_items: Vec<String>,
    pub(super) unsafe_action_guidance_items: Vec<String>,
    pub(super) locale_preview_items: Vec<String>,
    pub(super) locale_preview_selected_locale: String,
    pub(super) locale_preview_selected_index: i32,
    pub(super) locale_dependency_items: Vec<String>,
    pub(super) locale_extraction_items: Vec<String>,
    pub(super) locale_diagnostic_items: Vec<String>,
    pub(super) resource_dependency_items: Vec<String>,
    pub(super) resource_diagnostic_items: Vec<String>,
}

impl UiAssetEditorSession {
    pub fn set_locale_preview(&mut self, locale: impl Into<String>) -> bool {
        let locale = locale.into();
        let locale = locale.trim();
        let normalized = if LOCALE_PREVIEW_OPTIONS.contains(&locale) {
            locale
        } else {
            DEFAULT_LOCALE_PREVIEW
        };
        if self.selected_locale_preview == normalized {
            return false;
        }
        self.selected_locale_preview = normalized.to_string();
        self.refresh_structured_diagnostics_for_current_document();
        true
    }

    pub(super) fn runtime_report_projection(&self) -> UiAssetRuntimeReportProjection {
        let runtime_policy = UiActionHostPolicy::runtime_default();
        let editor_policy = UiActionHostPolicy::editor_authoring();
        let runtime_action_policy =
            validate_document_action_policy(&self.last_valid_document, &runtime_policy);
        let editor_action_policy =
            validate_document_action_policy(&self.last_valid_document, &editor_policy);
        let localization = collect_document_localization_report(&self.last_valid_document);
        let (resource_report, resource_error) = collect_document_resource_dependencies(
            &self.last_valid_document,
            &self.compiler_imports.widgets,
            &self.compiler_imports.styles,
        )
        .map(|report| (report, None))
        .unwrap_or_else(|error| {
            (
                UiResourceCollectionReport::default(),
                Some(format!("Error resource collection: {error}")),
            )
        });

        let mut resource_diagnostic_rows = resource_diagnostic_items(&resource_report.diagnostics);
        if !self.resource_diagnostics.is_empty() {
            resource_diagnostic_rows = resource_diagnostic_items(&self.resource_diagnostics);
        }
        if let Some(error) = resource_error {
            resource_diagnostic_rows.push(error);
        }
        let mut locale_diagnostics = localization.diagnostics.clone();
        locale_diagnostics.extend(self.localization_resolver_diagnostics());

        UiAssetRuntimeReportProjection {
            action_policy_items: editor_action_policy
                .diagnostics
                .iter()
                .map(action_policy_item)
                .collect(),
            capability_explanation_items: capability_explanation_items(),
            host_enforcement_items: host_enforcement_items(
                &runtime_policy,
                &runtime_action_policy,
                &editor_policy,
                &editor_action_policy,
            ),
            unsafe_action_guidance_items: unsafe_action_guidance_items(
                &runtime_policy,
                &runtime_action_policy,
                &editor_policy,
                &editor_action_policy,
            ),
            locale_preview_items: locale_preview_items(&localization),
            locale_preview_selected_locale: self.selected_locale_preview.clone(),
            locale_preview_selected_index: locale_preview_selected_index(
                &self.selected_locale_preview,
            ),
            locale_dependency_items: localization
                .dependencies
                .iter()
                .map(|dependency| {
                    let table = dependency.reference.table.as_deref().unwrap_or("default");
                    let fallback = dependency.reference.fallback.as_deref().unwrap_or("<none>");
                    format!(
                        "{} -> {}:{} ({:?}, fallback = {:?})",
                        dependency.path,
                        table,
                        dependency.reference.key,
                        dependency.direction,
                        fallback
                    )
                })
                .collect(),
            locale_extraction_items: localization
                .extraction_candidates
                .iter()
                .map(|candidate| format!("{} -> {:?}", candidate.path, candidate.text))
                .collect(),
            locale_diagnostic_items: locale_diagnostics
                .iter()
                .map(|diagnostic| {
                    format!(
                        "{:?} [{}] {}: {}",
                        diagnostic.severity,
                        localization_diagnostic_code(diagnostic),
                        diagnostic.path,
                        diagnostic.message
                    )
                })
                .collect(),
            resource_dependency_items: self
                .resource_dependencies
                .iter()
                .map(resource_dependency_item)
                .collect(),
            resource_diagnostic_items: resource_diagnostic_rows,
        }
    }
}

fn localization_diagnostic_code(
    diagnostic: &zircon_runtime_interface::ui::template::UiLocalizationDiagnostic,
) -> &str {
    if diagnostic.code.is_empty() {
        "localization_invalid_ref"
    } else {
        &diagnostic.code
    }
}

fn action_policy_item(diagnostic: &UiActionPolicyDiagnostic) -> String {
    let route = diagnostic.route.as_deref().unwrap_or("<none>");
    let action = diagnostic.action.as_deref().unwrap_or("<none>");
    format!(
        "{:?} node={} binding={} route={} action={} side_effect={:?}: {}",
        diagnostic.severity,
        diagnostic.node_id,
        diagnostic.binding_id,
        route,
        action,
        diagnostic.side_effect,
        diagnostic.message
    )
}

fn capability_explanation_items() -> Vec<String> {
    vec![
        format!(
            "allowed side effects: {}",
            side_effect_list([
                UiActionSideEffectClass::LocalUi,
                UiActionSideEffectClass::EditorMutation,
                UiActionSideEffectClass::AssetIo,
            ])
        ),
        format!(
            "blocked side effects: {}",
            side_effect_list([
                UiActionSideEffectClass::SceneMutation,
                UiActionSideEffectClass::ExternalProcess,
                UiActionSideEffectClass::Network,
            ])
        ),
    ]
}

fn host_enforcement_items(
    runtime_policy: &UiActionHostPolicy,
    runtime_report: &UiActionPolicyReport,
    editor_policy: &UiActionHostPolicy,
    editor_report: &UiActionPolicyReport,
) -> Vec<String> {
    let mut items = vec![
        format!(
            "runtime-default allowed side effects: {}",
            side_effects_from_policy(runtime_policy)
        ),
        format!(
            "editor-authoring allowed side effects: {}",
            side_effects_from_policy(editor_policy)
        ),
    ];
    items.extend(policy_enforcement_items("runtime-default", runtime_report));
    items.extend(policy_enforcement_items("editor-authoring", editor_report));
    items
}

fn policy_enforcement_items(profile: &str, report: &UiActionPolicyReport) -> Vec<String> {
    if report.diagnostics.is_empty() {
        return vec![format!("{profile} allowed: no blocked action bindings")];
    }
    report
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let route = diagnostic.route.as_deref().unwrap_or("<none>");
            let action = diagnostic.action.as_deref().unwrap_or("<none>");
            format!(
                "{profile} blocked node={} binding={} route={} action={} side_effect={:?}",
                diagnostic.node_id, diagnostic.binding_id, route, action, diagnostic.side_effect
            )
        })
        .collect()
}

fn unsafe_action_guidance_items(
    runtime_policy: &UiActionHostPolicy,
    runtime_report: &UiActionPolicyReport,
    editor_policy: &UiActionHostPolicy,
    editor_report: &UiActionPolicyReport,
) -> Vec<String> {
    let mut items = Vec::new();
    for diagnostic in &editor_report.diagnostics {
        items.push(format!(
            "editor-authoring binding {} uses {:?}; explicit host capability required before authoring or packaging. Move unsafe work behind an approved host service.",
            action_binding_label(diagnostic),
            diagnostic.side_effect
        ));
    }
    for diagnostic in &runtime_report.diagnostics {
        if editor_policy.allows(diagnostic.side_effect) {
            items.push(format!(
                "runtime-default binding {} is editor-only {:?}; keep it in editor profile or replace it with a LocalUi runtime action.",
                action_binding_label(diagnostic),
                diagnostic.side_effect
            ));
        } else if !runtime_policy.allows(diagnostic.side_effect)
            && !editor_report.diagnostics.iter().any(|editor_diagnostic| {
                editor_diagnostic.node_id == diagnostic.node_id
                    && editor_diagnostic.binding_id == diagnostic.binding_id
            })
        {
            items.push(format!(
                "runtime-default binding {} uses {:?}; explicit host capability required before runtime packaging.",
                action_binding_label(diagnostic),
                diagnostic.side_effect
            ));
        }
    }
    if items.is_empty() {
        items.push("all action bindings are compatible with runtime-default and editor-authoring host policies".to_string());
    }
    items
}

fn action_binding_label(diagnostic: &UiActionPolicyDiagnostic) -> String {
    let route = diagnostic.route.as_deref().unwrap_or("<none>");
    let action = diagnostic.action.as_deref().unwrap_or("<none>");
    format!(
        "{} on node {} (route={}, action={})",
        diagnostic.binding_id, diagnostic.node_id, route, action
    )
}

fn side_effect_list<const N: usize>(items: [UiActionSideEffectClass; N]) -> String {
    items
        .into_iter()
        .map(|item| format!("{item:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn side_effects_from_policy(policy: &UiActionHostPolicy) -> String {
    policy
        .allowed_side_effects
        .iter()
        .map(|item| format!("{item:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn locale_preview_items(report: &UiLocalizationReport) -> Vec<String> {
    let key_summary = if report.dependencies.is_empty() {
        "no localized refs".to_string()
    } else {
        report
            .dependencies
            .iter()
            .map(|dependency| dependency.reference.key.clone())
            .collect::<Vec<_>>()
            .join(", ")
    };
    LOCALE_PREVIEW_OPTIONS
        .iter()
        .map(|locale| {
            format!(
                "{locale} • refs: {key_summary}; extraction candidates: {}",
                report.extraction_candidates.len()
            )
        })
        .collect()
}

fn locale_preview_selected_index(locale: &str) -> i32 {
    LOCALE_PREVIEW_OPTIONS
        .iter()
        .position(|candidate| *candidate == locale)
        .map(|index| index as i32)
        .unwrap_or(0)
}

fn resource_dependency_item(dependency: &UiResourceDependency) -> String {
    let fallback = dependency
        .reference
        .fallback
        .uri
        .as_deref()
        .unwrap_or("<none>");
    format!(
        "{:?} {:?} {} -> {} fallback={:?} {}",
        dependency.reference.kind,
        dependency.source,
        dependency.path,
        dependency.reference.uri,
        dependency.reference.fallback.mode,
        fallback
    )
}

fn resource_diagnostic_items(diagnostics: &[UiResourceDiagnostic]) -> Vec<String> {
    diagnostics
        .iter()
        .map(|diagnostic| {
            format!(
                "{:?} [{}] {}: {}",
                diagnostic.severity, diagnostic.code, diagnostic.path, diagnostic.message
            )
        })
        .collect()
}
