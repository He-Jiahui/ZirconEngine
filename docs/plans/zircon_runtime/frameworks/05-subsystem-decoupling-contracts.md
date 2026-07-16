---
related_code:
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/graphics/extract/mod.rs
  - zircon_runtime/src/scene/mod.rs
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
    {"id": "M5", "title": "Core contracts 反向依赖移交修复", "depends_on": []}
  ]
}
```

## 2. 现状接缝清单（证据）

| # | 接缝 | 现状 | 目标契约 |
|---|------|------|---------|
| S1 | graphics ↔ ui 文本 | 2026-07-13 production 基线 graphics→ui 已从 4 硬切为 0；ui→graphics 为 28，仍包含 UI 对 `graphics::text::{font,layout,shaping}` 的实现依赖 | 共享文本服务契约（shaping/字形图集/render mode）进 framework::text；实现下沉为 `zr_text`（勾稽 render/14 的"shaping/字形图集下沉共享"既定方向） |
| S2 | asset → ui 模板 loader | M2 已从 3 清零；`.zui` backend/注册归 `ui_document_importer`，asset 仅保留 DTO wrapper、local codec/reference helper 与通用 `AssetImporterRegistry` | `AssetImporterRegistry` 是唯一 loader 扩展契约；UI 文档插件通过 runtime extension registry 注册 `.zui`，asset 域不再拥有/调用 UI loader 实现 |
| S3 | graphics → scene | production-only 审计已把 13 条收敛为 1 条真实接缝：graphics 模块描述符仍从 scene 域读取 `SCENE_MODULE_NAME`；其余 12 条均为内联测试或测试专用支持模块 | 已有 extract packet 契约的扩面：graphics 只消费 framework::render 的 extract/snapshot DTO 与 scene 句柄，禁止触 ECS 内部类型；模块依赖标识必须改由中立 runtime module contract 持有，不允许从邻域 root 借常量；与 render 计划集"extract 即 proxy 快照"口径一致 |
| S4 | 各域 → core/manager 具名服务 | 已完成 current-source 分类：`core/manager` 的 14 个 holder 仍全部内持 `Arc<dyn Trait>`；跨域 `ProjectAssetManager` 有 34 条 production 引用（animation 6 / dynamic_api 2 / graphics 25 / plugin 1），graphics 内另有 27 处 `Arc<ProjectAssetManager>` 字段/签名传播与 3 个跨域 concrete `resolve_manager` 调用 | 复核 manager 面：跨域访问一律 `ManagerServiceHandle<T>{index,generation,service}` + use-point resolver，句柄携带 index+generation（godot RID 纪律），禁止跨域长期持有 `Arc<具体类型>` 或 `Arc<dyn Trait>` |
| S5 | lib.rs 声明顺序耦合 | "ui must be declared before asset" 注释 | S2 完成后该顺序约束自然消失，删除注释并加守卫（声明顺序不再承载语义） |

M1 全量扫描已落地；M2 基线 [`../../_archive/zircon_runtime/frameworks/05/baselines/2026-07-10-runtime-domain-dependencies.json`](../../_archive/zircon_runtime/frameworks/05/baselines/2026-07-10-runtime-domain-dependencies.json) 持有 asset→ui 3→0 的历史证据。M3 基线 [`05/baselines/2026-07-13-runtime-domain-dependencies.json`](05/baselines/2026-07-13-runtime-domain-dependencies.json) 保留旧审计口径下 graphics→ui 1→0 的历史切片证据；当前 production-only 机器基线 [`05/baselines/2026-07-13-runtime-domain-dependencies-production-only.json`](05/baselines/2026-07-13-runtime-domain-dependencies-production-only.json) 排除内联 `cfg(test)` 项及测试入口递归挂载的支持文件，持有 2,290 条生产逐行证据与 72-edge 矩阵，asset→ui=0、graphics→ui=0、ui→graphics=28、graphics→scene=1，且本次 handoff 九组禁止方向全部为 0。初始 2,151 / 77 数值只保留在 M4 历史产出行中，不再冒充当前机器基线。另由 [`05/baselines/2026-07-10-contract-signatures.md`](05/baselines/2026-07-10-contract-signatures.md) 继续锁定 S1–S4 签名和计数验收规则。后续切片必须使用 production-only 口径复测。

## 3. 设计决策

1. **契约放置**：所有跨域 trait/DTO 进 `core/framework/<domain>/`；契约层零实现、零重依赖（`*-contracts` feature 可单独编译，勾稽计划 03 §3.3-4）。
2. **访问形态优先级**：extract/snapshot DTO（数据面）> registry 注册（扩展面）> handle+resolver（服务面）。禁止新增"直接引用邻域模块路径"的第四形态。
3. **句柄纪律**：跨域句柄统一 index+version 语义（防复用悬垂），由 `core/resource`/`core/manager` 既有机制承载，不另造。
4. **文本服务归属**：shaping/字形/排版属共享底座（graphics 与 ui 都消费），按 render/14 既定方向独立成域（`zr_text`），本计划先立契约后搬实现。

## 4. 里程碑

### M1 接缝普查与契约定稿

实现切片：交叉引用扫描脚本（输出"域→域"引用矩阵与逐条清单）；对 §2 清单逐条写契约草案（trait/DTO 签名级）；矩阵与草案入 baselines/ 并在本文件锁定。

测试阶段：无编译门；验收证据 = 引用矩阵 + 契约签名清单 + 本文件 §2 表补全。

### M2 S2/S5：AssetLoaderRegistry

实现切片：复用现有 `AssetImporterHandler` + `AssetImporterRegistry` 唯一扩展契约；`.zui` loader/映射由 `ui_document_importer` runtime plugin 注册；asset 内旧 builtin `.zui` backend/转换 owner 硬删除；asset DTO wrapper 改用本域 local codec/reference helper；删除 asset→ui 引用与 lib.rs 顺序注释。

测试阶段（按 `docs/plans/milestone-validation-policy.md` §3 最小批次）：
- `cargo check -p zircon_runtime --lib --locked` + focused 过滤词批：`cargo test -p zircon_runtime --lib --locked asset loader importer`（loader 注册/解析单测 + 变更面 asset 回归）；全量 lib 回归留给波次收口（policy §4）；
- 验收证据：`grep -r "use crate::ui" zircon_runtime/src/asset/` 为空。

### M3 S1：共享文本服务契约

实现切片：framework::text 契约（shaping 请求/字形度量/render mode）；graphics 侧改走契约；ui/text 实现挂注册。实现搬迁到独立域（zr_text 目录形态）可与计划 01 M3 合批。

测试阶段（policy §3 最小批次）：
- focused 过滤词批：`cargo test -p zircon_runtime --lib --locked text shaping glyph`（graphics/text 与 ui/text 两侧变更面）；全量 lib 回归留给波次收口；
- 渲染冒烟：editor-host 启动含文本场景截屏对比；
- 验收证据：`grep -r "use crate::ui" zircon_runtime/src/graphics/` 为空。

### M4 S3/S4：graphics↔scene 与 manager 面收口

实现切片：35 处 scene 引用逐条归类（extract DTO 可保留的、需要换句柄的、违规触 ECS 内部的），违规项改契约；manager 面跨域裸类型复核清零。

测试阶段（policy §3 最小批次）：
- focused 过滤词批：`cargo test -p zircon_runtime --lib --locked graphics scene extract` + editor/runtime 双启动冒烟；全量 lib 回归留给波次收口；
- 验收证据：引用矩阵复测——graphics 对 scene 的引用只剩 framework 契约路径与公开句柄；守卫脚本（计划 06）就位后此矩阵成为常驻断言。

## 5. 风险与回退

- **性能面**：extract/句柄间接层不得引入每帧堆分配或动态派发热点；契约签名评审时对热路径（文本 shaping、extract）要求零成本抽象（泛型/静态派发或批量 DTO）。
- **契约过度设计**：只为已存在的接缝立契约，不预铺"未来可能"的接口（架构深度测试以现有两个消费方为准）。
- **与 render/ui 计划集撞车**：S1/S3 的语义口径以 render 计划集为准，本计划只负责"引用形态合规"，不改渲染行为；动手前在对应计划 index 勾稽一行。

## 6. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

- fixed 已修复：[milestone-finalize-per-path-blob-verification-stall](05/fixed-2026-07-15-milestone-finalize-per-path-blob-verification-stall.md)
- fixed 已修复：[text-physical-owner-hard-cut-compile-break](../text/03/fixed-2026-07-15-text-physical-owner-hard-cut-compile-break.md)
- open 已移交 Text03：[text-layout-ellipsis-paragraph-regressions](../text/03/failure-2026-07-15-text-layout-ellipsis-paragraph-regressions.md)
- fixed 已修复：[ai-runtime-root-lockfile-drift](05/fixed-2026-07-16-ai-runtime-root-lockfile-drift.md)
- fixed 已修复：[text-hard-cut-runtime-consumer-type-drift](../runtime/04/fixed-2026-07-15-text-hard-cut-runtime-consumer-type-drift.md)
- fixed 已修复：[rich-text-dto-render18-retest-gate](../render/18/fixed-2026-07-15-rich-text-dto-render18-retest-gate.md)
- fixed 已修复：[rich-text-dto-shader06-source-staging-gate](../shader/06/fixed-2026-07-15-rich-text-dto-shader06-source-staging-gate.md)

已回传 Failure 概述：[`core/contracts` 反向依赖上层域与 facade](01/fixed-2026-07-13-core-contract-reverse-dependencies.md)（Frameworks01 M0 发现；current-source 禁止边清零，受管 Windows 编译与核心行为门已通过）；[runtime-profile-id-consumer-cutover](../../zircon_editor/editor/09/fixed-2026-07-13-runtime-profile-id-consumer-cutover.md)；[level-manager-export-cutover-incomplete](../runtime/02/fixed-2026-07-14-level-manager-export-cutover-incomplete.md)；[level-manager-name-core-error-import-drift](../../zircon_editor/editor/02/fixed-2026-07-14-level-manager-name-core-error-import-drift.md)；[project-asset-manager-access-test-consumer-drift](../../zircon_editor/editor/02/fixed-2026-07-14-project-asset-manager-access-test-consumer-drift.md)；[editor-retained-host-manager-resolver-consumer-drift](../../zircon_editor/editor/02/fixed-2026-07-14-editor-retained-host-manager-resolver-consumer-drift.md)；[manager-service-reactivation-lifecycle](../runtime/15/fixed-2026-07-14-manager-service-reactivation-lifecycle.md)；[ui-text-project-asset-manager-access-consumer-drift](../runtime/15/fixed-2026-07-14-ui-text-project-asset-manager-access-consumer-drift.md)；[ui-text-manager-access-cross-frame-retention](../runtime/15/fixed-2026-07-14-ui-text-manager-access-cross-frame-retention.md)。
- 产出记录：[`05/2026-07-10-subsystem-decoupling-contracts-output-records.md`](05/2026-07-10-subsystem-decoupling-contracts-output-records.md)
- M3 当前状态：`completed`。共享文本中立契约、单一 `zircon_runtime::text` implementation owner、旧 owner 物理删除、canonical UI/prewarm/fallback 链、generation-aware font/SDF invalidation、graphics-only neutral transport 与所有 production consumer 已完成硬切。fresh current-source 证据包括 static 13/13、default/target-server/graphics-only production exit 0、neutral transport 5/5、source-manifest 5/5、font-asset 4/4、default/UI integration 6/6、graphics-only upward 28/28，以及 multilingual GPU exporter 1/1；真实 1080x2000 framebuffer 位于 `docs/tests/runtime/text` 且受管 target PNG 计数为 0。独立复审 **Critical 0 / Important 0 / Minor 0**，最终 543-path manifest 已冻结。详细证据见 [`05/2026-07-15-m3-shared-text-service-contract-closeout.md`](05/2026-07-15-m3-shared-text-service-contract-closeout.md)。M4/M5、Text03/Shader06 外部行为待办和整份计划仍未完成。
- 当前状态：M1 已完成；M2 S2/S5 代码与静态门已完成，asset→ui 3→0，旧 builtin `.zui` owner 和声明顺序注释均已删除；历史 focused/package 验证由编号归档持有，完整 Runtime 门仍 pending。M3 已删除无 production 调用者的 graphics→`PublicRuntimeFrame` 转换 owner，不保留 UI 反向 shim，graphics→ui 由 1 收敛为 0。M4 的 dependency audit 已修复测试 owner 与分组导入误判，graphics→scene 的真实接缝由统计值 13 收敛为 1；2026-07-14 完成 versioned manager identity、registration adapter、lifecycle stale/unloaded、single-flight、跨线程依赖环与 Immediate activation reentry 约束，以及 asset/graphics/editor use-point access 硬切，删除 14 组旧 Arc holder、asset 旧 resolver 与 `IntoProjectAssetManagerAccess` 隐式 Arc adapter。Frameworks05 lib-test consumer failure 已按协调器流程 fixed 回传 Editor02，真实 CoreRuntime 测试 owner 覆盖 framework/renderer/streamer，不恢复兼容转换；静态门 22/22、Immediate integration 1/1，最终独立 review Critical=0、Important=0，因此 M4 manager hard-cut owner slice 已完成并进入 coordinator milestone 精确提交。完整 Runtime/lib-test 仍分别被 Shader06/Render18 活动错误和外部 `host_modules.rs` 缺失阻断，不冒充全包通过。Frameworks01 layer audit 的旧口径 56 条移交已完成十八类最低层硬切，最终 current-source matrix 为 2,290 / 72，reverse-layer=0、facade-inbound=0，总违规 0。计划 05 的其他 M4/M5 范围（模块依赖名称中立 owner、ui→graphics 文本服务与全工作区验证）仍独立 pending，不能用本切片冒充整份计划完成。
2026-07-14 M4 correction 概述：单模块与批量 activation 已按完整 `ModuleEntry::service_names` 恢复 `Unloaded` slot，当前状态为 `frameworks_05_m4_manager_service_reactivation_lifecycle_current_source_passed`。UI text 的初次 constructor 类型修复经独立 review 发现长期 text owner 仍跨帧保存 concrete manager；现已硬切为跨帧只存 versioned access、构造与每帧 use point 各解析一次，专门守卫、24/24 layer suite 与 fresh managed default-feature Runtime build 通过，Failure 已原子回传，状态为 `frameworks_05_ui_text_manager_access_cross_frame_retention_fixed_returned_re_review_pending`。两项工作均不改变其余 M3/M4/M5 pending 范围，详细证据由 `05/` 编号归档持有。
- fixed 已修复：[manager-resolver-weak-core-test-consumer-drift](../../zircon_editor/editor/03/fixed-2026-07-15-manager-resolver-weak-core-test-consumer-drift.md)

2026-07-16 current-source 恢复补录：Text owner 已补齐 `ScreenSpaceUiNativePrepareReport::font_faces_changed` 两处测试夹具，并完成 font-face generation rollover 后的 SDF atlas 全页 dirty-until-upload 语义；独立复审最终为 **Critical 0 / Important 0 / Minor 0**。fresh `python -m unittest tools.tests.test_frameworks_05_text_boundary -v` 为 **13/13**。Windows managed job `70627811c1204085a79ca1ef08772262` 顺序通过 default production 与 target-server 编译，graphics-only production job `ebf608cbe64f414797afe4edf9511bf5` 也以 exit 0 release。fresh multilingual GPU job `294971bdfc37467c80858318c6e4edfd` 通过 exact exporter **1/1**；docs PNG 已 fresh 写回并完成尺寸、哈希、target 零副本与目检。最终 source-manifest 5/5、font-asset 4/4、default/UI integration 6/6 与 graphics-only upward 28/28 均受管通过，M3 不再受 Runtime15 外部 test-anchor 漂移阻塞。

2026-07-16 M3 收口续录：Runtime15 archive-anchor Failure 已 fixed return，五处 active anchor 归零；managed exact job `eaebe4e27b7c4f6ab267a512c0854a2b` 为 **1/1**、released / exit 0。Frameworks05 current-source neutral transport job `139ecf26f34b4a578edf4cbb98bec8fe` 为 **5/5**、released / exit 0；最终 `773e431acb27467694f41861660ad0d4` graphics-only upward **28/28**、`e306fe34652c49c58edd0f1c59976418` default/UI **6/6**、`e1fdd58a751144f4a319787038c2f1f3` source-manifest **5/5**、`c3b71610d86f494bbcd40691d6fd32a5` font-asset **4/4** 均通过。M3 已完成并冻结 543-path exact manifest；M4/M5 与 Text03/Shader06 外部行为范围不随本切片提升。
