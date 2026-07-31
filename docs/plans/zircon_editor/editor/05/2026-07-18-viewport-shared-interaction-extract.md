# Editor05 viewport shared interaction extract

## 目标与状态

- 状态：源码切片完成；受管编译、独立复审与空间 broad phase 未完成。
- 目标：让 render snapshot 与 pointer routing 共享同一 world/selection/settings/camera/viewport 代际的 handle、scene gizmo 与 runtime render-mesh extract，硬切 pointer 私有 scene key、重复 gizmo builder 和 `Scene::nodes()` renderable 扫描。
- failure 生命周期：继续保持 [`failure-2026-07-18-viewport-pointer-candidate-regeneration.md`](failure-2026-07-18-viewport-pointer-candidate-regeneration.md) 为 open；本切片不把 camera/layer 过滤的 runtime mesh extract 伪称为空间/BVH 查询。

## 架构落地

- `SceneViewportController` 新增 `ViewportInteractionExtractCache`；cache key 包含 `world_generation`、选中、完整 viewport settings、camera snapshot 与 viewport size，没有手工失效旁路。
- render 先构造 runtime viewport packet，再用 `packet.scene.meshes` 给同代际 cache 播种；pointer 复用同一 `Arc<ViewportInteractionExtract>`，仅在新代际 pointer 先于 render 到达时补做一次 runtime packet extract。
- handle/gizmo 只在 cache miss 构建一次；pointer layout 持有三组 `Arc` slice，router 用 `Arc::ptr_eq` 在精度投影、候选分配和 retained surface rebuild 前 early-out。
- renderable candidates 改由排序后的 `RenderMeshSnapshot` 构造，并把同 entity 的多 primitive 合并为一个 coarse candidate；旧 `Scene::nodes()` + `active_in_hierarchy` 全扫删除。
- 旧 `viewport_pointer_scene_key.rs` 与 pointer-only `scene_gizmo_candidates.rs` 硬删除，不保留兼容 API。
- runtime `RenderOverlayExtract` 仍要求 owned `Vec`，render boundary 会从共享 slice 复制 DTO；事实构建与场景扫描不重复。后续若 runtime overlay 合同改为共享所有权，应由 runtime/render owner 单独会签。

## TDD 与证据

- RED：`python -m unittest tools.tests.test_editor05_viewport_interaction_extract_contract -v` 初次执行 4/4 failed，分别命中 controller 无 cache、layout 仍为 Vec、renderable 仍扫 scene、两个旧文件仍存在。
- GREEN：同命令 4/4 passed。
- Rust 行为测试已加入：稳定 key `Arc::ptr_eq` 且 handle builder 仅调用一次；world generation 变化发布新 extract；render-seeded 与 pointer-resolved extract 指针相同；runtime mesh 多 primitive 按 owner 折叠。
- `rustfmt --edition 2021` 已覆盖本切片 Rust 文件；未运行 Cargo。Coordinator01 的 full compile-input immutable snapshot failure 仍 open，当前共享树盲跑不能形成可审计 current-source 证据。
- 结构守卫明确禁止重建 `ViewportPointerSceneKey`、pointer-only gizmo builder、`Scene::nodes()` renderable 扫描。

## 未完成项

- runtime mesh packet 只完成 active/camera-layer/LOD extract，并非 cursor-query bounded visible/BVH set；仍需 runtime spatial index、pick-id broad phase或等价方案。
- 仍需 1/1k/10k nodes 深测，证明 candidate visits 由 query hits 主导；仍需 1k stable moves 与 changed-move scan/matrix/trig/allocation/CPU p95 受管证据。
- 仍需受管 focused/broad Cargo、独立 review、failure fixed-return 与 coordinator managed commit。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-18 | shared interaction extract + runtime mesh candidate source | 源码完成 / failure 保持 open | controller generation cache、render/pointer shared `Arc`、handle/gizmo single build、runtime `RenderMeshSnapshot` candidate source、primitive owner dedupe、旧 scene key/gizmo adapter/scene node scan hardcut已完成；Python TDD RED 4/4→GREEN 4/4，Rust行为测试已落盘并完成rustfmt。Cargo/review/p95/BVH尚未完成，不声明fixed或全绿。 |
