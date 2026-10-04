---
doc_type: architecture-audit-execution-flow
status: in_progress
date: 2026-10-03
plan_sources:
  - docs/plans/performance/index.md
  - docs/plans/performance/pending.md
  - docs/plans/mvp/index.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
  - docs/plans/optimize/00-engine-wide-review.md
---

# ZirconEngine 架构审查与优化执行流

这份记录是现有性能计划的执行索引，不替代各模块 owner 计划，也不把历史静态审查提升为当前验收。当前工作区含有其他会话和用户的既有修改；每次实施前都必须按路径确认 owner，不能用整树清理或回退建立假干净基线。

## 1. 当前架构地图

```text
Product/CLI
  -> zircon_app::entry::EntryRunner / ProductComposition
  -> RuntimeSession (versioned C ABI, owned output/release, wake registration)
  -> zircon_runtime::CoreRuntime
       -> module/service lifecycle + TaskGraph + EventBus + frame clock
       -> World/ECS + scene generations + asset/project authority
       -> render extract -> render graph -> RHI/WGPU submission
  -> App frame host (Winit cadence, host requests, redraw, surface/present)

zircon_editor::retained_host
  -> EditorRuntimeGateway / SessionGateway
  -> the same RuntimeSession handle and viewport surface binding
  -> command/transaction/selection/save/reopen flows

Plugins
  -> catalog/discovery -> ABI/version admission -> runtime/editor registration
  -> lifecycle ticket -> callback/teardown barrier

Tooling
  -> managed Windows validation -> package/product evidence
  -> Tracy/WPR/RenderDoc/GPU timestamps only after a current product binary exists
```

### 1.1 关键调用链与权威 owner

| 业务路径 | 当前入口与核心链 | 权威状态 | 线程/失败边界 | 证据 |
|---|---|---|---|---|
| 启动 | `EntryRunner` -> `RuntimeSession::create_with_profile_and_project` -> versioned `create_session` -> module composition receipt -> Winit app | `RuntimeSession` 持有 ABI handle、动态库和输出释放器；Runtime 持有模块图与 World | 创建失败须保留可清理 session；宿主线程负责窗口和 surface | E1，需 M0.3 current-source compile |
| 帧循环 | `about_to_wait` -> `pump_frame_loop` -> `tick_frame` -> host-request drain -> `request_redraw` -> redraw -> `capture_frame`/present | Runtime 产生 frame demand 与 owned output；App 负责 cadence、窗口生命周期和退出 | tick/present 失败走终止路径；请求和输出必须有预算、generation 与 affinity | E1/E2，需 F0/F2 动态证据 |
| 场景/渲染 | World commit -> derived scene -> camera-neutral render extract -> render graph -> RHI/WGPU | World/scene generation 应是唯一生产真值，per-view 只消费 sealed extract | commit、extract、GPU resource lifetime 需明确屏障和设备丢失回滚 | E2/E3 静态，动态阻塞 |
| 编辑器 | retained host -> `SessionGateway` -> command/transaction/selection -> Runtime operation -> save/reopen | Runtime registry/project persistence 是真值；Editor 只持 authoring projection | callback 不得跨任意 World 锁；save/import/close 必须可取消和有 deadline | E2 静态，F4 未验收 |
| 插件 | catalog/discovery -> ABI admission -> registration -> lifecycle callback -> unload barrier | 单一 catalog/version/lifecycle generation | ABI 拒绝、回调异常、卸载等待必须原子且可诊断 | E2/E3，M3 未验收 |
| 停机 | close request -> stop admission -> release viewport surfaces -> destroy session -> unregister wake | App owns host teardown; Runtime owns module/task shutdown | 失败必须保留可重试状态，不得静默 Drop 或跨线程释放 | E1/E2，需 F0/F5 |

## 2. 统一问题队列

问题先归最低共享 owner；以下条目是当前可追踪的队列，不代表全部已经复现。

