use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{json, Map, Value};
use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::retained_host::host_contract::paint_template_nodes_for_evidence_with_background;
use crate::ui::retained_host::host_contract::paint_theme::{
    apply_host_appearance_from_tokens, apply_host_paint_scale_factor,
    capture_host_paint_theme_snapshot, enter_host_paint_theme_scope,
};
use crate::ui::retained_host::host_contract::{
    paint_product_presentation_for_evidence, PaintEvidenceScope, TextPaintEvidenceScope,
};
use crate::ui::retained_host::ui::workbench_window_projection::to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale;
use crate::ui::template_runtime::EditorUiHostRuntime;
use crate::ui::v2_design_tokens::active_editor_v2_design_tokens_snapshot;

use super::contract::{canonical_hash, contained, hash_file, write_json, Entry, ReviewCase};
use zircon_runtime::core::CoreHandle;
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

#[cfg(test)]
#[path = "tests/capture.rs"]
mod tests;

fn consumed_tokens_receipt(tokens: &EditorDesignTokens) -> Result<Value, String> {
    let canonical_tokens = tokens
        .cascade_token_values()
        .into_iter()
        .filter(|(name, _)| name.starts_with("editor."))
        .collect::<BTreeMap<_, _>>();
    if canonical_tokens.is_empty() {
        return Err("active editor design-token cascade has no canonical editor.* values".into());
    }
    let canonical_tokens = serde_json::to_value(canonical_tokens).map_err(|error| {
        format!("active editor design-token cascade is not serializable: {error}")
    })?;
    Ok(json!({
        "complete": true,
        "sha256": canonical_hash(&canonical_tokens)?,
        "tokens": canonical_tokens,
    }))
}

pub(super) fn render(
    repo: &Path,
    entry: &Entry,
    case_value: &Value,
    case: &ReviewCase,
    directory: &Path,
) -> Result<Map<String, Value>, String> {
    render_internal(repo, entry, case_value, case, directory, None)
}

pub(super) fn render_with_context(
    repo: &Path,
    entry: &Entry,
    case_value: &Value,
    case: &ReviewCase,
    directory: &Path,
    core: &CoreHandle,
    app_preflighted_build_set: &ZrRuntimeBuildSetId,
) -> Result<Map<String, Value>, String> {
    render_internal(
        repo,
        entry,
        case_value,
        case,
        directory,
        Some((core, app_preflighted_build_set)),
    )
}

