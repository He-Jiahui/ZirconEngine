use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug)]
pub struct RuntimePluginId(RuntimePluginIdStorage);

#[derive(Clone, Debug)]
enum RuntimePluginIdStorage {
    Static(&'static str),
    Dynamic(Arc<str>),
}

#[allow(non_upper_case_globals)]
impl RuntimePluginId {
    pub const Ui: Self = Self::from_static("ui");
    pub const Ai: Self = Self::from_static("ai");
    pub const Physics: Self = Self::from_static("physics");
    pub const Sound: Self = Self::from_static("sound");
    pub const Texture: Self = Self::from_static("texture");
    pub const Net: Self = Self::from_static("net");
    pub const Navigation: Self = Self::from_static("navigation");
    pub const Particles: Self = Self::from_static("particles");
    pub const Animation: Self = Self::from_static("animation");
    pub const Terrain: Self = Self::from_static("terrain");
    pub const Tilemap2d: Self = Self::from_static("tilemap_2d");
    pub const PrefabTools: Self = Self::from_static("prefab_tools");
    pub const GltfImporter: Self = Self::from_static("gltf_importer");
    pub const ObjImporter: Self = Self::from_static("obj_importer");
    pub const AssetImporterData: Self = Self::from_static("asset_importer.data");
    pub const AssetImporterModel: Self = Self::from_static("asset_importer.model");
    pub const AssetImporterShader: Self = Self::from_static("asset_importer.shader");
    pub const TextureImporter: Self = Self::from_static("texture_importer");
    pub const AudioImporter: Self = Self::from_static("audio_importer");
    pub const OpusImporter: Self = Self::from_static("opus_importer");
    pub const ShaderWgslImporter: Self = Self::from_static("shader_wgsl_importer");
    pub const UiDocumentImporter: Self = Self::from_static("ui_document_importer");
    pub const Rendering: Self = Self::from_static("rendering");
    pub const VirtualGeometry: Self = Self::from_static("virtual_geometry");
    pub const HybridGi: Self = Self::from_static("hybrid_gi");
    pub const Solari: Self = Self::from_static("solari");
    pub const ZrVmLanguage: Self = Self::from_static("zr_vm_language");

    pub const fn from_static(key: &'static str) -> Self {
        Self(RuntimePluginIdStorage::Static(key))
    }

    pub fn new(raw: impl AsRef<str>) -> Self {
        Self::parse_key(raw.as_ref()).expect("runtime plugin id must be a non-empty key")
    }

    pub fn key(&self) -> &str {
        match &self.0 {
            RuntimePluginIdStorage::Static(key) => key,
            RuntimePluginIdStorage::Dynamic(key) => key,
        }
    }

    pub fn as_str(&self) -> &str {
        self.key()
    }

    pub fn label(&self) -> &str {
        match self.key() {
            "ui" => "Ui",
            "ai" => "AI",
            "physics" => "Physics",
            "sound" => "Sound",
            "texture" => "Texture",
            "net" => "Net",
            "navigation" => "Navigation",
            "particles" => "Particles",
            "animation" => "Animation",
            "terrain" => "Terrain",
            "tilemap_2d" => "Tilemap2d",
            "prefab_tools" => "PrefabTools",
            "gltf_importer" => "GltfImporter",
            "obj_importer" => "ObjImporter",
            "asset_importer.data" => "AssetImporterData",
            "asset_importer.model" => "AssetImporterModel",
            "asset_importer.shader" => "AssetImporterShader",
            "texture_importer" => "TextureImporter",
            "audio_importer" => "AudioImporter",
            "opus_importer" => "OpusImporter",
            "shader_wgsl_importer" => "ShaderWgslImporter",
            "ui_document_importer" => "UiDocumentImporter",
            "rendering" => "Rendering",
            "virtual_geometry" => "VirtualGeometry",
            "hybrid_gi" => "HybridGi",
            "solari" => "Solari",
            "zr_vm_language" => "ZrVM Language",
            key => key,
        }
    }

