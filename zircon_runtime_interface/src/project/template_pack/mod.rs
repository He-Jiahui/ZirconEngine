//! 编译期封装的项目模板注册表、内容身份、创建来源和渲染契约。
//! 渲染结果拥有安全相对路径与字节，供 Editor 创建事务消费，不依赖运行时源码目录。

#[path = "content_digest.rs"]
mod content_digest;
#[path = "descriptor.rs"]
mod descriptor;
#[path = "embedded.rs"]
mod embedded;
#[path = "error.rs"]
mod error;
#[path = "project_template_id.rs"]
mod project_template_id;
#[path = "receipt.rs"]
mod receipt;
#[path = "render.rs"]
mod render;
#[path = "rendered_entry.rs"]
mod rendered_entry;
#[path = "rendered_template.rs"]
mod rendered_template;

pub use content_digest::{ProjectTemplateContentDigest, ProjectTemplateContentDigestParseError};
pub use descriptor::{
    project_template_descriptor, ProjectTemplateCapability, ProjectTemplateDescriptor,
    ProjectTemplateEntryDescriptor, ProjectTemplateTargetRequirement,
};
pub use error::ProjectTemplatePackError;
pub use project_template_id::ProjectTemplateId;
pub use receipt::{
    ProjectCreationProvenance, ProjectTemplateReceipt, ProjectTemplateReceiptError,
    PROJECT_TEMPLATE_RECEIPT_SCHEMA_VERSION_V1,
};
pub use render::render_project_template;
pub use rendered_entry::RenderedProjectTemplateEntry;
pub use rendered_template::RenderedProjectTemplate;
