# Plugins09 wgpu 29 Neutral Identity/Prepare Repair Record

- Date: 2026-08-23
- Owner: `plugins09-particles-neutral-identity-runtime-r3-20260823`
- Source plan: `docs/plans/optimize/zircon_plugins/09-first-party-particle-vfx-source-runtime-editor-dist-catalog-simulation-render-product-integration-review.md`
- Origin failure: `docs/plans/zircon_runtime/frameworks/05/failure-2026-08-23-particles-wgpu29-neutral-prepare-compile-regression.md`
- Status: implementation, source review, focused tests, and production package check complete;
  product upward gate blocked by an independent Plugins07 glTF importer failure
- Ownership transfer: r2 dirty blobs were transferred by coordinator fingerprint
  `55c671ffc09915ec4a9dfa92503870af81307ebded8bdb29c2fb6ee037ce4184`

## Current-Source Problem

Frameworks05 的 App product gate 越过 runtime profile、ZUI importer provider 与 sound runtime 后，被
Particles neutral runtime 的 wgpu 29 API/borrow 编译错误阻断。首轮 source repair 修正了 mapped view
和 prepare context 的借用，但更完整的 call-graph 与 wgpu-core 复核证明，原 neutral 设计本身还在为
没有命令消费者的 fallback identity 分配、归零并逐帧上传 capacity-scaled payload。只修编译错误会保留
错误的 O(particles + emitters) 算法和无效 CPU/GPU 流量，因此本记录执行 hard cut，不保留旧行为。

## Architecture And Algorithm Review

### Consumer graph

- `collect_neutral_particle_gpu_runtime_prepare` 先 `owner.deactivate()`，因此真实 aggregate backend 的
  `aggregate_executed` 必为 false。
- spawn/update、compact 与 indirect executors 只校验 render-graph resource metadata；没有编码 compute
  dispatch 或读写 neutral buffer。`emit_particle_gpu_readback` 从 frame extract 投影结果。
- transparent executor 在真实 aggregate backend 未执行时不会建立 GPU particle bind group，而是回落到
  CPU billboard path。
- neutral buffer 的实际职责仅是为 render graph 提供七个类型化、device-lifetime 稳定的 external resource
  identity。间接参数 identity 需 16 bytes，其余 identity 只需 wgpu 的 4-byte buffer 最小单位。

因此旧的 particle/emitter power-of-two capacity、frame shadow、scratch vectors、queue writes 和 encoder
copies 均没有功能消费者。新实现保留七个不同 resource identity，但把它们固定为六个 4-byte buffer 与
一个 16-byte indirect buffer；只在 device 变化或 bundle 缺失时创建，不再接收 queue/encoder。

### Wgpu initialization path

本地 wgpu-core 29.0.3 `device/resource.rs` 证明：对没有 `MAP_WRITE` 的 buffer 使用
`mapped_at_creation: true` 会创建等长 staging allocation，先 `write_zeros`，随后在 `unmap` 时复制到
GPU buffer。wgpu init tracker 又保证未映射 buffer 在首次读取前完成零初始化，因此 identity buffer 可用
`mapped_at_creation: false`，让 backend 按需初始化；neutral 路径通常没有读取，因而也不强制产生清零
command。初版 `.slice(..).fill(0)` 虽去除了额外 `Vec`，仍保留了整容量 host staging 和第二次传输，现已
完全删除。

### Reference-engine alignment

Unreal Niagara 的无资源 fallback 使用固定的一元素 dummy shared mesh buffer，并以零 slot 描述空状态；
GPU scratch 清理走 RHI/UAV command，而不是为逻辑占位资源建立 capacity-scaled CPU mapped staging。
本次 fixed identity bundle 采用同一原则：fallback 只保存资源身份，真实 simulation payload 仍由
`ParticleGpuBackend` 的按资产容量 buffer 独占管理。

## Quantified Result

在最大粒子容量 `1,048,576` 下，旧 neutral bundle 的精确逻辑分配为 `139,493,440 bytes`
（`133.031311 MiB`）；两个 particle buffer 单个峰值均为 `67,108,864 bytes`（`64 MiB`）。
`mapped_at_creation` 路径还产生同规模 host staging，原始等长 zero `Vec` 版本则再增加一轮
`139,493,440 bytes` source allocation/write traffic。

新 bundle 固定为 `40 bytes`：

- 六个 storage identities：`6 * 4 = 24 bytes`；
- 一个 indirect identity：`4 * u32 = 16 bytes`；
- 总计：`40 bytes`，相对旧逻辑分配减少 `139,493,400 bytes`，即 `99.999971%`；
- host staging、source zero allocation、capacity growth、queue upload 与 encoder copy 全部归零；
- 空间复杂度从 O(particles + emitters) 收敛为 O(1)。

旧 changed-frame hot path 最大上传 `4,210,720 bytes/frame`（`4.015655 MiB/frame`）；按 60 FPS 为
`252,643,200 bytes/s`（`240.939331 MiB/s`）无效 CPU write/upload 带宽。新路径为 `0 bytes/frame`。
这些是源码/容量模型的精确上界，不冒充硬件功耗测量；功耗与 wall-time 对比必须在具备真实 GPU 的
runtime benchmark milestone 中单独采样。

