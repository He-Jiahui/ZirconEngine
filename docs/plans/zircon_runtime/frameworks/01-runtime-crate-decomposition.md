---
related_code:
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/engine_module/engine_module.rs
  - zircon_runtime/src/rhi.rs
  - zircon_runtime/crates/zr_rhi/src/lib.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/lib.rs
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
- 旧声明顺序注释及 `graphics→scene`、`graphics→ui`、`asset→text` 直接依赖已由 Frameworks05 硬切清零；2026-08-03 又将 `rhi→rhi_wgpu=1` 硬切为物理 `zr_rhi <- zr_rhi_wgpu` 正向依赖。2026-08-10 的 animation sampling contract inversion 进一步把 `scene→animation=2` 收敛为 0：中立 cursor/request/batch/sampler 归 `core::framework::animation`，animation 实现资产加载与采样，scene 只保留有界队列和重试策略。fresh production matrix 中上述边均为 0；M0 历史快照仍用于对比，禁止继续把历史 35 处接缝或已清零边当作 current-source 输入。
- 依赖治理已有文档（`runtime/01-tech-stack-and-dependency-governance.md`）但缺编译单元层面的强制手段。
- 无开发期动态链接选项；重型依赖（wgpu/naga/winit/gltf/image）与纯逻辑代码同一编译单元。

## 3. 目标拓扑与分层规则

