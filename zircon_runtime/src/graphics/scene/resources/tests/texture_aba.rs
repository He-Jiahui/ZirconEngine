use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use zircon_runtime_interface::project::RelPath;
use zr_rhi::{RenderDevice, RenderQueueClass, SubmissionStatus, SubmissionTicket};

use crate::asset::assets::SceneAsset;
use crate::asset::watch::{AssetChangeKind, AssetChangeSubscription, AssetEventRecvTimeoutError};
use crate::asset::{
    AssetKind, AssetManager, AssetMetaDocument, AssetUri, AssetUuid, ProjectAssetManager,
    ProjectManifest, ProjectPaths, TextureAsset,
};
use crate::core::framework::render::RenderFrameSubmissionTransaction;
use crate::core::resource::{
    ResourceActiveSnapshotError, ResourceActiveVersionKey, ResourceHandle, ResourceId,
    ResourceSnapshot, ResourceState, TextureMarker,
};
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::resources::{GpuTextureResource, ResourceStreamer};

const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const YELLOW: [u8; 4] = [255, 255, 0, 255];

#[test]
fn recreated_png_same_revision_refreshes_classic_sprite_texture_binding() {
    let project = PngProject::new();
    let a_source = project.png("same", RED);
    let ordinary_source = project.png("ordinary", BLUE);
    project
        .manager
        .open_project(project.root.to_str().unwrap())
        .unwrap();
    let changes = project.manager.subscribe_asset_changes();
    let resources = project.manager.resource_manager();
    let handle = a_source.handle(&project.manager);
    let ordinary = ordinary_source.handle(&project.manager);
    let (a_key, a_cpu) = installed_texture(&project.manager, handle, RED);
    let (ordinary_key, ordinary_cpu) = installed_texture(&project.manager, ordinary, BLUE);

    let backend = RenderBackend::new_offscreen().expect("native offscreen GPU");
    let layout = sprite_texture_layout(&backend.device);
    let mut streamer = ResourceStreamer::new_for_test(
        Arc::clone(&project.manager),
        &backend.device,
        &backend.queue,
        &layout,
    );
    let missing = ResourceId::from_stable_label("r33-texture-aba-never-published");
    let mut missing_admission = begin(&backend, 1);
    assert!(resources.registry().get(missing).is_none());
    assert!(streamer
        .ensure_sprite_texture_for_frame(&backend, &layout, missing, &mut missing_admission)
        .is_err());
    assert!(missing_admission.pre_scene_submission_tickets().is_empty());
    assert!(Arc::ptr_eq(
        &streamer.texture(Some(missing)),
        &streamer.texture(None)
    ));

    let (a, a_upload) = prepare_sprite(&mut streamer, &backend, &layout, handle, 2);
    assert_eq!(gpu_texel(&backend, &a, 2, &a_upload), RED);
    assert_eq!(a_upload.len(), 1, "one COPY upload for a fresh 1×1 PNG");
    let (stable_a, stable_a_upload) = prepare_sprite(&mut streamer, &backend, &layout, handle, 3);
    assert_same_binding(&a, &stable_a);
    assert!(stable_a_upload.is_empty());

    let (ordinary_a, ordinary_a_upload) =
        prepare_sprite(&mut streamer, &backend, &layout, ordinary, 4);
    assert_eq!(
        gpu_texel(&backend, &ordinary_a, 4, &ordinary_a_upload),
        BLUE
    );
    assert_eq!(ordinary_a_upload.len(), 1);
    write_png(&ordinary_source.path, YELLOW);
    await_change(&changes, &ordinary_source.uri, AssetChangeKind::Modified);
    assert_eq!(ordinary_source.handle(&project.manager), ordinary);
    let (ordinary_new_key, ordinary_new_cpu) =
        installed_texture(&project.manager, ordinary, YELLOW);
    assert_ne!(ordinary_key, ordinary_new_key);
    assert!(ordinary_new_cpu.revision() > ordinary_cpu.revision());
    let (ordinary_b, ordinary_b_upload) =
        prepare_sprite(&mut streamer, &backend, &layout, ordinary, 5);
    assert_eq!(
        gpu_texel(&backend, &ordinary_b, 5, &ordinary_b_upload),
        YELLOW
    );
    assert_eq!(ordinary_b_upload.len(), 1);
    let (unrelated_a, unrelated_upload) =
        prepare_sprite(&mut streamer, &backend, &layout, handle, 6);
    assert_same_binding(&a, &unrelated_a);
    assert!(
        unrelated_upload.is_empty(),
        "one modified PNG must not rebuild the other"
    );
    assert_eq!(
        resources
            .projection_snapshot()
            .active()
            .key(handle.id())
            .as_ref(),
        Some(&a_key)
    );
    println!("TEXTURE_180_CONTROLS missing_fallback=true stable_reused=true ordinary_revision={}->{} ordinary_gpu={YELLOW:?} uploads={}", ordinary_cpu.revision(), ordinary_new_cpu.revision(), ordinary_b_upload.len());

    // The editor's source deletion retires both authored files. Keep the real sidecar
    // for restoration, and do not give the renderer a frame in the gap.
    let saved_sidecar = fs::read(&a_source.sidecar).expect("back up the authored sidecar");
    let removed = project
        .manager
        .delete_project_source(a_source.uuid)
        .expect("durably delete the unreferenced PNG and its sidecar");
    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0].uri, a_source.uri.to_string());
    await_change(&changes, &a_source.uri, AssetChangeKind::Removed);
    assert!(!a_source.path.exists());
    assert!(!a_source.sidecar.exists());
    assert!(project.manager.resolve_asset_id(&a_source.uri).is_none());
    assert!(resources.registry().get(handle.id()).is_none());
    assert!(resources
        .projection_snapshot()
        .active()
        .key(handle.id())
        .is_none());
    assert!(resources
        .snapshot::<TextureMarker, TextureAsset>(handle)
        .is_none());
    // Restore the original identity before submitting the new real PNG bytes.
    // The production generated-source transaction imports and publishes them as Added.
    fs::write(&a_source.sidecar, saved_sidecar).unwrap();
    assert_eq!(
        AssetMetaDocument::load(&a_source.sidecar).unwrap().uuid,
        a_source.uuid
    );
    let green_png = png_bytes(GREEN);
    let added = project
        .manager
        .publish_generated_project_source(a_source.uri.clone(), green_png.clone())
        .expect("durably import the replacement PNG under the restored sidecar");
    assert_eq!(added.source_uri(), &a_source.uri);
    assert!(added.previous_source_hash().is_none());
    assert!(added.status().is_some());
    assert_eq!(fs::read(&a_source.path).unwrap(), green_png);
    await_change(&changes, &a_source.uri, AssetChangeKind::Added);
    assert_eq!(a_source.handle(&project.manager), handle);
    assert_eq!(
        AssetMetaDocument::load(&a_source.sidecar).unwrap().uuid,
        a_source.uuid
    );
    let (b_key, b_cpu) = installed_texture(&project.manager, handle, GREEN);
    assert_ne!(
        a_key, b_key,
        "same UUID must not reuse the retired active payload key"
    );
    assert_eq!(
        a_cpu.revision(),
        b_cpu.revision(),
        "the real producer must exercise the colliding numeric revision"
    );
    assert_ne!(a_cpu.record().source_hash, b_cpu.record().source_hash);
    assert!(matches!(
        resources.snapshot_expected::<TextureMarker, TextureAsset>(handle, &a_key),
        Err(ResourceActiveSnapshotError::Stale)
    ));
    assert_eq!(a_cpu.resource().rgba.as_slice(), RED.as_slice());
    println!("TEXTURE_180_READY id={} revision={} removed=true added=true active_keys_differ=true cpu_a={RED:?} cpu_b={GREEN:?} gpu_a={RED:?} scene_graph_submitted=false", handle.id(), b_cpu.revision());

    let (b, b_upload) = prepare_sprite(&mut streamer, &backend, &layout, handle, 7);
    let observed = gpu_texel(&backend, &b, 7, &b_upload);
    // A late native deletion echo must not make a missing B look like a cache ABA.
    let still_b = resources
        .snapshot_expected::<TextureMarker, TextureAsset>(handle, &b_key)
        .expect("the same B generation remained active through GPU readback");
    assert!(Arc::ptr_eq(still_b.resource(), b_cpu.resource()));
    println!("TEXTURE_180_OBSERVED selected_id={} same_texture={} same_binding={} copy_uploads={} gpu={observed:?}", b.id.unwrap(), a.texture == b.texture, a.bind_group == b.bind_group, b_upload.len());
    assert_eq!(observed, GREEN, "TEXTURE_180_ABA: same numeric revision cannot select old GPU texels for a different active payload");
    assert_eq!(
        b_upload.len(),
        1,
        "the new PNG payload must cause a COPY upload at admission"
    );
    let (stable_b, stable_b_upload) = prepare_sprite(&mut streamer, &backend, &layout, handle, 8);
    assert_same_binding(&b, &stable_b);
    assert!(stable_b_upload.is_empty());
}

