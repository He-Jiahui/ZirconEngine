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
- `core/framework` 不得按目录名整体迁入 `zr_contracts`。2026-08-24 schema-1 可复现审计的迁移前基线为
  205 个 declaration-only、380 个 declaration/behavior mixed 与 22 个 behavior-only production 文件；
  原样移动会把 render/animation/picking 等实现算法错误下沉到“纯 trait/DTO”层。
  物理 crate move 前先在当前 Runtime 中完成 declaration/behavior hard partition：`zr_contracts` 只接收
  trait、error、opaque handle、descriptor、immutable snapshot/receipt 与维护自身不变量的轻量 DTO 方法；
  compiler/evaluator、IBL/PMREM、post-process、camera ordering/sequence、virtual-geometry stream decode、
  picking resolution 等行为随 animation/graphics/scene 等 implementation owner 走，旧文件不保留 forwarding
  module、alias 或副本。分区后再按域拆 feature（ai/physics/sound/net/render/ui/... 各成 feature，默认全开，
  勾稽计划 03）；
- runtime-wide finite-state machine 已作为首个 physical partition 整体硬切到
  `core/runtime/state_machine`，未来随 `zr_kernel` 迁移；12 文件/519 行旧 framework owner、10 处消费路径和
  旧 module declaration 同批删除，不保留 forwarding surface。当前可复现基线为 204 declaration-only、
  369 mixed、22 behavior-only，chained alias mutation RED→GREEN 后 owner guard `5/5` GREEN；详见
  [`01/2026-08-24-m1-state-kernel-owner-hard-cut.md`](01/2026-08-24-m1-state-kernel-owner-hard-cut.md)；
- animation 已完成一项 compiler/evaluator declaration-behavior partition：
  `core/framework/animation/compiler` 是唯一纯 source semantic IR owner，animation plugin 只把成功 artifact
  降低为 executable graph/state/layer 与 Spade 2.15.1 Delaunay topology。production cache 不再独立解析
  graph/state/transition/condition source string；旧 associated source compiler、公开 `BlendSpacePoint*` 与手写
  `O(N^4)` triangulation/overlap path 均为 0。96/128 点共圆 RED 的 0 triangle 已恢复为 94/126，post-profile
  median 为 142.8/194.2 us；128 点随机从旧 11.434 ms probe 收敛到 245.0 us median。source guard
  又锁定 shared validation：canonical float-bit `BTreeSet` 去重和 `robust::orient2d` 基线线性扫描替代
  `Vec::contains` + triple enumeration；4096 点有效输入从 5.849 ms 收敛到 0.437 ms，512 点共线拒绝从
  38.257 ms 收敛到 0.112 ms。source guard `11/11` GREEN，完整证据见
  [`performance/01/2026-08-24-animation-state-machine-pose-capability-current-source-algorithm-performance-review.md`](../../performance/01/2026-08-24-animation-state-machine-pose-capability-current-source-algorithm-performance-review.md)。
  graph IR 的 blend/additive weight parameter 也已从残留 `Option<String>` 硬切为 dense
  `Option<usize>`；plugin lowering 删除第二份参数 `BTreeMap` 与逐节点字符串查找，只保留 slot 容量收窄。
  runtime barycentric/hull projection 也已移除 `f32::EPSILON` 尺度退化与 f32 cross overflow：极小/极大
  triangle 均恢复 0.5/0.25/0.25 权重，常规坐标 probe 的 f64 中位额外成本为 1.822 ns/query；
  per-instance previous-triangle hint 已由 `MachineInstanceKey` keyed、4,096-entry bounded instance cache 收口，稠密
  state-slot hint 贯穿 entry/transition/time/event/pose 的真实 graph sampling；共享 compiled asset 保持不可变，
  replacement reset 与 nested/layer instance 均已接线。hint 不进入 semantic event checkpoint：16-slot
  deep-copy probe 在 128/512/4,096 active instances 的增量中位成本为 22.4/225.7/2,127.7 us，会抵消
  warm-start 收益；source guard 明确封锁该回归。smooth trajectory standalone probe 相对 triangle-zero seed
  改善约 14x--16.5x；source guard `11/11` 于 34.349 秒 GREEN。最新 managed Windows request
  `9fd854d4743f49959bea70760f862d27` 在 Cargo 前因未登记共享产物
  `E:/ZirconBuilds/mvp-resource-management-projects` 终态失败，未创建 job/test binary；插件 tests、
  allocation/power/product frame evidence 仍 pending，当前只记
  `source_implemented / static_and_algorithm_profile_green / managed_cargo_preflight_blocked`，不提升 M1。
  后续 whole-module review 又定位到 compiled dense slot 之前仍反复执行字符串投影：普通 root sample 的
  entry/time/event/pose 四个消费者会各自遍历 parameter name、查 `BTreeMap` 并分配引用 `Vec`。独立 release
  probe 在 8/32/128/512 parameters 下把四次投影收敛为一次后，中位耗时从
  2.0/11.9/76.0/363.2 us 降至 0.6/3.2/19.5/92.1 us（3.33x--3.94x）。生产修正计划已先写入
  performance record：compiled asset 拥有不可变 layout identity，`MachineInstanceKey` bounded instance cache 缓存
  owned dense row，nested/layer/transition/time/event/pose 贯穿同一 typed revision；纯 evaluator 只允许单次
  临时投影，不保留旧 production map-projection 双轨。首版按 ECS player change tick 失效的实现已在产品路径
  二次复核中否决：`active_state` 与参数共处同一 component，正常状态推进也会刷新该 tick，并且 scan request
  仍克隆完整 parameter map，稳定帧会错误重建 dense row。instance
  cache 同时拥有 layout identity、dense values 与 triangle hint，按 inactive entity/replacement/bounded eviction 退休；产品
  pipeline 删除会克隆整张 parameter map 且携带未消费 graph/transitioned 字段的共享 evaluation DTO，改为仅含
  active state/requested transition 的 plugin-private result。post harness（SHA-256
  `a1509abd27755a60e2304b4d5d9d4ce2f2bd6901d71312725947fc82e04568ea`）在稳定 revision 的
  8/32/128/512 parameters 下为 0.028/0.089/0.414/1.815 us median；每次 revision 变化并重建 row 为
  0.590/3.168/20.082/104.743 us；旧 clone + 四投影为 2.742/16.957/144.017/491.103 us。
  source guard `11/11` 于 35.192 秒 GREEN，但该结果只覆盖被否决首版的下游 cache 结构。后续已新增共享
  `AnimationParameterSet`（`Arc<AnimationParameterMap>` + 独立 content revision + COW mutation），graph/state
  ECS projection 各保留 per-entity runtime snapshot，frame request 与 graph cache 改为常量大小 clone；state
  instance cache 仅按 content revision 与 compiled layout identity 重建。测量当前仍含 map equality 的实际
  proxy 时，8/32/128/512 parameters 的稳定路径为 0.196/0.742/3.029/13.123 us median，旧 map clone + rebuild
  为 1.632/7.806/39.771/182.368 us（8.33x--13.90x）；changed proxy + rebuild 为
  1.790/7.845/40.918/178.129 us。当前 runtime proxy/source guards GREEN，但 graph/state player component 与
  scene schema 尚未硬切：所需 `scene_asset.rs` 仍由 active MVP00 owner 占有；baseline epoch 443 的 transfer
  preview request `b4e6860a973d4ad8a6435cbd5aa1fd37` 对当前 blob
  `e3305645731840b122ee2b4f41636a74796ea2e1f27716482f89fd327654aa92` 返回
  `source_owner_executable`。遗漏的 `scene/tests/support.rs` fixture 已按 fingerprint
  `0f7a18551f898541e09aace5cc52d646065f43f0c94c98bfb2b5a77cdc064c7c` 转入本 session，未改写代码。
  Frameworks01 不越权、不留兼容字段；完整 boundary batch 当前 `13/14`（37.219 秒），唯一失败即 component
  owner hard-cut guard，故本项状态为 `parameter_proxy_implemented /
  component_parameter_owner_hard_cut_red / managed_rust_compile_blocked`。managed job
  `75f8a49cf9c34f3099a12150a5c34a4a` 仍先失败于 foreign interface edition-2024 let-chain；带固定
  ZrVM commit 的 validation-copy `fc2edfe9091341289f62e2fef02b32a3` 又被已知 missing `skybox.rs`
  closure bug 截断；该 Rust 动态验证阻塞仍未解除，不把 standalone ratio 当作 frame/power 验收，也不提升 M1；
  同一 whole-module review 继续修正 2D BlendSpace hull-exit 定位：旧路径在 adjacency walk 已证明越界后仍全扫
  `T` 个 triangles、再扫 `H` 个 hull edges，并丢弃 hint，连续越界输入每帧为 `O(T+H)`。对照 Unreal
  `FBlendSpaceData::GetSamples2D` 后，runtime 现区分 inside/proven-hull-exit/abnormal-failure，无 hint 从中间
  triangle 起步，正常越界直接走 prepared unique hull 并保留 boundary triangle；只有异常失败才全量 fallback。
  含真实 adjacency walk 与连续 hint 的 F-drive release model 在 450/1,922/7,938/32,258 triangles 下把中位
  3.9369/24.1178/67.0902/258.4417 us 收敛为 0.6364/1.7345/2.1968/5.2138 us（6.19x--49.57x）。
  source guard GREEN，产品单元测试新增 rolling hinted walk 与 exhaustive 权重等价覆盖；managed Rust 执行仍
  pending，不据此声称 frame/power/engine parity；
  后续 eviction review 又确认旧 nominal-LRU 在 4,096 容量的每次 miss 都以 `min_by_key` 全扫实例表，连续
  4,096 churn 的 realistic-key model 为 180.333 ms median。首版 `BTreeSet` exact-LRU 虽把 churn 降到
  5.859 ms，却让 262,144 stable hits 从 36.201 ms 回退到 169.620 ms，已否决并删除。最终 production hard-cut
  使用 `BTreeMap` + fixed-length second-chance `VecDeque`：hit 只置 reference bit，cold miss 旋转一次命中项后
  淘汰 probationary key，inactive-entity/reset 同步收缩两结构，不保留全表最小值 fallback。final mixed model
  的 4,096 churn 为 6.556 ms（27.51x），hit-only model 为 1.016x；source/unit guards 已锁定 clock invariant。
  随后的 admission rollback review 又删除每帧按全部 active entities 过滤/深拷贝三张状态表的 checkpoint，
  改为 evaluation 前空 journal、每个 `MachineInstanceKey` 首写时合并保存三表旧值、admission 后只恢复 deferred
  journal keys。精确模拟三个 owner lookup 的 4,096-instance model 在 10%/100% writes、无 rollback 时为
  7.70x/1.30x，50% deferred 时为 7.67x/1.27x；512-instance 的 100% writes + 50% deferred 单批中位为
  0.887x，故不声明零回退。source guard 与 admitted/deferred/present/absent unit coverage 已加入，完整批次仍为
  `13/14`，唯一 RED 是 foreign component parameter owner。以上仍是 F-drive 数据结构 probe，不声明
  product frame、功耗或 reference-engine parity；随后 graph evaluation cache 的 whole-module review 又确认
  旧 `VecDeque::iter().find` 对同 graph/skeleton 的 `E` 个不同参数集执行全内容比较，帧复杂度可达
  `O(E^2 * P)`。现由 `AnimationParameterSet` 维护 process-local content fingerprint（signed zero 归一，命中后
  仍完整 equality 防碰撞），per-frame cache 硬切为最多 256 项的
  `BTreeMap<(graph, skeleton, fingerprint), entry>`，满额后停止 admission、帧边界整体清空，不保留旧线性/FIFO
  路径。256 entries、8/32/128 parameters 的 post release model 把 lookup median 从
  17.989/88.563/801.954 us 收敛为 0.0566/0.0553/0.0619 us（317.60x--12,952.70x）；实际变更时全量
  fingerprint refresh 为 0.253/1.025/9.954 us median，仍是 `O(P)`。signed-zero/mutation/collision equality、
  bounded admission、same-content reuse、distinct-content separation 与无 sequential scan 已有 unit/source guard；
  完整 Scene/Animation boundary batch 为 `13/14`（39.653 秒），唯一 RED 仍是 foreign component parameter
  owner；随后 compiled graph DAG review 确认 shared compiler 已生成 dependency-first order，但 plugin runtime
  丢弃该顺序并递归按路径展开；20-layer/61-node diamond 会输出 1,048,576 个 clip contribution。runtime 现保留
  compiled order，以 `Empty / One / Many(BTreeMap)` context accumulator 反向拓扑传播，删除 `collect_clips`，同一
  clip/mask/additive context 只物化一次；Base/Additive、clip source slot、mask source slot 构成新版确定性输出顺序。
  exact production-shaped post probe 在 25/37/49/61 nodes 下的 topology median 为
  0.865/1.104/1.367/1.779 us，旧 recursive median 为 6.721/83.082/2,788.200/57,462.000 us，20-layer
  ratio 为 32,300.17x，输出收敛到 1。diamond、nested mask/additive、source-slot order、4,096-depth contract tests
  与无递归 source guard 已加入；独立 Rust 2021 exact-source typecheck/semantic smoke GREEN；fresh complete
  boundary batch 为 `13/14`（41.615 秒），唯一 RED 仍是 foreign component parameter owner。Rust product 执行仍被
  foreign current source 阻塞，故该项状态为 `source_implemented / static_and_isolated_profile_green /
  managed_product_validation_blocked`，不据此声明产品帧耗、功耗、完整 pose program、引擎 parity 或 M1 验收；
  随后的最终姿态 publication review 又确认旧 `Arc<BTreeMap<EntityId, AnimationPoseOutput>>` 在 partial
  admission 时深拷贝全部 bone/name payload，并让 physics 清空重建、render extract 与 history 再各复制一次。
  production 已硬切为 `AnimationPoseSnapshot` 外层快照 + `Arc<AnimationPoseOutput>` 密封行；partial update 只比较
  supplied/removal identities，未变化批次零分配返回，变化批次复用未变 row handle 并把 exact delta 交给
  `SkeletalPoseTargets`。旧 whole-map recording 与 owned render row 未保留兼容入口。64-bone exact post probe 在
  4,096 entities 的 0%/1%/10% update 下把 publication median 从 61,729.0/83,535.9/82,914.0 us 收敛为
  236.3/829.9/6,248.2 us（261.23x/100.66x/13.27x）；1% allocation bytes 从 50,298,128 降到
  338,542。完整替换并非普遍胜出：512 entities median 为 0.92x，4,096 entities 虽为 1.08x 但增加
  4,473 allocations/468,384 bytes，后续须由 instance-local dense pose page/arena 收口，禁止恢复旧深拥有 API。
  二次实现复核又发现 `LevelSystem::record_animation_pose_snapshot` 在 pointer fast path 前对整张 pose map 做
  semantic equality；production 已删除该兼容比较，以 sealed outer `Arc` identity 作为 publication identity。
  4,096 entities × 64 bones 的独立 Level admission probe 把 same-outer median 从 17,114.562 us 收敛到
  6.294 ns（2,719,186.84x）；该子路径数据不与 pipeline/physics 独立样本相加伪造 whole-frame ratio。
  temporal history 的派生 pose equality 也已改为 entity/skeleton/handle identity；4,096 visible rows 的独立
  post probe 从 13,492.200 us 收敛到 5.810 us（2,322.24x），保留检测可见 pose 集变化所需的 `O(V)`。
  sealed-publication guard GREEN，fresh complete boundary batch 为 `14/15`（40.692 秒），唯一 RED 仍是 foreign
  component parameter owner。managed request `8484973835bc46a2a5066129b6eac35b` 在 Cargo 前因 foreign unmanaged
  `E:/ZirconBuilds/mvp-resource-management-projects` 终态失败；本项状态为 `source_implemented /
  static_and_isolated_post_profile_green / managed_product_validation_blocked`，不声明 frame/power/parity 或 M1 验收；
