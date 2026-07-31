---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/engine_module/engine_module.rs
  - zircon_runtime/src/rhi/mod.rs
  - zircon_runtime/src/rhi_wgpu/mod.rs
  - zircon_runtime/src/render_graph/mod.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - Cargo.toml
plan_sources:
  - docs/plans/zircon_runtime/frameworks/index.md
  - docs/plans/zircon_runtime/runtime/01-tech-stack-and-dependency-governance.md
  - docs/engine-architecture/workspace-root-rules-and-hard-cutover.md
reference_engines:
  - dev/bevy/crates/bevy_internal
  - dev/bevy/crates/bevy_dylib
  - dev/bevy/Cargo.toml
  - dev/Fyrox/fyrox-dylib
---

# 01 · Runtime 内部 crate 化与编译速度治理

## 1. 目标

把 `zircon_runtime` 从 ~120 万行单编译单元重组为"门面 crate + 分层内部成员 crate"，在不改变任何对外路径（`zircon_runtime::*`）与三包公开形态的前提下，获得：

- 编译器强制的依赖方向（取代 `lib.rs` 声明顺序纪律与人肉边界审查）；
- 并行编译与按域增量编译（单域修改增量 check 时间目标下降 ≥50%）;
- 每个域独立的 feature 门控挂点（承接计划 03）；
- 开发期 `dynamic_linking` 快链模式（bevy_dylib / fyrox-dylib 模式）。

## 2. 现状与差距

- `zircon_runtime` 仍是 `rlib+cdylib` 单 crate；`lib.rs` 当前有 22 个 production 顶层 `mod` 声明（另有 `prelude` 与 `tests`），production dependency audit 覆盖 20 个域，尚未形成编译器强制的内部 crate 边界。
- 常驻 `operation` 域不是 kernel 子目录：它的 owner-thread apply 同时依赖 `CoreHandle` 与
  `scene::World`，并由 optional navigation 注册 handler。目标拓扑已将其锁定为 layer-3
  supporting crate `zr_operation`；不得并入 `zr_kernel` 形成 layer-0 到 scene 的反向依赖，
  也不得滞留门面成为 optional crate 反向依赖 facade 的入口。
- 旧声明顺序注释及 `graphics→scene`、`graphics→ui` 直接依赖已由 Frameworks05 硬切清零；当前最低结构缺口收敛为 `asset→text=2`、`scene→animation=2`、`rhi→rhi_wgpu=1` 三条硬反向边，见 M0 current-source 基线。禁止继续引用历史 35 处接缝作为迁移输入。
- 依赖治理已有文档（`runtime/01-tech-stack-and-dependency-governance.md`）但缺编译单元层面的强制手段。
- 无开发期动态链接选项；重型依赖（wgpu/naga/winit/gltf/image）与纯逻辑代码同一编译单元。

## 3. 目标拓扑与分层规则

```
layer 0a zr_math        core/math（继续薄转发 zircon_runtime_interface::math）
         zr_resource    core/resource
         zr_contracts   core/framework（纯 trait/DTO；按域 feature 门控子模块）
layer 0b zr_kernel      core/runtime + engine_module（生命周期/调度/描述符；依赖 0a，禁止重依赖）
layer 1  zr_diagnostics diagnostic_log
         zr_foundation  foundation（Kernel 级 config/event module 与共享原子持久化实现）
         zr_platform    platform（winit 等平台依赖收拢于此）
         zr_input       input
layer 2  zr_asset       asset          zr_scene   scene（ECS 世界）
layer 3  zr_rhi         rhi            zr_rhi_wgpu  rhi_wgpu（直接 wgpu 实现层）
         zr_render_graph render_graph  zr_operation operation（依赖 kernel + scene 的有界 owner-thread operation service）
         zr_text        graphics/text + ui/text 下沉的 CPU shaping/layout/font/SDF 服务（勾稽 render/14；禁止直接依赖 wgpu/naga/glyphon）
layer 4  zr_graphics    graphics（含 text GPU atlas/upload/draw backend；直接 wgpu/naga/glyphon 归此或 zr_rhi_wgpu）
layer 5  zr_ui          ui
optional zr_script / zr_animation / zr_navigation
facade   zircon_runtime 门面：builtin 组装、plugin 加载、dynamic_api、prelude、curated re-export、cdylib 出口
```

分层规则：

