# Runtime11A · UiSurface 子树提交完成列表

日期：2026-10-09。对应[优化记录](../../../optimize/zircon_runtime/11a/2026-10-09-transactional-child-mount.md)。

## 已完成源码工作

- [x] 原 UiSurface 内公开准备/提交工作区子树，复用同一 tree/style/binding/input/session owner。
- [x] 提交前核对实际公开状态，陈旧或失败候选保留 live 工作区。
- [x] 成功路径保留最新 private session/secure stores，Debug 不展示捕获状态。
- [x] 接入真实公开 stale commit 回归，提供原 Runtime #2 的同配置筛选入口。
- [x] 随 Module 的实际 19 文件接入通过独审、前后像、依赖、apply/fmt/diff 检查，未修改共享 index/control。

实际 receipt SHA-256：`5f77fb1c2fb16a4085c8c4ff870eb535a93e7e1101e91b4524847493217e6a45`。

## 待实际验收

- [ ] 真实 Runtime 公共提交回归及 Editor 八项回归执行成功，数量/名称与 receipt 匹配。
- [ ] 实际产品多 Surface、输入、窗口、IME、Accessibility 与清退合同按 Runtime11A/200 原计划通过。
- [ ] 同来源、同负载 before/after CPU/RSS/allocation requests/bytes 与完整延迟分位数达标。
- [ ] 原产品、完整矩阵和全 workspace 接受门完成。

此源码切片未完成独立 Runtime 产品 UI driver、完整 ABI 派发或 OS Accessibility 的全部后续实现。