- camera-controller 已完成一项独立 declaration/behavior hard partition：input/settings/state/output DTO 保留在
  `core/framework/camera_controller`，Free/Orbit/Pan 执行唯一 owner 硬切到 `input/camera_controller`；旧 controller
  文件、旧 Framework 导出、consumer import 与 compatibility forwarding 均为 0。current guard `15/15` GREEN
  （35.755 秒）；cancelled r10 留下的 7 个新增 owner blob 已按 current hash 通过 transfer fingerprint
  `47c8d704901c0545ba2b3939d0668c49b5c02332c4b7fce4315e9401405afbcd` 原子接续到 r12，完整 21-path dirty
  candidate 已租赁/attribution。算法复核发现 Orbit zoom 目前只消费 `delta.signum()`、不保留输入幅值；本次 owner
  迁移不改写该产品语义，也不声称性能收益。后续修正前必须由 Input/Editor 产品路径记录真实 viewport wheel/gesture
  delta 分布、交互回放和 controller update/frame-time 基线，再在 input implementation owner 内完成 TDD 与 profile
  对比；当前状态为 `source_implemented / managed_validation_pending`，不提升 M1；
- deterministic random 已完成第二项 physical partition：algorithm/key/state/service-state/receipt DTO
  保留在 `core/framework/random`，BLAKE3 stream derivation、master-seed authority 与 PCG32 执行硬切到
  `core/runtime/random`，未来分别随 `zr_contracts`/`zr_kernel` 迁移。旧 framework implementation 文件、
  三处 implementation consumer 和兼容导出均为 0；implicit module-leaf alias mutation RED→GREEN 后 owner
  guard `13/13` GREEN（新增 glob-import、renamed raw-selector、四类 implicit stream-copy mutation，及
  master-seed service/accessor/私有 backing field 单一 authority 守卫）。后续正确性复核已用受检
  `RandomSequenceId` 显式锁定 PCG32 的 63-bit stream space，
  `reseed` 在 generation 终点改为 typed reject 且失败不改变 seed authority；完整 BLAKE3->PCG 初始状态向量与
  12/12 exact-source Rust tests 已锁定；独立复核指出的逃逸均已以 mutation 与 rejection
  draw accounting 收口；mutable `RandomStream` 与 master-seed `RandomService` 均已删除 `Clone/Copy`，
  `CoreRuntimeInner` backing field 已私有化，`CoreHandle/CoreRuntime` 只返回 `&RandomService`，禁止
  assignment/argument 隐式 state/authority duplication，也禁止 crate 内经 `Arc::get_mut` 直达 reseed。
  owner guard 现扫描全部命中 `RandomService` 的产品 Rust 候选，拒绝任意其他模块新增 authority-returning
  impl 或 `Clone/Copy`，拒绝 `type`/`use as` authority alias 源，并要求 Runtime、Handle、
  `CoreRuntimeInner` 三个 owner 各且仅各有一个共享借用 accessor；split-file alias mutation 与最终独立
  复核均已 GREEN（0 Critical / 0 Important / 0 Minor）。
  `RandomState`/`RandomServiceState` snapshot/restore 只是显式状态重建，不是最终 fork policy；stable-key
  admission/registry 仍归 Runtime22。camera-controller owner 15 + random contract/kernel 13 +
  scene-animation boundary 11 的明确组合边界为 `39/39`（其中本轮 animation boundary `11/11` GREEN，
  历史组合耗时 104.517 秒未重跑）；当前
  可复现基线为 204 declaration-only、368 mixed、22 behavior-only；
  2026-08-25 又以 transfer fingerprint
  `ce98561b713ab6784d030145e8e9ad7d01731a4898eca55ad8417bbecbd0161c` 将 cancelled r10 留下的 10 个
  未漂移 contract/kernel blob 原子转入 r12，preview/apply requests 分别为
  `1102cbf209764560b1c72abde14cf91a` / `9b9931b9b0664362837f44c73b8cd044`，没有改写源码。
  但 `core/runtime/handle/mod.rs` 仍属于 archived Runtime core lifecycle owner 且 current hash 已漂移，
  `runtime.rs`/`core_runtime_state.rs` 又是 Random/Time/State/lifecycle mixed gateway；完成 scope rotation 或
  exact transfer 与 managed Runtime 验证前，本项继续 `source_implemented / ownership_pending`；canonical
  Runtime01 node `2493698` 见
  [`open`](01/failure-2026-08-25-random-runtime-handle-gateway-ownership.md)，不提升 M1；
- state kernel 后续结构/性能复核已达到
  [`source_implemented / static_and_profile_validation_green / managed_cargo_foreign_blocked`](01/2026-08-25-m1-state-transition-retention-performance-review.md)：
  current `Vec` 永久历史的 retained payload `O(N)` 与整段 query clone `O(N)` 已硬切为 singular latest event；
  旧 plural API/产品 consumer/compatibility path 为 0，owner guard TDD RED 后 `6/6` GREEN（32.119 秒）。D 盘
  production-source harness 在 1,000--1,000,000 次 transition 均只保留 40 B；1,000,000 次时 query median 从
  pre-cut 45.498 ms 降为 34 ns，完整 registry transition 为 168.466 ms（约 168.5 ns/次）。Unreal/Bevy 的
  bounded update-lifetime working-set 原则得到 current-source 对照支持；managed Cargo 与能耗仍 pending，
  不声明产品帧耗时或功耗改善；
- state managed validation 已把两个前置层逐项分离：`Cargo.lock` 经 D 盘 shadow workspace offline resolver
  校正后与解析器输出逐字一致，SHA-256 为
  `f8df4d979bd86eb91e58df1031a828a65ca2de43de64a5362ad166ccaa8023de`；随后 job
  `673d51016b2d4679842de468abca4ec0` 越过 locked resolution，在生成 Runtime test target 前停止于
  `zircon_runtime_interface/build.rs` 的 E0106。最低共享层已交由
  `interface08-build-spec-lifetime-fix-r1-1b2684b4-20260825`，canonical handoff 为
  [`Interface08 slot-list lifetime failure`](../../optimize/zircon_runtime_interface/08/failure-2026-08-25-interface-spec-slot-list-lifetime.md)。
  当前签名已把返回 slice 只绑定到 InterfaceSpec value，scoped rustfmt/diff/handoff validation GREEN；真实
  build script 的 D 盘 standalone `rustc --test` 为 5/5 GREEN（0.10 秒，测试 executable SHA-256
  `e7a2fa792ed95d7b0d319b0186a3ca75c253c93529275113cfbd49b0ddcee0a4`）。首次
  managed Interface acquire 因 foreign job `e6317bbbd76747258772c039543379f4` 持有兼容池而未创建新 job；
  后续 request `d42bd1c1cb154b9a8b23d7a85c154df4` 精确对账为
  `admission_checkpoint_stale` terminal failed，fresh retry 又被 running foreign Runtime job
  `12c25c64e2a14bb0848aa24788755168` 的 compatible pool 拒绝，仍未创建 Interface08 Cargo job。
  因此这里仍是 `managed_cargo_pending`，不把 Interface 修复或 state 行为记为已验收；
- time product policy 已完成第三项 contract/kernel behavior partition：version/profile/budget/validation DTO
保留在 `core/framework/time`，client/headless/editor/test preset selection 与 canonical BLAKE3 digest 唯一实现
  硬切到 `core/runtime/time/product_policy.rs`。旧 inherent preset/digest API、framework BLAKE3 命中和生产
  consumer 均为 0。2026-08-25 又将 `Time<T>` 的 `context_mut`/advance、virtual pause/resume、fixed debt
  accumulation 和旧 batch `drain_steps` 全部收为 production crate-internal authority；产品源码 batch drain
  caller 为 0，旧 drain 只在 crate test 配置存在。owner guard `6/6` GREEN（50.107 秒），精确 rustfmt/diff-check
  GREEN。首张 Windows/D 盘 managed `core-min` job 在本切片编译前被 foreign Runtime Interface BuildSet
  `String`/ABI-symbol bytes 表示漂移截断；其 owner 随后已继续改写该 blob。current-source 复验请求
  `4d72209f4f49492ca8e5d5cf1ed7caac` 在 `cargo.acquire` accepted 后进入 coordinator post-response timeout，
  没有可声称的 Cargo 终态，因此本项为 `source_implemented / managed_validation_pending`，不提升 M1；该切片
  不改变 Runtime22 的 World-local fixed-step transaction 语义，也不在 managed profile 前声明性能或功耗结果；
