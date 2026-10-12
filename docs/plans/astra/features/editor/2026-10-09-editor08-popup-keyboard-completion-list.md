# Editor08 键盘弹出目标完成列表

优化记录：[键盘弹出目标借用行数据](../../../optimize/zircon_editor/08/2026-10-09-popup-keyboard-borrowed-rows.md)。

- [x] option/menu 目标借用 ModelRc 行，只复制返回字段并提前拒绝无效几何。
- [x] 保留精确 action/UUID、输出所有权、源行 frame、裁剪和 hover/focus/selection 优先级。
- [x] 原 dispatch 用例完整保留；新增两个调用真实 builder 的公共回归。
- [x] 独审、源码实际接入、格式/diff 与 LF 内容哈希校验。
- [ ] 后继合并批次原生编译、相关原测试与新回归通过。
- [ ] 原 20%/40% Release 预算实际通过，本次行借用的分配/耗时收益量测。
- [ ] 原 Editor08 全规模 catalog、owner、Unicode、分配、延迟及产品验收。

前一 12 命令批次已因 Editor 测试契约编译错误退出 101，后续测试未调用；失败证据保留，源码根因修复与本补丁合并后再批量验证。测试与性能状态保持开放。
