use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use image::{codecs::png::PngEncoder, ExtendedColorType, ImageEncoder};
use serde_json::Value;
use zircon_runtime::core::resource::io::atomic_write;
use zircon_runtime::{core::math::UVec2, ui::icon_atlas::builtin_icon_supported};
use zircon_runtime_interface::ui::surface::{UiRenderExtract, UiVisualAssetRef};

use super::{
    assets,
    catalog::{self, Case, Catalog, Entry},
    evidence::{self, Evidence},
    preview,
};

pub(super) fn repo_root() -> PathBuf {
    std::env::var_os("ZUI_LAYOUT_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("runtime crate parent")
                .to_path_buf()
        })
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<String, String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    atomic_write(path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(catalog::sha256(&bytes))
}

fn write_png(path: &Path, rgba: &[u8], width: u32, height: u32) -> Result<String, String> {
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(rgba, width, height, ExtendedColorType::Rgba8)
        .map_err(|e| e.to_string())?;
    atomic_write(path, &bytes).map_err(|e| e.to_string())?;
    Ok(catalog::sha256(&bytes))
}

fn has_renderable_text(extract: &UiRenderExtract) -> bool {
    extract.list.commands.iter().any(|command| {
        command
            .text
            .as_deref()
            .is_some_and(|text| !text.trim().is_empty())
            && command.frame.width > 0.0
            && command.frame.height > 0.0
            && command.opacity > 0.0
    })
}

fn native_text_stats_are_sufficient(
    has_renderable_text: bool,
    glyph_count: usize,
    visible_raster_glyph_count: usize,
) -> bool {
    !has_renderable_text || (glyph_count > 0 && visible_raster_glyph_count > 0)
}

