# Editor09 主窗口关闭计划完成列表

日期：2026-10-17。任务：optimize-20261013-01a13451。状态：源码已写入，验收待完成。

- [x] 在实际 checkout 写入 40 个 UI 与依赖源文件，接入持有真实 Session 的异步主窗口关闭操作。
- [x] 接入场景/文档精确关闭能力及最终保存所需的真实服务调用 owner。
- [x] 捕获前取得单文档独占许可，并在保存负载持有期间按实际 Vec.capacity 记账。
- [x] 接入真实 autosave/jobs/settings 终态轮询；Pending、失败及期限到达继续保留 owner 与唤醒路径。
- [x] 写入两个主窗口行为回归；Core v2 的十一例、Runtime/Settings 的五例和既有输入夹具修复共同合批路由。
- [x] 保留并发 app.rs 外观改动、历史 source receipts、失败证据及各自归属。
- [x] 独立审查确认 decision/Cancel 生命周期 P1，并实际写入两个生产 owner 的窄修；原证据保持不改。
- [x] 写入三个真实 admitted Session 回归，核实激活、heartbeat、编辑能力及 Pending/Cancel 竞争；独立增量窄审 SOURCE_ONLY PASS。
- [x] 中央 typed legacy commit gate 保留 owned close 的 Session 释放权及 Complete 后清理。
- [x] 修复两项旧项目关闭测试的文本锚点，保留七个旧测试身份和原计数门槛。
- [ ] 完成当前完整源码 union 的公共 change guard 与后续批次环境绑定。
- [ ] 原生编译与全部必需 named/subset/floor 用例通过；当前均为 UNRUN。
- [ ] GUI 关闭期间响应、保存/放弃/取消、错误保留及真实退出行为验收通过。
- [ ] 序列化峰值、RSS 与 P95 实测通过，并满足既有 22 个性能组和 Runtime03 绝对及相对门槛。

源码记录：docs/plans/optimize/zircon_editor/09/2026-10-17-owned-nonblocking-main-window-close-source-record.md。

UI source receipt：.codex/outbox/reports/optimize-20261013-01a13451/editor09-owned-nonblocking-main-window-close-v1-source-write-receipt.json，SHA256 6e4e0c6ac56890c4af2addd2dd82a3c11414c851ed12d27b6427f3e4760fd874。实际 writer PowerShell 32084 / Python 37628 / fff957，outer exit 0。落盘交接不代表 Jenkins 已读取或验证通过。

没有运行自有构建或创建源码副本；保留登记证据，释放空间 0 B。本列表未关闭 Editor09 计划或总目标。

当前记录包含历史 40 路径及明确增量，共 44 个路径；不是同时点全量校验。最终 7 owner pins 见源码记录。reviewer 复用 root676/root677 及作者 cell41/cell44 实际源码证据，没有新 OS reader 成功声明。三个 Session 夹具未创建或运行，native、GUI、公共 guard、RSS/P95 均未通过。