| 优先级 | 问题与根因假设 | owner | 当前证据 | 最小验收 |
|---|---|---|---|---|
| P0 | `00` current-source Runtime -> Editor -> App 编译门未在同一指纹上闭合；旧请求/历史 receipt 不能替代当前编译 | `docs/plans/mvp/00` + 最早报错的 Runtime owner | 23:32 UTC 独立 Runtime package check exit 0，53 个选定文件及 17,871 个观察输入前后一致。随后共享输入变化；新 libtest 编译在 Interface build script 报错，已做局部修正，重新验证中。测试和 Editor/App 仍开放 | Windows 独立本地 package check；保护其他构建和原生锁；错误按最低 owner 向上收敛 |
| P0 | Event/Message writer 可以同时持有同一 Store 的完整可变引用，异类型 writer 缺少 typed channel ownership | Runtime event/message owner | 17-path typed owner/grant 实现与 7 项行为回归已冻结，独立源码审查未发现剩余具体阻断；23:32 UTC 生产代码 check 通过，测试目标、行为与别名模型验证待执行 | typed grant；observer 顺序、空批次、Message 延迟创建、retention 与唯一释放回归 |
| P1 | App host-request 链存在多次 Vec drain、编码/解码和逐帧重复预算工作，且 tick 与 drain 顺序需要真实产品证据 | App entry / Runtime10 | `performance/pending.md` 的 App 行；静态调用链已读，尚无 current product trace | 同一 fixture 记录 queue count、bytes、alloc、wait；稳定空批次工作为零且语义不变 |
| P1 | ECS/scene/render extract 仍存在多套 generation、World/DTO clone 与 per-view 重建路径 | Runtime03/07/08/10/11、Render04/07/12/17 | `2026-08-15-world-ecs-frame-extract-current-architecture-review.md`；静态 E2/E3 | 单一 commit/extract generation；稳定帧无 full clone/rebuild；F2/F4 像素和编辑行为不变 |
| P1 | Render graph 以全量 `previous -> pass` 串行链、裸 handle、锁内 compile 和逐帧 lint 扫描限制并行并制造重复工作 | Render01/02/17、Runtime11 | `2026-08-15-render-graph-current-architecture-review.md`；源模型未实测 | versioned handles、编译锁外发布、可并行 ready width、稳定帧 lint allocation 为零；RenderDoc current MVP capture |
| P1 | Editor jobs/gateway 与 Runtime TaskGraph 存在并行调度权威；callback/World lock、同步 wait 和终止 generation 不统一 | Editor01/12/14、Runtime06/10/11 | `performance/pending.md` editor jobs/gateway closures；静态 E2 | 一个 task authority、bounded lease/deadline、callback 锁外、F4 save/reopen 和 shutdown fault injection |
| P2 | catalog/插件入口、source-shape tests、重复 DTO/兼容路径增加维护分支和 false-green 风险 | Plugin/ABI、Tooling、各最低 owner | 现有 review/pending 条目；未完成动态 parity | 删除前覆盖 feature/registration/reflection/ABI；行为/复杂度测试替代源码字符串测试 |

## 3. 阶段与进入条件

### M0 架构阅读与归属确认

- 固定 source fingerprint、toolchain、workspace 状态计数和现有 failure/receipt 索引。
- 逐入口读 descriptor、lifecycle、schedule、World、extract、Gateway、plugin ABI 和 teardown。
- 为每项问题登记唯一 owner、调用链、线程约束、失败/退出路径和证据等级 E0-E4。
- 退出条件：所有 MVP 相关入口达到 E1；P0/P1 具有明确最低 owner 和验证计划；未读域保持 pending。

### M1 恢复可运行基线

- 依 `00 -> F0` 顺序修复 current-source 最早共享编译/启动阻塞。
- Windows 原生受管 Cargo 输出只写 `D:\cargo-targets`、`E:\cargo-targets` 或 `F:\cargo-targets`；先确认当前 builder 和磁盘容量。
- 仅在当前产品二进制可重复运行后验证 Tracy、RenderDoc、GPU timestamp、WPR 能力。
- 退出条件：可归因 binary、启动/退出诊断和工具能力矩阵；否则保持 `blocked_by_00`，继续修最低 owner。

### M2 并行审查与复现

- 按不相交源码范围并行静态阅读；复现代理只验证问题是否成立，不直接扩大修复范围。
- 动态问题必须有触发条件、根因、影响、样本 fixture 和 acceptance counter；静态风险不能写成瓶颈。
- 退出条件：问题去重完成，每条待修项均有 E2/E3 或明确待测状态。

### M3 分批修复与精简

- 先修最低共享原因，再更新全部消费者、测试、ABI/版本、持久化迁移和文档。
- 删除重复实现前检查 feature/cfg、反射注册、插件入口和外部契约；旧 ABI/旧格式按显式迁移或拒绝处理。
- 共享契约、根清单和跨模块迁移串行集成；每个可写文件只有一个 owner。

### M4 验证与性能对比

- 受影响 package focused batch -> 依赖完整波次 -> F0-F5 产品门；失败从最低支持层修复。
- 相同 fixture、profile、硬件和显示设置，优化前后各至少 5 次，稳态预热后采 30 秒，记录 p50/p95/p99、CPU/GPU、RSS、分配、等待、上传。
- Tracy 负责 CPU/线程/任务/等待；RenderDoc 只捕获当前 MVP 产品帧；WPR 补 CPU sampling、上下文切换和 I/O。

