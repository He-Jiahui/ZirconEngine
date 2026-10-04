//! 源文档迁移与原型直载共用平面表表示，分别产出作者树和句柄节点表。
//! 常规加载由迁移器给出版本与结构诊断；原型直载保留源头部供后续存储使用。

mod flat_nodes;
mod migrator;

pub(crate) use flat_nodes::load_flat_prototype_toml_str;
pub use migrator::UiAssetSchemaMigrator;
