# 层级重命名会把一个组合字符拆开编辑

## 1. 差异与输入

在 Hierarchy 选中一个名字为 `Cafe\u{0301}` 的节点，按 **F2 → End → Shift+左 → Backspace → Enter**。末尾的 `e` 和 `U+0301` 组合重音构成同一个可见字符。按字素编辑时，选区应包含两者，提交结果应为 `Caf`。按 ZE 当前实现推演，光标从 scalar 5 移到 4，选区只含重音，提交结果为 `Cafe`。把初始名字改为预组字符 `Caf\u{00E9}`，相同按键会提交 `Caf`。两种名字均可由现有接口正常保存。

## 2. 用户能看到的后果

F2 从当前场景读取选中的节点和原名，将它们交给层级内联编辑器。编辑时，层级行显示焦点内的草稿。Enter 把这串文字作为 `RenameNode` 送入场景命令。用户看到相同的末尾字母，执行同一次选择删除，却可能得到两个不同的节点名称。[F2 选择节点](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/hierarchy_rename.rs#L25-L56) · [草稿初值与提交](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/app/hierarchy_rename.rs#L97-L145) · [显示层级行](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/workbench/overlay/componentized/panes.rs#L126-L148) · [应用到场景](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs#L93-L102)

## 3. 两个引擎怎样决定删除范围

ZE 的 Editor 私有文本焦点在横向移动时将 scalar 偏移加一或减一，Backspace 和 Delete 也取相邻一个 scalar。最后的 `char_indices()` 将这个范围转换成合法的 UTF-8 字节边界。对分解形式的 `Café`，末尾 `é` 的完整字节范围为 `3..6`，基线选取了 `4..6`。[编辑算法](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/data/text_focus/editing.rs#L4-L94) · [按键派发](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/window/text_input/keyboard.rs#L26-L99)

本地 UE 参考树为 `6.0.0 / UE5 / CL 0`。`Engine/Source/Runtime/Slate/Private/Framework/Commands/GenericCommands.cpp:24` 将 Rename 绑定到 F2。`Engine/Source/Editor/SceneOutliner/Private/ActorTreeItem.cpp:53–78,269–286` 将编辑与提交绑定到 actor 标签。启用 ICU 时，`Engine/Source/Runtime/Slate/Private/Widgets/Text/SlateEditableTextLayout.cpp:145,1575–1687,3124–3164` 使用字符边界迭代器移动光标，未选中文字时按 `GetGraphemeAt` 的范围删除。`Engine/Source/Runtime/Core/Private/Internationalization/ICUBreakIterator.cpp:139–153` 实现前后边界查找。非 ICU 的旧迭代器采用另一套规则，此次对照采用 ICU 配置。

## 4. Rust 侧的改动范围

Editor 已依赖 `unicode-segmentation`。可用它枚举扩展字素簇边界，并将边界映射回焦点已经使用的 scalar 坐标。对 `Ae\u{0301}B`，有效位置为 `0、1、3、4`。从 3 向左应停在 1。输入原有的簇内坐标、替换选区或在组合标记前插入字符后，按新文本重新定位光标和锚点。ZE Runtime 的控件已经有同类键盘回归，可以复用输入与断言的思路。[Editor 依赖](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/Cargo.toml#L22-L30) · [Runtime 字素边界](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/text/grapheme.rs#L1-L75) · [Runtime 键盘原测](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/tests/widget_text_input_keyboard/basic_editing.rs#L437-L492)

## 5. 接口与问题归属

现有绘制和焦点接口读取 scalar 偏移。修复集中在 Editor 草稿选择哪些位置作为编辑边界，用户输入的 UTF-8 字节和 `NodeId` 继续按当前事务提交。相邻的 #174 处理 Runtime `TextInput` 接收新 Mark 时的输入准入，#222 处理 Details 浮点轴的提交条件，#215 和 #241 分别处理菜单翻译与 Web HUD 字体。本例只需在已有场景名字上执行原生键盘编辑。[焦点接口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/data/host_interaction/text_focus.rs#L8-L34) · [绘制端](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/paint/node_text.rs#L286-L352) · [#174](https://github.com/He-Jiahui/ZirconEngine/pull/174) · [#222](https://github.com/He-Jiahui/ZirconEngine/pull/222)

## 6. 原测试依据与验收

UE 的 `Slate.Core.Text.TextLayout.TextEditing` 在 `Engine/Source/Runtime/Slate/Private/Test/TextLayoutTest_TextEditing.cpp:144–162,195–218,425–443,474–495` 用 `NoWrap.RemoveAt`、`Wrapping.RemoveAt` 及插入用例断言修改之后的完整文本。它们处理的是已给定的文本范围。新用例补上这次调查发现的键盘选区边界。

ZE 的产品回归从同一节点分别载入预组和分解两种标签，正常派发上述五个按键。编辑中核对草稿，提交后核对权威场景名字、一次 `RenameNode` 事件和一次历史记录。另以焦点级测试检查左右移动、双向删除、替换、簇内锚点，以及插入和删除后相邻字符重新组成字素的情形。既有三个焦点用例已通过。新用例的原生结果将在此草稿后续提交记录。[既有焦点回归](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/data/text_focus/editing/tests/cases.rs#L11-L56)

## 冷启动真实工程后，F2 重命名复现了字素拆分

首次夹具只有空 `World::new()`，F2 未到达目标行为，不能作为公开路径的 RED。重跑在独占的真实 RenderableEmpty 工程中写入预组 `Caf\u{00E9}` 与分解 `Cafe\u{0301}` 场景名，经正常打开验证其 Edit World/文档身份。两种输入使用不同的真实节点，并分别按 F2、End、Shift+左、Backspace、Enter。

- [实际 F2 用例](https://github.com/He-Jiahui/ZirconEngine/blob/1fde8273c28a5624de2fbbe6f8296feff5126e52/zircon_editor/src/ui/retained/app/tests/hierarchy_rename.rs#L149-L301)：预组提交 `Caf`；分解臂草稿、单条 `RenameNode` 和真实读回均为 `Cafe`，历史各恰增一笔。失败位于[最后语义断言](https://github.com/He-Jiahui/ZirconEngine/blob/1fde8273c28a5624de2fbbe6f8296feff5126e52/zircon_editor/src/ui/retained/app/tests/hierarchy_rename.rs#L291)，其期望是 `Caf`，不是初始化、焦点或提交失败。
- [焦点用例](https://github.com/He-Jiahui/ZirconEngine/blob/1fde8273c28a5624de2fbbe6f8296feff5126e52/zircon_editor/src/ui/retained/contract/data/text_focus/editing/tests/cases.rs#L1-L137)：3 过、5 败。左右移动选区 `(2, 4)` 而非 `(1, 4)`；退格留下 `AeB` 而非 `AB`；拼接后停在 offset 1 而非 2；外来中间标量 offset 2 未贴边至 1；Shift 重输留下 `AeQ` 而非 `AQ`。

本次仅加测试，未改生产。

Windows x64/MSVC、Rust 1.94.1：仅 `zircon_editor --no-default-features --lib --profile test --locked --offline --jobs 1 --no-run --message-format=json`；两组选定 case 单线程。测试使用真实已准入 Project 的 retained 事件分发与 authoring Scene，不启动物理桌面窗口，不是 OS 输入法、磁盘保存或 UE 产品验收。

## 修复私有编辑边界后，同一批用例通过

修前在已经提交的分解标签上[观察到 `Cafe`](https://github.com/He-Jiahui/ZirconEngine/blob/1fde8273c28a5624de2fbbe6f8296feff5126e52/zircon_editor/src/ui/retained/app/tests/hierarchy_rename.rs#L291)。只改[私有 textarea 焦点编辑实现](https://github.com/He-Jiahui/ZirconEngine/blob/f40cd739eee6bd6b0fd0dafbb47f67b8d2934175/zircon_editor/src/ui/retained/contract/data/text_focus/editing.rs)：沿已经依赖的 `unicode-segmentation` 扩展字素边界移动、选择和双向删除，在拼接后将 caret 贴回边界；不改变标量坐标契约、原始 UTF-8、现有事件及场景事务。

同一编译范围、相同测试源码与 filters 下，[F2 用例](https://github.com/He-Jiahui/ZirconEngine/blob/f40cd739eee6bd6b0fd0dafbb47f67b8d2934175/zircon_editor/src/ui/retained/app/tests/hierarchy_rename.rs#L149-L301) 1/1：预组和分解名都经同节点真实提交成为 `Caf`，各一条 `RenameNode` / 一笔历史；[焦点用例](https://github.com/He-Jiahui/ZirconEngine/blob/f40cd739eee6bd6b0fd0dafbb47f67b8d2934175/zircon_editor/src/ui/retained/contract/data/text_focus/editing/tests/cases.rs#L1-L137) 8/8，涵盖原对照与新增边界。未改测试源码、依赖清单或锁文件。

Windows x64/MSVC、Rust 1.94.1：仅 `zircon_editor --no-default-features --lib --profile test --locked --offline --jobs 1 --no-run --message-format=json`；两组选定 case 单线程。测试使用真实已准入 Project 的 retained 事件分发与 authoring Scene，不启动物理桌面窗口，不是 OS 输入法、磁盘保存或 UE 产品验收。