- 2026-08-25 fresh contract-partition audit 以当前 production source 重新分类 `core/framework`：667 files、
  55,285 non-empty lines、3,842 function bodies，其中 226 declaration-only / 418 mixed / 23 behavior-only。
  render 单域已有 244 files / 2,199 bodies / 176 mixed，animation 为 47 / 276 / 31，time 为 15 / 85 / 9；
  因此不得把整个目录按名称搬入 `zr_contracts`。可复现 JSON 位于
  `D:\zircon-frameworks01-r12-current-contract-partition.json`，SHA-256 为
  `b17c3f28b52c7444c1bbef6dac3881563378f430c08983afe1b858d9ca87e937`，产物未写入 C 盘。
  Time 深读确认 Runtime22 当前方向应保留：`RuntimeTimeAuthority` 只拥有 outer monotonic real input 与默认
  policy，`WorldTimeController` 在 scene/Level owner 内拥有 virtual/fixed debt 和 proposal/begin/commit/abort，
  contract 层只保留 immutable stamp/snapshot/policy/receipt 或维护自身不变量的只读值对象。Bevy 的公开
  `Time<T>` mutator 是其 ECS resource owner 形态，不是把 mutation authority 下沉到纯 contracts 的依据；Unreal
  `FApp` 的全局 time setters 已标注 delta-time refactor deprecation TODO，Godot 也把 process 与 physics process
  分开。Zircon 后续物理硬切禁止为跨 crate 编译把 `advance`/pause/rate/debt/commit setter 扩为 public，禁止
  hidden assembly re-export 或重复 core/world clock。
  当前 `core/framework/time/domain/*`、`fixed_step_plan.rs`、`virtual_clock.rs`、`core/runtime/time.rs` 和 5 个
  `scene/world_time/*` blob 仍为 `attribution_missing`；原 Runtime22 Session 已 stale，且其
  `2026-08-24-fixed-step-transaction-architecture-and-performance-plan.md` 明确 managed validation/profiling pending。
  ownership matrix requests `d335f1e81c87472e847802b1e704355d` / `bd54c2f4b75b41e2bad8085c5c82fc56`
  固化该状态；audited scope 扩展 request `f08d0769014947af8928bb4d0b1b81aa` 终态拒绝且未改变 scope，
  Frameworks01 不改写或冒领这些 blob。entry gate 是 Runtime22 原子 current-hash 归属、focused/upward managed
  Cargo，以及其既定 0/1/8/capped steps、1/100/1000 systems 的锁等待/分配/CPU profile；取得前不开始该项代码
  优化，不声明性能、功耗或整机帧耗时。
- `zr_contracts -> zr_rhi` 是批准的独立低层 contract edge，只用于
  `RenderNativeSurfaceTarget`/`UiSurface{Descriptor,Presenter}`；`zr_rhi` 不得反向依赖 `zr_contracts`，
  App/Editor/plugins 仍只消费 Runtime curated facade。该边不授权把 `zr_rhi_wgpu`、`wgpu`、`naga`、
  `glyphon` 或 graphics backend 实现带入 contracts；
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
- M1 DAG 前置记录：[`01/2026-07-18-m1-runtime-diagnostics-facade-collector-hardcut.md`](01/2026-07-18-m1-runtime-diagnostics-facade-collector-hardcut.md)（manager-resolving diagnostics 已移出 core；2026-08-25 current guard 又锁定 Framework `CoreError` consumer 为 0、folder-backed collector 仅由 Runtime 根私有挂载，focused 3/3 GREEN（92.448 秒）；Cargo 与 Shader06 foreign doc pending）
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
- 2026-08-24 M1 `zr_math` physical hard cut 已达到
  [`source_implemented / focused_validation_green`](01/2026-08-24-m1-zr-math-physical-hard-cut.md)：
  `zircon_runtime/crates/zr_math` 是唯一实现 owner，Runtime/Interface 仅保留批准的 curated projection，
  旧 Interface 实现已删除。当前 14 个 crate Rust 文件共 1,306 行；locked production build、完整 lib
  tests、math boundary 4/4 与退化 look-at RED→GREEN 均通过。`Transform::looking_at` 已按 Unreal/Bevy/Fyrox
  参考收敛为确定、有限、正交的 fallback，严格 `try_looking_at` 仍拒绝非法输入。Runtime 产品 build
  随后停止在 foreign `zr_rhi_wgpu`，所以 M1 未 accepted、未提交、未发送企微，也不声明性能或功耗结果。
- 2026-08-25 shader invocation hard-cut 已完成
  [`current-source preflight`](01/2026-08-25-m1-shader-invocation-binding-hard-cut-preflight.md) 与 RED fixture，
  但仍未进入生产迁移：最新 12-file exact consumer transfer-preview request
  `cefad0f7135c4078a8ba2216b55bdac9` / fingerprint
  `2e030adee42a846134e0aaf7885da24bc5cf04867ffd9a13091c8550015629a1` 为 10 eligible / 2 blocked；
  `core/framework/render/mod.rs` 仍由 active `mvp00-current-source-convergence-r2-01a00797-20260818` 持有，
  `asset/assets/mod.rs` 仍由 resolving `text01-font-artifact-service-20260825` 持有，均返回
  `source_owner_executable`。因此 4 个 shader owner guard 保持预期 RED；Frameworks01 不做部分迁移、
  不建立 forwarding/compatibility owner，也不把该 RED 计为 current-source 回退。排除这 4 个预期 RED 后，
  当前 Frameworks01 静态集合 59/59 GREEN（377.603 秒）；该结果不替代 Shader 生产迁移或 managed Cargo
  验收，因此不提升 M1 状态。
- 2026-08-25 baseline epoch 436 的 Shader contract-owner 复核确认，上述 2 个 mixed consumer 并不是
  唯一准入条件：`core/framework/render/shader` 下 12 个 contract blob 仍为 `attribution_missing`，
  `compute_dispatch.rs` current hash `5779f98dda52eac00bd9dbe9d9d1656ae5fb2cc0606f1c1085276d92856b62ce`
  仍指向 archived Shader04 attribution 且 hash/baseline stale；render root current hash `b8b5908e...`、
  asset root current hash `76003c49...` 继续分别由 MVP00/Text01 executable owner 持有。最新 hard-cut guard
  在 18.391 秒内取得 2 passed / 4 expected failures。完整 contract/behavior、asset schema、graphics compiler、
  mixed consumers、old-export 删除面与 product fixture 未形成同一 current-hash owner union 前，禁止开始
  Shader ABI 半迁移；历史 12-file preview 不得复用，也不得用 re-export、type alias 或 wrapper 保持旧架构。
- 2026-08-25 M1 state-transition retention 的最低层 InterfaceSpec E0106 已由 Interface08 完成并
  [`fixed 回传`](01/fixed-2026-08-25-interface-spec-slot-list-lifetime.md)：真实 `build.rs` 定向测试 5/5 GREEN
  （0.10 秒），Windows managed job `7f8adbb03fdf4616a9d6d887045c74a2` 的
  `cargo build -p zircon_runtime_interface --locked` 在 D 盘 pool 用时 3 分 12 秒通过，原 Frameworks01 Runtime
  gate 已越过该编译前沿。随后 Interface lib-test 在执行 filter 前被 9 条
  foreign UI text-shape / Project GUID 测试源码错误阻断；managed job
  `29a2e88837ab4890863b3b59ce7fd251` 的 Frameworks01 state build/test 则同时停止在
  `project/manifest_summary/summary.rs` 的 transient `super::ProjectGuid` E0432。作业结束后 current 文件已改回
  `crate::project::ProjectGuid`，SHA-256 为
  `4302116d5634d08b1ec62156a281d1015b415e32562d9654fc8d328f60f15c4c`，但 Project 全域 ownership matrix 仍为
  `attribution_missing`、无 owner、无 live lease。两张 job 均已 released 且 process tree 为空。Frameworks01
  不吸收该 Project06 blob。其共享树在本次 job 后继续从 71 files / 100,823 bytes 漂移到 74 files /
  112,187 bytes，再到 75 files / 123,232 bytes；最新 deterministic tree SHA-256 为
  `e4c2c3da3b57c68d53f3f7ca04844c3f3b4b5ccb2641e6a7a46bec142799d12d`，最后写入为
  `2026-08-25T13:49:57.2819771Z`。当前新增 `CanonicalDescriptorIdentity` 仍把裸 `PathBuf` 序列化为跨进程
  identity 并把 physical filesystem resolution 前置条件交给 caller，`ProjectIdentity` 未拒绝未知字段，
  `ProjectManifestSummary` 仍可通过 public fields 绕过 domain construction，compatibility product state 也未
  收敛到计划要求的 Open/Copy/Migrate/Safe/Reject typed decisions；因此单个 import 修复不能把该 blob 提升为
  可接受基线。Frameworks01 仅在 Project06 提供稳定、已归属且符合 wire/domain 边界的原子快照后重跑 state
  gate；当前 state slice 仍是 `managed_cargo_foreign_blocked`，M1 不提升，也不声明产品编译、功耗或整机帧耗时。
