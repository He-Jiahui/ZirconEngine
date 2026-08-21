---
related_code:
  - .gitignore
  - examples/woc
  - examples/woc/README.md
  - examples/woc/LICENSES.md
  - examples/woc/zircon-project.toml
  - examples/woc/contracts/world-state.md
  - examples/woc/assets/scenes/bootstrap.scene.toml
  - examples/woc/assets/scenes/eastbrook_mvp.scene.toml
  - examples/woc/native/Cargo.toml
  - examples/woc/native/crates/woc_protocol
  - examples/woc/native/crates/woc_parity
  - examples/woc/native/plugins/woc_runtime
  - examples/woc/native/apps/woc_client
  - examples/woc/native/apps/woc_server
  - examples/woc/native/apps/woc_bot
  - examples/woc/native/apps/woc_headless
  - examples/woc/scripts/woc_game/plugin.toml
  - examples/woc/scripts/woc_game/src/main.zr
  - examples/woc/scripts/woc_game/src/world/state.zr
tests:
  - examples/woc/native/crates/woc_parity/tests/goldens.rs
  - examples/woc/native/plugins/woc_runtime/tests/transaction.rs
  - examples/woc/native/plugins/woc_runtime/tests/hot_reload.rs
  - examples/woc/native/plugins/woc_runtime/tests/identity.rs
  - examples/woc/native/plugins/woc_runtime/tests/client_projection.rs
  - examples/woc/native/apps/woc_server/tests/fixed_tick_driver.rs
  - examples/woc/native/apps/woc_client/tests/application.rs
  - examples/woc/native/apps/woc_client/tests/presentation/frame_driver.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/woc/00-woc-engine-capability-foundation.md
  - docs/plans/woc/01-woc-zrvm-one-to-one-replication.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/GameInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/NetDriver.h
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/main_schedule.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/godot/main/main.cpp
  - dev/godot/scene/main/scene_tree.cpp
  - dev/godot/scene/main/multiplayer_api.cpp
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/PathTracing/RenderPipeline/RayTracingRenderPipelineAsset.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/PathTracing/RenderPipeline/RayTracingRenderPipelineInstance.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 03 · WOC Product Role Host、ZrVM Transaction、World State 与 Client/Server Integration 工程化差距

## 1. 结论

`examples/woc` 不是一个普通小示例：当前物理工作区有 2,416 个文件、98,353,987 bytes，原生工作区包含 8 个 crate、约 46,406 行 Rust 与 513 个测试属性，ZrVM 源码包含 817 个 `.zr`、246,765 行、接近 10 MiB；协议层已有稳定 command ID、payload budget、finite-number 校验、movement acknowledgement、golden hash 和 client projection 校验。这些投入应保留，不能用一次性 demo 重写抹掉。

但当前 WOC 还不是由 ZirconEngine 驱动的产品，更不能作为引擎完整性或性能证据。四个角色 binary 的 `main.rs` 都只有 8 行：定位源码树、调用 `identity_report_json()`、打印 JSON 后退出。Client 不创建窗口、scene、render/UI/audio/input host，Server 不监听网络或运行服务循环，Bot/Headless 同样没有 simulation runner。`woc_runtime` 没有依赖 Zircon runtime/interface/plugin SDK，也没有任何产品实现 `WocProjectVm`；现存实现全部是测试 double。项目 manifest 选择九类引擎插件，并不等于这些 provider 已被装载、注册、运行或关闭。

仓库重建边界本身也已失效。根 `.gitignore` 的 `examples/*` 隐藏整个 WOC；本机有 449 个 ignored 文件，其中 110 个 `.zr`、103 个 `.mjs`、102 个 `.ts`、79 个 `.zrp` 和 11 个 `.rs`。从 tracked `src/main.zr` 可达的 graph 有 29 个本地模块只存在于 ignored overlay，因此 clean clone 即使绕过当前 Rust 编译错误也无法装载当前产品入口。原生 `cargo test --workspace` 本轮又在 `woc_protocol` 编译阶段得到 6 个错误，测试没有开始执行。

最严重的运行时合同是状态 authority 自相矛盾：native protocol 声明 WOS83，Zr package `stateSchema()` 声明 WOS113，`world/state.zr` 当前 writer 写版本 118，decoder 却只接受到 117。也就是说默认 writer 产物会被同文件的默认 decoder 拒绝。与此同时，所谓 transactional wrapper 在调用可变 `fixed_tick(&mut self)` 后才做预算、digest 和 projection 校验，接口没有 rollback token、snapshot restore 或 candidate isolation；失败只能不替换 Rust wrapper 的 `CommittedSnapshot`，无法证明 VM 内部状态回滚。

性能路径同样不是可扩展基线。每个 20 Hz tick 会 clone 最多 64 MiB committed state、再次编码复制为 VM input、接收另一份 state output，并解析最多 16 MiB JSON presentation；仅一次 64 MiB 单向复制的理论上限已是 1.28 GB/s，尚未计入其余复制、分配、JSON、事件与 VM 执行。预算又只检查 VM 自报的 `TickUsage`，不能中止挂死执行或独立测量时间、内存、host call 和 GC。该路径必须被视为结构性 P0，而不是日后微优化。

本轮登记 **9 项 P0、66 项 P1 和 14 项 P2**。首要工作不是继续补充业务模块，而是先让 clean clone 可重建并编译，再建立引擎拥有的 `WocProductHost`、真实 ZrVM adapter、可回滚 state transaction、唯一 schema authority、增量 snapshot/projection、真实 client/server transport 与可执行 parity runner。WOC 自己的 README 已明确当前内容是 partial authored work、不是 playable；本报告把这项诚实声明落实为可验收的工程重构边界。

