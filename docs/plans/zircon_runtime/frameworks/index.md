---
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/Cargo.toml
  - Cargo.toml
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/engine_module/engine_module.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/plugin/mod.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_app/src/plugins/groups.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/plugin_sdk/src/lib.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
plan_sources:
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - docs/plans/engine-code-structure-convention.md
  - docs/engine-architecture/workspace-root-rules-and-hard-cutover.md
  - docs/plans/zircon_runtime/runtime/index.md
---

# Zircon 引擎整体框架与工程化组织总体计划（frameworks 计划集）

本目录是 ZirconEngine 整体组织架构的权威框架计划集。它回答一个问题：**在保持公开三包形态（`zircon_app` / `zircon_runtime` / `zircon_editor`）与既有收束规则的前提下，如何把当前的半成品引擎组织形态推进到"开发者友好、插件开发友好、核心简洁、工程化精细、可维护、鲁棒、编译快速、功能高度解耦"的最终框架形态**，并把开发规范固化为可执行的守卫机制。

与既有计划集的分工与优先级：

- `docs/plans/zircon_runtime/runtime/`：子系统语义级对齐（调度、asset、ECS、UI、脚本等）的权威。本计划集不重复其内容；两者交叠处（runtime/01 依赖治理、runtime/02 core 脊柱、runtime/06 插件面、runtime/15 结构规范），**宏观组织决策（crate 拓扑、feature 矩阵、DX 工具链、守卫机制）以本目录为准，子系统内部语义以 runtime/ 计划集为准**。
- `docs/plans/zircon_runtime/render|shader|text/`：渲染域权威计划集，本计划集只约束其所在 crate/feature 的组织边界，不触碰渲染语义。
- `.codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md` 与 `全系统重构方案.md`：三包公开形态、core 脊柱角色、Scene/Editor 边界仍是绑定规则。本计划集在其之上做**内部组织的演进**，其中"runtime 内部 crate 化"（计划 01）是对"单 crate 吸收层"实现方式的一次显式修订，理由与决策记录见下文 §3 D1。

## 1. 现状评审结论（2026-07-02 实查）

调研口径：`zircon_runtime/src` 全量结构盘点、`zircon_plugins`/`zircon_runtime_interface`/`zircon_app` 插件链路盘点、`dev/` 参考引擎（bevy/Fyrox/godot/UnrealEngine/Graphics/Piccolo）结构对照。

### 1.1 规模画像

| 模块 | 职责 | 行数（含测试） |
|------|------|------|
| ui | 布局/模板/表面渲染/事件/无障碍 | ~327K |
| graphics | 渲染内核/材质/GPU 场景 | ~322K |
| core | 内核脊柱（runtime/framework/manager/math/resource） | ~148K |
| scene | ECS 世界/层级/序列化 | ~137K |
| asset | 资产管线/导入/工件缓存 | ~110K |
| plugin | 插件加载/bridge/扩展注册 | ~64K |
| script/rhi/rhi_wgpu/dynamic_api/platform/render_graph/animation/input/其余 | — | 合计 ~100K |

`zircon_runtime` 是**约 120 万行的单 crate**（`rlib` + `cdylib`），是全仓编译时间与增量迭代速度的最大瓶颈；任何一处修改都触发整个 crate 重编。

### 1.2 主要问题清单

