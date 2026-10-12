use crate::asset::MaterialAsset;
use crate::core::framework::render::{
    RenderMaterialDiagnosticSource, RenderMaterialValidationError,
};
use crate::core::resource::ResourceId;
use crate::core::CoreError;

use super::super::ProjectAssetManager;

const MAX_MATERIAL_PARENT_DEPTH: usize = 4;

impl ProjectAssetManager {
    /// 渲染准备入口：展平同 shader 的父材质覆盖；无效父链保留可渲染的子材质并返回诊断。
    pub(crate) fn load_effective_material_asset(
        &self,
        root_id: ResourceId,
    ) -> Result<(MaterialAsset, Vec<RenderMaterialValidationError>), CoreError> {
        let root = self.load_material_asset(root_id)?;
        Ok(self.resolve_effective_material_asset(root_id, root))
    }

    pub(crate) fn resolve_effective_material_asset(
        &self,
        root_id: ResourceId,
        root: MaterialAsset,
    ) -> (MaterialAsset, Vec<RenderMaterialValidationError>) {
        let root_shader = root.shader.clone();
        let mut diagnostics = Vec::new();
        let mut lineage = Vec::with_capacity(MAX_MATERIAL_PARENT_DEPTH + 1);
        lineage.push((root_id, root));

        loop {
            let Some(parent_reference) = lineage
                .last()
                .and_then(|(_, material)| material.parent.clone())
            else {
                break;
            };
            if lineage.len() > MAX_MATERIAL_PARENT_DEPTH {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent chain exceeds depth limit {MAX_MATERIAL_PARENT_DEPTH}"
                )));
                break;
            }
            let Some(parent_id) = self.resolve_asset_id(&parent_reference.locator) else {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent `{}` is not registered",
                    parent_reference.locator
                )));
                break;
            };
            if lineage.iter().any(|(id, _)| *id == parent_id) {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent chain contains cycle at {parent_id}"
                )));
                break;
            }
            let Ok(parent) = self.load_material_asset(parent_id) else {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent `{}` failed to load",
                    parent_reference.locator
                )));
                break;
            };
            if parent.shader != root_shader {
                diagnostics.push(invalid_parent_diagnostic(format!(
                    "material parent `{}` uses shader `{}` but child uses `{}`",
                    parent_reference.locator, parent.shader.locator, root_shader.locator
                )));
                break;
            }
            lineage.push((parent_id, parent));
        }

        let mut effective = lineage
            .pop()
            .map(|(_, material)| material)
            .expect("material lineage contains root");
        while let Some((_, mut child)) = lineage.pop() {
            child.inherit_parent_values_from(&effective);
            effective = child;
        }
        effective.parent = None;
        (effective, diagnostics)
    }
}

fn invalid_parent_diagnostic(diagnostic: String) -> RenderMaterialValidationError {
    RenderMaterialValidationError::InvalidMaterialParent {
        source: RenderMaterialDiagnosticSource::MaterialOverride,
        path: "parent".to_string(),
        diagnostic,
    }
}

#[cfg(test)]
#[path = "tests/load_effective_material.rs"]
mod tests;
