---
related_code:
  - zircon_runtime/src/scene/reflect/mod.rs
  - zircon_runtime/src/scene/reflect/conversion.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/mod.rs
  - zircon_runtime/src/scene/tests/authoring_boundary.rs
  - zircon_editor/src/ui/preferences.rs
  - zircon_editor/src/ui/asset_editor/undo_stack.rs
  - zircon_editor/src/ui/workbench/layout/manager/persistence.rs
  - zircon_runtime/src/asset/project/manifest.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime_interface/src/manifest.rs
reference_sources:
  - dev/bevy/crates/bevy_asset/src/meta.rs
  - dev/bevy/crates/bevy_reflect/src/lib.rs
  - dev/godot/editor/editor_file_system.h
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_runtime/runtime/13-script-binding-and-reflection.md
status: planned
---

# 11 数据序列化与版本迁移

横切基座（W1）：为所有持久化面提供统一版本壳与迁移链。

## 参照证据（dev/）

**bevy 显式格式版本**（`bevy_asset/src/meta.rs:27-37`）：`AssetMeta { meta_format_version: String, processed_info, asset }`——版本字段是载荷第一公民；`ProcessedInfo` 指纹（源 hash+处理器版本）判定产物过期，非时间戳。

**bevy_reflect**（`bevy_reflect/src/lib.rs`）：`TypeRegistry` 驱动的反射序列化——序列化器按 TypeInfo 走不认识具体类型。zircon 场景序列化已同思路（reflect 中转），本计划只在外面包版本壳，不重做反射。

**godot 文本纪律**：创作态文本可 diff、产物二进制、指纹关联——「文本创作/二进制交付」双轨先例。

## 现状与证据（zircon，2026-07-05 实读）

### 序列化面清单（版本化程度总表，逐面接入的执行合同）

| 面 | 现状 | 版本机制 | 目标 |
| --- | --- | --- | --- |
| `DynamicScene`（`dynamic_scene/scene/mod.rs:13-17`） | `DYNAMIC_SCENE_FORMAT_VERSION=1` 内嵌字段 + **`ensure_supported()` 版本检查钩子已有**（:51）；`from_world/spawn_into/preview_spawn_into(EntityRemap)` | 内嵌 u32 | 收编入壳（钩子改查壳头） |
| 场景反射 JSON（`reflect/conversion.rs:65-69`） | `reflected_from_json(Value)->ReflectedValue::Json` 四入口（`reflect/mod.rs:12-15`）**无 schema 头** | 无 | v0→v1 带头（与 10 AssetRef 同步） |
| `EditorAppearancePreferences`（`preferences.rs:12-13`） | `APPEARANCE_PREFERENCES_VERSION=1` 常量 + 四方法 | 常量比对 | 收编（17 settings 迁移时） |
| `AssetMetaDocument`（`project/meta.rs`，09 核） | uuid/url/kind sidecar，load/save | 无 | 接壳（09 M2 加字段时同步） |
| `ProjectManifest`（`manifest.rs:14-33`，10 核） | `format_version=1` serde default + `library_version(alias schema_version)` | 内嵌 u32 | 收编（10 M1 v2 迁移步即首个迁移链实例） |
| `UiAssetEditorUndoStack.replay_records` | serde 可序列化 | 无 | 03 journal 化时出生带壳 |
| workbench 布局（`persistence.rs:6-27`） | 无 IO 空实现 | — | 06 M3 实 IO 出生带壳 |
| ABI 清单（`interface/manifest.rs:19-42`） | `abi_version` + size_bytes 自校验 | ABI 纪律 | **不动**（非目标） |
| 03 journal / 08 keymap 用户层 / 09 zmeta 扩展 / 15 preset / 16 hub 信箱 / 17 settings | 未出生 | — | 出生即带壳 |

三种版本策略并存（内嵌 u32 / 常量比对 / ABI size），**无迁移管线**——旧版本=失败或静默容错。

**守卫资产**：`SERIALIZED_AUTHORING_TOKENS`（`authoring_boundary.rs:3-13`）+ `assert_sorted_and_deduplicated`（:73）——runtime 序列化纯净性守卫在案，本计划全程继承，新持久化面纳入覆盖矩阵。

## 目标

1. **统一版本壳**（`zircon_runtime_interface/src/serialization/`）：