1. 只允许上层依赖下层；layer 0 的固定顺序为 `zr_math/zr_resource → zr_contracts → zr_kernel`。同层横向依赖必须经 `zr_contracts` 契约或显式批准并记录在本文件。
2. `core/manager` 的 handle/resolver 访问层留在门面 crate（它天然需要看到各域实现以组装 resolver），但其 trait/名字常量在 `zr_kernel`/`zr_contracts`。
3. 内部 crate 位于 `zircon_runtime/crates/`，`publish = false`，根 workspace members 收录；命名前缀 `zr_`（与 `zr_vm_rust_binding` 一致），M0 批准后锁定。
4. 门面 crate 的 re-export 是结构性 curated re-export（workspace-root-rules 允许项），不是迁移桥；内部 crate 之间禁止任何 re-export 兼容层。
5. `zircon_app`/`zircon_editor`/`zircon_plugins` 一律只依赖 `zircon_runtime` 门面与 `zircon_runtime_interface`，禁止直接依赖 `zr_*`（守卫见计划 06）。
6. `zr_operation` 依赖 `zr_kernel`、`zr_scene`、`zr_contracts` 与
   `zircon_runtime_interface`，拥有 `RuntimeOperationService`、task admission、owner-thread
   apply context 与 handler contract；`zr_navigation` 等 optional 域可依赖它。门面只做
   `zircon_runtime::operation` curated re-export，不保留内部旧路径或平行 registry。
7. `zr_text` 只拥有 backend-neutral shaping/layout/font/source/SDF 数据与服务。当前 text
   feature 中的完整 `wgpu`、`naga`、`glyphon` 依赖在 M3 同批硬切到
   `zr_graphics::text_backend`（底层资源操作可经 `zr_rhi`/`zr_rhi_wgpu`）；禁止让
   `zr_text` 与 `zr_graphics` 形成同层循环。该边界参考 Bevy 的 `bevy_text` 与
   `bevy_sprite_render`/`bevy_ui_render` 分离，但不保留兼容 adapter。

## 4. 里程碑

### M0 基线与决策批准

实现切片：
- 采集编译基线：根 workspace 与 `-p zircon_runtime` 的冷/增量 `cargo build --timings` 报告，存入本计划编号目录 `docs/plans/zircon_runtime/frameworks/01/baselines/`（含硬件与命令说明）；
- 生成当前模块依赖图（脚本扫描 `use crate::` 交叉引用），确认除计划 05 已列接缝外无未知横向依赖；
- 批准并锁定：crate 清单、`zr_` 命名、`zircon_runtime/crates/` 路径、CI 影响面（`.github/workflows/ci.yml` 需要的 members 变化）。
- 锁定 `operation -> zr_operation` 与 `text CPU service -> graphics GPU backend` 两项 owner
  决策；新的依赖图必须显式包含 `zr_operation -> zr_scene/zr_kernel`，且不得把
  `operation` 计入 kernel 或 facade implementation。

测试阶段：本里程碑为度量与决策，验收证据 = 基线报告 + 依赖图 + 本文件更新的锁定清单；无编译门。

### M1 Phase 1：零重依赖脊柱先行（kernel/contracts/math/resource/diagnostics）

实现切片：
- 新建 `zr_kernel`/`zr_contracts`/`zr_math`/`zr_resource`/`zr_diagnostics`，源码整目录移动（git mv），门面 `zircon_runtime` 以 `pub use` 恢复原公开路径；
- 按 `zr_math/zr_resource → zr_contracts → zr_kernel → zr_diagnostics` 顺序硬切；`core/framework/render/environment/source_cubemap/tests/projection.rs` 当前有两处 concrete Runtime `TaskPool` 反向测试引用，必须先迁到 kernel integration owner 并以静态守卫降为 0；禁止让 contracts 以 dev-dependency 反向依赖 kernel；
- `core/framework` 迁入 `zr_contracts` 时按域拆 feature（ai/physics/sound/net/render/ui/... 各成 feature，默认全开，勾稽计划 03）；
- 移动后同批修正所有 crate 内引用（`crate::core::…` → `zr_kernel::…` 等），不留旧路径别名。

测试阶段：
- 编译门：`cargo check -p zircon_runtime --lib --locked`、`cargo check -p zircon_editor --lib --locked`、`cargo check -p zircon_app --locked`
- 测试门（policy §3 最小批次）：focused 过滤词批 `cargo test -p zircon_runtime --lib --locked framework kernel resource diagnostic`（脊柱迁移域变更面回归）、`cargo test -p zircon_runtime_interface --locked`；全量 lib 回归留给波次收口（policy §4）
- 插件工作区防回归：`cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked`
- 验收证据：以上命令通过；`grep` 证明无 `path = "src/core/framework"` 类残留与迁移桥；增量基线复测记录。
- 文档更新：`docs/zircon_runtime/` 受影响模块文档的 `related_code` 路径、本文件状态表。

