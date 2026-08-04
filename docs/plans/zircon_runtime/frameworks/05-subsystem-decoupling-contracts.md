---
related_code:
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/graphics/extract/mod.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/platform/preferences/atomic_file.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/close_project.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/index.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/render/index.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
reference_engines:
  - dev/godot/servers/rendering/rendering_server.h
  - dev/godot/core/templates/rid.h
  - dev/bevy/crates/bevy_render
---

# 05 · 子系统解耦契约收束

## 1. 目标

把 runtime 内部所有**跨域直接引用**收敛为 `core/framework`（未来 `zr_contracts`）契约 + handle/registry 访问，使每个域满足："上层只见契约与句柄，不见邻域内部类型"。这是计划 01 Phase 3（graphics/ui/text 拆 crate）的硬前置：接缝不切干净，crate 拆分会在孤儿规则和循环依赖上撞墙。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-runtime-subsystem-decoupling-contracts",
  "goal": "收敛 runtime 跨域依赖为中立契约、句柄与注册表边界",
  "milestones": [
    {"id": "M1", "title": "接缝普查与契约定稿", "depends_on": []},
    {"id": "M2", "title": "AssetLoaderRegistry 与声明顺序解耦", "depends_on": ["M1"]},
    {"id": "M3", "title": "共享文本服务契约", "depends_on": ["M2"]},
    {"id": "M4", "title": "graphics-scene 与 manager 面收口", "depends_on": ["M3"]},
    {"id": "M5", "title": "Core contracts 反向依赖移交修复", "depends_on": []},
    {"id": "M6", "title": "跨平台偏好存储与有界持久化", "depends_on": ["M4"]},
    {"id": "M7", "title": "ProjectAssetManager generation publication", "depends_on": ["M4"]}
  ]
}
```

## 2. 现状接缝清单（证据）

| # | 接缝 | 现状 | 目标契约 |
|---|------|------|---------|
| S1 | graphics ↔ ui 文本 | 2026-07-13 production 基线 graphics→ui 已从 4 硬切为 0；ui→graphics 为 28，仍包含 UI 对 `graphics::text::{font,layout,shaping}` 的实现依赖 | 共享文本服务契约（shaping/字形图集/render mode）进 framework::text；实现下沉为 `zr_text`（勾稽 render/14 的"shaping/字形图集下沉共享"既定方向） |
| S2 | asset → ui 模板 loader | M2 已从 3 清零；`.zui` backend/注册归 `ui_document_importer`，asset 仅保留 DTO wrapper、local codec/reference helper 与通用 `AssetImporterRegistry` | `AssetImporterRegistry` 是唯一 loader 扩展契约；UI 文档插件通过 runtime extension registry 注册 `.zui`，asset 域不再拥有/调用 UI loader 实现 |
| S3 | graphics → scene | 已完成：`SCENE_MODULE_NAME` 唯一 owner 硬切到 `core::framework::scene::module_identity`，graphics/script/ui/scene 与测试调用方全部改读中立契约；scene root 不再导出该常量，fresh production-only 审计为 graphics→scene=0 | graphics 只消费 framework::render 的 extract/snapshot DTO、scene 句柄与中立 module identity；禁止触 ECS 内部类型或从 scene root 借常量；与 render 计划集“extract 即 proxy 快照”口径一致 |
| S4 | 各域 → core/manager 具名服务 | 2026-07-14 已删除 14 组旧 Arc holder、asset 旧 resolver 与隐式 Arc adapter；当前稳定接缝是 versioned `ManagerServiceHandle<T>`、注册 adapter、lifecycle stale/unloaded/single-flight/reentry 约束及 use-point access。仍须持续审计 graphics 等构造面是否把 concrete manager `Arc` 重新变成跨帧/跨域状态 | 跨域访问一律使用 `ManagerServiceHandle<T>{index,generation,service}` + use-point resolver；句柄携带 index+generation（godot RID 纪律），长期状态不得持有邻域 concrete manager 或 `Arc<dyn Trait>`，构造期注入也不得绕过此规则形成第二套 owner |
| S5 | lib.rs 声明顺序耦合 | "ui must be declared before asset" 注释 | S2 完成后该顺序约束自然消失，删除注释并加守卫（声明顺序不再承载语义） |

M1 全量扫描已落地；M2 基线 [`../../_archive/zircon_runtime/frameworks/05/baselines/2026-07-10-runtime-domain-dependencies.json`](../../_archive/zircon_runtime/frameworks/05/baselines/2026-07-10-runtime-domain-dependencies.json) 持有 asset→ui 3→0 的历史证据。M3 基线 [`05/baselines/2026-07-13-runtime-domain-dependencies.json`](05/baselines/2026-07-13-runtime-domain-dependencies.json) 保留旧审计口径下 graphics→ui 1→0 的历史切片证据；production-only 机器快照 [`05/baselines/2026-07-13-runtime-domain-dependencies-production-only.json`](05/baselines/2026-07-13-runtime-domain-dependencies-production-only.json) 持有当时的 2,290 / 72 事实。2026-07-18 Foundation successor 在共享 current source 上重新扫描得到 2,401 条 production refs / 74 edges。2026-08-02 对共享 current source 的 latest read-only 复测为 **2,584 refs / 79 edges**：`asset→text=0`、`asset→ui=0`、`ui→graphics=0` 与 `graphics→scene=0`，但 `graphics→ui=3`（resolved glyph artifact owner 回流）、`scene→animation=2`（已 feature-gate、尚未完成物理 crate DAG 解耦）、`rhi→rhi_wgpu=1`（`rhi/mod.rs` 直接构造默认 WGPU UI presenter）以及 `asset→foundation=9` 仍开放。另有 12 个 `rhi/tests` backend imports 直接导入 deterministic `rhi_wgpu` device；它们不计入 production matrix，但必须在物理 hard-cut 前同批迁移。同日 full-tree 审计另发现 1 个 asset font test 曾经 `crate::text::FontScript` 投影，现已在 exact12 前向改读 `crate::asset::FontScript`，asset production 与 test tree 的该反向路径均为 0。该 fresh 计数不是原子 baseline；同日两次扫描间总 refs 已由 2,583 增至 2,584，证明它不能覆盖并发 Session 的后续源码变化，物理迁移前仍须重新冻结。初始 2,151 / 77 数值只保留在 M4 历史产出行中，不再冒充当前机器基线。另由 [`05/baselines/2026-07-10-contract-signatures.md`](05/baselines/2026-07-10-contract-signatures.md) 继续锁定 S1–S4 签名和计数验收规则。后续切片必须使用 production-only 口径复测。

## 3. 设计决策

1. **契约放置**：所有跨域 trait/DTO 进 `core/framework/<domain>/`；契约层零实现、零重依赖（`*-contracts` feature 可单独编译，勾稽计划 03 §3.3-4）。
2. **访问形态优先级**：extract/snapshot DTO（数据面）> registry 注册（扩展面）> handle+resolver（服务面）。禁止新增"直接引用邻域模块路径"的第四形态。
3. **句柄纪律**：跨域句柄统一 index+version 语义（防复用悬垂），由 `core/resource`/`core/manager` 既有机制承载，不另造。
4. **文本服务归属**：shaping/字形/排版属共享底座（graphics 与 ui 都消费），按 render/14 既定方向独立成域（`zr_text`），本计划先立契约后搬实现。
5. **Generation publication**：manager generation 的状态提交与对外事件发布属于同一线性化边界；旧 generation 的 retire/Removed 必须先于新 generation 的 Added。可阻塞的 watcher join/drop 必须移出 generation fence，不能用缩短锁生命周期换取错误事件顺序。
6. **持久化线程纪律**：Platform 偏好合同保持 backend-neutral；同步 filesystem/backend 调用不得出现在 frame/UI caller。接线前必须复用 Runtime11 统一 bounded I/O/persistence lane，按 key 合并 generation，并提供 read-your-write、flush/shutdown fence、容量与取消/错误可观测性，禁止私有线程池或无界队列。

## 4. 里程碑

### M1 接缝普查与契约定稿

实现切片：交叉引用扫描脚本（输出"域→域"引用矩阵与逐条清单）；对 §2 清单逐条写契约草案（trait/DTO 签名级）；矩阵与草案入 baselines/ 并在本文件锁定。

测试阶段：无编译门；验收证据 = 引用矩阵 + 契约签名清单 + 本文件 §2 表补全。

### M2 S2/S5：AssetLoaderRegistry

实现切片：复用现有 `AssetImporterHandler` + `AssetImporterRegistry` 唯一扩展契约；`.zui` loader/映射由 `ui_document_importer` runtime plugin 注册；asset 内旧 builtin `.zui` backend/转换 owner 硬删除；asset DTO wrapper 改用本域 local codec/reference helper；删除 asset→ui 引用与 lib.rs 顺序注释。

测试阶段（按 `docs/plans/milestone-validation-policy.md` §3 最小批次）：
- `cargo check -p zircon_runtime --lib --locked`；focused 过滤词批分别执行 `cargo test -p zircon_runtime --lib --locked asset`、`cargo test -p zircon_runtime --lib --locked loader`、`cargo test -p zircon_runtime --lib --locked importer`（loader 注册/解析单测 + 变更面 asset 回归）；全量 lib 回归留给波次收口（policy §4）；
- 验收证据：`grep -r "use crate::ui" zircon_runtime/src/asset/` 为空。

### M3 S1：共享文本服务契约

实现切片：framework::text 契约（shaping 请求/字形度量/render mode）；graphics 侧改走契约；ui/text 实现挂注册。实现搬迁到独立域（zr_text 目录形态）可与计划 01 M3 合批。

2026-08-02 post-closeout 结构修正：`CompositeFontDescriptor`、`SubFontRange`、`FontScript`、`FontCultureTag` 与 `FontFamilyName` 的唯一声明 owner 已硬切到 `asset::assets::font`，因为这些类型描述资产 authoring/cache wire schema，而不是 text runtime 实现。`zircon_runtime::text` 只直接投影同一 asset-owned 类型；asset production 对 `crate::text` 的反向边由 2 收敛为 0，旧 `text/model/font/composite.rs` 已物理删除，不保留 alias、转换 DTO 或 forwarding module。类型 derives、variant/字段顺序、serde 属性以及 `FontAsset::composite_font`/cache 字段的 `text` feature gate 均保持不变。exact11 代码/计划范围的 Rust 1.94.1 scoped format、diff-check、唯一 owner/production 零反向边/旧路径删除/公共投影静态守卫与独立二次审查 **Critical 0 / Important 0 / Minor 0** 已由 snapshot 1435 冻结。后续 full-tree 复核发现 `asset/tests/assets/font.rs` 尚有 1 条 `crate::text::FontScript` 投影；现已认领该文件并前向改读 `crate::asset::FontScript`，exact12 scoped Rust 1.94.1 format、diff-check 与 asset tree 旧投影零残留守卫通过，fresh independent review 为 **Critical 0 / Important 0 / Minor 0**。当前状态为 `production_and_test_consumer_implementation_and_review_completed / managed_acceptance_pending`。Windows/Rust 1.94.1 package check、owner focused 与 `font_artifact_cache_contract` 行为门已分别收到受管 queued receipt `a368138706e9463dbcf6f0454afcdb98`、`4f9a8cb36d1d49feaf08ac2ca518f907`、`f5011a172f56415a99b795b3d42f17bf`；未查询 terminal，且这些 receipt 早于 exact12 test-consumer 修复，不能作为完整 hard-cut acceptance。

测试阶段（policy §3 最小批次）：
- focused 过滤词批分别执行 `cargo test -p zircon_runtime --lib --locked text`、`cargo test -p zircon_runtime --lib --locked shaping`、`cargo test -p zircon_runtime --lib --locked glyph`（graphics/text 与 ui/text 两侧变更面）；全量 lib 回归留给波次收口；
- 渲染冒烟：editor-host 启动含文本场景截屏对比；
- 验收证据：`grep -r "use crate::ui" zircon_runtime/src/graphics/` 为空。

### M4 S3/S4：graphics↔scene 与 manager 面收口

实现切片：35 处 scene 引用逐条归类（extract DTO 可保留的、需要换句柄的、违规触 ECS 内部的），违规项改契约；manager 面跨域裸类型复核清零。

测试阶段（policy §3 最小批次）：
- focused 过滤词批分别执行 `cargo test -p zircon_runtime --lib --locked graphics`、`cargo test -p zircon_runtime --lib --locked scene`、`cargo test -p zircon_runtime --lib --locked extract`，再执行 editor/runtime 双启动冒烟；全量 lib 回归留给波次收口；
- 验收证据：引用矩阵复测——graphics 对 scene 的引用只剩 framework 契约路径与公开句柄；守卫脚本（计划 06）就位后此矩阵成为常驻断言。

### M5 Core contracts 反向依赖移交修复

实现状态：`completed`。Frameworks01 移交的 `core/contracts` 反向依赖已按中立 owner 修复并通过 current-source 禁止边、受管 Windows 编译与核心行为门；canonical fixed 记录为 [`01/fixed-2026-07-13-core-contract-reverse-dependencies.md`](01/fixed-2026-07-13-core-contract-reverse-dependencies.md)。本里程碑不再列为 pending，也不替代 M4/M6/M7 的独立验收。

### M6 跨平台偏好存储与有界持久化

已实现：中立 `PreferenceStorage` 契约、错误分类、Platform manager handle/capability projection、desktop atomic-file backend 与 host 注入；mobile/browser/headless 在 host 未显式注入时稳定报告 unavailable，Minimal profile 不偷偷激活 Platform。当前 source/static boundary 为 7/7 与完整 Frameworks05 52/52。

剩余实现：WOC adapter 仍是项目内 trait，真实 mobile/browser backend、fresh-process 多角色隔离及 Runtime/WOC upward gates 未关闭；在任何 WOC/Editor frame/UI 接线前，Runtime11 必须提供唯一 bounded persistence lane，按 key 合并 latest generation、缓存稳定 hash/path、在内存发布 read-your-write，并以显式 flush/shutdown fence 传播 timeout/cancel/error。验收沿用 `PERF-MVP-589` 的 queue entries/bytes/age/coalesce、filesystem wall 与 durability matrix；frame/UI caller filesystem wall 必须为 0。

### M7 ProjectAssetManager generation publication

已实现：`AssetManager::close_project` 唯一 close/deactivate contract、project snapshot/resource/source-index/watcher retirement 与 no-active-project no-op；Editor 不拥有 manager 内部清理路径。

当前源码实现：新增唯一 manager-owned `publish_project_generation` 线性化 owner；close、targeted import、full reimport 与 watcher commit 均在释放 project lock 后继续持有 generation write fence，完成 asset-change broadcast 后才由该 owner 释放 fence。close 的 watcher join/drop 保持在 fence 外；无活动 project 的 close 不推进 preparation epoch，不再误 supersede 并发 open。focused 并发回归证明 broadcast 阻塞期间 generation read/write try-lock 均不可用，另有 no-op epoch 与四调用点禁止 early-drop 守卫。Rust 1.94.1 scoped format、diff-check 与静态守卫已通过，独立二次审查为 Critical 0 / Important 0 / Minor 0；fresh managed Runtime manager、Editor01 document/Editor12 bridge upward gates、fixed return 与 milestone commit 仍待完成。

## 5. 风险与回退

- **性能面**：extract/句柄间接层不得引入每帧堆分配或动态派发热点；契约签名评审时对热路径（文本 shaping、extract）要求零成本抽象（泛型/静态派发或批量 DTO）。
- **契约过度设计**：只为已存在的接缝立契约，不预铺"未来可能"的接口（架构深度测试以现有两个消费方为准）。
- **与 render/ui 计划集撞车**：S1/S3 的语义口径以 render 计划集为准，本计划只负责"引用形态合规"，不改渲染行为；动手前在对应计划 index 勾稽一行。

## 6. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

2026-08-03 状态校正：M1、M3、M5 的历史 accepted milestone 已完成；M3 composite font asset-schema 的 production owner hard-cut 与 test-consumer forward fix 已在 exact12 完成静态门及 independent re-review **Critical 0 / Important 0 / Minor 0**，当前仅 managed acceptance pending；M2 的 S2/S5 代码与静态门完成、完整 Runtime acceptance pending；M4 的 module identity/manager hard cut 已落地。fresh read-only 2,586/70 矩阵确认此前列出的 graphics→ui、rhi→rhi_wgpu 与 asset→foundation concrete seams 均为 0，当前最低源码接缝只剩 scene→animation=2；这些已清零切片各自的受管 closeout 仍按 owner 生命周期完成，不以源码计数替代验收。M6/M7 为当前 failure-first 实现线。开放节点分别为 [cross-platform-preference-storage-service](05/failure-2026-07-19-cross-platform-preference-storage-service.md)（引擎合同已实现，bounded lane/WOC/真实平台与受管门 pending）、[preference-quota-error-kind-toolchain-drift](05/failure-2026-07-22-preference-quota-error-kind-toolchain-drift.md)（source/review GREEN，validation receipt accepted）、[animation-scene-hook-guard-stale-path](05/failure-2026-07-29-animation-scene-hook-guard-stale-path.md)（Frameworks05 28/28 static GREEN，Plugins04 managed compile/fixed return pending）、[project-asset-manager-close-contract-missing](05/failure-2026-07-29-project-asset-manager-close-contract-missing.md)（generation publication current-source implementation/static/independent review GREEN，受管 upward gates pending）与 Runtime04-owned [shared-atomic-file-owner-reverse-dependencies](../runtime/04/failure-2026-08-02-shared-atomic-file-owner-reverse-dependencies.md)（current production edge 已清零，受管验证/fixed return 仍归该 owner）。任何 queued/running validation 只延迟 accepted closeout，不把 Session 置为 blocked，也不阻止独立实现继续。

- fixed 已修复：[milestone-finalize-per-path-blob-verification-stall](05/fixed-2026-07-15-milestone-finalize-per-path-blob-verification-stall.md)
- fixed 已修复：[text-physical-owner-hard-cut-compile-break](../text/03/fixed-2026-07-15-text-physical-owner-hard-cut-compile-break.md)
- open 已移交 Text03：[text-layout-ellipsis-paragraph-regressions](../text/03/failure-2026-07-15-text-layout-ellipsis-paragraph-regressions.md)
- fixed 已修复：[ai-runtime-root-lockfile-drift](05/fixed-2026-07-16-ai-runtime-root-lockfile-drift.md)
- fixed 已修复：[text-hard-cut-runtime-consumer-type-drift](../runtime/04/fixed-2026-07-15-text-hard-cut-runtime-consumer-type-drift.md)
- fixed 已修复：[rich-text-dto-render18-retest-gate](../render/18/fixed-2026-07-15-rich-text-dto-render18-retest-gate.md)
- fixed 已修复：[rich-text-dto-shader06-source-staging-gate](../shader/06/fixed-2026-07-15-rich-text-dto-shader06-source-staging-gate.md)

已回传 Failure 概述：[`core/contracts` 反向依赖上层域与 facade](01/fixed-2026-07-13-core-contract-reverse-dependencies.md)（Frameworks01 M0 发现；current-source 禁止边清零，受管 Windows 编译与核心行为门已通过）；[runtime-profile-id-consumer-cutover](../../zircon_editor/editor/09/fixed-2026-07-13-runtime-profile-id-consumer-cutover.md)；[level-manager-export-cutover-incomplete](../runtime/02/fixed-2026-07-14-level-manager-export-cutover-incomplete.md)；[level-manager-name-core-error-import-drift](../../zircon_editor/editor/02/fixed-2026-07-14-level-manager-name-core-error-import-drift.md)；[project-asset-manager-access-test-consumer-drift](../../zircon_editor/editor/02/fixed-2026-07-14-project-asset-manager-access-test-consumer-drift.md)；[editor-retained-host-manager-resolver-consumer-drift](../../zircon_editor/editor/02/fixed-2026-07-14-editor-retained-host-manager-resolver-consumer-drift.md)；[manager-service-reactivation-lifecycle](../runtime/15/fixed-2026-07-14-manager-service-reactivation-lifecycle.md)；[ui-text-project-asset-manager-access-consumer-drift](../runtime/15/fixed-2026-07-14-ui-text-project-asset-manager-access-consumer-drift.md)；[ui-text-manager-access-cross-frame-retention](../runtime/15/fixed-2026-07-14-ui-text-manager-access-cross-frame-retention.md)。
- 产出记录：[`05/2026-07-10-subsystem-decoupling-contracts-output-records.md`](05/2026-07-10-subsystem-decoupling-contracts-output-records.md)
- M4 module identity 记录：[`05/2026-07-18-m4-scene-module-identity-contract-hardcut.md`](05/2026-07-18-m4-scene-module-identity-contract-hardcut.md)（生产硬切、fresh 24/24 layer suite 与扩展 27 路径独立复审 P0/P1/P2=0 已通过；双 canonical lock 已闭合，当前源码受管 Cargo 尚待 accepted）
- M4 module identity 记录：[`05/2026-07-18-m4-platform-module-identity-contract-hardcut.md`](05/2026-07-18-m4-platform-module-identity-contract-hardcut.md)（`PLATFORM_MODULE_NAME` 已硬切到中立 platform contract owner，concrete platform 旧声明与 Runtime 内部旧路径消费者归零；公开 `zircon_runtime::platform` 仅直接 curated re-export neutral owner。focused unique-owner guard 按 RED→GREEN，review 修正后的完整 layer suite 25/25、production audit 2,395 refs / 76 edges 且 graphics/input→platform 均为 0、Rust 1.94.1 rustfmt 与 diff-check 已通过；第三轮独立复审 Critical/Important/Minor=0/0/0，受管 Cargo pending）
- M4 module identity 记录：[`05/2026-07-18-m4-input-module-identity-contract-hardcut.md`](05/2026-07-18-m4-input-module-identity-contract-hardcut.md)（`INPUT_MODULE_NAME` 已硬切到中立 input contract owner，UI/builtin/core-spine 内部旧路径归零，公开 `zircon_runtime::input` 只直接 curated re-export neutral owner；focused RED→GREEN、完整 layer suite 26/26、production audit 2,398 refs / 75 edges 且 ui→input 与两条 platform target edges 均为 0。948 行 guard route 已拆为 856+275 行 owners；Rust 1.94.1 rustfmt、Python compile、diff-check 已通过，review 修正后独立复审 Critical/Important/Minor=0/0/0，受管 Cargo blocked）
- M4 module identity 记录：[`05/2026-07-18-m4-ui-module-identity-contract-hardcut.md`](05/2026-07-18-m4-ui-module-identity-contract-hardcut.md)（`UI_MODULE_NAME` 已从 concrete `ui/module.rs` 硬切到中立 UI contract owner，builtin/core-spine 内部旧路径归零，公开 `zircon_runtime::ui` 只直接 curated re-export neutral owner；focused RED→GREEN、完整 layer suite 27/27、production audit 2,400 refs / 75 edges，既有 ui→input 与 graphics/input→platform target edges 均保持 0。两轮 scanner finding 修正后第三轮独立复审 Critical/Important/Minor=0/0/0；受管 Cargo 仍 blocked，不提升 M4）
- M4 contract owner 记录：[`05/2026-07-18-m4-foundation-module-identity-contract-hardcut.md`](05/2026-07-18-m4-foundation-module-identity-contract-hardcut.md)（旧 `core/framework/foundation.rs` 与 concrete identity 声明已删除，ConfigManager/report/EventManager/identity 各归 folder-backed owner；Asset/Platform/builtin/core-spine identity 消费改读中立 contract，公开 `zircon_runtime::foundation` 仅 direct structural projection。focused RED→GREEN、fresh 完整 layer suite 28/28、production audit 2,415/74、structure audit 证明真实 FoundationModule owner/stub=0。clean-HEAD exact40 复审为 Critical/Important/Minor=2/1/0：Scene neutral identity 前置及 Runtime02/Runtime07 Foundation 支撑尚未由各 owner 提交；当前状态为 prerequisite-owner-commit 与 re-review pending，不提升 M4）
- M3 当前状态：历史 milestone `completed`，2026-08-02 asset-schema owner correction 为 `production_and_test_consumer_implementation_and_review_completed / managed_acceptance_pending`。共享文本中立契约、单一 `zircon_runtime::text` implementation owner、canonical UI/prewarm/fallback 链、generation-aware font/SDF invalidation、graphics-only neutral transport 与所有 production consumer 已完成硬切；原 closeout 的 fresh 证据包括 static 13/13、default/target-server/graphics-only production exit 0、neutral transport 5/5、source-manifest 5/5、font-asset 4/4、default/UI integration 6/6、graphics-only upward 28/28，以及 multilingual GPU exporter 1/1。post-closeout 修正进一步把五个 composite font authoring/cache schema 类型的唯一 owner 移入 `asset::assets::font`，清零 asset→text production edge 并物理删除旧 `text/model/font/composite.rs`；exact11 静态门和独立复审为 **Critical 0 / Important 0 / Minor 0**，snapshot 1435 仅作为 test-consumer 修复前的不可变前驱。当前 exact12 已把 asset font test 直接改读 asset owner 并通过 scoped format/diff/旧路径零残留守卫，fresh independent review 为 **Critical 0 / Important 0 / Minor 0**；仍须 managed terminal，故不把旧 543-path、snapshot 1435 或先前 receipts 复标为完整 current-source acceptance。详细历史证据见 [`05/2026-07-15-m3-shared-text-service-contract-closeout.md`](05/2026-07-15-m3-shared-text-service-contract-closeout.md)。M4、Text03/Shader06 外部行为待办和整份计划仍未完成。
- 当前状态：M1 已完成；M2 S2/S5 代码与静态门已完成，asset→ui 3→0，旧 builtin `.zui` owner 和声明顺序注释均已删除；历史 focused/package 验证由编号归档持有，完整 Runtime 门仍 pending。M3 的 `PublicRuntimeFrame` graphics→ui 与 composite-font asset→text 边在 2026-08-03 fresh 2,586/70 production matrix 中均为 0，相关 managed acceptance 仍由各 owner 闭合。M4 工作树已实现 `SCENE_MODULE_NAME`、`PLATFORM_MODULE_NAME`、`INPUT_MODULE_NAME`、`UI_MODULE_NAME`、`FOUNDATION_MODULE_NAME` 的中立 contract hard cut，并按结构规范拆出 Foundation named owners；当前 graphics→scene、graphics→ui、asset→foundation 与 rhi→rhi_wgpu 均为 0，但不以 fresh 计数替代各 failure 的受管验收。2026-07-14 完成 versioned manager identity、registration adapter、lifecycle stale/unloaded、single-flight、跨线程依赖环与 Immediate activation reentry 约束，以及 asset/graphics/editor use-point access 硬切，删除 14 组旧 Arc holder、asset 旧 resolver 与 `IntoProjectAssetManagerAccess` 隐式 Arc adapter。Frameworks05 lib-test consumer failure 已按协调器流程 fixed 回传 Editor02；Foundation successor 当前 layer suite 28/28、production audit 2,415/74，但 clean-HEAD exact40 独立复审为 Critical/Important/Minor=2/1/0，正等待 Scene、Runtime02、Runtime07 owner commits 后再做受管 Cargo 与复审。当前最低源码 DAG 阻断只剩 `scene→animation=2`；Frameworks01 RHI 物理 hard cut 与 Runtime04 shared I/O owner 的受管 closeout 仍须独立完成，不能用本切片冒充整份计划完成。
2026-07-14 M4 correction 概述：单模块与批量 activation 已按完整 `ModuleEntry::service_names` 恢复 `Unloaded` slot，当前状态为 `frameworks_05_m4_manager_service_reactivation_lifecycle_current_source_passed`。UI text 的初次 constructor 类型修复经独立 review 发现长期 text owner 仍跨帧保存 concrete manager；现已硬切为跨帧只存 versioned access、构造与每帧 use point 各解析一次，专门守卫、24/24 layer suite 与 fresh managed default-feature Runtime build 通过，Failure 已原子回传，状态为 `frameworks_05_ui_text_manager_access_cross_frame_retention_fixed_returned_re_review_pending`。两项工作在当日未提升其余 M3/M4；M5 的历史 pending 已由本页 2026-07-31 `completed` 状态校正取代，详细证据由 `05/` 编号归档持有。
- fixed 已修复：[manager-resolver-weak-core-test-consumer-drift](../../zircon_editor/editor/03/fixed-2026-07-15-manager-resolver-weak-core-test-consumer-drift.md)

2026-07-16 current-source 恢复补录：Text owner 已补齐 `ScreenSpaceUiNativePrepareReport::font_faces_changed` 两处测试夹具，并完成 font-face generation rollover 后的 SDF atlas 全页 dirty-until-upload 语义；独立复审最终为 **Critical 0 / Important 0 / Minor 0**。fresh `python -m unittest tools.tests.test_frameworks_05_text_boundary -v` 为 **13/13**。Windows managed job `70627811c1204085a79ca1ef08772262` 顺序通过 default production 与 target-server 编译，graphics-only production job `ebf608cbe64f414797afe4edf9511bf5` 也以 exit 0 release。fresh multilingual GPU job `294971bdfc37467c80858318c6e4edfd` 通过 exact exporter **1/1**；docs PNG 已 fresh 写回并完成尺寸、哈希、target 零副本与目检。最终 source-manifest 5/5、font-asset 4/4、default/UI integration 6/6 与 graphics-only upward 28/28 均受管通过，M3 不再受 Runtime15 外部 test-anchor 漂移阻塞。

2026-07-16 M3 收口续录：Runtime15 archive-anchor Failure 已 fixed return，五处 active anchor 归零；managed exact job `eaebe4e27b7c4f6ab267a512c0854a2b` 为 **1/1**、released / exit 0。Frameworks05 current-source neutral transport job `139ecf26f34b4a578edf4cbb98bec8fe` 为 **5/5**、released / exit 0；最终 `773e431acb27467694f41861660ad0d4` graphics-only upward **28/28**、`e306fe34652c49c58edd0f1c59976418` default/UI **6/6**、`e1fdd58a751144f4a319787038c2f1f3` source-manifest **5/5**、`c3b71610d86f494bbcd40691d6fd32a5` font-asset **4/4** 均通过。M3 已完成并冻结 543-path exact manifest；M4 与 Text03/Shader06 外部行为范围不随本切片提升。