### M5 归档与下一轮

- 只将已读源码、问题处置、回归和动态验收齐全的模块移入 accepted/review；待验收项留在 pending。
- 每个里程碑写一条带 source/artifact fingerprint 的 outcome；未完成问题保留 canonical failure 链接。

## 4. 当前并行任务包

当前执行上限按最新指令为 **3 个 `gpt-6-sol` subagents**。每个任务只读或拥有明确不相交文件范围；子代理不独立启动构建，主代理串行负责本地验证：

1. Runtime/App entry：确认启动、帧循环、输入、surface、退出和动态失败路径；返回 E1-E3 调用链与最低 owner。
2. Runtime data/lifecycle：复核 Query/ECS/事件消息/任务调度的共享状态和冗余实现；只提交根因与验证设计。
3. Graphics/render hot path：复核 extract、render graph、RHI submission 的并行边界、复制、锁和上传；只提交问题证据与修复切片。

每个返回必须包含：源码路径与行号、调用链、触发条件、根因、影响等级、owner、建议切片、回归测试、证据等级，以及是否与已有 pending/review 条目重复。主代理负责去重、串行迁移和最终验收；代理报告不直接解除任何 MVP gate。

## 5. 当前边界

- 旧本地协调器及其 lease/heartbeat/claim/replay 流程已退役；本记录不请求、不恢复也不推断任何外部服务端点。
- 保留其他会话的文件、索引、进程、历史 receipt 和未跟踪内容；不以 `git reset`、整树删除或手工清理制造基线。
- 当前 Query 切片保持原有精确应用记录；最新 Runtime package check 没有绑定其全部 46-path 指纹，focused 行为、性能和 MVP 验收仍开放。
- 当前阶段结论只表示架构流程和 owner 归属已经建立；M1/M2/M3/M4/M5 仍开放。

## 6. 本轮并行审查结果

以下结论均来自当前源码阅读；没有把静态风险写成运行时瓶颈，也没有解除任何 MVP 或性能 gate。

| 编号 | 结论 | 最低 owner | 进入修复前的证据门 |
|---|---|---|---|
| R1 | `CoreRuntime::shutdown_until` 的模块错误早返已改为完整模块/graph 聚合；相同绝对 deadline 继续 graph 关闭，保留模块主要错误和 graph 次要错误。 | `core/runtime/shutdown.rs`、模块 shutdown report 与 TaskGraph owner | 源码和消费者迁移已独立静态复核；Core 的 3 个失败/重试测试及 Dynamic close 的 2 个真实 scope/module/native-worker 顺序与聚合重试测试尚未执行。RetainedPacket 仍只在完整 Core 成功后标记完成，graph 成功回执不能绕过失败模块。 |
| R2 | 默认 App 的 `gamepad-gilrs`、Editor/Dev 的 DesktopApp/Reactive、Runtime Idle、帧许可内 polling 生产链已确认；没有 Gilrs 事件到 Winit proxy 的独立唤醒接线。 | App entry / gamepad polling | 动态复现 pending；最小方案是单一可取消/join 的 Gilrs owner、有限事件队列、空转非空合并 proxy wake，保留 ABI 和 rumble 结果。超时检查取消时不能请求帧；退出验收还依赖 R11。 |
| R3 | surface unbind 失败后 App 清零状态并释放 Window，Runtime ledger 却保持 Bound；ABI 只复制 HWND，RHI 要求 native owner 活到 surface session 销毁。 | App surface lifecycle + RuntimeSession binding ledger | 9-path 实现与受控 ABI provider 已独立静态复核：解绑失败保留宿主资源，销毁成功后才释放；销毁失败沿用既有 fatal 边界。10 项行为回归及独立 fatal child 尚未执行；真实 HWND/Destroyed/Suspend/rebind 验收仍开放。 |
| R4 | 动画 pose 的 mesh 存在性检查先构造完整 `SceneNode` 投影，只为读取 `mesh.is_some()`；这会放大 extract 分配。 | `scene/level_system_render_extract.rs`，必要时 World cheap presence API | profile pose 数量、投影分配和直接组件 presence 查询的对照；没有采样前保持 pending。 |
| R5 | shader PBR viewer 的 `build_viewport_render_packet` 无条件 clone World；静态或重复请求可能重复投影重建。 | `scene/world/render.rs` 与 viewer caller | 用稳定重复帧 profile 确认 clone/投影成本，再设计可复用 prepared snapshot，保持 mutable render systems 语义。 |
| R6 | render graph compile 的依赖推断与 resource state plan 分别扫描 access/history，可能重复工作；影响 graph compile/recompile，不等于每帧执行成本。 | render graph builder compile/inference | 加 visit counter 和代表性 graph compile 样本，确认重复访问后再合并中间表示。 |
| R9 | 多 primitive mesh 的 render extract 为每个 snapshot 重复 clone 同一 `morph_weights`，大权重向量会按 primitive 数量放大分配。 | `scene/world/render.rs` 的 mesh snapshot builder | 用多 primitive + 大 morph fixture 记录 snapshot 数、morph bytes 和分配；先确认 `RenderMeshSnapshot` 的 ownership/ABI，再考虑共享 backing。 |
| R7 | Event/Message writer 原先同时持有同一 Store 的完整可变引用；当前实现改为分别借用稳定的类型队列与受同步保护的活跃元数据。 | Runtime event/message owner | 唯一 owner 保存 `Box::into_raw` 的原始分配指针，类型查找只借 owner 元数据；错误的 shared payload 转可变草稿已保留并撤回。23:32 UTC 生产代码 check 通过；7 项真实回归、测试目标编译、别名模型及成本测量待验收，不推断整个 Scheduler 的安全性。 |
| R8 | Query/ECS 46-path 切片已精确应用，原 normal run 在 compiler 前阻塞；新的独立 Runtime check 不关闭该切片的行为、MVP 或性能门。 | Query/ECS owner + `00` baseline | 保留原 validation receipt，当前 toolchain 已可运行，后续绑定切片指纹执行最小 focused batch；不重放旧请求。 |
| R10 | 项目打开后替换已登记的资产根时，普通 World 保存和 Editor 事务的写入 guard 都重新解析登记根，将新目标作为授权基准；现有 ignored regression 只检查共享 URI 解析器。 | `PackageAssetRegistry` + 两个写入消费者 | 最低 owner 与实际保存调用链已独立静态确认；注册时保留身份、两个消费者共用基准。动态故障、真实保存、目录替换竞态及路径验收仍开放。 |
| R11 | 锁定 Gilrs 0.11.2 默认启用 FF；其 server 线程丢弃 JoinHandle，channel 断开后循环仍继续。WGI 已有 stop、回调注销与 join，FF 是本次剩余 owner 缺口。 | Gilrs FF lifecycle / App gamepad owner | 官方锁定源码与 cache 字节一致，调查时 master 的 FF server 也未修复。保留 rumble 的独立取消、设备清零与 join 候选仅存 OUT；尚未接入、编译或动态验证。App worker 的 join 不能替代依赖线程退出验收。 |