- 2026-08-24 `zr_resource` current-source admission 已由
  [`preflight_complete`](01/2026-08-24-m1-zr-resource-current-source-preflight.md) 刷新：57 个 tracked Rust
  文件、11,111 行、412,214 bytes、tree SHA-256
  `c824c74936e8a533954de2017aed726269ac3df4edd01a18c6323cf0684b23c2`，因此 epoch-321 manifest 不得复用。
  Runtime51/Runtime25 仍持有 mutation/I/O 输入，Runtime24 仍持有 Interface stable-UUID ABI 输入；当前 r9
  scope 不含完整 Resource/consumer 原子联集，物理
  hard cut 尚未启动。UI12 的 IBL E0432 已确认为旧/混合指纹：current facade 与 implementation 各保留唯一
  public `core::resource::io::atomic_write`，runtime cache 与无配对 source 的 asset-derived 单文件 store 保持该
  curated publication 入口；需要同时发布 source/derived 两文件的 staging 已消费 durable transaction，不迁移或
  改写 Shader06 consumer。current 五个关键 blob SHA-256 为 façade
  `8977312a56e9a4228c0534092c8e91882a56b449553596590a686d82b0d3bed8`、implementation
  `0b192fe6bc73802bc7f83f3fb968d3d11313c0f25aa5b15c545811041bdc1746`、asset-derived
  `a6b804ce5da2b69b4376d20c51633f5239ce859df4e5cde359db57c1300955cf`、runtime cache
  `09a8bae2c523c6e5c9cce8591a49d48a97b796228078517c7ea4762f08420b18`、staging
  `257c0499e5a8a81e42dbb6204463271719aacc56aa0ea2851533b637b48a3a77`。Windows managed editor production
  build job `b8c230e7d5da41bd855c1b2d3fa82278` 于 19:00:23--19:00:52 执行约 29.6 秒并 release，进入
  `zircon_runtime`/IBL 前被 unowned 新增 `zr_rhi/src/surface.rs` 的 2 条 E0499 截断；因此该 job 只否定旧行号
  指纹，不作为三条 import 的 compile GREEN，也不把 foreign RHI 阻断归给 Shader06。
  同一 current-source 复核发现后续无 owner 漂移新增的 `atomic_write_if_unchanged` 不是文件内容 CAS：compare
  与 replace 之间可插入另一个 writer，而 Editor06 已把 `Replaced` 当成无外部冲突并更新 disk baseline。
  `atomic_write_new` 的 Unix hard-link/Windows no-replace move 方向可保留，但仍缺 create-race test；conditional
  save 必须由 DocumentToolkit normalized-path authority 与 Resource durability owner 联合硬切，不能用多读一次或
  公开 transaction internals 伪装原子性。canonical node `2495781` 见
  [`open / source_implemented / static_and_review_green / managed_rust_blocked`](01/failure-2026-08-25-resource-conditional-atomic-write-authority.md)；
  r12 已通过 transfer request `3098bc07218c4d4cafd4495fac60ce75` 与
  `b82da4b538994cea99e200c8670f8390` 收口 Resource、Editor caller 和
  `DocumentToolkitRegistry` 精确 owner，删除 `atomic_write_if_unchanged`/`AtomicWriteCompare` 及全部
  Runtime+Editor Rust 引用，不保留 compatibility；新 normalized physical-path authority 持有 lease 直到 durable
  publication、disk baseline/digest 与 persisted revision 同步提交。首轮独立复核为 `C1/I2/M1`：animation save 与
  UI asset undo/redo external effects 绕过 authority、可见 replace 后 durability-barrier failure 会留下错误 baseline、
  save report 未公开 best-effort external-writer guarantee，且旧 wait test 未证明 Condvar admission。r12 通过 transfer
  `7edcc10fa443472696e446c4b0ff620d` 扩展 exact owner 后已修正：所有 cooperating document writer 共用同一 normalized
  path lease，publication outcome 区分 `NotPublished`/`PublishedNotDurable`/`SourceChanged`/durable best effort；
  post-publication failure 更新 disk baseline 但不确认 persisted save token，`DocumentSaveReport` 显式公开
  cooperating-writer serialization 与 external-conflict best-effort 保证。durable receipt 由 authority 私有铸造并由
  `SaveCtx` 消费，caller 不能自证；pre-publication observation 以 missing/matching/different/unknown 四态保守归因，
  same-content 或 unknown post-image 不会被误报为本次 durable publication。15 个 Editor authority tests、1 个
  save-report guarantee test 与 Resource 双 staging create-only contention test 已写入；当前 18 个 attributed Rust blob
  的 exact rustfmt/diff-check、production-writer 与 retired-API 扫描均 GREEN。新增 Frameworks01 current-source guard
  的 TDD RED 精确暴露 2/7 缺口，最终 primary 为 `7/7` GREEN / 6.747 秒；独立 reviewer 复跑为 `7/7` GREEN /
  6.263 秒，post-record 复核为 `7/7` GREEN / 8.826 秒；session
  `frameworks01-interface08-lifetime-review-r1-20260825` 的最终结论为 `C0/I0/M0`，authority/guard
  SHA-256 分别为 `78227abf4b98cc36cf419096d7729efee49bbf6448210b6f3054186a011ebc85` 与
  `2d1055f530b11074280991fb37e16cef618a84288b0f15a52b800c99c4f197d8`。Windows managed job
  `dc439b2ec8a14db6a0a1b4d2ea34fbfe` 仅使用 D 盘 ephemeral
  target，于 16:45:57--17:11:00 UTC 运行后以 101 退出；共享 Runtime lib-test 图在执行测试前产生
  416 errors / 1,517 warnings，覆盖 139 个 error-bearing Rust 文件，本轮 8 个 owned implementation 文件直接诊断为
  0、执行测试数为 0。job 已 release 且进程树为空。随后 Editor managed job
  `2d839ac9d78b4b56a829bb015784a36f` 仅使用 D 盘 target，于 21:24:24--21:32:51 UTC 运行后以 101
  退出；它在共享 `zircon_runtime` dependency 编译阶段被 80 errors / 118 warnings 阻断，Editor test binary
  未生成、执行测试数为 0。完整 stderr 有 81 个 error heading，对本轮 9 个 owned Rust path 的直接命中为 0；
  journal/recovery visibility 与 Platform host/window-registry 等均为 foreign current-source failure。release request
  `3269486b865e4c51ba8e4aa27c244c04` 确认进程树为空、job 为 `released` 且 D 盘 target cleanup 已排队；
  随后的 production build job `af4dbe2fcadf47a0a1e9c660c1966c33` 复用 coordinator D 盘 compatibility pool，
  06:36:02--06:40:42 Asia/Shanghai 在 shared `zircon_runtime` dependency 以 83 errors / 119 warnings 截断，未生成
  `zircon_editor` 或 authority test binary，执行测试数为 0；error cluster 为 foreign animation compiler、render
  visibility、Resource transaction journal/engine、ECS tick-policy、Platform host/window registry 与 material move
  drift，未观察到本轮 3 个 Resource atomic-file facade 文件诊断。06:40:44 job 已 release 且进程树为空。
  后续 isolated cargo-copy `b5a8fc35080148928c87fc59aaecf992` 在未提供 sibling descriptor 时按契约于
  closure planning 拒绝；固定 immutable `zr_vm` commit `61b79becf64efdae8406385ba2c880620831b4b3`、mount
  `zr_vm` 与 binding/sys roots 后，copy `79657cb067264d0dad6db28ab28dd9d6` 仍被 loaded daemon 扫描
  unrelated dirty Runtime test，因其引用 worktree 已删、HEAD 仍一致存在的 `render/environment/skybox.rs` 而在
  Cargo 前终止。这归入既有
  [`Coordinator01 wrapped package closure failure`](../../zircon_tooling/session_coordinator/01/failure-2026-08-25-wrapped-cargo-package-closure-scope.md)。
  full explicit copy `405fc4d9c26347b4bd5c936cc01b5650` 已以 21 overlay、input manifest
  `350790704da044d45e689dc9d10331740d61ceb5a2c4dcc655fd84da9b303876` 和 external-source hash
  `984a7062d9607791aa97e338032e547e00d658a81795a56973d7109a32c2c404` materialize，但显式 copy 不含
  workspace `Cargo.toml`，run `c40f6af8ce574115b3cdad3500d22c5c` 只在 Cargo root discovery 以 101 退出，
  编译/测试均为 0；不得用 foreign dirty workspace 路径扩容绕过。managed race matrix、profile、foreign
  current-source compile closure 与 Coordinator01 forward fix/daemon load 完成前仍不提升。
  normalized-identity 后续独立复核先后返回 `C0/I2/M2`、`C0/I3/M1` 与 `C0/I1/M2`，暴露 lossy
  fallback key、错误折叠、raw recovery entry 绕过和 multi-path waiter 饥饿风险。当前已硬切为唯一
  `ResolvedProjectPathIdentity` 有序契约；路径解析 fail-closed；artifact/import recovery 同时校验 canonical target
  与 resolved raw parent + original leaf 的同一布局规则；meta authority 使用 ticketed conflict-aware waiter queue，
  仍允许 disjoint waiter 并发。最终定向复核为 `C0/I0/M0` / `Ready`；architecture guard 为 `10/10` GREEN /
  13.866 秒，exact rustfmt/diff 与 legacy helper/facade scan GREEN，production owners 均低于 800 物理行。
  managed Rust 行为测试仍被 artifact audit `05a2ae944da84de8a8e3ab31f22b49b1` 的 9 份 foreign unmanaged
  output 在 Cargo 前阻断，profile/power 与产品验收均 pending，因此状态为 `source_implemented /
  static_and_review_green / managed_product_validation_blocked`，不提升 M1。
  同日 `zr_contracts` 创建前置继续 source hard cut：state、random 与 time product-policy contract/kernel
  分区后，共享 current `core/framework` 为 641 个 Rust 文件、71,953 行、2,664,501 bytes；按
  `path<TAB>bytes<TAB>lines<TAB>file-sha256<LF>` 复算的 manifest SHA-256 为
  `7d34b55b7eac23296cf81f7258ba63ae4d2c392e67820c34cd2b03ac7887ae3f`。结构化审计发现并移除仅有的
  2 条合同到 kernel-error 反向边；`ConfigManagerError`/`LevelManagerError` 成为合同自有错误，runtime
  implementation 显式映射，不保留 `CoreError` compatibility。边界守卫 TDD RED 精确 2 violations，当前
  `3/3` GREEN。可复现 partition audit 为 `5/5` GREEN，state/random owner guards 为 `5/5` 与 `6/6`
  GREEN；time policy 分区后的 production contracts 候选为 594 文件/51,925 行、3,553 个函数体/
  2,514 个 public 函数体，分类 204/368/22；对应 D 盘 audit SHA-256 为
  `b56335397cf752dc9e783686262d2bdd9647571165004ed824562c4700b0fb48`。Runtime55 仍拥有唯一 stale typed
  assertion，已路由到
  [`Runtime55 consumer handoff`](../../optimize/zircon_runtime/55/failure-2026-08-24-config-manager-domain-error-consumer.md)；
  foreign RHI 与该 consumer 返回前不声明 Rust product GREEN、不开始 `zr_contracts` 物理 move。random 的
  generation exhaustion 与 63-bit PCG sequence identity 已按
  [`source_implemented / focused_validation_green`](01/2026-08-24-m1-random-contract-kernel-partition.md)
  收口；BLAKE3 stream-creation 独立 release 预检已于 2026-08-27 完成：unique key derivation median
  426.462 ns，相对 PCG seed init 2.559 ns 与 contiguous draw 1.657 ns 分别约 166.7x/257.4x，same-key
  与 unique-key 同量级，未观察到可复用路径。Frameworks01 因此否决局部 hash 替换或隐藏 cache；权威
  stream registry/reuse、replay 与 CPU/GPU parity 仍归 Runtime22 的结构性产品切片。本轮不把独立 harness
  数据外推为产品帧耗、功耗或算法最优。2026-08-24 23:33:04 至 2026-08-25
  00:08:34 的 Windows managed F 盘 ephemeral job `bffbcc35cb1e48fe98e46697df13bd81` 已编译通过
  `zr_math`/`zircon_runtime_interface`/`zr_rhi`，build 与 random-filter lib test 均在进入 Runtime 前被 foreign
  `zr_rhi_wgpu` 同一组 14 errors 截断；最低为 `production/device/context.rs:6` E0432 unresolved
  `wgpu_device_features`。job exit 1 后由协调器删除 target，Frameworks01 未改写或接管 RHI blob，故仍不声明
  Rust product GREEN。
- 2026-08-24 open-failure re-audit：typed CoreError single-source guards GREEN 2/2；Scene/Animation
  hard-cut guard GREEN 9/9，并锁定 neutral manager 不再暴露进程级 IK inbox。Plugins04 明确把 graph-local
  IK 产品集成保持 reopened，Frameworks01 不恢复旧 queue。time product-policy split 后的 fresh Runtime 只读审计为
  2,791 production references / 71 edges、`rhi -> rhi_wgpu = 0`，审计 SHA-256 为
  `e069a73204d68e603152444a1cbf509315fee3b5e994a2cf426ad651220ffc1e`；`zr_rhi` tree 23 行且 backend 命中 0；
  四个 diagnostics
  compile blockers 原 hash 未变，canonical open handoff 已落到
  [`Runtime90 fixing child`](../../optimize/zircon_runtime/90/failure-2026-08-24-rhi-wgpu-diagnostics-current-source-compile-blocker.md)。
  三条 failure 均仍为 `open`，等待 exact Rust product GREEN、current immutable review 与 canonical return。

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
  2026-08-27 已在实现前重新审计全部 production consumer，并确认真正的当前读侧热点不是无生产调用者的
  `page(offset, limit)`，而是 Editor asset workspace 对每个 asset 调用 `row_by_locator`；对照 Unreal
  `FAssetRegistryState` 的同 owner secondary-index 规则后，采用 generation-owned 64-shard
  `locator -> ResourceId` 索引，拒绝 Editor cache 与每次 mutation 全量复制的单 HashMap。Rust 1.94.1
  D 盘 frozen release harness 的配对 rerun 显示，1k/10k/100k 单次 locator 查询中位数由
  2,947.740/31,342.000/118,931.573 ns 降为 170.273/540.427/911.293 ns，即
  17.31x/57.99x/130.51x；100k 普通 revision update 中位数改善 5.28%，但 add/remove、rename 与
  locator-skewed rename 分别付出 21.70%、34.77%、10.71% 的 publication 成本。100k locator index
  capacity/entry 为 1.1469，保守增量模型约 6.87 MiB；locator 字符串通过 `Arc<str>` 共享。实现只 clone
  发生 membership 变化的 locator shards，并先完成全部 remove 再 insert，exact-source smoke 覆盖 10,000
  records、swap、replacement、rename/remove、非派生 ID 与 shard/outer-Arc sharing 后 GREEN；独立复审
  C0/I0/M0。详细输入、输出 hash 与决策见
  `01/2026-08-27-m1-resource-management-current-source-read-profile.md`。这些仍是 isolated algorithm
  数据，不是 Editor frame time 或功耗；managed Cargo、真实产品 trace、system power 与跨引擎经验值验证
  仍 pending，offset pagination 因无 production consumer 保持不变。指定 Tooling job
  `57769fdbe2394a6f9919dc66b5f81946` 当前已自然 `released` 且无 live PID；随后恢复的受管 Windows
  `zircon_runtime core-min` check 在 Cargo 启动前被 foreign unmanaged artifact
  `D:\ZirconBuilds\tooling15-wave163-runtime-20260827-135832` 拒绝，Frameworks01 不越权删除或接管该目录。
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
- 2026-08-27 M1 `ResourceManager::commit` ordered-staging 结构优化已达到
  [`source_implemented / exact_rustc_and_profiles_green / independent_review_green /
  managed_cargo_foreign_blocked`](01/2026-08-27-m1-resource-commit-ordering-preflight.md)：完整 owner/consumer 与
  Unreal Asset Registry/Bevy event queue 复核确认旧实现用 `HashMap<ResourceId, StagedResource { order }>` 暂存，
  apply 在 authority 写锁内再执行 `into_values().collect::<Vec<_>>() + sort_by_key` 重建 first-touch 顺序。新实现硬切为
  `HashMap<ResourceId, usize> + Vec<StagedResource>` 单一顺序 authority，删除 `order`/`next_order`/排序，保持
  prepared gate、rollback、generation、residency 与解锁后事件发布边界。首轮独立复核发现按 operation count
  eager reserve 会使 repeated batch staged storage 退化为 `O(N)`；修复后 staged/ID/locator 初始容量统一限制为
  `min(N, 64)`，实际空间为 `O(K + L)`。D 盘 exact isolation 为 focused `4/4`、完整
  `112 passed / 0 failed / 3 ignored`；最终复核撤销了含 unstable sort/pre-sized receipt map 的 hybrid 基线，按
  Git blob `bf7b8d69e` 重建后，真实旧容器结构模型在 32,768 distinct、1,000 distinct、
  100,000/4,096 random 分别降低 88.51%、92.00%、79.68%，100,000/64 repeated 回归 7.69% 且仍在门内。真实 pre/post
  commit-window allocation requested bytes 在 32,768 distinct、100,000/64、100,000/4,096 分别降低
  27.13%、2.11%、22.94%，最大单次请求均约减半。同进程 12 对 AB/BA public timing 对高方差 10K/100K
  case 明确记为 inconclusive；独立复核 `C0 / I0 / M0, Ready`，只确认没有 paired median 超过 15% 回归门，不声明产品级 speedup、功耗或整条
  commit 已最优。managed job `433c40bf07154cbf9bc1d712f94dcf09` 仍停在 55 条 foreign current-source
  errors，故不提升 M1、不提交、不发送企微。
