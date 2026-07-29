# Frameworks01 M0 当前结构、依赖图与内部 crate 决策基线

> 来源：[`01-runtime-crate-decomposition.md`](../01-runtime-crate-decomposition.md) M0。2026-07-30 已用同一 production-only 审计器刷新 current-source JSON；7 月 13 日基线保留为历史比较，不再作为物理迁移输入。

## 1. 当前工作区事实

| 项目 | 2026-07-30 captured current-source snapshot |
|---|---|
| 根 workspace members | `zircon_app`、`zircon_reflect_derive`、`zircon_runtime`、`zircon_runtime/reflection_macros`、`zircon_editor`、`zircon_runtime_interface`、`zircon_hub` |
| Runtime 编译单元 | 单一 `zircon_runtime` package，`crate-type = ["rlib", "cdylib"]`；`zircon_runtime/crates/` 不存在 |
| Runtime 默认 profile | `default = ["target-client"]`；另有 `target-server`、`target-editor-host` 与 additive domain features |
| 重依赖当前位置 | `wgpu`、`winit`、`gltf`、`image`、`naga` 仍由 `zircon_runtime/Cargo.toml` 直接声明，尚未下沉到内部成员 crate |
| CI 当前入口 | workspace build/test、plugin workspace build/test、runtime profile matrix、runtime additive-domain matrix；尚无 `zr_*` member/依赖方向 job |
| 共享 dirty 边界 | `Cargo.toml`、`zircon_runtime/Cargo.toml`、`.github/workflows/ci.yml` 均有其他 Session 的 current changes；本切片只读，不覆盖、不归属这些文件 |

## 2. Production-only 域依赖图

- 原子采集的 current-source 机器快照：[`baselines/2026-07-30-runtime-domain-dependencies-production-only.json`](baselines/2026-07-30-runtime-domain-dependencies-production-only.json)，SHA-256 `cc3c01dce8aa4a5c200560984c056b30e9ee1b777bc0f37f7ba531b45af6deba`。
- 采集窗口对 `zircon_runtime/src` 全部 9,188 个 `.rs` 输入做 ordinal `path=sha256`、LF/no-final-LF 指纹；pre/post 均为 `348fe59c72c798e5e64babb5489910f30dbd00a58441ec11c1e176826519339e`。这证明 JSON 与一个不可变时点一致，不声称共享工作树此后不会变化；每个物理迁移切片必须原子重算。
- 历史比较：[`../05/baselines/2026-07-13-runtime-domain-dependencies-production-only.json`](../05/baselines/2026-07-13-runtime-domain-dependencies-production-only.json)。
- 该快照扫描结果：2,391 条 production direct references，76 条 domain edges。
- 已关闭前置：`core→asset/graphics/scene=0`、internal domain→facade=0、`asset→ui=0`、`graphics→ui=0`、`graphics→scene=0`。
- 仍阻止物理拆分的硬反向边：`asset→text=2`、`scene→animation=2`、`rhi→rhi_wgpu=1`。

当前最大边如下；它们用于确定后续拆分顺序，不代表全部允许保留：

| source | target | refs | M0 结论 |
|---|---|---:|---|
| graphics | core | 822 | M1 先拆 math/resource/contracts/kernel 后，graphics 改为依赖明确的底层内部 crate |
| asset | core | 262 | M2 asset 拆分前必须完成 core spine 的真实 crate 边界 |
| scene | core | 180 | 同上；scene 与 graphics 不得通过门面形成反向环 |
| graphics | render_graph | 113 | 合法 layer-4 → layer-3 候选边 |
| graphics | asset | 80 | 需要稳定 resource/manager handle 边界；不得把 concrete `ProjectAssetManager` Arc 带入 `zr_graphics` |
| graphics | text | 47 | 同层候选边；`zr_text` 必须先成为共享底层服务或经 contracts，禁止双向依赖 |
| ui | text | 38 | 合法 layer-5 → layer-4 候选边 |
| scene | asset | 30 | 同层边须通过稳定 asset/resource contract 或显式批准，禁止 crate 环 |

### 2.1 Layer-direction 失败复核

对全部 76 edges 应用刷新后的层级：

- 旧 lower layer → upper layer 的 `core→asset/graphics/scene` 已为 0；`core→engine_module=5` 与 `engine_module→core=8` 在 M1 同批吸收到 `zr_kernel`，不形成 crate 环。
- internal domain → facade 已为 0，证明 Frameworks05 的反向依赖硬切已生效。
- 当前硬反向边为 3 edges / 5 refs：`asset→text=2`、`scene→animation=2`、抽象 RHI 对实现 `rhi→rhi_wgpu=1`；必须在对应 Phase 前清零，禁止借 facade/re-export 绕过。
- 同层重点边为 `scene→asset=30`、`graphics→text=47`、`platform→foundation=1`、`rhi_wgpu→rhi=10`；最后一项是实现依赖抽象的批准方向，其余必须在物理拆分前固化为单向服务/contract。
- 新 current domain `foundation` 不是扫描缺口：它拥有 Kernel 级 config/event module 与跨 asset/scene 的原子持久化实现，依赖 `core/engine_module`，按权威吸收层规则锁定为 layer-1 `zr_foundation`。

