---
related_code:
  - zircon_runtime/src/scene/dynamic_scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/world_operations.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction.rs
  - zircon_runtime/src/scene/dynamic_scene/document/schema.rs
  - zircon_runtime/src/scene/dynamic_scene/session/archive.rs
implementation_files:
  - zircon_runtime/src/scene/dynamic_scene
plan_sources:
  - user: 2026-09-09 动态场景与序列化说明
tests:
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests.rs
  - zircon_runtime/src/scene/dynamic_scene/session
doc_type: module-detail
---

# 动态场景、补丁与会话归档

## 用途与模型

`DynamicScene` 是由反射组件、资源和实体记录组成的可移植快照。`DynamicEntity`、`DynamicComponent`、`DynamicResource` 保存序列化值；`ScenePatch` 描述差异。实体 ID 冲突由 `EntityRemap` 统一映射，父级路径和组件引用在映射后保持一致。

## 捕获与实例化流程

`DynamicScene::from_world` 捕获可序列化组件和资源；`ensure_supported` 在执行前拒绝未支持类型。`preview_spawn_into` 只编译计划并生成 `ScenePatchPreviewReport`。`spawn_into` 依次执行：编译（解析类型和引用）、generation/schema/component-registry 快照、隔离 World 预检、反射写入和资源写入，最后一次性提交。目标 World 在期间改变会返回 `TargetWorldChanged` 等错误，避免部分提交。

```rust
use zircon_runtime::scene::{DynamicScene, World};
let source = DynamicScene::from_world(&world)?;
source.ensure_supported()?;
let preview = source.preview_spawn_into(&world)?;
println!("entities: {}", preview.entity_count);
let remap = source.spawn_into(&mut world)?;
```

## 文档和会话

`document::schema` 定义带版本的 JSON wire schema；读取/写入阶段执行未知字段拒绝、迁移和验证。`RuntimeSessionArchive` 以 `RuntimeSessionSlot` 保存运行时捕获，维护 generation、revision、标签和更新时间二级索引；`RuntimeSessionArchiveReader/Writer` 提供受限异步读写，`RuntimeSessionRetentionPolicy` 控制裁剪。

## 热重载数据流

`DynamicSceneAssetReloadQueue` 接收资产变更，按任务序列和 generation 排序；`drain` 在帧边界应用成功结果，旧结果标记 stale/superseded，失败转为 `DynamicSceneAssetReloadApplyFailure`，并由 limits 限制字节、任务和处理时间。

## 错误与限制/状态

反射类型描述冲突、资源缺失、schema 迁移失败、目标 generation/change tick 改变都会中止事务；预检大小受 `limit_bytes` 限制。归档格式当前 `RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION = 1`，最大 artifact 大小由 `MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES` 约束。动态场景、预检提交、归档和重载队列均已实现并有专项测试。

## 源码与测试

[dynamic_scene/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/dynamic_scene/mod.rs)、[world_operations.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/dynamic_scene/scene/world_operations.rs)、[transaction.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction.rs)、[archive.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/dynamic_scene/session/archive.rs)。
