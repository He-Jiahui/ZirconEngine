---
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/runtime-feature-presets.toml
  - zircon_runtime/src/lib.rs
  - zircon_app/Cargo.toml
  - zircon_app/src/plugins/groups.rs
  - examples/vampire/README.md
  - docs/superpowers/plans/2026-06-09-vampire-dark-content-upgrade.md
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - tools/check-runtime-domain-features.ps1
  - tools/tests/test_frameworks_03_domain_feature_matrix.py
  - tools/tests/test_frameworks_03_server_feature_boundary.py
  - tools/tests/test_runtime_tech_stack_boundary.py
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
    {"id": "M4", "title": "profile API hard-cut 消费收敛", "depends_on": []},
    {"id": "M5", "title": "ExportProfile 显式 runtime profile 身份", "depends_on": ["M4"]}
  ]
}
```

## 2. 现状与差距

- M1 feature 门代码已经落地：animation/navigation/script/`diagnostic-log` 由 crate-root cfg 控制，
  ai/net/physics/sound contracts 独立门控；`target-server` 不再隐式启用 client/editor 重域。
  十二域 additive runner 与既有离线证据已覆盖逐域单开，current-main 默认 lib suite 仍 RED/pending，
  因此只声明 `code-complete / validation-pending`，不把 M1 标为 accepted。
- M2 已将 `runtime-feature-presets.toml` 升级为 schema v2：同一输入生成 feature preset 与
  runtime module/plugin assembly 两个 Rust 投影，开发工具、Python guard 与 CI matrix 同样读取该
  TOML。手写 `runtime_profile/defaults.rs` 已物理删除；模块注册表的 Rust variant/cfg 门和六 profile
  查找均由生成期严格验证，公开 descriptor API 不变且未保留 alias、shim 或 fallback。
  实现与独立二次审查修复已完成，current-main 受管 Cargo 验收仍 pending，故 M2 尚未 accepted。
- 当前 canonical feature 命名已经硬切为 kebab-case role prefixes：域开关裸名、contracts 用
  `*-contracts`、backend 用 `backend-*`、first-party provider collection 用
  `first-party-*-runtime-plugin`/`first-party-*-editor-plugin`。旧 `plugin-ui`、`jolt`、
  `zr-vm-real-backend` 均不得恢复；`backend-zr-vm` 是当前合法 backend 名，不是旧前缀兼容项。
  可复制的用户示例同样属于 hard-cut consumer：`examples/vampire/README.md` 与 Vampire content
  implementation plan 已迁到 `backend-zr-vm`，旧 `first-party-zr-vm-real-backend` 不作为示例、
  alias 或兼容 feature 保留。
- CI workflow、六 profile/十二域 matrix source 与 profile 两个生成投影均已落地；剩余是
  current-main 实际全绿证据，以及计划 01 拆 crate 后的成员 feature 转发收口。

## 3. 设计决策

### 3.1 三层 feature 激活链（bevy 模式）

```
zircon_app features（面向使用者/CI 的组合层）
  target-client / target-server / target-editor-host / dev-tools ...
    ↓ 只做转发与组合
zircon_runtime 门面 features（面向域的开关层）
  ui, graphics, text, animation, navigation, script, diagnostic-log,
  physics-contracts, sound-contracts, net-contracts, ai-contracts,
  platform-winit / platform-headless, profiling-*, dynamic_linking
    ↓ 转发到成员 crate（计划 01 落地后）
