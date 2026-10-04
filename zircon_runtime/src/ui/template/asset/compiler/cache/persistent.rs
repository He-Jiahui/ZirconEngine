use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::template::{
    UiAssetFingerprint, UiCompileCacheKey, UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION,
    UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION, UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION,
};

use super::super::package::UiRuntimeCompiledAssetArtifact;

const STORE_RECORD_VERSION: u32 = 1;
const STORE_ARTIFACT_EXTENSION: &str = "zuiart";
const STORE_PAYLOAD_EXTENSION: &str = "zuicache";
const MAX_ASSET_STEM_LEN: usize = 80;

/// 磁盘复用身份：编译输入摘要与 envelope/编译器版本共同隔离产物；V2 不透明载荷也可使用自己的 schema 版本。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiCompiledArtifactKey {
    pub asset_id: String,
    pub fingerprint: u64,
    pub schema_version: u32,
    pub compiler_version: u32,
}

impl UiCompiledArtifactKey {
    pub fn new(
        asset_id: impl Into<String>,
        fingerprint: u64,
        schema_version: u32,
        compiler_version: u32,
    ) -> Self {
        Self {
            asset_id: asset_id.into(),
            fingerprint,
            schema_version,
            compiler_version,
        }
    }

    /// 为当前模板 envelope 生成键；传入的资产 ID 必须对应这个编译输入，写入产物时还会核对二者。
    pub fn from_compile_cache_key(
        asset_id: impl Into<String>,
        cache_key: &UiCompileCacheKey,
    ) -> Self {
        Self::from_compile_cache_key_with_versions(
            asset_id,
            cache_key,
            UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION,
            UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION,
        )
    }

    /// 供格式有独立版本的调用方显式划分存储域；同一输入摘要不能替代 schema 或编译器版本兼容性。
    pub fn from_compile_cache_key_with_versions(
        asset_id: impl Into<String>,
        cache_key: &UiCompileCacheKey,
        schema_version: u32,
        compiler_version: u32,
    ) -> Self {
        Self::new(
            asset_id,
            Self::fingerprint_compile_cache_key(cache_key),
            schema_version,
            compiler_version,
        )
    }

    /// 从已编译包的头部派生复用身份，避免调用方分别拼装资产、输入摘要和编译器版本。
    pub fn from_artifact(artifact: &UiRuntimeCompiledAssetArtifact) -> Self {
        let header = &artifact.report.header;
        Self::from_compile_cache_key_with_versions(
            header.asset.id.clone(),
            &header.compile_cache_key,
            UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION,
            header.compiler_schema_version,
        )
    }

    /// 把所有失效维度按固定顺序编码；长度前缀保留字符串和导入集合边界，摘要用于缓存定位。
    pub fn fingerprint_compile_cache_key(cache_key: &UiCompileCacheKey) -> u64 {
        let mut bytes = Vec::new();
        push_fingerprint(&mut bytes, cache_key.root_document);
        push_fingerprint_map(&mut bytes, &cache_key.widget_imports);
        push_fingerprint_map(&mut bytes, &cache_key.style_imports);
        push_fingerprint(&mut bytes, cache_key.declared_widget_imports_revision);
        push_fingerprint(&mut bytes, cache_key.declared_style_imports_revision);
        push_u64(&mut bytes, cache_key.descriptor_registry_revision);
        push_fingerprint(&mut bytes, cache_key.component_contract_revision);
        push_fingerprint(&mut bytes, cache_key.resource_dependencies_revision);
        UiAssetFingerprint::from_bytes(&bytes).value
    }
}

/// 可再生成产物的磁盘缓存，损坏或版本不匹配视为未命中；权限等真实 I/O 失败仍交给宿主处理。
/// 模板包与 V2 不透明载荷分开存储；载荷内容的反序列化和自身版本检查由各调用方负责。
#[derive(Clone, Debug)]
pub struct UiCompiledArtifactStore {
    root: PathBuf,
}

/// 按资产跨版本清理的实际删除统计，供宿主观察缓存回收；不统计无法识别或无法读取的记录。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UiCompiledArtifactStoreEvictionReport {
    pub files_removed: usize,
    pub bytes_removed: u64,
}

impl UiCompiledArtifactStore {
    /// 宿主选择缓存根目录；创建实例本身不访问磁盘，写入时才创建对应版本目录。
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// 提供模板包的版本化位置，便于宿主诊断和测试损坏记录；V2 载荷使用独立后缀。
    pub fn artifact_path(&self, key: &UiCompiledArtifactKey) -> PathBuf {
        self.path_for_key(key, STORE_ARTIFACT_EXTENSION)
    }

