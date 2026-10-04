---
handoff_kind: failure
status: open
created_at: 2026-07-22
updated_at: 2026-09-27
summary_slug: ecs-event-message-bounded-lifecycle
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/tests/ecs_events_messages.rs
  - zircon_runtime/src/scene/tests/ecs_events_messages/lifecycle_scale.rs
  - zircon_runtime/src/scene/tests/ecs_observers_messages.rs
  - zircon_runtime/src/scene/ecs/events
  - zircon_runtime/src/scene/ecs/messages
  - zircon_runtime/src/scene/ecs/system/events.rs
  - zircon_runtime/src/scene/ecs/system/messages.rs
  - zircon_runtime/src/scene/world/events.rs
  - zircon_runtime/src/scene/world/messages.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib --locked -- messages --nocapture --test-threads=1
  - 10k idle frames and 1M retained-message counters
---

# Runtime08：ECS event/message bounded lifecycle交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：scene ECS events/messages 12/12逐Rust文件审查，PERF-MVP-485
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08 M3明确拥有events/messages分工、清理策略与cursor语义。
- 生命周期键：`ecs-event-message-bounded-lifecycle`

## 失败现象与复现证据

`Messages<T>`把全部payload保存在单个Vec，只有调用方显式`clear_messages<T>`才释放；仓内产品代码没有清理调用，连续writer会让entries和RSS随会话增长。`Events<T>`虽用current/next双缓冲和debounced capacity shrink保持两代有界，但`EventStore::update_all`每帧遍历所有注册channel并虚调用update，reader_count与本帧dirty状态不参与推进集合。

## 最低共享层根因

events/messages没有统一World/schedule lifecycle authority：messages把retention责任下放给未知consumer，events则以全registry扫描换取推进；channel缺少dirty membership、consumer cursor watermark、entry/byte/age预算和drop/backpressure diagnostics。

## 架构修复验收

- 明确定稿events与messages的持续时间和消费语义；messages采用cursor-aware generation/ring或等价结构，并同时有entry、byte、age硬预算与drop/lag指标。
- World/schedule拥有唯一推进/回收点；调用方无需记住每帧clear，slow reader策略显式且可观测。
- EventStore维护dirty/retirement channel集合；只有send、仍有current payload或容量回收倒计时的channel参与update，stable idle不全扫registered types。
- dormant event subscription保持late connect不回放历史；connected reader顺序不丢不重，clear/generation reset语义不变。
- types 1/1k/100k、writes 0/1/1M、idle 10k frames记录channel visits、retained entries/bytes、drop/lag与p95：idle visits近0，RSS严格有界。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止仅在编辑器或某一个system手动clear，继续让其他producer无界。
- 禁止无指标地静默丢弃oldest消息或按reader_count为0跳过必要generation推进。
- 禁止为dirty set复制另一份channel payload/queue truth。

## 修复结果与回传

Open state: `前向修复中`; no pass is claimed.

- 已完成的底层收敛：`EventStore` 以单一 `active_channels` 工作集合推进队列；仅写入、尚有 current/next payload 或仍处于 capacity-shrink debounce 的 channel 会进入下一帧。稳定 idle channel 不再随全部注册类型逐帧扫描。`last_update_channel_visits` 与回归测试覆盖 idle 0 visits、写入后推进及递送后退休。
- 已完成的消息下层收敛：`Messages<T>` 改为单调 `MessageId` 的 `VecDeque` 日志；`MessageRetention` 对 entry、消息自身声明的 retained-byte charge 与 age 设置硬上限。预算/age 逐出保留累计 entries/bytes telemetry；`MessageCursor` 用 sequence window 而非 Vec 下标读取，报告 slow-reader `dropped_count`，并以 explicit-clear generation 边界避免把主动清理误计为 lag。`MessageStore` 以单一 active-retention-channel worklist 推进，只有仍有 retained payload 的类型会留在下一帧；`World::last_message_advance_channel_visits()` 公开当前 First-stage worklist visit 计数；`advance_frame` 只由 First-stage `UpdateEvents` owner 调用，统一 age 回收。
- 保持未完成：10k idle / 1M retained-message 的受管计数器与 p95/RSS probe 尚未执行；在其真实结果返回前，此 failure 仍不得关闭，也不能声称完整运行时验收通过。
- 当前证据仅为消息/events 精确路径的 `rustfmt --check`、`git diff --check`、source invariant audit 与行为测试源码；尚未申请 Cargo 或受管 performance probe。

