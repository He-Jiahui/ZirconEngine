# Editor08：键盘弹出目标借用行数据

对应[原优化计划](../08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md)。键盘目标构造的源码优化已接入；原生与完整性能验收待实际结果。

## 已完成的代码修复

- `option_popup_keyboard_target` 与 `menu_popup_keyboard_target` 用 `ModelRc::get` 借用行数据，替代 `row_data` 的整条记录 clone；只复制返回目标需要拥有的字段。
- disabled/separator/empty action 行保持拒绝；geometry 在输出 String 复制之前求值，裁剪失败的行不再先分配输出文本。
- 保留源行索引对应的 frame、窗口裁剪、asset/workbench dispatch、精确 action/UUID、空 label fallback、hover/focus/selected/special/checked 的原优先级。输出行仍独立拥有全部字段，不借用模型。
- 原两个 dispatch 用例保持字节前缀，新增两个真实 builder 回归验证被过滤行的几何索引、精确动作/identity、source 数据未变、全局选择优先级及 clipped options。

源码因果：所有四个源文本字段非空的可返回行，原整条 source clone 加输出 clone 产生 7 次 String 复制，优化后输出需要 4 次；未使用的 description/tone/raw/shortcut 不复制。此计数排除目标自身、Vec 和 `current_row` 的原 clone，并非实测 allocator、延迟或全流程预算通过。

独审通过，补丁 SHA256 `edd59d9f127762e635e09db68de12c244bac36b9eb7ab9fe58ef7db6b56c7a97`。实际接入与格式/diff 检查证据 `actual-canonical-apply.json`，独审 `editor08-popup-core-review-v1/review.json`，均位于 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/editor-next-core-v2/`。原文件/依赖哈希匹配，索引和控制文件未变。

## 批量验证与剩余门槛

此前请求 `runtime-editor-source-repair-batch-20261009-02` 已终态：首三包 Cargo 检查退出 101，源码未变化。原因是 Editor 测试旧 ZUI 资源引用及旧 ServiceHandle/Result 构造契约，其他 11 命令未调用。该失败发生在本补丁接入前；原失败回执和编译诊断保留，根因修复正在准备。后继批次将合并现有 Runtime/Editor 修复、该键盘公共回归和原 Release producers 后再绑定、异步启动。

原 `EDITOR08_ACTIVE_KEYBOARD_ROW_SINGLE_PASS_BENCH_V1` 的 P95 至少减少 20%、`EDITOR386_IN_WINDOW_KEYBOARD_GATE_BENCH_V1` 至少减少 40% 的断言与原样本保留。这两个既有 producer 验证选择器/窗口逻辑，单独通过也不能证明本次 row clone 的实际收益。原 Editor08 的 10K/100K catalog、冷/热 common-byte/Unicode、owner churn、分配与帧预算仍须完整实际验收。

- [x] 两个生产 builder 源码优化、原测试保留、新公共回归实际接入。
- [x] 独立审查、限定文件格式/diff 与 LF 内容哈希校验。
- [ ] 合并批次原生编译与相关原键盘用例、新回归通过。
- [ ] 原 20%/40% Release producer 门槛实际通过。
- [ ] 本次 row 借用的实际公共工作负载分配/耗时证据及原产品指标达标。

不持续跟踪编译；原生结果在后续实际代码交接点核验。当前没有 Jenkins、里程碑或性能完成声明。
