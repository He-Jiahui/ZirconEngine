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

## 状态与产出记录迁移说明

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

Frameworks 总索引只保留计划集当前现状、架构决策与子计划路由；逐条镜像记录改由 Frameworks 02 子计划目录维护。

- 迁入记录：[`02/2026-07-09-index-output-records.md`](02/2026-07-09-index-output-records.md)

## 1. 现状评审结论（2026-07-30 current-source successor）

调研口径：`zircon_runtime/src` 全量结构盘点、`zircon_plugins`/`zircon_runtime_interface`/`zircon_app` 插件链路盘点、`dev/` 参考引擎（bevy/Fyrox/godot/UnrealEngine/Graphics/Piccolo）结构对照。

### 1.1 规模画像

| 模块 | 职责 | 行数（含测试） |
|------|------|------|
| tests | Runtime 吸收、结构、产品与集成回归 | 250,951 |
| graphics | 渲染内核/材质/GPU 场景 | 213,938 |
| ui | 布局/模板/表面渲染/事件/无障碍 | 180,794 |
| core | 内核脊柱（runtime/framework/manager/math/resource） | 97,373 |
| asset | 资产管线/导入/工件缓存 | 79,325 |
| scene | ECS 世界/层级/序列化 | 78,762 |
| plugin | 插件加载/bridge/扩展注册 | 49,790 |
| text | 共享 shaping/layout/raster/atlas 服务 | 45,019 |
| script/rhi/rhi_wgpu/dynamic_api/platform/render_graph/animation/input/其余 | — | 合计 79,213 |

`zircon_runtime/src` 的 2026-07-30 原子快照共有 9,188 个 Rust 文件 / 1,075,165 行（含测试），输入 pre/post 指纹同为 `348fe59c72c798e5e64babb5489910f30dbd00a58441ec11c1e176826519339e`；它们仍由同一 `rlib` + `cdylib` package 编译，是全仓编译时间与增量迭代速度的最大瓶颈。该计数只约束这一不可变时点，物理迁移前必须重算。

### 1.2 主要问题清单

| # | 问题 | 证据 | 承接计划 |
|---|------|------|---------|
| P1 | 单 crate 巨石：1,075,165 行 current Rust source 仍是单编译单元，无法按域并行/增量编译 | `zircon_runtime/src`、`zircon_runtime/Cargo.toml` | 01 |
| P2 | 历史声明顺序、graphics→scene 与 graphics→ui 直接边已硬切清零；current crate-DAG 阻断收敛为 `asset→text=2`、`scene→animation=2`、`rhi→rhi_wgpu=1`，另有 source-cubemap projection 两处测试反向构造 concrete Runtime `TaskPool` | `01/baselines/2026-07-30-runtime-domain-dependencies-production-only.json`、`src/core/framework/render/environment/source_cubemap/tests/projection.rs` | 01、05 |
| P3 | 域 feature/cfg 与 profile/domain CI source matrix 已落地；真实剩余是运行期 module/plugin selection 仍手写在 `runtime_profile/defaults.rs`，尚未与 feature preset TOML 单源生成，current-main acceptance 也 pending | `zircon_runtime/runtime-feature-presets.toml`、`src/plugin/runtime_profile/defaults.rs` | 03 |
| P4 | 四阶段 lifecycle、InitLevel、descriptor 与统一 sorter spine 已落地；Minimal 等生产组装仍有独立构造/选择路径，真实 readiness signal、SDK/native/managed consumers 与 current managed acceptance 未闭合 | `src/core/runtime/lifecycle.rs`、`src/builtin/runtime_modules`、`zircon_app/src/plugins` | 02 |
| P5 | runtime `declare_plugin!`、generated manifest parity、typed `PluginLoadError` 与 live-host/fixture reload 已落地；dist ABI identity/capability/symbol 仍手写，Rust `cargo-zircon` 三命令不存在，gltf importer reload callbacks 仍为空 | `zircon_plugins/plugin_sdk/src/declaration.rs`、`zircon_plugins/gltf_importer/dist/src/lib.rs`、`zircon_runtime/src/plugin/native_plugin_loader` | 04 |
| P6 | 统一 convention runner、fmt、scoped clippy、docs、layering/structure 与 profile/domain matrix 已接 CI；全库 G7 仍 RED，G5 `cargo-zircon` 与 G6 cargo-deny 未落地，Runtime 全量 clippy/真实分支 acceptance 仍 pending | `.github/workflows/ci.yml`、`tools/check_conventions.py`、`docs/plans/engine-code-structure-convention.md` | 06 |
| P7 | 开发期链接慢：无 bevy_dylib/fyrox-dylib 式 `dynamic_linking` 开发模式 | 对照 `dev/bevy/crates/bevy_dylib`、`dev/Fyrox/fyrox-dylib` | 01 |

