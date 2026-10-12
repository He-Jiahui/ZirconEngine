# Editor01：UiAsset 预览修复计划完成列表

日期：2026-10-09。[优化记录](../../../optimize/zircon_editor/01/2026-10-09-ui-asset-builtin-preview-lifecycle-record.md)。

- [x] 21 个既有内置预览的真实目录、物理 Source 闭包及默认属性接入。
- [x] 共用 retained-pane 输入接口和单一 Surface 输入 owner 源码采用。
- [x] 跨仍可见 pane 先取消旧 Session；隐藏、零 content、焦点丢失及 reopen 源码生命周期修复。
- [x] 完整 48 文件独立源码审查和实际采用；255 依赖、47 Rust 格式及补丁结构检查通过。
- [x] 四个真实回归用例加入统一 15 项批次草稿；保守第 15 项预期为 140。
- [ ] 编译及四个原生回归用例通过，核对真实名字及实际非零数量。
- [ ] 21 个真实 preview 的同画面 paint/hit、normalized input、capture / IME / retire 原生通过。
- [ ] 原全量组件 offender-empty 和 139 个公开事件断言通过。
- [ ] 原性能分位数、allocation / CPU / RSS / I/O 预算与 Editor/Runtime 实际产品验收达标。

状态：源码完成，原生、性能和产品验收待完成；目标未完成。
