use super::*;

#[test]
fn dynamic_cubemap_upload_has_no_queue_or_render_thread_texel_fallback() {
    let source = include_str!("../environment_cubemap.rs");
    let product = source
        .split("#[cfg(test)]")
        .next()
        .expect("product source precedes tests");
    let dynamic_upload = product
        .split("fn ensure_uploaded(")
        .nth(1)
        .and_then(|source| source.split("fn discard_pending_upload").next())
        .expect("dynamic upload owner must remain bounded");

    assert!(!dynamic_upload.contains("queue:"));
    assert!(!dynamic_upload.contains("queue.write_buffer("));
    assert!(!dynamic_upload.contains("queue.write_texture("));
    assert!(!dynamic_upload.contains("source_texels()"));
    assert!(!dynamic_upload.contains("pmrem_texels()"));
    assert!(!dynamic_upload.contains("upload_cubemap_texels("));
    assert!(!dynamic_upload.contains("create_sampler(device)"));
    assert!(!dynamic_upload.contains("self.sampler ="));
}

#[test]
fn fallback_slots_share_one_generation_owned_black_cube_and_sampler() {
    let source = include_str!("../environment_cubemap.rs");
    let fallback = source
        .split("fn fallback(")
        .nth(1)
        .and_then(|source| source.split("fn texture_layout_entry").next())
        .expect("fallback construction must remain bounded");

    assert_eq!(fallback.matches("create_texture(").count(), 0);
    assert_eq!(fallback.matches("create_view(").count(), 0);
    assert_eq!(fallback.matches("create_sampler(").count(), 0);
    assert_eq!(fallback.matches("write_texture(").count(), 0);
    assert!(fallback.contains("system_textures.black_cube_texture().clone()"));
    assert!(fallback.contains("system_textures.black_cube_view().clone()"));
    assert!(fallback.contains("system_textures.linear_clamp_sampler().clone()"));
    assert!(fallback.contains("resident_source_texture_bytes: 0"));
    assert!(fallback.contains("resident_specular_texture_bytes: 0"));
    assert!(fallback.contains("resident_irradiance_texture_bytes: 0"));
    assert_eq!(fallback.matches("fallback_texture.clone()").count(), 2);
    assert_eq!(fallback.matches("fallback_view.clone()").count(), 2);
}

#[test]
fn cubemap_upload_changes_skip_unaffected_texture_groups() {
    let current = upload_key(1, [1; 4], [2; 4], [3; 4]);

    assert_eq!(
        cubemap_upload_changes(current, current, false),
        CubemapUploadChanges {
            source: false,
            specular: false,
            irradiance: false,
        }
    );
    assert_eq!(
        cubemap_upload_changes(current, upload_key(1, [1; 4], [2; 4], [4; 4]), false),
        CubemapUploadChanges {
            source: false,
            specular: false,
            irradiance: true,
        }
    );
    assert_eq!(
        cubemap_upload_changes(current, upload_key(1, [1; 4], [4; 4], [3; 4]), false),
        CubemapUploadChanges {
            source: false,
            specular: true,
            irradiance: false,
        }
    );
    assert_eq!(
        cubemap_upload_changes(current, upload_key(2, [1; 4], [2; 4], [3; 4]), false),
        CubemapUploadChanges {
            source: true,
            specular: true,
            irradiance: false,
        }
    );
    assert_eq!(
        cubemap_upload_changes(current, upload_key(1, [4; 4], [2; 4], [3; 4]), false),
        CubemapUploadChanges {
            source: true,
            specular: true,
            irradiance: false,
        }
    );
    assert_eq!(
        cubemap_upload_changes(current, current, true),
        CubemapUploadChanges {
            source: true,
            specular: true,
            irradiance: true,
        }
    );
}

#[test]
fn cubemap_upload_key_advances_only_after_frame_submission() {
    let previous = upload_key(1, [1; 4], [2; 4], [3; 4]);
    let next = upload_key(2, [4; 4], [5; 4], [6; 4]);
    let mut state = CubemapUploadState::new(previous);

    state.record(next);
    assert_eq!(state.committed(), previous);
    state.discard();
    assert_eq!(state.committed(), previous);

    state.record(next);
    state.commit();
    assert_eq!(state.committed(), next);
}

#[test]
fn logical_texture_budget_counts_all_cube_faces_and_mips() {
    assert_eq!(cubemap_texture_texel_bytes(1, 1), 48);
    assert_eq!(cubemap_texture_texel_bytes(2, 2), 6 * (4 + 1) * 8);
    assert_eq!(cubemap_texture_texel_bytes(4, 3), 6 * (16 + 4 + 1) * 8);
}

#[test]
fn prepared_cubemap_upload_batches_all_faces_for_each_mip() {
    let source = include_str!("../environment_cubemap.rs");
    let product = source
        .split("#[cfg(test)]")
        .next()
        .expect("product source precedes tests");

    assert!(product.contains("CubemapUploadStagingArena"));
    assert!(product.contains(".encode(device, encoder, &prepared_uploads, frame_uploads)"));
    assert!(product.contains("self.upload_state.record(upload_key);"));
    assert!(!product.contains("self.upload_key = upload_key;"));
}

#[test]
fn cubemap_upload_validates_before_rebind_and_records_after_staging() {
    let source = include_str!("../environment_cubemap.rs");
    let dynamic_upload = source
        .split("fn ensure_uploaded(")
        .nth(1)
        .and_then(|source| source.split("fn discard_pending_upload").next())
        .expect("dynamic upload owner must remain bounded");
    let validate = dynamic_upload
        .find("environment.prepared_upload_artifact()")
        .expect("artifact must be validated");
    let rebind = dynamic_upload
        .find("if requires_rebind")
        .expect("resource replacement must remain explicit");
    let stage = dynamic_upload
        .find(".encode(device, encoder, &prepared_uploads, frame_uploads)")
        .expect("staging must enter the caller frame upload batch");
    let prepare = dynamic_upload
        .find("self.pending_resources = Some(CubemapResources")
        .expect("new cubemap resources must stay pending until submission");
    let record = dynamic_upload
        .find("self.upload_state.record(upload_key)")
        .expect("pending upload identity must be recorded");

    assert!(validate < rebind);
    assert!(rebind < stage);
    assert!(stage < prepare);
    assert!(prepare < record);
    assert!(!dynamic_upload.contains("self.resources ="));
    assert!(!dynamic_upload.contains("self.upload_state ="));
}

fn upload_key(
    source_revision: u64,
    source_hash: [u32; 4],
    pmrem_hash: [u32; 4],
    irradiance_cube_hash: [u32; 4],
) -> SourceCubemapUploadKey {
    SourceCubemapUploadKey {
        source_revision,
        source_hash,
        pmrem_hash,
        irradiance_cube_hash,
    }
}