zr_* 成员 crate features（实现细节层，外部不可见）
```

命名规则：域开关用裸域名（`animation`）；契约-only 用 `*-contracts`；平台/后端用
`platform-*`/`backend-*`；first-party provider collection 用
`first-party-<plugin>-runtime-plugin`/`first-party-<plugin>-editor-plugin`；插件内捆绑沿用 feature
bundle 文档词汇。全部 kebab-case。禁止恢复 `zr-vm-real-backend`；当前 `backend-zr-vm`
符合 backend role，`first-party-zr-vm-language-runtime-plugin` 符合 provider identity，不做二次更名。

### 3.2 profile = feature 预设 + 模块选择，单源生成

以 `zircon_runtime/runtime-feature-presets.toml` 为唯一 profile spec：每个 `RuntimeProfileId` 同时声明
（a）runtime/app Cargo feature 集，（b）target mode、maturity、required capabilities、default/optional
插件与内建模块选择。build script 生成 `RUNTIME_PROFILE_FEATURE_PRESETS` 以及
`RuntimeProfileDescriptor::for_id` 所需的 Rust 数据；删除 `runtime_profile/defaults.rs` 的手写六分支，
不保留第二份表或兼容读取。`zircon_app` 的 target-* feature、开发工具与 CI 命令从该 TOML 勾稽；
表变更必须同步 `docs/runtime-plugins/profile-selection.md`（M10 同步门既有规则）。

该 hard cut 使用 schema v2，并把编译预设与运行期装配作为同一文档的两个生成投影，而不是同一个
Rust 模块的混合职责：build script 只解析/校验一次 TOML，分别生成公开
`RUNTIME_PROFILE_FEATURE_PRESETS` 与私有 runtime assembly preset。assembly 行必须显式声明
`descriptor_name`、`target_mode`、`minimum_maturity`、`builtin_modules`、带 `required` 位的有序
`default_plugins`、有序 `optional_plugins`、`required_capabilities` 和
`allow_externalized_required_plugins`；不得依赖 descriptor builder 的默认值补齐表外事实。

内建模块使用独立 registry 行声明 `id`、Rust variant 与可选 `required_feature`。生成 Graphics/Script
引用时必须保留当前 `#[cfg(feature = "graphics")]` / `#[cfg(feature = "script")]` 语义，保证
no-default/server 组合不引用被 cfg 删除的 enum variant。生成器在写文件前 fail-fast 校验：schema
版本、六 profile 精确顺序与唯一性、profile/variant/descriptor name、target/maturity 枚举、module
registry 引用、module/plugin/capability 去重、default/optional plugin 不交叉、显式 required/allow
字段、非空 canonical capability，以及 plugin key 语法。生成阶段保持 TOML 声明顺序，不排序改写
运行时展示或 project manifest 顺序。

`target-server` 中的直接 `dep:naga` 是有意的：常驻 asset shader importer 需要解析/验证并转写
GLSL/SPIR-V/WGSL，即使 graphics/text 均关闭也仍使用 naga。profile generator 与 guard 必须把它
记录为 asset import dependency，不能当作 text 的重复依赖删除；若未来把 shader import 独立门控，
只能在对应 owner 计划中同批迁移 Cargo edge、importer cfg 与 server preset。

### 3.3 组合验证策略（防组合爆炸）

不做全组合，锁定守卫组合清单：

1. 六 profile 各自的预设组合；
2. `--no-default-features` + 每个域 feature 单独开启（可加性检查，防 feature 间隐式耦合）；
3. `--all-features`；
4. 每个 `*-contracts` 不带实现单独编译（契约纯度检查）。

## 4. 里程碑

### M1 现有模块补门（单 crate 内，先于计划 01 拆分）

状态（2026-07-31）：feature/cfg、contract gates、server 裁剪与十二域 runner 已 code-complete；
current-main 默认 feature Runtime 全量 lib suite 仍 RED/pending，M1 未 accepted。

实现切片：
- `lib.rs` 已为 animation/navigation/script/`diagnostic-log` 增加 `#[cfg(feature)]`，Cargo.toml
  对应 feature 已进入 default profile；
- `core/framework` 各契约域加 feature 门（默认全开）；
- `builtin/runtime_modules` 与 `core/manager` 中对可选域的引用同步 cfg 化（模块组装表按 feature 裁剪）；
- feature 更名（`plugin-ui`→`ui` 等）同批硬切，调用方（zircon_app/zircon_editor/CI/tools 脚本）一次迁完。

