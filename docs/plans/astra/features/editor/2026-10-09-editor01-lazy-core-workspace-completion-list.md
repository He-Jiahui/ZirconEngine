# Editor01 · 核心工作区按需挂载完成列表

日期：2026-10-09。对应[优化记录](../../../optimize/zircon_editor/01/2026-10-09-lazy-workspace-child-mount.md)。

## 已完成源码工作

- [x] 实际接入十个核心模块的公开 Window 选择、所选文档加载、原 UiSurface 子树替换与返回 Scene。
- [x] 默认工作区取消提前保留核心模块，切换清理旧节点/输入路由并复用有界 ID。
- [x] 接入真实来源约束下的 pane 投影、绘制和命中，以及失败/陈旧提交原子性。
- [x] 增加八项 Editor 与一项 Runtime 回归源码；保留 139 项原事件的逐项控件及派发断言。
- [x] 独审接受、实际 19 文件及四新路径/85 依赖守卫、apply/fmt/diff 通过，共享 index/control 未变。
- [x] 定义合入原 Runtime #2 / Editor UI #15 的入口，保留原 15 配置及性能标准。

实际 receipt SHA-256：`5f77fb1c2fb16a4085c8c4ff870eb535a93e7e1101e91b4524847493217e6a45`。以上是源码完成项。

## 待实际功能及验收

- [ ] 批量 native 成功执行八项新增 Editor、一项新增 Runtime 和所有既有相关用例，核精确名称、非零数量与实际来源。
- [ ] 六个 Tools 弹出菜单、43 个对应扩展页面及额外 Particle Library 入口的真实选择、可见页面、paint/hit/typed input/retirement。
- [ ] 原 96 项强静态可达性门及需要另证的动态消费合同；不通过添加登记根或无授权资产退休改变标准。
- [ ] 真实 Editor exe + Runtime dll 的完整产品 F0、Window 导航、IME/焦点/键盘/指针及退场验收。
- [ ] 原 UI CPU/RSS/分配、p50/p95/p99/max、像素、帧及同配置 before/after 标准达标。

编译接收、派发返回值、fmt 和来源哈希均不关闭上述验收项。