fn installed_texture(
    manager: &ProjectAssetManager,
    handle: ResourceHandle<TextureMarker>,
    expected: [u8; 4],
) -> (ResourceActiveVersionKey, ResourceSnapshot<TextureAsset>) {
    // A successful import publishes Ready metadata; the public texture loader restores its payload.
    let loaded = manager
        .load_texture_asset_snapshot(handle.id())
        .expect("public texture service must load the published PNG");
    let resources = manager.resource_manager();
    let key = resources
        .projection_snapshot()
        .active()
        .key(handle.id())
        .expect("the public texture load installed an active PNG key");
    let cpu = resources
        .snapshot_expected::<TextureMarker, TextureAsset>(handle, &key)
        .expect("resident bytes paired with the active key");
    assert_eq!(loaded.record(), cpu.record());
    assert!(Arc::ptr_eq(loaded.resource(), cpu.resource()));
    assert_eq!(cpu.record().state, ResourceState::Ready);
    assert_eq!(cpu.record().id, handle.id());
    assert!(cpu.revision() > 0 && !cpu.record().source_hash.is_empty());
    assert_eq!((cpu.resource().width, cpu.resource().height), (1, 1));
    assert_eq!(cpu.resource().rgba.as_slice(), expected.as_slice());
    (key, cpu)
}

