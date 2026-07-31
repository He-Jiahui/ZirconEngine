---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: gateway-stable-call-lock-and-clone
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/01
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/gateway/contract.rs
  - zircon_editor/src/core/gateway/handle.rs
  - zircon_editor/src/core/gateway/capabilities.rs
  - zircon_editor/src/core/gateway/session.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/lifecycle.rs
tests:
  - stable gateway tick/event/capture call lock-count regression
  - gateway replacement concurrency and lifetime matrix
  - editor idle and interaction WPR trace
  - session demand immediate/idle/after/malformed ABI matrix
  - retained host deadline replacement and native wait reset
---

# Editor01：gateway 稳态调用锁与快照复制

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_editor/src/core/gateway` 逐文件性能审查
- 修复责任计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 交接原因：gateway generation 与稳定调用所有权属于 Editor01 内核边界，不能由上层调用点各自缓存。

## 失败现象与复现证据

`EditorRuntimeGatewayHandle` 用 `Arc<RwLock<SharedEditorRuntimeGateway>>` 保存可替换 gateway，但每次 capabilities、session、tick、event drain、capture、profile、subscription 和 operation 调用都先获取读锁，再 clone trait-object `Arc`。`capabilities()` 随后还深 clone 包含多个 `String`/`Vec<String>` 的 `RuntimeCapabilities`。gateway 替换属于低频控制面，锁和 owned capability snapshot 却落在稳定数据面，放大编辑器 tick、事件泵和 viewport capture 的主线程成本。

## 最低共享层根因

可替换 gateway 的控制面 owner 与稳定数据面共用一个 `RwLock`，且 capability projection 没有绑定 gateway generation 的共享快照。

## 架构修复验收

- 把 gateway generation/replacement 与稳定调用快照分离；稳定调用读取 immutable `Arc` snapshot，不经过共享 `RwLock`，替换时原子发布新 generation。
- capability projection 随 gateway generation 构建一次并共享借用或 `Arc`；不得逐查询复制字符串集合。
- 旧 snapshot 在并发调用结束前保持存活；replacement、shutdown、session invalidation 与 poison/recovery 语义有并发测试。
- WPR/计数测试证明 idle/tick/event/capture 稳态 gateway read-lock 次数为零，并记录替换路径成本。

## 禁止临时方案

- 不得只把 `RwLock` 换成 `Mutex` 或给每个方法加独立缓存。
- 不得暴露借用跨越可替换 owner 的悬垂引用。
- 不得在未测调用频率时把该项宣称为帧占比结论。

## 修复结果与回传

Open state: `generation-bound ArcSwap gateway/capability snapshot、V3 frame-demand contract 与 capture-frame foreign-buffer ownership 已落地；并发/poison/RwLock-zero、demand matrix、foreign-frame explicit/drop release tests 已具备。静态合同、依赖 guard、格式与 scoped diff 均通过。新的 source-bound managed focused/matrix、独立复审与1080p/4K产品 trace 完成前不生成 fixed return`。

原 exact-11 managed reservation `a604598586b74e0e8e6b4d63fe948347` 在租约已过期且 Coordinator01 的 absolute-expiry FIFO failure 仍 open 时不再具备可接受的 current-source 保护，本会话已于 2026-07-18 在 `jobId=null/startedAt=null` 下主动释放。待 `pending-cpu-reservation-absolute-expiry-not-enforced` fixed return 后，必须基于重新取得的 exact scope 生成 fresh manifest/reservation；不得续约或复用该 reservation，也不得释放 foreign FIFO head 绕过协调器失败。

2026-07-22 current-source补充：ArcSwap generation快路仍成立；本轮删除Session owned-output decode的重复validate。failure继续open的当前P0不是恢复锁缓存，而是`tick_frame`丢弃frame-demand kind/delay以及`capture_frame`整帧foreign RGBA→Vec copy；分别回链PERF-MVP-424/023，须以idle wake与1080p/4K copied-bytes产品证据关闭。

## 产出记录与时间

| 时间 | 范围 | 状态 | 完成项与后续门禁 |
| --- | --- | --- | --- |
| 2026-07-29 03:55 CST | PERF-MVP-424 frame-demand P0 | 部分修复，未回传 | `SessionGateway::tick_frame` 现在将已验证的 V3 ABI demand 映射为 editor-owned `OnDemand/SleepUntil/Continuous`，host 仅通过外部 redraw bridge 与一个 `WaitUntil` deadline 消费该契约；`OnDemand` 同时恢复 native `ControlFlow::Wait`，不会保留旧 deadline 的额外唤醒。覆盖 immediate/idle/after 和 stale-wake replacement。精确 `rustfmt --check`、旧布尔 tick 签名 0、gateway 边界外 raw ABI 引用 0、scoped diff check 均通过。须在 source-bound managed Cargo、独立复审和 idle/interaction trace 后才可回传。`capture_frame` 的 foreign RGBA→`Vec` 全帧复制不在本次完成范围，继续按 PERF-MVP-023 交接。 |
| 2026-07-29 04:21-05:27 CST | PERF-MVP-424 focused managed validation | 外部编译失败，未回传 | Coordinator reservation `fa8f2fc27daa43e683058b728be7d6c4` 已一次性消费为 job `54235ab7814547538e41504536259ed1` / run `0b49fddf05ac4e5d8345f670dcaf5dc5`；11 个 gateway/cadence 路径的 source-manifest fingerprint 为 `7c96666b8658e2446beed13eb061313bc55e695366082f8e3ad76a274dba47fe`，运行中复核哈希逐项一致。精确命令为 `cargo test -p zircon_editor --lib gateway:: --locked --jobs 1 -- --test-threads=1`，target 为 `D:\\cargo-targets\\zircon-engine\\pool\\6086d4e99fffa706c76a62393215a5c41b9eb6111413eb4661949f6286829690`；运行 56 分钟后 exit `101`，`zircon_editor` lib test 在执行 gateway 过滤前因 75 个清单外错误失败。最低现有 owner 为 Editor12 插件/V2 runtime（descriptor、dynamic pane、play-pending visibility）、Editor08 command/settings、Editor09 asset refresh，另有工作台场景测试漂移；这些故障已有对应 open failure 记录，不能归入 Gateway 或以本次 run 回传。job 已 release，保留原始 stderr 路径；待依赖修复后以新 source manifest 重测。 |
| 2026-07-29 10:21 CST | PERF-MVP-023 capture-frame foreign-buffer ownership P0 | 源码完成，failure 保持 open | `EditorRuntimeFrame` 已硬切为不可 clone 的像素存储所有者：本地构造仍可用 `Vec`，Session capture 则直接持有已验证的 `ZrOwnedByteBuffer`，不再调用 `to_vec/into_vec`。帧同时持有 runtime provider `Arc`，所以 gateway 先析构不会让 ABI free callback 悬空；`release()` 可传回 callback 错误，Drop 作为回收兜底，ABI/shape 拒绝路径仍立即清理。回归测试覆盖“捕获后不提前 free、gateway 析构后 provider 仍存活、显式 release 恰好一次、仅 drop 亦恰好一次”。精确 `rustfmt --check`、`git diff --check` 及 capture-body 零复制静态检查通过；必须在新的 immutable current-source managed Cargo 和独立复审、1080p/4K copied-bytes 产品 evidence 后才可回传。 |
| 2026-07-29 11:05 CST | PERF-MVP-023 source-bound managed focused gate | 外部编译失败，failure 保持 open | reservation `a6d147f0e082442781920a2e7baeaef1` 已一次性消费为 job `640dc354cc38475daa1bd25e7217baf6` / run `bb226267623f4322839092a6f7365c15`，manifest fingerprint `d3510f7e2cd5a4ade996a4887d77103aee3b1c710e29b2714427666e9503eed5`。精确 gateway 命令自然终态 `exit 101`、测试未执行，先被 Render17 的 `RenderGraphPassProfileMetrics` root export E0432 与 Layout21 的 `BatchDrawPlanStats` E0063 阻断；均已写入对应 fixing-plan child failure record 并导入协调器。不得复用该 job 或将其当作 gateway 结论；待两个 fixed return 后生成新的 source-bound manifest/reservation。 |
