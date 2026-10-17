# Runtime08c 动画帧需求修复记录

日期：2026-10-15。任务：`optimize-20261013-01a13451`。对应原计划：[08C Animation Runtime](../08c-animation-runtime-review.md) 的 P1-2。

## 源码结果

本次修复限定于 17 个现有文件。WorldDriver 每次 tick 建立带 replacement epoch 和唯一 token 的需求事务；求值器只向该事务暂存连续帧需求。成功 tick 发布当前需求，没有求值器时发布 false；错误或 unwind 清除本次求值需求，旧 epoch、旧 token 的迟到发布被拒绝。显式主机需求与尚未消费的事件仍独立参与 OR。

动画插件沿既有采样路径和同一个 ResourceSnapshot 计算活动标志，将结果通过 clip worker、graph、state machine、layer 和 nested sample 的原有返回值传回 tick。该路径没有新增全场景扫描、第二次资源读取、深克隆、TLS 或独立后台任务。保留原有采样、事件和 worker join 顺序；实际 ready、enabled、playing、有限非零速度、有效权重及会继续推进的状态才产生连续需求。pause、速度零、非有限速度、Once 结束、disabled 和资源不可用按实际结果回到 idle。

## 验证与归因

- 源码绑定：[runtime08c-animation-frame-demand-v1-source-binding.json](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime08c-animation-frame-demand-v1-source-binding.json)。
- 原始增量补丁：[runtime08c-animation-frame-demand-v1.patch](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime08c-animation-frame-demand-v1.patch)。
- 根会话独立原始字节证明：[runtime08c-animation-frame-demand-v1-static.json](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime08c-animation-frame-demand-v1-static.json)。
- 独立源码复核：gpt-6-luna max，PASS_static_only，未发现可操作 P1/P2；最终只读会话 32305 实际 exit 0 并已关闭，系统 PID 未暴露（UNKNOWN）。。
- 已登记范围为 17，逐文件 before/after 原始 SHA-256、完整补丁正向/反向及 HEAD/index 证据见绑定和静态记录。混合 CRLF/LF 文件保留原始未修改片段。
- 保留参数测试预先存在的修改，其作者及原因 UNKNOWN；它在本次真实 before 中，不能归为本次修复。历史工具失败和未暴露的进程身份仍按原始证据保留。
- 仅源码库存：原选中 68 ordinary，新增 5 后为 73。plugin lib 原 92 ordinary / 24 ignored 保留，`physics_tick` 源码 40 → 42。参数测试实际命名空间是 `evaluation::pipeline::parameter_apply::tests::`。这些数字不表示原生测试运行或通过。
- 普通可用 Python 为 3.6.8，不能执行要求 Python >=3.11 的 scoped guard。当前静态记录仅为原始字节与源码复核；最终 scoped guard 必须在受支持 Jenkins job 中对实际最终路径并集执行，任何 finding 均须处理。尚未给出 guard/native/performance PASS。

## 合批交接与后续验收

Runtime 测试可复用既有 `target-editor-host,test-support` 配置；animation 的普通 lib/physics_tick 与 Runtime55 兼容配置归并，Release ignored 的原 producer、样本和阈值独立保留。Jenkins 交接将在 Editor02 实际调用链收尾和完整源码冻结后合批进行。提交后继续其他功能修复，不轮询编译、队列或 response。

共享文件 `zircon_runtime/src/scene/tests/level_system_frame_state.rs` 后续由 Editor02 在明确切层后迁移生命周期回调。其 before 必须是本次冻结 after，最终绑定保留两层作者与原始哈希；本补丁不能用于冒充迁移后 checkout 的独立 guard 结果。

以下验收保持 OPEN：

- 原计划 P1-2 的 App、Editor reactive cadence 真实播放/暂停/结束/卸载和无 busy loop 证据；完整版本化 demand snapshot、任务、backlog、next wake 的原目标。
- 30/60/120Hz、fixed/variable、pause/seek/loop 下的真实 provider 正确性。
- 1k 角色同语义同硬件矩阵、各阶段 P50/P95/P99、分配和缓存/驻留数据，以及一小时 loop/transition/event/reload soak。
- 原 compiled-channel、graph 等 Release producer 的全部原始样本、比例、零分配条件和现有内存门槛。
- 实际全部 compiler inputs、feature、lock、toolchain、环境的编译前后 seal，原生 Rust/Windows、产品与平台验收。

本次未创建本地编译产物、缓存、临时脚本、生成夹具或源码副本；实际释放 0 B。保留源码归因、失败回执、静态证据和后续原生验收所需报告。未知归属及历史 UNKNOWN 不能称为清理关闭。