| # | 问题 | 证据 | 承接计划 |
|---|------|------|---------|
| P1 | 单 crate 巨石：120 万行单编译单元，无法并行编译、无法按域增量 | `zircon_runtime/src`、`zircon_runtime/Cargo.toml` | 01 |
| P2 | 模块边界靠纪律而非编译器：`lib.rs` 模块声明顺序敏感（"ui must be declared before asset"）、graphics 35 处直接 `use crate::scene::`、graphics 直接引用 `crate::ui::text::shaper` | `zircon_runtime/src/lib.rs`、`src/graphics/scene/scene_renderer/ui/text.rs` | 01、05 |
| P3 | feature 门控不完整：animation/navigation/script/diagnostic_log 及 `core/framework` 的 ai/physics/sound/net 子域无条件编译，server/headless 目标携带无用代码 | `zircon_runtime/src/lib.rs`、`zircon_runtime/Cargo.toml` features 段 | 03 |
| P4 | 模块生命周期语义偏薄：五态生命周期无 bevy 式 `ready/finish` 异步就绪语义，无 godot/UE 式初始化层级（Core→Servers→Scene→Editor→Post），模块排序靠 `builtin/runtime_modules` 手工列表 | `src/core/runtime/lifecycle.rs`、`src/builtin/runtime_modules/core_modules.rs` | 02 |
| P5 | 插件 DX 债务：PLUGIN_ID 等元数据三处重复声明（plugin.toml / capability.rs / dist lib.rs）、新插件需 touch ~11 个文件、无脚手架、manifest 校验只在 Python 审计脚本、加载失败诊断贫弱、native 热重载 save/restore 未实装 | `zircon_plugins/gltf_importer/**`、`zircon_plugins/first_party_runtime_catalog/src/lib.rs` | 04 |
| P6 | 规范散落且部分不可执行：结构规范分散在 convention 文档 + 技能 + 审计脚本，CI 无 clippy、无依赖方向守卫、无 feature 组合验证 | `.github/workflows/ci.yml`、`docs/plans/engine-code-structure-convention.md` | 06 |
| P7 | 开发期链接慢：无 bevy_dylib/fyrox-dylib 式 `dynamic_linking` 开发模式 | 对照 `dev/bevy/crates/bevy_dylib`、`dev/Fyrox/fyrox-dylib` | 01 |

同时确认的**健康面**（保持，不推倒）：core 脊柱五分角色清晰、无循环依赖、生产文件全部低于 1000 行门槛（large-file gate 已 clear）、native 插件 ABI v3 有版本化与能力协商、profile 六态与 feature bundle 规则已成文。

## 2. 参考引擎结论（对齐 zr-reference-engine-routing）

| 参考 | 采纳的模式 | 锚点 |
|------|-----------|------|
| bevy | 分层多 crate + 收集器 crate（bevy_internal）；Plugin 四阶段生命周期 build/ready/finish/cleanup；三层 feature 激活链（根→internal→子 crate）；`dynamic_linking` 开发期动态链接 | `dev/bevy/crates/bevy_app/src/plugin.rs`、`dev/bevy/crates/bevy_internal`、`dev/bevy/crates/bevy_dylib`、`dev/bevy/Cargo.toml` |
| Fyrox | Static/Dynamic 双形态插件容器与热重载三步（prepare_to_reload→reload→on_loaded）；editor plugin 与 game plugin 分离 | `dev/Fyrox/fyrox-impl/src/plugin/mod.rs`、`dev/Fyrox/editor/src/plugin.rs`、`dev/Fyrox/fyrox-dylib` |
| godot | core→servers→scene→editor 单向分层；模块初始化四层级；RID/handle 解耦 scene 与后端 | `dev/godot/modules/gdscript/config.py`、`dev/godot/core/templates/rid.h`、`dev/godot/servers/rendering/rendering_server.h` |
| UnrealEngine | Runtime/Developer/Editor/Programs 四分层；模块 Public/Private 依赖可见性；插件 LoadingPhase | `dev/UnrealEngine/Engine/Source/Runtime/AIModule/AIModule.Build.cs`、`dev/UnrealEngine/Engine/Plugins/Bridge/Bridge.uplugin` |
| Graphics(Unity) | core RP 包与具体管线包分层、包级依赖声明 | `dev/Graphics/Packages/com.unity.render-pipelines.core` |
| Piccolo | 仅作最小启动流程对照，不采纳其无阶段初始化 | `dev/Piccolo/engine/source/runtime/engine.h` |

明确不抄：Godot 自研脚本语言、UE 的 .Build.cs 元构建（Cargo features 足够）、Unity asmdef 全套（Rust crate 天然隔离）、Fyrox 一体化 fyrox-impl 巨石（正是要避免的形态）。

## 3. 目标架构与决策记录

### D1（本计划集核心决策）：公开三包不变，`zircon_runtime` 内部 crate 化