硬反向边的最低 source owner 与硬切方向已经锁定，后续切片不得重新解释为允许依赖：

| edge | current source owner | 物理迁移前硬切方向 |
|---|---|---|
| `asset→text` | `asset/artifact/cache_payload/font.rs`、`asset/assets/font.rs` | `CompositeFontDescriptor` 的序列化契约下沉到 contracts/resource 可依赖的 owner；asset 不直连未来 `zr_text` 实现，也不通过 facade re-export 绕回。 |
| `scene→animation` | `scene/level_system.rs`、`scene/level_system/frame_state.rs` | level-system 所需的 clip event/cursor 输入下沉为 contracts DTO 或由 animation 通过 scene-facing service 提供；`zr_scene` 不依赖 future optional `zr_animation`。 |
| `rhi→rhi_wgpu` | `rhi/mod.rs` | 默认 WGPU presenter 构造上移到 facade/builtin 组装 owner；`zr_rhi` 只保留抽象 factory contract，具体 `zr_rhi_wgpu` 继续单向依赖 `zr_rhi`。 |

这 5 个 source reference 是 current JSON 中上述三条边的完整集合。任何 source path 或计数变化都必须先刷新机器基线，再修改本表；不能用历史计数继续执行 crate move。

Frameworks05 历史 failure 已返回 fixed：[`fixed-2026-07-13-core-contract-reverse-dependencies.md`](fixed-2026-07-13-core-contract-reverse-dependencies.md)。本次刷新发现的 3 条硬反向边作为 M2/M3 明确前置，不恢复旧 handoff 或兼容层。

## 3. 锁定的内部 crate 拓扑

以下决策按 Frameworks01 §3 与固定三包公开架构锁定：

1. `zircon_app`、`zircon_runtime`、`zircon_editor` 仍是公开根包；`zr_*` 只是 `zircon_runtime/crates/` 下的内部编译单元，不形成第四套公开引擎架构。
2. 内部 crate 统一 `publish = false`，加入根 workspace members；外部包、Editor、App 与插件只能依赖 `zircon_runtime` facade 和 `zircon_runtime_interface`，禁止直连 `zr_*`。
3. 名称与层级锁定为：
   - layer 0a：`zr_math`、`zr_resource`、`zr_contracts`
   - layer 0b：`zr_kernel`
   - layer 1：`zr_diagnostics`、`zr_foundation`、`zr_platform`、`zr_input`
   - layer 2：`zr_asset`、`zr_scene`
   - layer 3：`zr_rhi`、`zr_rhi_wgpu`、`zr_render_graph`
   - layer 4：`zr_graphics`、`zr_text`
   - layer 5：`zr_ui`
   - optional：`zr_script`、`zr_animation`、`zr_navigation`
   - development-only：`zr_dylib`
4. 依赖只允许高层指向低层；同层横向边必须经 `zr_contracts` 或在计划中逐条批准。`zr_contracts` 保持纯 trait/DTO，禁止 wgpu/winit 与业务实现。
5. 物理迁移使用源码移动和同批引用修正；内部 crate 之间不保留旧 module、alias crate、compat facade、bridge folder 或 legacy-path re-export。
6. `zircon_runtime` facade 的 curated re-export 只用于维持既定公开 API 所有权，不得让内部 crate 反向依赖 facade，也不得同时暴露旧内部 owner 与新内部 owner。

## 4. Phase 与 CI 影响锁定

- M1 先按 `zr_math/zr_resource → zr_contracts → zr_kernel → zr_diagnostics` 的底层顺序切出；当前 production 是 `framework→math/resource` 与 `runtime→framework/math`，反序会制造真实 Cargo 环。
- `core/framework/render/environment/source_cubemap/tests/projection.rs` 当前两处测试直接导入并构造 concrete Runtime `TaskPool`；M1 创建 contracts crate 前先迁到 kernel integration owner，再以静态守卫维持为 0。不允许 contracts 通过 dev-dependency 反向依赖 kernel。
- M2 再按 `zr_foundation → zr_platform/zr_input → zr_asset/zr_scene → zr_rhi/zr_rhi_wgpu/zr_render_graph` 切出，并把 winit/wgpu 等依赖下沉到真实 owner。
- M3 只有在 Frameworks05 的 `graphics↔ui`、`graphics→scene` 和 manager handle 前置完成后，才拆 `zr_graphics/zr_text/zr_ui` 与 optional domains。
- 每个新 member 都必须进入 workspace build/test；CI 另加 app/editor/plugin 禁止直连 `zr_*`、Cargo metadata 依赖方向、重依赖越层与 feature matrix 守卫。
- `Cargo.lock` 与 manifest 只在对应物理迁移切片中原子更新；本 M0 记录不编辑其他 Session 当前持有的 manifest/CI 内容。

### 4.1 M1 current-source 物理体量

下表按 2026-07-30 工作区中实际存在的 `.rs`（含尚未提交但已进入 current source 的文件）统计。它用于约束原子提交和 review 半径，不是允许把整个 layer 一次搬完的理由：

