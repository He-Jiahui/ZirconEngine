# [Editor/Localization] File 菜单入口应随 Editor Language 切换

基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。跟踪已发货的 `menu.file.label`。本报告是源码调查而非语言切换的运行录像。

## 1. 用户路径

普通桌面 Level Editor 打开项目；Edit → Editor Settings → Editor / Language → Editor Language，由 `en` 改受支持的 `zh-CN`，等正常刷新、关设置后点顶栏 File。该设置是 [User 热切 enum](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/settings/defaults.rs#L118-L136>)，不需重启。

## 2. 预期与固定源码结果

同一挂载的 File 入口应由 `File` 变 `文件`；实际源码仍给按钮 `File`，该 popup 的 `file.scene.open` 已可显示 `打开场景...`。两份键都已发货（[en](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/assets/i18n/en.toml#L25-L71>)、[zh-CN](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/assets/i18n/zh-CN.toml#L25-L71>)）；不是请求补译文。仅源码见证，尚未运行 GUI、取得 RED/GREEN 或修复。

## 3. ZE 断点

[生产模型解析根](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/commands/menu.rs#L24-L55>)；[挂载模板按钮](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/assets/ui/editor/unreal/components/menus.zui#L38-L44>) 是 literal，实际 [Bridge:59–101](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/dispatch/templates/workbench/main_menu_items.rs#L59-L101>) 消费命令叶及 trigger 可见/禁用，不投影根标签。[实际绘制](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/workbench/overlay/componentized.rs#L383-L397>) 取同一 authored 节点，非另一路 legacy model。

## 4. UE 相邻控制，不等价

本机 `6.0.0 / UE5 / CL 0`、revision unknown。`Engine/Source/Runtime/Core/Tests/Internationalization/KeysCultureTest.cpp:48,63–111,130–141`（`FKeysCultureTest` / `Development::System::Core::Misc::KeysCulture`，SHA-256 `6c97d6c64438e961bd89fab9625ea8d1f24ae30cb540b9cbddd0be377d35f040`）证明同一文本对象按已有译文刷新，缺译文走 native fallback；不测实际 File 控件。`Engine/Source/Developer/ToolMenus/Private/Tests/ToolMenusTestUtilities.spec.cpp:10,49–75,197–227`（`System.ToolMenus.TestUtilities` / `Should match the exact entries`，SHA-256 `e0548bc3f2c9ce13f17475b4664c5ce4a10a51385b7bdfe30e241088fe199d7f`）只测菜单结构，不切语言。这两组均是本机已有的 UE 测试源码，未运行或移植。

作为实际编辑器生产对照，`FMainMenu::RegisterMainMenu` 在 `Engine/Source/Editor/MainFrame/Private/Menus/MainMenu.cpp:432–461` 分开稳定 File ID 与可本地化显示字符串（SHA-256 `92b103c0f67656480908097446bea9e2feb9999b065ea44778b3e9aa983dba20`）。这里既不转载 UE 源码，也不声称测试过它的窗口。

## 5. 回归

借[真实菜单路由与 enablement 用例](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/dispatch/workbench/tests/main_menu_items.rs#L8-L107>)，验证同一可见 `MenuSlot0` 在 `en→zh-CN→en` 产生 `File→文件→File`；现有已本地化叶作正控，稳定 action、popup、快捷键、启用态都不变。只接通该键到已发货消费者，不要求新增语言、伪本地化或屏幕阅读器协议。

## 6. 独立

[#174](<https://github.com/He-Jiahui/ZirconEngine/pull/174>) 是文本输入字素编辑；[#212](<https://github.com/He-Jiahui/ZirconEngine/pull/212>) 是不可见 Runtime 回退菜单的命令；[#213](<https://github.com/He-Jiahui/ZirconEngine/pull/213>) 是 Runtime 触摸取消。本例是普通可见 Editor 控件的显示标签，不改输入语义或运行时。 不改 ECS、ABI、持久设置格式或 action ID。本轮独立调查在共同 main 上起草，后续行为回归与修复只能追加在本主题原 PR。

## 7. 原有工作台：真实标题与弹窗命令的对照

[实跑提交 `8ffe8c73df5210e9e9f18f2fefe26f498e2529c0` 的同一回归](https://github.com/He-Jiahui/ZirconEngine/blob/8ffe8c73df5210e9e9f18f2fefe26f498e2529c0/zircon_editor/src/ui/retained/dispatch/workbench/tests/main_menu_items.rs)。通过实际的启动文档加载器创建一个组件化工作台；在同一个已挂载 bridge 上，把 User 语言从英文切成中文再清回英文。每次先解析原 command model，把同一节点提交为 native host presentation，再从实际已提交的可见节点命中 File、分发 Click 并检查同一弹窗。结论不是从读取 TOML 推测的，也没有打开操作系统窗口或做像素 OCR。

| 已提交或真实解析的内容 | 英文 → 中文 → 英文 |
|---|---|
| 已发布的 File 触发器 | `File → File → File` |
| 同一弹窗命令 `file.scene.open` | `Open Scene... → 打开场景... → Open Scene...` |
| 命中路由、快捷键和可用性 | 三次均为 `workbench.unreal.menu.file.open`、`Ctrl+O`、可用 |

仅在最后的三阶段标题断言失败；中文阶段命令根模型与弹窗叶都已翻译，只有真实可点击的顶栏文字仍然是 File。尚未在本提交修改生产。 Windows x64/MSVC 1.94.1，`zircon_editor --no-default-features --lib`（无额外 feature）、`test` profile；单独运行 `file_menu_locale_`，**0 过、1 败**。子进程未设置资产根 override，实际 exe 旁没有 `assets`，按生产 dev-root 规则从对应提交的规范 Editor asset 根读取菜单。原生构建回执 `7bfa969b150100b75275853e4f4845ee6794cc16b1eec1e3072d459668022170`；实际执行 `a9a03d611a546cbcfbda2be2580dcdba82b015e9a491f4a15a315942345a892b`；stdout `1b90bd5c363d9a34d6f1ddb4a86f792ed4054de6e1aa8d08d0148935d8e88a6e`。没有覆盖 OS 辅助技术、Locale 国际化全量、设备绘制或 UE 原测。

## 8. 修复：复用菜单已有的翻译条目

[实跑提交 `41e96aaec1fff213fa7fc9185c7edaf120c26517` 的同一回归](https://github.com/He-Jiahui/ZirconEngine/blob/41e96aaec1fff213fa7fc9185c7edaf120c26517/zircon_editor/src/ui/retained/dispatch/workbench/tests/main_menu_items.rs)。通过实际的启动文档加载器创建一个组件化工作台；在同一个已挂载 bridge 上，把 User 语言从英文切成中文再清回英文。每次先解析原 command model，把同一节点提交为 native host presentation，再从实际已提交的可见节点命中 File、分发 Click 并检查同一弹窗。结论不是从读取 TOML 推测的，也没有打开操作系统窗口或做像素 OCR。

| 已提交或真实解析的内容 | 英文 → 中文 → 英文 |
|---|---|
| 已发布的 File 触发器 | `File → 文件 → File` |
| 同一弹窗命令 `file.scene.open` | `Open Scene... → 打开场景... → Open Scene...` |
| 命中路由、快捷键和可用性 | 三次均为 `workbench.unreal.menu.file.open`、`Ctrl+O`、可用 |

同一测试修后通过。只把实际 `MenuSlot0.text` 绑定到现有 `menu.file.label` 并保留英文 fallback；没有改 tooltip、ID、其它菜单、翻译表或 i18n 总线。 Windows x64/MSVC 1.94.1，`zircon_editor --no-default-features --lib`（无额外 feature）、`test` profile；单独运行 `file_menu_locale_`，**1 过、0 败**。子进程未设置资产根 override，实际 exe 旁没有 `assets`，按生产 dev-root 规则从对应提交的规范 Editor asset 根读取菜单。原生构建回执 `c7345e9bd8c8aa6484730e7f17228b41183adcfccb8383a516efe484845b45f1`；实际执行 `90c86db6021c906e858fcde73b74a6ce254a257d702f2736769b11cbbee9fd2e`；stdout `ea4ca2b980e4c9a5f42aac2489b986e29299293428dc41f00a5fd664a0c6a613`。没有覆盖 OS 辅助技术、Locale 国际化全量、设备绘制或 UE 原测。

## Windows 桌面项目的普通启动

从 `cc8e45d861618ed5975034f91e5e9189c483db2f` 构建 `zircon_editor` 与启用 `target-editor-host` 的运行库，使用构建工具生成的同一 BuildSet。首次测试直接打开模板目录，编辑器报告缺少 `project_guid`。随后通过 `--create-project` 创建 RenderableEmpty，实际写入的 v3 清单包含相同的项目 GUID、模板回执和 BuildSet。使用这份项目的两个副本执行普通 `--project` 启动，每个副本包含 18 个文件，合计 13,004 字节。

两组都使用独立偏好设置和相同的默认场景。两套 671 个引擎资源只有菜单定义的内容不同。普通启动关闭首帧 PNG 捕获，保留既有的已提交界面状态记录。

| 菜单资源 | 启动观察时长 | 日志中最后一个共同的工作台动作 |
|---|---:|---|
| 公共 main 的资源 | 244.7 秒 | Camera 选择与视口布局 |
| 此 PR 的资源 | 245.0 秒 | Camera 选择与视口布局 |

观察结束时，两组均未生成已提交的界面状态，窗口枚举也未返回可操作的编辑器窗口。测试未能进入 Editor Settings。既有组件回归中同一菜单显示 `File → 文件 → File`。桌面上切换设置、选择场景和观察实际字形仍需验证。[菜单定义](https://github.com/He-Jiahui/ZirconEngine/blob/cc8e45d861618ed5975034f91e5e9189c483db2f/zircon_editor/assets/ui/editor/unreal/components/menus.zui#L38-L44)

## 普通窗口的首次重绘跟踪

沿用 `8d004310beb2cb3cb8d47f8ed6cdae889a182d10` 中记录的 BuildSet、修前菜单资源和正式 RenderableEmpty 项目。新建独立用户目录，仅开启 `ZIRCON_EDITOR_TRACE_NORMAL_CAPTURE_REDRAW=1`。55 秒观测内，跟踪记录了以下顺序。

| 记录序号 | 经过的调用 | 观测 |
|---:|---|---|
| 9 | `replace_host_presentation` | 结构、几何和点击区域的代数由 0 变为 1 |
| 11 | `patch_host_presentation_panes` | 三者由 1 变为 2 |
| 12 | `about_to_wait_impl` | Winit 事件循环消费后台唤醒 |
| 13–14 | `external_redraw`、`pending_redraw_merge` | 资源刷新请求与待处理的完整启动重绘汇合 |

截至观测结束，日志中没有呈现器选择和原生重绘回调，编辑器也未写出可操作界面状态。后续诊断的位置是原生表面可用回调、呈现器启动轮询和窗口创建的返回点。本次跟踪使用修前菜单组，桌面语言切换仍待正常窗口显示后验证。

[呈现器创建路径](https://github.com/He-Jiahui/ZirconEngine/blob/cc8e45d861618ed5975034f91e5e9189c483db2f/zircon_editor/src/ui/retained/contract/window/event_loop/lifecycle.rs#L41-L94) · [重绘请求合并](https://github.com/He-Jiahui/ZirconEngine/blob/cc8e45d861618ed5975034f91e5e9189c483db2f/zircon_editor/src/ui/retained/contract/window/event_loop/redraw.rs#L102-L145)