    pub fn parse_key(raw: &str) -> Option<Self> {
        let normalized = normalize_runtime_plugin_key(raw)?;
        Some(match normalized.as_ref() {
            "ui" => Self::Ui,
            "ai" | "artificial_intelligence" | "game_ai" => Self::Ai,
            "physics" => Self::Physics,
            "sound" | "audio" => Self::Sound,
            "texture" => Self::Texture,
            "net" | "network" => Self::Net,
            "navigation" | "nav" => Self::Navigation,
            "particles" => Self::Particles,
            "animation" => Self::Animation,
            "terrain" => Self::Terrain,
            "tilemap_2d" | "tilemap" | "tile_map_2d" => Self::Tilemap2d,
            "prefab_tools" | "prefab" | "prefabs" => Self::PrefabTools,
            "gltf_importer" | "gltf" | "glb_importer" => Self::GltfImporter,
            "obj_importer" | "obj" | "wavefront_obj" => Self::ObjImporter,
            "asset_importer.data" | "asset_importer_data" | "data_asset_importer" => {
                Self::AssetImporterData
            }
            "asset_importer.model" | "asset_importer_model" | "model_asset_importer" => {
                Self::AssetImporterModel
            }
            "asset_importer.shader" | "asset_importer_shader" | "shader_asset_importer" => {
                Self::AssetImporterShader
            }
            "texture_importer" | "image_importer" => Self::TextureImporter,
            "audio_importer" | "sound_importer" | "wav_importer" => Self::AudioImporter,
            "opus_importer" | "opus" => Self::OpusImporter,
            "shader_wgsl_importer" | "wgsl_importer" => Self::ShaderWgslImporter,
            "ui_document_importer" | "ui_importer" | "ui_asset_importer" => {
                Self::UiDocumentImporter
            }
            "rendering" | "renderer" | "graphics" => Self::Rendering,
            "vg" | "virtual_geometry" => Self::VirtualGeometry,
            "gi" | "hybrid_gi" => Self::HybridGi,
            "solari" => Self::Solari,
            "zr_vm_language" | "zr_vm" | "zrvmlanguage" => Self::ZrVmLanguage,
            _ => Self(RuntimePluginIdStorage::Dynamic(Arc::from(
                normalized.as_ref(),
            ))),
        })
    }
}

impl PartialEq for RuntimePluginId {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for RuntimePluginId {}

impl Hash for RuntimePluginId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}

impl PartialOrd for RuntimePluginId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RuntimePluginId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(other.key())
    }
}

impl AsRef<str> for RuntimePluginId {
    fn as_ref(&self) -> &str {
        self.key()
    }
}

impl fmt::Display for RuntimePluginId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.key())
    }
}

impl FromStr for RuntimePluginId {
    type Err = RuntimePluginIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse_key(raw).ok_or(RuntimePluginIdParseError)
    }
}

impl Serialize for RuntimePluginId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.key())
    }
}

impl<'de> Deserialize<'de> for RuntimePluginId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse_key(&raw)
            .ok_or_else(|| serde::de::Error::custom(format!("invalid runtime plugin id `{raw}`")))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimePluginIdParseError;

impl fmt::Display for RuntimePluginIdParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid runtime plugin id")
    }
}

impl std::error::Error for RuntimePluginIdParseError {}

fn normalize_runtime_plugin_key(raw: &str) -> Option<Cow<'_, str>> {
    let trimmed = raw.trim();
    let mut bytes = trimmed.bytes();
    let first = bytes.next()?;
    if !first.is_ascii_alphanumeric() {
        return None;
    }
    let mut has_uppercase = first.is_ascii_uppercase();
    for byte in bytes {
        if !byte.is_ascii_alphanumeric() && !matches!(byte, b'_' | b'-' | b'.') {
            return None;
        }
        has_uppercase |= byte.is_ascii_uppercase();
    }
    if has_uppercase {
        Some(Cow::Owned(trimmed.to_ascii_lowercase()))
    } else {
        Some(Cow::Borrowed(trimmed))
    }
}