### 6.1 修复排序

1. 先恢复 `00 -> F0` 的可运行 current-source 基线，并解决 R1 的 shutdown 聚合语义；关闭路径不完整时不做运行时性能结论。
2. 在同一产品二进制上复现 R2/R3，再按最低 owner 修复输入唤醒和 surface rollback；每项先有失败测试或可重复 trace。
3. 以 profile counter 验证 R4/R5/R6 的实际成本，只合并能减少重复投影、clone 或 compile visit 且不改变 generation/ABI 契约的切片。
4. R7 的 typed writer ownership 与 R8 的 Query 迁移是正确性门，须在依赖它们的产品验收前完成；R10 在持久化门复现并修复。完成 focused、依赖波次和 F0-F5 后再接受 Tracy/RenderDoc 性能结论。

## 7. 当前独立编译恢复证据

2026-10-03 UTC，主代理使用 `python -B -m tools.dev.local_cargo` 执行 `check -p zircon_runtime --locked`，目标位于 `D:\cargo-targets\zircon-local\windows\m1-runtime-default-20261004`。命令终止于 exit 101，实际报 15 条源码错误、431 条警告。

- 原始诊断：[diagnostics.jsonl](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-runtime-check-20261003-diagnostics.jsonl)，SHA256 `f9a83376766b55c153479907c60193e09ecc7a14a5cd4439c4b0cd6ebf0eeab6`。
- 本地结果：[receipt.json](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-runtime-check-20261003-receipt.json)，SHA256 `89c3d4d0d89c01abbd78c9f141fdbba375249065aacc6a2213be59c303256ace`。记录读时指纹不等于整树编译 source seal。
- 三个独立审查包确认：生命周期 `Option<Result>` 与导出、错误属性归属、IME 两层可见性、缓存组件 import、typed root resolver、序列化 borrow/move 顺序、导航类型/关联函数、操作服务 checked snapshot reservation。
- 10-path before 字节已保留在 [before.json](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-runtime-forward-20261003-r1/before.json)，原业务实现、外来修改与索引保持；局部前向修正已应用。操作服务超预算行为另需真实 focused regression。
- 修正后的 `check -p zircon_runtime --locked --message-format=short` 已终止于 exit 0；[本地回执](D:/cargo-targets/zircon-local/windows/m1-runtime-default-20261003-r1/runtime-check-receipt.json) 记录 10 个修正文件前后指纹一致，整树 source seal 和正式验收仍为 false。[编译日志](D:/cargo-targets/zircon-local/windows/m1-runtime-default-20261003-r1/runtime-check-short.log) SHA256 `b22aab757187ca1e557822692ff2f2dd8e4e1b8a8eb685dd30049d56b45ba8ee`；Runtime lib 仍报告 1656 条警告，其中 25 条重复。此结果只关闭本轮 15 条编译错误，不提供测试、Editor/App 或 F0-F5 验收。
- `git diff --check` exit 0。7 个无既有格式差异的文件通过 `rustfmt --check`；场景、导航和操作服务的原有格式差异保持。格式检查不提供行为或编译验收。
- RenderDoc MCP 的一次只读 `list_instances` 返回 0 个实例；MCP 方法可调用，但当前捕获/回放能力未验收，没有执行 capture。

