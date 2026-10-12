---
doc_type: implementation-record
created: 2026-10-11
---

# 2026-10-11 Runtime 完成与缓存维护状态

## 当前运行与托盘

当前 driver 为 `7b94004801313ed7d03b47ba1e4f8b6c9aa4e076ec01e4aed412cf4d458480f5`，runtime 启动身份为 `start-1791708207030401800`，管理地址为 [http://127.0.0.1:18080/](http://127.0.0.1:18080/)。Ready 实测确认 controller executor 0、唯一 agent `zircon-windows-agent` 在线且 executor 1，79 个插件检查通过；三类 job 的固定节点标签、runtime operation 和 driver digest 均逐项核对。回执为 `.jenkins/state/migration-baseline/usable-runtime-observation-1791708265262573300.json`。

首次固定节点切换取得旧服务 public Stop 的实际进程终止证明，随后退役已停止的 keeper，旧托盘通过公开 Exit 菜单退出。该版在复核历史 interrupted request 时阻断：读取器没有沿已验证的 handoff 追踪原请求。修复通过完整回归后，公开 manager Stop 取得该版真实 NativeJob 终止及 EOF；原 blocked descriptor 和 committed command 保留原文，再以核验过身份的 Win32 handle 退役旧 keeper。独立的 `external-native-stop` 迁移契约记录真实停机来源，未把它伪装成旧 keeper 执行的 Stop。

新 keeper 已实际提交启动，admission 为 `idle`，历史请求未重放。桌面 `WinSta0\Default` 上的新托盘实际可见，右键回调和原生菜单弹出通过；回执为 `.jenkins/state/migration-baseline/desktop-tray-observation-1791708289821103700.json`。切换前后的两次启动均复用同一固定 node。

本记录只固定当前运行证据和维护边界，不关闭 M6–M8、整产品或生产验收门槛。

## 支持层与 M5

当前部署版本的完整支持层回归收集 1100 项，其中 1098 项通过、2 项因环境无法创建目录链接而跳过，失败及错误均为 0；官方 `zircon-strict` 沙箱运行，源码身份前后保持一致。报告为 `.jenkins/state/migration-baseline/patch-support-parallel-regression-1791707726638391000.json`，部署 driver 封存自同一份已验证源码。首次固定节点版本的 1077 项报告及旧的成功、失败记录全部保留。

固定节点版本实测 M5 三个 Session 使用同一输入和 execution `dc621ce1627ec57ae026aed803e8e173`：A consumer 已 released/cancelled，并通过原请求 reconcile 保留源码；B/C 共用一个执行者，完成真实 Cargo 两项测试并获 formal acceptance。Jenkins flow `#47/#48` 均为 SUCCESS，编译、测试列表和执行的原生进程退出及输出 EOF 逐项核对。功能验收回执为 `.jenkins/state/migration-baseline/public-m5-1791706144162367300/functional-acceptance-1791706595143176800.json`。旧版 `#45/#46` 证据保留；这些结果不代表完整 M5 milestone 或后续里程碑已完成。

## 维护 #3、#4 与 #5

维护 #3 在 pre-intake 失败，原因是 `request_payload_mismatch`：`cleanupScope` 丢失 `relativeSubtree`。因此没有 intake、没有 deletion，reclaimed 为 0。原 request 与 public cancel 均保留为 `cancel_requested`，`finalCancellationProven=false`；保留 `relativeSubtree` 的 Groovy 修复已包含在当前封存 driver。

固定节点上的维护 #4 在 pre-intake 返回 `invalid_control_input`，同样没有 intake、deletion 或回收量。官方 strict 下的实际只读诊断确认，缺失 blob 的检查误以写权限打开 `D:\cargo-targets` 根；该权限请求被拒绝。只读模式的最小修复已通过实际路径观察、61 项相关测试及完整回归，已随当前 driver 部署。原请求已通过 public cancel 保留为 `cancel_requested`，原失败、历史资源及四份受保护记录完整保留；#4 仍记为失败。

当前版本的维护 #5 为 Jenkins `SUCCESS`，受管 GC 记录为 `complete`；在核验过的单一叶子目录实际删除 2 个单链接增量文件，记录回收 `1,181,871,000` 字节（约 1.10 GiB）。该请求的原队列、参数、构建结果和删除记录逐项对账；四个历史 holds 及缺失对象的受保护原始行均保持不变。回执为 `.jenkins/state/migration-baseline/public-build-cache-gc-1791708320187721800/observation-1791708429974102100.json`。这次有限回收不授权删除其他历史构建池。

## 缓存候选与保护

只读复核定位到 `D:\cargo-targets\zircon-local\dependency-fix`（约 48.354 GiB）和 `E:\cargo-targets\zircon-local\win32`（约 14.764 GiB）。两处均没有 managed generation 或 native termination/owner proof，当前不得删除或自动 adopt；D 目录下一处叶子已确认存在 hard-link，E 本次未采样 hard-link，不扩大该结论。四个 historical active holds 保持不变。

## 节点与验收边界

本机已部署固定唯一正式 node `zircon-windows-agent`，节点名和标签不再随 runtime 变化；runtime operation、driver digest 和原生进程身份仍独立校验。经 exact XML review、offline/idle 与空队列核对，13 个历史随机节点已通过 Jenkins API 移除，原固定节点配置已更新。14 份原节点的 API XML、磁盘 XML 和身份 metadata 在变更前完整保存在 `.jenkins/state/deployment/retired-agent-nodes/`，任务与构建记录保留。之后启动复用同一固定 node。

维护完成后的最后复核确认唯一 agent 仍在线、controller 0 槽、keeper admission `idle`，原生进程身份、可见托盘及四份历史 holds 均与已有证据一致。统一观察回执为 `.jenkins/state/migration-baseline/fixed-single-agent-completion-1791708591741233200.json`；它引用当前 Ready、实际菜单、完整回归、受控维护及前版固定节点的真实 Cargo 功能回执，不改变正式产品验收门槛。

M6–M8、whole product 和 production 仍未通过；本记录不把真实 M5 功能证据扩大为完整里程碑通过。

相关入口：[单机协调主计划](../01-single-host-jenkins-coordination.md) · [Jenkins 托盘说明](../../../tooling/jenkins-tray.md)。