## 2. 审查边界与证据

### 2.1 物理范围

| 子域 | 物理规模 | Git 可重建状态 | 本轮深度 |
|---|---:|---:|---|
| WOC 全目录 | 2,416 文件 / 98,353,987 bytes | 1,967 tracked / 449 ignored / 0 normal-untracked | E1 全量 inventory；E2-E3 纵向产品链 |
| `assets` | 111 / 29,941,344 | 109 tracked / 2 ignored | scene/bootstrap、M8 closure、字体/图像/音频/模型盘点 |
| `contracts` | 50 / 1,079,156 | 49 tracked / 1 ignored | identity、protocol、world-state 文档 authority 联读 |
| `native` | 196 / 1,505,575 | 185 tracked / 11 ignored | 8 crate manifest、production、test、角色入口逐文件 |
| `reference` | 162 / 18,742,331 | 158 tracked / 4 ignored | current-head catalog/golden/provenance 边界 |
| `scripts` | 1,210 / 18,176,820 | 1,021 tracked / 189 ignored | package entry、world state、import graph、binary artifact |
| `tests` | 52 / 2,469,726 | 52 tracked | Zr/contract fixture 分类 |
| `tools` | 632 / 26,364,026 | 390 tracked / 242 ignored | 本报告只消费 Tooling05 结论，不重复 codegen finding |

根 `.gitignore:128` 的 `examples/*` 使 ignored source 不出现在普通 `git status`。449 个 ignored 文件按主要扩展名为：110 `.zr`、103 `.mjs`、102 `.ts`、79 `.zrp`、20 `.json`、11 `.rs`、9 `.js`，其余为少量资源/文档。它们不能计作 clean-clone 产品能力。

### 2.2 Native workspace

| member | 物理文件 | Rust 行 / 测试属性 | 产品角色 |
|---|---:|---:|---|
| `woc_contract_codegen` | 7 | 1,598 / 14 | typed contract projection；另见 Tooling05 |
| `woc_protocol` | 44 | 15,145 / 89 | wire DTO、payload codec、command catalog、state identity |
| `woc_parity` | 10 | 2,421 / 12 | golden inventory/diff API；当前无 VM runner |
| `woc_runtime` | 15 | 3,730 / 37 | identity、transaction wrapper、client projections；不是 Zircon plugin |
| `woc_client` | 109 | 23,033 / 355 | DOM-free shell/input/preferences/presentation models；main 不运行它们 |
| `woc_server` | 5 | 471 / 6 | fixed tick queue/driver；main 不运行它 |
| `woc_bot` | 2 | 8 / 0 | identity reporter |
| `woc_headless` | 2 | 8 / 0 | identity reporter |

513 个 Rust test attributes 中没有 `#[ignore]`。这只能说明源码有较多 unit/contract fixture，不能抵消 workspace 当前不能编译、无真实 VM adapter、无产品 runner 和关键 parity 测试自比较的问题。

### 2.3 ZrVM package

| 集合 | 文件 / 行 / bytes | import / control-flow 信号 |
|---|---:|---:|
| tracked `.zr` | 707 / 237,361 / 9,602,905 | 产品仓库可见部分 |
| ignored `.zr` | 110 / 9,404 / 375,525 | clean clone 不存在 |
| 合计 `.zr` | 817 / 246,765 / 9,978,430 | 1,941 imports、80,159 `if`、769 test-symbol hits |
| `world/state.zr` | 1 / 68,730 / 3,298,740 | 538 imports、5,662 `if`、1,010 `while`、886 `throw`、1,691 顶层函数 |

从 `src/main.zr` 做静态 import BFS 得到 178 个可达本地 module，其中 149 tracked、29 ignored；这 29 项都由 tracked `world/state.zr` 导入，覆盖 combat talent scaling、chain heal、feral charge、fear DR、next-cast/resource precision、deed/weapon contract、craft/harvest/town-focus/weapon state 与 chat wire。另有 `zr.zircon.math` 是外部 package import，不计入这 29 项。tracked set 另有 558 个未从当前入口可达的 module，其中包含大量测试/fixture，需 owner 分类，不能一概认定死代码。

包目录还提交了 37 个 `.zro`、8,143,561 bytes，但 `plugin.toml` 选择 `execution_mode = "interp"`；这些 binary 没有 product receipt、compiler identity、target/ABI 或入口 materialization 证据，不能作为 clean source graph 缺失的替代品。

### 2.4 动态验证

在 Windows PowerShell 使用独立 managed target 目录执行：

```powershell
cargo test --manifest-path examples/woc/native/Cargo.toml --workspace
```

命令耗时 132.6 秒并在编译 `woc_protocol` 时 exit 101，测试阶段未开始。6 个编译错误为：

1. `command_payload.rs:867/871` 找不到 `ENTER_DELVE_COMMAND_ID`；
2. `market_payload.rs:76/81` 找不到 `MARKET_SEARCH_COMMAND_ID`；
3. `command_payload.rs:1425` 不存在 `CommandPayloadKind::WeaponSkinChange`；
4. `generated_command_payloads.rs:591` 同样引用不存在的 `WeaponSkinChange` variant。

相关常量实际存在于 generated module，但调用文件没有正确建立 import/authority；descriptor 已生成 `WeaponSkinChange`，enum authority 却未生成/未同步。Cargo 本次解析曾改写 `examples/woc/native/Cargo.lock`，成文前已逐行恢复并用 `git diff --exit-code -- examples/woc/native/Cargo.lock` 确认与 `HEAD` 无差异。本轮没有修改任何 WOC production、test、manifest、asset 或 generated artifact。

Tooling05 已记录 `npm run check` 在 typed contract 157 与固定期望 148 处失败，并因 `&&` 短路剩余 21 步；本轮不重复执行未变化的失败 lane，也不重复登记 codegen P0。