### 7.1 后续源码与测试门

首个停机回归使用原有 `graph_report` 接口，但 `cargo test -p zircon_runtime --lib --locked module_failure_still_returns_graph_receipt` 在测试目标编译阶段失败：38 条源码诊断、1693 条警告，测试执行数为零。[原命令回执](D:/cargo-targets/zircon-local/windows/m1-runtime-default-20261003-r1/core-shutdown-baseline-red-receipt.json) 保留 exit 101 与 5 个选定文件前后指纹；[原日志](D:/cargo-targets/zircon-local/windows/m1-runtime-default-20261003-r1/core-shutdown-baseline-red.log) SHA256 `879012fb5e35519dc6f75878dde3513274c4e2c4f2a5bdf1303cb7f472fc0751`。这不是 R1 行为复现成功；其中 10 条诊断来自新增 snapshot 测试调用内域私有方法。

支持层已前向修正测试文件定位、descriptor/asset/snapshot 导入、工厂闭包借用类型、ProjectManager mutable fixture 及最小内部可见性；缺失的 World publication 子模块已补入 5 个真实 owner/generation/lifecycle 行为测试。[10-path before 与 delta](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-runtime-test-support-20261003-r1/before.json) 保留原字节。这些修正的测试目标尚未完成编译，不能称 38 条错误已关闭。

R1 的 [4-path before 与 delta](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-core-shutdown-aggregate-20261003-r1/before.json) 保留原模块/动态调用方/App 测试。完整聚合测试已替代临时单项测试；操作 snapshot 的 3 个测试也已挂载。7 个新/契约/测试文件通过 `rustfmt --check`，既有支持文件的无关格式差异保留；格式与静态审查不能替代编译和测试。

较早的 D/E/F 可用空间样本均低于 `tools.dev.local_cargo` 的 35 GiB 启动门。21:16 UTC 的一次只读样本显示 F 盘回到 35.915 GiB，但随后新的 Runtime `test --lib --no-run` 在实际启动预检时 exit 2：目标盘再次不足 35 GiB，Cargo 未启动，没有 compiler artifact 或测试执行。[该独立预检回执](F:/cargo-targets/zircon-local/windows/m1-runtime-tests-20261003-r2/runtime-test-compile-receipt.json) 保留 27 个选定文件前后指纹一致和 `formal_acceptance: false`，没有重放历史请求。

后续顺序保持 Runtime affected check/tests → Editor → App → F0-F5；容量门实际通过后再执行新的独立本地命令并保留原回执。当前新代码的编译、行为、MVP 与性能验收继续开放。

旧协调器已退役。独立 Jenkins 的只读 status 因托盘源码与激活版本不一致而失败；此观察不提供当前 MAIN 端点、恢复结论或历史 transport 故障根因，也不授权重启或重放既有请求。Runtime -> Editor -> App 与 F0-F5 继续开放。

### 7.2 当前源码切片与验证配置

R3 的 [9-path 当前后像与完整 delta](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-app-surface-lifetime-20261003-r2/after-r2-sha256.json) 及 [独立源码报告](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-app-surface-lifetime-20261003-r2-independent-review.json) 保留原文件字节。fatal child 的返回与 panic 路径直接退出，避免 Drop 补发 abort 造成误通过；父进程有 10 秒退出观察和 1 秒终止观察期限，只有确认退出后收集输出。受控 provider 使用真实 App/RuntimeSession/绑定账本与 ABI，但没有创建物理 Window，不能关闭 native window 门。