## 2026-09-27 当前测试实现约束修复

- 稳定 fixing Session：`failure-roll-01a0df1a-runtime08-message-lifecycle-r1`；base `bc02eefafead65dbf5050482110e8175250a5e77`，epoch `628`。领取范围仅为 `scene/tests/ecs_events_messages.rs` 和本记录；保留 archived `astra-task-contract-tests-20260905` 的现行格式与 event-reader 闭包类型修正，未接管其他活动生产 owner。
- 根因：`message_retention_source_has_single_first_stage_lifecycle_owner` 仍硬断言 `MessageStore::advance_frame` 使用 `std::mem::take`。当前 HEAD 已合法使用双 HashSet 的容量复用 worklist，旧断言条件为 false；这是精确静态复现，尚非 Cargo RED 结果。
- 删除这条过时实现字面约束，保留 retention、推进入口、回调、visit diagnostics 与唯一 First-stage authority 检查。扩展既有公开 World 行为测试：两种消息 age=2/0、空通道退休、不同年龄通道 visit 计数、等于/超过 age 上限、retained entries/bytes 与 age-drop 计数、跨通道 clear 隔离、退休后再次写入和连续 10,000 idle 帧零 visits。测试总数保持 22；没有替换为 swap 字面约束，没有修改生产算法。
- 原始复现命令保留为证据：`cargo test -p zircon_runtime --lib ecs_events_messages --locked --jobs 1 -- --nocapture --test-threads=1`。现行受管命令采用文头的 Rust 1.94.1、Windows 原生、`--locked`，不覆盖协调器 jobs 或产物目录。
- 动态验收仍待受管执行：精确 guard 必须实际执行 1 项，完整 `ecs_events_messages` 必须实际执行 22 项；还需直接消费者、向上 gates 与原文 types 1/1k/100k、writes 0/1/1M、10k idle、p95/RSS 规模矩阵。新增 10k idle 行为测试不能替代完整性能验收。
- 目前只记录源码修复；未取得本切片 Cargo 通过票据，不声明 failure return、完整验收、closeout 或提交完成。

### 冻结源码、审查与受管等待

- 精确源码及本记录已冻结为 snapshot `4599`（请求 `2859ddb0edac4c2b932c05762700b938`）；测试 SHA-256 为 `9597b91b60a2ab158531e9abbb0aaaaf005de793875202acbd5bd2368e932d50`。`rustfmt +1.94.1 --check` 与两路径 `git diff --check` 实际通过。独立只读 source/record 审查为 Critical / Important / Moderate = `0 / 0 / 0`，反向还原本轮两处改动后精确匹配领取前测试 SHA-256 `30d54afa9719ee845781ba303ef8a59d11e824db004d24bc3754cbb5a959dbd6`；这是 scoped 审查，不能替代用户命名的最终 closeout 审查。
- 最小下层/原复现/直接消费者批次已准备：`cargo +1.94.1 test -p zircon_runtime --lib --locked -- messages --nocapture --test-threads=1`。必须在真实输出中核对 `ecs_events_messages` 22 项、`ecs_observers_messages` 20 项及基底 `messages::store::hash_active_channel_tests` 两项常规测试实际执行；其 ignored set-helper benchmark 不是本 failure 的真实 MessageStore 性能验收。生产输入使用 pinned baseline，不吸收活动 owner 的 dirty cursor/queue/id 等文件。
- 计划请求 ID 为 `failure-roll-01a0df1a-runtime08-messages-4599-20260927-r1`；规格保存在 `.codex/tmp/failure-roll-01a0df1a-runtime08-events-validation-prepared.json`。状态明确为 `prepared_not_submitted`：尚未受理、没有 ticket、没有 Cargo 执行或通过结果。外部工作树已知真实 hash 漂移尚未恢复（Editor17 admission 请求 `e3f31ee4f2854ed8922a0d247206f537`）；Coordinator01 诊断 attempt `62e6c8e520d04ba9bf246ad6b8a0d996` 又在 pinned metadata 阶段因配置代理连接失败而结束，未编译。15:08:35 UTC 的只读 TCP 复核确认 `127.0.0.1:7897` 返回 Windows `10061`。
- 仅在实际捕获/网络恢复条件满足后，重新核对本稳定 Session、源码哈希、基底和外部 current commit/state，通过正常受管路径提交既有规格；不盲重试、不手工 Cargo、不绕过外部 inventory/archive。Coordinator01 已向原 goal 线程排入一次结果消息 `01a0e360-0b90-7692-a3be-890bc4c58978`，没有周期监控或重复消息。全部动态、规模、性能和正式回传/closeout 门继续开放。

