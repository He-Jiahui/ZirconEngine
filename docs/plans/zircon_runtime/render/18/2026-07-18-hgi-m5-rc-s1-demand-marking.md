---
record_kind: milestone_candidate
status: waiting_validation
created_at: 2026-07-18
plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
workflow_node: HGI-M5-RC
slice: RC-S1-demand-marking
session: render18-hgi-m5-rc-s1-demand-marking-r3-20260718
related_code:
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/radiance_cache_state.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/scene_representation/representation.rs
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/test_sources/hybrid_gi_scene_representation.rs
  - zircon_plugins/hybrid_gi/runtime/src/test_support/render_feature_fixtures.rs
tests:
  - hybrid_gi_scene_representation_seeds_radiance_cache_from_surface_cache_then_voxel_fallback
  - hybrid_gi_radiance_cache_marks_unique_probe_lattice_demands
  - hybrid_gi_radiance_cache_clipmap_topology_is_independent_from_voxel_budget
  - hybrid_gi_radiance_cache_selects_clipmaps_symmetrically_at_strict_edges
  - hybrid_gi_radiance_cache_rejects_invalid_positions_and_clears_empty_scene
---

# HGI-M5-RC RC-S1 Radiance Cache demand marking

## 完成项目

- 为 Radiance Cache 建立独立类型的 4 级 clipmap 拓扑，每级逻辑分辨率固定为 48，cell size 固定为 `1/2/4/8`；该拓扑不读取 Voxel clipmap 数量、ID、中心、范围或驻留状态。
- Screen Probe 完成放置后，以其世界空间中心选择最细且具备完整三线性邻域的 Radiance Cache clipmap，并标记包围该位置的 8 个 Radiance Probe 格点。
- RC-S1 使用首个有限 Screen Probe 作为 frame-local anchor，以相对坐标计算 clipmap；输入、相对坐标和最终浮点格点任一非有限时拒绝 demand，不执行饱和整数转换。
- clipmap 两侧边界均按参考合同使用严格开区间 `(0.5, resolution - 0.5)`，避免镜像位置选择不同层级。
- 使用稳定有序集合合并重叠 Screen Probe 的重复需求，输出顺序固定为 `clipmap level + xyz probe coordinate`，为后续确定性 allocation/budget 阶段提供纯数据输入。
- 保留既有 Surface Cache 优先、Voxel radiance fallback 次之的 entry 构建合同；RC-S1 没有把 Voxel clipmap 冒充为 Radiance Cache clipmap。
- 新增四条 TDD 合同，覆盖重叠 Screen Probe 的 8 邻点去重、level 1 实际可达、Voxel budget 为 0/4 时 topology/demand 一致、正负边界对称、NaN/超范围拒绝和空场景清空。

## 当前证据

