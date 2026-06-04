use super::super::types::{PendingOptionalFeatureManifest, StaticOptionalFeatureManifest};
use super::line::{
    parse_optional_feature_dependency_line, parse_optional_feature_line,
    parse_optional_feature_module_line,
};
use super::pending::{
    push_optional_feature, push_optional_feature_dependency, push_optional_feature_module,
};
use super::section::OptionalFeatureSection;

// Keeps the hand-rolled static TOML scanner's pending rows coherent across table changes.
#[derive(Default)]
pub(super) struct OptionalFeatureParserState {
    features: Vec<StaticOptionalFeatureManifest>,
    current_feature: Option<PendingOptionalFeatureManifest>,
    current_dependency_plugin_id: Option<String>,
    current_dependency_capability: Option<String>,
    current_dependency_primary: Option<bool>,
    current_module_name: Option<String>,
    current_module_kind: Option<zircon_runtime::plugin::PluginModuleKind>,
    current_module_crate_name: Option<String>,
    current_module_target_modes: Vec<zircon_runtime::RuntimeTargetMode>,
    current_module_capabilities: Vec<String>,
    section: OptionalFeatureSection,
}

impl OptionalFeatureParserState {
    pub(super) fn parse_manifest_line(&mut self, line: &str) {
        if let Some(section) = OptionalFeatureSection::from_table_header(line) {
            self.enter_section(section);
            return;
        }

        self.parse_section_line(line);
    }

    pub(super) fn finish(mut self) -> Vec<StaticOptionalFeatureManifest> {
        self.close_optional_feature_scope();
        self.features
    }

    fn enter_section(&mut self, section: OptionalFeatureSection) {
        match section {
            OptionalFeatureSection::Feature => self.start_optional_feature(),
            OptionalFeatureSection::Dependency => self.start_dependency(),
            OptionalFeatureSection::Module => self.start_module(),
            OptionalFeatureSection::None => self.close_optional_feature_scope(),
        }
    }

    fn start_optional_feature(&mut self) {
        self.close_optional_feature_scope();
        self.current_feature = Some(PendingOptionalFeatureManifest::default());
        self.section = OptionalFeatureSection::Feature;
    }

    fn start_dependency(&mut self) {
        self.flush_pending_dependency();
        self.flush_pending_module();
        self.section = OptionalFeatureSection::Dependency;
    }

    fn start_module(&mut self) {
        self.flush_pending_dependency();
        self.flush_pending_module();
        self.section = OptionalFeatureSection::Module;
    }

    fn close_optional_feature_scope(&mut self) {
        self.flush_pending_dependency();
        self.flush_pending_module();
        self.flush_pending_feature();
        self.section = OptionalFeatureSection::None;
    }

    fn parse_section_line(&mut self, line: &str) {
        match self.section {
            OptionalFeatureSection::Feature => parse_optional_feature_line(
                line,
                self.current_feature
                    .as_mut()
                    .expect("optional feature table should have a current feature"),
            ),
            OptionalFeatureSection::Dependency => parse_optional_feature_dependency_line(
                line,
                &mut self.current_dependency_plugin_id,
                &mut self.current_dependency_capability,
                &mut self.current_dependency_primary,
            ),
            OptionalFeatureSection::Module => parse_optional_feature_module_line(
                line,
                &mut self.current_module_name,
                &mut self.current_module_kind,
                &mut self.current_module_crate_name,
                &mut self.current_module_target_modes,
                &mut self.current_module_capabilities,
            ),
            OptionalFeatureSection::None => {}
        }
    }

    fn flush_pending_dependency(&mut self) {
        push_optional_feature_dependency(
            &mut self.current_feature,
            &mut self.current_dependency_plugin_id,
            &mut self.current_dependency_capability,
            &mut self.current_dependency_primary,
        );
    }

    fn flush_pending_module(&mut self) {
        push_optional_feature_module(
            &mut self.current_feature,
            &mut self.current_module_name,
            &mut self.current_module_kind,
            &mut self.current_module_crate_name,
            &mut self.current_module_target_modes,
            &mut self.current_module_capabilities,
        );
    }

    fn flush_pending_feature(&mut self) {
        push_optional_feature(&mut self.features, &mut self.current_feature);
    }
}
