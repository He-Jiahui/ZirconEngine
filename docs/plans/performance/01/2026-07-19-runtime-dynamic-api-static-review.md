---
related_code:
  - zircon_runtime/src/dynamic_api
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/pipelined_rendering.rs
  - dev/godot/core/templates/command_queue_mt.h
  - dev/godot/servers/rendering/rendering_server_default.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderingThread.h
tests:
  - zircon_runtime/src/dynamic_api/tests
  - zircon_runtime/src/dynamic_api/session/tests
  - zircon_runtime/src/dynamic_api/shader_prewarm/tests.rs
  - current-source Windows Cargo and F2 product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime dynamic API逐文件性能静态审查（2026-07-19）

## 范围与覆盖

`zircon_runtime/src/dynamic_api/**`当前源 **64/64** 个Rust文件、**10,511** 行、**103** 条测试已逐文件阅读，其中生产37、测试27。范围包含ABI表、session registry/lifecycle、tick/input/host/plugin/operation、extract/cache/diagnostics、capture/present、UI/menu/HUD、project启动、shader prewarm及全部聚焦测试。registry拆分与V3 wake/frame-demand改造含其他会话当前改动，本轮保留并只在已取得租约的独立切片上修改。

## 关键瓶颈

- **PERF-MVP-429 / Runtime10**：`error_status`对每个动态错误执行`Box::leak`，而`ZrStatus`只有borrowed diagnostics slice，无free或明确有效期；错误/恶意输入风暴永久增长RSS。
- **PERF-MVP-430 / Runtime10**：global registry锁只用于lookup并clone slot，方向正确；但slot session mutex仍跨完整tick、GPU capture/present、profile snapshot+JSON、plugin drain+JSON和operation poll，同session所有动作被最慢工作串行化。
- **PERF-MVP-431 / Runtime07**：extract cache hit返回完整deep clone，miss又clone进cache；随后`record_frame_extract_stats`每capture/present全扫宽extract估算bytes。现有测试只证明cache hit/rebuild诊断数值，不证明clone bytes或统计访问为0。
- **PERF-MVP-432 / Runtime10/06/Plugins01**：plugin deliveries无配额全drain，逐payload复制event/schema descriptor，空/非空批次均JSON编码且session锁覆盖该工作。
- **PERF-MVP-433 / Runtime09**：`current_ui_extract`每capture/present全扫World，稳定menu/HUD仍重建command/style/text所有权；无component/viewport generation owner。
- host request、input/gamepad分别复用PERF-MVP-425/426；shader prewarm按variant复制WGSL复用357；extract双owner复用342，不创建重复根因。

## 本轮直接止损

1. `session/ffi.rs`在ABI/viewport验证后、`session.capture_frame`前拒绝null `out_frame`，不再先渲染/拥有RGBA后由`write_frame`报错并遗失payload。`tests/viewport.rs`新增顺序源码守卫及ABI状态/消息断言。
2. `session/hud.rs`删除每条HUD文本的临时token Vec和5次window扫描，改为一次borrowed sliding token流；保留HP/XP/weapon判定，并加零collect源码守卫。

两项均执行RED→GREEN源码守卫、`rustfmt --edition 2021`和`git diff --check`。受管Windows validator仍在调用Cargo前因`ConvertFrom-Json`读取非JSON首字符失败，所以未把Cargo标为通过。

## 参考引擎约束

Bevy pipelined renderer用容量1的双向channel把Nth render与N+1 simulation重叠；Godot把render调用进入`CommandQueueMT`并把同步操作显式区分为`push_and_sync`，还警告逐帧同步显著影响性能；Unreal用render command pipe/`ENQUEUE_RENDER_COMMAND`，把`FlushRenderingCommands`作为显式阻塞点。Zircon不复制其API，但采用共同原则：admission/publish短锁、昂贵render/encode在有界owner lane、同步点显式且可测。

## 动态验收缺口

需要1/1k/1M error、slow-GPU/JSON并发、1/1k/100k mesh+64MiB extract、1/1k/10k plugin burst及stable UI 240Hz capture夹具，记录RSS、clone bytes、payload visits、session lock wait/hold、queue age/depth/drop、extract/UI builds与p95。F2需补产品frame diagnostics、WPR/Tracy/Chrome scope与GPU timestamp；本机`renderdoccmd.exe`仍不可用且现有RDC由其他计划租约持有。Cargo、规模counter、产品trace和RenderDoc证据齐全前，本模块留在`pending.md`，不进入`review.md`。