### 2.5 参考引擎约束

- Unreal `FEngineLoop` 把 PreInit/Init/Tick/Exit 与模块启停组成产品宿主；`UGameInstance` 持有一局产品生命周期、local player/online session；`UNetDriver` 明确区分 connect/listen、dispatch/flush、connection 和 shutdown。WOC 可以采用不同架构，但 Client/Server 不能只是身份报告程序。
- Bevy `App` 的 runner、plugin finish/cleanup、SubApps 与 fixed schedule 是可组合 owner；`Fixed` time 保留 timestep/overstep，而不是让业务 VM 自报执行成本。它证明 Rust 产品可同时拥有明确 schedule、runner 和 teardown。
- Godot `Main`、`SceneTree` 与 `MultiplayerAPI` 分别拥有宿主阶段、physics/process iteration、scene/multiplayer lifecycle；网络 poll/RPC/peer 不是 DTO 存在即可宣称。
- Fyrox `Executor` 持有 event loop、graphics context、window/headless 和 plugin lifecycle；`Plugin` 接收 init/update/OS/graphics-context 事件。WOC 的 runtime adapter 必须进入同类真实 host callback 链。
- Unity Graphics 本地镜像只用于具体 graphics pipeline 对照：pipeline asset 创建 instance，instance 遍历 camera、执行 command buffer、submit/dispose。它不提供完整 Unity Player/Netcode 参考，因此本报告只用它证明 WOC 当前没有实例化或提交任何 render pipeline，不外推闭源产品宿主。

## 3. 可保留的正确基础

### 3.1 Protocol 已有 typed budget 与稳定 ID 意识

`woc_protocol` 对 command/event/movement/state envelope 使用显式字段顺序和版本检查，多类 payload 有 bytes/items/UTF-8/finite-number 限制，command catalog 与 reference commit 也有固定 identity。重构应把这些能力收敛到单一 schema/IR，而不是退回无类型 JSON command。

### 3.2 Transaction wrapper 已区分 candidate 与 committed projection

Rust wrapper 只有在 output codec、digest、presentation 和 caller projection 都成功后才替换 `CommittedSnapshot`；hot reload 也尝试 save/deactivate/load/migrate/activate/restore，并保留 rollback/cleanup error。这是合理的外层状态机骨架。缺口是 VM 接口没有实现骨架所声称的内部事务语义。

### 3.3 Client projection 有若干真实不变量

Actor projection 要求严格排序、viewer 存在、坐标/旋转/alpha 为 finite；HUD/window projection 也校验 quest/inventory 引用一致性。Client frame driver 对 pending command 和 catch-up tick 有上限。这些检查应保留并前移到 bounded binary decoder/reader，而不是删除后依赖渲染容错。

### 3.4 Reference 与 golden 有 provenance 基础

`reference/current-head` 固定 source commit，54 个 golden 有 SHA-256 和防误更新 guard。问题不是 golden 不值得保留，而是测试没有执行实现产生 actual。后续 runner 应消费同一 provenance 和 expected files。

### 3.5 README 没有把阶段性资产伪装成完整产品

README 明确说明 authored work 是 partial、不能证明 current-head parity 或 playable；`LICENSES.md` 也把资产范围限定为 first Eastbrook Vale MVP closure。这种 truthfulness 应进入 machine-readable capability status、CI 和发行资格，而不是只存在于说明文字。

## 4. P0：产品进入工程化之前必须硬阻断

### WOC-APP-P0-001 · Clean clone 无法重建当前产品入口

根 ignore 规则隐藏整个 `examples/woc`，当前 449 个 ignored 文件不进入普通状态检查。tracked `main.zr -> world/state.zr` 的可达 graph 直接依赖 29 个 ignored 本地模块，因此 clean clone 的解释器入口必然 module-not-found。tracked package/manifest、ignored source 与本机物理目录共同构成了不可审计的隐式产品版本。

必须先删除 broad ignore authority，建立 WOC 局部 ignore 与 source manifest；在 `git archive`/临时 clone 中验证所有 manifest entry、Rust path dependency、Zr import、ZUI/resource URI 和 codegen producer closure。资格门要求 ignored production source 为 0，缺失/重复 module owner 为 0。

### WOC-APP-P0-002 · Native workspace 当前不能编译

本轮 workspace test 在 `woc_protocol` 产生 6 个 E0425/E0599 风格编译错误，所有 513 个测试均未执行。错误横跨 generated descriptor、手写 enum/import 和 weapon-skin新增合同，证明 protocol projection 当前不是原子 generation，也没有 compile gate 阻止不一致进入仓库。

先修 authority 与 generator transaction，不允许只在三个调用点临时补 import/variant。clean clone 必须依次通过 generator check、`cargo check --workspace --all-targets`、`cargo test --workspace`，结果绑定 source/build/generator fingerprint；Tooling05 的 mixed-generation P0 是该修复的前置依赖。

### WOC-APP-P0-003 · 四个 product role 都不是可持续运行的产品

Client、Server、Bot、Headless 的 main 均只打印 identity report 并正常退出；没有 host init、runtime/session、fixed/update loop、signal、window、scene、transport、health、shutdown 或 fatal exit owner。大量 client/server library model 从未被 binary 调用，角色名称因此只是一组 artifact label。

需要引擎拥有的 `WocProductHost` 与 `ProductRoleDescriptor`：Client 进入真实 window/render/input/audio/network loop；Server 进入无窗口 fixed schedule/listen/admin/health loop；Bot/Headless 明确连接方式、simulation owner 和退出条件。每个角色都必须有 startup-ready-terminal receipt、signal/fault/drain 和 packaged-artifact 测试。