fn begin(backend: &RenderBackend, generation: u64) -> RenderFrameSubmissionTransaction {
    let poll = backend.poll_submission_completions().unwrap();
    RenderFrameSubmissionTransaction::begin(generation, poll)
}

fn prepare_sprite(
    streamer: &mut ResourceStreamer,
    backend: &RenderBackend,
    layout: &wgpu::BindGroupLayout,
    handle: ResourceHandle<TextureMarker>,
    generation: u64,
) -> (Arc<GpuTextureResource>, Vec<SubmissionTicket>) {
    let mut transaction = begin(backend, generation);
    streamer
        .ensure_sprite_texture_for_frame(backend, layout, handle.id(), &mut transaction)
        .unwrap();
    let uploads = transaction.pre_scene_submission_tickets();
    assert!(uploads
        .iter()
        .all(|ticket| ticket.queue_class() == RenderQueueClass::Copy));
    let chosen = streamer.texture(Some(handle.id()));
    assert_eq!(
        chosen.id,
        Some(handle.id()),
        "do not mistake fallback for an admitted texture"
    );
    assert_eq!(chosen.view().texture(), &chosen.texture);
    (chosen, uploads)
}

fn gpu_texel(
    backend: &RenderBackend,
    texture: &GpuTextureResource,
    generation: u64,
    uploads: &[SubmissionTicket],
) -> [u8; 4] {
    assert!(texture
        .texture
        .usage()
        .contains(wgpu::TextureUsages::COPY_SRC));
    assert_eq!(texture.texture.mip_level_count(), 1);
    // The test orders the real pre-scene uploads before the diagnostic copy. It does not submit a scene graph.
    backend
        .render_device
        .flush_submissions()
        .expect("flush classic texture uploads");
    let texel = backend
        .read_product_diagnostic_texture_rgba8_blocking(
            generation,
            texture.view().texture(),
            1,
            1,
            "r33-texture-cache-aba",
            &mut |_| Ok(()),
        )
        .expect("read texel from the actual bound GPU backing");
    for &ticket in uploads {
        assert!(matches!(
            backend.render_device.submission_status(ticket).unwrap(),
            SubmissionStatus::Submitted | SubmissionStatus::Completed
        ));
    }
    texel.try_into().expect("one RGBA8 texel")
}

