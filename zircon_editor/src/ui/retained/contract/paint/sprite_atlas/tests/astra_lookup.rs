use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use zircon_runtime::asset::{
    validate_sprite_atlas_asset, AssetUri, SpriteAtlasAsset, SpriteAtlasEntry, SpriteAtlasPadding,
    SpriteAtlasRect, SpriteAtlasUvRect,
};

use super::*;

struct Fixture {
    root: PathBuf,
    atlas_dir: PathBuf,
    source: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("zircon-astra-atlas-{}-{nonce}", std::process::id()));
        let source = root.join("assets/icons/search.png");
        let atlas_dir = root.join(".zircon/cache").join(ATLAS_CACHE_DIR);
        std::fs::create_dir_all(source.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&atlas_dir).unwrap();
        Self {
            root,
            atlas_dir,
            source,
        }
    }

    fn write(&self, name: &str, atlas: &SpriteAtlasAsset) -> PathBuf {
        let path = self.atlas_dir.join(name);
        std::fs::write(&path, toml::to_string(atlas).unwrap()).unwrap();
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn atlas(names: impl IntoIterator<Item = String>) -> SpriteAtlasAsset {
    SpriteAtlasAsset {
        atlas_texture: AssetUri::parse("lib://editor-sprite-atlases/icons.png").unwrap(),
        width: 1,
        height: 1,
        padding: SpriteAtlasPadding::default(),
        entries: names
            .into_iter()
            .map(|name| SpriteAtlasEntry {
                source: Some(AssetUri::parse(&format!("res://icons/{name}.png")).unwrap()),
                name,
                pixel_rect: SpriteAtlasRect {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                },
                uv_rect: SpriteAtlasUvRect {
                    min: [0.0, 0.0],
                    max: [1.0, 1.0],
                },
                source_width: 1,
                source_height: 1,
            })
            .collect(),
    }
}

#[test]
fn astra_m4_resolver_skips_invalid_and_nonmatching_manifests_and_keeps_first_match() {
    let fixture = Fixture::new();
    std::fs::write(fixture.atlas_dir.join("01-corrupt.toml"), b"not [valid").unwrap();
    fixture.write("02-other.toml", &atlas(["other".into()]));
    let mut invalid = atlas(["search".into()]);
    invalid.width = 0;
    fixture.write("03-invalid.toml", &invalid);
    let image_name = format!(
        "{}.png",
        fixture.root.file_name().unwrap().to_str().unwrap()
    );
    let mut matching = atlas(["search".into()]);
    matching.atlas_texture =
        AssetUri::parse(&format!("lib://editor-sprite-atlases/{image_name}")).unwrap();
    let expected = fixture.write("04-match.toml", &matching);
    let mut second = atlas(["search".into()]);
    second.atlas_texture = AssetUri::parse("lib://editor-sprite-atlases/second.png").unwrap();
    fixture.write("05-match.toml", &second);
    let selected = resolve_atlas_uncached("search", &fixture.source).unwrap();
    assert_eq!(selected.manifest_path, expected);
    assert_eq!(selected.resource_key, matching.atlas_texture.to_string());
    assert!(resolve_atlas_uncached("missing", &fixture.source).is_none());
    ::image::save_buffer(
        fixture.atlas_dir.join(image_name),
        &[12, 34, 56, 255],
        1,
        1,
        ::image::ColorType::Rgba8,
    )
    .unwrap();
    let image = resolve_editor_sprite_atlas_image("template-icon:search", &fixture.source).unwrap();
    assert_eq!(image.resource_key, selected.resource_key);
    assert_eq!((image.width, image.height), (1, 1));
    assert_eq!(image.uv.min, [0.0, 0.0]);
    assert_eq!(image.uv.max, [1.0, 1.0]);
}

#[test]
fn astra_m4_manifest_hits_share_allocation_and_negative_hits_do_not_reload() {
    let fixture = Fixture::new();
    let original = atlas((0..1_000).map(|id| format!("entry_{id}")));
    let path = fixture.write("manifest.toml", &original);
    let cache = cache::AtlasManifestCache::default();
    let first = cache.load(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
    let second = cache.load(&path).unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(*first, original);
    let absent = fixture.atlas_dir.join("absent.toml");
    assert!(cache.load(&absent).is_none());
    fixture.write("absent.toml", &original);
    assert!(cache.load(&absent).is_none());
    let reloaded = cache::AtlasManifestCache::default().load(&absent).unwrap();
    assert_eq!(*reloaded, original);
}

#[derive(Default)]
struct LegacyCache {
    entries: Mutex<HashMap<PathBuf, Option<SpriteAtlasAsset>>>,
}

impl LegacyCache {
    fn load(&self, path: &Path) -> Option<SpriteAtlasAsset> {
        let key = path.to_path_buf();
        if let Some(cached) = self
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(&key)
        {
            return cached.clone();
        }
        let value = std::fs::read_to_string(path)
            .ok()
            .and_then(|text| toml::from_str::<SpriteAtlasAsset>(&text).ok())
            .filter(|atlas| validate_sprite_atlas_asset(atlas).is_ok());
        self.entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(key, value.clone());
        value
    }
}

fn percentiles(samples: &mut [u128]) -> [u128; 3] {
    samples.sort_unstable();
    [50, 95, 99].map(|p| samples[(samples.len() * p).div_ceil(100) - 1])
}

#[test]
#[ignore = "Windows release evidence through the coordinator"]
fn astra_m4_manifest_cache_release_evidence() {
    assert!(
        !cfg!(debug_assertions),
        "performance evidence requires release"
    );
    const SAMPLES: usize = 101;
    const HITS: usize = 200;
    let fixture = Fixture::new();
    for count in [1, 1_000, 10_000] {
        let path = fixture.write(
            &format!("{count}.toml"),
            &atlas((0..count).map(|id| format!("entry_{id}"))),
        );
        let legacy = LegacyCache::default();
        let shared = cache::AtlasManifestCache::default();
        assert_eq!(legacy.load(&path).unwrap(), *shared.load(&path).unwrap());
        for _ in 0..8 {
            black_box(legacy.load(&path).unwrap());
            black_box(shared.load(&path).unwrap());
        }
        let mut before = Vec::with_capacity(SAMPLES);
        let mut after = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            for optimized in if sample % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                let start = Instant::now();
                for _ in 0..HITS {
                    if optimized {
                        black_box(shared.load(black_box(&path)).unwrap());
                    } else {
                        black_box(legacy.load(black_box(&path)).unwrap());
                    }
                }
                let elapsed = start.elapsed().as_nanos();
                if optimized {
                    after.push(elapsed);
                } else {
                    before.push(elapsed);
                }
            }
        }
        let before = percentiles(&mut before);
        let after = percentiles(&mut after);
        let limit = if count == 1 { 105 } else { 20 };
        println!(
            "ASTRA_M4_ATLAS_CACHE profile=release entries={count} hits={HITS} samples={SAMPLES} warmup=8 before_p50_p95_p99_ns={before:?} after_p50_p95_p99_ns={after:?} p95_limit_percent={limit}"
        );
        assert!(
            after[1] * 100 <= before[1] * limit,
            "atlas cache p95 target missed: {before:?} -> {after:?}"
        );
    }
}
