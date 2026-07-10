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

## 2. 现状接缝清单（证据）

| # | 接缝 | 现状 | 目标契约 |
|---|------|------|---------|
| S1 | graphics ↔ ui 文本 | 当前 production 基线 graphics→ui 4、ui→graphics 19；graphics 调 UI shaper/render-mode，UI 反向调用 `graphics::text::{font,layout,shaping}` | 共享文本服务契约（shaping/字形图集/render mode）进 framework::text；实现下沉为 `zr_text`（勾稽 render/14 的"shaping/字形图集下沉共享"既定方向） |
| S2 | asset → ui 模板 loader | M2 已从 3 清零；`.zui` backend/注册归 `ui_document_importer`，asset 仅保留 DTO wrapper、local codec/reference helper 与通用 `AssetImporterRegistry` | `AssetImporterRegistry` 是唯一 loader 扩展契约；UI 文档插件通过 runtime extension registry 注册 `.zui`，asset 域不再拥有/调用 UI loader 实现 |
| S3 | graphics → scene | 当前 production direct-reference 基线 13；逐行证据见 M1 JSON（旧抽查的 35 包含已迁移/测试 owner） | 已有 extract packet 契约的扩面：graphics 只消费 framework::render 的 extract/snapshot DTO 与 scene 句柄，禁止触 ECS 内部类型；与 render 计划集"extract 即 proxy 快照"口径一致 |
| S4 | 各域 → core/manager 具名服务 | resolver/handle 模式已存在，但域实现类型仍在部分调用点裸露 | 复核 manager 面：跨域访问一律 `*Handle` + resolver，句柄携带 index+version（godot RID 纪律），禁止跨域持有 `Arc<具体类型>` |
| S5 | lib.rs 声明顺序耦合 | "ui must be declared before asset" 注释 | S2 完成后该顺序约束自然消失，删除注释并加守卫（声明顺序不再承载语义） |

M1 全量扫描已落地；M2 更新后的 [`05/baselines/2026-07-10-runtime-domain-dependencies.json`](05/baselines/2026-07-10-runtime-domain-dependencies.json) 持有 2401 条生产逐行证据与 79-edge 矩阵，asset→ui 已从 3 降到 0；[`05/baselines/2026-07-10-contract-signatures.md`](05/baselines/2026-07-10-contract-signatures.md) 锁定 S1–S4 签名和计数验收规则。后续切片必须用同一脚本复测。

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

测试阶段：
- `cargo check -p zircon_runtime --lib --locked`、`cargo test -p zircon_runtime --lib --locked`（loader 注册/解析单测 + 现有 asset 测试零回归）；
- 验收证据：`grep -r "use crate::ui" zircon_runtime/src/asset/` 为空。

### M3 S1：共享文本服务契约

实现切片：framework::text 契约（shaping 请求/字形度量/render mode）；graphics 侧改走契约；ui/text 实现挂注册。实现搬迁到独立域（zr_text 目录形态）可与计划 01 M3 合批。

测试阶段：
- `cargo test -p zircon_runtime --lib --locked`（文本渲染现有测试全量，graphics/text 与 ui/text 两侧）；
- 渲染冒烟：editor-host 启动含文本场景截屏对比；
- 验收证据：`grep -r "use crate::ui" zircon_runtime/src/graphics/` 为空。

### M4 S3/S4：graphics↔scene 与 manager 面收口

实现切片：35 处 scene 引用逐条归类（extract DTO 可保留的、需要换句柄的、违规触 ECS 内部的），违规项改契约；manager 面跨域裸类型复核清零。

测试阶段：
- `cargo test -p zircon_runtime --lib --locked` + editor/runtime 双启动冒烟；
- 验收证据：引用矩阵复测——graphics 对 scene 的引用只剩 framework 契约路径与公开句柄；守卫脚本（计划 06）就位后此矩阵成为常驻断言。

## 5. 风险与回退

- **性能面**：extract/句柄间接层不得引入每帧堆分配或动态派发热点；契约签名评审时对热路径（文本 shaping、extract）要求零成本抽象（泛型/静态派发或批量 DTO）。
- **契约过度设计**：只为已存在的接缝立契约，不预铺"未来可能"的接口（架构深度测试以现有两个消费方为准）。
- **与 render/ui 计划集撞车**：S1/S3 的语义口径以 render 计划集为准，本计划只负责"引用形态合规"，不改渲染行为；动手前在对应计划 index 勾稽一行。

## 6. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

- 产出记录：[`05/2026-07-10-subsystem-decoupling-contracts-output-records.md`](05/2026-07-10-subsystem-decoupling-contracts-output-records.md)
- 当前状态：M1 已完成；M2 S2/S5 代码与静态门已完成，asset→ui 3→0、生产扫描为 2401 references / 79 domain edges，旧 builtin `.zui` owner 和声明顺序注释均已删除。focused Rust 首轮 16/18 后修复两个 fixture；2026-07-11 默认 feature lib-test binary 精确执行 `asset::tests::assets::ui` 为 18/18、0 failed、7433 filtered；完整 `cargo check -p zircon_runtime --lib --locked` 通过（5m24s，418 warnings）。后续 `asset::tests` 扩展回归暴露字体 artifact 直接复用 authoring skip 形状的 `UnexpectedEof`，现已硬切为独立完整字段 cache DTO；公开磁盘往返 contract 1/1 通过，`core-min` production lib check 4m59s 通过。当前默认 lib-test 被活动 rich-text 重构的 6 个外部编译错误阻断，修复后 asset 全组与计划规定的全量 Runtime lib-test 仍 pending，因此 M2 只标记 package check + UI/font focused validation passed；M3–M4 仍 pending，不声明计划 05 完成。
