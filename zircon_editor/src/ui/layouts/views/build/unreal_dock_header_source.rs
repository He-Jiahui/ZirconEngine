use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use zircon_runtime::ui::v2::{UiV2DocumentCompiler, UiV2PrototypeStoreLoadOutcome};
use zircon_runtime_interface::ui::v2::{UiV2AssetDocument, UiV2AssetError, UiV2CompiledDocument};

use super::ViewTemplateNodePatch;

const ASSET_PATH: &str = "/assets/ui/editor/unreal/components/dock_header.zui";
const ASSET_ID: &str = "res://ui/editor/unreal/components/dock_header.zui";
const SLOT_COUNT: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct HeaderRequest {
    used_count: usize,
    selected: [bool; SLOT_COUNT],
}

struct HeaderVariant {
    document: Arc<UiV2AssetDocument>,
    compiled: Arc<UiV2CompiledDocument>,
}

struct HeaderSourceCache {
    original_compiled: Arc<UiV2CompiledDocument>,
    variants: BTreeMap<HeaderRequest, HeaderVariant>,
}

thread_local! {
    static CACHE: RefCell<Option<HeaderSourceCache>> = const { RefCell::new(None) };
}

// This is one view's instance input. Keep the file-cache receipt and authored
// provenance; title changes remain ordinary retained property mutations.
pub(super) fn prepare(
    mut outcome: UiV2PrototypeStoreLoadOutcome,
    asset_path: &str,
    text_overrides: &BTreeMap<String, String>,
    node_patches: &BTreeMap<String, ViewTemplateNodePatch>,
) -> Result<UiV2PrototypeStoreLoadOutcome, UiV2AssetError> {
    if asset_path != ASSET_PATH || outcome.root_document.asset.id != ASSET_ID {
        return Ok(outcome);
    }
    let used_rows = text_overrides
        .keys()
        .filter_map(|control| control.strip_prefix("DockTab"))
        .filter_map(|suffix| suffix.parse::<usize>().ok())
        .collect::<BTreeSet<_>>();
    let used_count = used_rows.len();
    // More slots and non-contiguous inputs retain the existing fallback decision.
    if used_count > SLOT_COUNT || !used_rows.iter().copied().eq(0..used_count) {
        return Ok(outcome);
    }
    let request = HeaderRequest {
        used_count,
        selected: std::array::from_fn(|row| {
            row < used_count
                && node_patches
                    .get(&format!("DockTab{row}"))
                    .and_then(|patch| patch.selected)
                    .unwrap_or(false)
        }),
    };
    CACHE.with(|cell| {
        let mut cache = cell.borrow_mut();
        if cache
            .as_ref()
            .is_none_or(|entry| !Arc::ptr_eq(&entry.original_compiled, &outcome.compiled))
        {
            *cache = Some(HeaderSourceCache {
                original_compiled: Arc::clone(&outcome.compiled),
                variants: BTreeMap::new(),
            });
        }
        let entry = cache.as_mut().expect("header source cache was installed");
        if !entry.variants.contains_key(&request) {
            // Always start from the actual full asset, including when slots return.
            let mut document = outcome.root_document.as_ref().clone();
            let strip = document
                .nodes
                .get("dock_tab_strip")
                .ok_or_else(|| invalid("missing authored dock_tab_strip"))?;
            let expected = (0..SLOT_COUNT)
                .map(|row| format!("dock_tab_group_{row}"))
                .collect::<Vec<_>>();
            if strip
                .children
                .iter()
                .map(|child| &child.node)
                .ne(expected.iter())
            {
                return Err(invalid(
                    "authored dock tab groups do not match the three-slot contract",
                ));
            }
            let mut removed = BTreeSet::new();
            for row in request.used_count..SLOT_COUNT {
                collect_subtree(&document, &format!("dock_tab_group_{row}"), &mut removed)?;
            }
            document
                .nodes
                .get_mut("dock_tab_strip")
                .unwrap()
                .children
                .retain(|child| !removed.contains(&child.node));
            document.nodes.retain(|key, _| !removed.contains(key));
            for row in 0..request.used_count {
                for key in [format!("dock_tab_group_{row}"), format!("dock_tab_{row}")] {
                    let node = document
                        .nodes
                        .get_mut(&key)
                        .ok_or_else(|| invalid("missing authored live dock tab node"))?;
                    // Initial materialization must include the actual selected
                    // stylesheet, rather than publishing normal colors then
                    // changing only the projection's selected flag.
                    node.props.insert(
                        "selected".into(),
                        toml::Value::Boolean(request.selected[row]),
                    );
                }
            }
            let compiled = UiV2DocumentCompiler::compile_with_prototype_store(
                &document,
                outcome.store.as_ref(),
            )?;
            entry.variants.insert(
                request,
                HeaderVariant {
                    document: Arc::new(document),
                    compiled: Arc::new(compiled),
                },
            );
        }
        let variant = entry.variants.get(&request).unwrap();
        outcome.root_document = Arc::clone(&variant.document);
        outcome.compiled = Arc::clone(&variant.compiled);
        Ok(outcome)
    })
}