fn render_case(
    repo: &Path,
    catalog_root: &Path,
    entry: &Entry,
    case: &Case,
    raw: &Value,
    evidence: &mut Evidence,
) -> Result<(), String> {
    let (source_hash, dependency_hash, mut dependency_paths) =
        catalog::verify_entry(repo, catalog_root, entry)?;
    evidence.source_sha256 = source_hash;
    evidence.dependency_sha256 = dependency_hash;
    evidence.case_sha256 = catalog::canonical_hash(raw)?;
    catalog::validate_case(case, entry)?;
    let metrics = catalog::window_metrics(case)?;
    let source = catalog::source_path(repo, &case.source_path)?;
    let consumer = catalog::verify_review_host(catalog_root, entry, case)?;
    let theme_source = case
        .theme_source_path
        .as_ref()
        .map(|path| catalog::source_path(repo, path))
        .transpose()?
        .or_else(|| {
            consumer
                .as_ref()
                .filter(|_| !matches!(case.host.as_str(), "component" | "toolbar"))
                .map(|_| source.clone())
        });
    if consumer.is_some() {
        dependency_paths.push(source.clone());
    }
    evidence.review_host_sha256 = case.review_host.as_ref().map(|host| host.sha256.clone());
    let relative = catalog::case_directory(entry, case)?;
    let directory = catalog_root.join(&relative);
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    // Build once before renderer construction so the asset preparation can inspect every
    // referenced image/font. The renderer admits the project font into the process-owned
    // collection during construction; text artifacts produced before that admission carry
    // the previous collection revision and are intentionally rejected by the renderer.
    let bootstrap_preview = preview::build_with_data(
        consumer.as_deref().unwrap_or(&source),
        &dependency_paths,
        &format!("native-zui.{}", case.id),
        metrics,
        &case.state,
        theme_source.as_deref(),
        &case.data,
    )?;

    let mut limitations = Vec::new();
    if !matches!(case.host.as_str(), "component" | "fixture" | "woc") {
        limitations.push(format!(
            "Runtime GPU diagnostic does not establish {} product host parity",
            case.host
        ));
    }
    let work = directory.join(format!("engine-{}-work-{}", case.id, std::process::id()));
    if work.exists() {
        return Err(format!(
            "capture work directory already exists: {}",
            work.display()
        ));
    }
    let prepared = assets::prepare_asset_manager(
        repo,
        &source,
        &dependency_paths,
        &work,
        &bootstrap_preview.extract,
    )?;
    evidence.runtime_asset_fingerprints = prepared.fingerprints;
    let mut renderer = super::product_renderer::ProductUiFrameRenderer::new(
        UVec2::new(metrics.physical_size.width, metrics.physical_size.height),
        prepared.manager,
    );

    // Rebuild after the renderer has admitted the project font. This gives the captured
    // extract artifacts the same font-collection revision that the GPU text pipeline uses.
    let preview::NativePreview {
        surface,
        extract,
        source_ids,
    } = preview::build_with_data(
        consumer.as_deref().unwrap_or(&source),
        &dependency_paths,
        &format!("native-zui.{}", case.id),
        metrics,
        &case.state,
        theme_source.as_deref(),
        &case.data,
    )?;
    if extract.list.commands.iter().any(|command| {
        matches!(
            command.image.as_ref(),
            Some(UiVisualAssetRef::Icon(icon)) if !builtin_icon_supported(icon)
        )
    }) {
        limitations.push(
            "UiVisualAssetRef::Icon uses an icon outside the Runtime built-in icon set".into(),
        );
    }
    let semantic_nodes = super::semantic::nodes(&surface, &source_ids)?;

    let geometry_name = format!("engine-{}.geometry.json", case.id);
    evidence.geometry_sha256 = Some(write_json(
        &directory.join(&geometry_name),
        &serde_json::json!({
            "case": raw, "rendererKind": evidence.renderer_kind, "arrangedTree": surface.arranged_tree,
            "windowMetrics": metrics, "logicalRenderExtract": surface.render_extract,
            "layout": {"coordinateSpace":"logical", "semanticNodes":semantic_nodes},
            "renderExtract": extract, "rebuildReport": surface.last_rebuild_report
        }),
    )?);
    evidence.geometry_path = Some(
        relative
            .join(&geometry_name)
            .to_string_lossy()
            .replace('\\', "/"),
    );
    let text_name = format!("engine-{}.text.json", case.id);
    evidence.text_sha256 = Some(write_json(
        &directory.join(&text_name),
        &serde_json::json!({"case":raw,"coordinateSpace":"logical", "nodes":super::semantic::text_nodes(&surface, &source_ids)?}),
    )?);
    evidence.text_path = Some(
        relative
            .join(&text_name)
            .to_string_lossy()
            .replace('\\', "/"),
    );
    let (frame, stats) = renderer.render_ui_extract_frame(extract.clone());
    if !native_text_stats_are_sufficient(
        has_renderable_text(&extract),
        stats.last_ui_text_glyph_count,
        stats.last_ui_text_visible_raster_glyph_count,
    ) {
        limitations.push(format!(
            "native text commands were emitted but GPU rasterization produced no visible glyphs (glyphs={}, visibleRasterGlyphs={})",
            stats.last_ui_text_glyph_count, stats.last_ui_text_visible_raster_glyph_count
        ));
    }
    if stats.last_ui_text_unmapped_glyph_count != 0
        || stats.last_ui_text_visible_missing_raster_image_count != 0
        || stats.last_ui_text_layout_fallback_count != 0
        || stats.last_ui_text_other_layout_error_count != 0
    {
        limitations.push(
            "native text reported unmapped glyphs, missing raster images or layout fallback".into(),
        );
    }
    if frame.width != metrics.physical_size.width
        || frame.height != metrics.physical_size.height
        || frame.rgba.len() != frame.width as usize * frame.height as usize * 4
    {
        return Err("captured framebuffer dimensions do not match case physical viewport".into());
    }
    if frame
        .rgba
        .chunks_exact(4)
        .all(|pixel| pixel == &frame.rgba[..4])
    {
        limitations.push(
            "captured framebuffer is uniform; visible product content was not established".into(),
        );
    }
    let image_name = format!("engine-{}.png", case.id);
    evidence.screenshot_sha256 = write_png(
        &directory.join(&image_name),
        &frame.rgba,
        frame.width,
        frame.height,
    )?;
    evidence.screenshot_path = relative
        .join(image_name)
        .to_string_lossy()
        .replace('\\', "/");
    write_json(
        &directory.join(format!("engine-{}.stats.json", case.id)),
        &serde_json::json!({
            "submittedFrames":stats.submitted_frames, "uiCommands":stats.last_ui_command_count,
            "textGlyphs":stats.last_ui_text_glyph_count, "unmappedGlyphs":stats.last_ui_text_unmapped_glyph_count,
            "visibleRasterGlyphs":stats.last_ui_text_visible_raster_glyph_count,
            "rasterSourceImages":stats.last_ui_text_raster_source_image_count,
            "persistentRasterKeys":stats.last_ui_text_raster_persistent_key_count,
            "sourceCacheMisses":stats.last_ui_text_raster_source_cache_miss_count,
            "missingRasterImages":stats.last_ui_text_missing_raster_image_count,
            "visibleMissingRasterImages":stats.last_ui_text_visible_missing_raster_image_count,
            "visibleRasterPlaceholders":stats.last_ui_text_visible_raster_placeholder_count,
            "pendingGlyphWork":stats.last_ui_text_raster_worker_pending_count,
            "failedGlyphWork":stats.last_ui_text_raster_worker_failed_count,
            "retryQueuedGlyphs":stats.last_ui_text_raster_retry_queued_glyph_count,
            "retryQueueOverflowGlyphs":stats.last_ui_text_raster_retry_queue_overflow_glyph_count,
            "retryRejectedSources":stats.last_ui_text_raster_retry_rejected_source_count,
            "rasterUploadRequeues":stats.last_ui_text_raster_renderer_upload_requeued_count,
            "rasterUploadFailures":stats.last_ui_text_raster_renderer_upload_failure_count,
            "sdfPendingBatches":stats.last_ui_text_sdf_generation_pending_batch_count,
            "sdfCompletionBacklog":stats.last_ui_text_sdf_generation_completion_backlog_count,
            "sdfGenerationFailures":stats.last_ui_text_sdf_generation_failure_count,
            "layoutFallbacks":stats.last_ui_text_layout_fallback_count,
            "otherLayoutErrors":stats.last_ui_text_other_layout_error_count,
            "imagePayloads":stats.last_ui_image_payload_count
        }),
    )?;
    // Detect edits while asset import and GPU capture were running.
    catalog::verify_entry(repo, catalog_root, entry)?;
    catalog::verify_review_host(catalog_root, entry, case)?;
    for (path, expected) in &evidence.runtime_asset_fingerprints {
        let actual = if Path::new(path).is_absolute() {
            catalog::file_hash(Path::new(path))?
        } else {
            catalog::file_hash(&catalog::source_path(repo, path)?)?
        };
        if actual != *expected {
            return Err(format!("runtime asset changed while capturing: {path}"));
        }
    }
    if limitations.is_empty() {
        Ok(())
    } else {
        Err(limitations.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime::ui::icon_atlas::builtin_icon_supported;

    #[test]
    fn native_visual_acceptance_allows_the_runtime_builtin_feedback_icons() {
        assert!(builtin_icon_supported("info"));
        assert!(builtin_icon_supported("check-circle"));
        assert!(builtin_icon_supported("alert-triangle"));
        assert!(builtin_icon_supported("x-circle"));
        assert!(builtin_icon_supported("package"));
    }

    #[test]
    fn native_visual_acceptance_preserves_the_unknown_icon_limitation() {
        assert!(!builtin_icon_supported("project-custom-icon"));
    }

    #[test]
    fn native_visual_acceptance_rejects_text_without_visible_gpu_glyphs() {
        assert!(!super::native_text_stats_are_sufficient(true, 0, 0));
        assert!(!super::native_text_stats_are_sufficient(true, 12, 0));
        assert!(!super::native_text_stats_are_sufficient(true, 0, 12));
        assert!(super::native_text_stats_are_sufficient(true, 12, 12));
        assert!(super::native_text_stats_are_sufficient(false, 0, 0));
    }
}

pub(super) fn export() -> Result<(), String> {
    let repo = repo_root().canonicalize().map_err(|e| e.to_string())?;
    let catalog_path = std::env::var_os("ZUI_LAYOUT_CATALOG")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo.join("docs/layout/catalog.json"))
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let catalog_root = catalog_path.parent().ok_or("catalog parent missing")?;
    let catalog: Catalog =
        serde_json::from_slice(&std::fs::read(&catalog_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let renderer_hash = catalog::file_hash(&executable)?;
    let source_filter = std::env::var("ZUI_LAYOUT_SOURCE").ok();
    let case_filter = std::env::var("ZUI_LAYOUT_CASE").ok();
    let mut report = Vec::new();
    let mut delegated = Vec::new();
    let mut identities = std::collections::BTreeSet::new();
    for entry in &catalog.entries {
        if source_filter
            .as_ref()
            .is_some_and(|filter| filter != &entry.source_path)
        {
            continue;
        }
        if entry.cases.is_empty() {
            return Err(format!(
                "{} has no native review cases; regenerate the catalog",
                entry.source_path
            ));
        }
        for raw in &entry.cases {
            let case: Case = serde_json::from_value(raw.clone()).map_err(|e| e.to_string())?;
            if case_filter
                .as_ref()
                .is_some_and(|filter| filter != &case.id)
            {
                continue;
            }
            if catalog::requires_editor_renderer(entry, &case) {
                delegated.push(serde_json::json!({
                    "sourcePath": entry.source_path,
                    "caseId": case.id,
                    "rendererKind": "zircon-editor-retained-host",
                    "reason": "product Editor/plugin surfaces are captured by the retained host",
                }));
                continue;
            }
            let identity = (entry.category.clone(), entry.name.clone(), case.id.clone());
            if !identities.insert(identity) {
                return Err("duplicate native evidence output identity".into());
            }
            let mut item = evidence::failed(
                &case,
                &entry.source_path,
                String::new(),
                String::new(),
                String::new(),
                String::new(),
            );
            item.renderer_sha256 = renderer_hash.clone();
            item.renderer_path = executable.to_string_lossy().into_owned();
            let result = catch_unwind(AssertUnwindSafe(|| {
                render_case(&repo, catalog_root, entry, &case, raw, &mut item)
            }));
            match result {
                Ok(Ok(())) => {
                    item.status = "passed".into();
                    item.error = None;
                }
                Ok(Err(error)) => item.error = Some(error),
                Err(payload) => {
                    item.error = Some(
                        payload
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| payload.downcast_ref::<&str>().map(|text| text.to_string()))
                            .unwrap_or_else(|| "native renderer panicked".into()),
                    )
                }
            }
            report.push(item);
            write_json(
                &catalog_root.join("engine-report.json"),
                &serde_json::json!({
                    "schema":"dev.zircon.zui.native-evidence",
                    "version":1,
                    "rendererSha256":renderer_hash,
                    "delegatedCases":delegated,
                    "evidence":report
                }),
            )?;
        }
    }
    if report.is_empty() {
        return Err("native capture filters selected no cases".into());
    }
    let failed = report.iter().filter(|item| item.status != "passed").count();
    if failed != 0 {
        return Err(format!(
            "{failed} of {} native cases failed; see engine-report.json",
            report.len()
        ));
    }
    Ok(())
}
