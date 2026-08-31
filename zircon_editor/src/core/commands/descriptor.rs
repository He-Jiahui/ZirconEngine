use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::editor_event::EditorEvent;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::i18n::{EditorI18nService, EditorLocale, EditorLocalizationBundle};

use super::{
    AssetWriteTargetDescriptor, CommandEvalCtx, EditorCommandExecutionContract,
    EditorCommandMenuPath, EditorCommandPresentation, EditorKeyChord, WhenClause,
};

/// Canonical metadata and executable route for every editor command.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorCommandDescriptor {
    id: EditorOperationPath,
    presentation: EditorCommandPresentation,
    category: EditorCommandCategory,
    menu_path: Option<EditorCommandMenuPath>,
    menu_projection: EditorCommandMenuProjection,
    action: EditorCommandAction,
    default_chord: Option<EditorKeyChord>,
    #[serde(default)]
    when: WhenClause,
    keywords: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    payload_schema_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    headless_commandlet_route: Option<EditorOperationPath>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    headless_commandlet_name: Option<String>,
    #[serde(default = "default_callable_from_remote")]
    callable_from_remote: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    asset_write_target: Option<AssetWriteTargetDescriptor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    execution_contract: Option<EditorCommandExecutionContract>,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_capabilities"
    )]
    required_capabilities: Vec<String>,
}

impl EditorCommandDescriptor {
    pub fn new(
        id: EditorOperationPath,
        category: EditorCommandCategory,
        action: EditorCommandAction,
    ) -> Self {
        let presentation = EditorCommandPresentation::builtin(&id);
        Self {
            id,
            presentation,
            category,
            menu_path: None,
            menu_projection: EditorCommandMenuProjection::CommandRegistry,
            action,
            default_chord: None,
            when: WhenClause::Always,
            keywords: Vec::new(),
            payload_schema_id: None,
            headless_commandlet_route: None,
            headless_commandlet_name: None,
            callable_from_remote: true,
            asset_write_target: None,
            execution_contract: None,
            required_capabilities: Vec::new(),
        }
    }

    pub fn localized(
        id: EditorOperationPath,
        presentation: EditorCommandPresentation,
        category: EditorCommandCategory,
        action: EditorCommandAction,
    ) -> Self {
        let mut descriptor = Self::new(id, category, action);
        descriptor.presentation = presentation;
        descriptor
    }

    pub fn operation(id: EditorOperationPath) -> Self {
        Self::new(
            id,
            EditorCommandCategory::Command,
            EditorCommandAction::Operation,
        )
    }

    pub fn native(id: EditorOperationPath) -> Self {
        Self::new(
            id,
            EditorCommandCategory::Command,
            EditorCommandAction::NativeEndpoint,
        )
    }

    pub fn localized_operation(
        id: EditorOperationPath,
        presentation: EditorCommandPresentation,
    ) -> Self {
        Self::localized(
            id,
            presentation,
            EditorCommandCategory::Command,
            EditorCommandAction::Operation,
        )
    }

    pub fn with_category(mut self, category: EditorCommandCategory) -> Self {
        self.category = category;
        self
    }

    pub fn with_menu_path(mut self, menu_path: EditorCommandMenuPath) -> Self {
        self.menu_path = Some(menu_path);
        self
    }

    pub fn with_menu_projection(mut self, projection: EditorCommandMenuProjection) -> Self {
        self.menu_projection = projection;
        self
    }

    pub fn with_default_chord(mut self, chord: EditorKeyChord) -> Self {
        self.default_chord = Some(chord);
        self
    }

    pub fn with_when(mut self, when: WhenClause) -> Self {
        self.when = when;
        self
    }

    pub fn with_keywords<I, S>(mut self, keywords: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let keywords = keywords.into_iter();
        let (lower_bound, _) = keywords.size_hint();
        self.keywords = Vec::with_capacity(lower_bound);
        self.keywords.extend(keywords.map(Into::into));
        self.keywords.sort_unstable();
        self.keywords.dedup();
        self
    }

    pub fn with_payload_schema_id(mut self, schema_id: impl Into<String>) -> Self {
        self.payload_schema_id = Some(schema_id.into());
        self
    }

