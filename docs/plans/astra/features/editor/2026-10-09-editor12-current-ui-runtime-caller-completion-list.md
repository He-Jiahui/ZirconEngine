# Editor UI：当前资源调用合同完成列表

日期：2026-10-09。以下完成项仅覆盖实际源码接入和结构证据。

## 已完成

- [x] 按当前真实 loader、资源 kind/root/import 和现有组件目录修正 11 个测试源码文件。
- [x] 保留原有 47 个活跃用例、五个退役命令别名拒绝检查和强资源可达性断言。
- [x] 核对 21 个 canonical 命令身份及现有 Showcase 四 section、57 个原生类型来源。
- [x] 独审确认当前外部任务的 palette、几何与非持久 FileCache 路径兼容。
- [x] 完成 273 个依赖的前像核对、11 个后像核对与守卫接入；Rust 2021 fmt/apply/diff 检查均 exit 0。
- [x] 保留控制文件、共享 Git index 及其他任务改动。
- [x] 写入[优化记录](../../../optimize/zircon_editor/12/2026-10-09-current-ui-runtime-caller-contract.md)及真实接入凭据。

## 尚未完成

- [ ] 三包原生 compile/typecheck 通过，47 个指定合同用例实际运行并通过。
- [ ] 强资源可达性门剩余 96 项得到真实 owner 修复；原断言通过。
- [ ] 模块按需挂载、反复切换和旧 owner 退休，以及真实绘制、命中、dispatch 回归通过。
- [ ] Details 继承宽度与六项可编辑位置/缩放、三项只读旋转的真实 consumer 回归通过。
- [ ] 动态主题、键盘焦点、popup geometry/selection 等产品功能门通过。
- [ ] 原 Release producers、完整性能矩阵和原性能门槛达标。
- [ ] 核对匹配 request 的终态 source/output guards、实际 test names、raw producer 数据及 Jenkins/产品 acceptance。

实际接入凭据：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/editor-ui-runtime-caller-contract-fix-v2/actual-canonical-apply.json`，SHA-256 `39f86c7e22d48033daad71eaa22a5596a39a5ebc2685c74117d7ec6dd03953f1`。本次没有执行 native job，未把源码和格式检查计为测试或性能通过。
