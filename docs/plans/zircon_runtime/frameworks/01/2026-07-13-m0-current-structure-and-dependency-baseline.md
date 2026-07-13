# Frameworks01 M0 当前结构、依赖图与内部 crate 决策基线

> 来源：[`01-runtime-crate-decomposition.md`](../01-runtime-crate-decomposition.md) M0。当前机器依赖证据复用 Frameworks05 的 production-only 扫描结果，不复制第二份会漂移的 JSON。

## 1. 当前工作区事实

| 项目 | 2026-07-13 current source |
|---|---|
| 根 workspace members | `zircon_app`、`zircon_runtime`、`zircon_runtime/reflection_macros`、`zircon_editor`、`zircon_runtime_interface`、`zircon_hub` |
| Runtime 编译单元 | 单一 `zircon_runtime` package，`crate-type = ["rlib", "cdylib"]`；`zircon_runtime/crates/` 不存在 |
| Runtime 默认 profile | `default = ["target-client"]`；另有 `target-server`、`target-editor-host` 与 additive domain features |
| 重依赖当前位置 | `wgpu`、`winit`、`gltf`、`image`、`naga` 仍由 `zircon_runtime/Cargo.toml` 直接声明，尚未下沉到内部成员 crate |
| CI 当前入口 | workspace build/test、plugin workspace build/test、runtime profile matrix、runtime additive-domain matrix；尚无 `zr_*` member/依赖方向 job |
| 共享 dirty 边界 | `Cargo.toml`、`zircon_runtime/Cargo.toml`、`.github/workflows/ci.yml` 均有其他 Session 的 current changes；本切片只读，不覆盖、不归属这些文件 |

## 2. Production-only 域依赖图

- 权威机器基线：[`../05/baselines/2026-07-13-runtime-domain-dependencies-production-only.json`](../05/baselines/2026-07-13-runtime-domain-dependencies-production-only.json)。
- 扫描结果：2,151 条 production direct references，77 条 domain edges。
- 已关闭前置：`asset→ui=0`、`graphics→ui=0`。
- 已定位前置：`graphics→scene=1`，唯一 current owner 为 graphics module descriptor 读取 scene module name。
- 仍阻止重域物理拆分的主要边：`ui→graphics=28`、`graphics→asset=80`、`scene→asset=25`、`dynamic_api→graphics=8`。

当前最大边如下；它们用于确定后续拆分顺序，不代表全部允许保留：

| source | target | refs | M0 结论 |
|---|---|---:|---|
| graphics | core | 874 | M1 先拆 kernel/contracts/math/resource 后，graphics 改为依赖明确的底层内部 crate |
| asset | core | 247 | M2 asset 拆分前必须完成 core spine 的真实 crate 边界 |
| scene | core | 145 | 同上；scene 与 graphics 不得通过门面形成反向环 |
| graphics | render_graph | 109 | 合法 layer-4 → layer-3 候选边 |
| graphics | asset | 80 | 需要稳定 resource/manager handle 边界；不得把 concrete `ProjectAssetManager` Arc 带入 `zr_graphics` |
| ui | graphics | 28 | Frameworks05 M3 必须先把共享 text owner 独立为 `zr_text`/contract，再允许拆 `zr_ui` |
| scene | asset | 25 | 需通过稳定 asset/resource contract 或批准的下层依赖固化，禁止 crate 环 |

### 2.1 Layer-direction 失败复核

对全部 77 edges 应用 §3 锁定层级后，不满足目标 DAG 的边不是 0：

- lower layer → upper layer：3 edges / 18 refs（`core→asset=6`、`core→graphics=1`、`core→scene=11`）。
- internal domain → facade：6 edges / 38 refs（`animation→plugin=1`、`asset→plugin=4`、`core→plugin=15`、`platform→builtin=7`、`scene→plugin=9`、`script→plugin=2`）。
- 同层边为 9 edges / 74 refs；其中 `rhi_wgpu→rhi` 等是预期候选，`rhi→rhi_wgpu=1` 与 `scene→asset=25` 仍需在物理拆分前逐条证明或消除。
- 所有 19 个 current domains 均已映射，没有 unmapped domain；问题是已知 owner 方向不合规，不是扫描缺口。

最低共享原因与验收已发布到 Frameworks05：[`failure-2026-07-13-core-contract-reverse-dependencies.md`](../05/failure-2026-07-13-core-contract-reverse-dependencies.md)。该 handoff 未修复前禁止通过 facade dependency、alias 或 re-export 强行开始 M1。

## 3. 锁定的内部 crate 拓扑

以下决策按 Frameworks01 §3 与固定三包公开架构锁定：

1. `zircon_app`、`zircon_runtime`、`zircon_editor` 仍是公开根包；`zr_*` 只是 `zircon_runtime/crates/` 下的内部编译单元，不形成第四套公开引擎架构。
2. 内部 crate 统一 `publish = false`，加入根 workspace members；外部包、Editor、App 与插件只能依赖 `zircon_runtime` facade 和 `zircon_runtime_interface`，禁止直连 `zr_*`。
3. 名称与层级锁定为：
   - layer 0：`zr_kernel`、`zr_contracts`、`zr_math`、`zr_resource`
   - layer 1：`zr_diagnostics`、`zr_platform`、`zr_input`
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

- M1 先按 `zr_kernel → zr_contracts/zr_math/zr_resource → zr_diagnostics` 的底层顺序切出，避免把重依赖带入 layer 0。
- M2 再按 `zr_platform/zr_input → zr_asset/zr_scene → zr_rhi/zr_rhi_wgpu/zr_render_graph` 切出，并把 winit/wgpu 等依赖下沉到真实 owner。
- M3 只有在 Frameworks05 的 `graphics↔ui`、`graphics→scene` 和 manager handle 前置完成后，才拆 `zr_graphics/zr_text/zr_ui` 与 optional domains。
- 每个新 member 都必须进入 workspace build/test；CI 另加 app/editor/plugin 禁止直连 `zr_*`、Cargo metadata 依赖方向、重依赖越层与 feature matrix 守卫。
- `Cargo.lock` 与 manifest 只在对应物理迁移切片中原子更新；本 M0 记录不编辑其他 Session 当前持有的 manifest/CI 内容。

## 5. 未完成验收

- M0 冷构建、增量构建与 `cargo build --timings` 仍未采集：协调器当前有两个运行中的 compatible Windows Cargo blockers，按单池规则不得创建 fallback target。
- M0 尚不能标记完成；只有受管 Windows timings、硬件/命令说明、baseline artifact 路径与依赖图/决策记录同时齐全后才能进入完成态。
- M1–M4 均未开始物理 crate 迁移；当前 `zircon_runtime/crates/` 缺失是明确的未完成证据。

## 6. 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M0 | Current workspace/dependency reconstruction + internal crate/CI decision lock | `frameworks_01_m0_structure_2151_refs_77_edges_reverse_18_facade_inbound_38_handoff_open_timings_pending` | 2026-07-13 | 读取 current root/runtime manifests、CI 和两个架构权威计划；确认 6 个根 workspace members、Runtime 单 package、`zircon_runtime/crates/` absent。复用 production-only 2,151 refs / 77 edges JSON，锁定 `zr_*` 内部层级、依赖方向、公开 facade 与 CI 影响；完整 layer classification 捕获 reverse-layer 18 refs 与 facade-inbound 38 refs，并向 Frameworks05 发布 open failure handoff。Cargo timings 因受管单池忙而明确 pending，不声明 M0 完成。 |