同时确认的**健康面**（保持，不推倒）：core 脊柱角色清晰、生产文件继续受 1000 行门禁约束、native 插件主 ABI v3 与 behavior ABI v4 均有版本化/能力协商、profile 六态与显式 feature preset 单源已成文。M1 不用推倒这些合同，只改变其物理编译 owner。

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
    ├── layer 0a  zr_math / zr_resource / zr_contracts（纯契约 trait/DTO，按域 feature 门控）
    ├── layer 0b  zr_kernel（core/runtime + engine_module；只依赖 0a）
    ├── layer 1   zr_diagnostics / zr_foundation / zr_platform / zr_input
    ├── layer 2   zr_asset / zr_scene
    ├── layer 3   zr_rhi / zr_rhi_wgpu / zr_render_graph / zr_operation / zr_text
    ├── layer 4   zr_graphics
    ├── layer 5   zr_ui
    └── optional  zr_script / zr_animation / zr_navigation
```

依赖方向自下而上单向，M1 固定为 `zr_math/zr_resource → zr_contracts → zr_kernel → zr_diagnostics`，再进入 foundation/platform/input 与中重域，最后到 graphics/ui 和门面；由 Cargo 强制，替代模块内纪律（P2）。`zr_operation` 是依赖 kernel/contracts/scene/interface 的 layer-3 supporting crate，optional navigation 可依赖它；不得把 operation 并入 kernel 或留在门面实现。`zr_text` 也是 layer-3，但只拥有 backend-neutral shaping/layout/font/source/SDF；GPU atlas/upload/draw 归 layer-4 `zr_graphics::text_backend` 或 `zr_rhi_wgpu`，禁止 `zr_text` 直接依赖完整 wgpu/naga/glyphon 或与 graphics 成环。这是对收束计划中"runtime 物理吸收为单 crate"实现细节的显式修订：**吸收层语义不变（外部只见 `zircon_runtime`），物理编译单元分层**。2026-07-30 M0 已锁定 `zr_` 命名、crate 清单与物理切片顺序；2026-07-31 又同步锁定 operation/text owner 边界。四份受管 cold/incremental timings 尚未生成，所以 M0 仍未完成。

依赖治理同步执行 stable-first。当前 `notify 9.0.0-rc.3`、`winit 0.31.0-beta.2`、
`zip 9.0.0-pre2` 只能以精确 package/version、owner、理由、到期日与 ticket 的限期例外存在；
到期未迁 stable 的例外保持 RED，禁止 wildcard、整类禁用或 advisory 静默。该例外机制不改变
`timings-pending / physical-migration-not-started` 状态，也不授权提前创建兼容 crate 或双轨 owner。

### D2：模块生命周期统一为"描述符 + 四阶段 + 初始化层级"

`EngineModule`/`RuntimePlugin` 收敛到统一内核语义：保留五态生命周期与 Driver/Manager 依赖规则，补齐 bevy 式 `build/ready/finish/cleanup` 四阶段（支持 GPU 上下文等异步就绪）与 `InitLevel::{Kernel, Services, Scene, Editor, Post}` 初始化层级，`builtin/runtime_modules` 的手工排序列表退役为"按层级 + 声明依赖自动排序"。非网络语义统一使用 `Services`，不恢复已退役的 `Servers` 变体。详见计划 02。

### D3：feature 矩阵单源，profile 即 feature 预设

所有可选子系统（animation/navigation/script/ui/graphics 相关域、framework 各契约域）都有 feature；`RuntimeProfileId` 六 profile 映射为 feature 预设单源；CI 验证关键 feature 组合。详见计划 03。

### D4：插件元数据单源 + SDK 工具链

插件声明收敛为单一 Rust 源（proc-macro 生成 plugin.toml 常量镜像与导出符号），脚手架 `cargo zircon plugin new`、manifest 校验器、加载诊断增强、热重载 harness。详见计划 04。

### D5：跨域接缝全部走契约

历史 asset→ui、graphics→ui、graphics→scene 生产直接边已硬切为 0；对应共享类型/服务继续由中立 contract + handle/registry 承接，不恢复旧 owner 或 facade shim。current successor 还必须处理 `asset→text`、`scene→animation` 与 `rhi→rhi_wgpu` 三条硬反向边，才可执行 D1 对应物理拆分。详见计划 01 current baseline 与计划 05。

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
- **阶段 B（接缝与拆分）**：05 接缝契约化 → 01 Phase 1（math/resource → contracts → kernel → diagnostics）→ Phase 2（foundation → platform/input → asset/scene → operation → rhi/rhi_wgpu/render_graph）→ Phase 3（backend-neutral text → graphics text backend/graphics → ui/可选域）。每 Phase 硬切换，无兼容桥；operation 必须在 scene 后、navigation 前完成，text CPU owner 必须先于 GPU backend 拆分。
- **阶段 C（DX 与守卫收口）**：04 全部；06 守卫全部入 CI。可与阶段 B 后半并行。

## 5. 全局边界约束（各子计划必须遵守）

1. 公开三包 + `zircon_runtime_interface` 形态不变；`zircon_runtime::*` 对 app/editor/plugins 的公开路径不变（门面 curated re-export 是结构性而非迁移性，不违反 hard-cutover 规则）。
2. 内部 crate 不对外发布、不被 `zircon_app`/`zircon_editor`/插件直接依赖；越过门面直接依赖 `zr_*` 视为架构违规，由计划 06 守卫拦截。
3. 动态边界继续只传 ABI 安全值与序列化载荷（`zircon_runtime_interface` 规则不变）。
4. 迁移一律硬切换：调用方同批迁移，禁止 `legacy/compat/shim` 与迁移语境 bridge；非网络语义禁用 `server` 命名。
5. 根文件（lib.rs/mod.rs/main.rs）保持薄；1000 行生产文件门槛维持；docs 源路径镜像规则维持。
6. 每个里程碑遵循 [`milestone-validation-policy.md`](../../milestone-validation-policy.md)：实现切片不逐片编译，里程碑测试阶段统一批量运行声明的命令集。
7. 与 `runtime/`、`render/` 等计划集冲突时：组织与工程化条目以本目录为准，子系统语义以对应计划集为准；发现实质冲突先回写双方 index 勾稽，再动代码。

## 6. 验收总纲

- 编译速度：以阶段 0 基线为准，阶段 B 完成后"单域修改的增量 check 时间"（如只改 ui）目标下降 ≥50%；全量冷构建不劣化超过 10%。
- 解耦：`cargo check -p zircon_runtime --no-default-features --features target-server` 不编译 ui/graphics/animation/navigation 任何代码；六 profile 各有可编译的最小 feature 组合并入 CI。
- 插件 DX："新建一个最小 runtime 插件"从 touch ~11 个文件降到脚手架一条命令 + 填一个 Rust 声明文件；元数据零重复声明。
- 鲁棒性：插件加载/manifest 校验失败信息含错误码、期望符号/能力、修复提示；内核生命周期含依赖环、缺失依赖、重复注册的类型化错误与测试。
- 规范：06 的守卫清单全部有对应 CI 步骤或守卫测试，规范文档单源且与守卫一一勾稽。

## 7. 当前状态镜像

Frameworks 当前实现状态、验证证据与未完成项统一记录在各编号子计划及其产出归档中；本总览只保留计划集现状与子计划路由。

## Code Review 收敛 (2026-07-31)

- P3–P6 已按 current source 从历史“全部缺失”改为已落地 spine 与真实剩余缺口，避免后续重复实现
  feature gates、lifecycle sorter、typed loader errors 或 CI convention entrypoint。
- 四份架构 authority 已同步 operation/text 完整物理顺序、stable-first 三个精确 prerelease 例外、
  `timings-pending / physical-migration-not-started`，且保持 facade curated-only、无 alias/shim/双轨。
