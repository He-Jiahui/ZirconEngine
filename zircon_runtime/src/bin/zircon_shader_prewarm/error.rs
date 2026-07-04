use thiserror::Error;

pub type ShaderPrewarmArgsResult<T> = std::result::Result<T, ShaderPrewarmArgsError>;
pub type ShaderPrewarmAssetScanResult<T> = std::result::Result<T, ShaderPrewarmAssetScanError>;
pub type ShaderPrewarmManifestResult<T> = std::result::Result<T, ShaderPrewarmManifestError>;
pub type ShaderPrewarmPermutationRegistryResult<T> =
    std::result::Result<T, ShaderPrewarmPermutationRegistryError>;
pub type ShaderPrewarmReportResult<T> = std::result::Result<T, ShaderPrewarmReportError>;
pub type ShaderPrewarmResourceRegistryResult<T> =
    std::result::Result<T, ShaderPrewarmResourceRegistryError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ShaderPrewarmArgsError {
    #[error("{0}")]
    Usage(String),
}

#[derive(Debug, Error)]
pub enum ShaderPrewarmAssetScanError {
    #[error("failed to read shader prewarm asset root {path:?}: {source}")]
    ReadAssetRoot {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read shader prewarm asset root {path:?} entry: {source}")]
    ReadAssetRootEntry {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to load shader asset metadata {path:?}: {source}")]
    LoadShaderMetadata {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read zshader {path:?}: {source}")]
    ReadZShader {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse zshader {path:?}: {source}")]
    ParseZShader {
        path: std::path::PathBuf,
        #[source]
        source: zircon_runtime::asset::ZShaderV2Error,
    },
    #[error("failed to read WGSL {path:?}: {source}")]
    ReadWgsl {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read zmaterial {path:?}: {source}")]
    ReadZMaterial {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse zmaterial {path:?}: {source}")]
    ParseZMaterial {
        path: std::path::PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("shader source {path:?} has no runtime WGSL payload")]
    EmptyShaderSource { path: std::path::PathBuf },
    #[error("failed to read shader package {path:?}: {source}")]
    ReadShaderPackage {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read shader package {path:?} entry: {source}")]
    ReadShaderPackageEntry {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("shader source {source_path:?} is outside package dir {package_dir:?}: {source}")]
    ShaderSourceOutsidePackageDir {
        source_path: std::path::PathBuf,
        package_dir: std::path::PathBuf,
        #[source]
        source: std::path::StripPrefixError,
    },
}

#[derive(Debug, Error)]
pub enum ShaderPrewarmManifestError {
    #[error("failed to read shader prewarm manifest {path:?}: {source}")]
    Read {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse shader prewarm manifest {path:?}: {source}")]
    Parse {
        path: std::path::PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("shader prewarm manifest schema {actual} is not supported; expected {expected}")]
    UnsupportedSchema { actual: u32, expected: u32 },
}

#[derive(Debug, Error)]
pub enum ShaderPrewarmPermutationRegistryError {
    #[error("failed to read shader prewarm permutation registry {path:?}: {source}")]
    Read {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse shader prewarm permutation registry {path:?}: {source}")]
    Parse {
        path: std::path::PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("{0}")]
    InvalidToken(#[from] ShaderPrewarmArgsError),
    #[error(
        "shader prewarm permutation registry {path:?} assigns geometry source id {id}; plugin geometry source ids must be >= {minimum}"
    )]
    GeometrySourceIdBelowPluginRange {
        path: std::path::PathBuf,
        id: u8,
        minimum: u8,
    },
    #[error(
        "shader prewarm permutation registry {path:?} assigns shading model id {id}; plugin shading model ids must be >= {minimum}"
    )]
    ShadingModelIdBelowPluginRange {
        path: std::path::PathBuf,
        id: u8,
        minimum: u8,
    },
    #[error("custom geometry source {token} was assigned both id {existing_id} and id {new_id}")]
    DuplicateGeometrySourceToken {
        token: String,
        existing_id: u8,
        new_id: u8,
    },
    #[error(
        "custom geometry source id {id} is already assigned to {existing_token} and cannot be reused by {new_token}"
    )]
    DuplicateGeometrySourceId {
        id: u8,
        existing_token: String,
        new_token: String,
    },
    #[error("custom shading model {token} was assigned both id {existing_id} and id {new_id}")]
    DuplicateShadingModelToken {
        token: String,
        existing_id: u8,
        new_id: u8,
    },
    #[error(
        "custom shading model id {id} is already assigned to {existing_token} and cannot be reused by {new_token}"
    )]
    DuplicateShadingModelId {
        id: u8,
        existing_token: String,
        new_token: String,
    },
    #[error(
        "custom geometry source descriptor id {id} was registered with incompatible descriptors"
    )]
    IncompatibleGeometrySourceDescriptor { id: u8 },
    #[error(
        "custom shading model descriptor id {id} was registered with incompatible descriptors"
    )]
    IncompatibleShadingModelDescriptor { id: u8 },
    #[error(
        "shader module {import_path} was registered with content hash {existing_content_hash} and {new_content_hash}"
    )]
    DuplicateShaderModuleContentHash {
        import_path: String,
        existing_content_hash: String,
        new_content_hash: String,
    },
}

#[derive(Debug, Error)]
pub enum ShaderPrewarmReportError {
    #[error("failed to encode shader prewarm report: {source}")]
    ReportEncode {
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to create shader prewarm report directory {path:?}: {source}")]
    CreateReportDirectory {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write shader prewarm report {path:?}: {source}")]
    WriteReport {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Error)]
pub enum ShaderPrewarmResourceRegistryError {
    #[error("failed to read shader prewarm resource registry {path:?}: {source}")]
    Read {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse shader prewarm resource registry {path:?}: {source}")]
    Parse {
        path: std::path::PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error(
        "shader prewarm resource registry {path:?} must be a ResourceRecord array or contain a resources/records array"
    )]
    MissingRecordsArray { path: std::path::PathBuf },
    #[error("failed to decode shader prewarm resource records {path:?}: {source}")]
    DecodeRecords {
        path: std::path::PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to read shader resource registry root {path:?}: {source}")]
    ReadRoot {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to read shader resource registry root {path:?} entry: {source}")]
    ReadRootEntry {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to load shader resource registry metadata {path:?}: {source}")]
    LoadMetadata {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "shader resource registry record id {id:?} maps both {existing_locator} and {new_locator}"
    )]
    DuplicateRecordId {
        id: zircon_runtime::core::resource::ResourceId,
        existing_locator: zircon_runtime::core::resource::ResourceLocator,
        new_locator: zircon_runtime::core::resource::ResourceLocator,
    },
    #[error("shader resource registry locator {locator} maps both {existing_id:?} and {new_id:?}")]
    DuplicateLocator {
        locator: zircon_runtime::core::resource::ResourceLocator,
        existing_id: zircon_runtime::core::resource::ResourceId,
        new_id: zircon_runtime::core::resource::ResourceId,
    },
    #[error("failed to encode shader resource registry: {source}")]
    EncodeExport {
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to create shader resource registry directory {path:?}: {source}")]
    CreateExportDirectory {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write shader resource registry {path:?}: {source}")]
    WriteExport {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}
