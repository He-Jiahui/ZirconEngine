---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: project-asset-manager-access-test-consumer-drift
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_runtime/frameworks/05
related_code:
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs
  - zircon_runtime/src/graphics/tests/
tests:
  - cargo test -p zircon_runtime --locked
resolved_at: 2026-07-14
---


# Frameworks05：ProjectAssetManagerAccess 硬切未覆盖 lib-test 消费方

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：M1 测试阶段 / default-feature runtime gate
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 交接原因：最低共同原因是 Frameworks05 M4 刚完成的 `Arc<ProjectAssetManager>` → `ProjectAssetManagerAccess` 硬切；Editor02 不拥有 graphics 构造器、resource streamer 或其测试夹具。

## 失败现象与复现证据

协调器管理的 Windows job `13130c87201b444db8bcb9654a7f118a` 执行：

```text
cargo test -p zircon_runtime --locked
```

lib-test 编译以 exit `101` 失败。完整日志：
`E:\ZirconBuilds\editor02-m1-runtime-default-after-generation-20260714.log`。
错误按根因聚类为：

- `174` 处 `E0308`：生产构造器已要求 `ProjectAssetManagerAccess`，测试仍传入 `Arc<ProjectAssetManager>`；
- `14` 处后继 `E0308`：`ScreenSpaceUiRenderer::new(...)` 已返回 `Result`，测试把未解包的 `Result` 传给 GPU context；
- `23` 处 `E0615` 与 `3` 处后继 `E0282`：`#[cfg(test)]` resource-streamer accessor 仍把 `asset_manager()` 当字段访问，没有处理其 typed `Result`。

同一日志另有一条 Plugins08 VM catalog 测试导入错误，已单独交接，不计入本 Failure。

## 最低共享层根因

Frameworks05 的生产 build 已验证新的 versioned access/resolve-at-use 边界，但 hard cut 只迁移了生产调用面，未把 lib-test 构造器与 `#[cfg(test)]` diagnostics accessors 纳入同一消费方矩阵。测试因此仍依赖已退役的裸 `Arc` 注入与字段式 manager 访问。

## 架构修复验收

- 所有 runtime lib-test 构造器通过真实 `ProjectAssetManagerAccess`/versioned handle 入口建立夹具，不恢复隐式 `From`/`Into` adapter。
- `ResourceStreamer` 的 test-only accessors 显式解析 `asset_manager()` 的 typed result，并保留失败语义，不使用字段兼容层或静默默认。
- `ScreenSpaceUiRenderer::new` 的测试调用按当前 typed `Result` 契约处理，不改变生产签名。
- 原 default-feature `cargo test -p zircon_runtime --locked` 至少越过全部 `ProjectAssetManagerAccess`、`asset_manager()` 与派生 `ScreenSpaceUiRenderer` 编译错误。

## 禁止临时方案

- 不恢复 `Arc<ProjectAssetManager>` 构造器重载、`IntoProjectAssetManagerAccess`、旧 resolver 文件或字段 façade。
- 不给 `ProjectAssetManagerAccess` 增加为测试服务的隐式转换。
- 不用 `unwrap_or_default`、空 diagnostics 或 cfg-only bypass 掩盖失效 handle。

## 修复结果与回传

- 根因：Frameworks05 production hard-cut initially omitted lib-test constructor and accessor consumers.
- 架构修复：Migrated tests to explicit real CoreRuntime-owned ProjectAssetManagerAccess fixtures and typed resolve-at-use paths without compatibility conversions.
- 验证：22/22 Frameworks05 static tests pass; managed runtime compile eliminated all ProjectAssetManagerAccess/asset_manager/ScreenSpaceUiRenderer consumer errors and now stops only on separately owned Shader06/Render18 compile drift.
- 回传：Frameworks05 owner scope fixed and returned to Editor02; no compatibility export or Arc adapter restored.
