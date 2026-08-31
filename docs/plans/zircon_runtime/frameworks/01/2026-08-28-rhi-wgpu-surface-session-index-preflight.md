# Frameworks01 RHI/WGPU Surface Session Index Preflight

## 状态

- 日期：2026-08-28
- 范围：`zr_rhi_wgpu` deterministic/production surface session、frame lease、submission
  attachment 与 terminalization owner
- 当前状态：`architecture_review_complete_correctness_repair_and_focused_behavior_validation_complete_performance_profile_pending`
- 优化授权：`not_started`。本记录只确定待测瓶颈与测量方法；pre-profile 数据完成前不得实现
  session/frame 索引优化，也不得宣称耗时、功耗或复杂度瓶颈已经消失。

## MVP 与计划边界

Frameworks01 M1 的 `zr_contracts` workspace wiring 仍受 Shader06 executable attribution
阻塞，因此本切片只处理当前 Session immutable scope 已包含的 WGPU backend 基础设施。它不改变
`zr_rhi <- zr_rhi_wgpu` crate DAG，不恢复旧 monolith/backend facade，也不修改 Runtime90 所有的
`zr_rhi/src/surface.rs` allocator。

本轮先修的是 correctness，不是性能优化。`SurfaceFrameLease::new` 是公开构造器，而此前
deterministic 与 production 的 present/discard 只消费 `frame.frame()`，没有在终结真实 frame 前
核对 session、target、default view 与 descriptor。伪造 lease 因而可以用一个真实 frame id 消费
不属于它的 acquired frame，违反 PFO-4c 的 owner/generation/lease fail-closed 约束。

当前 source repair 让两套 backend owner 保存并核对完整 lease identity；production discard 在取消
Accepted submission 之前完成校验。新增 deterministic 行为回归分别伪造 session、target、default
view 和 descriptor，并要求真实 frame 在四次拒绝后仍可正常 discard。该修复维持 `O(1)` lookup，
没有以额外全表扫描换取安全性。

二次源码重审确认 surface reconfigure、discard 和 destroy 原先逐 ticket 执行
`status() -> cancel()`。并发 `flush()` 可以在两次调用之间把 Accepted ticket 推进到 Submitted，
导致 teardown 非确定性失败。submission owner 已提供面向 abandoned frame 的
`settle_abandoned_submissions` 单锁批量原语，因此 production surface lifecycle 已统一切到该原语，
并在批量结果返回后一次性投影 Cancelled diagnostics。旧逐 ticket 路径已硬删除，公开 ABI、
surface/registry owner 与 Submitted ticket 语义不变。

## 当前模块重审

### Owner 与调用顺序

1. `WgpuRenderDevice` 是 admission、submission、diagnostics、surface 和 registry 的唯一组合 owner。
2. `WgpuSurfaceService` 持有 native `wgpu::Surface`、acquired `SurfaceTexture`、session/frame table 与
   bounded terminal history。
3. `WgpuResourceRegistry` 持有 surface target/default view 的 generational resource identity、last-use
   和 submission set；present/discard 后 public handles 立即 stale，native references 继续按 ticket
   retirement。
4. 产品 native surface target 通过 RAII 保留完整 lease；未 present 的 target 在 Drop 中走 neutral
   discard，不向 Editor 暴露 WGPU queue 或 raw surface owner。
5. 当前锁顺序在 surface 路径内统一为 `surfaces -> registry -> submission`；submission 的
   queue/state guard 在获取 diagnostics 前释放。全 production device 复扫未发现 `registry -> surfaces`、
   `submission -> registry/surfaces` 或 `diagnostics -> registry/surfaces` 的反向持锁边，因此本次批量结算
   没有形成锁环。submission cancel 与 diagnostics terminalization 的持锁时间仍需要 profile，本轮没有
   动态证据支持重排。

### 已确认的复杂度