- 2026-08-27 M1 `ResourceManager::ready_records_for_kind` 已达到
  [`implementation_complete / direct_projection_and_review_fixed_profile_green /
  independent_review_green / managed_cargo_foreign_blocked`](01/2026-08-27-m1-resource-registry-export-snapshot-preflight.md)：
  先复核 Resource authority、三份真实 consumer 与 Unreal Asset Registry/Bevy/Fyrox 查询策略，再用完整 API、锁
  持有时间、writer collision 和分配计数拒绝只比较 stable/unstable sort 的旧微基准。实现以同一 authority 读锁
  O(1) 克隆 COW registry 与 management generation 后立即解锁，并按已发布 N/K summary 自适应选择 registry
  snapshot scan + sort 或现有 64-shard canonical-display locator/id ordered management scan；两条路径统一跨 scheme
  顺序，旧 `ResourceScheme` enum-order 不作为兼容语义保留。未增加 persistent per-kind index、公开 snapshot API、
  兼容分支或 mutation-time memory；scan 缺行/多行、registry id 缺失或 kind/locator/revision/state 漂移均回到完整
  registry 真值，不返回半截结果。首轮独立 review 的 `C1/I2/M1 Not Ready` 精确发现排序、fallback、单 Barrier
  writer 证据和历史 hash 保留问题；修复后 focused `10/10`、完整 `zr_resource` 投影
  `120 passed / 0 failed / 2 ignored`，stdout 已保留到 D 盘。两轮 11-sample 复测暴露互相矛盾的 isolated regression，
  第一轮 review profile 又因 comparator 不等价被撤销；最终两轮等价 31-sample/3-warmup release matrix 均过门，
  canonical run 最差 median 为 +11.027%，100,000 全命中由 310.55 ms 降至 170.79 ms（-45.003%），所有 10,000+
  workload 的 lock p95 至少降低 99.893%，writer-attempt handoff 下 writer-wait p95 由 362.89 ms 降至
  66 us（-99.982%），100,000
  全命中 requested/peak allocation 分别降低 58.79%/21.47%。证据只覆盖本机 CPU/锁/分配，不声明功耗或跨引擎
  能耗等价；修复后独立复审为 `C0/I0/M0, Ready`。current-hash managed job
  `03ec12d4aed34026a43ca913a6c901e8` 在进入本切片前仅被 foreign
  `zircon_runtime_interface/src/runtime_api/session/editor_transform.rs:182` 的 E0599 截断：
  `ZrByteSliceError` 未实现 `Display`；job exit 101 后已 release，Cargo 进程树为空且 D 盘 ephemeral target 已删除。
  因此 managed Cargo 记为 foreign blocked，Frameworks01 不越权接管 interface owner，仍不提升 M1、不提交、
  不发送企微。
- 2026-08-27 M1 `ResourceManagementProjection::apply_delta` 已达到
  [`three_authority_source_and_direct_profile_green / independent_review_green /
  managed_cargo_current_hash_pending / milestone_not_accepted`](01/2026-08-27-m1-resource-management-projection-cow-page-plan.md)：
  在完整 Resource authority、真实 runtime/editor/graphics consumer 与 Unreal Asset Registry/Bevy 参考复核及 D 盘
  pre-profile 后，私有 64 个 ID-hash ordered-row shard + 64-way merge scan 已硬切为全局 canonical 256-row COW page、
  1,024 个随机化 UUID ID shard 与 1,024 个 locator shard；首版 exact8 profile 揭示全局 `Sparse|Rebuild` 会把
  summary/locator/ID 三个 authority 一起重建，4,096 mixed/remove/rename 分别回退 184.62%/107.57%/98.17%。
  分阶段 profiler 定位 summary+locator 与 ID 全量重建占可拆分工作约 76%，随后硬切为 ordered storage、ID index、
  locator index 三策略独立选择：summary 永远 delta，结构 merge 只排序 order-key 变化项，dense 同键页替换线性扫描，
  sparse 页替换按页二分，ID/locator 各自按规模选择 COW 或 rebuild。最终 direct current-source 为行为 `11/11`、
  mixed differential `24/24`；旧 shard/heap/固定 hasher/compatibility path 均无正向 consumer。
  三轮 reconstructed-old/final median-of-medians 中，100k/64 spread time/bytes 改善 98.86%/88.57%，
  100k/4,096 spread 改善 72.84%，100k 全量 revision 改善 25.54%，4,096 全量改善 35.76%，no-op +0.15%；
  4,096 structural add/mixed/remove/rename 改善 73.57%/43.91%/44.46%/47.63%，均过既定 timing 门。
  小规模 initial build 的固定 1,024-shard 成本为 0.34-0.48 ms，保留为后续 adaptive/empty-shard-sharing 项；
  shared-machine round spread 为 22-174%，故不升级为功耗或跨引擎绝对值结论。
  最终独立只读复核逐一匹配六份源码指纹并返回 `C0/I0/M0, Ready`：三类 authority、重复/交换/全删恢复、
  结构 range merge、页上限与 `Arc` identity、delta summary、随机 hash authority 及 sparse/dense 门槛均未发现问题；
  此 Ready 只允许进入受管验收，不等同于 managed Cargo 或里程碑 accepted。
  优先 docs convention gate 在共享树仍为 1,352 条/372 文档 RED，但本父计划与 ResourceManagement 子计划均为 0 条；
  `engine-code-review-findings-2026-06.md` 对三项 ResourceManagement 专属符号定向扫描为 0 命中，通用的大文件、
  单 authority、复制热点与禁止兼容双轨要求已由本切片的结构和 profile 证据覆盖。
  managed job `a576e52426bf4a7e9928b7ebc8093f7e` 自然运行约 36 分 54 秒后，在进入本切片编译前被 foreign
  `zr_rhi_wgpu` 3 条 `u32 -> u64` E0308 阻断，已 release 且 D 盘 target 删除；又因 job 早于最终 exact-reserve
  source，不冒充 current-hash 票。最终 real-interface/UUID harness 也没有 current-source rlib 票，必须等待
  managed current-hash Cargo，因此不提升 M1、不提交、不发送企微；
  `core::resource::io::{atomic_write, atomic_write_new}` 保持稳定 façade，未改写 IBL 或两份冻结 Editor blob。
- 2026-08-28 `zr_resource` schema-3 已修正为显式 read-set/write-set 模型：779 个 atomic inputs、555 个 Rust
  consumers 与 225 个 supplemental candidates 只作为封存读集；真实迁移图为 87 个 no-compat operations、
  156 个写路径，write manifest 为 `12377714...`。旧 56 条 executable-owner 门禁是把 dirty read inputs
  错当成 writes 的假阻塞；真实外域写路径只有根 `Cargo.toml`，已由 coordinator delayed patch 151 精确接线并
  转移归属。108 个其余 eligible 路径完成原子 transfer，156 条租约零冲突。共享树随后应用确定性
  `4ee6898d...` 补丁：155 个 emitted changes + 1 个 pre-applied root path，建立 71-file `zr_resource` 唯一实现，
  删除 69 个旧实现路径，只保留 2 个 curated Runtime facades；旧 private-owner 引用为 0，
  `core::resource::io::{atomic_write, atomic_write_new}` 继续服务 IBL。完整量化记录见
  [`01/2026-08-24-m1-zr-resource-current-source-preflight.md`](01/2026-08-24-m1-zr-resource-current-source-preflight.md)。
  非 Cargo 工具链与静态结构检查已绿；Windows managed `zr_resource` build 已绿，同一 current-hash test
  binary 三次为 `150 passed / 0 failed / 3 ignored`，但 Cargo wrapper 两次只返回无 test stdout 的 exit 101，
  故受管测试不冒充 GREEN。覆盖全部 156 路径/69 deletion tombstones 的验证票 `ccf45045...` 已排队且不轮询；
  Runtime/App/Editor upward gates、integration review、M1 commit 与企微通知仍 pending，不把 source
  implementation 冒充 milestone accepted。
- 2026-08-28 M2 RHI/WGPU surface owner 复审确认一项 fail-closed 缺口并完成 source repair：公开可构造的
  `SurfaceFrameLease` 此前在 deterministic/production present/discard 只按 frame id 终结，伪造 session、target、
  default view 或 descriptor 可消费真实 acquired frame，production discard 还会在完整 lease 校验前取消真实 frame
  的 Accepted tickets。两套 backend 现在保存并比较完整 lease identity，public discard 在任何 cancellation/
  terminalization 前校验；session/device teardown 继续走私有 trusted frame-id owner。新增行为回归覆盖四类 forged
  lease 并要求真实 frame 保持可用。精确 `rustfmt`/diff GREEN、source invariant `8/8`，四个 touched production/
  test owner 为 769/483/586/434 行，均低于 800 行；434 行 owner 的本轮新增内容仅位于 `#[cfg(test)]`。二次源码复审还
  发现 reconfigure/discard/destroy 原先逐 ticket 执行 `status -> cancel`，并发 flush 可在两次锁获取之间把
  Accepted 推进到 Submitted。production surface lifecycle 已硬切到 submission owner 的单 `queue/state` 锁批量
  `settle_abandoned_submissions`，并在锁释放后一次性投影 Cancelled diagnostics；旧逐 ticket 路径为 0。
  独立首轮 review 为 `C0/I1/M0`，I1 是缺少真实批结算/并发行为回归。首版 mixed unknown、reserved + pending +
  duplicate、diagnostic exactly-once 与单-ticket barrier race 虽有 5/5 票，第二轮仍为 `C0/I1/M1 Not Ready`：
  单 ticket 无法检测批内 partial cancel、两种执行顺序没有确定性覆盖，且设备初始化可能静默跳过。最终测试硬切为
  同批 4 个 committed-pending tickets，分别确定执行 settle-first/flush-first 后再做 16 轮竞争，任何 `1..3/4`
  partial flush 立即失败；adapter/device 不可用改为 fail-fast，并接受合法 Submitted/Completed 回调。最终 source hash
  `9c8bf87e5c2be96ff80f5e83dee324a3ff5988e365da9d3c0ed9d6436dd5719a` 的 managed job
  `5cf107ed3fd4438183c0040f4945962d` / run `cb6d67c9e1944979b8705f1f9b07f691` 已终态
  `5 passed / 0 failed / 389 filtered`，测试 14.75 秒、编译 1m24s、exit 0；第三轮独立 review 为
  `C0/I0/M0 Ready`。fresh domain audit 为 3,258 refs / 72 edges，
  `rhi→rhi_wgpu` 保持 0；docs convention 全局仍为 1,514 条/401 文档 RED，但本父计划、Failure 与新 preflight
  为 0 条。完整架构/参考与验收状态见
  [`01/failure-2026-08-02-rhi-wgpu-presenter-and-backend-contract-test-owner.md`](01/failure-2026-08-02-rhi-wgpu-presenter-and-backend-contract-test-owner.md)。
  同次 review 识别 acquire/session teardown 的 `O(active_frames)` 扫描与 deterministic submission attachment 的
  `O(active_frames * commands)` 候选瓶颈；已在
  [`01/2026-08-28-rhi-wgpu-surface-session-index-preflight.md`](01/2026-08-28-rhi-wgpu-surface-session-index-preflight.md)
  写入 Unreal/Bevy 约束、E 盘 pre-profile、WPR/WPA/power 与斜率门。没有 pre-profile 数据前不实现索引优化，
  不声明耗时/功耗改善。focused managed `zr_rhi_wgpu` 与独立 review 已完成；Runtime/Editor upward gate、
  真实窗口/PNG/RenderDoc/WPR、fixed return、M2 commit 与企微仍 pending，因此 M2 和计划 01 均不 accepted。
- 2026-08-30 R7-C namespace-admission profile harness 已落到
  `zr_resource/src/io/transaction/engine/tests/namespace_profile.rs`，覆盖 depth `2/16`、规模
  `1/100/1,000/10,000` 与最少 `31` 个样本，并把 CPU/墙钟、MAD 与 metadata/alloc/RSS/power unavailable
  明确输出到 D 盘 managed target。首个 current-source managed job `973eb8ed8f9044b39a5e15d361675b83`
  以 exit `101` 暴露夹具错误：单路径样本不可能构成祖先/后代冲突，却被错误断言为 RED；生产
  `reject_live_namespace_overlaps` 未被该失败证明有缺陷。夹具已修为 `path_count=1` 归入
  `no_conflict`，`rustfmt +1.94.1 --check` 与 Frameworks01 resource static guard `15/15` GREEN；
  当前修复哈希为 `8F0206FC41F8AC0FFE7036434E73186EA0B66D3C4FCA4EEB2A7BFD8B0070D8A3`；report 写入只增加
  可审计输出，不改变生产校验算法，尚不声明 R7-C/R7-D、功耗或优化收益。