### WOC-APP-P0-004 · 没有真实 ZrVM adapter、WOC plugin registration 或 materialization

`WocProjectVm` 只有 trait 与测试 double，仓库没有 production impl。`woc_runtime` 不依赖 Zircon runtime/interface/plugin SDK，没有 registration、extern ABI、capability、host service binding 或 unload callback。identity checker 只确认 TOML 中存在 `zr_vm_language`/`woc_runtime` selection；它不构建 resolved plugin graph、admit provider、载入 package 或调用 `activate/fixedTick`。

必须实现 versioned ZrVM host adapter 和 canonical WOC plugin contribution，使用 Zircon plugin admission、BuildSet、capability、allocator/handle generation、panic/trap containment、budget/cancellation 与 unload quiescence。没有真实 provider 时产品 role 必须 Unavailable/fail-close，不能继续打印 valid identity 即 exit 0。

### WOC-APP-P0-005 · World-state identity、writer 与 decoder 相互矛盾

native authority 为 WOS83；Zr `stateSchema()`/README 为 WOS113；`world/state.zr::encodeState()` 当前写 118，但 `decodeState()` 的 accepted chain 只到 117。当前默认 roundtrip 会拒绝自己的 writer。旧 foundation/master plan 还保留 WOS39/WOS65 等阶段编号，说明 schema truth 已分裂到 source、generated contract、native constant、README 与计划。

必须选择唯一 qualified schema authority，生成 Rust/Zr/docs projection；对当前 writer 版本先建立 encode/decode golden roundtrip，再做 hard cut 或显式 migration chain。任何 package/host/snapshot/handshake identity 不一致都应在执行前拒绝，不能靠 README 解释“native 尚未升级”。

### WOC-APP-P0-006 · “Transactional tick / complete rollback” 无法由接口证明

`fixed_tick(&mut self, ...)` 可以先修改 VM 内部状态，wrapper 随后才校验 usage、output、digest 与 presentation。任一 trap、budget overrun、decode/digest/projection failure 都只保留旧 Rust snapshot，没有 API 恢复 VM。Zr `main.zr` 的 `saveState()` 又只返回独立 `restoredState` 字符串，`fixedTick()` 从不更新它，无法代表实际 `world/state`。

需要 engine-owned candidate VM/state generation：tick 必须在 snapshot/COW transaction、forked instance 或可验证 undo journal 中运行；commit 返回 generation receipt，失败执行 rollback 并比对 state digest。hot reload 与 full snapshot install 也必须同步 VM internal state 和 wrapper committed state，而不是更新其中一侧。

### WOC-APP-P0-007 · 全量 state/JSON tick 与自报预算不能达到工程级性能/隔离

每 tick clone 最多 64 MiB state，重新编码完整 input，VM 再返回完整 state；presentation 上限 16 MiB JSON 并经过分配型 `serde_json`。20 Hz 下单个 64 MiB copy 就是 1.28 GB/s，真实路径至少多次复制。`TickBudgets` 只在 VM 返回后检查其自报 `TickUsage`，挂死、谎报、host-call storm 或 allocator 失控不能被阻断。

必须改为 immutable snapshot/artifact handle、paged COW/delta、arena/borrowed buffer 与 bounded binary projection；VM host独立测量 wall/CPU/memory/GC/host call，支持 fuel/deadline/cancel/interrupt。性能门要报告 p50/p95/p99、RSS/working-set、allocation、copy bytes 和 recovery，不得用功能裁剪后的空角色与 Unreal 比较。

### WOC-APP-P0-008 · Client/Server 没有网络、持久化或 authoritative service 产品链

项目选择 Net plugin，但 Server crate没有网络依赖或 socket，Client没有 transport，`woc_runtime`也没有 net adapter。`FixedServerTickDriver` 只收集内存命令并调用 generic VM；没有 authentication/session、listen/connect、packet ordering/replay、snapshot replication、reconnect/resync、PostgreSQL、admin、world persistence、backup 或 graceful failover。

必须建立独立 authoritative server product：secure transport/session handshake、command admission、ordering/idempotency、fixed simulation、replication/delta/snapshot、durable journal/checkpoint、recovery、health/admin 与 drain。Client shell effect 只能在真实 transport consumer 存在后标记 Available。

### WOC-APP-P0-009 · Parity 测试不执行 WOC、ZrVM 或产品角色

54 个 golden 的存在/hash测试是有效 inventory gate，但所谓 double-run 测试读取 expected 后让 closure 两次返回 `expected.clone()`，只证明同一 JSON clone 等于自身。由于没有 production `WocProjectVm`，当前也没有任何 current-head scenario 实际进入 Zr interpreter、Rust wrapper、Client/Server 或 renderer。

必须建立真实 parity runner：固定 BuildSet/package/artifact/seed/input，分别执行两次 real VM 并比较 determinism，再与 golden 比较；失败记录 first divergence、event/state/projection digest 与 trace。随后增加 server-client journey、offline save/reload、pixel/input/audio 和平台 soak，不得把 fixture self-comparison作为“one-to-one replication”完成证据。

## 5. P1：Workspace、BuildSet 与 Identity