### M2 Phase 2：中层域拆出（platform/input/asset/scene/rhi/rhi_wgpu/render_graph）

实现切片：
- 依 layer 1–3 顺序逐域拆出；先拆 `zr_foundation`，再拆
  `zr_platform/zr_input -> zr_asset/zr_scene -> zr_operation -> zr_rhi/zr_rhi_wgpu/zr_render_graph`。
  `zr_operation` 必须在 `zr_navigation` 前完成硬切，并证明 internal crate 不依赖 facade。
  `zr_rhi_wgpu`、`zr_platform` 收拢 wgpu/winit 依赖，`zircon_runtime/Cargo.toml` 中对应依赖
  随迁移下沉到成员 crate；
- `builtin/runtime_modules` 组装代码留在门面，改为引用成员 crate 的模块描述符构造函数。

测试阶段：同 M1 命令集，另加：
- 运行门：`cargo run -p zircon_app --features target-client --bin zircon_runtime`（冒烟启动）与 `ZR_EXPORT_CONTRACT_PLATFORM=windows cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked`
- 验收证据：命令通过；`cargo tree -p zr_asset | grep -c wgpu` 为 0 等依赖下沉断言；增量编译对比报告（预期此阶段开始出现显著收益）。

### M3 Phase 3：重域拆出（graphics/text/ui/可选域）

前置：计划 05 的接缝契约化完成（graphics↔ui、asset↔ui、graphics↔scene 均已走 `zr_contracts`）。

实现切片：
- 先拆 backend-neutral `zr_text`，再把 text GPU atlas/upload/draw 实现硬切到
  `zr_graphics::text_backend`，然后拆 `zr_graphics`、`zr_ui`、`zr_script`、`zr_animation`、
  `zr_navigation`；可选域 crate 在门面 Cargo.toml 中转为 `optional = true` 并接入计划 03
  的 feature 矩阵；
- 清理门面 `lib.rs`：只剩 crate 声明、prelude、curated re-export、builtin 组装、dynamic_api、plugin 加载。

测试阶段：M1+M2 全部命令，另加：
- feature 门：`cargo check -p zircon_runtime --no-default-features --features target-server --locked`（断言不编译 zr_ui/zr_graphics/zr_animation/zr_navigation，用 `cargo tree` 证据）；
- 编辑器集成：`cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked`；
- 验收证据：编译时间收官报告 vs M0 基线（目标：单域增量 check ≥50% 改善，冷构建劣化 ≤10%）。
- 依赖验收：`cargo tree -p zr_text` 不含完整 `wgpu`、`naga`、`glyphon`；text GPU backend
  只存在于 `zr_graphics`/`zr_rhi_wgpu`，`zr_text <-> zr_graphics` 循环计数为 0；
  `cargo tree -p zr_operation` 只包含批准的 kernel/contracts/scene/interface 支撑边。
- 文档更新：`docs/engine-architecture/` 相关文档补 crate 拓扑章节；`CLAUDE.md` workspace layout 段。

### M4 dynamic_linking 开发模式与依赖治理收口

实现切片：
- 新增 `zr_dylib` 成员 crate 与门面 `dynamic_linking` feature（bevy_dylib 模式），`tools/dev-fast-build.ps1` 增加开关；
- 依赖治理：workspace 依赖全部收敛 `[workspace.dependencies]` 单源；引入 `cargo-deny`（license/duplicate/advisory）配置文件，接入计划 06 CI。
- 在首次 `cargo deny check` 前处理当前三个 prerelease pin：优先升级到兼容的 stable
  `notify`/`winit`/`zip`；若上游尚无可用 stable，只允许 exact-version、带 owner/原因/到期日期与
  升级 ticket 的局部例外。禁止 prerelease wildcard 或关闭 bans/advisories；到期例外使
  M4 保持 RED，而不是自动续期。

测试阶段：
- `cargo check -p zircon_runtime --features dynamic_linking --locked`、正常路径全命令复测、`cargo deny check`（本地证据即可，CI 接入归 06）；
- 验收证据：dynamic_linking 下 editor 启动冒烟；重复依赖版本清单归零或白名单化。