- 2026-08-30 R7-C 修复夹具的 managed rerun 已登记为 job
  `92f7f054159f411ca855aeb055e21f6c`，使用 D 盘隔离 target，开始于 `01:13:36`；截至 `03:20`
  仍自然运行在 `last_ancestor_collision-16-10000`，测试进程 CPU 约 `5,900 s`，无 failure/error 输出，
  进程树与临时输出目录均存活。该中间状态只证明深层大批量物理路径解析是 material-cost 候选，不能代替
  16 组 `p50/p95/MAD` 样本；作业仍不得取消，R7-C/R7-D 与功耗结论继续保持 pending。
- 上述 job 已于 `03:38:35` 以 exit `0` 完成并于 `03:38:45` release，但 validator wrapper 没有把测试
  stdout 暴露到外部日志，故不把成功状态误记为量化证据。为保留可审计指标，profile 夹具仅增加了 D 盘
  独立 report 文件写入（不改变 `validate_inputs` 或断言），同一 current source 的第二次 managed job
  `2794501fb32743a794423b8bab0279d6` 于 `03:49:07` 启动，构建已 OK；该作业随后以 exit `0` 完成并产出
  report，详见下一条完成记录。
- 第二次 job 已于 `06:12:55` 以 exit `0` 完成并于 `06:13:10` release，进程树为空；D 盘 report
  `zircon-namespace-profile-report-36316.txt` 共 16 行，SHA-256=`ECDC6C1E3D04EA41E79996C41D46406977C534F89285BEA44B970BE0F4BBB744`。
  关键 current-source admission p50/p95（31 samples）为：depth=2 无冲突的 100/1,000/10,000
  路径分别 `343.4165/379.7604 ms`、`3.5643/3.8088 s`、`38.1404/41.5826 s`；depth=16 无冲突
  分别 `1.4248/1.9142 s`、`9.0043/17.5372 s`、`82.7582/86.0916 s`。depth=16 末项祖先冲突
  的 100/1,000/10,000 路径分别为 `750.6735/831.3955 ms`、`7.8062/8.4091 s`、
  `80.1322/84.0125 s`；MAD 同样已写入 report。测试仅测 admission wall-clock；metadata query、
  allocations、RSS 与 power 由夹具按约定标记 `unavailable`，所以 R7-C 仍是 partial baseline，
  不得提升为完整 WPR/ETW/RSS/I/O/功耗证据；后续 R7-D 仅允许在不改变这些合同的前提下做受限实现，
  且必须补齐同条件复测后才能声明收益。
- 2026-08-30 R7-D 受限切片已落到 `zr_resource::io::transaction::engine::validate_inputs`：对已解析的
  `PathIdentity` 引用做 component-aware ordering，再做相邻 containment 检查，移除逐项物化全部
  strict ancestor identity 及其重复 `PathBuf` 克隆。排序比较仍受路径深度 `D` 影响，重叠阶段
  最坏为 `O(P log P * D + P * D)`、临时内存为 `O(P)` 引用；不改变 `O(P*D)` 的物理解析，也不
  缓存 filesystem metadata；Windows 使用 UTF-16、大小写不敏感的
  `CompareStringOrdinal` 组件序，非 Windows 保留 raw basename 语义。新增
  `component_ordering_catches_interleaved_ancestor_targets` 回归测试；`rustfmt +1.94.1 --check`、
  `git diff --check` 与 resource static guard `15/15` 通过。focused managed job
  `ea010f901c064decab4956a8b3bb1063` 因宿主等待窗口结束而记录为 `orphaned/exit_code=null`，已由
  request `85190d6bf37d4ce7a266896b2ce6c2a9 release`；第二次尝试因活动 job
  `153344c01616403fa7732f0d1a03f43d` 占用兼容池而被 `cargo_reuse_pool_busy` 拒绝。故 R7-D 仍为
  partial implementation，尚无 terminal Cargo ticket、paired profile 或任何性能/功耗收益结论。
- 随后重试仍在 Cargo 启动前被协调器以 `cargo_reuse_pool_busy` 拒绝，当前持有者为 leased job
  `8f1caadae3ac484d85b8c9b82a8b887b`（观测时无运行进程）；这是调度门禁而非测试结果，不再重复
  占用同一兼容池。
- 2026-08-30 R14 根据 R7-C wall-clock 证据完成路径解析结构复核：`PathIdentity` 缺失路径解析已从
  根到叶逐组件探测改为叶到根寻找最近存在祖先，保留一次 canonicalize 与既有尾部规范化；新增
  `split_at_deepest_existing_ancestor_scans_from_leaf` 回归，`pathing.rs` SHA-256=`E22FC4FF9A9D943B9F4A9834AD229EDCE2A0CD6D30AE5D18B9F254B85E127471`，Rustfmt/diff-check 通过。
  focused managed Cargo 在 build 前仍被外部 `D:\ZirconBuilds\mvp-test-fixtures-36724` 的
  `unmanaged_artifacts_detected` 阻断；先前 profile job `738d82371b2c41808724ae44dac61708` 因
  超时后以 exit 137 终止，未产出报告。R14/R7-C 仍 pending，未声明性能或功耗收益。
- 2026-08-30 R14 独立只读复核返回 `C0/I1/M0, Not Ready`：component ordering + adjacent
  containment 的 R7-D 算法未发现漏判，但 leaf-to-root 初版依赖 `Path::file_name()`，会在未创建 tail
  中的 `..` 上错误拒绝旧版可规范化的绝对路径。修复后先用 `Path::components()` 构造 probe path，
  并在 filesystem probe 前显式剥离 trailing `ParentDir/CurDir`；这同时避免 Windows 把
  `missing\..` 的 metadata probe 误判为一个已存在物理祖先。新增
  `split_at_deepest_existing_ancestor_preserves_parent_components`，并把 commit/recovery 测试夹具改为
  canonical Windows wire path，保留 recovery 的 `has_exact_operation_path_encoding` hard cut。
  当前 `pathing.rs` SHA-256=`A1494DD1D9BB8D269A7CF0E4B550F2E069CB8D2F9251F9852720667B11BB6FDC`；
  Frameworks01 resource static guards `20/20`、Rustfmt 与 diff-check 通过。
- 本轮 managed `zr_resource --lib` 首次完整 job `f295a8ef81464f64af1a3c0b9be312da` 为
  `188 passed / 2 failed / 4 ignored`：两处失败均由恢复测试夹具把非 verbatim `D:\...` 当成 normalized
  physical wire path 引起，生产 recovery 正确 fail-closed；夹具 canonicalize 后第二次完整 job
  `17022bc8a1d84629be775fd5e56a055a` 为 `190 passed / 1 failed / 4 ignored`，唯一失败是新增
  ParentDir 回归暴露 Windows metadata 的 `missing\..` 规范化差异，随后已按上一条修复。最终 focused
  rerun request `482b996bb963487f9f4642c114c335c8` 在 Cargo 启动前终态失败为
  `cargo_cpu_lane_reserved`（reservation `531837a1b7fe442a8dfa057afbed4a4d`，foreign Session），不是测试
  结果。第二次独立只读复核已对当前 `A1494D...B6FDC` blob 返回 `C0/I0/M0, Ready`，确认
  ParentDir/CurDir、Windows prefix/root、symlink ancestor 与非 `NotFound` 错误语义未回退；因此 R14
  仅剩相同 current-source managed terminal GREEN 才可进入集成候选。
  当前没有 paired post-change profile、WPR/ETW metadata query、allocation/RSS 或 power 证据，继续禁止
  声明 R7-D/R14 的实测耗时/功耗收益、瓶颈消失或最优规模。
- 2026-08-31 R14/current-source Resource 行为门已由 coordinator-native 终态证据收口：最终
  `pathing.rs` SHA-256=`EE5EADBD55DAFB10F098D1DB102C3CF1443BC7E8C6A29DC9964E43835C107AFD`；focused
  parent-component job `6dacbed14ba7471b855d6d58e27e1a9b` 为 `1 passed / 0 failed / 194 filtered`，完整
  job `bf4648ee2deb47efaba2dbb6ef22b000` 为 `191 passed / 0 failed / 4 ignored`，libtest 12.16 秒。
  同一 current source 的 event-stream owner 已从 780 行内联混合文件机械拆为 615 行 production root 与
  610 行 folder-backed tests，哈希分别为 `C34D625D...27D313`、`B3CD7783...63A02`；完整 managed job
  `4bcc417333ec45039d835225e5c41448` / run `d6f1b5b6fdd645e0a8c76b4bbab4b424` 再次为
  `191 passed / 0 failed / 4 ignored`，libtest 9.48 秒、exit 0。durable transaction test owner 又把三条
  pre-active abort 测试移入既有 leaf，使 root/leaf 为 782/128 行，哈希为
  `77DBA07F...1F17D` / `3F49B38D...410BD`；focused managed job
  `6a9b7c043d7f4c089a24b1562c9930a3` / run `4b8741119d634c67aa91d59f85652196`
  为 `4 passed / 0 failed / 191 filtered`，libtest 0.20 秒、exit 0。全 crate Rust owner 当前 0 个超过
  800 行，Resource priority static guards 为 20/20 GREEN。两次 F 盘冷构建主要成本来自协调器 release 后删除
  target 并重新物化索引/依赖，不作为引擎性能数据。以上只收口结构与 current-source 行为票；R7-C/R7-D 的
  same-source paired profile、WPR/ETW allocation/RSS/I/O 与功耗仍 pending。
- 2026-08-31 ResourceManagement 算法重审继续保留已验证的三 authority COW 结构，不做无 profile 的局部改写。
  当前物理 owner 已在 hard cut 后漂移，旧 performance report 不能冒充 current-hash 票；后续必须用同一当前源码
  复跑 release workload 与功耗观测，再判断 page/shard/threshold。Unreal Asset Registry 的独立 canonical asset
  storage 加 package/path/class/tag accelerator、stable slot 与解锁后事件发布仍是本轮结构基准。既有旧/新对照中
  100k/64 spread 改善 98.86%、100k/4096 spread 改善 72.84%、dense 100k 改善 25.54%，但这些数值只证明
  当前三 authority 方向，不能证明 hard-cut 后当前 blob 的产品级收益或功耗等价。
- 2026-08-31 Random checkpoint schema hard cut 由 Runtime22 Session
  `root-runtime22-checkpoint-atomicity-20260829` 独占；Frameworks01 不编辑其 DTO、registry、service、restore、
  replay 或 eviction owners。验收固定为 version-2-only：每条 stream 绑定 master-seed generation，constructor、
  manual serde 与 restore fail-closed 拒绝 mismatch；checkpoint/evict_world/evict_entity 在唯一
  `registry -> seed` 锁序下原子捕获 entries+generation；`evict_stream -> Option<RandomState>` 仅为不可独立恢复
  的移除观测，除非统一硬切为 generation-bound checkpoint。root `Cargo.toml`、`Cargo.lock` 与 Runtime manifest 的
  fresh preview `95c197de4b6c46e6a3a35663ed139da4` 仍只被 foreign executable owner 阻断；不得局部接线或先创建
  disconnected `zr_kernel`。先完成 Runtime22 v2，再原子接线 `zr_contracts`，最后才 materialize `zr_kernel`。
- 当前 M1 仍为 `source infrastructure materially advanced / milestone_not_accepted`：Runtime/App/Editor upward
  product gate、ResourceManagement product trace/power、Random v2 + manifest integration、独立集成复审均未
  完成。因此本轮不提交 coordinator milestone commit，也不发送企微；计划继续转向不依赖这些验收门的可落地基础设施。
- 2026-08-31 Resource readiness graph 已先完成整体算法重审，未直接改生产代码。current DFS 的依赖顺序/重复项
  不规范、原生栈深度、cycle synthetic `Loaded`、64-bit fingerprint 语义相等与 64-shard sparse clone 风险已写入
  `frameworks/01/2026-08-31-m1-resource-readiness-graph-architecture-audit.md`；候选方向为 iterative、fail-closed
  SCC + incremental work queue，但必须先由 current-source RED/profile 决策。第二轮 whole-module review 又确认
  generation/dependency revision 的 `wrapping_add` 会破坏 cache identity 单调性，semantic stamp 必须保留 exact
  canonical edge/child revision equality，hash 只能加速而不能决定 revision。Unreal `FDependsNode` 的显式
  dependency/referencer + sorted state 与 `FEventLoadNode2` barrier/work queue 分层作为主要结构依据。test-only 基础设施已形成首版：生产 owner
  由 432 行降至 357 行，新增 cycle/依赖规范化/10k deep-chain behavior RED，以及隔离子进程的 1k--100k release
  profile（p50/p95/MAD、allocation count/bytes/peak、graph cardinality，报告根强制非 C 盘）。复审发现当前
  `iter().cloned()` 在计时区内混入 registry-to-update 快照成本后，worker 已硬切为每个 topology 两个独立进程：
  `manager_end_to_end` 保留输入快照成本，`evaluator_only` 在计时/allocator 前准备 owned updates；orchestration、
  raw/summary CSV 与 metadata 均携带稳定 scope，schema 升为 v2。profile owner 637 行，SHA-256=
  `2F020B4C...6EDBC9`，focused 静态契约 1/1、Rustfmt/whitespace GREEN，但两类 scope 均尚未执行，不存在性能样本。
  Resource 静态边界 22/22 GREEN。两次受管执行中，首轮按 `--locked`
  正确拒绝新增依赖，移除 `sha2` 后第二轮已越过 lock 与依赖编译，
  但在 foreign Runtime Interface `ActivateLink { href } -> { link_target }` mixed-era bridge 处以 E0026/E0027 终止，
  尚未编译/执行 `zr_resource`。因此 production readiness hard cut、性能/功耗结论和算法最优规模声明继续保持 blocked。