prepare 侧复制 `context.frame_extract` 字段引用，并在 external binding registration 前物化 bounded
outputs，使 frame 不可变借用在 context 可变借用前结束；不克隆 frame、context 或 GPU owner。

## Completed Evidence

- production-only RED probe 曾同时确认 host-mapped creation、per-frame queue upload、encoder copy、shadow
  payload 与 capacity growth 存在；hard cut 后对应 source guards 转为 GREEN 条件。
- standalone manifest 的 normal dependency 显式启用 `zircon_runtime/graphics`；metadata/tree 证明只补齐
  production source 实际使用的 graphics/render-graph surface，没有依赖 App feature unification。
- 排除 `src/tests` 后，production source 有 10 个 direct graphics/render-graph references，分布于 8 个
  文件；test source 另有 14 个 references/4 个文件，crate 合计 24/12。没有 UI、script、sound、
  navigation 或 animation feature 的直接需要。
- r2 managed baseline job `89b1e4f36858441b8b2944c0f9b2efc6` / run
  `7591615a3e9f4190bf3fbdd14cf10127` 执行 exact neutral test，`12 passed / 0 failed / 32 filtered`，
  exit `0`，耗时 `36m39s`；原 24 条 graphics/render_graph E0432/E0433 和首轮 E0599/E0502 均消失。
  该结果只证明 manifest 与 pre-hard-cut repair baseline，不代替当前源码的 post-hard-cut validation。
- `runtime_owner` 已去除 neutral queue/encoder 参数；`runtime_prepare` 保留 field-level borrow 和
  output-before-registration 顺序。
- post-hard-cut managed focused job `9dbd865e3d9c44ac8312d3d90bc6d05e` / run
  `19d52a2ce89a44dbbf0803e9157f5beb` 执行 exact neutral test，`10 passed / 0 failed / 32 filtered`，
  exit `0`，测试执行 `0.14s`，完整增量构建耗时 `31m32s`。
- managed production check job `a621dd14ac5d48ce9283d197013ad0e0` / run
  `4eba9d7d09c2494582f892cfa8fdf742` 执行 standalone package check，exit `0`，耗时 `16m28s`；
  Particles production crate 仅报告 1 条既有 dead-code warning，原 E0599/E0502/E0432/E0433 均未复现。
- independent source review 对 exact five 报告 **Critical 0 / Important 0 / Moderate 0 / Minor 0**；确认
  40-byte distinct resource identity、wgpu 4/16-byte usage、field-level borrow、graphics-only dependency
  和旧 staging/upload/copy/capacity 路径删除均成立。scoped rustfmt、static guards、manifest tree 与
  `git diff --check` 均 GREEN。
- 按 `#[cfg(test)]` 前 production source 统计，owner 从 343 行/15 个函数收敛为 142 行/5 个函数；
  `Vec` token 5 -> 0、queue write
  3 -> 0、encoder copy 2 -> 0、mapped creation 1 -> 0、`next_power_of_two` 2 -> 0。
- Frameworks05 exact App upward job `a523910f7a204dc28d3e15461103e9f4` / run
  `b575c7a8217743adbe0e5b124c1cb9b7` 已成功越过 Runtime、ZUI importer provider、Particles 和 Sound，
  随后在 foreign `zircon_plugin_gltf_importer_runtime` 以 E0599 终止，exit `101`，耗时 `40m07s`。
  新阻断已记录为同目录 failure handoff；该结果不回退本切片 package acceptance，也不冒充 App gate GREEN。

## Pending Acceptance

- [x] standalone production dependency feature hard cut；
- [x] neutral payload owner 从 capacity-scaled fake data 收敛为 fixed resource identities；
- [x] prepare borrow/API 收敛，旧 mapped/upload/copy/shadow/capacity 路径删除；
- [x] exact source formatting、static guards 与独立 source review；
- [x] managed post-hard-cut
  `cargo +1.94.1 test -p zircon_plugin_particles_runtime --locked neutral --jobs 1 -- --test-threads=1`；
- [x] managed production `cargo +1.94.1 check -p zircon_plugin_particles_runtime --locked --jobs 1`；
- [ ] managed Frameworks05 App `runtime_profile_bootstrap` upward gate（Particles 已越过，当前阻断为
  Plugins07 glTF empty-texture source contract）；
- [ ] origin failure fixed return、scoped validation ticket、integration candidate、milestone commit 与企微量化回执。

当前声明 Plugins09 focused/package Rust gates 完成，但不声明 App gate、failure closeout 或完整 milestone 完成。Runtime
`neutral_graph_buffers.rs::zeroed_buffer` 仍保留另一套 host-mapped/等长 `Vec` 逻辑；该 foreign Render
owner 路径必须用独立 failure handoff 收口，不能混入 Plugins09 ownership。