| ID | 当前差距 | 必须重构的内容 |
|---|---|---|
| WOC-APP-P1-001 | 物理目录比 tracked set 多 449 个文件，产品状态依赖本机 overlay | 生成 `WocSourceManifest`，记录 tracked/producer/artifact/fixture/retired 分类与 hash；clean clone 是唯一 release input |
| WOC-APP-P1-002 | `examples/*` 让后续新增 production 文件默认不可见 | 改为 WOC-owned allowlist/局部 ignore；CI 拒绝 ignored source/import/manifest entry |
| WOC-APP-P1-003 | 29 个缺失 module 都集中从 68,730 行 world god-file 引入 | import resolver 输出完整 dependency graph、owner、cycle、missing、entry reachability；禁止运行期临时发现 |
| WOC-APP-P1-004 | 558 个 tracked module 未从产品 entry 可达，test/fixture/library/obsolete 混合 | 为每个 package module声明entry/library/test/fixture/archive；unclassified unreachable 固定为 0 |
| WOC-APP-P1-005 | 37 个 `.zro` checked-in binary 与 interp mode 并存 | 定义 source/artifact边界、compiler/target/ABI/dependency hash和retirement；禁止无receipt binary被产品发现 |
| WOC-APP-P1-006 | Rust lock、JS generated output、Zr source与reference commit没有共享 BuildSetId | 建立 source+lock+generator+schema+plugin+target qualified BuildSet，贯穿role receipt/parity/crash/profile |
| WOC-APP-P1-007 | identity report只验证两项selection存在 | 对全部 required rendering/texture/animation/gltf/navigation/net/sound/ZrVM/WOC provider做resolve、admit和capability result |
| WOC-APP-P1-008 | role到target mode映射把Bot/Headless折叠为server语义 | 每个role声明artifact、host services、platform、network、render、persistence和terminal policy，不能仅复用字符串target |
| WOC-APP-P1-009 | source-root由 `CARGO_MANIFEST_DIR/../../..` 推导 | packaged artifact必须消费安装manifest/resource mount；禁止依赖Cargo源码布局和可写源码树 |
| WOC-APP-P1-010 | README、contract、native constant、Zr source与plan都能自称schema authority | machine-generated identity projection；人类文档只引用qualified authority与current receipt |
| WOC-APP-P1-011 | WOC capability只在README用“not playable”降级 | 将 role/capability 状态写入typed descriptor：Unavailable/Experimental/Ready及reason/evidence，Hub/Editor/CLI统一消费 |

## 6. P1：Product Host、Lifecycle 与 Plugin

| ID | 当前差距 | 必须重构的内容 |
|---|---|---|
| WOC-APP-P1-012 | Client main不调用 `woc_client::application` 或任何shell/model | ProductHost创建client application，绑定window/input/UI/render/audio/net，事件驱动更新而非孤立model tests |
| WOC-APP-P1-013 | Server main不创建 `FixedServerTickDriver` | server runner拥有clock、command inbox、VM session、replication/persistence与shutdown lifecycle |
| WOC-APP-P1-014 | Bot没有agent policy、observation/action adapter或transport | 定义bot principal/session、bounded observation、action validation、seed/replay和训练/评估隔离 |
| WOC-APP-P1-015 | Headless与Server都是identity reporter，没有用途差异 | Headless明确为local deterministic simulation/test product，Server为network authority；artifact和capability分离 |
| WOC-APP-P1-016 | runtime没有 initialize/ready/running/draining/stopped状态机 | 使用O02 lifecycle与typed terminal receipt，所有阶段有deadline、cancel、幂等cleanup和fatal exit code |
| WOC-APP-P1-017 | 没有OS signal/service-control owner | Windows console/service、Unix SIGINT/SIGTERM、container stop进入同一drain流程，并有forced-timeout策略 |
| WOC-APP-P1-018 | 没有 package activate/deactivate 与 role host联动 | admission成功后activate，停止接单后deactivate；trap/partial activation执行逆序rollback |
| WOC-APP-P1-019 | plugin manifest宣称 Windows/Linux/macOS/Android/iOS/Wasm，无对应artifact/evidence | platform capability从实际build/run/package矩阵生成；无证据保持Unavailable |
| WOC-APP-P1-020 | `execution_mode="interp"` 没有解释器版本、fuel、JIT/AOT policy | role descriptor固定VM implementation、compiler/interpreter ABI、optimization、sandbox和artifact identity |
| WOC-APP-P1-021 | WOC runtime与ZrVM lifecycle没有 unload quiescence | 追踪VM instance、host callback、allocation、worker与snapshot lease；全部归零后才释放package/provider |
| WOC-APP-P1-022 | identity failure使用 `expect` panic | startup错误进入structured diagnostic/exit receipt，保留path/schema/plugin原因且不泄露secret或产生部分启动 |

## 7. P1：Transaction、World State 与 Performance

| ID | 当前差距 | 必须重构的内容 |
|---|---|---|
| WOC-APP-P1-023 | `CommittedSnapshot` 的state/presentation是可任意clone的`Vec<u8>` | 使用immutable generation handle、lease和明确retirement；consumer不能复制修改authority bytes |
| WOC-APP-P1-024 | `prepare_tick`先clone committed state再encode | VM input引用snapshot handle或paged view，只编码command/movement delta和qualified base generation |
| WOC-APP-P1-025 | VM output每tick返回完整state | 定义delta/write-set、event journal或COW page result；commit验证base generation和changed ranges |
| WOC-APP-P1-026 | presentation每tick为最多16 MiB JSON | 采用versioned bounded binary/columnar projection；字符串intern、entity/item IDs、零拷贝reader与增量publication |
| WOC-APP-P1-027 | JSON在结构数量验证前已完成全量分配 | decoder先验证bytes/depth/items/string长度，再按budget arena materialize；拒绝压缩炸弹/巨大cardinality |
| WOC-APP-P1-028 | FNV-1a 32-bit 被用于state/event/presentation digest | 区分快速checksum、content identity、corruption与authenticated integrity；BuildSet/artifact使用抗碰撞digest |
| WOC-APP-P1-029 | full snapshot install不验证event digest、schema、package或tick/generation单调性 | 引入qualified `SnapshotReceipt`，含BuildSet/schema/package/world/base/target generation与全部payload digest |
| WOC-APP-P1-030 | install只替换wrapper committed，不同步VM internal state | VM restore与wrapper publish组成一次事务，失败恢复两侧last-good并返回recovery receipt |
| WOC-APP-P1-031 | hot reload成功只换VM/清presentation，不证明migrated state等于committed world | migration消费source/target schema和committed snapshot，产出new state digest；双实例切换后才退休旧VM |
| WOC-APP-P1-032 | hot reload用自由字符串schema进行相等/迁移判断 | 使用qualified schema ID、version range、migration graph、unknown-data policy和roundtrip fixture |
| WOC-APP-P1-033 | 单个world file拥有1,691函数、534公共字段及跨十余domain逻辑 | 按kernel/world/combat/progression/social/instances owner拆模块；固定schedule、state shard与跨域command/event合同 |

