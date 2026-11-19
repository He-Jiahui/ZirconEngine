# Details 数值字段：未输入字符的 Enter 不应改写 World 或清除重做

ZE 固定 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。本项落实[原有 Inspector 计划 E-INSP-P0-01](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_editor/183-inspector-property-grid-review.md#L186-L199)里的同值不变承诺。当前结论是已交付入口的源码调查，首个提交不含改引擎或已运行断言的主张。

## 1. 正常重现步骤

普通 Edit 场景中选中一个 Cube，使用[已发货的 Details 位置字段](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/assets/ui/editor/unreal/components/details.zui#L122-L142)将 Location X 提交为 `12.3456`，保存；再将它改成 `20.0`，通过 Edit > Undo 回到保存点。此时 `World.x` 仍是 `12.3456`，输入框按设计仅显示 `12.35`，Redo 能恢复 `20.0`。重新点击该输入框，不输入任何字符，直接 Enter。应仅结束编辑，不应悄悄改变属性或截断重做。本例不使用拖动 gizmo、Simulate、多选或切换场景。

## 2. 影响为什么不容易看出？

两位小数的显示前后都一样。依据[两位小数的视图投影](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/workbench/state/editor_state_selection.rs#L396-L420)和[活动文档历史和 redo 游标](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/editing/engine/history.rs#L709-L738)，实际 `f32` 却从 `0x41458794` 变成 `0x4145999a`；这是一条真正的新历史记录，会替换 redo 分支，并使已经回到保存点的场景重新变脏。应检查 World、活动文档的保存点和[可用性快照](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs#L107-L125)，不能以文字还显示 `12.35` 作为通过。

## 3. ZE 哪里把显示值当成了用户输入？

[鼠标命中后初始化文本](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/pointer/buttons/text_focus/activation.rs#L7-L28)从命中的已投影节点拿到舍入后的显示字符串。[原生键盘 Enter](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/window/text_input/keyboard.rs#L25-L46)进入[焦点提交](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/window/text_input/edit.rs#L76-L99)，即使这次没有编辑事件也提交当前缓冲。[数值解析和轴掩码](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/dispatch/templates/workbench/transform_edit.rs#L96-L122)将其解析为 `f32` 并标记这一个轴；[对当前 World 精确值的单轴合并](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/workbench/state/editor_state_selection.rs#L565-L599)只更改该轴，其余轴的精度保护无法防止这一轴被舍入值覆盖。原有 `f32` 实际不同，因而历史服务在这里清除旧 redo 是正常行为，根因发生得更早。

## 4. UE 有什么可用的参照？

真实变换编辑（`Engine/Source/Editor/DetailCustomizations/Private/ComponentTransformDetails.cpp:1438–1560`）按实际对象的候选值与旧值决定是否开启普通编辑事务；`UTransBuffer::End`（`Engine/Source/Editor/UnrealEd/Private/EditorTransaction.cpp:1299–1369`）能丢弃无有效变化的事务并保留原先 redo。UE 的数值控件也可能在 Enter 时发出提交回调，精度与 ZE 不同，所以不声称 UE 已证明所有“聚焦后 Enter”都是 no-op。可迁移的是零实际编辑不应占用历史，以及在同一对象上核实 Undo/Redo。

- `System.Promotion.Editor.Blueprint Editor`（`Engine/Source/Editor/Kismet/Private/Tests/BlueprintEditorTests.cpp:1044–1081`）用图节点编辑、撤销和重做后的真实图结构作断言，不是数值框输入用例。
- `System.Engine.Transactions.EditPropertyData`（`Engine/Source/Runtime/Engine/Private/Tests/TransactionDiffingTests.cpp:22–42`）区分实际属性变化，未测试编辑器按键、保存点或 redo 菜单。没有定位到与本重现序列完全相同的 UE 原测。

## 5. 有限修复方向

在真实 Details 轴字段入口保留“这次聚焦后有没有输入”的证据。只对未编辑确认做 no-op；用户**明确重新输入** `12.35` 时仍可把 `12.3456` 改成该值。单纯比较结束文本与最初的显示文本会误吞这一操作。不要给所有文本字段关闭 Enter 提交，不改轴显示精度、ECS 存储、反射或 Undo 栈本身。与 [#205](https://github.com/He-Jiahui/ZirconEngine/pull/205) 的 Move 预览、[#209](https://github.com/He-Jiahui/ZirconEngine/pull/209) 的 Simulate 手势及 [#220](https://github.com/He-Jiahui/ZirconEngine/pull/220) 的拾取不同。

## 6. 应当如何实跑？

复用[现有宿主中的 Inspector 撤销/重做](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/tests/retained_host_automation.rs#L227-L266)和[原生按键测试入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/window/tests/test_support.rs#L259-L286)，在确实绑定当前项目和场景 Document 的宿主中推进。使用已发布数值框的真实命中区域聚焦，并走键盘 Enter，不用一个直接调用属性 setter 的测试冒充。必须先证实原精度、实际 Undo 到保存点、有待重做的第二条变更；无输入确认后应能通过真实 Redo 恢复 `20.0`。另测实际键入不同值、和明确重输舍入后的值，保留对子轴、历史步数、脏态的断言。只签实际达到的入口；不把模拟回调、IME、虚拟键盘、Windows 桌面可视性或 GPU 验收混进来。

本地参考为 6.0.0 / UE5 / CL 0，未知上游 revision；没有运行 UE。这些哈希固定参考文件而非发行版本：

- `ComponentTransformDetails.cpp`：`4a0dd5fe0170770785db56cf1d60c61e6ea81a9db2b14e42fb0d2e6bb51d61ae`
- `BlueprintEditorTests.cpp`：`2838f0ab4ae694f4c35580d99e0d28dc47a725d99a8daf614a5e5582dbba7b72`
- `EditorTransaction.cpp`：`4a445e4183060b9cf870ed9acc4b14a8ef36e4e22760c36e2449edf635ea2e4b`
- `TransactionDiffingTests.cpp`：`49a8ea72cd88f54f4313b6cd57766c691c29f2e004f2d4cbce2e28f2e39969a6`

## 7. 在原 owning Editor 宿主上的首次执行

[固定的仅测试提交](https://github.com/He-Jiahui/ZirconEngine/blob/280585580550850ff10b6206b065f6ba83137480/zircon_editor/src/ui/retained/app/tests/details_enter.rs)仅增加该问题的回归及挂载。Windows x64/MSVC 1.94.1，`zircon_editor --no-default-features --lib`，Rust 的 `test` profile（包内依赖需要 `target-editor-host`、`test-support`）。过滤 `details_enter_`，`--ignored --test-threads 1 --show-output`；专用物理项目、最近工程与用户设置，不借用实际用户状态。

真实 retained libtest 两例：0 过、2 败。未观测到预设的错误行为签名。

两例均实际运行，但在绑定启动场景时就遇到代际拒绝；没有聚焦 Details、按 Enter、记录保存点或执行 Redo，不能把本次失败当作精度丢失的 RED。

“明确重新输入显示值”和“仅确认已保存值”分别使用独占的 Windows 项目、设置和用户目录。两者都已编译、成功创建正式项目，随后在 `apply_startup_session` 收到 `the authoring scene belongs to gateway generation 1, but generation 2 is current`。既有 `ChildWindowHostHarness` 在同一服务中先建示例 World，再构造 Welcome，最后装回更早的示例状态；这一测试装配顺序不等于真实产品冷启动，也不能据此声称用户的保存文件或历史已经改变。

后续验证应直接从已预检的真实项目意图建立宿主、加载磁盘场景并绑定 Document，以实际项目中名为 Cube 的 Mesh 为目标；不重写网关代号、不创建替代实体、不用手工发送属性 setter 代替按键。必须验证精确 World、已保存文件、两位显示和存续的 redo 分支都已就位，才能判断无输入确认以及明确重输两种动作。

编译回执 `639ad560db92ffeda110d579c3848ec646f40f0a64d0202f98ab5da8ac33076e`；执行回执 `36881b1ddae16688cb779004dad29361ea7cf5b37ef2b7dde0860a35b4210ebd`，stdout `9e0df3cee2d535dd2d31cf427c64fb1468990fad2e093674c2983a9421ef407e`。

未改生产逻辑；没有把其它 PR 的修复叠上来。本阶段不是 UE、GPU 或 stock 桌面端到端验证。