- Open cross-plan Failure：RuntimeInterface03 已完成 `ActivateLink` generic host bridge 的 typed
  current-source hard cut；ResourceManagement profile R3 已证明原 E0026/E0027 消失，但 managed interface
  integration 仍未回传；见
  [runtime-interface-ui-activate-link-field-mismatch](../../optimize/zircon_runtime_interface/03/failure-2026-08-31-runtime-interface-ui-activate-link-field-mismatch.md)。
- 2026-08-31 ResourceManagement profile R3：managed job `28eb6b1ee6a649e79a8cac8c19dc5c21` / run
  `071e0c99214e4abd965e52a0ebf9bfda` 以相同 release harness 越过 typed-link bridge，随后在 foreign
  `zircon_runtime_interface/reflect/schema_catalog/admission.rs` 的两条 E0502 停止，仍未编译 `zr_resource`。
  coordinator rollover 后以 `cargo_run_reconciled_from_orphaned_job` 终态回收，stderr SHA-256=
  `8EBDBF17...45E0C4`，release request `823f601d...` 后 Cargo/rustc 为空。该 foreign source 随后已由 owner
  改为 immutable alias validation、drop borrowed set、再 sort 的形态（current SHA-256=`18D866B7...B2BA7`），
  但尚无 managed owner receipt；Frameworks01 未编辑/claim。当前仍无 latency/allocation/RSS/power 样本，
  不启动 ResourceManagement 或 readiness production 算法改写。
- 2026-08-31 ResourceManagement profile R4：managed job `84f3507f1dee480184e94f5cbaf9fdb2`
  在 sccache 编译 Runtime Interface 依赖时，job-scoped
  `scratch/<job>/temporary/sccacheFuHIFi/deps.d` 父路径已消失，rustc 以 OS error 3、Cargo 101、wrapper 1
  终止；03:32:31 +08:00 已释放，Cargo/rustc 为空。该执行仍未进入 `zr_resource`，没有任何 profile 样本。
  唯一 handoff 为
  `frameworks/01/failure-2026-08-31-managed-cargo-sccache-temporary-path-lifecycle.md`，已路由给 active App08
  runtime-artifact-reuse/compact-validation owner；Frameworks01 不修改 Tooling source，也不在修复回执前重复
  Cargo。M1 继续为 `source infrastructure materially advanced / milestone_not_accepted`，不提交、不发企微。
- 2026-08-31 ResourceManagement profile R5：App08 第一版回传虽设置 `SCCACHE_CLIENT_SIDE=1` 并报告
  Pester 5/5，但 exact managed job `680c28eeb45f44ada781073ea28a3e50` 仍在进入 `zr_resource` 前失败。
  coordinator receipt 明确 `reused_from_job_id=84f3507f...`；常驻 sccache PID 1660 继续尝试在已删除的 R4
  `scratch/84f3507f.../temporary/sccacheY8m77e` 下创建服务端临时目录，Cargo 101、wrapper 1，无 profile
  产物。该回传已拒绝，canonical Failure 保持 open；修复必须提供稳定的非 C 盘 daemon TEMP 或受控
  health/rebind，并用真实 `--emit=dep-info,metadata,link` 请求复现/验证。M1 状态、禁止提交与禁止企微不变。
- 2026-08-31 Resource lease identity I0：在验收基础设施阻塞期间继续完成非验收生产基础设施。整模块复核确认
  wrapping residency token 与手工 `usize` ref-count 是同一错误生命周期 authority，已一起硬切为每个 payload
  incarnation 独立的 `Arc<ResourceLeaseIdentity>`。lease Drop 把精确 identity 移入 manager，在 Resource write lock
  内先消费该 owner，再以 slot sole-owner 判定最后释放；因此同时封闭 token 复用、count overflow、旧 lease 污染
  直接替换/重注册 payload，以及两个并发 Drop 都误判非最后 owner。结构 RED/GREEN、rustfmt parse、diff check 已
  通过，新增直接替换与并发最终释放回归；完整设计和 Unreal/Bevy 对照见
  `frameworks/01/2026-08-31-m1-resource-identity-rollover-architecture-audit.md`。managed Cargo 仍由 open sccache
  Failure 与共享 foreign compiler window 阻断，本 slice 仅 source-complete，未验收、未提交、不发企微。
- 2026-08-31 ResourceManagement profile R6：App08 的稳定非 C 盘 sccache daemon TEMP、独立端口与
  PID/start-time marker 已通过 exact origin 的真实 dep-info/metadata/link 验证。job
  `96c7732d445d4596b5e86f662d8333ed` 已同时编译 Runtime Interface 与 `zr_resource`，未复现 deleted TEMP、
  `deps.d` 或 OS error 3；基础设施根因按 origin 证据消失。该 run 随后在 Frameworks 自有 ignored readiness RED
  构造上触发 E0382，已以“move 前保存 ResourceId”修复，当前 SHA-256=`F7A2E749...DE7282`、rustfmt/diff-check 与
  attribution `423fe159...` 通过。下一次 exact acquire request `b2b1a4b0...` 因 RuntimeInterface03 FIFO
  reservation 未创建 job；canonical Failure 继续 open 到相同命令实际产出两类 profile artifact。当前仍无
  current-source latency/allocation/RSS/power 样本，M1 不验收、不提交、不发企微。
- 2026-08-31 Resource event order I2 已完成代码前原子性预检，生产源保持不变。整模块复核确认当前
  `next_sequence`/receiver cursor 的 `wrapping_add` 会复用事件顺序，mandatory
  `oldest_available_sequence: u64` 无法表达终态，手工 `AtomicUsize publisher_count` 又形成第二生命周期
  authority；同时 Resource authority 已提交后再做 fallible publish 会丢事件。锁定方案是在
  `prepare_commit` 持有 `commit_serial` 时先导出 exact event batch 并做不推进游标的 private permit preflight，
  空间不足则在任何 Resource mutation 前 typed reject；commit 释放 Resource write lock 后再不可失败地消费
  permit。顺序硬切为 `Option<u64>`，允许 `u64::MAX` 恰好发布一次后进入 `SequenceExhausted`，Gap successor
  同样为 `Option<u64>`；publisher 断开改由 `Arc/Weak` 生命周期投影，旧 raw publish、wrap 和 manual count
  均不保留兼容入口。Unreal Asset Registry 的 write-lock 内聚合 `FEventContext`、解锁后 broadcast 是主要锁序
  参考。完整协议和 RED/GREEN 矩阵见
  `frameworks/01/2026-08-31-m1-resource-event-order-exhaustion-atomicity-preflight.md`。I2 仍为
  `architecture_locked / production_not_started / current_profile_pending`，不据此提升 M1、提交或发送企微。
- 2026-08-31 ResourceManagement profile R7 调度：App08 已把 R6 helper 的职责进一步收紧为“只有 sccache
  daemon 初始化使用稳定 `cache/sccache-temporary`，Cargo/rustc/build scripts 继续使用
  `scratch/<job>/temporary`”。线上又发现 `\\?\E:\...` 与 `E:\...` 被字符串比较误判，Python 与
  validate-matrix 会交替无谓 rebind；已改为 Windows path identity 等价比较。Frameworks01 精确核对最新
  helper/test SHA-256 为 `7D1EB4FE...F74D9` / `4798293A...2CC9`，旧哈希不再作为 return contract；owner
  TDD 为旧实现 6/7、修正后 7/7，extended/display 正反向均复用。线上票
  `8b0d0f42f3854ce28982fb3400d2583e` 为 passed，物化约 56 秒、Cargo/link 10.63 秒，42261 daemon PID
  14596 全程不变且 `Restarted=false`。Frameworks01 artifact audit request
  `0943b0b4b4e847029b52a894ae86a81b` 终态 `completed`、`unmanaged=[]`。相同
  origin acquire 随后分别在创建 job 前让位于 RuntimeInterface02 reservation `FDDB9268...` 与 Runtime04
  reservation `B6AA834B...`，后者已消费为 foreign leased job `98F39BA0...`。Frameworks01 未取消、消费或
  绕过队首。最新 exact acquire request `16c82d55152a49f685c381263053016a` 又在创建 job 前终态失败为
  `cargo_cpu_lane_reserved`，队首是 RuntimeEditor reservation `7551B566...`；当前不存在 R7 profile
  job/artifact。canonical Failure、M1 未验收、禁止提交和禁止企微状态不变。
- 2026-08-31 Resource generation identity I1 已完成 whole-consumer 代码重审和架构预检，未改生产源码。除
  management/readiness 的 `wrapping_add` 外，复核确认 page/receipt、ProjectAsset 聚合、Render residency
  ticket、shader/material/pipeline cache 与 Editor09 workspace 仍把 immutable `Arc` 身份降级为 `u64`；同时
  management/readiness 两个 getter 分别加锁，paired Render 读取可能混合两个 commit era。锁定硬切为一次
  Resource authority read lock 捕获 `ResourceProjectionSnapshot`，对 generation/row 发布 opaque Arc identity，
  page/receipt 携带对象身份，ticket/cache key 停止 `Copy`/数值比较，missing 使用 `None` 而非 `0`。Unreal
  Asset Registry 的锁内 exact context + 解锁后分发、shared handle lifetime 是主要参考；不采用 Bevy 可回收有限
  generation。完整 source hash、owner matrix、RED/GREEN、复杂度和分层执行门见
  `frameworks/01/2026-08-31-m1-resource-generation-object-identity-consumer-preflight.md`。Asset、Render、
  resource-streamer 与 Editor09 当前没有一个可合法覆盖全 union 的 owner，Frameworks01 不吸收 mixed blob；
  I1 状态为 `architecture_locked / production_not_started / current_source_profile_pending`，M1 仍不提交、不发企微。
- 2026-08-31 durable I/O artifact identity I3 已完成整模块代码重审和代码前架构预检，未改生产源码。
  `atomic_file` 与 durable transaction 的 `fetch_add` identity 都会回绕；transaction artifact 又只带
  `{pid}-{sequence}`，不同 journal owner 指向同一 target 时，后发 owner 的
  `remove_reserved_if_exists` 可能删除前者 artifact。锁定硬切为共享 checked nonzero sequence：
  `u64::MAX` 只发放一次后进入 zero terminal exhaustion；transaction ID 增加 canonical journal-directory
  的完整 BLAKE3 owner token，journal version 6 硬切 version 7，旧两段 ID/version 6 无兼容 reader。
  `create_new`、WAL、owner lock、`PathIdentity` 与 fail-closed recovery 均保留；owner token 只做 namespace
  partition，不替代路径身份或扩张为全局 target lock。Unreal GUID + no-replace 分层是主要参考，Godot
  datetime/ticks suffix 只作较弱对照；当前本地 Bevy snapshot 未找到等价 durable multi-file journal，故不作
  正面设计证据。完整 hash、文件名预算、复杂度、RED/GREEN、
  profile 与 ownership gate 见
  `frameworks/01/2026-08-31-m1-durable-io-artifact-identity-exhaustion-preflight.md`。I3 状态为
  `architecture_locked / production_not_started / current_source_profile_pending`；M1 状态、禁止提交和禁止
  企微不变。
- 2026-08-31 Resource I/O false-capability hard cut：current-source 全联集确认公开
  `ResourceIo` 被 private supertrait sealing，且 Runtime/Interface/Editor/Plugins 中实现与调用均为 0；配套
  `ResourceIoError` 也只服务于该 dead trait。依据 no-placeholder 规则以及 Runtime25/160 的
  `FILESYSTEM-P1-011`，已删除两个声明文件和 `zr_resource`/Runtime 四处 re-export，不保留 compatibility。
  当前真正工作的 `io::{atomic_write, atomic_write_new}` 与 private durable transaction assembly 保持不变；
  future filesystem/source/mount provider 必须在具有 local provider、typed error、queue/cancel/shutdown 和真实
  Asset consumer 后以新契约发布。结构 TDD 为 RED 1 项、GREEN focused 1/1、full 8/8，产品 Rust 扫描
  `ResourceIo*` 为 0。完整参考、hash 与限制见
  `frameworks/01/2026-08-31-m1-resource-io-false-capability-hard-cut.md`。managed Cargo 仍待 FIFO exact profile，
  M1 不验收、不提交、不发企微。
