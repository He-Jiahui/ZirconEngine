# Runtime 第三批异步验证完成列表

日期：2026-10-09。状态：源码守卫拒绝，Cargo 未启动；全部测试与原始性能验收仍未执行。

## 当前修复范围

- Runtime02：已有任务调度与 loader 所有者修复、EventBus 空 displacement 预检；执行原 scope/loader/reload/事件回归与 Release producer。
- Runtime03：已接入借用父路径、仅计数 workload 线程的嵌套分配 probe、owned frozen profile export；分别验证 profiling、profiling-chrome，并保留原 nonnested/streaming/Perfetto producer。
- Runtime51：已有选择性查询优化与权威顺序回归；执行原 unfiltered/G39 producer。
- runtime/editor 边界、owned result 和 app F0 diagnostics 一起提交。F0 diagnostics 测试不代表实际反复产品启动验收。

原数值门槛、规模矩阵和 feature 配置未下调。新 probe 的 after 单侧执行不能证明前后收益；Runtime03 去除一次 snapshot clone 的源码因果也不能代替实际性能达标。

## 批次与证据

- 请求：`runtime-editor-source-repair-batch-20261009-03`，15 项命令，337 个具名输入及全部主包文件清单守卫。
- worker 启动：`2026-10-09T10:33:27.3700407Z`，PID `31764`；此身份只证明启动，Cargo 执行结果等待终态回执。
- manifest SHA256：`60bb0606a90864d28f1088b21b869d431ebf3df359a67e162914e327922fbfe0`。
- 回执目录：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v3`。保留 `source-binding-preparation.json`、`launcher-identity.json`，完成后核对 `terminal-receipt.json` 与实际日志。
- Jenkins 激活文件缺失。本次通过现有 `tools/dev/local_cargo.py` 独立入口启动；Jenkins、里程碑与产品验收仍未通过。
- 所有编译输出、缓存与临时目录仍受 `D:/cargo-targets/zircon-local` 物理路径检查。没有修改 tooling、生成源码副本或更改控制配置。
- 不等待或持续监控编译。编译期间准备后续功能修复候选；在实质审查节点一次核对已有终态。源码绑定请求运行期间不接入新候选。

## 验收规则

- [x] 当前源码、历史接入回执、原始性能 producer、工具链和共享索引已绑定。
- [x] 已启动单批 15 项 worker，历史失败批次和拒绝接入证据保留。
- [ ] 实际 Cargo 命令执行成功，所有选中的测试真实匹配并通过，源码和输出守卫成立。
- [ ] 原始 Release producer 的具名用例、标记和原始矩阵完整；零退出码不能单独满足此项。
- [ ] 完成原计划要求的相同配置前后性能比较及产品验收。

上一批 `runtime-editor-source-repair-batch-20261009-02` 的首项 Cargo 返回 101，后续 11 项未执行。它没有测试或性能通过证据；本次启动不会覆盖该事实。

## 后续功能工作

- [ ] 审查选中模块的 lazy subtree mount 与 Runtime 所有者、状态退休契约；当前批次不能认证尚未接入的候选。
- [ ] 在终态完成后按具体失败修复，并将下一组修改合并为新的源码绑定批次。

关联：[Editor 同批验证](../editor/2026-10-09-editor-batch03-native-validation-list.md)。

## 批次最终拒绝（2026-10-09）

- `prelaunch-refusal.json` 在 `2026-10-09T10:33:28.309263+00:00` 记录 `source-guard-refused`、`cargoInvoked: false`。
- 绑定后变化的具名输入是 `Cargo.lock`、`zircon_runtime/Cargo.toml`。保留共享修改，没有回滚或修改控制配置。
- 拒绝回执 SHA256：`c3fd91500b5355b3b2107e682b6d7b23bea84494315fa7c8dacdc0b9acd3d864`。全部 15 项 Cargo 命令未调用，因此没有编译、测试或性能通过结果。
- 在完成逐事件源码核对与原 producer 矩阵核对的实质节点读取一次终态；无编译等待、周期监控、PID 查询或持续日志跟踪。
- 该请求保留为已拒绝历史。继续候选独审与真实功能修复，接入后按最新源码合并下一批验证；新请求不覆盖本次失败。