| future crate | current owner roots | Rust files | lines | M1 切片约束 |
|---|---|---:|---:|---|
| `zr_math` | `core/math` | 1 | 3 | 先建立最薄内部 crate；继续直接转发 Interface math，不复制类型。 |
| `zr_resource` | `core/resource` | 18 | 1,373 | 第二个硬切 owner；resource typed error/registry/lease 同批移动。 |
| `zr_contracts` | `core/framework` | 592 | 66,346 | 必须按 domain feature 分批迁入同一 crate 并逐批验证，禁止 592 文件单次无审查搬迁；source-cubemap projection 的两处 concrete TaskPool 测试先迁出。 |
| `zr_kernel` | `core/runtime` + `engine_module` | 203 | 29,695 | contracts 当前 source 迁完后再吸收这两个互相引用 owner；禁止保留旧 `engine_module→core` facade 回边。 |
| `zr_diagnostics` | `diagnostic_log` | 31 | 3,554 | kernel 稳定后独立；进程日志 I/O 不下沉到 contracts。 |
| `zr_foundation` | `foundation` | 16 | 2,420 | 只记录为 M2 第一个切片，不混入 M1 commit。 |

`core/mod.rs` 当前 33 行，只是 facade 组装/re-export 清单；`core/manager` 的 concrete handle/resolver 也按目标拓扑留在 facade。每个 M1 子切片必须同步缩减 `core/mod.rs`，但不得把这两个 facade owner 误计入 `zr_kernel` 源目录移动。

`zr_contracts` 的 66,346 行并非均匀分布：`core/framework/render` 单独占 249 文件 / 41,931 行（约 63%），其余 31 个 direct owners 合计 343 文件 / 24,415 行；`core/framework/tests` 与 `tests.rs` 又占 4 文件 / 2,571 行。因而 M1 的 contracts 物理迁移锁定为多次原子切片：

1. 先把 `render/environment/source_cubemap/tests/projection.rs` 的两处 concrete Runtime `TaskPool` 测试迁到 kernel integration owner，再以静态边界守卫证明 framework tests 对 kernel implementation 的反向引用为 0；此后才能创建 contracts crate，禁止引入 dev-dependency 环。
2. 创建 `zr_contracts` 后，先迁 kernel-neutral 的 shared DTO/trait owners，并逐批建立 facade curated re-export 与 domain feature；每批都必须证明该批没有 runtime/manager concrete 依赖。
3. input/window/camera/scene/picking/gizmos/text/ui 与 optional AI/physics/sound/net/animation/navigation 按审计得到的单向依赖序迁移；不能按目录字母序猜测 DAG。
4. `render` 最后单独分批迁移并运行 graphics/shader 上行门；禁止把 41,931 行 render contracts 与 kernel 物理移动放进同一 milestone commit。

上述 direct-owner 文件/行统计必须在每个 contracts 子切片开始前重算；M0 数值用于切片设计，不是永久预算豁免。

## 5. 未完成验收

- M0 冷构建、增量构建与 `cargo build --timings` 仍未采集：四个受管批次的方法、硬件和目标产物已锁定在 [`baselines/2026-07-30-build-timings.md`](baselines/2026-07-30-build-timings.md)，实际作业必须遵守现有 CPU FIFO，不能用 fallback target 抢跑。
- M0 尚不能标记完成；只有受管 Windows timings、硬件/命令说明、baseline artifact 路径与依赖图/决策记录同时齐全后才能进入完成态。
- M1–M4 均未开始物理 crate 迁移；当前 `zircon_runtime/crates/` 缺失是明确的未完成证据。

## 6. 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M0 | 2026-07-13 historical snapshot: workspace/dependency reconstruction + internal crate/CI decision lock | `historical_snapshot_frameworks_01_m0_structure_2151_refs_77_edges_reverse_18_facade_inbound_38_handoff_open_timings_pending` | 2026-07-13 | 历史快照：读取当日 root/runtime manifests、CI 和两个架构权威计划；确认当时 6 个根 workspace members、Runtime 单 package、`zircon_runtime/crates/` absent。复用当日 production-only 2,151 refs / 77 edges JSON，捕获 reverse-layer 18 refs 与 facade-inbound 38 refs，并向 Frameworks05 发布 failure handoff。该行只用于历史比较，不是 current-source 迁移输入。 |
| M0 | 2026-07-30 atomic current-source snapshot successor: graph/crate-DAG refresh + M1 slicing lock | `frameworks_01_m0_snapshot_2391_refs_76_edges_reverse_3_edges_5_refs_handoff_fixed_timings_pending` | pending | 7 个根 workspace members；9,188 Rust 输入 pre/post 指纹同为 `348fe59c...`，production-only JSON 为 2,391 refs / 76 edges，SHA-256 `cc3c01dc...`。历史 Frameworks05 handoff 已 fixed；快照内硬反向边为 3 edges / 5 refs。M1 物理体量与 contracts 分批边界已锁定；共享源码可继续演进，物理迁移前必须重新原子采集。四份受管 timings 尚未生成，因此 M0 仍未完成。 |