- RED：测试首先调用尚不存在的 `radiance_cache_probe_demands` 与 `radiance_cache_clipmap_topology`，静态符号审计确认生产侧无定义。
- GREEN 候选：生产实现和测试访问器已落地，Rust 1.94.1 `rustfmt` 与 scoped `git diff --check` 通过。
- 初次独立 review 为 `Critical 0 / Important 2 / Minor 2`；4 级层级不可达、非有限转换、边界不对称和测试不足均已修复，等待 fresh immutable snapshot 复审。
- fresh 复审为 `Critical 0 / Important 0 / Minor 1`；唯一剩余项是 production demand state 只由测试读取可能触发 `dead_code`，已沿用现有 cache entry 模式增加定向 non-test allowance，等待最终单点复核。
- snapshot 502 最终单点复核为 `Critical 0 / Important 0 / Minor 0`，5/5 路径两次 preview 均无漂移；reviewer 未运行 Cargo，focused validation 仍待协调器 FIFO。
- managed reservation `d4503b83179f4eb0a3ef192809cc382d`、job `d7ac0bd9c1ac4b09932ee4dff9378282`、run `4c1a706522ea4811b7bd1ca87c789e4e` 在编译 HybridGI lib test 时 terminal/released exit 101；5 个 `radiance_cache` 测试尚未执行，唯一错误是 test-support fixture 未跟随 `ProjectAssetManagerAccess` 硬切。
- r3 successor 复用已有 volumetric product fixture 的真实 `CoreRuntime + ManagerServiceHandle` 模式，并以持有 runtime owner 的 `Deref<Target = WgpuRenderFramework>` wrapper 保持所有既有 HybridGI test call site 不变；该修复仍是 GREEN 候选，不能把前一 job 记为通过。
- r3 snapshot 533 独立 review 为 `Critical 0 / Important 0 / Minor 1`；唯一缺口是无效坐标测试没有证明“首个 probe 无效时跳过并选取后续首个有限 probe 为 anchor”。测试已改为 NaN 在前、有限 probe 在后，并断言仍建立 4 级 topology 与精确 8 个 level-0 demand。snapshot 536 单点复核确认 exact6 无漂移，最终为 `Critical 0 / Important 0 / Minor 0`；受管 focused Cargo 仍待完成。
- snapshot 537 exact6 最终复核保持 `Critical 0 / Important 0 / Minor 0`。Rust 1.94.1 受管 exact gate 使用 reservation `95926ba08e5f4ed68572cbd44b113084`、job `feef12b0258748eda07e3c630d732585`、run `30f96f1a28e946b68583c3b148112a80`；命令为 `cargo +1.94.1 test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_hybrid_gi_runtime --locked --lib radiance_cache --jobs 1 -- --nocapture --test-threads=1`。
- 该 gate 已于 2026-07-19 14:14:53 +08:00 审计 terminal/released，`exit 101`、`live_process_pids=[]`、stdout 为空，5 个 `radiance_cache` 测试均未执行。最低失败位于 HGI 之外：`plugin/bridge` 的 `arc_swap` 依赖未接入 runtime manifest、`WeakBridge` 泛型推断未闭合，以及 `runtime_plugin_catalog/feature_blocking.rs` 的借用生命周期错误，共 7 个编译错误。
- 外部失败已路由到既有 Plugins01 owner/failure：`plugins01-bridge-stable-snapshot-r1-20260719` / `failure-2026-07-17-bridge-import-stable-call-double-mutex.md`，以及 `plugins01-runtime-catalog-derived-projection-r1-20260719` / `failure-2026-07-17-runtime-plugin-catalog-derived-projection-rebuild.md`。RC-S1 不吸收这些路径；须等待 fixed return 后在同一 source snapshot 语义下重跑 canonical focused gate。
- 共享 Git index 保持 0 staged paths；本记录未吸收其他 Render18、Voxel、Global SDF 或 shader 路径。
- fresh exact6 review 已完成，但 focused Cargo 因外部 Plugins01 编译闭包未到达测试执行阶段，因此本记录状态保持 `waiting_validation`，不申领 accepted/committed。

## 未完成项目

- `RC-S2 persistent state`：camera scroll、稳定取整、上一帧 indirection/cache 传播、history clear、generation、participation epoch 与 invalidation。
- `RC-S3 allocation`：free-list、分配/回收、淘汰、预算截断和确定性耗尽降级。
- `RC-S4 update pipeline`：allocate trace tiles、trace、filter、border fixup、mip 和完成 generation 的可见性门。
- `RC-S5 validation` 以及真实 WGPU/readback、产品 PNG、DX12 RenderDoc、长期性能与显存证据。

## 参考核对

- 本地 `dev/LumenInUE5.5.4WithComputeShader/Res/Shader/UpdateRadianceCache/MarkRadianceProbes.hlsl` 同样从 Screen Probe 世界位置选择 Radiance Cache clipmap，并标记三线性插值所需的 8 个邻接 probe。
- 本切片只采纳该 demand-marking 数据依赖，不复制参考实现的 allocator、历史传播或 D3D12 资源布局。
