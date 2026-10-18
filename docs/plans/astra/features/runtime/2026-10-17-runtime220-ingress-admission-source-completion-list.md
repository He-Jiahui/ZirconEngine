# Runtime220 输入接纳与消费者终结计划完成列表

日期：2026-10-17。任务：optimize-20261013-01a13451。状态：源码已写入，原生和性能验收待完成。

- [x] 在真实 InputModule 注册 InputDriver，由真实 Session 持有并核验精确 attachment 凭据。
- [x] 保持 DefaultInputManager 为唯一物理状态 owner，保留原物理输入先于 UI/camera 的顺序及 InputConfig.enabled 含义。
- [x] required Rust 批次接口在同一状态锁内完成整批预检后再修改状态与 recorder。
- [x] 以 checked 算术统计实际 Vec/String 拥有容量，接入计数、容量和 runtime 接收年龄 policy。
- [x] 超载触发保留的真实 FocusLost 重置并进入 Gap，拒绝后续自动重新接入；Touch 双事件按原子子批次提交。
- [x] 在 17 个实际源文件写入 12 个行为回归，并完成正反 delta、fsync 和双次原始字节读回。
- [x] 最初 17 源 / 12 回归完成限范围核心语义审查 SOURCE_ONLY PASS；保留原 receipt 与审查来源，后续增量另行登记。
- [x] 实际核实十二例父模块与两个 Session 的完整 FQN；追加到同一后续 runtime 命令的清单已交接，编译与运行仍 UNRUN。
- [ ] 完成当前 union 公共 guard、原生编译、16 个 Runtime220 新增 named AND 和全部旧 floor/subset/config 要求；当前均 UNRUN。
- [ ] 实测时间、内存和锁等待并达到相应门槛，满足既有 22 个性能组及 Runtime03 绝对/相对/RSS 条件。
- [ ] 实现并验收原 M220.1/M220.2 与完整 P0/P1 所需的 host-qualified 协议；V1 无 host 时间、source epoch/sequence、seat/user。

源码优化记录：docs/plans/optimize/zircon_runtime/56/2026-10-17-runtime220-ingress-admission-source-record.md。

Source receipt：.codex/outbox/reports/optimize-20261013-01a13451/runtime220-session-ingress-admission-source-write-receipt.json，176113 bytes，SHA256 d538df75fea031c51d900d95909d644270bc53e9b9d0149fafabb9b3ce2ac5e8。实际 writer PowerShell 33476 / Python 49688 / 013d67，outer exit 0。root671 曾在其实际读回时点机械核对 17 个 after；初始增量语义审查已完成，后续实际源码由明确增量覆盖。原生结果仍待完成。

本轮没有自有构建或源码副本；保留 source receipt 与必要失败证据，释放空间 0 B。此列表不关闭 Runtime220 全计划或总目标。

M220.1a 是本地工程阶段标签，原计划没有 M220.1a/M220.1b 子里程碑。原计划的 M220.1、M220.2 及完整验收继续 OPEN。


## 消费者终结增量

- [x] 在 10 个实际产品文件终结 camera drag、全部 UI capture/press/交互 timer 与 IME；保留 toast/typeahead 内容过期。
- [x] 限定 Accepted FocusLost、精确 FrameCapacity/attachment 和精确 detach；其他 Rejected 不重置，Gap/Detached 在 UI-first text/IME 前拦截。
- [x] 修复精确 detach 后 fallible cleanup 的 localDetached 状态，以及 window pump 失败后未交付 host release 的责任丢失；pending 重试只触及本 Session 所有字段。
- [x] 新增 4 个真实 Scene/IME/UiSurface/MissingNode 回归，更新 1 个已有 gamepad 用例的 mutable/typed 接口；身份、旧断言和原 floor 谓词保留。
- [x] 14 个最终 after 已实际机械核对（PS47796/Py48372/7152c5，outer0），原17 receipt 不变；Sol28 完成限范围 SOURCE_ONLY PASS。
- [ ] Jenkins 重新绑定实际 checkout，执行公共 whole-union guard、同一批次原生测试/GUI/性能；Runtime220 共16个新增用例及修复的已有用例均 UNRUN。
- [ ] 完成原 M220.1/M220.2，尤其 V1 host wire 跨 Session/seat、host timestamp/source epoch/sequence 的完整资格及实测 RSS/P95。

独立审查依据作者实际 A–D/callee/四个真实测试和最终 pins；没有新增审查者 OS reader。两项 confirmed HOLD 已在源码修复，不代表原生/性能通过。早期 apply 终态 UNKNOWN 和 precreation reader 拒绝历史保留。最终14文件 raw映射和精确回归清单见上方优化记录及同一 Jenkins successor 交接。

交接不启动或轮询编译；保留31 Cargo +2 Python、旧门槛和所有性能条件。实际释放 0 B，保留已登记交付和失败证据。