App 默认 `check -p zircon_app --locked` 保留 Gilrs。R3 行为批次另使用 `test -p zircon_app --lib --locked --no-default-features --features shipping-runtime,dynamic-api entry::runtime_entry_app::surface_present::lifecycle::tests -- --test-threads=1`；默认 App libtest 不会包含这组受 `!gamepad-gilrs` 保护的回归。两种配置分别验收。

R7 的 [17-path manifest](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-event-message-owner-20261003-r1/manifest.json) 与 [独立源码报告](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-event-message-owner-20261003-r1/typed-channel-owner-independent-review-r1.json) 绑定当前 typed owner、读写 grant、消费者与文档。State 不缓存跨轮 payload 指针；同类型读写冲突仍由原准入规则拒绝，observer 顺序与 Message 延迟发布保持。回归覆盖异类型交替写入、reader 共存、准确的元组错误、空批次、frame/ID/retention、scoped thread 及 clear/World drop 唯一释放。结构回放、源码审查和生产代码 check 不提供测试目标、行为或 Miri 验收；新增分配、Atomic 和 Mutex 的成本尚未量测。

R11 的 [依赖调查与候选](E:/zircon-profiles/r2-r11-gilrs-shutdown-support-20261003/shutdown-support-report.md) 基于官方提交 `07e286e24b046cf39e5c367daa2770b805a64692`，包含 FF owner 和真实 MsgServer 测试候选。依赖来源、许可与受维护 package 的接入方式尚待确定；Main、Cargo cache 和默认 FF 配置保持。真实线程退出、设备停振及 native 调用期限仍须验证。

当前 [00 基线流程](../mvp/00-current-source-baseline-recovery.md) 已与协调器退役规则对齐。23:09 UTC 的一次整合样本显示 D/F 盘分别有 37.857/37.040 GiB；M8 的 `soleEntryEnforcementApplied` 仍为 false，独立本地入口继续适用，实际命令仍须通过 35 GiB 启动检查。新的 Runtime check/test → Editor check → App check/聚焦回归按当前后像验证，源文件变化则保留原结果并重新归因。

本次 [有限协调结果](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-coordination-boundary-20261003-r3.json) 记录 Jenkins status exit 1 的激活版本不一致阻断，当前 MAIN 端点与实际服务负责 chat 未得到验证。该记录不改变历史请求或任何 MVP gate。

### 7.3 当前生产代码 check 与新的测试编译边界

23:25–23:32 UTC，新独立命令 `check -p zircon_runtime --locked --message-format=short` 已终止于 exit 0。[本地回执](D:/cargo-targets/zircon-local/windows/m1-current-validation-20261003-r3/runtime-check-r1-receipt.json) SHA256 `5a6c23b1a8bf43e0c6ff71073e5ab2bd8adb9a4c8ac8b86923f176fda8094fa6`，绑定 R1/R3/R7 与测试支持等 53 个选定文件，并记录 17,871 个 workspace Rust/Cargo 输入前后无变化。它是该次共享源码观察上的独立编译证据，整树 source seal、测试执行与正式验收仍为 false。

后续 Runtime libtest 的 D 盘尝试在 35 GiB 预检 exit 2，Cargo 未启动；新的 F 盘尝试通过容量门，但依赖索引请求继承的 `127.0.0.1:7897` 代理不可连接，entry exit 101。原回执分别保存在 [D 盘预检](D:/cargo-targets/zircon-local/windows/m1-current-validation-20261003-r3/runtime-test-compile-r1-receipt.json) 和 [F 盘代理失败](D:/cargo-targets/zircon-local/windows/m1-current-validation-20261003-r3/runtime-test-compile-r2-f-receipt.json)，没有重放旧协调器请求。

沿用用户此前选定的直连方式，仅为新验证子进程设置代理覆盖；全局配置保持。00:02–00:05 UTC 的 [直连尝试](D:/cargo-targets/zircon-local/windows/m1-current-validation-20261003-r3/runtime-test-compile-r3-f-direct-receipt.json) SHA256 `7fdee73055a4f3d82bf6402ba001c029cfc7b5877af10ed91294f38079c61380` 已取得依赖并实际编译，但在 `zircon_runtime_interface/build.rs` 终止于 3 条编译错误、entry exit 101，测试执行数为零。该次 53 个选定文件与 17,871 个观察输入前后稳定；与 23:32 check 之间有 13 个共享输入变化，因此前一结果不能替代后一次编译。

最低层错误是把字符串拼接表达式放进 `format!` 的首参数。主代理保留 [当前共享原像、精确 delta 与后像](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-interface-render-support-20261004-r1/postimage.json)，只改为含 `{literal}` 的格式字符串；既有 BuildSet 验证、生成契约、所有其他外来字节和 HEAD/index 保持。build.rs 后像 SHA256 `86858edc3a7fb92d3f3adf66c0fa9361d6a544f5775b5f2fc261ce8e56571f8b`，[独立源码复核](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-interface-render-support-20261004-r1/renderer-independent-review-r1.json) 未发现剩余源码阻断。新的 54 文件独立验证已开始，日志已记录新 build-script 可执行产物；完整测试目标编译和行为结果仍待返回。