## 2026-09-27 直接批量写入消费者复现与修正

- 同一稳定 Runtime08 Session 通过审计 transfer-preview/apply 接管 `ecs_observers_messages.rs`：原 owner `runtime08-ecs-bundle-width-current-compile-r1-20260808` 已 archived，原 attribution 的哈希/基底过时，无 live lease。领取前 SHA-256 `6c9bc0062561974a9edd00cc6b65c216c08261235ca05ae229a1e7fad446cbda` 相对 HEAD 只有格式变化，全部保留。首 preview 在请求日志事务前因 SQLite `database is locked` 失败，没有受理请求或部分转移；自有 columnar 租约正常释放成功后取得新 preview，apply 请求 `329d00f1467d47a796a713cf59a50e8f`。生产源码不在转移范围。
- 最低根因是直接消费者的旧实现断言：HEAD/current 的 `Events::send_batch` 已由 `next.extend(events)` 插入并按长度差返回 written，仅非空批次更新 high-water；旧 `into_iter/size_hint/next.reserve` 三条在 HEAD 已静态为 false。`Messages` 已按当前 store frame 走 `write_batch_at_frame`，旧 `ids.push(self.write(message))` 在 HEAD/current 均为 false。消息 payload 全量 lower-bound reserve 在 HEAD 仍成立，但当前其他活动 owner 的有界保留优化已改变它；本修正不把该 foreign delta 当成 HEAD 失败、不接管其源码，也不要求恢复无界预分配。
- 批量 source 检查改为限定相应生产方法段，覆盖事件 extend、written 计数、非空 high-water、消息 returned-ID Vec 的 size hint 分配、当前 frame 传递及 system→store batch 路由。保留单事件写入、drain 和已有 cursor/observer 检查；没有改为接受 foreign bounded-reserve helper 的拼写。
- 扩展既有 `message_writer_batch_preserves_order_and_ids`：保留原五消息的 IDs/顺序验收，再在非零 First-stage frame 使用真实 System MessageWriterParam 写 batch；核空 batch 无计数变化、age=1 仍保留两条且 bytes/IDs/值正确、age=2 两条逐出、drop bytes 与 dormant visit=0，以及再次写入 ID 连续。20 项测试名称及顺序不变。新增源码与静态复现不代表已执行 Cargo RED/GREEN；原规模、p95/RSS 和向上 gates 继续待验。
- 原 snapshot4599/4600、未提交的 r1 规格和全部历史证据保留；本次补齐直接消费者源码闭包后另存版本，尚无本修正的受理 validation request、ticket 或动态通过结果。

### 直接消费者最终冻结与等待