## 5. 风险与回退

- **cdylib 符号面**：dynamic_api 留在门面，`#[no_mangle]` 出口不动，风险低；每 Phase 用 `zircon_app` libloading 启动冒烟兜底。
- **孤儿规则**：跨 crate 的 trait impl 可能被迫移动归属；原则是 impl 随 trait 或随类型走，禁止 newtype 包装做兼容层；处理不了的接缝回流计划 05 重切。
- **工作量失控**：每 Phase 独立可验收、可暂停；任一 Phase 完成态都是合法长期形态，不存在"半迁移"中间态依赖桥。
- **与收束计划的表述冲突**：D1 的“门面 + 内部 crate”原则以及 `zr_operation`
  owner、`zr_text` CPU/GPU 分层已于 2026-07-31 同步到 index §3 与 architecture overview。
  如后续发现 `.codex/plans` 条目与本计划硬冲突，先更新双方勾稽再动代码。

## 6. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

- fixed 已修复：[`core/contracts` 反向依赖上层域与 facade](01/fixed-2026-07-13-core-contract-reverse-dependencies.md)（修复责任：Frameworks05；禁止方向 current-source 引用已清零，编译与核心行为门已通过）
- 产出记录：[`01/2026-07-13-m0-current-structure-and-dependency-baseline.md`](01/2026-07-13-m0-current-structure-and-dependency-baseline.md)
- M1 DAG 前置记录：[`01/2026-07-17-m1-resource-error-owner-dag-prerequisite.md`](01/2026-07-17-m1-resource-error-owner-dag-prerequisite.md)（resource registry error owner hard-cut 已实现，locked Cargo 验收 pending）
- M1 DAG 前置记录：[`01/2026-07-18-m1-runtime-diagnostics-facade-collector-hardcut.md`](01/2026-07-18-m1-runtime-diagnostics-facade-collector-hardcut.md)（manager-resolving diagnostics 已移出 core，静态门通过，Cargo 与 Shader06 foreign doc pending）
- M1 DAG 前置记录：[`01/2026-07-18-m1-runtime-error-owner-dag-prerequisite.md`](01/2026-07-18-m1-runtime-error-owner-dag-prerequisite.md)（`CoreError/CoreResult` 已硬切到 runtime kernel owner，静态门通过，复审/Cargo pending）
## Code Review 收敛 (2026-07-31)

- `operation` owner 已按当前 `CoreHandle + scene::World` 依赖事实锁定为 layer-3
  `zr_operation`，而不是错误并入 `zr_kernel` 或留在 facade。M2/M3 顺序、依赖验收和
  optional navigation 消费方向已同步。
- `zr_text` 已锁定为 backend-neutral 服务；完整 `wgpu`/`naga`/`glyphon` 实现随 M3
  迁到 `zr_graphics::text_backend`/`zr_rhi_wgpu`，并新增无循环、无 direct GPU dependency
  的机器验收。该决策消除了旧拓扑对 text 直接 GPU 依赖的遗漏。
- M4 已加入 `notify`/`winit`/`zip` prerelease 的 stable-first、exact scoped exception 与到期 RED
  规则，不以全局 allow 或关闭 advisory/bans 绕过治理。

- 当前状态：M0 已在 2026-07-30 对 9,188 个 Runtime Rust 输入完成 pre/post 同指纹的原子快照：7 个根 workspace members、2,391 production refs / 76 domain edges；旧 `core→asset/graphics/scene` 与 internal→facade 生产反向边已经清零。新出现的 `foundation` 顶层域按 runtime absorption 权威确定为 layer-1 `zr_foundation`，不并入 core、不形成公开根包。2026-07-31 目标拓扑又补齐 `zr_operation` owner、text CPU/GPU backend 分层和 prerelease deny 决策；这些是物理迁移前的设计收敛，不是代码完成。物理硬切前必须重新采集同口径快照，并消除 `asset→text=2`、`scene→animation=2`、`rhi→rhi_wgpu=1`，同时把 source-cubemap projection 的两处 concrete Runtime `TaskPool` 测试迁到 kernel integration owner。M1 当前已完成 resource error、manager diagnostics、runtime error 三个 owner-DAG 静态前置且无 alias/shim；`zircon_runtime/crates/` 仍不存在，M1–M4 尚未开始物理迁移。四份冷/增量 `cargo build --timings` 仍在受管 FIFO 测试阶段 pending，故不声明 M0、M1 或计划 01 完成。