真实隐藏 Window 的 [OUT 测试候选](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-app-surface-lifetime-20261003-r1/out/physical-window-candidate-sol/README.md) 及 [独立源码报告](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-app-surface-lifetime-20261003-r1/out/physical-window-candidate-sol/independent-review.json) 尚不提供合入、编译或运行证据。Winit 的 Window Drop 只投递销毁消息，因此候选同时检查无额外强引用的 Weak owner 计数、HWND 和创建线程，并等待真实 OS Destroyed；仅 `IsWindow` 不能证明 App 仍持有 owner。该 Destroyed 仅覆盖 App 外层回调不重入 ABI，内部 `handle_window_destroyed` 分支未被执行；实际 RHI/GPU/DLL worker quiescence 继续开放。

后续 [聚焦验证清单](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-current-validation-batch-20261004-r1.json) 合并停机、typed writer/observer、snapshot、World publication 和动态会话重试，随后验证生成契约与 Editor/App。清单是准备记录，运行中的原命令须先终止并核对实际结果，清单本身不提供通过证据。

R10 的 [有限支持层审查](E:/zircon-profiles/engine-audit-20260930-01a0f06a/m1-r10-root-identity-support-20261004-r1/root-identity-independent-support-review-r1.json) SHA256 `c2ab1f3b24fa0faf6430907709acb7869d3398cfcc93247c86baab0669c0c769` 确认已有 canonical 路径在两个写入入口被重新解析；重复校验仍会刷新同一授权基准。修复候选须在现有 registry 成功注册时捕获 resolved identity，保留路径顺序、别名预登记、失败原子性与既有 owner/clone 边界。13 个审查文件前后指纹一致，但没有运行目录替换或保存；resolved path 身份也不等于持有文件对象句柄，最终检查之后的竞态仍是独立开放门。


### 7.4 当前受限写入与验证边界（2026-10-04 UTC）

当前 chat 的 15 项真实文件操作探针已终止并全部通过：[当前命令探针回执](E:/cargo-targets/zircon-local/codex-permissions/chat-probe-504f92c3f96948858545508ae4c15c70/receipt.json)。结论限于该次命令及继承子进程；其他 chat 与宿主后台进程没有由此获得权限证明。以后新证据、临时文件与编译缓存均写明确授权的 `D/E/F:/cargo-targets/zircon-local`；历史 `E:/zircon-profiles` 仅作只读引用。

本轮实际路径查询发现部分历史 E/D 回执与候选文件当前不存在，原记录保留为历史记录；没有删除、迁移或重建旧证据，也没有获得缺失原因或负责者证明。R7 当前 17 路指纹的 [独立续核](D:/cargo-targets/zircon-local/evidence/engine-audit-20260930-01a0f06a/m1-r7-typed-channel-owner-current-boundary-review-20261004-r1.json) 未发现新源码阻断，完整 alias/Miri、测试执行与成本门仍开放。R3 当前 9 路实现保持，受控 provider 不含真实 Window/ HWND；物理窗口候选尚未应用。

R1 的 [真实 Dynamic close 测试与精确 delta](D:/cargo-targets/zircon-local/evidence/engine-audit-20260930-01a0f06a/m1-dynamic-shutdown-behavior-20261003-r1/postimage-r2.json) 覆盖 scope 先排空、module 后清理、graph 最后实际 join，以及完整错误来源和原 runtime 重试。旧 session lifecycle 的字符串顺序断言已迁到行为测试，mirror/core/process-log 顺序守卫保留。格式检查通过，行为尚未执行。

本次有限 [服务与准备边界记录](D:/cargo-targets/zircon-local/evidence/engine-audit-20260930-01a0f06a/m1-current-validation-20261004-r4/coordination-boundary.json) 保存正常 Jenkins status 的 exit 1：`Jenkins 协调器尚未正式启用`。当前 MAIN 端点和负责 chat 未经验证，进程详情查询被权限拒绝，zr_vm 全组 census 与 capture readiness 未得到证明。没有 START、hold、服务恢复或旧请求重放。