测试阶段：
- 编译门：`cargo check -p zircon_runtime --lib --locked`（default）、`cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked`、逐域单开组合脚本化跑一轮；
- 测试门（policy §3 最小批次）：focused 过滤词批 `cargo test -p zircon_runtime --lib --locked feature profile animation navigation script diagnostic` + `cargo test -p zircon_app --locked plugins profile`；全量 lib 回归留给波次收口（policy §4）；
- 验收证据：server 组合的 `cargo tree` 无 wgpu/winit/taffy；逐域单开全绿记录。
- hard-cut guard：生产 manifest、工具、CI 与可复制示例均不得出现退役 feature；允许命中的旧 token
  只能位于明确断言“旧名必须失败/不得成为 owner”的负例测试或历史结果记录中。
  `test_vampire_executable_commands_use_canonical_zr_vm_backend_feature` 常驻读取两个 Vampire
  可复制命令，要求 language provider + `backend-zr-vm` 并拒绝退役 backend feature。
- 文档更新：`CLAUDE.md` 常用命令段、`docs/runtime-plugins/profile-selection.md` feature 列。

### M2 profile 预设单源与 CI 矩阵

状态（2026-08-04）：schema v2 TOML 已统一生成 feature preset 与 runtime assembly；手写
`runtime_profile/defaults.rs` 已硬删除，模块 cfg 门与 `RuntimeProfileId` 穷举查找均由生成器校验。
实现与首轮独立审查问题修复已完成；current-main 受管 Cargo 验收 pending，所以 M2 尚未 accepted。

实现切片：
- 扩展现有 profile TOML 到 schema v2，使其同时生成 feature preset 与 runtime module/plugin
  selection；生成物按编译预设/装配预设分离，删除手写 `runtime_profile/defaults.rs` 六分支并将
  wiring 直接切到 `assembly_presets`，不保留 alias、shim 或 fallback；
- 断言生成结果与 Runtime/App Cargo features、descriptor name、target mode、maturity、capabilities、
  default plugin required 位、optional plugin、cfg-aware module identities 和
  `allow_externalized_required_plugins` 一致；保留 `RuntimeProfileDescriptor::for_id` /
  `builtin_profiles` 的公开签名和稳定顺序，不增加消费者兼容层；
- `.github/workflows/ci.yml` 增加守卫组合矩阵 job（§3.3 清单，check-only，控制时长）；
- `tools/dev-fast-build.ps1` 的 profile 映射改为读同一张表的导出（或至少加断言测试防漂移）。

测试阶段：
- Python/source guard 覆盖 schema v2 全字段、生成器 fail-fast 负例、旧 `defaults.rs`/`mod defaults`
  物理消失以及原 feature/matrix 工具输出不变；Rust focused contract 覆盖六 profile 黄金 parity、
  `for_id`/`builtin_profiles` 等价、default plugin required/target 投影，并覆盖 no-default、
  graphics-only、script-only、graphics+script 的模块裁剪；
- 本地全矩阵脚本一轮绿 + CI dry-run（push 到分支验证 workflow）；
- 验收证据：CI 出现矩阵 job 且全绿；预设表断言测试入库。
- 文档更新：profile-selection.md M10 同步门条目补"feature 预设"行。

### M3 拆分后收口（依赖计划 01 M3）

状态（2026-08-10）：已对计划 01 先行物理拆出的 `zr_rhi`/`zr_rhi_wgpu` 完成首个成员
feature-forwarding 边界切片。根 workspace 两条内部依赖均显式关闭默认 feature；闭包审计修复了
`default-features = false` 且无请求 feature 的常驻本地包会被漏算的问题，并证明 Server 的
Runtime/App 闭包包含中立 `zr_rhi`、不包含 `zr_rhi_wgpu` 或其 `platform-winit`。该切片 Source
闭包同时证明 Client/Editor 正向激活 `zr_rhi_wgpu` 并把 `platform-winit` 转发到两个成员。该切片
Source Ready，独立二审 C0/I0/M0；managed Cargo pending。其余成员尚未物理拆出，因此 M3
整体仍未完成。

