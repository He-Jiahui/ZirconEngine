---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/lib.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/plugins/groups.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - tools/check-runtime-domain-features.ps1
  - tools/tests/test_frameworks_03_domain_feature_matrix.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/index.md
  - docs/runtime-plugins/profile-selection.md
  - docs/engine-architecture/plugin-optional-feature-bundles.md
reference_engines:
  - dev/bevy/Cargo.toml
  - dev/bevy/crates/bevy_internal
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/AIModule.Build.cs
---

# 03 · 可选功能 feature 矩阵与 profile 预设

## 1. 目标

让"功能可选地开启或关闭"从口号变成编译事实：每个可选子系统都有 feature、每个 profile 都是一份可验证的 feature 预设、关键组合常驻 CI。目标状态：

- `target-server` 构建物不含 ui/graphics/animation/navigation/script 任何代码；
- 六 profile（Minimal/Client2d/Client3d/Editor/Dev/Server）与 Cargo feature 预设单源勾稽；
- feature 命名与分层规则成文，插件 feature（first-party catalog）与 runtime feature 同一词汇。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-runtime-optional-features-profile-matrix",
  "goal": "收敛可选功能 feature 矩阵、profile 预设单源与显式消费边界",
  "milestones": [
    {"id": "M1", "title": "现有模块补门", "depends_on": []},
    {"id": "M2", "title": "profile 预设单源与 CI 矩阵", "depends_on": ["M1"]},
    {"id": "M3", "title": "拆分后收口", "depends_on": ["M2"]},
    {"id": "M4", "title": "profile API hard-cut 消费收敛", "depends_on": []}
  ]
}
```

## 2. 现状与差距

- `zircon_runtime` 现有 22 个 feature 覆盖 targeting/platform/profiling/测试，但 **animation、navigation、script、diagnostic_log 无条件编译**（`src/lib.rs` 无 cfg 门），`core/framework` 的 ai/physics/sound/net 等契约域也全量编译（合计 ~30K+ 行无谓编译进 server 目标）。
- profile 与 feature 是两套系统：`RuntimeProfileId` 在运行期选模块，Cargo feature 在编译期裁剪，两者映射靠人脑。
- feature 命名不成体系：`plugin-ui`、`jolt`、`zr-vm-real-backend` 风格各异。
- 无 feature 组合 CI：`--no-default-features` 变体只有 CLAUDE.md 中的手工命令。

## 3. 设计决策

### 3.1 三层 feature 激活链（bevy 模式）

```
zircon_app features（面向使用者/CI 的组合层）
  target-client / target-server / target-editor-host / dev-tools ...
    ↓ 只做转发与组合
zircon_runtime 门面 features（面向域的开关层）
  ui, graphics, text, animation, navigation, script,
  physics-contracts, sound-contracts, net-contracts, ai-contracts,
  platform-winit / platform-headless, profiling-*, dynamic_linking
    ↓ 转发到成员 crate（计划 01 落地后）
zr_* 成员 crate features（实现细节层，外部不可见）
```

命名规则：域开关用裸域名（`animation`）；契约-only 用 `*-contracts`；平台/后端用 `platform-*`/`backend-*`；插件捆绑沿用 feature bundle 文档词汇。全部 kebab-case，禁止再造 `zr-vm-real-backend` 式前缀（迁移时更名）。

### 3.2 profile = feature 预设 + 模块选择，单源生成

在 `zircon_runtime` 新增单源表（Rust 常量 + 单测导出）：每个 `RuntimeProfileId` 声明（a）要求的编译 feature 集，（b）运行期模块/插件选择（既有 profile-selection 语义）。`zircon_app` 的 target-* feature 与 CI 命令从该表勾稽；表变更必须同步 `docs/runtime-plugins/profile-selection.md`（M10 同步门既有规则）。

### 3.3 组合验证策略（防组合爆炸）

不做全组合，锁定守卫组合清单：

1. 六 profile 各自的预设组合；
2. `--no-default-features` + 每个域 feature 单独开启（可加性检查，防 feature 间隐式耦合）；
3. `--all-features`；
4. 每个 `*-contracts` 不带实现单独编译（契约纯度检查）。

## 4. 里程碑

### M1 现有模块补门（单 crate 内，先于计划 01 拆分）

实现切片：
- `lib.rs` 为 animation/navigation/script/diagnostic_log 加 `#[cfg(feature)]`，Cargo.toml 增加对应 feature 并入 default；
- `core/framework` 各契约域加 feature 门（默认全开）；
- `builtin/runtime_modules` 与 `core/manager` 中对可选域的引用同步 cfg 化（模块组装表按 feature 裁剪）；
- feature 更名（`plugin-ui`→`ui` 等）同批硬切，调用方（zircon_app/zircon_editor/CI/tools 脚本）一次迁完。

测试阶段：
- 编译门：`cargo check -p zircon_runtime --lib --locked`（default）、`cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked`、逐域单开组合脚本化跑一轮；
- 测试门（policy §3 最小批次）：focused 过滤词批 `cargo test -p zircon_runtime --lib --locked feature profile animation navigation script diagnostic` + `cargo test -p zircon_app --locked plugins profile`；全量 lib 回归留给波次收口（policy §4）；
- 验收证据：server 组合的 `cargo tree` 无 wgpu/winit/taffy；逐域单开全绿记录。
- 文档更新：`CLAUDE.md` 常用命令段、`docs/runtime-plugins/profile-selection.md` feature 列。

### M2 profile 预设单源与 CI 矩阵

