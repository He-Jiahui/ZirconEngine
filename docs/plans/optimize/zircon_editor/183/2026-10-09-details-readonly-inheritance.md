# Details 只读与宽度继承回归修复

日期：2026-10-09。状态：三个测试源码文件已实际接入；原生执行和性能验收仍开放。

## 修复内容

原 Bootstrap 回归错误要求九个轴字段全部具备编辑事件。现有业务合同是六个位置/缩放字段可编辑，三个旋转字段显示只读的 local XYZ Euler degrees。按[Inspector 原评审](../183-inspector-property-grid-review.md)保留这一合同；未来 compound rotation editor 的独立 owner 和验收仍开放。

修正现有 Bootstrap 断言，保留原九个用例及九项宽度检查。新增一个隔离回归，沿真实资源导入、prototype preparation、编译、retained projection、snapshot getter 与 bridge sync 验证九项 token 宽度和六项完整 Change/Submit 绑定。

三个旋转字段使用实际可用的 `Some(rotation_degrees)` 同步值，并在同一 surface 上先验证可编辑字段的正向焦点。通过现有 dispatcher 消费正向 focus release 合法产生的 Submit/IME 生命周期后，再验证三个只读字段的严格交互拒绝。没有清空内部队列或伪造旋转编辑事件。

生产 property、event 和现有 host 只读负向门未变化。v1 的 all-nine 编辑误合同及 v2 的正向生命周期污染问题保留为 `request_changes` 历史，实际接入的是独审接受的 v3。

## 接入证据

- 三个精确源码后像匹配，54 个源码依赖和 17 个真实 resource import 依赖核对一致。
- 独立 review 为 `accepted_source_native_pending`，无未解决 findings。
- `git apply --check`、实际 apply、Rust 2021 fmt 和 scoped diff 检查均 exit 0。
- 本次接入期间控制文件和共享 Git index 的 hash 未变化；未创建源码副本。

补丁 SHA-256：`5d7d18d44852ad28a764d1ff9710ac49ccdbb6fd8eb6938afcd0e1e857af69a4`。

独审凭据：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/editor-details-inheritance-core-review-v3/review.json`，SHA-256 `4cc01cfb49d1d8780d80a7d405cf09cd2a6385a1bbd65fce3c4f2e44fd3b44a8`。

实际接入凭据：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/editor-details-inheritance-regression-v3/actual-canonical-apply.json`，SHA-256 `1d658f72e003208471b4530c68accc21d37f1d24ab443793690a98fe112a234c`。

## 待验证

原九个 Bootstrap 用例及新增 `tests::workbench::layout::details_inheritance::unreal_details_axes_inherit_resolved_width_tokens_and_editability_contract` 共十个指定用例尚未执行。下一批 UI arm 已包含九个 Bootstrap 名称，只需额外加入一个 Details 用例，当前最低选择数从 123 增为 124；后续 Module 用例另按最终实际入口绑定。

原 host 只读/可编辑回归、native geometry/paint、原 Release producers、完整产品和性能标准均保持开放。本次没有性能测量，源码与格式证据不计为测试或性能通过。

对应[计划完成列表](../../../astra/features/editor/2026-10-09-editor183-details-readonly-inheritance-completion-list.md)。
