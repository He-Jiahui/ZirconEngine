use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::v2::{
    UiV2DocumentCompiler, UiV2PrototypeStoreFileCache, UiV2SurfaceBuilder,
};
use zircon_runtime_interface::ui::v2::{
    UiV2AssetDocument, UiV2AssetKind, UiV2NodeDefinition, UiV2Root,
};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    surface::UiRenderExtract,
    window::UiWindowMetrics,
};

pub(super) struct NativePreview {
    pub surface: UiSurface,
    pub extract: UiRenderExtract,
    pub source_ids: BTreeMap<UiNodeId, String>,
}

pub(super) fn mount_component(document: &UiV2AssetDocument) -> Result<UiV2AssetDocument, String> {
    mount_component_with_data(document, &Value::Object(Default::default()))
}

pub(super) fn mount_component_with_data(
    document: &UiV2AssetDocument,
    data: &Value,
) -> Result<UiV2AssetDocument, String> {
    if document.root_node_id().is_some() {
        super::data::validate(data, "fixture")?;
        return Ok(document.clone());
    }
    if document.asset.kind != UiV2AssetKind::Component {
        return Err("style/theme asset needs a source-owned consumer specimen".into());
    }
    let component = document
        .components
        .keys()
        .next()
        .ok_or("component asset has no component declaration")?;
    if document.components.len() != 1 {
        return Err("component preview needs exactly one exported component".into());
    }
    // Mount the declared component, not its root node: the actual instancer applies
    // parameter defaults, classes, optional slot content and required-slot checks.
    let mut preview = document.clone();
    preview.asset.kind = UiV2AssetKind::View;
    preview.asset.id = format!("{}#native-preview", document.asset.id);
    preview.imports.widgets.push(document.asset.id.clone());
    preview.nodes.clear();
    preview.components.clear();
    preview.root = Some(UiV2Root {
        node: "native_preview_mount".into(),
    });
    preview.nodes.insert(
        "native_preview_mount".into(),
        UiV2NodeDefinition {
            component: format!("{}#{component}", document.asset.id),
            props: super::data::component_mount_props(document, data)?,
            ..Default::default()
        },
    );
    Ok(preview)
}

pub(super) fn build(
    path: &Path,
    dependencies: &[PathBuf],
    id: &str,
    metrics: UiWindowMetrics,
    state: &str,
    theme_source: Option<&Path>,
) -> Result<NativePreview, String> {
    build_with_data(
        path,
        dependencies,
        id,
        metrics,
        state,
        theme_source,
        &Value::Object(Default::default()),
    )
}

pub(super) fn build_with_data(
    path: &Path,
    dependencies: &[PathBuf],
    id: &str,
    metrics: UiWindowMetrics,
    state: &str,
    theme_source: Option<&Path>,
    data: &Value,
) -> Result<NativePreview, String> {
    if let Some(theme) = theme_source {
        let document = zircon_runtime::ui::v2::UiZuiAssetLoader::load_zui_file(theme)
            .map_err(|error| error.to_string())?;
        if !matches!(
            document.asset.kind,
            UiV2AssetKind::Style | UiV2AssetKind::ThemeTokens
        ) {
            return Err("review host theme must be a style or theme_tokens asset".into());
        }
    }
    let mut cache = UiV2PrototypeStoreFileCache::new();
    let loaded = cache
        .load_store(
            std::iter::once(path)
                .chain(
                    dependencies
                        .iter()
                        .map(PathBuf::as_path)
                        .filter(|dependency| {
                            dependency
                                .extension()
                                .is_some_and(|extension| extension == "zui")
                        }),
                )
                .chain(theme_source),
        )
        .map_err(|e| e.to_string())?;
    let mut document = mount_component_with_data(loaded.root_document.as_ref(), data)?;
    super::data::apply_collection_case(&mut document, data)?;
    let compiled =
        UiV2DocumentCompiler::compile_with_prototype_store(&document, loaded.store.as_ref())
            .map_err(|e| e.to_string())?;
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new(id),
        &document,
        &compiled,
    )
    .map_err(|e| e.to_string())?;
    surface.window_state.metrics = Some(metrics);
    super::state::apply(&mut surface, state)?;
    surface
        .compute_layout(metrics.logical_size)
        .map_err(|e| e.to_string())?;
    if super::state::apply_scroll_position(&mut surface, state)? {
        surface
            .compute_layout(metrics.logical_size)
            .map_err(|e| e.to_string())?;
    }
    let extract = super::dpi::physical_extract(&surface.render_extract, metrics)?;
    if extract.list.commands.is_empty() {
        return Err("arranged surface emitted no native render commands".into());
    }
    let mut source_ids = BTreeMap::new();
    let mut pending = vec![(
        compiled.arena.root.ok_or("compiled preview has no root")?,
        *surface
            .tree
            .roots
            .first()
            .ok_or("native preview has no tree root")?,
    )];
    while let Some((handle, node_id)) = pending.pop() {
        let source_node = compiled
            .arena
            .node(handle)
            .ok_or("compiled node is missing")?;
        let runtime_node = surface
            .tree
            .nodes
            .get(&node_id)
            .ok_or("runtime node is missing")?;
        if source_node.children.len() != runtime_node.children.len() {
            return Err(
                "native tree expansion requires an explicit semantic identity adapter".into(),
            );
        }
        source_ids.insert(node_id, source_node.source_id.clone());
        pending.extend(
            source_node
                .children
                .iter()
                .zip(&runtime_node.children)
                .map(|(child, node_id)| (child.child, *node_id)),
        );
    }
    Ok(NativePreview {
        surface,
        extract,
        source_ids,
    })
}