    /// Identifies the canonical headless commandlet route for this command.
    pub fn with_headless_commandlet_route(mut self, route: EditorOperationPath) -> Self {
        self.headless_commandlet_route = Some(route);
        self
    }

    /// Identifies the exact CLI name resolved through the canonical command registry.
    pub fn with_headless_commandlet_name(mut self, name: impl Into<String>) -> Self {
        self.headless_commandlet_name = Some(name.into());
        self
    }

    pub fn with_callable_from_remote(mut self, callable_from_remote: bool) -> Self {
        self.callable_from_remote = callable_from_remote;
        self
    }

    pub fn with_asset_write_target_arguments(
        mut self,
        asset_type_argument: impl Into<String>,
        locator_argument: impl Into<String>,
    ) -> Self {
        self.asset_write_target = Some(AssetWriteTargetDescriptor::new(
            asset_type_argument,
            locator_argument,
        ));
        self
    }

    pub fn with_event(mut self, event: EditorEvent) -> Self {
        self.action = EditorCommandAction::Emit(event);
        self
    }

    pub fn with_required_capabilities<I, S>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.required_capabilities
            .extend(capabilities.into_iter().map(Into::into));
        self.required_capabilities.sort();
        self.required_capabilities.dedup();
        self
    }

    pub fn with_execution_contract(mut self, contract: EditorCommandExecutionContract) -> Self {
        self.execution_contract = Some(contract);
        self
    }

    pub fn id(&self) -> &EditorOperationPath {
        &self.id
    }

    pub fn presentation(&self) -> &EditorCommandPresentation {
        &self.presentation
    }

    pub fn localized_label(&self, i18n: &EditorI18nService, locale: &EditorLocale) -> Arc<str> {
        self.presentation.resolve_label(i18n, locale)
    }

    pub fn localized_description(
        &self,
        i18n: &EditorI18nService,
        locale: &EditorLocale,
    ) -> Arc<str> {
        self.presentation.resolve_description(i18n, locale)
    }

    pub fn category(&self) -> EditorCommandCategory {
        self.category
    }

    pub fn menu_path(&self) -> Option<&EditorCommandMenuPath> {
        self.menu_path.as_ref()
    }

    pub fn menu_projection(&self) -> EditorCommandMenuProjection {
        self.menu_projection
    }

    pub fn action(&self) -> &EditorCommandAction {
        &self.action
    }

    pub fn event(&self) -> Option<&EditorEvent> {
        match &self.action {
            EditorCommandAction::Emit(event) => Some(event),
            EditorCommandAction::Operation
            | EditorCommandAction::NativeEndpoint
            | EditorCommandAction::HeadlessAssetMigration
            | EditorCommandAction::HeadlessPluginList
            | EditorCommandAction::HeadlessAuthoringAutomation => None,
        }
    }

    pub fn default_chord(&self) -> Option<&EditorKeyChord> {
        self.default_chord.as_ref()
    }

    pub fn when(&self) -> &WhenClause {
        &self.when
    }

    pub fn effective_when(&self) -> WhenClause {
        WhenClause::all(
            std::iter::once(self.when.clone())
                .chain(
                    self.required_capabilities
                        .iter()
                        .cloned()
                        .map(WhenClause::Capability),
                )
                .chain(
                    self.asset_write_target
                        .as_ref()
                        .map(|_| WhenClause::AssetWritable),
                ),
        )
    }

    pub fn is_enabled(&self, context: &CommandEvalCtx) -> bool {
        self.when.eval(context)
            && self
                .required_capabilities
                .iter()
                .all(|capability| context.has_capability(capability))
            && (self.asset_write_target.is_none() || WhenClause::AssetWritable.eval(context))
    }

    pub(crate) fn missing_required_capabilities(&self, context: &CommandEvalCtx) -> Vec<String> {
        let mut missing = Vec::with_capacity(self.required_capabilities.len());
        for capability in &self.required_capabilities {
            if !context.has_capability(capability) {
                missing.push(capability.clone());
            }
        }
        missing
    }

    pub fn keywords(&self) -> &[String] {
        &self.keywords
    }

    pub fn payload_schema_id(&self) -> Option<&str> {
        self.payload_schema_id.as_deref()
    }

    pub fn headless_commandlet_route(&self) -> Option<&EditorOperationPath> {
        self.headless_commandlet_route.as_ref()
    }

    pub fn headless_commandlet_name(&self) -> Option<&str> {
        self.headless_commandlet_name.as_deref()
    }

    pub fn callable_from_remote(&self) -> bool {
        self.callable_from_remote
    }

    pub fn asset_write_target(&self) -> Option<&AssetWriteTargetDescriptor> {
        self.asset_write_target.as_ref()
    }

    pub(super) fn set_asset_write_target(&mut self, target: AssetWriteTargetDescriptor) {
        self.asset_write_target = Some(target);
    }

    pub fn required_capabilities(&self) -> &[String] {
        &self.required_capabilities
    }

    pub fn execution_contract(&self) -> Option<&EditorCommandExecutionContract> {
        self.execution_contract.as_ref()
    }

    pub(crate) fn bind_localization_bundle(
        &mut self,
        bundle: &EditorLocalizationBundle,
    ) -> Result<(), String> {
        self.presentation.bind_bundle(bundle)
    }
}