fn render_internal(
    repo: &Path,
    entry: &Entry,
    case_value: &Value,
    case: &ReviewCase,
    directory: &Path,
    product_context: Option<(&CoreHandle, &ZrRuntimeBuildSetId)>,
) -> Result<Map<String, Value>, String> {
    let physical_viewport = case.physical_viewport()?;
    let font_assets = crate::ui::retained_host::host_contract::UiHostWindow::load_font_assets()
        .map_err(|error| error.to_string())?;
    let has_product_presentation = case.data.get("workbenchPresentation").is_some();
    let startup_tokens = (!has_product_presentation).then(active_editor_v2_design_tokens_snapshot);
    let identities = super::source_identity::SourceIdentityIndex::load(repo, entry, case)?;
    let product = if has_product_presentation {
        let (core, app_preflighted_build_set) = product_context.ok_or(
            "product workbench capture requires the App-preflighted CoreHandle and runtime BuildSet",
        )?;
        Some(
            super::product_presentation::build_product_workbench_presentation_with_context(
                repo,
                UiSize::new(case.viewport.width as f32, case.viewport.height as f32),
                case.dpi as f32,
                case,
                core,
                app_preflighted_build_set,
            )?,
        )
    } else {
        None
    };
    // Product construction installs the active manager's settings snapshot into the V2 token
    // registry. Capture and paint from that exact snapshot; standalone template captures keep
    // the snapshot they observed before building their isolated host.
    let tokens = startup_tokens.unwrap_or_else(active_editor_v2_design_tokens_snapshot);
    if let Some(theme) = &case.theme_source_path {
        let value: toml::Value = toml::from_str(
            &std::fs::read_to_string(contained(repo, theme)?).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        super::theme::verify(&value, &tokens)?;
    }
    // Match the product host boundary: layout stays logical, while retained nodes and
    // theme metrics cross into physical pixels exactly once before native painting.
    apply_host_appearance_from_tokens(&tokens);
    apply_host_paint_scale_factor(case.dpi as f32);
    let physical_theme = capture_host_paint_theme_snapshot();
    apply_host_paint_scale_factor(1.0);
    let _theme = enter_host_paint_theme_scope(physical_theme);
    let (mut geometry_evidence, semantic, retained_nodes) = if let Some(product) = &product {
        let mut geometry = super::geometry::build_product_presentation(
            &product.host_presentation,
            &identities,
            case_value,
            case,
        )?;
        let actual_locale = product.active_locale();
        let expected_locale = match case.locale.as_str() {
            "en" | "en-US" => "en",
            "zh-CN" => "zh-CN",
            _ => {
                return Err(format!(
                    "unsupported product review locale {:?}",
                    case.locale
                ))
            }
        };
        let locale_complete = actual_locale == expected_locale;
        geometry.value["localeAudit"] = json!({
            "complete": locale_complete,
            "source": "editor-i18n-service",
            "actualLocale": actual_locale,
            "expectedLocale": expected_locale,
        });
        if !locale_complete {
            geometry
                .issues
                .push("actual Editor locale differs from the requested review locale".into());
        }
        let workbench_layout = product.snapshot.workbench_layout()?;
        let host_floating_windows_empty = product
            .host_presentation
            .host_scene_data
            .floating_layer
            .floating_windows
            .row_count()
            == 0
            && product
                .host_presentation
                .native_floating_surface_data
                .floating_windows
                .row_count()
                == 0;
        let overlay_audit_complete =
            workbench_layout.floating_windows.is_empty() && host_floating_windows_empty;
        geometry.value["hostOverlayAudit"] = json!({
            "complete": overlay_audit_complete,
            "windows": [],
        });
        if !overlay_audit_complete {
            geometry
                .issues
                .push("floating host windows need a separate factual host-overlay receipt".into());
        }
        let semantic = json!({
            "workbenchPresentation": serde_json::to_value(&product.snapshot)
                .map_err(|error| error.to_string())?,
            "semanticNodes": geometry.value.pointer("/layout/semanticNodes")
                .cloned().unwrap_or(Value::Null),
        });
        (geometry, semantic, None)
    } else {
        let mut runtime = EditorUiHostRuntime::default();
        // Explicit editor review hosts mount the same built-in Workbench templates
        // as the product host. Register their component descriptors and bindings
        // before compiling the review document so authored events (for example
        // `UiHostWindow/ActivateMainPage`) resolve through the real template runtime.
        runtime
            .load_builtin_host_templates()
            .map_err(|error| error.to_string())?;
        super::source::register(&mut runtime, repo, entry, case, directory)?;
        runtime
            .ensure_builtin_host_bindings()
            .map_err(|error| error.to_string())?;
        let projection = runtime
            .project_document("layout-review")
            .map_err(|error| error.to_string())?;
        let mut surface = runtime
            .build_shared_surface("layout-review")
            .map_err(|error| error.to_string())?;
        super::state::apply_case(&mut surface, case)?;
        surface
            .compute_layout(UiSize::new(
                case.viewport.width as f32,
                case.viewport.height as f32,
            ))
            .map_err(|error| error.to_string())?;
        if super::state::apply_scroll_position_for_case(&mut surface, case)? {
            surface
                .compute_layout(UiSize::new(
                    case.viewport.width as f32,
                    case.viewport.height as f32,
                ))
                .map_err(|error| error.to_string())?;
        }
        let model = runtime
            .build_host_model_with_surface(&projection, &surface)
            .map_err(|error| error.to_string())?;
        let retained_projection = runtime
            .build_retained_host_projection_with_surface(&projection, &surface)
            .map_err(|error| error.to_string())?;
        let geometry = super::geometry::build(&model, &identities, case_value, case)?;
        let semantic = serde_json::to_value(surface.accessibility_snapshot())
            .map_err(|error| error.to_string())?;
        let nodes = to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
            Some(&retained_projection),
            None,
            None,
            case.dpi as f32,
        );
        if nodes.row_count() == 0 {
            return Err("retained host produced no native controls".into());
        }
        (geometry, semantic, Some(nodes))
    };
    geometry_evidence.value["sourceIdentityProvenance"] = identities.provenance();
    geometry_evidence.value["sourceIdentityProvenance"]["entrySourceFingerprint"] =
        json!([entry.source_path.clone(), entry.source_sha256.clone()]);
    geometry_evidence.value["dependencyFingerprints"] =
        json!(entry.dependency_fingerprints.clone());
    if let Some(product) = &product {
        let (runtime_source_audit, runtime_source_issues) = identities
            .runtime_loaded_sources_audit(
                &product.source_document_ids,
                &product.source_receipts,
                &product.source_unresolved_imports,
            );
        geometry_evidence.value["sourceIdentityProvenance"]["runtimeLoadedSources"] =
            runtime_source_audit;
        geometry_evidence.issues.extend(runtime_source_issues);
    }
    let active_design_tokens = serde_json::to_value(tokens.as_ref())
        .map_err(|error| format!("active editor design tokens are not serializable: {error}"))?;
    geometry_evidence.value["activeDesignTokens"] = json!({
        "complete": true,
        "sha256": canonical_hash(&active_design_tokens)?,
        "tokens": active_design_tokens,
    });
    geometry_evidence.value["consumedTokens"] = consumed_tokens_receipt(tokens.as_ref())?;
    let paint_scope = PaintEvidenceScope::begin(repo, case.dpi as f32)?;
    let text_scope = TextPaintEvidenceScope::begin_for_capture(repo, case.dpi as f32)?;
    let pixels = match (&product, retained_nodes) {
        (Some(product), None) => paint_product_presentation_for_evidence(
            physical_viewport.width,
            physical_viewport.height,
            &product.host_presentation,
        ),
        (None, Some(nodes)) => paint_template_nodes_for_evidence_with_background(
            physical_viewport.width,
            physical_viewport.height,
            tokens.palette.surface[0].to_u8(),
            nodes,
        ),
        _ => return Err("capture renderer selection did not produce one paint source".into()),
    };
    let text_evidence = text_scope.finish()?;
    let paint_evidence = paint_scope.finish()?;
    let screenshot = directory.join(format!("editor-{}.png", case.id));
    image::save_buffer_with_format(
        &screenshot,
        &pixels,
        physical_viewport.width,
        physical_viewport.height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .map_err(|error| error.to_string())?;
    geometry_evidence
        .issues
        .extend(identities.verify_current(repo)?);
    geometry_evidence
        .issues
        .extend(identities.dependency_issues(entry));
    if !geometry_evidence.issues.is_empty() {
        geometry_evidence.value["semanticAudit"]["complete"] = json!(false);
    }
    let relative = format!("{}/{}/evidence", entry.category, entry.name);
    let case_sha256 = canonical_hash(case_value)?;
    let painter_path = directory.join(format!("editor-{}.paint-painter.json", case.id));
    write_json(&painter_path, &paint_evidence)?;
    let painter_text = directory.join(format!("editor-{}.text-painter.json", case.id));
    write_json(&painter_text, &text_evidence)?;
    let native_text_nodes = text_evidence
        .get("nodes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let (runtime_asset_fingerprints, mut fingerprint_issues) =
        super::painter_receipts::merge_runtime_fingerprints(&paint_evidence, &text_evidence);
    let media_readiness = paint_evidence
        .get("assetAudit")
        .cloned()
        .unwrap_or_else(|| json!({"complete": false, "resources": []}));
    let font_audit = text_evidence
        .get("fontAudit")
        .cloned()
        .unwrap_or_else(|| json!({"loaded": false}));
    let mut text = json!({
        "schema": "dev.zircon.zui.native-text-evidence",
        "version": 1,
        "case": case_value,
        "caseSha256": case_sha256,
        "caseId": case.id,
        "coordinateSpace": "logical",
        "nodes": native_text_nodes,
        "fontAudit": font_audit,
        "runtimeAssetFingerprints": runtime_asset_fingerprints,
        "rawPainterPath": format!("{relative}/editor-{}.text-painter.json", case.id),
        "rawPainterSha256": hash_file(&painter_text)?,
        "mediaReadiness": media_readiness,
    });
    let mut geometry = geometry_evidence.value;
    geometry_evidence
        .issues
        .extend(super::painter_receipts::merge_into_geometry(
            &mut geometry,
            &paint_evidence,
        ));
    geometry_evidence.issues.append(&mut fingerprint_issues);
    geometry["paintPainterPath"] =
        json!(format!("{relative}/editor-{}.paint-painter.json", case.id));
    geometry["paintPainterSha256"] = json!(hash_file(&painter_path)?);
    let readiness =
        super::readiness::evaluate(&geometry, &geometry_evidence.issues, &text, &text_evidence);
    geometry["nativeReadiness"] = readiness.clone();
    text["nativeReadiness"] = readiness.clone();
    let geometry_path = directory.join(format!("editor-{}.geometry.json", case.id));
    write_json(&geometry_path, &geometry)?;
    let semantic_path = directory.join(format!("editor-{}.semantic.json", case.id));
    write_json(
        &semantic_path,
        &json!({
            "renderer": "zircon-editor-retained-host", "caseId": case.id,
            "coordinateSpace": "logical", "tree": semantic,
        }),
    )?;
    let text_path = directory.join(format!("editor-{}.text.json", case.id));
    write_json(&text_path, &text)?;
    let result = json!({
        "runtimeAssetFingerprints": text["runtimeAssetFingerprints"].clone(),
        "captureProgramFingerprints": identities
            .source_map_fingerprint()
            .map(|fingerprint| vec![fingerprint])
            .unwrap_or_default(),
        "nativeReadiness": readiness,
        "fontAssets": font_assets.ready().iter().map(|ready| json!({
            "assetRef": ready.asset_ref, "family": ready.family,
            "registeredFaceCount": ready.registered_face_count,
        })).collect::<Vec<_>>(),
        "screenshotPath": format!("{relative}/editor-{}.png", case.id),
        "screenshotSha256": hash_file(&screenshot)?,
        "geometryPath": format!("{relative}/editor-{}.geometry.json", case.id),
        "geometrySha256": hash_file(&geometry_path)?,
        "semanticPath": format!("{relative}/editor-{}.semantic.json", case.id),
        "semanticSha256": hash_file(&semantic_path)?,
        "textPath": format!("{relative}/editor-{}.text.json", case.id),
        "textSha256": hash_file(&text_path)?,
        "textPainterPath": format!("{relative}/editor-{}.text-painter.json", case.id),
        "textPainterSha256": hash_file(&painter_text)?,
        "paintPainterPath": format!("{relative}/editor-{}.paint-painter.json", case.id),
        "paintPainterSha256": hash_file(&painter_path)?,
    });
    if let Some(product) = product {
        product.close()?;
    }
    Ok(result.as_object().ok_or("invalid capture result")?.clone())
}