| 路径 | 当前数据结构 | 当前复杂度 | 结论 |
| --- | --- | --- | --- |
| frame-id lookup / terminal history | `HashMap` + bounded history | average `O(1)` | 保持 |
| present/discard lease identity | active frame direct lookup | average `O(1)` | correctness repair 后保持 |
| acquire: session 是否已有 frame | `frames.values().any(...)` | `O(active_frames)` | 待测候选 |
| session submission tickets | scan all frames, then ticket set | `O(active_frames + tickets)` | 待测候选 |
| session destroy/reconfigure | scan all frames | `O(active_frames)` | 待测候选 |
| deterministic submission attachment | every frame scans every command | `O(active_frames * commands)` | 高优先级待测候选 |

因此，既有 PFO-4c 记录中的“acquire 为均摊 `O(1)`”不是当前源码事实。典型单窗口 Editor
未必受该斜率主导，不能仅凭复杂度表把它升级为当前产品主瓶颈；multi-viewport、插件预览窗口和
deterministic contract 压测才可能放大该成本。

## 参考引擎约束

### Unreal Engine（主参考）

- `DynamicRHI.h` 把 viewport create、resize 与 end-drawing/present 都定义在 `FDynamicRHI`，而
  `FRHIViewport` 是 RHI resource。Zircon 应继续由 RHI/backend device owner 管理 session/frame，
  不能把索引或 native owner 上移到 Editor。
- `D3D12Viewport.cpp` 的 resize 先阻止重入、flush/wait、清理 dangling state、释放所有旧
  backbuffer，再 resize/recreate；end drawing 进入 RHI command lane，并明确 GPU work 应统一经
  submission owner。Zircon 的 session-local active-frame slot 必须服从同样的 ordered teardown，
  不能成为绕过 registry retirement 的第二 owner。

### Bevy（辅助参考）

- `ExtractedWindow` 对每个窗口保存一个 `Option<SurfaceTexture>`，`present` 通过 `take()` 消费
  acquired texture；这与“每 session 至多一个 active frame slot”一致。
- `RenderDevice::configure_surface` 明确旧 `SurfaceTexture` 仍存活时不得 reconfigure。Zircon 的
  session slot 若落地，必须在 reconfigure 前通过统一 discard/retirement 清空，而不是只覆盖 id。

这些参考只支持 owner 与生命周期方向，不证明 Zircon 应复制 Unreal 的阻塞 wait 或 Bevy 的错误
分类，也不代替本仓库的 pre-profile。

## Pre-profile 方案

### Harness

在取得新的受管 Cargo/benchmark grant 后，新增 E 盘 release-profile harness；产物与原始数据只能
进入 `E:\Git\ZirconEngine\.codex\targets` 或批准的 D/F 验证池，不得写 C 盘。harness 先测当前
实现，再在相同 immutable source、toolchain、CPU affinity 和 power mode 下测候选实现。

1. session cardinality：`1, 8, 64, 256, 1024`。
2. command count：`1, 16, 128, 1024`，包含 unrelated command、单 surface target 和多 target。
3. 每组至少 3 次独立进程、warm-up 后 10,000 次操作；记录 p50/p95/p99、CPU cycles、instructions、
   LLC misses 与 allocation count。
4. 加入结构计数器：`active_frame_scan_visits`、`command_surface_pair_visits`、ticket visits、terminal
   operations。计数器必须仅在 profile feature/harness 内存在，不污染 shipping hot path。
5. 产品层运行 300-frame 单窗口与 32-session synthetic multi-viewport；用 WPR/WPA CPU sampling、
   context switch 和 power trace，对齐现有 submission/present latency telemetry。GPU 行为用 PIX 或
   RenderDoc 复核没有新增 submission、wait、copy 或 backbuffer lifetime 变化。

### 候选设计（未实施）

若数据确认扫描占主导，优先把 `Option<SurfaceFrameId>` 放进每个 session owner，并在 acquire、
present、discard、reconfigure、destroy 和 device-loss terminalization 的同一 transaction 中更新。
deterministic submission attachment另维护 `TextureHandle -> SurfaceFrameId` 反向索引，使每条 command
只解析一次 surface target，而不是对每个 active frame 重扫整个 command list。

