---
related_code:
  - zircon_runtime/src/ui/v2/mod.rs
  - zircon_runtime/src/ui/v2/compiler.rs
  - zircon_runtime/src/ui/v2/loader.rs
  - zircon_runtime/src/ui/v2/cache.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/v2/surface_builder.rs
implementation_files:
  - zircon_runtime/src/ui/v2/surface_tree/mod.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine UI Wiki
tests:
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_runtime/src/ui/tests/v2_asset/asset_loading.rs
  - zircon_runtime/src/ui/tests/v2_asset/style_runtime.rs
doc_type: module-detail
---

# v2 资产管线

## 资产与编译

`UiV2AssetLoader`/`UiZuiAssetLoader` 将资产文件解码为 `UiV2AssetDocument`。`UiV2DocumentCompiler` 把节点展平到 `UiV2NodeArena`，解析组件图、slot、repeat 和导入。`UiV2PrototypeStore` 在会话级缓存导入组件，`UiV2PrototypeStoreFileCache` 提供跨加载持久化和 canonical revalidation。

## 样式解析

`UiV2StyleResolver` 首先计算静态规则，匹配 selector 与 style token；`UiV2RuntimeStyleIndex` 保留运行时伪状态和属性覆盖。主题可通过 `UiV2SurfaceBuilder::build_surface_from_compiled_document_with_theme` 注入。

## 建立表面

```rust
use zircon_runtime::ui::v2::{UiV2SurfaceBuilder, UiV2PrototypeStoreBuilder};
let mut store_builder = UiV2PrototypeStoreBuilder::default();
for document in documents {
    store_builder.insert(document);
}
let store = store_builder.build()?;
let surface = UiV2SurfaceBuilder::build_surface_with_prototype_store(tree_id, &doc, &store)?;
```

文本测量缓存绑定共享字体集合；带 `TextRuntimeContext` 的内部构造可保证字体 revision 一致。构建失败返回 `UiV2AssetError`，包括导入、schema、样式和节点约束诊断。

## 运行时限制

每个导入文档应在会话创建时放入 prototype store，避免每帧重复解析。静态样式解析与运行时状态解析分离；修改属性后必须通过 invalidation/dirty domain 触发布局或渲染更新。
