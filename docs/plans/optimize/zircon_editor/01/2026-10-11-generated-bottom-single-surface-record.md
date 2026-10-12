# Generated bottom 单一真实 surface 优化记录

日期：2026-10-11

## 已采用的代码

完整 004 候选 `5357513bfd3d1939292fdc194399bbaa28552a67d2df111bae8b0ed55e58a4dd` 经独立审查后一次采用：38 文件，37 Rust、1 ZUI，其中 34 个既有文件、4 个新文件。保留此前真实采用的 ui.rs 生产导出修复；其他 34 个 003 后像逐字节相同。三份遗漏调用文件共四处参数改用实际 typed Default context，经真实模块导出链调用，无生产兼容 shim。

Generated pane 的来源、paint/hit、输入和退休断言围绕同一实际 BuiltinPaneSurface。保留真实 source 闭包的完整有序字段比较、46 个 authored 事件关联及隐藏后复用/重新打开/CloseView 退休合同。这 46 个关联是 45 个控件上的事件，不能计为 Rust 用例。

## 实际检查与测试数量

142 个当前 raw/LF/大小输入和四个新文件缺席条件在采用前核对。采用后 38 个 LF 后像精确、108 个只读输入不变。四个增量文件实际 rustfmt check 通过，其他 33 个 Rust 后像复用完全相同的既有格式证据；限定 whitespace、四个新文件 no-index whitespace 和完整 patch reverse-check 通过。

真实正常用例新增筛选五个：四个既有、一个新 Rust 用例。四个新 Rust 文件中仅一个定义新增测试；原五个 chrome ignored 测试保持。原 V9 第 15 项未选择这五个名字，V10 已加入各自完整名字及 required 名称字段。与十二个 paint/hit 用例合并，最低数量由 174 调整为 191。

[实际采用收据](../../../../../.codex/outbox/editor-generated-drawer-single-surface-v1/complete-candidate-004-v1/actual-canonical-apply-v1/actual-canonical-apply.json)  
[独立审查](../../../../../.codex/outbox/editor-generated-drawer-complete38-independent-review-v1/review.json)  
[完成列表](../../../astra/features/editor/2026-10-11-editor01-generated-bottom-completion-list.md)

## 待验证

尚未运行 Cargo、原生用例或性能。下一次批量验证保留原 15 项和 14 个性能 producer，在终态逐项核用例名、最低数量、原完整矩阵及阈值。代码采用不等于完整组件覆盖、真实产品或 Jenkins 通过。