不得同时保留新索引与旧全表扫描作为兼容路径。debug/test invariant 必须证明 session slot、frame
table、target reverse index 和 registry surface-owned sets 同步；任一 mutation 失败时 fail closed，
不得留下可被第二 lease 消费的半提交 frame。

## 实现门与验收门

只有同时满足以下条件才开始优化实现：

1. managed focused correctness tests GREEN，包括 forged lease、double terminal、reconfigure/destroy、
   accepted submission cancellation 与 device-loss teardown；
2. pre-profile 提供每组原始样本、环境指纹和结构计数器，证明扫描而非 driver acquire/present 或锁
   等待主导 CPU 成本；
3. 当前 Session 或后继 Session 合法拥有全部被修改的数据结构、测试与 profile harness；
4. 不与 Runtime90 surface allocator 或产品 PFO-4c owner 形成双重归属。

优化后的算法门：

- acquire/session teardown 的 scan visits 不随其它 session 数增长；`64 -> 256` session 的中位耗时
  斜率不得接近 4 倍，结构计数必须保持常数级；
- deterministic submission attachment 从 `frames * commands` 收敛为 `commands + matched_targets`；
- 1-session 稳定路径 p95 不得回退超过 3%，内存增量必须按 session/index entry 量化；
- 300-frame 产品 submission count、present outcome、PNG/RenderDoc 输出与 device-loss/reconfigure
  语义不变；
- WPR power 数据至少 3 次重复且报告均值/离散度。没有动态数据不得声称功耗接近其它引擎、瓶颈
  消失或算法达到最优规模。

## 当前证据

- source correctness repair：forged lease identity 与 teardown batch settlement 均已实现；
- teardown source regression：RED 已证明旧 `status -> cancel` 路径存在，GREEN 静态约束已证明只保留
  单锁批量结算；
- managed focused correctness test：旧 `cargo.acquire` request
  `01cf7cdd5ed5442fbdab4c807113fe19` 已终态失败，错误为 `cargo_reuse_target_mismatch`，不是代码失败且
  不能计作 compile/test ticket；后续 successor acquire request
  `406abfe9b2c143e89fcfbc89d549f3b4` 只完成了 lease，最终被协调器标记为 `orphaned`，没有启动 Cargo，
  同样不能计作票据。随后复用同一主 target pool 的受管 job
  `51c4981906f64a7888dfef5002e60ce1`，由 Frameworks01 session
  `frameworks01-shader-invocation-hard-cut-r12-1b2684b4-20260825` 通过 run
  `54250374869c44c89762d4d11fbfd535` 执行：
  `cargo test -p zr_rhi_wgpu --locked --verbose --lib surface_teardown_settles_tickets_under_one_submission_lock --target-dir D:\\cargo-targets\\zircon-engine\\pool\\f9fef644bf8e441a49ad1c139495499657f126cd246ffca80d13868db535561d`。
  该 run 于 2026-08-30 07:17:26 UTC 以 exit code `0` 终态完成，报告
  `1 passed; 0 failed; 389 filtered out`，测试耗时 `0.00s`，编译耗时 `5m25s`；job 已自动 release。
- RHI surface 三轮独立只读复核：首轮对 source hash
  `b3164ac0ddd8fe14a1dcc674a4fd64da6d00dee106735d297f62fa938ed093f8` 与 submission owner hash
  `709a93865da381151d8a9b03dca8e33ff3a7dce918d1c77404e43c6190961850` 的锁序、批输入原子性、
  TOCTOU、diagnostic terminalization 与 reconfigure/discard/destroy 结论为 `C0/I1/M0`；I1 是缺少
  真实行为与竞争覆盖，不是生产实现缺陷。第二轮在首版行为测试通过后仍给出 `C0/I1/M1 Not Ready`：
  单 ticket 竞争不能捕获批内部分取消，barrier 不能确定覆盖两种先后顺序，且 adapter/device 初始化
  失败会静默 return。最终 source hash
  `9c8bf87e5c2be96ff80f5e83dee324a3ff5988e365da9d3c0ed9d6436dd5719a` 把竞争批扩为 4 个 committed
  pending tickets，任何 `1..3/4` partial flush 立即失败；同一测试确定执行 settle-first 和 flush-first，
  再执行 16 轮并发压力，并接受 callback 合法推进到 Submitted/Completed。设备初始化改为 fail-fast，
  不再允许无 adapter/device 环境报告 GREEN。第三轮复核结论为 `C0/I0/M0 Ready`，生产锁序、原子性
  与 diagnostic exactly-once 均无回归。