#[cfg(test)]
#[path = "plugin_id/single_pass_normalization_tests.rs"]
mod single_pass_normalization_tests;

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::{BTreeSet, HashSet};
    use std::sync::Arc;
    use std::time::Instant;

    use super::{normalize_runtime_plugin_key, RuntimePluginId, RuntimePluginIdStorage};

    #[test]
    fn runtime_plugin_id_accepts_external_keys_without_core_variant() {
        let id = RuntimePluginId::new("third_party.weather_sim");

        assert_eq!(id.key(), "third_party.weather_sim");
        assert_eq!(id.label(), "third_party.weather_sim");
        assert_eq!(
            RuntimePluginId::parse_key("Third_Party.Weather_Sim"),
            Some(id)
        );
    }

    #[test]
    fn runtime_plugin_id_rejects_empty_or_non_key_text() {
        for raw in ["", " ", ".starts_with_dot", "bad id", "bad/id"] {
            assert_eq!(RuntimePluginId::parse_key(raw), None);
        }
    }

    #[test]
    fn canonical_plugin_keys_borrow_the_input_during_normalization() {
        assert!(matches!(
            normalize_runtime_plugin_key("  third_party.weather_sim  "),
            Some(Cow::Borrowed("third_party.weather_sim"))
        ));
        assert!(matches!(
            normalize_runtime_plugin_key("Third_Party.Weather_Sim"),
            Some(Cow::Owned(value)) if value == "third_party.weather_sim"
        ));
    }

    #[test]
    fn dynamic_plugin_ids_do_not_use_process_global_leaks() {
        let source = include_str!("plugin_id.rs");

        assert!(!source.contains(concat!("Box", "::leak")));
        assert!(!source.contains(concat!("HashSet<&'static", " str>")));
        assert!(!source.contains(concat!("static INTERNED", "_KEYS")));
    }

    #[test]
    fn dynamic_plugin_id_storage_retires_with_the_last_generation_owner() {
        let id = RuntimePluginId::new("third_party.weather_sim");
        let weak = match &id.0 {
            RuntimePluginIdStorage::Dynamic(key) => Arc::downgrade(key),
            RuntimePluginIdStorage::Static(_) => panic!("external plugin ID must be dynamic"),
        };
        let generation = vec![id.clone(); 64];

        drop(id);
        assert!(weak.upgrade().is_some());
        drop(generation);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn builtin_plugin_ids_keep_static_zero_allocation_storage() {
        assert!(matches!(
            &RuntimePluginId::Sound.0,
            RuntimePluginIdStorage::Static("sound")
        ));
    }

    #[test]
    fn dynamic_plugin_id_serde_hash_and_order_use_the_canonical_key() {
        let canonical = RuntimePluginId::new("third_party.weather_sim");
        let normalized = RuntimePluginId::new("Third_Party.Weather_Sim");

        assert_eq!(canonical, normalized);
        assert_eq!(
            serde_json::to_string(&canonical).expect("plugin ID should serialize"),
            "\"third_party.weather_sim\""
        );
        assert_eq!(
            serde_json::from_str::<RuntimePluginId>("\"Third_Party.Weather_Sim\"")
                .expect("plugin ID should deserialize"),
            canonical
        );

        let mut hashed = HashSet::new();
        hashed.insert(canonical.clone());
        hashed.insert(normalized);
        assert_eq!(hashed.len(), 1);

        let ordered = BTreeSet::from([
            RuntimePluginId::new("third_party.zeta"),
            RuntimePluginId::Sound,
            RuntimePluginId::new("third_party.alpha"),
        ]);
        assert_eq!(
            ordered.iter().map(RuntimePluginId::key).collect::<Vec<_>>(),
            ["sound", "third_party.alpha", "third_party.zeta"]
        );
    }

    #[test]
    #[ignore = "PERF-MVP-436 long-running ownership benchmark"]
    fn runtime_plugin_id_generation_churn_benchmark() {
        for id_count in [1, 1_000, 1_000_000] {
            let rss_before = current_rss_bytes();
            let started = Instant::now();
            let generation = (0..id_count)
                .map(|index| RuntimePluginId::new(format!("bench.plugin_{index}")))
                .collect::<Vec<_>>();
            let elapsed = started.elapsed();
            let string_bytes = generation.iter().map(|id| id.key().len()).sum::<usize>();
            let last_owner = dynamic_storage_weak(
                generation
                    .last()
                    .expect("benchmark generation should contain an ID"),
            );
            let rss_active = current_rss_bytes();

            assert_eq!(generation.len(), id_count);
            drop(generation);
            assert!(last_owner.upgrade().is_none());
            let rss_retired = current_rss_bytes();
            eprintln!(
                "PERF-MVP-436 ids={id_count} elapsed_ns={} interner_locks=0 probes=0 active_entries={id_count} active_string_bytes={string_bytes} rss_before={rss_before:?} rss_active={rss_active:?} rss_retired={rss_retired:?}",
                elapsed.as_nanos()
            );
        }

        for reload_count in [1, 1_000, 100_000] {
            let rss_before = current_rss_bytes();
            let started = Instant::now();
            for generation in 0..reload_count {
                let id = RuntimePluginId::new(format!("reload.plugin_{generation}"));
                let owner = dynamic_storage_weak(&id);
                drop(id);
                assert!(owner.upgrade().is_none());
            }
            let rss_retired = current_rss_bytes();
            eprintln!(
                "PERF-MVP-436 reloads={reload_count} elapsed_ns={} interner_locks=0 probes=0 retained_dynamic_entries=0 retained_dynamic_string_bytes=0 rss_before={rss_before:?} rss_retired={rss_retired:?}",
                started.elapsed().as_nanos()
            );
        }

        for thread_count in [1, 64] {
            let started = Instant::now();
            let workers = (0..thread_count)
                .map(|thread_index| {
                    std::thread::spawn(move || {
                        (0..1_000)
                            .map(|index| {
                                RuntimePluginId::new(format!(
                                    "thread_{thread_index}.plugin_{index}"
                                ))
                                .key()
                                .len()
                            })
                            .sum::<usize>()
                    })
                })
                .collect::<Vec<_>>();
            let checksum = workers
                .into_iter()
                .map(|worker| worker.join().expect("benchmark worker should finish"))
                .sum::<usize>();
            assert!(checksum > 0);
            eprintln!(
                "PERF-MVP-436 threads={thread_count} ids_per_thread=1000 elapsed_ns={} interner_locks=0 probes=0",
                started.elapsed().as_nanos()
            );
        }
    }

    fn dynamic_storage_weak(id: &RuntimePluginId) -> std::sync::Weak<str> {
        match &id.0 {
            RuntimePluginIdStorage::Dynamic(key) => Arc::downgrade(key),
            RuntimePluginIdStorage::Static(_) => panic!("benchmark ID must be dynamic"),
        }
    }

    #[cfg(windows)]
    fn current_rss_bytes() -> Option<usize> {
        use std::ffi::c_void;
        use std::mem::{size_of, MaybeUninit};

        #[repr(C)]
        struct ProcessMemoryCounters {
            cb: u32,
            page_fault_count: u32,
            peak_working_set_size: usize,
            working_set_size: usize,
            quota_peak_paged_pool_usage: usize,
            quota_paged_pool_usage: usize,
            quota_peak_non_paged_pool_usage: usize,
            quota_non_paged_pool_usage: usize,
            pagefile_usage: usize,
            peak_pagefile_usage: usize,
        }

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetCurrentProcess() -> *mut c_void;
        }

        #[link(name = "psapi")]
        unsafe extern "system" {
            fn GetProcessMemoryInfo(
                process: *mut c_void,
                counters: *mut ProcessMemoryCounters,
                size: u32,
            ) -> i32;
        }

        let mut counters = MaybeUninit::<ProcessMemoryCounters>::zeroed();
        let counters_ptr = counters.as_mut_ptr();
        // SAFETY: the zeroed structure has the layout required by PROCESS_MEMORY_COUNTERS,
        // and both pointers remain valid for the duration of the OS call.
        unsafe {
            (*counters_ptr).cb = size_of::<ProcessMemoryCounters>() as u32;
            if GetProcessMemoryInfo(
                GetCurrentProcess(),
                counters_ptr,
                size_of::<ProcessMemoryCounters>() as u32,
            ) == 0
            {
                return None;
            }
            Some(counters.assume_init().working_set_size)
        }
    }

    #[cfg(not(windows))]
    fn current_rss_bytes() -> Option<usize> {
        None
    }
}