公开架构继续是 `zircon_app`（组合与启动）/ `zircon_runtime`（引擎实现）/ `zircon_editor`（作者态）/ `zircon_runtime_interface`（稳定 ABI）。变化在 `zircon_runtime` 的实现方式：从"单 crate 吸收一切"演进为 **"门面 crate + 内部分层成员 crate"**（bevy_internal 收集器模式）：

```
根 workspace
├── zircon_app / zircon_editor / zircon_runtime_interface / zircon_hub   （不变）
├── zircon_runtime                    ← 门面 + 组装 crate（保持 rlib+cdylib、dynamic_api、
│                                        builtin 模块组装、curated re-export；对外路径
│                                        zircon_runtime::* 不变）
└── zircon_runtime/crates/            ← 内部成员 crate（不发布、无独立稳定性承诺）
    ├── zr_kernel        ← core/runtime + engine_module：生命周期/调度/描述符（零重依赖）
    ├── zr_contracts     ← core/framework：纯契约 trait/DTO（按域 feature 门控）
    ├── zr_math / zr_resource
    ├── zr_platform / zr_input / zr_diagnostics
    ├── zr_asset / zr_scene
    ├── zr_rhi / zr_rhi_wgpu / zr_render_graph
    ├── zr_graphics
    ├── zr_ui / zr_text
    └── zr_script / zr_animation / zr_navigation   （全部可选）
```

依赖方向自下而上单向（kernel/contracts → 中层 → graphics/ui → 门面），由 Cargo 强制，替代今天靠 `lib.rs` 声明顺序与纪律维持的边界（P2）。这是对收束计划中"runtime 物理吸收为单 crate"实现细节的显式修订：**吸收层语义不变（外部只见 `zircon_runtime`），物理编译单元分层**。修订理由：120 万行单编译单元已实测成为迭代瓶颈，且 bevy/Fyrox/godot/UE 无一例外采用分层编译单元。命名前缀 `zr_` 与最终清单在计划 01 M0 批准后锁定。

### D2：模块生命周期统一为"描述符 + 四阶段 + 初始化层级"

`EngineModule`/`RuntimePlugin` 收敛到统一内核语义：保留五态生命周期与 Driver/Manager 依赖规则，补齐 bevy 式 `build/ready/finish/cleanup` 四阶段（支持 GPU 上下文等异步就绪）与 godot/UE 式初始化层级 `InitLevel::{Kernel, Servers, Scene, Editor, Post}`，`builtin/runtime_modules` 的手工排序列表退役为"按层级 + 声明依赖自动排序"。详见计划 02。

### D3：feature 矩阵单源，profile 即 feature 预设

所有可选子系统（animation/navigation/script/ui/graphics 相关域、framework 各契约域）都有 feature；`RuntimeProfileId` 六 profile 映射为 feature 预设单源；CI 验证关键 feature 组合。详见计划 03。

### D4：插件元数据单源 + SDK 工具链

插件声明收敛为单一 Rust 源（proc-macro 生成 plugin.toml 常量镜像与导出符号），脚手架 `cargo zircon plugin new`、manifest 校验器、加载诊断增强、热重载 harness。详见计划 04。

### D5：跨域接缝全部走契约

graphics↔ui（文本 shaper）、asset↔ui（模板 loader）、graphics↔scene（extract packet）等直接引用改为 `zr_contracts` 契约 + handle/registry。这是 D1 第三阶段拆分的前置。详见计划 05。

### D6：规范即守卫

开发规范从"文档 + 自觉"升级为"单一规范文档 + CI 守卫测试 + 审计脚本入 CI"。详见计划 06。

## 4. 子计划地图与执行顺序

| 计划 | 文档 | 主题 | 依赖 |
|------|------|------|------|
| 01 | `01-runtime-crate-decomposition.md` | runtime 内部 crate 化、编译速度、依赖治理、dynamic_linking | 02/03 先行；Phase 3 依赖 05 |
| 02 | `02-module-kernel-and-lifecycle-unification.md` | 内核生命周期/初始化层级/描述符单源 | 无（最先） |
| 03 | `03-optional-features-and-profile-matrix.md` | feature 矩阵、可选子系统、profile 预设、组合 CI | 02 |
| 04 | `04-plugin-dx-and-sdk-toolchain.md` | 插件元数据单源、脚手架、诊断、热重载 | 02；与 01 并行 |
| 05 | `05-subsystem-decoupling-contracts.md` | 跨域接缝契约化（graphics/ui/asset/scene） | 02；是 01 Phase 3 前置 |
| 06 | `06-development-conventions-and-guardrails.md` | 开发规范总纲与守卫机制 | 无（规范先行，守卫随各阶段落地） |