- 最终 I1/M1 行为回归证据：Frameworks01 只在已拥有的 `surface_lifecycle.rs` test-only 模块保留
  mixed `[valid Accepted, unknown]` 零副作用、reserved + committed-pending + duplicate 单次取消/context
  释放、submission-qualified diagnostic 恰好一次 Cancelled delivery，以及上述 4-ticket 原子竞争；
  没有改写已归档 RHI90 owner 的 `submission.rs` 或 `production/tests.rs`。受管 job
  `5cf107ed3fd4438183c0040f4945962d`、run `cb6d67c9e1944979b8705f1f9b07f691` 在同一 D 盘兼容
  target pool 执行 `cargo test -p zr_rhi_wgpu --locked --verbose --lib surface_teardown_ ... --
  --test-threads=1`，于 2026-08-30 08:12:55 UTC 以 exit code `0` 完成：
  `5 passed; 0 failed; 389 filtered out`，测试耗时 `14.75s`，编译耗时 `1m24s`；job 随后 release，
  live process 为 0。上一轮 job `9f319fab7af84771840e773d5370f478` 的 5/5 只保留为历史证据，
  不冒充最终 source hash 的验证票。
- resource/WAL independent review：`engine.rs`、`journal/{intent,mod}.rs` 与 conditional-write guard 的
  当前 shared-worktree 指纹通过 `C0/I0/M0` 静态复核；owner lock 覆盖 pending scan、create-only WAL、
  stage/commit/cleanup，transaction id 在 plan/persist 间同源，未发现应由本 RHI 切片修改的底层缺陷；
- Frameworks01/02 typed-error owner guards：2/2 GREEN，耗时 43.422 秒；
- `rustfmt --check`：GREEN；
- scoped `git diff --check`：GREEN（仅既有 LF/CRLF 提示）；
- 文件预算：`device.rs 769`、`device/surfaces.rs 483`、`production/surface.rs 586`、
  `production/device/surface_lifecycle.rs 434`，均低于 800 行；本轮新增行均位于 `#[cfg(test)]`，
  production owner 与 shipping hot path 未扩张；
- 最终 owned snapshot：source/doc 写入前 exact-path claim request
  `35b3bf5f80c745a3add33c7c7e9094d`；focused run 后 source hash 仍为
  `9c8bf87e5c2be96ff80f5e83dee324a3ff5988e365da9d3c0ed9d6436dd5719a`；本文更新后以协调器
  `baseline attribute` 和 ownership matrix 返回的最终 hash 作为归属依据；
- performance pre-profile：未运行；
- 受管 benchmark 入口审计（2026-08-30）：Coordinator 的 `native-plugin-benchmark`
  grant 只允许 allow-listed `zircon_runtime` native-plugin 基准，并固定生成
  `cargo test -p zircon_runtime ... --ignored` 的 release/profiling 命令；当前 allow-list
  没有 `zr_rhi_wgpu` surface/session benchmark，也没有可绑定本切片的 RHI benchmark
  harness。因而不能把该入口的任意结果当作 surface/session 的同源样本，当前没有发起
  benchmark grant，也没有把静态复杂度推断升级为性能结论。后续必须先由协调器提供同一
  source manifest、同一 toolchain 和 RHI-specific harness 的受管 grant，再按上文矩阵执行
  pre-profile；在此之前保持 `优化授权: not_started`。
- product window、PNG、RenderDoc、WPR power：未运行；
- milestone/commit/WeCom：未满足验收门，不执行。