实现切片：
- profile→feature 预设表落地（常量 + 断言测试：表内容与 Cargo.toml feature 定义一致）；
- `.github/workflows/ci.yml` 增加守卫组合矩阵 job（§3.3 清单，check-only，控制时长）；
- `tools/dev-fast-build.ps1` 的 profile 映射改为读同一张表的导出（或至少加断言测试防漂移）。

测试阶段：
- 本地全矩阵脚本一轮绿 + CI dry-run（push 到分支验证 workflow）；
- 验收证据：CI 出现矩阵 job 且全绿；预设表断言测试入库。
- 文档更新：profile-selection.md M10 同步门条目补"feature 预设"行。

### M3 拆分后收口（依赖计划 01 M3）

实现切片：
- 门面 feature 全部改为转发成员 crate feature；可选域 crate `optional = true`；
- 复核守卫组合，补"成员 crate 不被越级启用"的 cargo tree 断言。

测试阶段：M1+M2 全命令复测；验收证据：server 组合编译单元列表（`cargo build --timings` 的 crate 清单）不含 zr_ui/zr_graphics/zr_animation/zr_navigation/zr_script。

### M4 profile API hard-cut 消费收敛

实现切片：
- `RuntimeProfileId` 只保留 project contract 身份，不恢复 plugin façade re-export；
- 删除 enum convenience method 后，preset 消费者显式读取 plugin owner 的 `RUNTIME_PROFILE_FEATURE_PRESETS`；
- 禁止 compatibility alias、shim 或回退到旧方法。

测试阶段：聚焦 integration contract 2/2；全仓 Rust source scan 确认 `.feature_preset()` 零命中。

## 5. 风险与回退

- **cfg 蔓延污染代码**：规则是"门开在模块声明与组装表上，不开在业务逻辑里"；出现深层 `#[cfg]` 分支视为设计缺陷回流计划 05 切接缝。
- **CI 时长**：矩阵 job 全部 check-only + 共享 sccache；超预算则收缩到 profile 六组合 + all-features。
- **默认行为漂移**：M1 所有新 feature 进 default，行为与现状逐位等价；裁剪只发生在显式 `--no-default-features` 路径。

## 6. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

- 产出记录：[`03/2026-07-10-optional-features-and-profile-matrix-output-records.md`](03/2026-07-10-optional-features-and-profile-matrix-output-records.md)
- 2026-07-15 硬切记录：[`03/2026-07-15-runtime-profile-id-canonical-import.md`](03/2026-07-15-runtime-profile-id-canonical-import.md)（canonical type owner 与显式 preset-table lookup 已完成；受管 integration contract 2/2）
- fixed 已修复：[target-server-libtest-feature-gating](../../zircon_editor/editor/11/fixed-2026-07-11-target-server-libtest-feature-gating.md)
- fixed 已修复：[runtime-module-structure-cfg-fence](../../zircon_editor/editor/10/fixed-2026-07-11-runtime-module-structure-cfg-fence.md)
- fixed 已修复：[planar-filter-test-surface-export](03/fixed-2026-07-13-planar-filter-test-surface-export.md)
- 当前状态：M1 进行中；feature 命名、`target-server` 域裁剪、AI/Net/Sound/Physics contract 独立门控与 ZRPack asset owner 硬迁移已完成。Sound channel topology 已硬迁到常驻 `core::framework::audio`；Physics 持久化 material/joint/skeleton schema 已硬迁到常驻 `core::framework::scene::physics`，可选 simulation/query/world-sync/manager 合同保留在 `physics-contracts`，LevelSystem 与 diagnostics 通过声明期 enabled/disabled adapter 隔离。两条迁移均无旧路径兼容重导出。Client/Editor 预设包含四个 contract 域，Server 不隐式包含，直接 plugin 消费者显式请求各自契约。Frameworks Python 合同门当前 76/76；其中四个 optional manager guard 已硬切为无 trait root re-export 的 `ManagerServiceHandle`/`define_manager_handle_access!`，六 profile TOML 也已逐项同步当前 Runtime/App Cargo members（含 Editor navigation plugin 与 Server `dep:naga`）。Runtime `physics` 聚焦 35/35，Physics plugin owner 的 feature-on 46/46、feature-off 43/43 已通过；nightly `core-min + physics-contracts` 单开通过（12m39s，52 条既有 warning），nightly `target-server` 排除组合通过（15m14s，53 条既有 warning）。M1 逐域 runner 固定 12 域、`core-min + 单域`、locked/no-default/lib check 与失败汇总；2026-07-11 fresh locked/offline 独立目标矩阵已 12/12 全绿。首轮 11/12 精确暴露 `graphics` 反向依赖 `ui`；修复后 Graphics rich-text layout 改为消费 `graphics::text` owner，UI-only frame conversion 在声明处受 `ui` gate 控制，公共 render-mode resolution 下沉 `zircon_runtime_interface`，不增加兼容重导出。M1 App 当前完整 harness 与包级 Runtime absorption 门均已 GREEN；默认 feature Runtime 全量 lib suite 仍为 RED/pending。2026-07-16 fresh G7 文档审计在用户优先的结构约定与代码评审计划中重新发现 58 个 Text 旧 owner 引用；58/58 已映射到真实 canonical 路径，但在 maintenance hard-cut 与复验完成前不声明闭环。M2 的六 profile TOML 单源、生成 Rust 常量、开发工具/本地 runner、六 profile CI 源码矩阵与十二域 CI 源码矩阵已落地；Python 契约 7/7、server production check、Rust 断言 2/2、Editor CLI 目标测试 14/14、Windows nightly locked/offline 六 profile 本地全矩阵 6/6，以及带真实 ZR VM MSVC 原生后端的 Runtime `--all-features` 均已通过。仅分支 CI 实际全绿证据仍 pending，不声明 M1、M2 或计划 03 完成。