fn assert_same_binding(first: &Arc<GpuTextureResource>, next: &Arc<GpuTextureResource>) {
    assert!(Arc::ptr_eq(first, next));
    assert_eq!(first.texture, next.texture);
    assert_eq!(first.bind_group, next.bind_group);
}

fn await_change(changes: &AssetChangeSubscription, uri: &AssetUri, want: AssetChangeKind) {
    let deadline = Instant::now() + Duration::from_secs(12);
    loop {
        match changes.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
            Ok(change) if &change.uri == uri && change.kind == want => return,
            Ok(_) if Instant::now() < deadline => {}
            Ok(_) | Err(AssetEventRecvTimeoutError::Timeout) => {
                panic!("watcher did not publish {want:?} for {uri}")
            }
            Err(error) => {
                panic!("lost ordered asset publication while waiting for {want:?} {uri}: {error}")
            }
        }
    }
}

struct PngProject {
    root: PathBuf,
    manager: Arc<ProjectAssetManager>,
}

impl PngProject {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("zircon-r33-texture-{}", AssetUuid::new()));
        fs::create_dir(&root).expect("unique private project root");
        let project = Self {
            root,
            manager: Arc::new(ProjectAssetManager::default()),
        };
        let paths = ProjectPaths::from_root(&project.root).unwrap();
        let assets = RelPath::project_assets();
        paths.ensure_layout(&[assets.clone()]).unwrap();
        let root = paths.asset_root(&assets);
        fs::create_dir_all(root.join("textures")).unwrap();
        fs::create_dir_all(root.join("scenes")).unwrap();
        let scene = SceneAsset {
            entities: Vec::new(),
        }
        .to_project_toml_string(|_| unreachable!("empty scene has no asset references"))
        .unwrap();
        fs::write(root.join("scenes/main.scene.toml"), scene).unwrap();
        ProjectManifest::new(
            "Native texture ABA regression",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();
        project
    }

    fn png(&self, filename: &str, rgba: [u8; 4]) -> PngSource {
        let paths = ProjectPaths::from_root(&self.root).unwrap();
        let directory = paths
            .asset_root(&RelPath::project_assets())
            .join("textures");
        let uri = AssetUri::parse(&format!("res://textures/{filename}.png")).unwrap();
        let path = directory.join(format!("{filename}.png"));
        let sidecar = directory.join(format!("{filename}.png.zmeta"));
        let uuid = AssetUuid::new();
        write_png(&path, rgba);
        let mut metadata = AssetMetaDocument::new(uuid, uri.clone(), AssetKind::Texture);
        // Sampled enters the same classic sprite path; COPY_SRC permits a direct 1×1 GPU probe without changing production allocation code.
        metadata.import_settings.insert(
            "usage".to_owned(),
            toml::Value::Array(
                [
                    toml::Value::String("sampled".to_owned()),
                    toml::Value::String("copy_src".to_owned()),
                ]
                .into(),
            ),
        );
        metadata.save(&sidecar).unwrap();
        PngSource {
            uri,
            path,
            sidecar,
            uuid,
        }
    }
}

impl Drop for PngProject {
    fn drop(&mut self) {
        let _ = self.manager.close_project();
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct PngSource {
    uri: AssetUri,
    path: PathBuf,
    sidecar: PathBuf,
    uuid: AssetUuid,
}

impl PngSource {
    fn handle(&self, manager: &ProjectAssetManager) -> ResourceHandle<TextureMarker> {
        let id = manager
            .resolve_asset_id(&self.uri)
            .expect("real project PNG ID");
        assert_eq!(id, ResourceId::from_asset_uuid(self.uuid));
        ResourceHandle::new(id)
    }
}

fn write_png(path: &Path, rgba: [u8; 4]) {
    fs::write(path, png_bytes(rgba)).unwrap();
}

fn png_bytes(rgba: [u8; 4]) -> Vec<u8> {
    let mut encoded = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba(rgba)))
        .write_to(&mut encoded, ImageFormat::Png)
        .unwrap();
    encoded.into_inner()
}

fn sprite_texture_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("classic-sprite-texture-aba"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}
