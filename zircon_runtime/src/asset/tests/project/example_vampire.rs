//! 复用仓库中的 Vampire 示例项目，分别检查脚本与资产可导入性以及场景到渲染帧的提取契约。

use std::path::{Path, PathBuf};

#[cfg(all(feature = "graphics", feature = "script"))]
mod manifest_scene_imports;
#[cfg(feature = "graphics")]
mod third_person_render_extract;

// TODO: [CR-ASSET-TEST-PROJECT-0002] 两项测试共用仓库示例根并分别执行完整扫描；需确认并发写入与示例源文件的隔离保证，下一步用并行测试及扫描前后文件摘要验证。
fn vampire_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("examples")
        .join("vampire")
}
