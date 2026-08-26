---
handoff_kind: failure
status: open
created_at: 2026-08-23
summary_slug: particles-wgpu29-neutral-prepare-compile-regression
origin_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
fixing_plan: docs/plans/optimize/zircon_plugins/09-first-party-particle-vfx-source-runtime-editor-dist-catalog-simulation-render-product-integration-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/05
fixing_child_dir: docs/plans/optimize/zircon_plugins/09
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/particles/runtime/src/render/gpu/neutral_buffers.rs
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/neutral_graph_buffers.rs
  - zircon_runtime/src/graphics/runtime_prepare_collector.rs
tests:
  - cargo +1.94.1 check -p zircon_plugin_particles_runtime --locked --jobs 1
  - cargo +1.94.1 test -p zircon_plugin_particles_runtime --locked neutral --jobs 1 -- --test-threads=1
  - cargo +1.94.1 test -p zircon_app --lib --features first-party-runtime-plugins --locked runtime_profile_bootstrap --jobs 1 -- --test-threads=1
---

# Plugins09: Particles wgpu 29 neutral prepare compile regression

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 来源执行切片：ZUI importer provider linkage 的 App product compile
- 修复责任计划：`docs/plans/optimize/zircon_plugins/09-first-party-particle-vfx-source-runtime-editor-dist-catalog-simulation-render-product-integration-review.md`
- 交接原因：Frameworks05 的 exact App compile 已越过 Runtime、importer/provider 与 sound runtime，随后只被
  Particles runtime 的三条 current-source Rust error 阻断。两个失败文件均不在 Frameworks05 immutable scope，
  且 ownership matrix 未发现可执行 owner 或 live lease。

## 失败现象与复现证据

Managed job `9488aee1514f4b51a6e13b86b09e0e35` / run
`4e9d8f3b905f4367bbdea33fa3201be5` 在 D 盘 retained pool 执行：

`cargo +1.94.1 test -p zircon_app --lib --features first-party-runtime-plugins --locked runtime_profile_bootstrap --jobs 1 -- --test-threads=1`

Cargo 在 2026-08-23 11:15:54 以 exit 101 终止，精确错误为：

- `zircon_plugins/particles/runtime/src/render/gpu/neutral_buffers.rs:262` E0599：wgpu 29.0.3
  `BufferViewMut` 没有 `fill`。
- `zircon_plugins/particles/runtime/src/render/runtime_prepare.rs:113` E0502：通过
  `context.frame_extract()` 持有对整个 context 的不可变借用后，再可变借用 `context.encoder`。
- 同文件 `:115` E0502：上述不可变借用仍存活时，再把整个 context 可变传给 external-buffer registration。

Coordinator ownership matrix request `dd44fe631f61461095ccb0eb76ca3b13` 记录：

- `neutral_buffers.rs` current hash
  `9450994ff0afeb90b94e4d98eea5ef7eaf9703de6c98a5c4bcfd7d78c7c24659`，状态
  `attribution_missing`。
- `runtime_prepare.rs` current hash
  `4c437e033515c254126e055576f13103fc762cccf250230efa8cf0ed3402b5d1`，旧 owner 已 archived，
  attribution hash/baseline stale，且无 live lease。

## 最低共享层根因

- 本地 wgpu 29.0.3 源码明确说明 `BufferViewMut` 不能实现 `DerefMut`，只公开 `len`、`slice` 与
  `copy_from_slice` 写接口。因此 `.fill(0)` 不是 feature 或 import 缺失，而是 API 迁移错误。
- Runtime 已有同版本正确实现：`neutral_graph_buffers.rs::zeroed_buffer` 取得 mapped view，以等长 zero byte
  slice 调用 `copy_from_slice`，显式 drop view 后再 unmap。Particles 应复用同一 write-only 语义，不应降级
  wgpu 或引入 unsafe 可变 slice。
- `RuntimePrepareCollectorContext` 的 `device`、`queue`、`encoder` 与 `frame_extract` 是独立 public fields，
  但 `frame_extract()` accessor 的 `&self` 借用在当前调用形态中覆盖整个 context。neutral collector 应直接
  使用字段级借用，并在可变注册 context 前物化 bounded readback outputs，使 immutable frame borrow 结束；
  不需要克隆整个 `RenderParticleGpuFrameExtract`，也不需要复制 GPU buffer owner。
- 该回归由当前 Particles neutral owner 切片暴露，修复不改变 Plugins09/Runtime26 已记录的 canonical
  simulation/render architecture，不应借机扩展算法或 capability scope。

## 架构修复验收

- mapped neutral buffer 使用 wgpu 29 write-only API，view 在 `buffer.unmap()` 前释放；禁止 unsafe、版本回退
  或每帧 staging/file IO。初始化仅发生在 buffer create/device generation 路径。
- neutral prepare 采用字段级 borrow，bounded outputs 在可变 external binding registration 前完成；禁止为了
  绕过 borrow checker 深克隆 frame/context。
- 新增或更新 focused source/unit guard，先精确覆盖 mapped initialization ordering 与 neutral prepare 的
  bounded output/registration borrow shape。
- managed `zircon_plugin_particles_runtime` check 和 neutral focused tests GREEN；随后 Frameworks05 的 exact
  App command 必须越过 Particles 并实际执行 `runtime_profile_bootstrap` tests。
- scoped `rustfmt --check`、`git diff --check` 和独立 source review 通过后，才可回传 fixed。

## 禁止临时方案

- 不得关闭 Particles first-party feature、修改 App test feature 或从 product closure 排除该 crate。
- 不得降级 wgpu、给 `BufferViewMut` 增 unsafe facade，或用 test-only cfg 隐藏生产编译错误。
- 不得把 Plugins09 的长期 simulation/render redesign 混入本次两文件 compile unblock。

## 修复结果与回传

- 当前状态保持 `open`：Plugins09 尚需完成 wgpu 29 mapped-write 修复、prepare borrow 收敛与受管验证。
- 修复方只有在上述架构验收全部通过后，才可将本记录转为 `fixed` 并向 Frameworks05 来源计划回传终态证据。