```
layer 0a zr_math        数学实现 canonical owner；interface/runtime 只保留批准的产品投影
         zr_resource    resource foundation implementation（零重依赖；依赖 interface resource DTO）
         zr_contracts   core/framework（纯 trait/DTO；按域 feature 门控子模块）
layer 0b zr_kernel      core/runtime + engine_module（生命周期/调度/描述符；依赖 0a，禁止重依赖）
layer 1  zr_diagnostics diagnostic_log
         zr_foundation  foundation（Kernel 级 config/event module；共享原子持久化由 zr_resource::io 拥有）
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
8. `zr_math` 与 `zr_resource` 是 layer-0 foundation implementation crates，不与纯契约混为一类；
   只有 `zr_contracts` 保持纯 trait/DTO。`zr_resource -> zircon_runtime_interface::resource` 是
   本计划批准的同层稳定 ABI DTO 依赖，`zr_resource::io` 继续拥有 Runtime04 已硬切完成的共享
   atomic persistence 实现；禁止把该实现迁回 `zr_foundation` 或复制到 Asset/Platform/Scene。
9. `zr_*` 只是在 `zircon_runtime/crates/` 下的 `publish = false` 私有编译单元，不是第四类产品根包，
   也不是已删除 `zircon_resource` 等 pre-absorption crate 的复活。产品与跨包 canonical surface 仍是
   `zircon_runtime::core::{runtime,framework,manager,math,resource}`；App/Editor/plugins 不得直接依赖
   `zr_*`，Runtime facade 之外不得暴露 workspace-internal assembly surface。

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
- 新建 `zr_kernel`/`zr_contracts`/`zr_math`/`zr_resource`/`zr_diagnostics`；每个 owner 按下述原子
  manifest 物理迁移。`zircon_runtime` 与 `zircon_runtime_interface` 只投影批准的最终产品 API，
  不恢复已删除的顶层 `zircon_core`/`zircon_framework`/`zircon_resource` 等旧包，不恢复未批准的
  `core/*` 内部路径，不留下 forwarding module、alias 或迁移 bridge；
- 按 `zr_math/zr_resource → zr_contracts → zr_kernel → zr_diagnostics` 顺序硬切；`core/framework/render/environment/source_cubemap/tests/projection.rs` 的两处 concrete Runtime `TaskPool` 反向测试引用已硬切到中立 `ParallelSliceExecutor` 测试替身，并由 [`01/2026-07-30-m1-contracts-kernel-test-boundary.md`](01/2026-07-30-m1-contracts-kernel-test-boundary.md) 记录；后续物理迁移必须保持该计数为 0，禁止让 contracts 以 dev-dependency 反向依赖 kernel；
- `zr_math` 不从三行的 `core/math/mod.rs` projection 制造中转 crate。真实数学实现当前位于
  `zircon_runtime_interface/src/math.rs`；物理 hard cut 以 `zr_math` 为唯一 canonical owner，同批迁移实现、
  令 `zircon_runtime_interface::math` 与 Runtime 公共 math surface 仅做批准的 curated projection，并删除
  interface 内的旧实现。原子输入至少包含根/Runtime/interface 三份 Cargo manifest、`Cargo.lock`、
  `zircon_runtime_interface/src/{lib.rs,math.rs}`、`zircon_runtime/src/core/{mod.rs,math/mod.rs}` 及所有直接
  math 消费者、API guards、docs 与 examples；内部 crate 禁止依赖 `zircon_runtime` 或 interface 的 math facade；
- `zr_resource` 的原子输入按 2026-08-16 baseline epoch 321 current source 是 `core/resource` 全部 57 个
  Rust 文件、11,480 行，tree fingerprint 为
  `b06500a6f558b36880d5f051d566dddb054f2bf7bc23b370abdd06f4c16b9538`；
  正式迁移开始前仍须在同一 fingerprint 重新原子采集，但目标归属按职责
  拆分：production data/error/event-stream/io/lease/management-generation/manager/readiness/registry/runtime/
  snapshot 与内部行为测试迁入 `zr_resource`；读取 Runtime 源树、验证 facade/hard-cut 的架构测试迁到
  Runtime integration/absorption guard owner，不能让内部 crate 以 dev-dependency 反向依赖 Runtime。
  `zircon_runtime_interface/src/resource` 的 14 个文件、923 行保持稳定 ABI DTO owner，作为冻结且不迁移的
  下层依赖输入；同时显式冻结 `zircon_runtime_interface/src/lib.rs` 的 resource 根导出、
  `zircon_runtime_interface/src/tests/{mod.rs,resource_contracts.rs}` 及 resource ABI guards，不能依赖模糊
  扫描间接命中 crate-relative 测试。禁止把 interface DTO 复制进实现 crate或删除该稳定 owner；
- resource candidate 必须原子包含根 `Cargo.toml`、`Cargo.lock`、Runtime/interface manifests、
  `core/resource` 全树、interface resource 全树、Runtime facade/root modules，以及通过 literal path/import
  与 structured Rust use-tree inventory 联集找到的 tests、guard scripts、docs、examples、
  App/Editor/plugins consumers。epoch 321 预检中 literal consumers 为 463，structured parser 另发现
  7 个 literal-only 扫描遗漏，联集固定为 470（production 259 / test-only 211）；正式 hard cut 必须重取
  同一 current-source 指纹并以联集 manifest 为准，禁止退回会漏掉 nested use-tree 的 literal-only 清单。
  迁移同批删除旧 implementation directory 并改完消费者；禁止先复制 crate、保留旧实现、留下 forwarding module，或只提交
  未接线的新目录。`io/atomic_file` 继续作为 Runtime04 已锁定的 resource I/O owner，不回迁 Foundation；
  原 963 行 owner 已按 transaction/recovery/platform/directory/pathing folder-backed child owners 拆分，根 façade
  55 行、production child 最大 274 行、测试 owner 180 行；`io::atomic_write` 是 IBL 等上层 consumer 的 curated
  publication 入口。R8 已对 Git tracked 与 nonignored untracked current source 的 31 个 consumer/35 处旧
  `io::atomic_file::*` 引用完成同批 scope-transfer 和根 façade 迁移，并将 `atomic_file` 模块收为私有；不保留
  forwarding module 或 compatibility alias；
- Runtime 上层目前使用的 `approximate_event_bytes`、`ResourceReadinessRow`、atomic-write transaction
  helpers 与 `PendingAtomicWrite` 不扩大为产品公开 API。`zr_resource` 使用
  `#[doc(hidden)] pub mod assembly`（内部条目保持最小可见性）向 sibling Runtime crate 提供组装能力，
  Runtime facade 不 re-export `assembly`；rustdoc/public-API seal 与编译 guard 必须证明外部产品 surface
  仍只包含批准的 resource DTO/registry/runtime/snapshot，禁止为跨 crate 编译把全部内部符号改成公开 API；
- `core/framework` 迁入 `zr_contracts` 时按域拆 feature（ai/physics/sound/net/render/ui/... 各成 feature，默认全开，勾稽计划 03）；
- 移动后同批修正所有 crate 内引用（`crate::core::…` → `zr_kernel::…` 等），不留旧路径别名。

测试阶段：
- 编译门：`cargo check -p zircon_runtime --lib --locked`、`cargo check -p zircon_editor --lib --locked`、`cargo check -p zircon_app --locked`
- 测试门（policy §3 最小批次）：分别执行 `cargo test -p zircon_runtime --lib --locked framework`、
  `cargo test -p zircon_runtime --lib --locked kernel`、`cargo test -p zircon_runtime --lib --locked resource`、
  `cargo test -p zircon_runtime --lib --locked diagnostic` 与 `cargo test -p zircon_runtime_interface --locked`；
  另执行 resource public-API/rustdoc seal 与 literal-path hard-cut guard，全量 lib 回归留给波次收口（policy §4）
- 插件工作区防回归：`cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked`
- 验收证据：以上命令通过；`grep` 证明无 `path = "src/core/framework"` 类残留与迁移桥；增量基线复测记录。
- 文档更新：`docs/zircon_runtime/` 受影响模块文档的 `related_code` 路径、本文件状态表，以及
  `frameworks/{index.md,architecture-overview.md}` 中 layer-0 math/resource implementation 分类与 owner 描述；
  该规范同步是物理迁移前置，不得延迟到代码迁移后的验收清理。

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
- M2 RHI/WGPU failure：[`01/failure-2026-08-02-rhi-wgpu-presenter-and-backend-contract-test-owner.md`](01/failure-2026-08-02-rhi-wgpu-presenter-and-backend-contract-test-owner.md)（2026-08-03 物理 `zr_rhi`/`zr_rhi_wgpu`、curated facade、测试 owner 与旧目录删除已实现；静态门 GREEN，managed Cargo/固定回传 pending，状态 `resolving`）
- 2026-08-13 M1 math/resource 原子迁移合同已同步到本计划、`index.md` 与规范性
  `architecture-overview.md`：`zr_math`/`zr_resource` 明确为零重依赖 implementation owners，
  `zr_contracts` 才是纯 trait/DTO，resource stable ABI owner/root/tests 与全部路径敏感消费者已列入原子
  manifest，assembly 采用不由 Runtime facade re-export 的 hidden workspace-internal surface。该完成项只
  锁定迁移合同；`zr_math`/`zr_resource` 的物理硬切、managed validation 与 milestone commit 尚未开始。
- 2026-08-16 resource preflight 将旧 literal-only consumer 清单修正为 literal + structured use-tree
  联集 470（production 259 / test-only 211），并明确 `zr_resource` 只是 Runtime 内部私有 build-unit，
  不复活旧顶层 `zircon_resource`。当前 R8 immutable scope 不含 manifests 与完整联集所有权，因此没有创建
  占位 crate、forwarding module 或兼容路径；物理硬切、验证与提交仍未开始。

## Code Review 收敛 (2026-07-31)

- `operation` owner 已按当前 `CoreHandle + scene::World` 依赖事实锁定为 layer-3
  `zr_operation`，而不是错误并入 `zr_kernel` 或留在 facade。M2/M3 顺序、依赖验收和
  optional navigation 消费方向已同步。
- `zr_text` 已锁定为 backend-neutral 服务；完整 `wgpu`/`naga`/`glyphon` 实现随 M3
  迁到 `zr_graphics::text_backend`/`zr_rhi_wgpu`，并新增无循环、无 direct GPU dependency
  的机器验收。该决策消除了旧拓扑对 text 直接 GPU 依赖的遗漏。
- M4 已加入 `notify`/`winit`/`zip` prerelease 的 stable-first、exact scoped exception 与到期 RED
  规则，不以全局 allow 或关闭 advisory/bans 绕过治理。

- 当前状态：M0 已在 2026-07-30 对 9,188 个 Runtime Rust 输入完成 pre/post 同指纹的原子快照：7 个根 workspace members、2,391 production refs / 76 domain edges；旧 `core→asset/graphics/scene` 与 internal→facade 生产反向边已经清零。2026-08-03 M2 首个物理切片已创建 `zircon_runtime/crates/zr_rhi` 与 `zr_rhi_wgpu`，删除旧 `src/rhi`/`src/rhi_wgpu` 目录、迁移 12 个后端测试 owner，并将 Runtime 外部面收敛为 `src/rhi.rs` curated facade；2026-08-10 的 sampling contract inversion 又将 `scene→animation` 从 2 条降为 0，当时 current-source 静态审计为 2,669 refs / 70 edges。2026-08-14 对共享工作树 7,433 个 Runtime Rust 文件、1,110,882 行做原子只读复核，前后输入指纹同为 `5a2116bbb0cbbd6bebc8884252afaca817107684e99294c0bd91fbcd072861f3`，当前结果为 2,710 refs / 72 edges；`graphics→scene/ui`、`asset→text`、`scene→animation`、`rhi→rhi_wgpu` 均保持 0。RHI 与 animation 边界均仍在 managed Cargo 验证前的 `resolving` 状态，不代表 M2 accepted。source-cubemap 测试的 concrete `TaskPool` 反向构造已硬切为中立 executor；`zr_operation`、text CPU/GPU 分层和其余 M1/M2 crate 仍待执行。历史依赖图 JSON 尚未取得基线扩写所有权，四份 cold/incremental timings 也仍 pending，因此不声明 M0、M1、M2 或计划 01 完成。
- 2026-08-08 M1 resource-owner 前置已把 `ResourceManagement*` snapshot/query 声明从上层
  `core::framework::asset` 硬切到 `core::resource`，asset contract 与 Runtime/Editor 消费者直接读取
  新 owner，旧声明文件和旧路径 re-export 均删除；这只清除 `zr_resource` 物理迁移前的
  resource→contracts 反向边。首轮独立二次审查为功能 C0/I1/M0、候选归属 C0/I1/M1；简单旧 owner
  alias 漏检修复后的次轮仍为功能 C0/I1/M0，因为 nested use-tree、module alias、glob 与 integration
  test roots 可绕过 guard。词法 token/use-tree/alias-graph 版本的 immutable 复审随后为功能 C0/I2/M0：
  rustfmt 尾逗号使 focused guard 自身 RED，且 `include!` 可从外部文件重建 public compatibility surface；
  该快照已拒绝。当前已前向改为结构化 use-tree consumer invariant，并拒绝 `include!`、外部 `mod`、
  bang/attribute/custom-derive 宏生成面，锁定 asset 的既有 public item 集，并在整个 `core::framework`
  拒绝指向 `core::resource` 的 public use-tree/alias，加入对应 mutation。该版本复审为功能 C0/I1/M0：
  `framework/mod.rs` 可把 external `pub mod asset;` 改成 inline module 后用相对路径绕过逐文件扫描；该快照
  同样已拒绝。当前已锁定唯一、无属性重定向的 external asset module seam，并以跨文件/inline-module
  alias graph 规范化 `crate/self/super` 与真实 `zircon_runtime` crate-self 根；`crate`/`crate::core`/resource
  glob、macro body `pub use`、`extern crate`、`include!`、直接 `#[path]` 及 `cfg_attr(path = ...)` 注入均由
  mutation guard 封印，函数局部 alias 与合法 framework child/external-crate re-export 保持 negative control。
  fresh exact15 immutable 复审 pre/post 指纹均为
  `4d517941d30f210e7df5696007f4b51482dee689b5a5eeb0390eaa28356071ca`，count 15、drift 0、旧 owner
  tombstone `null`，功能结论 C0/I0/M1；M1 指向新 owner 物理 1,584 行而生产声明只有前 346 行。该 Minor
  已在 successor exact scope 中前向修复：该次拆分时生产 owner 为 444 行，测试按 scanner support、hard-cut
  mutation 和 projection 行为拆为 folder-backed `tests/{support,hard_cut,projection}.rs`，根 `mod.rs`
  仅负责导航；该次拆分快照的五个文件分别为 430/892/467/130/3 行。拆分前后生产前 346 行 diff 0、projection
  测试函数体 diff 0，4 个 `include_str!` 均指向现存 owner，Rust 1.94.1 rustfmt 与 scoped diff-check
  GREEN。exact5 独立二审 pre/post 指纹均为
  `54968d6a0d2e1c4dd3138883f385c2200b6d6d78f1c5722bd40d42edf9e19c9a`、drift 0，功能 C0/I0/M0。
  候选归属仍为 C0/I1/M1：两份 Editor consumer 同一 blob 含其他
  Session 的活动日志/状态栏/UI template/投影代际功能及 import-order drift，Frameworks01 不回滚也不
  认领这些外部内容，需真实 owner 集成后基于新 HEAD 刷新原子候选、manifest 与复审。本段为复审后的
  状态写入不属于 exact5 输入；受管 Runtime/Editor validation receipt 仍 pending，
  不提升 M1 状态。
- 2026-08-14 对 resource management 的 64-shard generation/scan/page 做完整算法重审并对照 Unreal
  AssetRegistry、Bevy asset storage、Fyrox resource registry/manager 与 Godot ResourceUID/Cache 分层。
  当前未用静态复杂度直接替换算法；先锁定 1k/10k/100k、均匀/偏斜 workload 与 publish、lookup、scan、
  首屏/深分页测量门。为采集 candidate checks 等工作量而加入的首版 recorder 使 `core/resource` 出现唯一
  2 条 `core::resource -> core::diagnostics` 反向依赖，已拒绝并前向改成 `pub(crate)` query-local metrics；
  direct/alias/glob guard 同批加入，并继续覆盖相对 `super`、crate-self `zircon_runtime` 与 `use crate as ...`
  绕过形式。全局 recorder 删除后遗留的 profiling scan clone 计数清零也已修复：clone 现在同时保留
  游标、yielded 状态和累计 query-local metrics，并由原 scan/clone 后续行与最终计数同态测试封印。
  layer guard 最终并入既有 `hard_cut.rs` 后，current exact5 行数为 production owner 430、support 790、
  hard-cut 566、projection 130、root mod 3。canonical manifest 采用按 path 排序的
  `path<TAB>bytes<TAB>lines<TAB>file-sha256<LF>` UTF-8 输入，SHA-256 为
  `44127ff6379d227f48de27b2fdf06fcf5db10220ffd37fcab4ef0b07d460e9da`。该值已由 PowerShell/.NET 与
  Node 两套独立实现按上述 exact-5 格式复算一致，纠正先前无法由声明输入重现的记录值；五个源文件内容未变。
  旧 owner tombstone 不存在，
  Rustfmt 与 scoped diff-check GREEN。正式 production dependency audit 仍为 2,710 refs / 72 edges，且
  `resource` source domain 的外部边为 0。2026-08-14 18:56 的受管 Windows profiling 验证作业
  `25f05854c2114f1ea657d76fea939358` 已通过准入并进入 `rustc`，但在执行 ResourceManagement 测试前被
  Runtime08 当时缺失的 `scene/ecs/schedule_runner/tests/{typed_worker_structural,worker_callback_order}.rs` 阻断；
  Frameworks01 未越权补文件或回滚其工作。2026-08-14 19:57 只读复核确认这两个 Runtime08 文件已经出现在
  current source，原编译前置已解除，但原 ResourceManagement gate 尚未以新受管作业重跑，故 managed Cargo 仍未 GREEN。
  20:02:35 创建的新受管作业 `ceaaffb6bb374111a92c40aee8cdb722` 复用同一 D 盘兼容池并跨过该 include，
  20:02:37--20:14:00 执行 683.3 秒、端到端 695 秒后以 Cargo exit 101 结束，20:14:03 进程树退出并释放；
  current `zircon_runtime` lib-test 在生成测试二进制前累计 361 个编译错误、1,520 个 warning，ResourceManagement
  测试仍未运行。可见末条错误为外部 Text blob `text/cache/rich_cache.rs:477` 的 `String`/`Arc<str>` 不匹配，
  同期 Runtime 源树共有 1,970 个 dirty path；本计划不把全局 current-source 编译失败冒充 focused 行为失败，也不
  越权修复外部模块。
  高层 asset/scene profiling adapter、
  current-source benchmark 与产品 trace 仍 pending；没有性能样本前不实施 heap merge、secondary index 或
  continuation token 优化，也不声明瓶颈、功耗或跨引擎经验值已经达标。
- 2026-08-14 对 owner 周边 mutation path 的第二轮完整复核发现一个先于查询性能的 Critical 一致性问题：
  `ResourceRegistry::upsert` 遇到同 locator、不同 id 时只从 `by_id` 移除旧记录并覆盖 `id_by_locator`，而
  `ResourceAuthorityWriteGuard` 只把新记录发布到 management generation；旧 id 的 management row、summary、
  payload、runtime slot、readiness 与 Removed event 均未在同一事务中收敛。批量 `register_lazy_records` 更会先
  registry-only 逐项修改、再把全部 outcomes 发布到 projection，因此批内前项被后项 locator 冲突挤出后仍可
  重新进入 generation；`rename` 对已占用目标 locator 还会覆盖 locator map 但保留目标 `by_id` 记录。同 id、不同
  locator 的普通 upsert 又会删除旧 locator 后发送 `Updated(previous_locator=None)`，使 locator 变化绕过唯一的
  `Renamed` 语义和 Asset/Scene 对旧路径的失效处理。该问题属于 authoritative mutation 设计，不允许仅在 projection
  删除旧行做半修。进一步复核确认 authority/management、payload、runtime slot、readiness 分属四把锁：`acquire`
  读取 payload 后才递增 refcount，期间 remove/re-register 可使旧 lease 重建已删除 slot，旧 lease 最后一次 release 还可
  删除同 id 的新 payload；`store_payload` 与 `Assets::load_state` 也存在跨锁混合快照窗口。Unreal 主参考的 `FAssetRegistryState`
  以 ObjectPath 为唯一 key，重复 `AddAssetData` 明确报错，`UpdateAssetData`/`RemoveAssetDatasImpl` 同步维护
  object/package/path/class/tag accelerators；Bevy 的 `AssetInfos` 同样由一个 owner 同步 `infos` 与
  `path_to_index`。Zircon 的前向硬切锁定为单一 coherent `ResourceAuthority` 与 staged `ResourceMutationBatch`：提交前
  一次性校验 id/locator/kind、显式 rename 和批内冲突，以 typed error 保证失败零修改；一次 commit 同步 registry、
  management/readiness generation、payload/runtime residency，并只在 commit 后发布 Added/Updated/Removed/Renamed 事件。
  `ResourceLease` 同批携带 residency token，旧代 release 只能释放自己的代际，不得改变同 id 的新 payload/runtime。
  现有 `upsert_registry_only -> publish_records` split、跨四锁 mutation 和无 `Result` 的 register API 均不保留兼容入口；
  生产迁移面集中在 asset facade insert、project resource sync/batch、ImportedAsset ready dispatch 三个 owner，剩余命中为
  core/inline tests。原 r2 immutable scope 不含 registry/error/lazy/registry/payload/readiness/lease 及上层生产 owner，
  r2 已终结并由 r3 successor 接管；新增依赖解析路径通过 audited ownership transfer 扩展 scope。在该 Critical 修复并完成 collision、same-id locator bypass、
  batch rollback、acquire-remove/re-register residency race、event ordering 与 upward tests 前不接受 M1，也不开始查询算法优化。
- Transaction successor 的实现合同已经锁定：新增 `ResourceMutationBatch`/`ResourceMutationReceipt`，唯一写入口为
  `ResourceManager::commit(batch) -> ResourceResult<ResourceMutationReceipt>`；record update 只允许同 id/locator/kind，
  locator 变化必须使用显式 rename，target occupied、同批重复 id/locator、kind drift 和非法 state transition 均返回
  typed error。commit 对零变化不递增 generation、不发事件；有效 batch 只发布一次 management/readiness generation，
  事件在解锁后按确定顺序发布，订阅者收到事件时必须能读取完整的新状态。payload install 携带 expected record revision，
  供 `ensure_resident` 在 project generation fence 后 compare-and-commit，删除当前 TOCTOU 的 `registry check -> store_payload`
  两段式路径。residency token 同时进入 `ResourceRuntimeSlot` 与 `ResourceLease`；remove/re-register 后旧 token release 是 no-op，
  旧 Arc lease 仍可读但不能重建 slot、扣减新 refcount 或驱逐新 payload。实现测试必须用 barrier 锁定两种合法线性化结果，
  禁止依赖 sleep 的概率测试。
- 2026-08-14 r3 已完成 transaction successor 的生产硬切主体：`ResourceAuthority` 在一把 `RwLock` 下统一
  registry、management/readiness projection、payload 和 runtime slot；所有 live mutation 进入 staged batch preflight/commit，
  旧 registry public mutation、registry-only publish split 和无 `Result` 入口均已删除。Asset 全量/targeted catalog 不是 live authority，
  已迁移到显式 `ResourceRegistryStaging`，staging 对 locator collision、implicit rename 和 kind drift 返回 typed error，完成后一次性发布
  只读 registry；没有恢复 `ResourceRegistry::upsert/remove_by_locator` 兼容面。ImportedAsset 的 31 路 payload 分派已收敛为一次
  erased conversion，project resource sync/builtin/close/targeted commit 均组装单 batch，`ensure_resident` 使用 expected revision CAS。
- r3 的第二轮事务复审发现并前向修复两个同源身份漏洞：live batch 和离线 staging 都曾在同批 remove 后把
  “当前有效记录为空”误当成可重定义身份，使同一 `ResourceId` 能绕过 kind drift 或 explicit rename。两条路径现在都把
  最终 record 状态与 transaction-local `(kind, authorized locator)` 身份锚点分离；remove 只清最终状态，只有显式 rename
  能推进 locator 授权。回归覆盖 remove/re-add kind drift、implicit locator change、explicit rename 后 remove/re-add 的合法
  negative control，以及 staging snapshot 失败不污染原 registry。另将 `ResourceManager::commit` 收敛为
  `prepare_commit(...).commit()`：`PreparedResourceMutation` 在 typed preflight 后持有共享 publication gate，但 registry、
  projection 与 event 在 reservation 提交前均不可见；这为上层持久文件事务提供一次预检、随后无第二次可失败资源校验的接线点。
  reservation 丢弃路径也已显式覆盖：外层文件事务失败时 drop reservation，registry/event 保持零可见且 gate 立即允许后续
  commit，不会因取消而遗留锁。并发测试不再以线程调度下的 `try_recv` 作为阻塞证据，而由 `try_lock` 测试钩子直接证明
  reservation 持有唯一 gate，再由 barrier 覆盖后续 commit 的确定事件顺序。lease 复审同时修复 failed reload 的 last-good
  生命周期：Ready payload 在 Reloading/Error 期间即使最后一个旧 lease 释放也不得被驱逐；Error fallback acquire/release 不得
  把 runtime state 偷改回 Loaded，成功 reload 或显式 remove 才结束旧 payload 代际。第三轮状态复审又发现 catalog 的同状态
  `UpsertLazy(Error)` 会因“非 Ready 即失效”的合并条件删除这份 last-good；现已把 payload invalidation 与 runtime-state projection
  分离：Ready 跌入非 Ready 仍按 catalog 失效合同驱逐，已经处于 Reloading/Error 的诊断或 catalog 更新保留旧 payload，同时
  显式刷新 Error/Reloading runtime state。回归覆盖 failed reload 后 catalog diagnostics refresh，证明 fallback payload 与 Error
  state 同时保持。
- 同一 coordinator-managed Windows check lane（D 盘 pool `f9fef644...`）的三次自然终态把 `zircon_runtime --no-default-features
  --features target-server` 编译错误从 14 降到 12，再降到 3：job `2073ee323c054aa3bf905b5b45effc2a` 执行 213.96 秒，
  job `4c8cfe4b33274318aa2fb793caaa1e6a` 执行 114.13 秒，job `1a4f6ffa6598415a94733b64181f68cd` 执行 151.07 秒；
  最后一轮 9 条 Resource/Asset 自有错误为 0。剩余 3 条均为 Runtime08 活跃 owner 的
  `first_stable_camera_entity` 重复定义（`query.rs`/`detached_entity_batch.rs` 及 `hierarchy.rs` 调用歧义），本计划不抢占其 scope。
  随后 D 盘 check pool `db56c122...` 的 job `32c6a4b503734d86805ef1d4065824a7` 于 23:59:07--00:06:12
  运行 425.84 秒；外层 validate-matrix 调用超时使 supervisor 最终记为 orphaned、exit code 缺失，但 Cargo/rustc 进程树已自然退出，
  最新 `zircon_runtime` fingerprint 的结构化 rustc 输出仍精确只有上述 Runtime08 3 个 error、85 个 warning，Resource/Asset 自有
  error 为 0。该 orphan job 已通过 coordinator release，`live_process_pids=[]`，目标仍保留在 D 盘；它只能证明代码到达同一外部
  编译阻塞点，不是 passed validation ticket。随后同一 D 盘 pool 的 job `d9a80db1a103477981c1d936d9096cfa`
  于 00:45:32--00:46:34 执行 62.15 秒并正常 released，结构化输出仍为 Runtime08 3 errors / 85 warnings、
  Resource/Asset 自有 error 0；它覆盖 reservation 取消、last-good catalog refresh、targeted file preflight 与 runtime test owner
  拆分后的 production 编译面，但 `-SkipTest` 没有编译或执行新增 test owner。事件顺序统一改造后的首轮 job
  `e614bf66c642471cb9145916ec68dbe8` 于 01:34:59--01:37:10 执行 131.04 秒，准确发现本 scope 1 条
  `candidate` 先移动后借用的 E0382；闭包下沉到分支并前向修复后，job `ef6b2a1355bd458c85c1169afae36c0c`
  于 01:39:30--01:41:39 执行 129.92 秒，重新收敛为仅 Runtime08 3 errors / 85 warnings、自有 error 0。三张 job
  均为 `released`、`live_process_pids=[]`；由于 Runtime08 在 lib build 阶段阻断 test target，仍无 passed validation ticket，
  故 M1 继续为 implementation-in-progress，不记 accepted/GREEN。
- ResourceManager 单 authority 事务只解决内存 authoritative mutation；ProjectManager 的 meta/artifact/registry 文件落盘与随后
  ResourceManager batch commit 仍不是一个 crash-safe 的跨磁盘/内存 authority 原子事务。代码复核确认 targeted import 已使用 candidate clone、
  全文件 staging 和进程内 rollback；r4 进一步把 open/watch/import/reimport/close 全部收敛到
  `commit_resource_batch_after_dependencies`，成功顺序固定为 Resource typed preflight/reservation -> targeted staged files commit
  -> source-path index -> project snapshot 与 watcher activation/retirement -> 释放 project state lock -> infallible Resource apply/event。
  locator/kind/revision/state preflight 失败时文件和项目状态完全不变；文件 commit 失败时 drop reservation 且 registry/event 零发布，
  文件成功后不再调用第二次可失败 Resource 校验。close 也不再先退休 watcher 再尝试可失败 Resource commit。对应行为测试在 file closure
  内确认 Resource 不可见并注入文件错误后证明后续 commit gate 可继续，另一 observer 测试以 Acquire/Release flag 证明依赖状态提交
  先于 Resource 事件。该修复消除了进程内“文件成功、Resource collision 失败”和“Resource 事件先于 Project/source 状态”的确定性分叉，
  但没有 crash journal，仍不得称为跨重启原子。r5 已把 full scan 硬切为 projected inventory 与
  `PreparedFullProjectGeneration`：source loop 只构建候选状态和 prepared artifact manifests，changed `.zmeta`、
  manifests 与 asset registry 最后进入同一进程内 rollback file commit；Resource preflight 失败时零文件发布。
  本地 Unreal `FAssetRegistryState` 继续证明 object key 与 package/path/class/tag accelerators 必须在同一 state owner 更新；磁盘恢复
  已把仓库原 AssetMigration 的 append-only 模型硬切为 Core 通用持久文件事务，不在 ProjectManager 复制较弱协议。targeted/full import
  都先准备全部 artifact/meta/index writes，再取得 `PreparedResourceMutation` reservation，随后 durable file commit、
  无失败内存 apply、event/generation publish；进程重启必须先按 journal phase rollback 或完成 cleanup，再从磁盘重建内存 authority。
  R6 非验收实现已完成：旧 AssetMigration `commit/journal/schema/stage` 与 Project `targeted_transaction` owner 删除；Core v4 frame WAL
  使用 immutable intent、校验 transition frame、digest/backup、幂等 rollback、128/64 MiB 有界读取、intent orphan cleanup 和固定 sibling
  OS owner lock。`all_committed` 之前恢复旧代，之后保留新代并 cleanup；Project open 在 registry/package 加载前恢复。Project policy
  只接受 registry、ResourceId `.zasset` manifest 与 asset-root `.zmeta`，Migration policy 只接受 scanner target 和合法 sidecar 配对。
  every-transition tests 已写入但仍未执行，因此 R6 整体保持 validation pending，不提升 M1。
- full-scan 的结构性复核、参考引擎对照、静态复杂度上界、唯一 owner/state machine 与测量门已写入
  [`01/2026-08-15-m1-project-generation-durable-transaction-review.md`](01/2026-08-15-m1-project-generation-durable-transaction-review.md)。
  current source 对已有 sidecar 最多形成约 `6N` 次 metadata 反序列化和至少四次项目树递归遍历；该计数是调用图上界，
  不是性能样本。R5 已完成 projected inventory、caller-owned duplicate normalization、borrowed registry build 与
  `PreparedFullProjectGeneration` 硬切；`scan_and_import.rs` 从 811 行收敛为 332 行，新增 owner 为 115/556 行。
  open/watch/reconciliation/reimport 全部进入 Resource reservation -> file commit -> project/source-path install ->
  Resource apply/event 顺序；全部 inventory sidecar 按 stripe 排序去重加锁并校验 prepare 原文档，changed 文档才写入，
  未改写但参与 registry 投影的文档同样受快照校验。R6 最新完整 fingerprint managed production build
  运行 279.6 s、输出 6 errors / 207 warnings，全部属于当时共享 current source 的外部错误（Scene 3、Graphics 1、
  UI 2），R6 owned production errors 为 0；库未构建、测试未运行，不能记 GREEN。R7 已在不改变算法的前提下
  加入 typed phase/counter：metadata 解析复用原读取 buffer，source/artifact/write metrics 复用提交路径已有数据，
  disabled/capture-inactive 不增加文件 I/O 或 recorder lock。`project_asset_manager/runtime.rs` 从 834 行收敛为
  583 行，resource publication 独立为 281 行；observer/full-generation owner 为 396/680 行。R7 default-feature
  managed build 输出 11 errors / 209 warnings，R7 owned production errors 为 0；新增外部阻断来自 IBL import、
  render scale/upscaler、Scene、Graphics 与 UI。`--features profiling` managed build 输出 13 errors / 190 warnings，
  R7 profiling cfg owned errors 仍为 0，新增 2 条属于 Graphics text profiling 私有 re-export；test target 未生成。
  profiling job 在 coordinator 恢复后已 release，进程树为空，D 盘兼容池 retained；这仍不是 GREEN。
  R7 文档更新后 docs convention gate 为全库 652 条/235 份文档，本父计划与 R7 记录命中 0 条。
  R7 后续 foundation-DAG 复核发现首版 transaction observation 直接依赖 Runtime profiler，会让未来
  `zr_resource -> diagnostics/runtime` 反向；该接线已拒绝并硬切为 Core `DurableRecoveryReport` 与 Project 高层
  profiler adapter。生产 Resource reverse-edge scan 当前为 0，剩余命中仅是 hard-cut mutation 测试字符串。
  live commit rollback 也已改为 caller-owned `DurableCommitReport`：journal 进入 `RollingBack` 后才计 restore attempt，
  restore 成功后另计 success，Project 在结果映射前投影，Migration 忽略报告；不再按 error phase 猜测。
  post-correction profiling managed job `56f4fc9dbc5e4d98bb20c275b7078ee3` 执行约 273 秒、wrapper wall 345.8 秒，
  输出 28 errors / 192 warnings，R7 owned errors 为 0；28 条均属于 IBL/Graphics text/render/Scene/Graphics buffer/
  Script/UI 外部 owner。job 已 release 且进程树为空，但 lib RED 仍阻止 report/crash tests 生成 test target，不能记 GREEN。
  当前 Resource 原子迁移输入
  已增长为 57 个 Rust 文件、11,480 行（interface resource 仍为 14 文件、923 行），父计划旧 31/5,504 数字已失效。
  2026-08-15 HEAD `0f57bdef66a481cb55c52310f80281e4b8909eb9` 的 focused literal path/import inventory
  扫描 Git tracked + nonignored untracked Rust 文件，排除 `dev/target` 和 Resource owner 本身，只匹配显式
  `crate::core::resource` / `zircon_runtime::core::resource`，并在首个独立 `#[cfg(test)]` 处分段。结果为 460 个
  consumer：250 个含生产引用、210 个仅含测试引用；共 565 个显式引用（production 284 / test 281），逐文件
  post-scan SHA-256 drift 为 0。最大域为 graphics 137、asset 112、plugins 60、core 57、scene 37、editor 18。
  完整 JSON 位于 `D:\zircon-frameworks01-r7-zr-resource-consumer-inventory.json`，inventory fingerprint 为
  `e5818ca795236d0223b932617cfe67f9517736babd2ca3b7f8a0cf8686cf0b4b`，report SHA-256 为
  `212d1f5456ba4352f4de305675b4a14938ca26d65918d8cf9e0dd391276d7917`。current domain audit 仍为
  2,733 production refs / 72 edges，但该审计会把 `core/resource` 折叠进 `core`，不能替代 Resource 专项反向边扫描。
  物理 `zr_resource` scope 必须以 current-source 57 文件及当前 union 470 个 consumers 重新生成，不得按旧 manifest 半迁移。
  2026-08-16 baseline epoch 321 的两次独立只读重扫得到同一 current fingerprint
  `3cd3b1c9940561c606f326fae346e7ca325eecd848e0c451aec6c499d450e0a2`：17,180 个候选路径中有
  61 个 tracked-but-deleted foreign 路径，均不是旧 consumer；连续文本 matcher 实际命中 463 个 consumer，其中
  production 253、test-only 210，显式命中为 production 287 / test 286。相较 2026-08-15 清单新增
  `zircon_editor/src/core/hub_link/{focus_signal,handshake,recent_writeback}.rs` 三个 Editor16 生产 consumer，旧清单
  93 个 blob hash 已漂移。`focus_signal.rs` attribution 指向 active Editor16 session 但无 live lease，另外两份仍
  attribution missing；它们必须在物理迁移前完成 audited ownership transfer，不得由 Frameworks01 冒领。
  `editor_event_runtime_access.rs` 当前是 foreign tracked deletion，继续按既有 mixed-blob freeze 处理，不能因为扫描时
  不存在就从最终 hard-cut scope 永久排除。

  连续文本 matcher 只用于 literal-path guard，不足以代表完整 Rust dependency graph；仓库现有
  `runtime_domain_dependency_audit` lexer 屏蔽注释/字符串并展开嵌套 use tree 后，找到 444 个真实 use consumer，
  其中 7 个不在 literal 清单：TextureImporter 1、Graphics 2、Scene 4。新增 use leaf 共 19 个，覆盖
  ResourceId/Manager/event/management query 与 diagnostics；两集合 union 为 470 文件，其中 production 259、
  test-only 211。tracked 与 731 个 nonignored untracked Rust 文件中的 `crate`/`zircon_runtime` alias、root glob、
  `core::*` glob 扫描均为 0；唯一 `extern crate self as zircon_runtime` 位于 Runtime lib 根，已由标准根路径解析覆盖。
  最终 scope 必须使用 470-file union，禁止继续把 463-file literal 清单冒充完整依赖图。

  结构化 use graph 共 83 个 Resource leaf path。产品 façade 的高频输入保持为稳定 DTO/handle/record/locator、
  ResourceManager/registry/snapshot/generation 和唯一 `io::atomic_write`；Runtime-only assembly 明确包含
  `ResourceRegistryStaging`、`ResourceReadinessRow`、`approximate_event_bytes`、fault/stage/sync helpers、durable
  transaction helpers，以及由返回类型跨 crate 使用的 `PreparedResourceMutation`。这些内部条目迁到
  `zr_resource::assembly`，不得从 `zircon_runtime::resource` re-export，也不得为过编译改成无边界产品 API。

  同一输入图确认 `core/resource` production 到其他 Runtime domain 的直接 `crate::` 引用为 0；扫描出的 Asset/
  Diagnostics/Framework 命中全部是 `management_generation/tests/hard_cut.rs` 的 mutation 字符串。该 architecture
  guard 及其 Runtime 源树 reader 必须迁到 Runtime integration/absorption owner，内部行为测试才迁入
  `zr_resource`。interface resource 仍为 14 文件、923 行，tree fingerprint 为
  `f22927513772a6e664676d32a6be7872067cfef05ab97288c41c049d1322c7ad`。R8 immutable scope 不包含根/
  Runtime/interface manifests、`Cargo.lock`、新 crate 路径或全部 470 consumer，故本轮不创建空 crate、不复制实现、
  不留下 forwarding module；必须先完成 coordinator scope rotation，再按同一 current fingerprint 原子硬切。
  全库 docs convention gate 返回 652 条既有 path violation、影响 235 份文档，本父计划与 R7 记录命中 0 条；
  全库 structure convention 组合门在 184.1 s 超时，未取得完整结果，不能记 GREEN。
  R7 observer 采用 Unreal AssetDataGatherer 的 owner-boundary phase scopes 与 Bevy typed/static path、deferred
  measurement 原则，复用 Zircon 现有 profiler，不创建第二套全局 telemetry store。后续仍须执行 R6
  crash/restart matrix、R7 profiling-feature tests、WPR/ETW 样本、focused/upward tests 与独立复审；未取得这些
  证据前不提升 M1，也不实现单遍 discovery/并行 import/索引布局优化。
  atomic owner folder cut 的 Windows `core-min` managed job
  `aa2ae675de25496bb20934955b48daba` 于 14:55:51--14:59:41 运行约 230.2 秒，输出 1 error / 83 warnings；
  唯一错误为 foreign `core/framework/render/camera.rs:84` 缺少 `DEFAULT_RENDER_RESOLUTION_SCALE`，Resource/IBL
  owned errors 为 0。job 已于 14:59:49 release，live process 为空，D 盘 pool retained；日志
   `D:\zircon-frameworks01-r7-atomic-file-folder-build.log` SHA-256 为
   `69db92b7b5906fc489937cd15f8026471652a52051590cad93afd568768273a7`。test target 仍未生成，不能记 GREEN。
  folder cut 后重跑全库 docs convention gate，返回 660 条 path violation、影响 238 份文档；本父计划与 R7
  记录仍命中 0 条。已删除的 `core/resource/io/atomic_file.rs` 被 6 份 foreign 文档引用 8 次，这些外部记录须随
  audited consumer scope-transfer 一并迁到 folder owner，不能在当前 immutable scope 内越权修复。JSON 证据位于
  `D:\zircon-frameworks01-r7-atomic-folder-doc-conventions.json`，SHA-256 为
  `3b535c65438a069e40fa2d19fdc0a65a4d134bba69ffa8e81699387f26f17c86`。
  R8 最终 scope rotation 补入此前漏扫的 scene project document 与 product framebuffer proof consumer；session
  `frameworks01-m1-durable-file-transaction-hard-cut-r8-20260815` 的 immutable scope 为 158 项。ownership
  transfer fingerprint `c34b0bf4aeb11a3c5e8317607c0d5d45a3cfd1913d7d628ceb8d19c4903e0539` 覆盖
  129/129 eligible current blobs，apply request `6e6f1810a7964d85b19a48f1cb829fa2` 原子成功。迁移后全工作区
  Rust 旧模块路径为 0 文件/0 引用，`pub mod atomic_file` 为 0，公开根 `atomic_write` 导出恰为 1；32 个精确
  Rust 文件的 rustfmt check 与 scoped diff-check 均为 GREEN。4 份 canonical 模块文档中的 6 处已删除文件路径
  已迁到 folder owner；全 `docs` 只剩 2 份 foreign 历史 failure 记录中的 3 处旧路径。atomic/durable owner 保持
  20 个 Rust 文件、3,858 行，最大 owner 751 行。该项只完成 R6-D 非验收实现；最终 current fingerprint 的 managed
  compile/test 与独立复审仍 pending，不提升 M1。
  R8 Windows managed `core-min` production job `e4947b70644e4ff7822156eba264a5f9` 于
  16:44:06--16:50:44 执行约 398 秒并以 0 退出，合计 0 errors / 293 warnings，target 仅位于协调器 D 盘 pool。
  随后仅把 production 图未使用的 `NEXT_ATOMIC_FILE_ID` 根重导出收为 `#[cfg(test)] pub(crate)`，所以该 build
  不冒充最终 current-hash 验收票。current-hash managed lib-test job
  `bf4d78e6e007484ea18fadc7e7a92fe9` 于 16:58:48--17:08:43 执行约 596 秒并以 1 退出；完整 test 图在生成
  测试二进制前被 58 errors / 1,352 warnings 阻断，测试数为 0。结构化 rustc fingerprint 中
  `core/resource/io` 与三份 IBL consumer 直接诊断均为 0；58 条错误分属 Asset/Render/ResourceManager/Scene-ECS
  owners，不能由 Frameworks01 越权批量修复。job 已 release、进程树为空，本 Session 未与后续 UI12 managed
  Cargo job 并发。R8 registered scope 内的 10 条 test-contract 漂移随后已按当前生产签名最小修复：4 个拆分后
  `include_str!` 相对路径、2 个 test-only mutation-batch 解析、`ResourceRecord` 导入、2 个 caller-owned durable
  report 参数和 1 个 consumed-record id；4 文件 exact rustfmt/diff-check GREEN，不改变生产算法。剩余 observed
  48 条 foreign error 未改写。修复后的受管复验在进入 Cargo 前因 coordinator `%TEMP%` failure snapshot Windows
  `Access denied` 失败，随后服务为 `offline/descriptor_absent`；没有新 Cargo job 或 test result，也未绕过服务。
  最新全库 docs convention gate 为 657 条既有 violation、影响 238 份文档、检查 74,763 个路径和 2,445 份文档；
  本 Session 精确 6 份 owned docs 命中 0 条。因此 R6-C/R6-D 仍不提升，需要 coordinator rollover、foreign
  lib-test repair、当前哈希受管测试和独立复审后才可进入 milestone closeout。
  本轮已把 `docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md` 与
  `04-core-resource-asset-serialization-review.md` 纳入后续门禁：可信 telemetry M0-M3 完成前不以当前 recorder
  自证优化，Asset exact identity/async residency/semantic section/last-good reload/bundle update 由 Runtime04 与
  对应 numbered plan 分层实现。本 façade hard cut 不冒充算法优化，R7-C/R7-D 的 WPR/ETW/RSS/I/O/功耗与同条件
  p50/p95/p99 证据仍未取得。
- r3 为加入 targeted 跨 authority reservation 接线时触及 1,058 行的
  `project_asset_manager/runtime.rs`，按结构预算不允许继续堆叠。coordinator 已将 r3 以 scope-rotation 原因取消，并注册 r4 完整继承
  blobs/owner；r4 把 247 行内嵌测试机械迁到 folder-backed `runtime/tests.rs`，root production orchestrator 拆分时为 813 行、test owner
  244 行，未增加 public API 或 compatibility module。后续 event-last 测试与统一 orchestration 使 current root/test owner 为 832/283 行，
  仍低于结构预算；两文件、open/close 与 service-contract caller 的 rustfmt/scoped diff-check GREEN。最新 managed production compile
  自有 error 为 0，但 test target 仍受 Runtime08 current-source duplicate method 阻断，故这是 implementation completed /
  managed validation pending，不提升 M1。
- 2026-08-14 production lexical inventory 在每个文件首个 `#[cfg(test)]` 前截断并排除 tests 目录：`register_record`
  与单条 `register_lazy_record` 均为 0 个生产 caller，显式 `rename` 也为 0；`register_ready` 仅有 asset facade 1 处和
  `ImportedAsset` 的 31 个类型 match arms，`register_lazy_records` 仅有 project runtime 3 处，`store_payload` 仅有另一组
  31 个 ImportedAsset match arms。故 successor 必须把两组重复分派收敛为一次 erased `Arc<dyn ResourceData>` 转换，
  project sync 将 remove/lazy/ready 合入一个 batch；不得以调用面大为由保留旧 API。