- 当前消息测试及直接消费者源码连同记录已冻结为 snapshot `4603`，请求 `4dd2410e8bf543d291e27f82b5377322`；原消息测试 SHA-256 `9597b91b60a2ab158531e9abbb0aaaaf005de793875202acbd5bd2368e932d50` 不变，消费者 SHA-256 `e823f3d71170a3f50a569e9a8d91560a7227ace2b1f9c71b63a4df94baa19e9a`。消费者 `rustfmt +1.94.1 --check` 和精确两路径 `git diff --check` 实际通过；新方法段 11 条条件在 pinned HEAD 与当前只读样本静态成立。独立 scoped source/record/r2-spec 审查 Critical / Important / Moderate = `0 / 0 / 0`，不是动态通过或正式命名 closeout 审查。
- 后续批次只采用新 r2 未提交规格 `.codex/tmp/failure-roll-01a0df1a-runtime08-events-validation-prepared-r2.json`，计划 ID `failure-roll-01a0df1a-runtime08-messages-4603-20260927-r2`。精确源码闭包为上述两个测试文件；必须真实执行原消息 22、直接消费者 20、下层 hash-active 常规 2 项及其中 batch guard/真实 System batch 测试，允许核对额外过滤命中；旧 r1 规格与哈希完整保存。accepted request/ticket 仍为 null，full/upward 为 false。现行命令仍是 Rust 1.94.1、Windows 原生、`--locked`、不覆盖 jobs/目录的 `messages` 批次。
- 已消费 SSA owner 的自然边界 readiness，未开始 hold；其报告自身子任务只读、构建终止，外部 main HEAD 为 `c1184f39340d8612656c30257d8a37b811fbadbd`。后续一次只读盘点 HEAD 已变为 `ed515c0b574743783aaba56f1a8ab58667833e19`，actual comments group history 又证明 `cc5c6ea414447c91b7e3ae09693269e442686936` 提交及 core/GC metadata 后续写入；comments 的 readiness 请求仍未证明完整消费/就绪，另有 dirty owner 未证实。就绪声明、文件样本或静态审查均不是 inventory/archive seal。
- 网络选择仍待回复；本项继续挂起受管验证。只有实际 writer 窗口和网络恢复后才按现行 commit/源码哈希正常提交；全部原始规模、性能、向上、回传、正式审查和 closeout 门保持开放，不重发已有请求或通知。

### 2026-09-27 真实百万写入与生命周期尺度回归准备

- 沿用同一 Runtime08 primary、base/epoch 和全部旧 scope；经 transfer-preview `8b2ed34c22814472bbb8a76030c6220e`、transfer-apply `a5cb9b53e51a459f87ff04d829d433f4` 仅增加未来子路径 `scene/tests/ecs_events_messages/lifecycle_scale.rs`。父文件移除本次新增内容后精确回到 snapshot4599 的 `9597b91b...`，最终只挂载新模块；原 22 项名称、顺序和正文保持。新增 fixture 与两项回归一起留在语义子模块，父 720 行、子 233 行。
- 消息回归经真实 System MessageWriter 在非零 First-stage frame 分别写入 0、1、1,000,000 个带 `Box<[u8; 16]>` 的消息；entry 上限 8 / byte 不限与 entry 上限 16 / byte 上限 128 独立生效，防止一个预算掩盖另一个失效。核全部单调 IDs、最后至多 8 条的顺序/值、retained entries/declared bytes、budget drop entries/bytes、payload Drop 与存活峰值、立即读者 lag 和延迟读者 age 后累计 lag；age=1 保留、age=2 自动退休，随后 10k First-stage 空闲帧零 visits。payload 对象释放与 logical charge 不等于 allocator/RSS 通过。
- 事件回归经真实 System EventWriter/Reader 分别执行 0、1、1,000,000 burst；核 written、推进前不可见、全部递送顺序、重读为空和单通道 visits；使用现行 debounce 常量等待 current/next 两个缓冲长度及容量归零，然后经 First-stage 连续 10k 帧零 event visits。一个附加 idle event 类型不代表 1k/100k types 矩阵。
- 两项新目标必须在受管输出中真实出现：`scene::tests::ecs_events_messages::lifecycle_scale::million_event_burst_retires_payload_and_capacity_before_idle_frames` 与 `...::million_messages_enforce_entry_byte_age_and_slow_reader_boundaries`。同一 `messages` 命令的挂载族总数从 22 增为 24（父 22 + 子 2），直接消费者 20 和基底 hash-active 常规 2 保留。只完成了限定 rustfmt、diff 与静态挂载/归属核对，尚无本版本 Cargo RED/GREEN、受理票据或动态结果。
- Snapshot4599/4600、4603/4604 和 r1/r2 规格完整保留；本次将另冻源码并保存 r3 规格，columnar 编译闭包也须带上新的父/子及既有消费者修正后另存版本。尚不将旧规格改成通过，也不复用 foreign queue/cursor/columnar 性能增量。
- 原 types 1/1k/100k、release p95、过程 RSS、直接上行和正式回传/closeout 门仍开放。Pinned HEAD 的 message batch payload queue 仍全量 reserve(lower_bound)；现行有界 reserve 属其他活动 owner，不能将逻辑计数或本 fixture 的 Drop 当作已证明全量内存门。SSA readiness 和 comments BLOCKED/no-hold 的实际回传均已消费，仍未建立全体 writer 的 quiet/seal；联网选择仍待回复，当前不重提受管 Cargo。