```rust
pub struct PayloadHeader { pub schema_id: SchemaId, pub schema_version: u32 }
pub trait VersionedSchema: Sized {
    const SCHEMA: SchemaId; const VERSION: u32;
    fn migrations() -> &'static MigrationChain<Self>;
}
// MigrationStep: from_version + fn(serde_json::Value) -> Result<Value, MigrateError>
// 值域逐步迁移，最后一步反序列化为 T —— 旧版本结构体不留 Rust 定义（防类型化石）
pub fn load_versioned<T: VersionedSchema>(bytes: &[u8], fmt: Format) -> Result<Loaded<T>, LoadError>;
pub struct Loaded<T> { pub value: T, pub migrated_from: Option<u32> }   // 17 通知事件源
```

2. **逐面接入**：按上表推进；`DynamicScene.format_version` 与 `ensure_supported` 收编（双写一版→删内嵌字段）；场景反射 JSON v0→v1（与 10 `AssetRef` 切换同一迁移步，避免两次断代）。
3. **迁移纪律**：每步带往返测试 + 真实旧样本夹具（`tests/fixtures/serialization/<schema>/v<N>/`）；写盘一律最新版；`migrated_from` 触发一次性通知与重存提示。
4. **canonical 文本形式**：BTreeMap 键序（已普遍）、浮点最短往返、数组换行策略——同内容同字节；`Format::{Text, Binary}` 双格式同 schema，cook（15）产二进制。
5. **守卫扩展**：新持久化面登记 authoring token 覆盖矩阵；壳头字段入禁入词复核（`schema_id` 中性词豁免记录）。

## 非目标

- 不重做反射（runtime/13；值域迁移正为不依赖其进度）；不定 pak 容器（15）；不处理外部格式版本（importer 域）；ABI size 校验不动。

## 架构设计

- 归属：壳与链在 `zircon_runtime_interface/src/serialization/`（headless 工具、hub、插件 SDK 均可依赖）；各面 schema 声明留各 owner。
- 迁移链注册：**显式常量表**（`migrations()` 返回静态链），不用 inventory/linkme——可 grep 性优先。
- `Format::Binary`：定长头 + postcard/bincode 类紧凑编码，M3 以夹具数据实测选型记状态节。
- 深度测试：新面接入=实现 `VersionedSchema` 三项 + 样本夹具目录，`serialization/` 零改动。首个真实迁移实例=10 M1 的 manifest v1→v2（本计划 M1 与其联合验收）。

## 里程碑

### M1 壳与场景面接入

- 切片 1.1：`serialization/` 落地（header/chain/load_versioned/双格式入口）+ 单测（断链/越版本/v0 识别）。
- 切片 1.2：场景反射 JSON 接壳（v0→v1 = 包头 + AssetRef 切换，与 10 M2 同步发布）；canonical 文本 writer；`ensure_supported` 改查壳头。
- 测试阶段：`cargo test -p zircon_runtime_interface --locked` + `cargo test -p zircon_runtime --lib scene:: --locked`；验收：v0 旧样本 加载→迁移→保存→再加载 幂等且字节稳定；authoring 守卫矩阵更新后全绿。更新 `docs/zircon_runtime/scene/serialization.md`。

### M2 编辑器状态面收口

- 切片 2.1：preferences（常量删除）/ keymap 用户层（08）/ 03 journal / 06 布局实 IO——四面接壳；路径按 17 分层。
- 切片 2.2：`DynamicScene.format_version` 收编（双写一版→删除，迁移步内完成硬切换）。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（preferences 既有测试迁移后须过 + 各面往返/旧格式迁移）；`migrated_from` 通知断言。

### M3 二进制格式与等价性

- 切片 3.1：Binary 编码选型实测 + 实现；文本↔二进制互转往返。
- 切片 3.2：15 cook 消费点接线（CookAssets 阶段调用双格式转换）。
- 测试阶段：Text→Binary→Text 恒等（逐 schema）；5k 实体夹具体积/耗时基线记状态节。

## 风险与开放问题

- 非 JSON 面（TOML preferences）值域归一有损点（日期/整数上限）：preferences 无此类字段，接受归一；未来有损面允许自持迁移函数绕值域路径，壳只强制 header。
- canonical 浮点 writer 只覆盖创作态文本面；维护负担超预期则退让「键序稳定+浮点原样」记 diff 噪声债。
- 迁移链堆积：允许「基线重置」（v1..vN 合并新 v1 + 全量重存 commandlet），公约写入模块文档。
- `library_version` 与壳 version 的双版本语义（10 已识别）：壳只管结构版本，内容版本是业务字段——模块文档显式区分。