## 8. P1：Protocol、Network、Security 与 Persistence

| ID | 当前差距 | 必须重构的内容 |
|---|---|---|
| WOC-APP-P1-034 | `NetworkEnvelope`仅有version/kind/sequence/ack/payload | 增加session/principal/world/channel/schema、flags、payload length、integrity和trace identity |
| WOC-APP-P1-035 | protocol version只允许exact equality | 定义handshake capability/version negotiation、compatible range、migration与明确拒绝原因 |
| WOC-APP-P1-036 | 没有secure transport/authentication binding | TLS/平台transport后绑定authenticated principal、session key、server identity与replay window |
| WOC-APP-P1-037 | 没有packet framing/fragmentation/compression policy | bounded frame、MTU/fragment、compression ratio limit、channel QoS和malformed-packet budget |
| WOC-APP-P1-038 | command去重仅覆盖pending batch，tick后清空 | server维护per-session sequence/idempotency/replay window、ack history和resync cursor |
| WOC-APP-P1-039 | movement high-water ack不等于可靠ordering | 明确late/duplicate/loss/reorder策略、input delay、prediction correction和authority tick mapping |
| WOC-APP-P1-040 | 无replication interest/relevancy/delta | world authority生成per-connection view、baseline/delta、bandwidth budget和full resync |
| WOC-APP-P1-041 | 无disconnect/reconnect/session resume | session lease、resume token、snapshot/event cursor、timeout和ownership transfer可测试 |
| WOC-APP-P1-042 | 无durable world/character persistence | journal+checkpoint、schema migration、atomic commit、backup/restore、corruption检测与recovery drill |
| WOC-APP-P1-043 | Auth model以普通可Clone/Debug String保存password/reset token/2FA | Secret wrapper禁止Debug/Clone，最短寿命、zeroize、OS secure storage、redacted diagnostic和clipboard policy |
| WOC-APP-P1-044 | Auth/realm/character effect没有transport consumer、RBAC或audit | typed online operation绑定principal、deadline/cancel/idempotency、server receipt、rate limit和security audit |

## 9. P1：Client、Presentation、UI 与 Assets

| ID | 当前差距 | 必须重构的内容 |
|---|---|---|
| WOC-APP-P1-045 | Client shell是纯state/effect model，没有retained UI host | 绑定ZUI/runtime UI tree、layout/hit/input/render/a11y同代publication，effect进入真实operation dispatcher |
| WOC-APP-P1-046 | `UiSurface`等真实runtime UI入口在client production中为0 consumer | 建立client root surface、viewport/scale/safe-area、focus/nav/IME/a11y owner与window lifecycle |
| WOC-APP-P1-047 | 默认scene只是一台320-byte bootstrap camera | role启动必须load qualified Eastbrook scene artifact、等待asset/renderer readiness并处理失败/取消/卸载 |
| WOC-APP-P1-048 | 115,884-byte Eastbrook scene存在但没有product caller | project/role manifest显式声明startup world和fallback，cook验证dependency closure与resource URI |
| WOC-APP-P1-049 | 当前资产只有26 GLB，而reference catalog记录949项 | 保持MVP truth；建立缺口/许可/provenance/import/cook/LOD/material/animation closure，不用placeholder声称完整复刻 |
| WOC-APP-P1-050 | 没有audio device/mixer/spatial/music/UI sound consumer | product host绑定sound provider、listener/world lifecycle、streaming和device loss；缺失时能力Unavailable |
| WOC-APP-P1-051 | input intent/keybind/gamepad/touch只有model与storage tests | 接入OS device/user、event timestamps、mapping context、rebinding conflict、focus/capture和disconnect lifecycle |
| WOC-APP-P1-052 | presentation只校验部分finite/sort/reference不变量 | 增加actor/string/item/quest cardinality，health<=max、cooldown<=total、quest<=required、inventory<=capacity等语义门 |
| WOC-APP-P1-053 | presentation schema精确v2且没有migration/unknown policy | version negotiation、compatible reader、migration与fail-closed diagnostic；projection identity绑定world generation |
| WOC-APP-P1-054 | fixed 20 Hz到60 Hz presentation只有local interpolation model | 真实clock sync、jitter buffer、authority/prediction/reconciliation、camera cut/teleport和late snapshot策略 |
| WOC-APP-P1-055 | 没有renderer/camera/command-buffer/submit consumer | 将published scene/projection转为RenderScene，按view submit并覆盖resize/surface/device loss；Unity Graphics只作pipeline lifecycle对照 |

## 10. P1：Test、Parity、Platform 与 Operations