实现切片：
- 门面 feature 全部改为转发成员 crate feature；可选域 crate `optional = true`；
- 复核守卫组合，补"成员 crate 不被越级启用"的 cargo tree 断言。
- 已落地的首批静态闭包守卫同时校验 workspace member dependency 的
  `default-features = false`、中立 RHI 常驻可达与 WGPU backend 的 Server 排除；后续每个物理
  `zr_*` 成员必须进入同一闭包模型，不得用源码字符串计数代替依赖图。

测试阶段：当前 Frameworks03 Python 合同 55/55 GREEN，新增 RHI 闭包 focused 1/1 先 RED
后 GREEN；11 输入 focused 静态票据与 8,021 输入 canonical Rust 1.94.1 Server check 请求均已
取得 durable receipt，终态待 coordinator wakeup；M1+M2 Cargo 全命令与 managed Server package
graph 仍待复测。最终验收证据：server 组合编译单元列表（`cargo build --timings` 的 crate 清单）不含
zr_ui/zr_graphics/zr_animation/zr_navigation/zr_script。

### M4 profile API hard-cut 消费收敛

状态（2026-07-31）：canonical owner、显式 preset lookup、旧 method 零命中及受管 integration
contract 2/2 已完成；M4 已完成，不恢复 facade export、alias 或 convenience shim。

实现切片：
- `RuntimeProfileId` 只保留 project contract 身份，不恢复 plugin façade re-export；
- 删除 enum convenience method 后，preset 消费者显式读取 plugin owner 的 `RUNTIME_PROFILE_FEATURE_PRESETS`；
- 禁止 compatibility alias、shim 或回退到旧方法。

测试阶段：聚焦 integration contract 2/2；全仓 Rust source scan 确认 `.feature_preset()` 零命中。

### M5 ExportProfile 显式 runtime profile 身份

状态（2026-07-31）：Runtime/App/Editor 构造与 export plan hard-cut 已 code-complete，旧 builder
及 name/target fallback 为零；current-source 受管 Cargo 验收 pending，M5 未 accepted。

实现切片：
- `ExportProfile::new` 强制接收 `RuntimeProfileId`，删除可延迟补写身份的 builder；
- export plan 只读取显式身份，不再依据 profile 名称或 target mode 推断；
- 反序列化缺失身份只保留为 fatal validation 路径，availability 保持为空，不提供兼容成功路径；
- Runtime、App 与 Editor 的所有构造点同批迁移，不增加 alias、shim 或旧签名重载。

测试阶段：Runtime export-profile 聚焦回归、App bootstrap 与 Editor export consumer 聚焦门均通过；全仓
Rust source scan 确认旧 builder、`.feature_preset()` 与名称推断为零；受管门启动前后验证完整编译输入未漂移。

## 5. 风险与回退

- **cfg 蔓延污染代码**：规则是"门开在模块声明与组装表上，不开在业务逻辑里"；出现深层 `#[cfg]` 分支视为设计缺陷回流计划 05 切接缝。
- **CI 时长**：矩阵 job 全部 check-only + 共享 sccache；超预算则收缩到 profile 六组合 + all-features。
- **默认行为漂移**：M1 所有新 feature 进 default，行为与现状逐位等价；裁剪只发生在显式 `--no-default-features` 路径。

## Code Review 收敛 (2026-07-31)

- 已把 animation/navigation/script/`diagnostic-log` 与四个 contract gates 从“缺失”改为
  code-complete；2026-07-31 当时的剩余项是 current-main acceptance、profile 两半单源和拆 crate
  后转发。2026-08-04 profile 两个投影已完成 schema v2 单源化，当前仅保留受管 acceptance 与
  计划 01 拆分后的 feature 转发收口。