    /// 读取并恢复已验证模板包；未命中应回到源资产编译，不能把缓存缺失解释为源资产不存在。
    pub fn load(
        &self,
        key: &UiCompiledArtifactKey,
    ) -> io::Result<Option<UiRuntimeCompiledAssetArtifact>> {
        let Some(artifact_bytes) = self.load_bytes(key)? else {
            return Ok(None);
        };
        Ok(UiRuntimeCompiledAssetArtifact::from_bytes(&artifact_bytes).ok())
    }

    /// 只返回 envelope、绑定程序结构和缓存键均通过检查的包字节；调用方仍需决定何时安装运行时状态。
    pub fn load_bytes(&self, key: &UiCompiledArtifactKey) -> io::Result<Option<Vec<u8>>> {
        let payload = match fs::read(self.artifact_path(key)) {
            Ok(payload) => payload,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let record = match bincode::deserialize::<UiCompiledArtifactDiskRecord>(&payload) {
            Ok(record) => record,
            Err(_) => return Ok(None),
        };
        if record.record_version != STORE_RECORD_VERSION || record.key != *key {
            return Ok(None);
        }
        let artifact = match UiRuntimeCompiledAssetArtifact::from_bytes(&record.artifact_bytes) {
            Ok(artifact) => artifact,
            Err(_) => return Ok(None),
        };
        if !artifact_matches_key(key, &artifact) {
            return Ok(None);
        }
        Ok(Some(record.artifact_bytes))
    }

    /// 只验证外层记录身份；V2 文件缓存随后检查自己的记录版本和源路径快照，通用存储不认识该载荷结构。
    pub fn load_payload_bytes(&self, key: &UiCompiledArtifactKey) -> io::Result<Option<Vec<u8>>> {
        let payload = match fs::read(self.payload_path(key)) {
            Ok(payload) => payload,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let record = match bincode::deserialize::<UiCompiledPayloadDiskRecord>(&payload) {
            Ok(record) => record,
            Err(_) => return Ok(None),
        };
        if record.record_version != STORE_RECORD_VERSION || record.key != *key {
            return Ok(None);
        }
        Ok(Some(record.payload_bytes))
    }

    /// 写入模板包前验证序列化结果与键的一致性；这是可重新编译的缓存写入，不能替代源文档持久化。
    pub fn store(
        &self,
        key: &UiCompiledArtifactKey,
        artifact: &UiRuntimeCompiledAssetArtifact,
    ) -> io::Result<PathBuf> {
        let artifact_bytes = artifact.to_bytes().map_err(invalid_data)?;
        self.store_bytes(key, &artifact_bytes)
    }

    /// 供已有序列化结果的调用方写入模板通道；拒绝不属于该键的包，避免把不同输入产物置于同一路径。
    pub fn store_bytes(
        &self,
        key: &UiCompiledArtifactKey,
        artifact_bytes: &[u8],
    ) -> io::Result<PathBuf> {
        let artifact =
            UiRuntimeCompiledAssetArtifact::from_bytes(artifact_bytes).map_err(invalid_data)?;
        if !artifact_matches_key(key, &artifact) {
            return Err(invalid_data(format!(
                "compiled artifact does not match persistent cache key for {}",
                key.asset_id
            )));
        }

        let path = self.artifact_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let record = UiCompiledArtifactDiskRecord {
            record_version: STORE_RECORD_VERSION,
            key: key.clone(),
            artifact_bytes: artifact_bytes.to_vec(),
        };
        let payload = bincode::serialize(&record).map_err(invalid_data)?;
        fs::write(&path, payload)?;
        Ok(path)
    }

    /// V2 缓存携带自己的编译文档和源快照，故此通道仅封装字节；调用方负责载荷语义与版本兼容性。
    pub fn store_payload_bytes(
        &self,
        key: &UiCompiledArtifactKey,
        payload_bytes: &[u8],
    ) -> io::Result<PathBuf> {
        let path = self.payload_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let record = UiCompiledPayloadDiskRecord {
            record_version: STORE_RECORD_VERSION,
            key: key.clone(),
            payload_bytes: payload_bytes.to_vec(),
        };
        let payload = bincode::serialize(&record).map_err(invalid_data)?;
        fs::write(&path, payload)?;
        Ok(path)
    }

    /// 只移除指定键的模板包；按资产回收全部版本以及不透明载荷应使用资产级淘汰入口。
    pub fn remove(&self, key: &UiCompiledArtifactKey) -> io::Result<bool> {
        match fs::remove_file(self.artifact_path(key)) {
            Ok(()) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// 资产失效时扫描所有 schema/编译器/输入版本，并按记录内的资产 ID 清理两种通道；不依赖文件名猜测身份。
    pub fn evict_asset(&self, asset_id: &str) -> io::Result<UiCompiledArtifactStoreEvictionReport> {
        let mut report = UiCompiledArtifactStoreEvictionReport::default();
        self.evict_asset_in_dir(&self.root, asset_id, &mut report)?;
        Ok(report)
    }

    // 无法读取或解码的缓存记录保留给后续诊断；只有能确认所属资产的文件进入删除统计。
    fn evict_asset_in_dir(
        &self,
        directory: &Path,
        asset_id: &str,
        report: &mut UiCompiledArtifactStoreEvictionReport,
    ) -> io::Result<()> {
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                self.evict_asset_in_dir(&path, asset_id, report)?;
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str())
                != Some(STORE_ARTIFACT_EXTENSION)
                && path.extension().and_then(|extension| extension.to_str())
                    != Some(STORE_PAYLOAD_EXTENSION)
            {
                continue;
            }
            let Ok(payload) = fs::read(&path) else {
                continue;
            };
            if !disk_payload_matches_asset_id(&payload, asset_id) {
                continue;
            }
            fs::remove_file(&path)?;
            report.files_removed += 1;
            report.bytes_removed += metadata.len();
        }
        Ok(())
    }

    fn payload_path(&self, key: &UiCompiledArtifactKey) -> PathBuf {
        self.path_for_key(key, STORE_PAYLOAD_EXTENSION)
    }

    // 可读文件名仅用于诊断；附加资产摘要区分经清理后重名的 ID，目录层级隔离输入和格式版本。
    fn path_for_key(&self, key: &UiCompiledArtifactKey, extension: &str) -> PathBuf {
        let asset_stem = sanitized_asset_file_stem(&key.asset_id);
        let asset_hash = UiAssetFingerprint::from_bytes(key.asset_id.as_bytes()).value;
        self.root
            .join(format!("schema-{:08x}", key.schema_version))
            .join(format!("compiler-{:08x}", key.compiler_version))
            .join(format!("{:016x}", key.fingerprint))
            .join(format!("{asset_stem}-{asset_hash:016x}.{extension}"))
    }
}

fn disk_payload_matches_asset_id(payload: &[u8], asset_id: &str) -> bool {
    bincode::deserialize::<UiCompiledArtifactDiskRecord>(payload)
        .map(|record| record.key.asset_id == asset_id)
        .unwrap_or_else(|_| {
            bincode::deserialize::<UiCompiledPayloadDiskRecord>(payload)
                .map(|record| record.key.asset_id == asset_id)
                .unwrap_or(false)
        })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct UiCompiledArtifactDiskRecord {
    record_version: u32,
    key: UiCompiledArtifactKey,
    artifact_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct UiCompiledPayloadDiskRecord {
    record_version: u32,
    key: UiCompiledArtifactKey,
    payload_bytes: Vec<u8>,
}

// 模板通道的兼容检查独立于 V2 载荷通道；后者允许自己的 schema，不能套用模板 envelope 版本。
fn artifact_matches_key(
    key: &UiCompiledArtifactKey,
    artifact: &UiRuntimeCompiledAssetArtifact,
) -> bool {
    let header = &artifact.report.header;
    key.schema_version == UI_COMPILED_ASSET_TOML_ENVELOPE_SCHEMA_VERSION
        && header.asset.id == key.asset_id
        && header.compiler_schema_version == key.compiler_version
        && header.package_schema_version == UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION
        && UiCompiledArtifactKey::fingerprint_compile_cache_key(&header.compile_cache_key)
            == key.fingerprint
}

fn sanitized_asset_file_stem(asset_id: &str) -> String {
    let mut stem = asset_id
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else if matches!(character, '.' | '_' | '-') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    if stem.is_empty() {
        stem.push_str("asset");
    }
    if stem.len() > MAX_ASSET_STEM_LEN {
        stem.truncate(MAX_ASSET_STEM_LEN);
    }
    stem
}

fn push_fingerprint(bytes: &mut Vec<u8>, fingerprint: UiAssetFingerprint) {
    push_u64(bytes, fingerprint.value);
}

fn push_fingerprint_map(
    bytes: &mut Vec<u8>,
    fingerprints: &std::collections::BTreeMap<String, UiAssetFingerprint>,
) {
    push_u64(bytes, fingerprints.len() as u64);
    for (reference, fingerprint) in fingerprints {
        push_str(bytes, reference);
        push_fingerprint(bytes, *fingerprint);
    }
}

fn push_str(bytes: &mut Vec<u8>, value: &str) {
    push_u64(bytes, value.len() as u64);
    bytes.extend_from_slice(value.as_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn invalid_data(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
