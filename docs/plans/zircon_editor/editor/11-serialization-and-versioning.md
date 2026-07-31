---
related_code:
  - zircon_runtime_interface/src/serialization
  - zircon_runtime/src/scene/reflect/mod.rs
  - zircon_runtime/src/scene/reflect/conversion.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/mod.rs
  - zircon_runtime/src/scene/tests/authoring_boundary.rs
  - zircon_editor/src/ui/preferences/persistence.rs
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
status: in_progress
---

# 11 数据序列化与版本迁移

横切基座（W1）：为所有持久化面提供统一版本壳与迁移链。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-editor-serialization-and-versioning",
  "goal": "完成传输中立的统一版本壳、显式迁移链、编辑器持久化面收口与永久二进制 wire 架构",
  "milestones": [
    {"id": "M1", "title": "统一版本壳与场景面接入", "depends_on": []},
    {"id": "M2", "title": "编辑器状态面收口", "depends_on": ["M1"]},
    {"id": "M3", "title": "二进制格式与等价性", "depends_on": ["M1"]}
  ]
}
```

- fixed 已修复：[wgpu-hal-windows-version-split](11/fixed-2026-07-12-wgpu-hal-windows-version-split.md)
- fixed 已修复：[target-server-libtest-feature-gating](11/fixed-2026-07-11-target-server-libtest-feature-gating.md)
- fixed 已修复：[wsl-vhdx-sharing-violation](11/fixed-2026-07-11-wsl-vhdx-sharing-violation.md)
- M2.2 产出：[DynamicScene 版本壳硬切换](11/2026-07-14-dynamic-scene-version-shell-hard-cut.md)
- M2.2 当前源验收：[DynamicScene 当前源验收](11/2026-07-17-dynamic-scene-current-source-acceptance.md)
- M3.1 当前源验收：[二进制 wire 当前源验收](11/2026-07-17-binary-wire-current-source-acceptance.md)
- fixed 已修复：[binary-value-visibility-compilation](../editor_layout/15/fixed-2026-07-14-binary-value-visibility-compilation.md)

## 参照证据（dev/）

**bevy 显式格式版本**（`bevy_asset/src/meta.rs:27-37`）：`AssetMeta { meta_format_version: String, processed_info, asset }`——版本字段是载荷第一公民；`ProcessedInfo` 指纹（源 hash+处理器版本）判定产物过期，非时间戳。

**bevy_reflect**（`bevy_reflect/src/lib.rs`）：`TypeRegistry` 驱动的反射序列化——序列化器按 TypeInfo 走不认识具体类型。zircon 场景序列化已同思路（reflect 中转），本计划只在外面包版本壳，不重做反射。

**godot 文本纪律**：创作态文本可 diff、产物二进制、指纹关联——「文本创作/二进制交付」双轨先例。

## 现状与证据（zircon，2026-07-05 实读）

### 序列化面清单（版本化程度总表，逐面接入的执行合同）

| 面 | 现状 | 版本机制 | 目标 |
| --- | --- | --- | --- |
| `DynamicScene`（`dynamic_scene/scene/mod.rs`） | Plan 11 M2.2 已删除内嵌版本字段与常量；`ensure_supported()` 只校验私有 payload header；`from_world/spawn_into/preview_spawn_into(EntityRemap)` 保持 | `$zircon.header.schema_version=2` | 已收编入壳；v0→v1 保留历史形状，v1→v2 迁移步删除旧字段 |
| 场景反射 JSON（`reflect/conversion.rs:65-69`） | `reflected_from_json(Value)->ReflectedValue::Json` 四入口（`reflect/mod.rs:12-15`）**无 schema 头** | 无 | v0→v1 带头（与 10 AssetRef 同步） |
| `EditorAppearancePreferences`（`ui/preferences/`） | `$zircon` schema `zircon.editor.appearance-preferences` v1；payload 不再内嵌版本；旧 v1/v2 TOML 仅作为无壳 v0 迁移输入 | 统一版本壳 + `migrated_from` | preferences 子切片已接壳；keymap/journal/layout 仍待完成 |
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
- 深度测试：新面接入=实现 `VersionedSchema` 三项 + 样本夹具目录，`serialization/` 零改动。10 M1 的 manifest v1→v2 是计划中的首个业务实例；本计划 M2.2 同时以 DynamicScene v1→v2 验证删除双写字段的正式迁移路径。

## 里程碑

### M1 壳与场景面接入

- [x] **M1.1 统一版本壳与迁移链内核.** `serialization/` 落地（header/chain/load_versioned/双格式入口）+ 单测（断链/越版本/v0 识别）。
- [x] **M1.2 场景版本壳、值域迁移与 canonical writer.** 场景反射 JSON 接壳（v0→v1 = 包头 + AssetRef 切换，与 10 M2 同步发布）；canonical 文本 writer；`ensure_supported` 改查壳头。
- 测试阶段：`cargo test -p zircon_runtime_interface --locked` + `cargo test -p zircon_runtime --lib scene:: --locked`；验收：v0 旧样本 加载→迁移→保存→再加载 幂等且字节稳定；authoring 守卫矩阵更新后全绿。更新 `docs/zircon_runtime/scene/serialization.md`。

### M2 编辑器状态面收口

- [ ] **M2.1 编辑器状态持久化面接壳.** preferences（常量删除）/ keymap 用户层（08）/ 03 journal / 06 布局实 IO——四面接壳；路径按 17 分层。preferences 子切片已实现，见 [2026-07-18 appearance preferences version shell](11/2026-07-18-appearance-preferences-version-shell.md)；其余三面保持开放，因此本项不勾选。
- [x] **M2.2 DynamicScene 双写字段硬删除.** `DynamicScene.format_version` 收编（双写一版→删除，迁移步内完成硬切换）。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（preferences 既有测试迁移后须过 + 各面往返/旧格式迁移）；`migrated_from` 通知断言。

### M3 二进制格式与等价性

- [x] **M3.1 Binary 编码选型与文本等价合同.** Binary 编码选型实测 + 实现；文本↔二进制互转往返。
- [ ] **M3.2 CookAssets 二进制消费点接线.** 15 cook 消费点接线（CookAssets 阶段调用双格式转换）。
- 测试阶段：Text→Binary→Text 恒等（逐 schema）；5k 实体夹具体积/耗时基线记状态节。

## 风险与开放问题

- 非 JSON 面（TOML preferences）值域归一有损点（日期/整数上限）：preferences 无此类字段，接受归一；未来有损面允许自持迁移函数绕值域路径，壳只强制 header。
- canonical 浮点 writer 只覆盖创作态文本面；维护负担超预期则退让「键序稳定+浮点原样」记 diff 噪声债。
- 迁移链堆积：允许「基线重置」（v1..vN 合并新 v1 + 全量重存 commandlet），公约写入模块文档。
- `library_version` 与壳 version 的双版本语义（10 已识别）：壳只管结构版本，内容版本是业务字段——模块文档显式区分。

## 产出记录与时间

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 | 1.1 统一版本壳与迁移链内核 | `实现完成-接口门禁通过-独立审查闭环` | 2026-07-11 | `zircon_runtime_interface/src/serialization/` 已按 schema id/header/trait/loaded、text wire、migration step/chain/error/validate/execute、load/format/typed error 拆为 folder-backed owner；`SchemaId` 支持 const 声明与 owned 反序列化。Text envelope 只由保留 `$zircon` magic 识别，无包头 v0 可合法拥有 `header`/`payload` 业务字段；整张迁移表在任何 payload decode 前强制等于升序唯一 `0..VERSION-1`，断链、重复、乱序、多余 step 与 step failure 均返回带 schema/version/source 的 typed error。`Format::Binary` 在 M3 选型前显式 `UnsupportedFormat`，不落临时 wire/兼容 reader。TDD 首轮 RED 为八个合同符号 E0432；独立审查两轮指出并闭环 magic 歧义、整链校验、owner 拆分、分阶段错误和上下文测试。最终 focused 17/17；完整 `cargo test -p zircon_runtime_interface --locked --offline -- --test-threads=1` 为 203/203，doc-tests 0 项。模块文档：`docs/zircon_runtime_interface/serialization.md`。M1.2 场景反射/canonical writer 尚未开始，本计划保持 in_progress。 |
| M1 | 1.2 场景版本壳、值域迁移与 canonical writer | `实现完成-规格质量复审通过-聚焦门通过-完整门待下层修复` | 2026-07-11 | 场景反射 JSON 与 `DynamicScene` 已接入 `$zircon` 版本壳；v0→v1 只按精确 `{uuid,url}` 形状迁移为 Plan10 `AssetRef`，额外字段、单字段和当前 `AssetRef` 均保持原值，任意 tagged JSON 不再猜测为 typed `ReflectedValue`。旧 `V1ProjectDocument` DTO/模块已硬删除，project-world 迁移只操作 `serde_json::Value`；`RuntimeSessionArchive` 内嵌场景信封在 payload decode 前验证 future header，writer 强制壳头与仍处于 M2.2 双写期的内嵌 `format_version=1` 一致。真实旧 writer 夹具 `tests/fixtures/serialization/scene-dynamic/v0/dynamic-scene.json` 无壳且保留历史内嵌版本 1，迁移→保存→重载同字节；独立 `plan11_scene_serialization_contract` 5/5。target-server production lib check exit 0；规格复审 `APPROVED`、质量复审整改真实夹具/单一测试 owner/迁移负例/API 文档后 `QUALITY APPROVED`，限定 rustfmt 与 `git diff --check` 通过。完整默认 Runtime 门仍由 Runtime01 `wgpu-hal` Windows 版本分裂阻断；target-server lib-test/全目标门由 Frameworks03 未按 profile 门控的 graphics/UI/script/dynamic-api/physics-contract 测试与可执行目标阻断，均已有对应 open failure 记录，故 M1 测试阶段不关闭。模块文档：`docs/zircon_runtime/scene/serialization.md`、`docs/zircon_runtime_interface/serialization.md`。 |
| M2 | 2.2 DynamicScene 双写字段硬删除 | `切片完成-专属合同8/8-broad外部失败已路由` | 2026-07-14 | DynamicScene schema 提升到 v2，v1→v2 迁移严格要求历史内嵌版本为 1 后删除；当前结构、writer、capture、session summary 与 scene root 均不再保留旧字段、常量或兼容重导出。TDD 负例作业 `c3c58d75dc7549c6adede931f806742b` 先红，修复后 exact 作业 `feded57b4f9946cdb5e70e8cf1c18075` 1/1，最终 Plan11 作业 `e6b4ac85c8994c0eadfea26fb026061f` 8/8。core-min broad 作业 `0828b8e5681045ccb47296cbcc1880f3` 为 595/596，唯一失败归属 Plugins08 dynamic reflection，并已追加到对应开放 failure；Text01 target-client 产品作业 `d80d6dabac754907b50aa3ae2c1c1056` 1/1 且 root-export failure 已回传。M2.1 与 M3 未完成，计划保持 `in_progress`。 |
| M2 | 2.1a appearance preferences 统一版本壳硬切 | `实现完成-静态合同4/4-rustfmt通过-Cargo待协调器解禁` | 2026-07-18 | `EditorAppearancePreferencesDocument` 删除 payload 内嵌 `version` 与本地版本常量，新写入统一为 canonical JSON `$zircon` schema v1；旧 v1/v2 TOML 只经无壳 v0 迁移链读取，不保留双写/旧 writer。启动加载消费 `migrated_from` 并提示重存，future shell fail closed。真实 v1 fixture、canonical 幂等与 future 拒绝 Rust 回归已落盘；Python 结构合同 4/4、精确 rustfmt check 通过。Cargo 因 Coordinator01 当前禁止非 immutable full compile-input snapshot 作业而未启动；M2.1 其余 keymap/journal/layout 未完成，本项仍不勾选。子记录：[2026-07-18 appearance preferences version shell](11/2026-07-18-appearance-preferences-version-shell.md)。 |

- fixed 已修复：[dynamic-scene-format-version-root-export-drift](../../zircon_runtime/text/01/fixed-2026-07-14-dynamic-scene-format-version-root-export-drift.md)
- 2026-07-23 serialization性能交接（当前源复核）：`zircon_runtime_interface/src/serialization/**`物理42/42、3,598行、44 tests已读；外部owner已让generic current reader借用RawValue/direct typed、writer删除Value DOM，但`inspect_text`仍重解析whole/envelope/header/whole/envelope后再typed，canonical writer仍为每scalar/subtree建String并在祖先逐层join/format，且text无hard budget、chain每load重验。PERF-MVP-570改用bounded single strict-envelope seed（envelope≤1+payload typed≤1）、static chain result与single bounded byte/chunk owner，输出byte copy与depth解耦；1,061行write owner按serializer/error/map-key/compound/output拆分。binary按571继续保留v1 golden与64MiB/128/2M/1M/16MiB门，不得引入兼容双reader。
- 2026-07-23 export preset补证：`zircon_runtime_interface/src/export/**` 6/6确认`load_export_preset`先把完整strict document及payload解为`Value`，随后进入generic multi-probe+typed路径；这是PERF-MVP-570的额外DOM/多遍实例。Editor11保留strict schema/version/unknown-field拒绝，但复用bounded single envelope入口，不另建preset payload DOM；1KiB/64MiB、depth 128/129验收current preset envelope≤1遍、payload typed≤1遍、payload Value=0、RSS硬有界。
- 2026-07-30 settings实例补证：`core/settings/io.rs::decode_current_document`先把完整source解为`serde_json::Value`仅检查`$zircon`，再把相同bytes交`load_versioned`完整解析；归入PERF-MVP-570，不在Editor17建立第二reader。Editor11提供single strict-envelope seed并保留legacy/current/future/schema/unknown-field fail-closed语义；settings `1KiB/64MiB`要求envelope≤1遍、payload typed≤1遍、payload Value=0、输入owner=1与RSS硬界。证据见`../../performance/01/2026-07-30-editor-core-settings-static-review.md`。