当前 M8 `sole_entry_enforced` 实际为 false，D 盘新样本有约 168 GiB 可用空间。本轮 [20,396 个所选 Rust/Cargo 输入观察](D:/cargo-targets/zircon-local/evidence/engine-audit-20260930-01a0f06a/m1-current-validation-20261004-r4/source-before.json) 只是读时指纹。新的 Runtime `test --lib --no-run` 准备在 [MSVC 启动环境预检](D:/cargo-targets/zircon-local/evidence/engine-audit-20260930-01a0f06a/m1-current-validation-20261004-r4/runtime-test-compile-r1-launcher-failure.json) exit 1，Cargo 未启动、测试为零；环境修复使用新独立命令身份。当前源码编译、行为、Jenkins、MVP 和性能验收继续开放。


### 7.5 当前目录迁移、依赖与实际编译边界

本轮实际查询确认独立本地验证入口已经迁到 `tools.dev.local_cargo`，Cargo 工具 crate 的实际目录为 `tools/cargo`、package 名仍为 `cargo-zircon`。主代理只前向修正根 workspace 中该成员路径，保留所有其他共享清单和源码字节：[精确原像与 delta](D:/cargo-targets/zircon-local/evidence/engine-audit-20260930-01a0f06a/m1-tool-workspace-owner-path-20261004-r1/postimage.json)。自有验证 runner 同步新入口，生产验证工具未由主代理修改。

正常直连 Cargo 下载在 Schannel `SEC_E_NO_CREDENTIALS` 失败；Python 默认 HTTPS 证书及主机名校验能取得完全匹配 `Cargo.lock` 的归档。只在授权 D Cargo cache 补齐三项实际缺失输入：`brotli 8.0.4`、`brotli-decompressor 5.0.3`、`ttf2woff2 0.13.1`，逐项校验锁定 SHA256 与读回值，保留原失败回执。没有全局代理、TLS 校验关闭或 C cache 修改；见 [两项 Brotli 归档](D:/cargo-targets/zircon-local/evidence/engine-audit-20260930-01a0f06a/m1-current-validation-20261004-r4/runtime-locked-brotli-archives-r1.json) 与 [ttf2woff2 归档](D:/cargo-targets/zircon-local/evidence/engine-audit-20260930-01a0f06a/m1-current-validation-20261004-r4/runtime-locked-ttf2woff2-archive-r1.json)。这些准备不等于网络路径、编译或测试验收。

[Runtime libtest 编译 r5](D:/cargo-targets/zircon-local/evidence/engine-audit-20260930-01a0f06a/m1-current-validation-20261004-r4/runtime-test-compile-r5-dev-entry-offline-receipt.json) 已实际终止于 exit 101：328 条 compiler artifact、6 条 Interface 编译错误、测试为零。20,400 个所选输入前后未变，不封存新路径或外部 sibling。最早错误是生产生成器被迁到 `zircon_runtime_interface/tests/unit/build.rs` 后 Cargo 没有构建入口，两个 `OUT_DIR` 缺失导致后续生成符号缺失。

主代理准备了生产 `build.rs`、独立六项 unit tests 与真实生成器测试引用的三路径修复候选，但应用前观察到共享源码已经完成相同生产入口修正。主代理没有覆盖或认领该源码；[当前重叠观察](D:/cargo-targets/zircon-local/evidence/engine-audit-20260930-01a0f06a/m1-interface-build-owner-20261004-r2/current-observation-before-root-apply.json) 记录实际三个后像、未知外来 owner 和仅 JSON fixture 缩进的候选差异。新的 `r6-current-layout-offline` 原进程正在执行，日志已记录 Interface custom-build 和 lib 产物，但 Runtime 测试目标尚未返回 terminal，不能称整包通过。

[新输入清单](D:/cargo-targets/zircon-local/evidence/engine-audit-20260930-01a0f06a/m1-current-validation-20261004-r4/source-before-r3.json) 用当前可见 Rust/Cargo/JSON 路径与独立验证入口重新选取 23,680 项输入，下一命令逐项重读；它仍不提供 index、ignored output、submodule 或 zr_vm 整树 seal。App 的真正 tests owner 已迁到 `lifecycle/tests/cases.rs`，旧路径物理窗口候选保持待验收，必须按真实模块挂载重做后再复核。

R1 Dynamic/Core 的 [当前独立源码复核](D:/cargo-targets/zircon-local/evidence/engine-audit-20260930-01a0f06a/m1-dynamic-core-shutdown-behavior-source-review-20261004-r1.json) 没有具体源码 blocker，仍不提供实际 join 或行为通过。Gilrs 第二版候选还存在未处理 `Message::Open` 裸设备释放边界，第三版候选正在修复；自线程 join 与原生驱动期限仍开放。SystemState 的 wrong-World run/rebind 与 Query 缓存身份正沿已有 World 运行期身份 owner 只读设计。所有当前编译、行为、MVP 与性能门继续按实际结果推进。