- feature spelling 已统一为 `diagnostic-log`。经 current manifest/owner 文档核验，未采纳对
  `backend-zr-vm` 与 `first-party-zr-vm-language-runtime-plugin` 的二次更名建议：前者符合
  `backend-*` role，后者是 provider collection identity；二者均是旧 `zr-vm-real-backend`
  硬切后的 canonical 名，重新命名会制造无依据的第三套 surface。
- 二次审查发现 `examples/vampire/README.md` 与 Vampire content implementation plan 仍含不可用的
  `first-party-zr-vm-real-backend`；已前向硬切为 `backend-zr-vm`，并把可复制示例纳入旧名
  absence guard，不以 alias 或兼容 feature 掩盖 consumer 漂移。历史 output records 与明确的退役名
  negative tests 保留旧 token 作为证据，不构成可调用 surface。
- 上述 consumer 修复已提升为常驻 TDD guard：helper 缺失时 focused 测试稳定 RED，加入 canonical
  feature-set validator 后 focused 1/1 GREEN；不依赖 Cargo 编译或兼容 alias 才能发现回流。
- 同文件完整 tech-stack upward suite 当前为 5/8：新增 canonical-command guard 通过；三个既有断言
  被两类外部审计同步漂移终止——Runtime01 的 retired-dependency inventory 仍把 Sound plugin 当前
  `kira` 依赖分类为禁止项，Runtime06 native-plugin public-surface 计数未同步 current namespace/App
  call sites。该结果不回滚 Frameworks03 consumer 修复，也不把外部 guard RED 误记为本切片 GREEN；
  两类最低 owner 须前向复核依赖归属与公开面分类，禁止通过放宽本 hard-cut guard 绕过。
- 已查明 server 直接 naga 的生产消费者是常驻 asset shader importer，而非 text/graphics；
  该边已写入 profile generator 约束，防止 M2 把它误判为重复依赖。
- 已识别比原建议更深的 M2 缺口：现有 TOML 只生成 feature presets，运行期 module/plugin
  selection 当时仍手写。目标 hard-cut 已改为扩展同一 TOML 并删除 defaults.rs 平行六分支；
  2026-08-04 该 hard-cut 已实现并通过独立二次审查，受管 Cargo 验收 pending。

## 6. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