- 2026-08-31 ResourceManagement profile R8/R9：首次 accepted acquire request `721ee523...` 只创建
  未 start、无 command/run/process 的空 leased job `a7829ba0...`，已通过 exact release request
  `7de2f504...` 回收。R8 job `9acd911f...` 真正编译到测试后暴露原记录命令遗漏 mandatory
  `ZR_RESOURCE_MANAGEMENT_PROFILE_DIR`；E 盘 binary 直接复现 `profile.rs:188` panic。R9 把报告目录显式
  绑定到 E 盘并保持 package/profile/filter/target 不变，job `f2f32800...` exit 0、release，42261 daemon
  PID 14596 未重启，sccache lifecycle Failure 已 fixed。产物为 14 场景 × 31 samples + 3 warmups：raw
  435 行 SHA-256 `8C8BB282...01230`、summary 15 行 `1244BF20...D05CD`、metadata 10 行
  `F7CC3E2B...79A07`。关键基线：100k no-change 为 p50 51.1567 ms、0 allocation，单条 revision 为
  p50 14.3 us，100k initial build 为 p50 420.7386 ms / 410,112 alloc / 46.86 MB requested，100k dense
  revision 为 p50 284.2293 ms。后续 whole-call-graph 复核确认 `no_projected_change_100000` 是 profile 直接向
  private `ResourceManagementProjection::apply_delta` 重放 100k 个相同 record 的防御路径；唯一 production caller
  `manager/commit.rs` 已在调用前以 `before != record` 只传真实 delta。因此 51.1567 ms 只能量化错误重放完整
  snapshot 的上界，不能证明当前引擎存在 no-change 全量扫描瓶颈，也不授权继续修改 page/shard/threshold。
  sparse/dense/initial-build 数据仍是有效的 projection 算法基线；下一轮热点结论必须来自 transaction/commit 或
  产品 trace。RSS/power unavailable，尚无能耗、engine parity 或额外瓶颈消失结论。I1 Resource support 可开始，但跨 Asset/Render/Editor
  owner 仍须合法分层；M1 不验收、不提交、不发企微。
- 2026-08-31 Resource generation identity I1 owned support 已完成 production hard cut：
  management/readiness generation 与 row 以 retained `Arc` 对象身份作为正确性 authority，
  `ResourceProjectionSnapshot` 在同一 authority lock 下捕获配对快照，page/receipt 不再把有限数字当身份；公开
  `sequence()` / `dependency_revision()` 正确性入口与 wrapping identity 均已删除，诊断计数只允许饱和观察。
  当前核心 hash 为 management `016E75CF...B24A5`、readiness `8441FC69...762D`、manager
  `F63C6A7F...805DA`、receipt `2E1581E2...F27C4`。Frameworks static boundary 为 `14/14`；Asset、Render、
  resource-streamer 与 Editor mixed consumer 仍按各自 owner 分层，不由 Frameworks01 吸收。I1 为
  `resource_support_implemented / managed_validation_blocked / consumer_migrations_pending`，未验收。
- 2026-08-31 Resource event order I2 已完成 production hard cut：顺序/cursor 改为 `Option<u64>` terminal，
  `u64::MAX` 只发布一次；prepare 在 `commit_serial` 内生成 exact batch 并做不推进状态的 private permit admission，
  空间不足在 Resource mutation 前 typed reject，commit 解锁 Resource authority 后不可失败地消费 permit；publisher
  lifetime 由 `Arc/Weak` 替代手工 `AtomicUsize`。final sequence、terminal lag、三种 receive、event-free terminal
  commit、dropped prepare、range rejection 与 final-publisher wakeup 均有 focused tests。核心 hash 为 event stream
  `BDCC7D4E...1D1E6`、commit `E227C5B5...BE5A`、error `E9BE4161...6678`。Editor exhaustive consumer 已交给
  legal Editor owner，Frameworks01 未改 mixed blob；I2 source-complete、未验收。
- 2026-08-31 durable I/O artifact identity I3 已完成 production hard cut：共享 checked nonzero allocator 在
  `u64::MAX` 后 typed terminal exhaustion；transaction ID 采用完整 canonical journal-owner BLAKE3 token + PID +
  nonzero sequence；journal v6 硬切 v7，legacy/malformed/owner mismatch 在任何 artifact mutation 前拒绝。不同 journal
  owner 不再能删除彼此的 staging/backup artifact，既有 create-new/WAL/owner-lock/PathIdentity 保持。新增 ignored
  release profile 固定 1/16/256 writes、31 samples、3 warmups，真实计时完整 durable commit，输出 p50/p95/MAD、
  allocation/requested-bytes/peak-live 与 source hashes 到显式非 C 盘；profile source hash
  `8CB2A4EE...629DC`。当前仅 `14/14` static GREEN，profile 尚未运行，RSS/I/O counters/power unavailable。
- 2026-08-31 I1-I3 首个合并 full managed job `8af64b6fdf4a4d928cc31fb92ea934ae` 于
  `08:47:50` 开始、`08:53:16` 结束、`08:53:26` release，exit `1`；rustc 在编译 foreign
  `zircon_runtime_interface/src/ui/dispatch/input/result.rs:116` 时 E0277，未到达 `zr_resource`。文件由
  RuntimeInterface03 exact owner 持有，Frameworks01 未 claim/edit；canonical Failure
  `frameworks/01/failure-2026-08-31-runtime-interface-input-route-clone-contract.md` 已由 request
  `a3c5aecac4594a2aa2cfeabae44ea2ec` 完成 import 并路由。该 failure 只阻断 managed evidence，不回滚
  production source；M1 仍不验收、不提交、不发企微。
- 2026-08-31 I1-I3 current-source 验证续跑：RuntimeInterface03 已由其 exact owner 修复 Clone 契约，
  `zr_resource` 现可独立到达。后台 managed job `a31f2a72cba34ced8b5dce40854359de` 首次暴露并收口 5 条
  owned compile diagnostics：transaction-private journal frame visibility、current fallible locator fixture 与 redundant
  mutable reborrow。后续 job `6486a2ea6b664b2ba0130ab61193090b` 的 production build 为 GREEN，并链接
  lib-test binary SHA-256 `D22BC22C...BCAEE`；单线程执行 229 tests 得到 217 passed / 1 failed / 11 ignored，
  I1/I2/I3 行为测试全部通过，唯一失败是 crate source guard 仍匹配旧两参数 `apply_staged` 文本。该 guard 已硬切到
  `self.events.len()`，同时清理一条 test-only must-use warning。精确 full rerun 当前在 Cargo 前让位于 foreign FIFO
  reservation `58f2405d88c346c786232b6a5bc956ab`，未创建 Frameworks job；因此 build GREEN 不外推为 full-suite
  GREEN，I3 release profile、RSS/power、独立复核、milestone commit 与企微仍 pending。
 - 2026-08-31 I2 blocking receiver 复核：更新 stale source guard 后，managed job
  `8295343708a64b3c91a2bf7feda1c96e` 再次 production build GREEN，但 current libtest binary
  `361E13E4...B9202` 卡在 `blocking_resource_event_receiver_wakes_when_the_last_publisher_is_dropped`。
  持久化 direct output 定位到生命周期自死锁：receiver 在持 event-state mutex 时 `Weak::upgrade()`，竞态下临时
  `Arc` 成为最后 owner，其析构进入 lifetime Drop 并重锁同一 mutex，Condvar notify 永远不可达。生产修复改为持锁
  读取不提升所有权的 `Weak::strong_count() == 0`；last-owner Drop 仍在同一 state mutex 下 notify，保留无丢唤醒
  握手。回归测试增加 1 秒结果通道上界，Frameworks static guard 禁止 locked upgrade；focused static `1/1`
  GREEN。foreign job `42419d28...` 与随后 fmt job 已自然结束，`cargo/rustc=0`后完整 Frameworks
  static boundary 复跑为 `14/14` GREEN（13.640 秒）。最新 managed-storage helper/test 精确 SHA-256 已
  核对为 `7D1EB4FE...F74D9` / `4798293A...2CC9`，不再接受旧哈希；与 storage lifecycle 相关的
  artifact-governance 定向审计 `4/4` GREEN（14.366 秒），42261 仍由 PID 14596 监听且未 rebind。
  exact focused managed request 随后在创建 job 前被 `request_overloaded` 拒绝；协调器一度显示 Frameworks
  successor leased job `b1f56fd4aad040119a839bcd90a8d072`，紧接着 daemon rollover 移除 runtime descriptor，
  `tray-recovery.json` 为 `circuit_open=true`。Frameworks01 不重复物化、不绕过 governance、不自行重启；
  focused/full Cargo、I3 profile 和独立复审仍 pending，当前仍不验收、不提交、不发企微。
 - 2026-08-31 full rerun reconciliation：在 artifact audit `47003a24...` 返回 `unmanaged=[]` 后，重新
   managed job `7111bf72605e4181ab5e46ab695228c6` 于 `12:01:40` 启动，build 于 3.05 秒完成，但
   libtest 终态 exit `101`；该次没有保留 harness 失败行。同一最新 binary 在持久 E 盘日志中
   单线程与默认并发各执行都得到 `218 passed / 0 failed / 11 ignored`（单线程 13.38 秒，5 次并发
   2.30--4.12 秒），因此无法将 `7111bf...` 的 exit 101 归因于当前 I2 生产逻辑。
   待 foreign Cargo 自然结束后必须按同一 current source 重跑 full managed 并收集 coordinator-native
   test 终态；I3 profile、RSS/power、独立复审、milestone commit 与企微仍 pending，不提产品性能结论。
- 2026-08-31 current-source Resource 行为门最终 GREEN：开启 `RUST_TEST_NOCAPTURE=1` 后，job
  `338da8564d6d4a8eab23b2b73968c76d` 将 opaque Cargo 101 定位为 Windows parent-component 夹具与词法
  authority 混用。生产 path resolver 现从平台原生 UTF-16/raw bytes 读取词法尾段，Win32 metadata 只判定物理
  ancestor；回归夹具用 `OsString::push` 传入未被 `PathBuf::join` 预先折叠的 `missing/../asset.zmeta`。
  `pathing.rs` SHA-256=`2AF0BAF1E6E5F5769E398D5537DFFB9850FA2CDFFA7A728E5DE4EA6626856445`。
  focused managed job `8bdf433f83b9403ebfc590c70f9f9c4a` exit 0；新 binary 默认并发全量 5/5 次均为
  `218 passed / 0 failed / 11 ignored`（1.77--2.84 秒）；full managed job
  `e0005cfdfca4412db14497195d7a52cc` 的 build/test 均 GREEN、exit 0。artifact audit
  `a52f5b4be1354507b81e488af0e46018` 为 `unmanaged=[]`，storage helper/test 仍精确为
  `7D1EB4FE...F74D9` / `4798293A...2CC9`。I1-I3 行为正确性门已通过；I3 release profile 正等待 foreign
  unmanaged structure-convention Cargo 自然结束，RSS/I/O counters/power、独立复审、milestone commit 与企微仍
  pending，不声明瓶颈消失、引擎对标或功耗收益。
- 2026-08-31 durable I/O release profile 已完成：foreign structure-convention Cargo 自然结束且
  `cargo/rustc=0` 后，artifact audit `32286d5a17f54e368eca884f86cfdeb7` 返回 `unmanaged=[]`；Windows
  managed job `04e2338eebc74091a4827eaedae49d98` 在 E 盘 target/report 上 `released / exit 0`。固定
  1/16/256 writes、31 samples + 3 warmups、128 B/write 的完整 durable commit p50/p95 分别为
  49.1682/74.8593 ms、269.5669/335.7085 ms、3594.3007/3996.5201 ms；对应 p50 吞吐为
  20.34/59.35/71.22 writes/s。raw 93 samples SHA-256=`B78344E1...8D0C1`，summary
  `BFB2A48D...C3EE9`，metadata `51A5CED6...90E1B`。256-write case 仍为约 362.10 allocations/write、
  60,503.50 requested bytes/write，说明固定事务成本被批量摊销但 allocation 工作仍近似 O(W)；没有
  before baseline 或 profiler 分解，不声称 hard cut 带来 speedup 或已定位下一结构优化。I3 当前为
  `source_complete / managed_behavior_green / release_profile_green / independent_review_pending`；RSS、OS I/O
  counters、功耗、引擎对标、独立复审、milestone commit 与企微仍 pending。