配套权威文档（规范性，非计划）：

| 文档 | 内容 |
|------|------|
| `architecture-overview.md` | 目标架构示意图集：总体三包形态、runtime 内部 crate 分层、启动/生命周期时序、帧循环数据流、插件双路径、editor↔runtime 权威边界。图中层次与箭头方向是规范性约束 |
| `development-conventions.md` | 《Zircon 开发规范准则》总纲：GEN 通用（结构/迁移/质量/测试/文档）+ RT/ED/PL/IF/WF 分域规则，每条 MUST 勾稽守卫 G1–G7。计划 06 M0 的交付物，先行生效 |

阶段划分：

- **阶段 0（基线与批准）**：编译时间基线（`cargo build --timings`）、依赖图快照、feature 现状矩阵；批准 D1 命名与 crate 清单。归计划 01 M0 与 06 M0。
- **阶段 A（内核与开关，单 crate 内完成）**：02 全部 → 03 全部。先把生命周期与 feature 矩阵理顺，拆分时才不必二次返工。
- **阶段 B（接缝与拆分）**：05 接缝契约化 → 01 Phase 1（kernel/contracts/math/resource）→ Phase 2（rhi/render_graph/platform/input/asset/scene）→ Phase 3（graphics/ui/text/可选域）。每 Phase 硬切换，无兼容桥。
- **阶段 C（DX 与守卫收口）**：04 全部；06 守卫全部入 CI。可与阶段 B 后半并行。

## 5. 全局边界约束（各子计划必须遵守）

1. 公开三包 + `zircon_runtime_interface` 形态不变；`zircon_runtime::*` 对 app/editor/plugins 的公开路径不变（门面 curated re-export 是结构性而非迁移性，不违反 hard-cutover 规则）。
2. 内部 crate 不对外发布、不被 `zircon_app`/`zircon_editor`/插件直接依赖；越过门面直接依赖 `zr_*` 视为架构违规，由计划 06 守卫拦截。
3. 动态边界继续只传 ABI 安全值与序列化载荷（`zircon_runtime_interface` 规则不变）。
4. 迁移一律硬切换：调用方同批迁移，禁止 `legacy/compat/shim` 与迁移语境 bridge；非网络语义禁用 `server` 命名。
5. 根文件（lib.rs/mod.rs/main.rs）保持薄；1000 行生产文件门槛维持；docs 源路径镜像规则维持。
6. 每个里程碑遵循 milestone-first 政策：实现切片不逐片编译，测试阶段统一跑声明的命令集（见各子计划"测试阶段"）。
7. 与 `runtime/`、`render/` 等计划集冲突时：组织与工程化条目以本目录为准，子系统语义以对应计划集为准；发现实质冲突先回写双方 index 勾稽，再动代码。

## 6. 验收总纲

- 编译速度：以阶段 0 基线为准，阶段 B 完成后"单域修改的增量 check 时间"（如只改 ui）目标下降 ≥50%；全量冷构建不劣化超过 10%。
- 解耦：`cargo check -p zircon_runtime --no-default-features --features target-server` 不编译 ui/graphics/animation/navigation 任何代码；六 profile 各有可编译的最小 feature 组合并入 CI。
- 插件 DX："新建一个最小 runtime 插件"从 touch ~11 个文件降到脚手架一条命令 + 填一个 Rust 声明文件；元数据零重复声明。
- 鲁棒性：插件加载/manifest 校验失败信息含错误码、期望符号/能力、修复提示；内核生命周期含依赖环、缺失依赖、重复注册的类型化错误与测试。
- 规范：06 的守卫清单全部有对应 CI 步骤或守卫测试，规范文档单源且与守卫一一勾稽。
