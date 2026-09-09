---
related_code:
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/project/manifest/project_manifest.rs
  - zircon_runtime/src/asset/project/manager/mod.rs
  - zircon_runtime/src/asset/pipeline/manager/mod.rs
  - zircon_runtime/src/asset/importer/mod.rs
  - zircon_runtime/src/asset/watch/asset_watcher.rs
implementation_files:
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/asset/pipeline/manager
plan_sources:
  - user: 2026-09-09 项目资产管线说明
tests:
  - zircon_runtime/src/asset/tests/project
  - zircon_runtime/src/asset/tests/pipeline
  - zircon_runtime/src/asset/tests/facade
doc_type: module-detail
---

# 项目资产与导入管线

## 项目清单

`ProjectManifest` 是项目根文档：`name`、`project_guid`、`default_scene`、`asset_roots`、`library_version`，以及可选 settings、插件、脚本和 export profiles。`ProjectManifest::new` 生成当前格式版本和 GUID；`summary` 输出跨进程摘要。`ProjectPaths` 负责根目录、源资产和 `.zmeta` 路径的规范化。

## 导入与生成

`ProjectManager` 扫描 asset roots，读取/创建 `AssetMetaDocument`，通过 `AssetImporterRegistry` 按扩展名和优先级选择 `AssetImporter`，生成 `ImportedAssetEntry` 与二进制 artifact。`ProjectImportReceipt` 记录本次 generation、导入和失败项。`AssetManager`/`ProjectAssetManager` 对运行时提供状态记录、生成 token、依赖图和资源租约。

```rust
use zircon_runtime::asset::project::ProjectManager;
let mut project = ProjectManager::open("./DemoProject")?;
let imported = project.scan_and_import()?;
println!("imported {} resources", imported.len());
```

## 资产类型与 URI

内置资产模型包括 `SceneAsset`、`ModelAsset`、`MeshAsset`、`MaterialAsset`、`TextureAsset`、`ShaderAsset`、`PrefabAsset`、动画、字体、地形、UI 等。运行时 URI 通过 `runtime_asset_path`、`AssetUri`（`ResourceLocator` 别名）解析；源文件与生成 artifact 由 `.zmeta` 中稳定 `AssetUuid` 关联。

## 监听、迁移与打包

`AssetWatcher` 折叠文件系统事件为 `AssetWatchBatch`，忽略 `.meta` 侧车回声并提交增量 generation。schema migrator 负责旧资产格式升级；`asset::pack` 生成/读取基础包和 delta 包。删除、重定位操作先执行 mutation preflight，再由 ProjectManager 持久化。

## 错误与限制/状态

路径越界、重复 UUID、导入器冲突、依赖缺失和迁移失败均以专用错误返回，不应直接修改 `.zmeta`。热重载只有在 generation 与目标资源匹配时才发布；未就绪依赖会保留 `AssetLoadState`。项目导入、监听、缓存、迁移和打包均已实现，具体格式能力取决于注册的 importer。

## 源码与测试

[manifest](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/asset/project/manifest/project_manifest.rs)、[pipeline manager](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/asset/pipeline/manager/mod.rs)、[importer](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/asset/importer/mod.rs)、[watcher](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/asset/watch/asset_watcher.rs)；测试见 [project](https://github.com/He-Jiahui/ZirconEngine/tree/main/zircon_runtime/src/asset/tests/project) 与 [pipeline](https://github.com/He-Jiahui/ZirconEngine/tree/main/zircon_runtime/src/asset/tests/pipeline)。