### 2026-09-27 新版精确冻结与验证规格

- 同一 Runtime08 fixing Session 的源码/记录 snapshot `4625`（请求 `014f47582a284a73b0b543651ea0566a`）已冻结：父测试 `f09dd45635d8480f1b1f708febd277ac7d3832ef0f442443c67de204a8da8d17`、新子测试 `f74680036bcac199ac3e5bd9051ddc6d8f8381b91d1253eea428f3b23f9849c6`、直接消费者 `e823f3d71170a3f50a569e9a8d91560a7227ace2b1f9c71b63a4df94baa19e9a`。本段追加后须再冻最终记录；旧 4599/4600/4603/4604 均保留为历史证据。
- 新 messages 规格 `.codex/tmp/failure-roll-01a0df1a-runtime08-events-validation-prepared-r3.json`，计划请求 `failure-roll-01a0df1a-runtime08-messages-4625-20260927-r3`：精确 `--locked` Windows 受管 `messages` 批次必须真实执行父+子 24、直接消费者 20、hash-active 常规 2，含两项新增百万写入目标。旧 r1/r2 未提交规格仍保留，不能按旧哈希提交。
- Columnar 四个原有最小命令及原需执行项数 `16/3/35/56` 保持，按现行父/子/消费者编译闭包另存 `.codex/tmp/failure-roll-01a0df1a-runtime08-columnar-validation-prepared-r2.json`，四个计划请求 ID 均带 `4625` 和 `r2`。既有 columnar 自身源码 snapshot `4601` 与记录 `4602` 的原证据不变；本轮未把任一 prepared 请求提交或当作通过。
- 独立只读源码审查未发现新增 fixture、时序或挂载的确定性编译/语义错误（scoped Critical/Important/Moderate `0/0/0`），但没有 Cargo 执行。现行 `retained_byte_size()` 只代表声明的 16-byte payload charge，不能推出 `Box`/`Arc`/队列/返回 ID Vec 的过程 RSS；pinned HEAD 的 `write_batch_at_frame` 仍按百万下界预留队列。原文 types 1/1k/100k、完整 0/1/1M 规模矩阵、p95/RSS 与向上验收必须另由真实受管结果核对。
- 截至本段，外部 `zr_vm` comments owner 实际报告 `BLOCKED` 且没有启动全树暂停；SSA 仅报自然边界 readiness，网络选择未落定。没有新的 Editor17 seal、Runtime08 validation ticket、failure return、正式命名审查或 closeout。仅在现行外部树与 Cargo 网络恢复后按新源码哈希正常受理，避免盲重试旧请求。

### 2026-09-27 当前源码格式与受管规格修订

- 消费者测试文件在同一 Runtime08 Session 的精确租约下仅按 Rust 1.94.1 rustfmt 调整四处多行 assert 排版，20 项测试名称及原有行为检查保持；当前 SHA-256 为 36f8de77c47465392a2f9a386696c8887bbbdc86c664f9d4d4027a8dc96153e5。父/子测试分别保持 `f09dd456...`/`f7468003...`；三路径 rustfmt --check、精确 diff --check 与本记录结构检查已通过，仍不等于 Cargo 动态结果。
- 当前三源码与本记录冻结为 snapshot 4627（请求 7dd5cae2b1674f81a7af3644d7782334）；上一节 snapshot 4625 和 r3/columnar r2 规格只是历史冻结，不能按旧消费者哈希提交。当前 messages 规格为 .codex/tmp/failure-roll-01a0df1a-runtime08-events-validation-prepared-r4.json，计划请求 `failure-roll-01a0df1a-runtime08-messages-4627-20260927-r4`；columnar 四命令当前规格为 .codex/tmp/failure-roll-01a0df1a-runtime08-columnar-validation-prepared-r3.json。两者仍是 prepared_not_submitted，accepted request/ticket 为零。
- 协调器 baseline.attribute 请求 7eb043a5b8a141b6a337f766b373b7d5 已把当前父/子/消费者与本记录归属到原稳定 Session；本段写入后须按最终字节再冻结并更新本记录 attribution。原 1/1k/100k 类型矩阵、完整性能规模、RSS/p95、向上验收与正式回传仍开放。
