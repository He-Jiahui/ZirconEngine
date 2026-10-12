use std::path::PathBuf;

use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::v2::{UiV2DocumentCompiler, UiV2SourceFileReceipt, UiV2SurfaceBuilder};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    event_ui::UiTreeId,
    template::UiBindingRef,
    v2::{UiV2AssetError, UiV2Root},
};

use crate::ui::binding::EditorUiBinding;
use crate::ui::template_runtime::RetainedUiProjection;
use crate::ui::v2_design_tokens::prepare_editor_v2_document;

use super::{v2_template_file_cache, EditorUiHostRuntime, EditorUiHostRuntimeError};

impl EditorUiHostRuntime {
    /// Loads only the selected canonical workspace and its imported component/theme closure.
    pub(crate) fn project_workbench_module_workspace(
        &self,
        document_id: &str,
        component_id: &str,
    ) -> Result<
        (RetainedUiProjection, UiSurface, Vec<UiV2SourceFileReceipt>),
        EditorUiHostRuntimeError,
    > {
        let invalid = |detail: String| UiV2AssetError::InvalidDocument {
            asset_id: document_id.to_string(),
            detail,
        };
        let relative = document_id
            .strip_prefix("res://ui/editor/components/workbench/modules/")
            .ok_or_else(|| invalid("workspace URI is outside the builtin module owner".into()))?;
        if !(relative.starts_with("core/") || relative.starts_with("extensions/")) {
            return Err(invalid(
                "workspace URI is outside the selected core/extension owner".into(),
            )
            .into());
        }
        if relative
            .split('/')
            .any(|part| part == ".." || part.is_empty())
        {
            return Err(invalid("workspace URI contains an invalid path segment".into()).into());
        }
        let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        let module_path = runtime_asset_path_with_dev_asset_root(
            &format!("/assets/ui/editor/components/workbench/modules/{relative}"),
            &asset_root,
        );
        let theme_path = runtime_asset_path_with_dev_asset_root(
            "/assets/ui/theme/editor_workbench_strict.zui",
            &asset_root,
        );
        let outcome = v2_template_file_cache()
            .lock()
            .expect("v2 template file cache mutex should not be poisoned")
            .load_store([module_path, theme_path])?;
        if !outcome.unresolved_imports.is_empty() || outcome.root_asset_id != document_id {
            return Err(invalid(
                "workspace source closure is unresolved or has a different identity".into(),
            )
            .into());
        }
        let mut document = outcome.root_document.as_ref().clone();
        let root = document
            .components
            .get(component_id)
            .ok_or_else(|| invalid(format!("workspace component {component_id} is missing")))?
            .root
            .clone();
        document.root = Some(UiV2Root { node: root });
        let compiled =
            UiV2DocumentCompiler::compile_with_prototype_store(&document, outcome.store.as_ref())?;
        let projection = super::super::projection::project_v2_document(
            document_id,
            &compiled,
            &self.template_adapter,
        )?;
        let prepared = prepare_editor_v2_document(&document);
        let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document_with_theme(
            UiTreeId::new(format!("template.v2.{document_id}")),
            &prepared,
            &compiled,
            &self.active_theme,
        )?;
        Ok((projection, surface, outcome.source_receipts))
    }

    /// The existing Window popup dispatches the same typed module selection bindings.
    pub(crate) fn workbench_module_navigation_binding(
        &self,
        binding_id: &str,
    ) -> Result<EditorUiBinding, EditorUiHostRuntimeError> {
        self.template_adapter
            .resolve_binding(&UiBindingRef {
                id: binding_id.to_string(),
                event: UiEventKind::Click,
                mode: Default::default(),
                component_event: None,
                route: None,
                action: None,
                targets: Vec::new(),
            })
            .map_err(Into::into)
    }
}
