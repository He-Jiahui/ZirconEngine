# Editor01：UiAsset 内置预览与输入生命周期修复记录

日期：2026-10-09。源码已采用；编译、测试、性能及产品验收待完成。

## 本次修复

- 68 个现有目录描述项保留；20 个既有 sample 和 DropdownPopup 共 21 个预览入口从真实 FileCache 闭包加载，保留各组件默认属性。
- Palette 分页、选择、详情和实际 preview frame 保存在现有 Session / PreviewHost 中。四行投影跟随当前页；保留详情内容。
- 实际 Surface frame 经原绘制转换进入 pane，paint 与输入使用同一变换和实际 content clip。Collapsed、零 content 和非有限尺寸拒绝输入。
- Window 仅保存当前 pane 的路由地址；焦点、捕获、按压和 IME 由实际 Surface 管理。跨两个仍可见 pane 的 Down 先对原 Session 发送 Focused(false)，再更新路由。
- 已提交画面更新、pointer lag Up/Cancel、来源退休、焦点丢失、窗口关闭、pane 隐藏和 reopen 沿现有 owner 处理。隐藏清除实际输入状态；reopen 拒绝旧 frame。
- 共用 retained_pane_input Callback3 接口及原三个 NativeWindow 输入站点，为 Generated 的实际 pane Surface 接入提供已采用边界。

## 采用与结构证据

48 个源码文件一次采用，其中 47 个 Rust 文件和 1 个既有 ZUI。采用前重新核对全部 255 个 raw/LF 依赖、48 个直接前像及两个新路径不存在；采用后 48 个 LF 后像完全相同，209 个未修改依赖保持原值。实际 Rust 格式检查与 git diff --check 均为 0。控制配置、Cargo 文件、共享索引及既有索引锁内容和元数据保持原样。

- 候选 SHA256：`b6ba91f23f2ba29ffabb7879f29dc1f7fc53923cf5fd37a6e1b44dfd8bb9f7e8`。
- [源码独立审查](../../../../../.codex/outbox/editor-ui-asset-builtin-palette-core-review-v7/review.json)与[顺序复核](../../../../../.codex/outbox/editor-ui-asset-builtin-palette-core-review-v7/order-rebind-v8/review.json)：accepted_source_native_pending。
- [实际采用回执](../../../../../.codex/outbox/editor-ui-asset-builtin-palette-v1/order-rebind-v8/actual-canonical-apply.json)，SHA256 `7e946d273716ead4e268b527432dbfb9b92c932a723ee3fb9019dff5b77ff6dd`。
- 首次检查因 Root 额外的索引锁不存在断言停止，未写源码；[历史记录](../../../../../.codex/outbox/editor-ui-asset-builtin-palette-v1/order-rebind-v8/initial-preflight-refusal.json)保留。后续只写工作区并校验锁原样保留。

## 批量验证与限制

四个 Rust 回归用例加入现有第 15 项验证，15 项总批次、前 14 项命令、原 feature/profile 和 Release 生产者与预算不变。第 15 项保守预期由 136 增到 140；21 个 sample 行归属一个 Rust 用例。最终需要原生输出核对真实名字和非零数量。

真实消费者 helper 为 `UiHostWindow::exercise_builtin_ui_asset_preview_consumers_for_test()`，要求调用者持有现有 env MutexGuard。它仅在同画面 Source、paint/hit、输入与退休断言完成后返回 21 个实际选中闭包；初始和最终 authoring 文档不取得组件覆盖信用。

当前没有 Cargo 或协调器提交、性能测量和产品通过结果。原 205 Component、53 roots、全量 offender-empty、139 个公开事件断言及所有原性能阈值仍需通过。Tools 后续在实际两个共享文件上重新生成补丁，Generated 使用实际泛用输入接口。三个旧聚合资产的实体退役仍待真实调用者迁移及产品决定。

[计划完成列表](../../../astra/features/editor/2026-10-09-editor01-ui-asset-preview-completion-list.md)。