/// Selects the single owner that materializes a command's menu metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorCommandMenuProjection {
    CommandRegistry,
    ExtensionRegistry,
}

fn default_callable_from_remote() -> bool {
    true
}

fn deserialize_capabilities<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut capabilities = Vec::<String>::deserialize(deserializer)?;
    capabilities.sort();
    capabilities.dedup();
    Ok(capabilities)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EditorCommandCategory {
    File,
    Edit,
    Selection,
    Runtime,
    View,
    Window,
    Help,
    Command,
}

impl EditorCommandCategory {
    pub fn localization_key(self) -> &'static str {
        match self {
            Self::File => "command.category.file",
            Self::Edit => "command.category.edit",
            Self::Selection => "command.category.selection",
            Self::Runtime => "command.category.runtime",
            Self::View => "command.category.view",
            Self::Window => "command.category.window",
            Self::Help => "command.category.help",
            Self::Command => "command.category.command",
        }
    }

    pub fn source_tag(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Edit => "edit",
            Self::Selection => "selection",
            Self::Runtime => "runtime",
            Self::View => "view",
            Self::Window => "window",
            Self::Help => "help",
            Self::Command => "command",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorCommandAction {
    Emit(EditorEvent),
    Operation,
    NativeEndpoint,
    HeadlessAssetMigration,
    HeadlessPluginList,
    HeadlessAuthoringAutomation,
}

#[cfg(test)]
mod tests {
    use crate::core::editor_operation::EditorOperationPath;

    use crate::core::commands::EditorCommandRegistry;

    use super::EditorCommandDescriptor;

    #[test]
    fn headless_commandlet_route_is_canonical_descriptor_metadata() {
        let registry = EditorCommandRegistry::default_workbench();
        let descriptor = registry
            .command("asset.migration.migrate_assets")
            .expect("the built-in migration command is registered");

        assert_eq!(
            descriptor
                .headless_commandlet_route()
                .map(crate::core::editor_operation::EditorOperationPath::as_str),
            Some("commandlet.route.migrate_assets")
        );
    }

    #[test]
    fn command_enablement_does_not_materialize_an_effective_when_clause() {
        let source = include_str!("descriptor.rs");
        let allocating_eval = ["self", ".effective_when().eval(context)"].concat();
        assert!(!source.contains(&allocating_eval));
    }

    #[test]
    fn command_presentation_is_derived_from_stable_localization_keys() {
        let descriptor = EditorCommandDescriptor::operation(
            EditorOperationPath::parse("weather.cloud_layer.refresh").unwrap(),
        );

        assert_eq!(
            descriptor.presentation().label_key(),
            "command.weather.cloud_layer.refresh.label"
        );
        assert_eq!(
            descriptor.presentation().description_key(),
            "command.weather.cloud_layer.refresh.description"
        );
    }

    #[test]
    fn descriptor_production_shape_has_no_literal_presentation_fields() {
        let production = include_str!("descriptor.rs")
            .split_once("#[cfg(test)]")
            .expect("descriptor tests should remain below production code")
            .0;

        assert!(!production.contains("display_name:"));
        assert!(!production.contains("description: String"));
        assert!(!production.contains("menu_path: Option<String>"));
    }
}

#[cfg(test)]
#[path = "descriptor/optimization_tests.rs"]
mod optimization_tests;
