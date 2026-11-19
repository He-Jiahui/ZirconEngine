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

## 8. 冷启动实测：能聚焦和键入，但第一次主动编辑未写进 World

[固定测试源码](https://github.com/He-Jiahui/ZirconEngine/blob/e56a302b9c4ae1c64181b0f23ac995cd5c356303/zircon_editor/src/ui/retained/app/tests/details_enter.rs)只替换了宿主装配，原两条测试体、断言和默认忽略状态未变。正式预检的 RenderableEmpty 项目由 retained host 直接创建，得到 Ready 的同根项目、真实绑定的 Scene / Document、磁盘模板中名为 Cube 的 Mesh 和选中它的 Edit World，不再使用先前两次创建的演示 façade。

正式冷启动已到达真实场景、Details 轴框和 Winit 按键路径。两项都在第一次主动输入 12.3456 后读到 World X=0；尚未到保存、Undo 或无输入 Enter。这是更早的未写入现象，不是原题的精度/Redo RED。 两个独立物理项目中，真实已挂载的轴框均命中，焦点正确且有 commit action；Ctrl+A 后逐键录入，输入缓冲在按 Enter 前确为请求的字符串，预览阶段还没有修改 World。收到合成的 Winit Enter 后，第一例在主动编辑的初始断言看到 `Vec3(0,0,0)` 而不是 `Vec3(12.3456,0,0)`；另一例在同一步读到 X 位模式 `0` 而不是 `1095075732`。两项目都没有到达最终无输入确认、保存点或重做验证，不推断为何没写入、也不改生产。

Windows x64/MSVC 1.94.1 / `zircon_editor --no-default-features --lib` / `test`，`details_enter_ --ignored --test-threads 1 --show-output`。实跑 0 过 2 败；编译回执 `34a2b2716d47673ab4fc78277d6b0c91c83a77780ac79a28869920780842ee75`，原生执行 `d79215438dc765a3be3eb10de3d3d9007fbba96f09b13270ebe459ae7c9deb8f`，stdout `759f2d7d415a90919fd28b73f2bf356807b606f64cc934f58a8c4b3caffba283`。它验证了 retained 输入路径，不代表桌面注入或 GPU 展示。下一步沿实际 commit binding、命令结果、选中对象的可编辑性和活动文档继续定位，不能把第一笔没提交的场景伪造成已经精确保存 `12.3456`。

## 9. 已在可编辑相机复现，区分有输入与无输入 Enter

在冷启动的真实默认工程中，保留原两例，另外选择模板自带的正常可编辑 Dynamic Camera；四例使用相同的命中、失焦/聚焦、合成 Winit 键入与事务观察。总计 **1 过 / 3 败**，不是三个原缺陷。

| 用户动作 | World 和 Scene 文档证据 |
|---|---|
| 主动输入 X=12.3456、保存；输入 Y=6.25，再 Undo | 磁盘 Camera X 是 `12.345600128173828`，原生 X 位为 `0x41458794`；Undo 后 top/saved 均为事务 1，Y=2、can_redo=true、dirty=false。 |
| 未作文字输入，只聚焦 X 并 Enter | 同一 Document、World generation 和 Camera，输入框显示 `12.35`。立即出现 `Inspector/TransformPositionXCommit:onSubmit`，原子日志记 `transaction_id=3`、changed=true；X 变 `0x4145999a`（12.35），Y/Z 不变，top/saved=3/1，can_redo=false、dirty=true。 |
| 随后尝试真实 Redo | 命令因 when 条件禁用，未到原测试末尾的联合 assert；前后 World 位、历史与 journal 已直接记录。保留工程磁盘仍是前一保存的精确值，**不是磁盘已被破坏**。 |
| 独立显式重新输入显示值 | Dynamic Camera 真正提交了 12.35，Undo/Redo 和保存均通过；修复不可禁止这一操作。 |
| 原模板 Static Cube 两例 | 正确命中/输入，提交返回 `reflection edit failed`，无事务，World 保持 `(0,0,0)`；日志没展开内部错误类型，不把它们算作第二、第三次 #222 RED。 |

动态相机实测：同一 Scene 已保存 X=12.3456 且可重做 Y=6.25，未输入就按 Enter 后 X 被改为 12.35，产生事务 #3，文档变脏，真正的 Redo 命令被禁用。显式重输同值对照通过；两个静态方块均是更早的反射写拒绝，不算复现。 下次先将 Static 改作诚实负对照，在可编辑例中于调用 Redo 前分别断言精度和历史，冻结这份回归后再做窄修复。

Windows x64/MSVC 1.94.1，`zircon_editor --no-default-features --lib --profile test`。实测输入 [同一测试版本](https://github.com/He-Jiahui/ZirconEngine/commit/3f18406a33d8277882bd87819ac86639f9419f39)，编译回执 `2fb9c4a44cd18ecf2f65578573124dcff6a7c0cf6d5a0dcaf8239ec8420c2e02`，实际执行 `c77aa2153f79d468963808141f11670cebe348c555c74d587ee3959013ab9884`，stdout `29d41d16fd45cf8315a569f4500aeb5c409ee8f41a7ae31f44352a6b39f382b7`。

## 10. 冻结可编辑和静态负控：只剩目标行为失败

新测试在同一 World 和保存点上直接断言 `(Transform, HistoryStatus)`，不是拿后续菜单错误代替原缺陷。将 Static 两案校正为真实拒绝写入的负控后，同一组 3 过 / 1 败，唯一失败在无编辑 Enter 刚返回、尝试 Redo 之前。明确重输与两个真实静态拒绝均通过，更早一轮的四案输出保留。 正控确证同一已提交 Scene 的 X/Y、显示值与可重做的分支，然后 `DETAILS_UNTOUCHED_ENTER_AUTHORING` 的精准联合检查失败。没有触碰产品代码。另外用同一 EXE 执行的四个既有门（设置、Unicode、层级重命名、搜索焦点）：3 过 / 1 败。 层级重命名原例在取得默认 Scene 的选中目标时停止（`default scene should expose a hierarchy rename target`），尚未走到原生重试 Enter，不能以它证明重命名通过。

Windows x64/MSVC 1.94.1，`zircon_editor --no-default-features --lib --profile test`。实测输入 [同一测试版本](https://github.com/He-Jiahui/ZirconEngine/commit/fa378b3dddf26efc540b353d8879f95573596955)，编译回执 `50218f13921aafdb2033c309735d93784f5260446c2d3cb9bc0f9fc9c5c2b5f6`，实际执行 `ac91c87bba24fbd4397f7a088c6bf85992f1d583d7e49e30ac869c8fccfd76fe`，stdout `61fcf9b8e80e74b7303e886d0294909cb74c7b4bc3985204df1f4cf933159f41`。

## 11. 相同输入、同一测试字节：保留精度和重做

在原 PR 上仅修改 Editor Details 数值轴的原生焦点/提交行为，不更改 Runtime 文档、事务系统或静态变换约束。未改冻结的测试、特性、过滤器和执行选项。

同一冻结回归由 3过/1败 变为 4过/0败。空 Enter 和只移动光标均保留原 Scene 的精度、保存点及可重做的 Y；显式重输相同显示值仍形成可撤销事务。另外独立核对设置、文本、搜索、重命名的既有测试，仍以各自实测结果为准。 主案前后两次无文字编辑 Enter 的实际事件序列均未增加，随后真实 Redo 仍能重放刚撤销的 Y 位移；相机精确值与磁盘已保存的字节均不变。静态方块仍能报告 `reflection edit failed`，不会把合法拒绝记成成功。另外用同一 EXE 执行的四个既有门（设置、Unicode、层级重命名、搜索焦点）：3 过 / 1 败。 层级重命名原例在取得默认 Scene 的选中目标时停止（`default scene should expose a hierarchy rename target`），尚未走到原生重试 Enter，不能以它证明重命名通过。 这不是 IME、系统粘贴、全平台 GUI 或所有属性编辑器验收。

Windows x64/MSVC 1.94.1，`zircon_editor --no-default-features --lib --profile test`。实测输入 [同一测试版本](https://github.com/He-Jiahui/ZirconEngine/commit/6efde4339752054cfb60a74ce37d202f9bacf874)，编译回执 `3086de4975b6180ea07d5fddfe82eabbb320e7fd980e216cb1740120401abce8`，实际执行 `34d9211dc838849ac382612c6377b64aeaeec36599c98d9a277e27c980b2b866`，stdout `7bcd17fc91a2dc621f86e291ff32c5ce43c8ffdb896277f5cf69b14e2cbb646e`。

## 12. 同焦点提交与重试已到位；额外保存的失败单列

原 4 例全过。新加静态拒绝/同焦点重试通过；动态相机的成功提交、有意重输同值后的 changed=false、两次空 Enter 均通过业务断言，随后清理中额外的 Save 在已写场景后重导入失败。整例计失败，不给保存或本组全绿。

相同焦点下，动态相机先记录 `changed=true / tx1`（X=12.345600128173828），重按 Enter 的 journal 保持 `3→3`；Ctrl+A 输入相同精度字串，实际提交 `changed=false / 无新事务`，再按 Enter 的 journal 保持 `4→4`。静态 Cube 的两份 Submit 记录都有 reflection veto，事务均为空；World 和历史未改、焦点保留。

动态案最终另调的标准 Save 不能当作成功：返回“源已写入，但场景重导入失败，历史仍脏”，所以新增组实际是 **1 过 1 败**。同一工程留下的磁盘场景确有 `12.345600128173828`，底层日志明确是 `targeted asset import was superseded by a newer project generation`。这不是空 Enter 不正确；没有凭此断言某个 watcher 是推进代次的写手，也不取消保护过期提交的校验。原来的精度/保存/重做四例使用同一 EXE 全过，照原样保留。下一次只为这条不依赖另存的窄 ACK 用例，在全部现有断言后真实 Undo 恢复原值/干净历史并正常关闭；保存失败的本次结果与根因调查仍单独保留。

Windows x64/MSVC 1.94.1，`zircon_editor --no-default-features --lib --profile test`，同一[受测提交](https://github.com/He-Jiahui/ZirconEngine/commit/c68a75c9b719f22eaba8d3c37fc51cc6bbe3a600)和编译回执 `2bb1cb11e42f58521bf51170eb79a821a24a2e5488d2db773b1653cb5547feae`。各次均显式 `--ignored --test-threads 1 --show-output`，独立项目和用户配置。

| 过滤器 | 实际运行 | 执行回执 SHA-256 |
|---|---|---|
| `details_axis_ack_` | 1 过 / 1 败 | `f378fded1edd5d447d545b18db05ec3855500d2192137cb7f4f6cfb246e7475f` |
| `details_enter_` | 4 过 / 0 败 | `baf5882c1eae6905d496768ba407a2d98931e701043ff3a4880fa57abaa0a3e6` |