- 产出记录：[`03/2026-07-10-optional-features-and-profile-matrix-output-records.md`](03/2026-07-10-optional-features-and-profile-matrix-output-records.md)
- 2026-07-15 硬切记录：[`03/2026-07-15-runtime-profile-id-canonical-import.md`](03/2026-07-15-runtime-profile-id-canonical-import.md)（canonical type owner 与显式 preset-table lookup 已完成；受管 integration contract 2/2）
- 2026-07-17 显式 export profile 记录：[`03/2026-07-17-export-profile-runtime-profile-explicit-hardcut.md`](03/2026-07-17-export-profile-runtime-profile-explicit-hardcut.md)（生产 hard-cut 已实现，受管 Cargo 验收 pending）
- 2026-07-22 Audio owner 硬切记录：[`03/2026-07-22-audio-channel-layout-owner-hardcut.md`](03/2026-07-22-audio-channel-layout-owner-hardcut.md)（Sound root 私有重导出已删除，无兼容层；静态门与受管 `sound-contracts` 编译均 GREEN）
- 2026-08-10 M3 首批成员 feature forwarding：[`03/2026-08-10-m3-rhi-feature-forwarding-boundary.md`](03/2026-08-10-m3-rhi-feature-forwarding-boundary.md)（workspace 默认 feature 硬关闭、Server RHI package closure 与 Neural editor profile 单源同步已实现；独立二审 C0/I0/M0，managed Cargo pending）
- fixed 已修复：[target-server-libtest-feature-gating](../../zircon_editor/editor/11/fixed-2026-07-11-target-server-libtest-feature-gating.md)
- fixed 已修复：[runtime-module-structure-cfg-fence](../../zircon_editor/editor/10/fixed-2026-07-11-runtime-module-structure-cfg-fence.md)
- fixed 已修复：[planar-filter-test-surface-export](03/fixed-2026-07-13-planar-filter-test-surface-export.md)
- 当前状态：M1 进行中；feature 命名、`target-server` 域裁剪、AI/Net/Sound/Physics contract 独立门控与 ZRPack asset owner 硬迁移已完成。Sound channel topology 已硬迁到常驻 `core::framework::audio`；Physics 持久化 material/joint/skeleton schema 已硬迁到常驻 `core::framework::scene::physics`，可选 simulation/query/world-sync/manager 合同保留在 `physics-contracts`，LevelSystem 与 diagnostics 通过声明期 enabled/disabled adapter 隔离。两条迁移均无旧路径兼容重导出。Client/Editor 预设包含四个 contract 域，Server 不隐式包含，直接 plugin 消费者显式请求各自契约。历史 Frameworks Python 合同门为 76/76；本次实际复跑的五个 owner suite 为 55/55，并把 Neural editor provider 同步进 Editor/Dev canonical profile，避免 App feature 已接线而 schema v2 单源漂移。其中四个 optional manager guard 已硬切为无 trait root re-export 的 `ManagerServiceHandle`/`define_manager_handle_access!`，六 profile TOML 也已逐项同步当前 Runtime/App Cargo members（含 Editor navigation/Neural plugin 与 Server `dep:naga`）。Runtime `physics` 聚焦 35/35，Physics plugin owner 的 feature-on 46/46、feature-off 43/43 已通过；nightly `core-min + physics-contracts` 单开通过（12m39s，52 条既有 warning），nightly `target-server` 排除组合通过（15m14s，53 条既有 warning）。M1 逐域 runner 固定 12 域、`core-min + 单域`、locked/no-default/lib check 与失败汇总；2026-07-11 fresh locked/offline 独立目标矩阵已 12/12 全绿。首轮 11/12 精确暴露 `graphics` 反向依赖 `ui`；修复后 Graphics rich-text layout 改为消费 `graphics::text` owner，UI-only frame conversion 在声明处受 `ui` gate 控制，公共 render-mode resolution 下沉 `zircon_runtime_interface`，不增加兼容重导出。M1 App 当前完整 harness 与包级 Runtime absorption 门均已 GREEN；默认 feature Runtime 全量 lib suite 仍为 RED/pending。2026-07-17 fresh G7 复核确认用户优先的 `engine-code-structure-convention.md` 与 `engine-code-review-findings-2026-06.md` 当前旧 owner 违规均为 0；父计划此前记录的 58 个 Text 引用属于已收敛历史快照。M4 已完成；M5 已把 `ExportProfile` Rust 构造、Runtime/App/Editor 消费者和 export build plan 硬切为显式 `RuntimeProfileId`，旧 builder 与 name/target fallback 均为零，受管 Cargo 验收 pending。M2 已把六 profile feature preset 与 runtime module/plugin assembly 收敛到 schema v2 TOML，生成器对 12 个 module identity/cfg gate 和六个 `RuntimeProfileId` 分支做严格检查，手写 `runtime_profile/defaults.rs` 已硬删除且无兼容层；实现与首轮独立审查问题修复完成，受管 current-main Cargo 验收 pending。M3 的首个已拆成员切片已显式关闭 RHI workspace 默认 feature，修复本地包闭包漏算，并静态证明 Server 不启用 WGPU backend；其余成员仍依赖计划 01 物理拆分。此前 Python 契约、server production check、Rust 断言、Editor CLI 目标测试、Windows nightly locked/offline 六 profile 矩阵及 Runtime `--all-features` 证据保留；因此仍不声明 M1、M2、M3、M5 或计划 03 accepted。
