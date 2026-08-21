use zircon_runtime::ui::template::{
    collect_document_localization_report, validate_localization_report_against_catalog,
    validate_resource_dependency_files, UiResourcePathResolver,
};
use zircon_runtime_interface::ui::template::{UiLocalizationDiagnostic, UiResourceDiagnostic};

use super::{
    diagnostics::map_localization_diagnostic,
    lifecycle::{compiled_resource_report, structured_compile_diagnostics},
    runtime_report_state::DEFAULT_LOCALE_PREVIEW,
    ui_asset_editor_session::UiAssetEditorSession,
};

impl UiAssetEditorSession {
    pub fn register_locale_table_keys<I, S>(
        &mut self,
        locale: impl Into<String>,
        table: impl Into<String>,
        source_uri: Option<String>,
        keys: I,
    ) -> bool
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let previous = self.localization_catalog.clone();
        self.localization_catalog
            .register_table_keys(locale, table, source_uri, keys);
        let changed = previous != self.localization_catalog;
        self.refresh_structured_diagnostics_for_current_document();
        changed
    }

    pub fn set_resource_path_resolver(&mut self, resolver: UiResourcePathResolver) -> bool {
        if self.resource_resolver == resolver {
            return false;
        }
        self.resource_resolver = resolver;
        self.refresh_resource_path_resolver();
        true
    }

    pub fn refresh_resource_path_resolver(&mut self) {
        let (dependencies, compiler_diagnostics) =
            compiled_resource_report(self.last_valid_compiled.as_ref());
        self.resource_dependencies = dependencies;
        self.resource_diagnostics = resource_resolver_diagnostics(
            &self.resource_dependencies,
            &compiler_diagnostics,
            &self.resource_resolver,
        );
    }

    pub(super) fn localization_resolver_diagnostics(&self) -> Vec<UiLocalizationDiagnostic> {
        if self.selected_locale_preview == DEFAULT_LOCALE_PREVIEW {
            return Vec::new();
        }
        let report = collect_document_localization_report(&self.last_valid_document);
        validate_localization_report_against_catalog(
            &report,
            &self.selected_locale_preview,
            &self.localization_catalog,
        )
    }

    pub(super) fn refresh_structured_diagnostics_for_current_document(&mut self) {
        let mut diagnostics = if self.diagnostics.is_empty() {
            Vec::new()
        } else {
            structured_compile_diagnostics(&self.last_valid_document, &self.compiler_imports)
        };
        diagnostics.extend(
            self.localization_resolver_diagnostics()
                .into_iter()
                .map(map_localization_diagnostic),
        );
        self.structured_diagnostics = diagnostics;
    }
}

fn resource_resolver_diagnostics(
    dependencies: &[zircon_runtime_interface::ui::template::UiResourceDependency],
    compiler_diagnostics: &[UiResourceDiagnostic],
    resolver: &UiResourcePathResolver,
) -> Vec<UiResourceDiagnostic> {
    let mut diagnostics = compiler_diagnostics.to_vec();
    diagnostics.extend(validate_resource_dependency_files(dependencies, resolver));
    diagnostics.sort_by(|left, right| {
        (&left.path, &left.code, &left.message).cmp(&(&right.path, &right.code, &right.message))
    });
    diagnostics.dedup_by(|left, right| {
        left.path == right.path && left.code == right.code && left.message == right.message
    });
    diagnostics
}
