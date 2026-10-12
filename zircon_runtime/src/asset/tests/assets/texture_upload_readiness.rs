//! 纹理上传测试按容器与边界拆分，共用人工头部 fixture；目标是守住导入元数据、设备能力和上传计划之间的契约。

#[path = "texture_upload_readiness/boundaries.rs"]
mod boundaries;
#[path = "../texture_upload_readiness/container_fixtures.rs"]
mod container_fixtures;
#[path = "texture_upload_readiness/containers.rs"]
mod containers;
#[path = "texture_upload_readiness/dds.rs"]
mod dds;
#[path = "texture_upload_readiness/ktx.rs"]
mod ktx;
