//! 将产品角色、运行配置和导出意图在启动前解析为不可变的宿主契约。
//! 调用端应先取得解析结果，再进行 Runtime 模块装配或动态库加载。

#[path = "entry_config.rs"]
mod entry_config;
#[path = "product_artifact_manifest.rs"]
mod product_artifact_manifest;
#[path = "product_config_source.rs"]
mod product_config_source;
#[path = "product_config_source_set.rs"]
mod product_config_source_set;
#[path = "product_host_capability_policy.rs"]
mod product_host_capability_policy;
#[path = "product_host_config_error.rs"]
mod product_host_config_error;
#[path = "product_host_config_provenance.rs"]
mod product_host_config_provenance;
#[path = "product_role_catalog.rs"]
mod product_role_catalog;
#[path = "product_role_descriptor.rs"]
mod product_role_descriptor;
#[path = "product_role_request.rs"]
mod product_role_request;
#[path = "request.rs"]
mod request;
#[path = "resolution.rs"]
mod resolution;
#[path = "resolved_product_host_config.rs"]
mod resolved_product_host_config;

pub use entry_config::EntryConfig;
pub use product_artifact_manifest::{
    ProductArtifactDeliveryStatus, ProductArtifactKind, ProductArtifactManifest,
};
pub use product_config_source::ProductConfigSource;
pub use product_config_source_set::ProductConfigSourceSet;
pub use product_host_capability_policy::{
    ProductCapabilityRequirement, ProductHostCapabilityPolicy, ProductPlatformClass,
};
pub use product_host_config_error::ProductHostConfigError;
pub use product_host_config_provenance::ProductHostConfigProvenance;
pub use product_role_descriptor::{
    ProductEntryKind, ProductRoleDescriptor, ProductRunnerKind, ProductRuntimeLinkage,
    ProductShutdownPolicy,
};
pub use product_role_request::ProductRoleRequest;
pub use resolved_product_host_config::ResolvedProductHostConfig;