fn collect_subtree(
    document: &UiV2AssetDocument,
    key: &str,
    removed: &mut BTreeSet<String>,
) -> Result<(), UiV2AssetError> {
    if !removed.insert(key.to_string()) {
        return Err(invalid("authored dock slot repeats a node"));
    }
    let node = document
        .nodes
        .get(key)
        .ok_or_else(|| invalid("missing authored dock slot node"))?;
    for child in &node.children {
        collect_subtree(document, &child.node, removed)?;
    }
    Ok(())
}

fn invalid(detail: &str) -> UiV2AssetError {
    UiV2AssetError::InvalidDocument {
        asset_id: ASSET_ID.into(),
        detail: detail.into(),
    }
}

/// One Header instance's immutable source input and publication. The owning
/// projection/composition ModelRc carries it; no cross-frame registry is used.
pub(crate) struct UnrealDockHeaderSource {
    pub(crate) document: Arc<UiV2AssetDocument>,
    pub(crate) frame: Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame>,
    pub(crate) viewport: zircon_runtime_interface::ui::layout::UiSize,
    pub(crate) source_receipts: Vec<zircon_runtime::ui::v2::UiV2SourceFileReceipt>,
    pub(crate) unresolved_imports: Vec<zircon_runtime::ui::v2::UiV2UnresolvedSourceImport>,
    pub(crate) identities: BTreeMap<String, UnrealDockHeaderNodeIdentity>,
}

pub(crate) struct UnrealDockHeaderNodeIdentity {
    pub(crate) source_path: String,
    pub(crate) source_node_id: String,
    pub(crate) instance_path: String,
}

pub(super) fn capture(
    outcome: &UiV2PrototypeStoreLoadOutcome,
    asset_path: &str,
    viewport: zircon_runtime_interface::ui::layout::UiSize,
    frame: &Arc<zircon_runtime_interface::ui::surface::UiSurfaceFrame>,
    text_overrides: &BTreeMap<String, String>,
    node_patches: &BTreeMap<String, ViewTemplateNodePatch>,
) -> Option<Arc<UnrealDockHeaderSource>> {
    if asset_path != ASSET_PATH || outcome.root_document.asset.id != ASSET_ID {
        return None;
    }
    let mut document = outcome.root_document.as_ref().clone();
    for node in document.nodes.values_mut() {
        let Some(control) = node.control_id.as_ref() else {
            continue;
        };
        if let Some(text) = text_overrides.get(control) {
            node.props
                .insert("text".into(), toml::Value::String(text.clone()));
        }
        if let Some(patch) = node_patches.get(control) {
            if let Some(selected) = patch.selected {
                node.props
                    .insert("selected".into(), toml::Value::Boolean(selected));
            }
            if let Some(variant) = patch.surface_variant.as_ref() {
                node.props.insert(
                    "surface_variant".into(),
                    toml::Value::String(variant.clone()),
                );
            }
            if let Some(tone) = patch.text_tone.as_ref() {
                node.props
                    .insert("text_tone".into(), toml::Value::String(tone.clone()));
            }
        }
    }
    // Read the compiled provenance which SurfaceBuilder copied into the actual
    // UiTree. A control id is only the lookup key, never an invented source id.
    let identities = outcome
        .compiled
        .arena
        .nodes
        .iter()
        .filter_map(|node| {
            let control = node.control_id.as_ref()?;
            let source_path = node.source_path.as_ref()?;
            let source_node_id = node.source_node_id.as_ref()?;
            let instance_path = node.instance_path.as_ref()?;
            let source_path = if source_path == ASSET_ID {
                "zircon_editor/assets/ui/editor/unreal/components/dock_header.zui".to_owned()
            } else {
                source_path.clone()
            };
            Some((
                control.clone(),
                UnrealDockHeaderNodeIdentity {
                    source_path,
                    source_node_id: source_node_id.clone(),
                    instance_path: serde_json::to_string(instance_path).ok()?,
                },
            ))
        })
        .collect();
    Some(Arc::new(UnrealDockHeaderSource {
        document: Arc::new(document),
        frame: Arc::clone(frame),
        viewport,
        source_receipts: outcome.source_receipts.clone(),
        unresolved_imports: outcome.unresolved_imports.clone(),
        identities,
    }))
}