| ID | 当前差距 | 必须重构的内容 |
|---|---|---|
| WOC-APP-P1-056 | 513个test attributes因compile failure全部不可达 | required test plan先过compile；报告started/completed/omitted/failed与source/build/test inventory |
| WOC-APP-P1-057 | parity API允许任意closure，测试用expected clone冒充runner | runner参数必须是artifact/VM/session factory，actual包含执行receipt且禁止expected进入producer closure |
| WOC-APP-P1-058 | golden只覆盖54个JSON场景，没有real world save/reload | 增加schema roundtrip、old-version migration、snapshot restore、hot reload/rollback和corruption fixture |
| WOC-APP-P1-059 | 没有client/server进程级journey | 启动server、连接client、auth/select/create/enter world、command、reconnect、shutdown全链自动化 |
| WOC-APP-P1-060 | 没有像素、UI输入、音频或asset streaming acceptance | 固定scene/camera/input/golden，真实GPU窗口/离屏双lane并记录artifact/device/driver identity |
| WOC-APP-P1-061 | 没有determinism跨进程/平台/优化级验证 | 同seed/input在debug/release、Windows/Linux、interp/AOT允许矩阵比较state/event digest和first divergence |
| WOC-APP-P1-062 | 没有tick overrun/backpressure/slow-client/VM hang故障注入 | 覆盖deadline interrupt、queue saturation、drop/resync、OOM、trap、network partition和disk-full recovery |
| WOC-APP-P1-063 | manifest声称mobile/wasm但无build/package/run lane | 每个平台真实artifact、startup/suspend/resume/input/storage/network/render gate；未覆盖平台不发布能力 |
| WOC-APP-P1-064 | 没有长期server soak、memory growth或world unload/reload证据 | 24h+ soak记录tick p99、RSS、allocation、snapshot size、queue、GC、disconnect和recovery |
| WOC-APP-P1-065 | 没有同场景同画质同硬件竞争基准 | 固定workload/quality/correctness oracle，报告CPU/GPU/frame/RSS/VRAM/I/O与统计置信，空host结果无效 |
| WOC-APP-P1-066 | README的千行里程碑叙述不是machine-readable release qualification | 建立WOC capability/evidence manifest、owner dashboard和promotion gate；每项claim链接可重放artifact |

## 11. P2：完成度与维护性

| ID | 差距 | 收敛方向 |
|---|---|---|
| WOC-APP-P2-001 | 四个main重复相同root/identity模板 | 在真实ProductHost落地后由role descriptor生成薄入口 |
| WOC-APP-P2-002 | `expect`文本把产品名写死 | structured startup diagnostic统一格式、localization和exit mapping |
| WOC-APP-P2-003 | `RuntimeRole`与`WocHostRole`是两套近似枚举 | 分清product role和simulation authority role，提供显式validated mapping |
| WOC-APP-P2-004 | 默认budget常量直接写在transaction module | 进入versioned workload/profile配置并记录来源，禁止调用方随意扩大 |
| WOC-APP-P2-005 | identity/schema返回自由字符串JSON | 使用typed descriptor和canonical codec，CLI仅做presentation |
| WOC-APP-P2-006 | 多处错误reason为无结构String | 统一code/span/stage/source/correlation/retryability，不把secret写入reason |
| WOC-APP-P2-007 | projection enum/字段命名混合业务与wire细节 | 划分wire DTO、validated domain projection与render/UI view model |
| WOC-APP-P2-008 | world-state注释中的大量WOS编号容易被误读为有效schema | 生成schema changelog，只从authority列出released/readable/writable版本 |
| WOC-APP-P2-009 | source plan长期保留过期WOS数字 | plan records标记historical，不参与current identity；索引显示current successor |
| WOC-APP-P2-010 | bot/headless极小crate仍各有独立手写Cargo重复 | role artifact由workspace模板/manifest生成，但保留独立binary identity |
| WOC-APP-P2-011 | scene启动路径散在代码/contract叙述 | project manifest拥有typed startup-world selection与fallback policy |
| WOC-APP-P2-012 | reference asset inventory与product closure难以快速比较 | 生成licensed/imported/cooked/used/missing矩阵，不把数量直接等同质量 |
| WOC-APP-P2-013 | 测试目录命名有深层重复module树 | 测试按contract/unit/integration/product/fault/perf分层并由manifest选取 |
| WOC-APP-P2-014 | WOC review范围跨App/Runtime/Tooling容易重复finding | 本篇拥有product integration；Tooling05拥有codegen；后续Runtime篇只拥有Zr/world内部算法与module结构 |

## 12. Owner 与依赖收敛

| Owner | 本篇责任 | 前置 |
|---|---|---|
| O00 Capability Truth | role/provider/playable状态，identity valid不等于ready | WOC-APP-P0-003/004 |
| O01 Build Set | clean clone、source/generator/plugin/target identity | P0-001/002，Tooling05 |
| O02 Lifecycle | ProductHost、VM/session、signal、drain、terminal receipt | P0-003/004 |
| O03 Schema | protocol/world/projection qualified schema与migration | P0-005 |
| O04 Artifact | Zr source/bytecode、scene/asset、snapshot artifact | O01/O03 |
| O05 Transaction | tick、snapshot install、hot reload、persistence rollback | P0-006 |
| O06 ABI/Plugin | ZrVM adapter、WOC contribution、foreign memory/unload | P0-004 |
| O07 Budget | producer-side VM/network/codec/queue/resource预算 | P0-007 |
| O08 World | fixed schedule、world authority、state shard、generation | P0-005/006 |
| O09 Graphics | scene-to-RenderScene、view/camera/submit/device recovery | O02/O04/O08 |
| O10 UI/Input | client retained UI、publication、IME/a11y/device user | O02/O03 |
| O11 Evidence | parity、profile、crash、determinism、performance | 全部P0完成后 |
| O12 Network/Online | principal、transport、replication、reconnect | P0-008 |
| O14 Delivery | required lanes、platform artifact、promotion | O01/O11 |
| O15 Security | secret、trust、sandbox、secure transport、audit | O06/O12 |

实施不得把这些owner重新塞进 `world/state.zr` 或 `woc_runtime::transaction`。ProductHost只编排，不拥有world codec；VM adapter只执行，不拥有online principal；Server只消费qualified snapshot，不自行定义schema；Parity只消费runner receipt，不把expected传回producer。

## 13. 重构里程碑

### M0 · Reproducibility、Compile 与 Truth Freeze

- 收回 `examples/*` broad ignore，分类449个ignored文件；
- clean clone验证import/resource/Cargo/package完整闭包；
- 修复protocol generator authority并通过all-target compile/test；
- 冻结唯一BuildSet/schema/capability manifest；当前角色保持Unavailable。

### M1 · Engine-owned Product Host 与 VM Admission

- 定义四个角色artifact/runner/host-service/terminal matrix；
- 实现ZrVM production adapter、WOC plugin registration和resolved graph admission；
- 接入activate/deactivate、budget/cancel、trap、foreign ownership和unload quiescence；
- Client/Server至少启动真实但最小的持续loop，不再以identity JSON作为产品。

### M2 · World Schema 与 Transaction Hard Cut

- 收敛WOS83/113/118/117为唯一writer/readers/migration authority；
- 建立snapshot handle、candidate/commit/rollback和qualified receipt；
- 修复Zr save/restore使其对应真实world state；
- hot reload/full snapshot通过故障注入证明两侧一致和last-good恢复。

### M3 · Incremental State、Projection 与 Enforced Budget

- 删除每tick全量state clone/encode/output；
- 使用paged COW/delta/event journal、bounded binary projection和arena reader；
- VM host独立计量并可interrupt；
- 建立20 Hz simulation/60 Hz presentation的allocation/copy/latency基线。

### M4 · Real Client、Server 与 Product Systems

- Client接入window、startup scene、UI/input/render/audio与online transport；
- Server接入listen/auth/fixed simulation/replication/persistence/admin/health；
- Bot/Headless建立明确runner与observation/action或local simulation合同；
- completion能力只来自实际provider、artifact和运行期reader。

### M5 · Parity、Platform 与 Competitive Qualification

- 54个current-head scenario执行真实VM双跑与golden比较；
- 增加process journey、save/reload、network fault、pixel/input/audio、soak和platform lanes；
- 所有结果绑定BuildSet、artifact、device/driver、schema、seed和完整test inventory；
- correctness/recovery/quality一致后才执行与Unreal等引擎的同负载性能比较。

## 14. 产品资格门

| Gate | 验收条件 |
|---|---|
| WOC-G01 | clean `git archive`中WOC source/import/resource/build闭包完整，ignored production source为0 |
| WOC-G02 | generator check、native all-target check与513项测试实际完成，0 omitted/compile error |
| WOC-G03 | Client/Server/Bot/Headless各有独立可运行artifact、ready和terminal receipt |
| WOC-G04 | 所有required plugin selection有provider/artifact/admission/runtime generation，缺失fail-close |
| WOC-G05 | 仓内存在并运行production `WocProjectVm` adapter，不再只有test double |
| WOC-G06 | world writer输出能被current reader roundtrip，所有supported旧版有migration golden |
| WOC-G07 | tick任一trap/budget/decode/projection故障后VM与wrapper状态均等于last committed |
| WOC-G08 | hot reload/full snapshot failure不会产生VM/wrapper mixed generation |
| WOC-G09 | steady tick不复制完整world，copy/allocation/state/projection bytes进入profile receipt |
| WOC-G10 | VM budget由host测量并可interrupt，不依赖guest自报后验判断 |
| WOC-G11 | Server真实listen/auth/tick/replicate/persist/drain；Client真实connect/resync/disconnect |
| WOC-G12 | credential不进入Debug/Clone/plain storage/log/crash，transport绑定authenticated principal |
| WOC-G13 | Client真实装载Eastbrook startup scene并提交RenderScene/UI/audio/input |
| WOC-G14 | 54个golden由real VM actual生成，double-run输入不包含expected value |
| WOC-G15 | state/event/projection divergence报告first tick/path/digest/trace，结果可重放 |
| WOC-G16 | Windows/Linux及声明平台的build/package/run能力与evidence一致，无证据即Unavailable |
| WOC-G17 | server soak、network partition、VM hang、OOM/disk-full/device-loss故障有恢复证据 |
| WOC-G18 | competitive report使用同场景同画质同硬件，包含CPU/GPU/RSS/VRAM/I/O与统计分布 |

## 15. 状态与边界

| 项目 | 状态 | 证据 |
|---|---|---|
| WOC product integration首轮审查 | review_complete | 2,416物理文件；8 crate、817 Zr module及role/VM/state/network/client/parity纵向追踪 |
| Finding | review_complete | 9 P0 / 66 P1 / 14 P2，ID在本篇唯一 |
| Native动态门 | blocked_by_current_source | 132.6秒后`woc_protocol` 6个compile error；0 tests executed |
| Tooling动态门 | inherited_blocker | Tooling05 `npm run check` 157/148失败并短路21步；未重复执行 |
| Production修改 | none | Cargo验证副作用`native/Cargo.lock`已恢复并确认与HEAD无差异 |
| 实施 | pending | 本篇只做review、owner路由与重构计划 |

本篇只拥有 WOC 产品角色、宿主、VM transaction、state identity和Client/Server产品集成差距。WOC codegen/build-script/generated-artifact问题由Tooling05拥有；后续 `zircon_runtime` WOC报告应继续逐模块审查world/combat/progression/social/instances内部算法、schedule和数据结构，不重复本篇产品P0。
