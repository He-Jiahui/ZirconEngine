# Widget/UI 实际 Penpot 与导出记录

## 2026-10-25：原 Widget rev48 的长标签与中文行框接续

原 file `b4a1e3ce-9e6a-81fe-8008-d69d60d24223` / page `b4a1e3ce-9e6a-81fe-8008-d69d60d24224` 从rev45经3次真实HTTP200保存到rev48，关闭fresh重开与随后认证API同data SHA `ab3278a5e2d23b04c6dcf4f109e8a6f30f7517d8f64d932bc2622eab82ade3b6`。20个候选用原字体声明实际加载并测量，13个完整单行advance加1px余量可容纳，才改原copy root row/nowrap/center与Text fill/auto/auto-height；7个超宽者保持原样。26对象有布局后继、3519非目标和整份板外data保持；8旧main、17Shared heads/source refs、Sharedrev5全data及插件registry保持。原文字/字号/paint/组件与style refs未改，无新对象/主题/detach。

详[本轮实作与差异](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-intrinsic-captions-r06/review.md)、[逐对象身份与前后hash](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-intrinsic-captions-r06/editable-design-delta.json)及[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-intrinsic-captions-r06/visual-review.json)。900默认/640默认/900中文各before/fresh共6原图及3并排/3原RGB差异图已全部逐张打开；只比较Penpot两个revision，不是正式PNG、EXE或1DIP证明。可视区外Grid/Canvas等仅有保存后的原生布局检查，不代替滚动验收。

640五个长Palette标签及长Details、900中文Details仍超宽；窄tabs/Search/中文query、More/scroll/caret/focus/IME和UE完整chrome继续open。本轮产品Rust/ZUI未改，原39714B/v2/97ID/6events及SHA `97992ab7cbab62665f17b550ef6ae115ea513302c48db47c44ba2c6030cec383`保持。字体buffer/加载与browser advance不证明引擎font/fallback；原#411和G09#412身份/范围不变，#412由原执行链对账，无本轮代投。正式导出/正常EXE/150%DPI/双端逐图与整族passed/accepted仍false，原失败保留。


## 2026-10-25：原 Widget 文件 rev45 的实际 caption flow 接续

原 file `b4a1e3ce-9e6a-81fe-8008-d69d60d24223` / page `b4a1e3ce-9e6a-81fe-8008-d69d60d24224` 已从 rev30 经单例 rev31、14 批真实保存到 rev45；关闭原 context 后 fresh 重开，同数据 SHA `4761ad43117f7acdc10be2b8542ba1d40d85aabbb68ab2f9fda3f078102fb282`。本轮总计201个既有本地 copy 的 root/text 共402对象；其余3143对象及整份非目标 file data 不变，8个既有 main、17个 Shared heads/source refs、Shared rev5 全 data 和插件 registry 不变。未创建组件、平行主题、字体或新对象。

报告根：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-caption-batched-r05`，详 `editable-design-delta.json` / `receipt.json` / `visual-review.json` / `image-comparisons.json`。实际6张 Penpot before/fresh workspace 图及3并排、3原始RGB差异图均已逐张打开复核。它们只证明本轮真实可编辑设计及保存变化，不是正式 shape PNG、Penpot/EXE 双端比较或1DIP验收。

当前可复核的6原图是报告根 `before-{1280-default,640-default,900-chinese}.png` 与对应 `fresh-*`；每图独立SHA在receipt/visual-review。`pair-*` 为原图并排，不重绘产品；`difference-*` 为同尺寸workspace RGB逐通道绝对差，没有配准/缩放。比较包含相邻设计板与Penpot chrome，不能据像素计数推断目标板或EXE的逻辑像素误差。

逐图现象：1280短菜单/Palette/Details/页脚的行框修正可见，Bind Widgets仍截断；640菜单/短条目修正可见，但Palette/Library/Hierarchy/Bind及Search依旧裁切；900中文原文/字体声明保持，Hierarchy query裁切与CJK行框仍open。空Designer仍为空文档设计态，不当作运行Preview/Graph/动画。正式导出写回原资产、真实handler/ACK与运行界面消费、EXE字体和DPI均未因此通过。所有失败与retention保留，实际释放0B，cleanupClosed=false。


原 Penpot file/page 已由 rev29 真实保存、关闭并 fresh 重开到 rev30：七块 Toolkit 板中40个旧实例改用 native row/nowrap/center 与 Text fill/auto/auto-height，原23/28高固定框及3/8向下越界收敛为16高原生行框。原字体/文字/样式、实例尺寸、source/shapeRef保持；3545对象、8主件、17 Shared heads不增加，3465非目标对象及Shared rev5完整data不变。六张1280/640/900中文 before/fresh图已逐张打开。640窄页签/搜索、中文query裁切及UE chrome仍open；原ZUI未改，正式导出/EXE/DPI/双端/1DIP及passed/accepted仍false。见[实际复核记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-hierarchy-designer-flow-r01/review.md)。 真实保存HTTP200/revn30，关闭重开data SHA `3d6c98d866b27f6416e494007361ae2307968ef2ce660c39c0ff82db4a2d8188`；[逐对象delta](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-hierarchy-designer-flow-r01/editable-design-delta.json)与[逐图台账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-hierarchy-designer-flow-r01/visual-review.json)均为本轮证据。

Palette clear生产链新增精确发现：原v2已声明has_clear_action，但ViewTemplateNodeData缺字段、retained转换固定false，且原生pane只有Press没有clear配对Release。已形成[19路径完整未应用consumer候选r02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/palette-native-clear-wiring-r01/review.md)，含typed source metadata→DTO/materializer/converter、原pointer/source/frame的Down/Up、原query/焦点链及5拟增回归；14额外公共路径只交Parent。32307B/SHA aaa219dd3178c74a66f8e7602e331d3b21544eaf9ac05932e66615e7eea07efd，19stdin fmt/apply-check0，未覆入源做guard/类型或执行。实际checkout仍为G09/actual-field r02的27项定义；候选32项不扩旧身份。IME/popup/ACK、公共left geometry与EXE/DPI/双端仍open，原Penpot rev29保持。

当前产品源码切片 actual-field r02 已在原 G09 两路径落实字段身份/路由/布局资格与退休检查。原Penpot rev29/3545objects/8main/17Shared heads与图像链未改；该设计图不作为本次输入修复的正常EXE证据，也未触发官方导出或新保存。 [当前字段资格r02静态回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/palette-current-field-r02/review.md)固定73保护输入/6旧identity-control保持、guard 0/0/0及87源依赖指纹。格式/精确补丁检查通过，类型与27项测试未执行；实际EXE/150%DPI/官方导出/双端1DIP及整族passed/accepted仍false。此前25项和各旧revision只代表原时点。

最新实证：原file`b4a1e3ce-9e6a-81fe-8008-d69d60d24223`/page`b4a1e3ce-9e6a-81fe-8008-d69d60d24224`已实际保存到rev29/3545objects/8local mains，新增900×620 Palette clear/focus/hide板`5b0b50ff-0901-801b-8008-d7ecdd7e3452`。3SearchInput与2Button为原Shared rev5真linked copies；现17heads，原12heads和3496非根对象保持。rev25→26→27→28→29真实HTTP save链、关闭/两次只读fresh重开及5张原图逐张记录在[来源映射](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-palette-clear-focus-r01/source-mapping.json)与[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-palette-clear-focus-r01/visual-review.json)。最终whole-data canonical SHA为`0a41899fa8d97a1f7c004879567934236847e159aa12190248ef36edf4b2bb8f`，Shared全data仍`8e6510bd…b37`；glyph/text遮挡、右侧组合、parse及缺省fills核对失败均保留，不改旧回执或重放修改。该真实浏览器图只证明可编辑设计与持久化，不是官方PNG/原生EXE/字体provider/DPI验收。原产品ZUI哈希97992ab7…c383不变，本次源导出0；所有产品passed/accepted继续false。

本轮Palette原生文本目标退休r01只有4条源修改和静态检查，[逐路径/保护输入及回归边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/palette-native-input-retirement-r01/review.md)已保存；未新增Penpot API/save/重开/截图、正式导出或字体provider证据。原rev24及历史图片保持，EXE/150%DPI/双端1DIP仍false。当前25项Palette、builtin4、关联Canvas34均0执行，不借旧图片或旧Jenkins身份补验。

G09当前产品源码进展：Parent签认的37路径r02已实际应用，完整query与同snapshot/rows经真实borrowed hit/host guard接原Session；[当前实施证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/view-retained-integration-r03/review.md)含原EOL/精确后像、43未写保护输入和静态guard0/0/0。此轮未写Penpot/产品ZUI、未导出或加载EXE，原97ID/6events和rev24保存设计不变；8新增定义执行0，不能用下方原图或工具ready证明新源码已运行。正式导出、字体、真实过滤/IME/焦点、EXE/150%DPI/双端比较继续open。

当前源码 hover/focus r03 仅改真实popup输入/状态/Button投影3文件，分离鼠标hover与键盘选择；共享目录几何导出和原ZUI保持。本轮没有新Penpot保存、截图、正式codec导出或normal EXE加载；最新设计证据仍是下方rev24，既有键盘/hover板与[当前源链映射](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-popup-hover-focus-r03/source-trace.json)不构成运行证明。34项定义执行0，静态guard通过仅限当前diff，#383/#389失败保持。

产品源码后继 r02：原43路径接线已存在，本次只修app/native合同2文件，将queued opener完整校验提前到close/begin前；没有Penpot写入、新PNG或产品ZUI导出。原rev24/3497objects设计证据保持。实际queue16085/#389因source capture中HEAD/full-index/membership变化而FAILURE，32项Rust执行0；见[当前源码与实际失败](widget-ui-validation.md#当前消费者-r02-与真实-389-失败)。不把新源码视为旧EXE已加载，不替换原设计/字体/正式导出/EXE/DPI及双端比较门。

## 当前首次 slot 状态板 rev24，2026-10-25T08:56Z

原分配file/page实际保存HTTP200→关闭旧context→fresh context重开为**rev24/3497objects/8local mains**，完整data SHA `2cf34e86572be01f36d8d8065f5d6d20765e7b94ea50213f7bd5bb63e2c3c993`一致。新900×620 board `d07f6d26-bfdd-8016-8008-d7d588c1fb7c`；两copy为 `d07f6d26-bfdd-8016-8008-d7d58cf9e004` / `d07f6d26-bfdd-8016-8008-d7d58f20964a`，仍指原Preset component `180e9578-8294-803b-8008-d6ae776be849` / main与shapeRef `180e9578-8294-803b-8008-d6ae75d79d3b`。

仅62新增对象/本域journal；3434原非根对象、原8mains、12Shared heads和所有其它data保持。40sourceRef、320style-reference字段真实读回，Shared rev5整data SHA `8e6510bd3b5ce83a0991ad912fa6a7091e803b55249fdb66f37fe3211ad88b37`及registry未变。没有新master、theme、库发布或runtime producer。前像/原save请求响应、字体response buffers、[回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-node-placement-r04/receipt.json)与[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-node-placement-r04/review.md)保留。

实际隔离headless完整截图SHA `f2fe864fc24bf492f351540a94a696cfb6d009bb7067833c687cb37164dc7906`已打开：三列和页脚未裁切，数值明确标为源码预期；不作为真实运行界面。Roboto检查true不证明精确provider/CJK/atlas，fontsReady仍false。原rev23与更早图/失败均保留；正式PNG/导出/EXE/150%DPI/并排差异/1DIP仍pending，passed/accepted=false。

源码后继说明（2026-10-25T08:51Z）：[Canvas r04](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-node-placement-preservation-r04/summary.md)修复首次生成slot时未变化定位字段的默认值切换，仅3条本域Rust路径；本次没有新Penpot写入或截图。以下rev23四修饰键板保留为明确标注的显式slot源码夹具预期，不作为无slot修复的运行证明。新增3项回归执行0，类型/正式导出/正常EXE/DPI与双端验收仍pending；#383原capture失败与全部旧图片保留。

## 当前修饰键板 rev23，2026-10-25T08:18Z

原file `b4a1e3ce-9e6a-81fe-8008-d69d60d24223` / page `b4a1e3ce-9e6a-81fe-8008-d69d60d24224`真实保存HTTP200，关闭原context后fresh重开，均**rev23/3435objects**，完整data SHA `5b94f7b41e30f484686a2348fc6618388de74e14e431cfef2f9e567858f03460`相同。新900×620 board `f7131a51-5260-8090-8008-d7ccf7174941`是明确标注源码预期的四修饰键设计夹具，位置(0,2348)，不是实时preview。

| 身份 | 实际值 |
| --- | --- |
| Preset component / main | `180e9578-8294-803b-8008-d6ae776be849` / `180e9578-8294-803b-8008-d6ae75d79d3b` |
| Default copy | `f7131a51-5260-8090-8008-d7ccf8a92b4e` |
| Shift copy | `f7131a51-5260-8090-8008-d7ccfa87e32b` |
| Ctrl copy | `f7131a51-5260-8090-8008-d7ccfc57fe02` |
| Shift+Ctrl copy | `f7131a51-5260-8090-8008-d7ccfea1a76a` |
| 新增与保留 | 121新增对象；3313原非根对象、8原local mains、12原Shared heads完整保持；80 sourceRefs解析到原master |
| 共享样式 | 474实际引用字段，复用Shared rev5颜色和body/caption排版；没有新样式或主件 |
| Shared完整data | rev5 / SHA `8e6510bd3b5ce83a0991ad912fa6a7091e803b55249fdb66f37fe3211ad88b37`未变 |

[真实回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-modifiers-r03/receipt.json)保留save request/response原字节哈希、原生实例/geometry、前后完整数据与实际font response buffers；原plugin registry未变，所有自有浏览器context已退出。未启停服务或占桌面。[modifiers-fresh.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-modifiers-r03/modifiers-fresh.png)（SHA `29e2ccf7fe5bab71ae56c0a24adfa6fb6bcffc96393050c3948c0a2368240dd8`）已实际打开：四列及完整footer可见，未观察到文字裁切，底部仍有留白。它是1600×1100隔离headless的真实Penpot fit view，非官方Shape PNG或正常EXE。完整[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-modifiers-r03/review.json)保留字体provider、导出和产品门未通过。

本轮Canvas47路径真实接线已另记[源码/验证输入](widget-ui-validation.md#当前实施和真实-jenkins-结果2026-10-25)；原ZUI仍SHA97992ab7…c383/v2/97ID/6events，gap6待正常EXE A/B。Jenkins #383实际在source capture中失败，Rust执行0；不把源码、设计夹具、HTTP200或queue回执算作产品passed/accepted。下文rev22和更早段落为历史，保留原失败。

最新实际状态（2026-10-24T22:41Z）：原file/page已保存、关闭并fresh重开为**rev22/3314objects/原8local mains**。本次仅新增原Anchor Picker的一个“鼠标移出后保留键盘焦点”原生设计状态板；原2977个非根对象、12个Shared linked heads、原组件定义及Shared rev5完整数据保持。默认/hover64状态已存在，本次只补return48状态，没有新建主件。两张真实Penpot浏览器图已逐张打开；字体资源字节已匹配，但标题的精确字体检查仍false，故不能通过文字/产品验收。见[本轮回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/receipt.json)、[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/review.json)与[字体来源分析](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/font-source-analysis.json)。

历史摘要（rev12及此前）：当时Shared为rev1/isShared=true/41objects/4mains，旧rev0空库观测保留为历史。[受控共享消费板](widget-ui-shared-consumer.md) `485c980b-5100-806a-8008-d6e3f7ef5ff8`的12个原生linked copy、4项覆盖/reset和保存重开成立。继revn11 Resize Slot修订后，28个被动页签已用原字体实测并保持单行，真实ACK/关闭重开为revn12/2978objects/原8local components；严格只变56个shape，其余2922个shape和Shared未变，14个搜索值保持。七张新浏览器图已逐张打开复核，搜索裁切及真实tab导航仍open，详见下方revn12记录。官方Shape PNG仍返回null；PropertyEditorRow.value合法字段未发布，sourceHash/slot/export、样式和产品门继续pending。此结果不替代正常EXE验收。

2026-10-24，本聊天复用 Parent 保留的真实服务、账户、插件发行物和独立无头 Chromium。没有重启服务、创建账户/文件、复制源码树或占用共享桌面。本记录核对局部源资产样本、一次正式导出的间距编辑，以及后续锚点业务设计；完整 UE 窗口和引擎验收未完成。

## 实际身份和复用

| 项目 | 实际结果 |
| --- | --- |
| 文件 / 页面 | `05 Widget and UI` / `b4a1e3ce-9e6a-81fe-8008-d69d60d24223` / `b4a1e3ce-9e6a-81fe-8008-d69d60d24224`。 |
| 板 | `c572a930-f329-801a-8008-d6a46839c961`，名称为 `Widget UI - Designer and Details source specimen - partial`；29 个投影节点，只呈现现有源资产，没有伪造生产数据。 |
| 原始源 | `zircon_editor/assets/ui/editor/ui_asset_editor.zui`；v2、97 node/control IDs、6 条原事件。初始 SHA `bc954f28a2f8edb61d86a23e6031d7ab26912cee1749bca9289a93263bd1c47d`。 |
| 依赖 | 现有 `LayoutDependencies` 准备 10 个真实源资产，包括 editor_tokens 和现有 Workbench 控件。路径/SHA 见实际输入回执。 |
| 原生组件 | 后端读回 6 个本地 native components、20 个带 component-id 的 shape 记录（包含 main 和 instance）、11 个 flex 容器。没有计作 20 个独立实例。 |
| WorkbenchButton UUID | `c572a930-f329-801a-8008-d6a46ea3d363`。 |
| WorkbenchCaption 样式变体 UUID | `c572a930-f329-801a-8008-d6a46fc17960`、`c572a930-f329-801a-8008-d6a470aaa3f1`、`c572a930-f329-801a-8008-d6a4714ce897`。 |
| WorkbenchLabel 样式变体 UUID | `c572a930-f329-801a-8008-d6a4705761b0`、`c572a930-f329-801a-8008-d6a470f205a0`。 |
| 版本和公共库 | 桥资产版本为 v2；未取得可宣称的原生组件独立语义版本。Penpot shared colors/typographies 均为 0；公共库发布/继承依赖 Parent，尚未通过共享样式门槛。 |
| 插件 worker | `served-dist-current-20261023/assets/plugin.js`，SHA `40d1150059b975aa02d616a390021e3b1a5a2592703661efc78f605e6c7bafb6`；复用原发行物，公共文件未改。 |
| 浏览器 | 独立 Chromium 151，EXE SHA `409805a16d6416087e6b2f778df1cf8f7bbb267d6b99f6b5bb0a618eace234f2`；浏览器像素比例不作为 Engine 实际 DPI。 |

[打开实际文件](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60d24223&page-id=b4a1e3ce-9e6a-81fe-8008-d69d60d24224)。完整目标仍以 [设计契约](widget-ui-design.md) 和 [差异台账](widget-ui-inventory.md) 为准。

## 保存、编辑和原资产

局部导入实际 ACK 为 revn 0→1。关闭原 BrowserContext，新建认证上下文重开同一 file/page，后端 typed metadata 完全匹配；官方 `Export selected` 导出的原 ZUI 语义一致。

随后在正常 Penpot Layout Inspector 选择原 `designer_panel`，将 gap 4 改为 6。目标 shape 仍为 `c572a930-f329-801a-8008-d6a468a6b640`。实际子区域 Y 坐标从 36/320/352/384 变成 38/324/358/392，证明原生布局响应编辑。保存 ACK、关闭、新认证上下文重开后保留该变化，读回 revn 5/vern 0。

官方导出 SHA `71e49bfcff0b2d86239b116462c233cf5ebc986df8395314a0acca22f1b95df7`。现有 `applyZuiExportToSourceFiles` 在内存严格校验完整闭包，只允许原资产的 `nodes.designer_panel.layout.container.gap` 一项 delta。实际原文件现已与候选逐字节一致，SHA `97992ab7cbab62665f17b550ef6ae115ea513302c48db47c44ba2c6030cec383`；ID、组件引用、bindings、事件和其余字段保持。

6 是传播验证用的局部覆盖，尚未实测为 UE 最终间距；产品截图验证后须决定恢复 token 或按测量保留。单文件 change guard 通过，0 errors/warnings/suppressions，patch digest `dd1e5ef610c0e8e81c7d21670a8d0173ce245ed9ce3555e4fcb852c24fa15ee6`。

Jenkins 请求 `widget-ui-penpot-r01-layout` / operation `validate-editor-layout-01` 使用真实 checkout，聚焦 `zircon_editor` library 的 `tests::ui::ui_asset_editor::workbench_projection_layout`，默认 features 关闭；尚未取得测试终态，不能记 passed。

## 失败和未完成门槛

| 项目 | 实际事实和下一步 |
| --- | --- |
| 裸导入 | 未带依赖闭包时，真实插件因 unresolved paint token 拒绝；失败会话 `session-e55cada6-1964-4455-a11b-a4c1f50dbf6c` 保留。局部路径复用原闭包能力后可导入。 |
| 完整窗口 TreeRow | 官方 SDK 回读可选 paint 字段 missing→null，当前 painter guard 误判并回滚；0 ACK。已向 Parent 交接公共窄范围修复提案和完整诊断，未并发改写公共源码。 |
| 官方 PNG | 实际点击 `Export Canvas PNG` 后 90 秒无 download；`designer-details-session` 最终 status=failed。已成立的 ACK/重开/ZUI 导出事实单独记录，不把整个失败改成 pass。 |
| 编辑会话 | `designer-gap-edit-session` 的 status=`design-input-roundtrip-verified` 只指局部样本的保存和源 delta；官方 PNG 继续 pending，不是 LayoutReviewCase passed/accepted。 |
| UE 目标 | 最新七板已补出完整边界的 Palette/Hierarchy/Designer/Details 布局目标，见末节；Graph、动画和实际数据操作仍未实现，公共 composite、业务 adapter 和正式源导出仍缺。空 Designer 和诊断标签不计成功复刻或实时预览。 |
| 产品和画面 | EXE/运行资源指纹、真实引擎加载与截图变化、四个尺寸/DPI、状态矩阵、双端并排差异和逐图复核均 pending。没有桌面/原生捕获时段，不伪造 nonce、HWND 或 EXE 身份。 |

实际证据根：[penpot-real-r01](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-real-r01)。关键回执为 `designer-details-session/receipt.json`、`designer-gap-edit-session/receipt.json`；后者保存 before/after/reopened 布局审计。普通浏览器截图只证明真实 Penpot 页面，不替代官方 Shape PNG 或正常 Engine 原生证据。验收继续使用 [原验证矩阵](widget-ui-validation.md)。

## Canvas 锚点业务组件 r01

同一真实文件和页面新增 Widget 专用 `Canvas Anchor Preset` / `Canvas Anchor Picker`，版本 `canvas-anchors-r01`。实际保存后与关闭后新认证 context 读回均为 revn 7 / vern 0、1434 objects、8 个本地 native components。本轮新增 1362 objects、2 个业务组件；原 source-import 样本的 shape/main component 写集合为零。

| 组件 | 实际 UUID | main instance / 依赖 |
| --- | --- | --- |
| Canvas Anchor Preset | `180e9578-8294-803b-8008-d6ae776be849` | main `180e9578-8294-803b-8008-d6ae75d79d3b`；复用原 WorkbenchButton `c572a930-f329-801a-8008-d6a46ea3d363`。 |
| Canvas Anchor Picker | `180e9578-8294-803b-8008-d6ae7d94d33b` | main `180e9578-8294-803b-8008-d6ae7788b380`；依赖上述 Preset 和 Button。 |

实际 component modified-at 为 `2026-10-24 19:26:54 +08`。自有业务 namespace 为 `shared/zircon-widget-design`，不是正式 codec namespace；两者 engineComponentId=null，尚未登记 catalog、公开发布或写入新 ZUI 组件，不把 design metadata 冒充正式源 owner。未新建公共基础控件或主题。

三板各为 320×440 的局部状态 wrapper，内有 16 个 linked preset instances 和 8 个 native flow containers；它们不是全窗口 profile。48 个稳定键 `0:0`～`3:3`、component-id、anchors min/max 在新 context 重开后逐项核对。

| 状态 | board UUID | 实际逐图复核 |
| --- | --- | --- |
| 默认 | `180e9578-8294-803b-8008-d6ae7e4d971c` | 4×4 预设与两行提示完整；现有 Button 圆角/边框与 UE SimpleSharpButton 不同，等待 Parent 共享变体。 |
| 键盘焦点 / hover 终态 | `180e9578-8294-803b-8008-d6ae833142ad` | 中间预设焦点框可见，终态 frame 在单元格内；只是状态设计，没有真实键盘导航/0.4 秒动画/关闭返回行为。 |
| 中文长文本 | `180e9578-8294-803b-8008-d6ae88af2f0c` | Roboto / Noto Sans SC 实际可用；中文 glyph 与长提示末句可见，菜单内换行。该覆盖文本不是官方 UE 译文，Engine 字体/IME/DPI 尚未比较。 |

三张实际浏览器图含 Penpot chrome、画布约 212%，1600×1100 / dSF=1；不能从中声称 1 DIP 误差或实际 150% DPI。逐图 PNG/SHA、UE/主题/原资产指纹、状态差异与 owner 见 [canvas-anchor-design-review](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchors-r01/canvas-anchor-design-review.md)，机器记录见 [canvas-anchor-review.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchors-r01/canvas-anchor-review.json)。

本轮原 authoring receipt 曾因自有 Transit Set decoder 顺序误报 ACK 失败，原失败保留。修正自有 decoder 后对相同原始 HTTP 字节运行未修改的共享 `verifySaveResponse`，严格验证 revn 7；后续关闭/重开回执为 `penpot-anchors-r01/review/receipt.json`。没有重造 ACK 或把失败 receipt 改为 pass。

新锚点板的官方 SDK PNG export 返回 4 字节 UTF-8 `null`，signature=`6e756c6c`，没有有效 PNG；无效的 `.png` 文件按失败证据保留，原因尚未定位。这与前一局部板的 download 90 秒超时分别记账。Parent/原 pipeline owner 接续 exporter 链，浏览器图不能替代正式导出。

本轮源 ZUI SHA 仍为 `97992ab7cbab62665f17b550ef6ae115ea513302c48db47c44ba2c6030cec383`。新增锚点业务未通过正式 ZUI 导出，Canvas inspector/preset/rebase/一次 undo 的真实 adapter、共享 style refs、插件检查、托管 Rust、正常 EXE/尺寸/DPI/双端测量均 pending，`passed=false` / `accepted=false`。Parent 官方队列交接消息为 `01a16db6-ec7b-7281-bfae-79fcafb3170f`，queued 不代表已读或授予桌面时段。所有自有 helper/browser 已退出；保留报告、缓存和失败证据，释放 0 B，生命周期仍 open。

## G01 待审增量与正式导出缺口

2026-10-24T11:52:38.583Z 再次独立认证 API 读回 revn 7 / vern 0、1434 objects、8 native components、shared colors/typographies=0；未写 Penpot。Parent 交来的 revn 0 是较早独立观测，原记录保留。

已形成单一原资产的 [Starship 增量提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/starship-class-adoption-r01.md)，patch SHA `8ef78627bedaacab57b41524d4b1a6a925ad1c6f0915b24adeebd130cb1da9a2`。候选只增加现有 import、3 个用途 class 及 Source inset 规则的局部作用域调整，97 ID/6 事件和其它节点字段保留；`git apply --check --whitespace=error-all` exit 0。该 patch **未应用、没有新的 Penpot 导出**。

实际 `isSupportedVisualPath` 只读探测接受 gap/background 等视觉字段，拒绝 classes、imports.styles、stylesheet selector。这个结论仅针对 source-owner visual-delta envelope，不否定普通 ZUI 文档保留序列化；Parent/原公共 owner 需交付明确的 class/import/rule 结构迁移合同。未弱化 source-owner/stale/实例 guard，也没有用 literal paint 复制共享主题来关闭 G01。该探测和静态可应用检查均不算托管插件/Rust测试或引擎验收。

## Canvas 域内源码接续，GUI 仍 open

基于已保存的 `canvas-anchors-r01` 业务设计，本轮仅在本域 session 和测试树增加预设事务候选。方法使用当前真实 preview 的源映射 Canvas 几何，扣除父 padding，依 UE 顺序重基/Shift/Ctrl，再通过既有 source/undo 一次提交。八项回归源码包含真实 v2 surface 布局、源 roundtrip、重开和 undo/redo 消费；尚未编译执行。当前仅支持单选、正尺寸、auto-size 关闭的原生 Canvas，其余明确返回错误并保留源；[完整限制和 diff](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-domain-r01.md)。新业务组件没有正式导出或接弹层；该源码 slice 时 Penpot 为 revn7，后续设计进展见末节，原产品资产 SHA `97992ab7…` 未因本 slice 改变。

独立公开 capability 本轮为 ready，但原 layout request 实际 query（chunk `84fde1`）仍为 `Current session authorization is pending`，无 Cargo；新增行为另用固定 `widget-ui-canvas-anchor-r01 / validate-editor-canvas-01` 和八项 filter，不让旧 layout 结果代替它。向 Parent 投递 G01 新交接在官方入口被拒（chunk `cd147c`：queue 已达 100 submissions，code -32600），没有 message ID，未记作送达。所有正式插件、托管 Rust、正常 EXE/DPI/并排差异和逐图验收继续 open。

新增域请求实际提交后，pre-submit 两轮因六处 Settings/粒子域变化被拒（`8b1158`），本域源保持稳定，Cargo/测试均未启动。同身份 query 仍 pending（`df0c01`）；PID 4932 已退出（`72dec8`）。失败与原身份见[完整回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-jenkins-attempt-r01.json)。2026-10-24T12:19:54Z 认证 API 复读 Widget revn 7 / 1434 objects / 8 components；公共 file `…607aac12` 仍 revn 0、仅 1 root、0 components/colors/typographies。公共 TreeRow 两个源哈希未变，不重复运行相同部署的完整导入。向 Parent 的新源码/请求通知也被 100 submissions 上限拒绝（`182eaa`），未送达；仅在本域既有交接合并记录，不操作队列数据库或改目标绕过。自有 client/helper 均终止，释放 0 B，保留待续输入及失败证据。

## 完整边界布局目标 r01，实际 revn9

2026-10-24T12:56Z 后在**同一分配 file/page**继续设计，没有创建新文件。实际 SDK 保存 ACK 到 revn8，关闭原 context 后新认证 context 重开，再仅修订这七板的工具栏/不可用状态/More 合同，保存 ACK 和再次关闭重开均到 revn9/vern0。独立 API 第三次读取核验 2843 objects、原8个 native components、colors/typographies=0；相对 revn7 新增1409 objects、**新 main=0**，原1434 shapes和原组件定义严格比较未变。修订写集合45个 shape均位于七板内部；未写公共文件、主题、catalog、导出器或产品源资产。

| 目标尺寸 / 状态 | 真实 board UUID | 复核结果与限制 |
| --- | --- | --- |
| 1280×800 / 空文档 | `b3c9f17a-7e25-8074-8008-d6c305149701` | opened 三列和左上下栈可见；首张 `Resize Slot` 仍两行，不能计排版修复。 |
| 900×620 / 空文档 | `b3c9f17a-7e25-8074-8008-d6c31c730477` | 保留相同拓扑；工具名已单行，Bind Widgets 页签换行/裁剪。 |
| 640×520 / 空文档 | `b3c9f17a-7e25-8074-8008-d6c338145bc0` | 保留 UE opened Dock；左95.7逻辑像素的 tab/search 明显不足，未自动合栈。 |
| 1280×800 / 搜索无匹配设计终态 | `b3c9f17a-7e25-8074-8008-d6c35397d610` | 无匹配文字可见、长 query 裁剪；未执行真实过滤/清空/切文档，G09 open。 |
| 1280×800 / Hierarchy 搜索焦点终态 | `b3c9f17a-7e25-8074-8008-d6c36b4a6ef5` | 蓝色边界可见；未执行键盘导航、IME 或焦点返回。 |
| 1280×800 / 锚点选择夹具与弹层 | `b3c9f17a-7e25-8074-8008-d6c389356344` | 真实复用原 Picker，16 linked presets 的键/min/max 全部核验；源测试夹具的选中行及字段可见，没有运行预览。 |
| 900×620 / 中文长文本 | `b3c9f17a-7e25-8074-8008-d6c3aef0c50e` | 标题及 Details 中文 glyph 可见；Hierarchy 长查询换行/裁剪，Engine 字体/DPI 未测。 |

七板合计342条叶/嵌套 native instance 引用，复用 Button `c572a930-f329-801a-8008-d6a46ea3d363`、Label `c572a930-f329-801a-8008-d6a4705761b0`、Picker `180e9578-8294-803b-8008-d6ae7d94d33b` 及其原 Preset。每板21个 native flow 容器，弹层板35个；Palette/Hierarchy/Details 各独立裁剪内容区。它们仍不是公共 Dock/Tree/Search/Property composite 实例，也未证明滚轮行为。没有复制动画/图表面或新建主题；实际共享 style refs/public library 仍未交付，不能把解析后的 source token paint 当作共享样式复用通过。

15条 Palette 标签来自实际 `editor_showcase/descriptors.rs` 的 layout primitive 声明，未验证实际运行 catalog 顺序。锚点字段来自现有 `canvas_anchors.rs::CANVAS_SOURCE`，明确为设计/回归夹具，画布保留空数据网格；不画假运行内容。Graph 已标真实后端缺失并使用既有 disabled 色，Source 仍独立。More 的 source node/event 为空，单独列三条 Designer overflow entries，未生成按钮执行或宿主绑定。弹层顺序保持 rebase→Shift pivot→Ctrl reset / one undo，GUI command 仍 pending。

两次早期 authoring 超时均0 ACK、API仍revn7，原失败回执和诊断图保留。随后采用有界分阶段 SDK 写入完成；不足以判定最初超时的完整根因。本轮七张 PNG 是实际重开后的1600×1100/dSF1浏览器视图，含选择框、邻板和不同zoom，**不是官方 Shape PNG或原生 EXE图**。七图均已打开逐图审阅，图像签名/SHA与16预设已独立核验，见[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-widget-toolkit-r01/widget-toolkit-design-review.md)、[native audit](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-widget-toolkit-r01/widget-toolkit-native-audit.json)、[原布局保存回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-widget-toolkit-r01/receipt.json)及[修订保存回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-widget-toolkit-r01/refinement-r01/receipt.json)。

八项设计源输入哈希未变；产品仍 v2/97唯一control IDs/6原事件/SHA `97992ab7cbab62665f17b550ef6ae115ea513302c48db47c44ba2c6030cec383`。新布局与业务组件尚无正式 ZUI 导出，不落回产品替换已有语义。gap6实验继续等真实EXE变化证明后恢复共享token。两项原Jenkins身份不变，本轮没有新Rust编译或公共插件测试。公共 composite/样式、官方导出/PNG、实际过滤与焦点/首尾滚动、域业务接入、正常EXE/实际150%DPI/双端逐图及1DIP比较继续open，`passed=false`、`accepted=false`。自有隔离浏览器/SDK HTTP host已退出，服务与共享桌面未动；保留失败及可复用证据，释放0 B。

### 新公共 worker 观察与安全重试边界

13:20Z后的独立核对发现公共TreeRow源码已有十个可选nil字段归一及其它修正，spec也有变化；当前worker产物SHA为 `442bed70a38d28c86a702002658c8d77598ac6205fe325b1696aedbd27b7b47d`，不同于旧失败的40d115…，未运行新的完整导入或据此宣称修复通过。公共文件同次API仍revn0/1root/0components/colors/typographies、isShared=false，Widget仍revn9。

重试前确认普通Import的replaceSelection会调用旧asset的 `removeNativeComponentAsset`；旧局部板仍拥有6个源导入主件，七张新板直接引用其中Button/Label共325次，锚点业务也依赖原Button。另起asset的native cache只扫描新asset的owned.components，会另建相同来源主件。此为源码/实例关系核查，未执行删除、未复现SDK删main后的GUI结果。公共owner需提供正式安全更新/追加复用合同，详见[前置检查与精确源码入口](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-widget-toolkit-r01/source-import-reuse-preflight.md)。本轮不删旧板/主件，不造第二套main，不用semantic策略关闭导出缺口；原失败、原Jenkins身份和产品open状态保留。

向Parent发送本次revn9设计及API提案的官方 `codex queue` 在13:18:23Z被100 submissions上限拒绝，code -32600，没有message ID，未送达。消息原件与[失败回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-widget-toolkit-r01/parent-notification.json)保留，不重复入队、改聊天数据库或换目标绕过；最新公共依赖核查以本域文件交接。

## 文字flow修订r01，实际revn11

2026-10-24T15:50Z在上述七个原board内继续修订，未创建新板或主件。用同字体的native auto-width临时文字量测Resize Slot，自然框67×16、glyph bounds 66.859375×16；首张1280原fixed文字实际高31.984375，对应两行，另五处原本已一行。保留96×28父按钮和原字体，以既有8像素padding与居中flex排布六处copy，文字使用auto-width；未修改Shared或公共Button主件，不将prototype覆盖当公共修复。

实际HTTP200保存ACK到revn11，关闭原context、新context重开后六处框均67×16，文字框居中偏差0、glyph中心最大偏差0.0703125，仅是同一Penpot控件内部几何检查。文件仍2978objects/8local components，严格仅变12个原shape，其余2966个shape和所有组件定义、source映射及Shared rev1/41objects保持；量测probe一次add/delete、不持久保留。101项真实changes和逐项检查见[独立诊断](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-text-flow-r01/text-diagnosis.json)及[保存重开回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-text-flow-r01/receipt.json)。

七张新浏览器图均已逐张打开查看，原七个board UUID保持，详见[逐图记录与七图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-text-flow-r01/review.md)。首张Resize Slot两行问题在设计端消除；900/640的页签及中文长查询裁切仍保留为G03/G09具体输入。图为1600×1100/dSF1、不同zoom的实际隔离浏览器，含选框和邻板，不是官方PNG、实际系统150%DPI或正常Engine EXE图；不计1DIP双端比较通过。原revn9图与失败作为历史保留。

新目标和flow修订未正式导出到产品，原gap6/97 IDs/6 events保持。两份原Rust请求在恢复后当前2790fb…胶囊各QueryOnly一次，仍exit2/pending且identity未变；[原身份对账r03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/managed-validation-recovery-r03/summary.md)已获原owner/Parent官方队列投递确认，不是准入或测试回执。未运行Cargo/公共插件测试/桌面/正常EXE，自有浏览器已退出；官方导出/PNG、公共样式/字段slots、真实交互、实际DPI和逐图比较继续open，passed=false、accepted=false，释放0 B。

## 被动页签溢出修订r01，实际revn12

2026-10-24T16:15:41Z起在原七板修订28个页签原copy。UE依据是`SDockTab.h:113`默认Ellipsis、`SDockTab.cpp:703–727`左对齐/垂直居中Label和`SDockingTabWell.cpp:304–342`均分宽度；搜索则为`SSearchBox.cpp:26–39`真实可编辑输入，未沿用被动省略。复用原`penpot-text-overflow.ts`的measuredEllipsis/cachedTextMeasure，24条实际Roboto700/13.333333 native字体量测生成显示值；完整原标题保留在未改root.name及本页journal.plan.source。原字体、实例引用、source metadata、Dock几何和14搜索值均保持，不写canonical全文属性或公共主件。

一次真实HTTP200 ACK保存到revn12/vern0，关闭authoring context、新context重开后28处文字框及glyph bounds均高16、宽不超当前lane；其左侧inset和垂直居中的内部检查误差为0，仅属Penpot内部几何。2978个对象仅56个原shape改变，其余2922个对象、原8个主件和Shared rev1/41对象保持；临时probe一add/一delete，540条真实changes逐项通过范围核查。后来独立认证API再次读回同一rev12及原对象闭包，见[保存重开回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-tab-overflow-r01/receipt.json)和[独立核查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-tab-overflow-r01/design-audit.json)。

本case实际读取两次浏览器Roboto700 WOFF响应，各HTTP200/78560B/SHA `fda7e86781dd30503ebc92b78adb3cbda2d7a6dfd05359c6b3a18b8213855ae2`，按当前get-font-variants精确resource ID关联normal-700。同时独立取得该注册行SFNT 162796B/SHA `2c7eb115902b569f184ee8faad5a710345b5411922d68f5ce1c5db6369a55861`，匹配原font manifest；两者编码不同，未当作同一字节流。原editor-ui.ttc指纹保持。此证据只覆盖被动页签使用的Roboto700实际响应，不表示中文、整套字体或Engine atlas ready，详见[字体证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-tab-overflow-r01/font-response-evidence.json)。

首次尝试在第10条量测后收到非任务SDK success消息并停止，0 ACK；隔离浏览器退出后独立API仍rev11/2978、全部对象哈希相同、journal未写，失败未擦除。限定任务消息类型并保持requestId核对后，第二次完成上述实际保存。独立检查另保留一次30与29.999999999999996严格浮点相等失败，改为1e-9数值等价核对；它没有设计写入。全部失败和真实七图逐张记录见[review.md](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-tab-overflow-r01/review.md)。

640仍为UE opened Dock目标，未自动合栈；被动标题的省略不证明tooltip/键盘/tab溢出入口可达。14搜索shape未改，长中文查询的换行/裁切继续G09；共享原生Search/Tree/Property整区接入、字段slots、canonical非破坏式import/export、旧runner与新dist同配置兼容、官方PNG、真实事件/ACK、anchors/undo及动画后端仍open。本轮没有公共插件构建/测试、Rust/Jenkins重投、桌面、EXE或实际144DPI；原request/identity/失败保持，passed=false、accepted=false，自有浏览器已退出，释放0 B。


## 18:13 原 owner 预构建入口能力对账

本域已核实原Jenkins owner三份 [预构建receiver证据](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/plugin-manifest-validation-20261024/workspace-entry/widget-penpot-prebuilt-receiver-20261024/receiver-proposal.md) 的给定SHA。当前只读e0635d5d…selector与11pins同原审查，supportedCall/schema/native receipt均未交付。管理validate/generate/Editor preview没有本案Penpot入口，固定Level owner、旧mjs非空续接与自行起host约束保留；认证state输出写入也须公共owner纳入新的受限不可变行为。全部旧sealed、源与失败均未改，未触发任何执行。

原同页source specimen `c572a930-f329-801a-8008-d6a46839c961` 的 `designer-gap-edit-session/receipt.json` 是一份既有真实局部SDK保存回执；两次历史HTTP200的原始request/response哈希和纯保存协议校验均通过，rev4/5，没有新网络/保存。其planned源SHA97992a…仍等于当前原资产，原10输入9同加已知单gap改动；97唯一control IDs/6events保持。这不改变原两份failed mjs回执，也不提供旧runner所需workspaceUrl或完整passed/native Job/EOF。局部成功范围继续保留，没有伪造或抹掉历史。

本轮实际交付为 [消费者对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/prebuilt-receiver-reconcile-r01/reconciliation.json)、[现有板/保存输入](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/prebuilt-receiver-reconcile-r01/board-and-receipt-inputs.json)、10项显式ZUI导入指纹和短二进制摘录。原source板与Shared板不能互换身份；声明导入清单不是注册生产闭包，当前membership/资源/兼容/native事实未知，不预填请求或caseRef。完整边界见 [验证记录](widget-ui-validation.md#预构建-penpot-入口原-owner-对账仍-unsupported)。本轮无浏览器/认证/服务/桌面/编译/新Jenkins请求，0新图/0产品通过、释放0B；整族继续open。


## 官方 Shape PNG 的实际分支诊断 r02

21:45与21:51Z在原file/page的原1280×800板`b3c9f17a-7e25-8074-8008-d6c305149701`调用真实`Shape.export({type:"png",scale:1})`，均返回`Uint8Array(4)`、字节`6e756c6c`（literal null，SHA `74234e98afe7498fb5daf1f36ac2d78acc339464f950703b8c019892f982b90b`），不是PNG。此前等待download的超时是另一阶段失败，不能全部归因于该字节问题。

本轮从实际加载的shared ESM单例只读取得`wasm-export/v1=true`、`render-wasm/v1=false`、`wasm/ready?=false`，前后相同；CDP在SDK调用区间只观察到一个blob Fetch，状态200、MIME image/png，没有HTTP exporter请求。当前HTTP main/shared字节分别与原物理发行物hash一致。Plugin `shape.cljs`仅检查wasm-export开关，WASM render未ready时返回nil，`create-blob`把nil包装为四字节字符串null；这条实测分支定位在HTTP exporter之前。普通asset PNG请求与该导出分开，不能拿其签名关闭导出门槛。完整[实际运行证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/native-png-export-r02/runtime-probe-r03c.json)及[分支提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/native-png-export-r02/export-branch.proposed.json)保留。

现有UI exports/assets的`wasm-export-enabled?`已同时检查文件render-wasm；本轮只形成Plugin同一条件的[单文件未应用diff](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/native-png-export-r02/export-branch.proposed.diff)，没有写dev/Penpot、公共插件、发行物或运行时方法。此条件修复后SVG文件将进入原exporter路径，其代理/服务就绪和当前source/资源兼容仍须由Parent与原public owner取证；两个开关均开但context未ready的拒绝/等待仍须验证。未为取得PNG启用WASM、关闭guard、重启服务或以浏览器图替代。

当前读回Widget rev20/2978对象/8本地主件、Shared rev5；完整前后data及profile插件注册表相同，0 update-file。原五个服务进程PID/原生出生/exeSHA与Parent current.json全匹配，原deadline未延长。两张[浏览器诊断图及逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/native-png-export-r02/visual-review.json)只证明真实会话：第一张为71%原1280空文档板，第二张为13%全页及SDK结果，不是正式Shape图、正常EXE或实际DPI。两个helper前置失败（delimiter与inline ESM预期错误）原样保留，均在SDK调用前退出；随后r03c取得实际分支证据。产品97ID/6events、两原测试身份保持，受管测试、正式PNG/EXE/150%DPI/双端及1DIP门槛继续open，passed/accepted=false。


## 七张 Toolkit 板的真实 Shared 颜色引用 r01

22:02Z沿原file/page将**147个既有非组件flow容器**的背景经原生`LibraryColor.asFill()`引用Shared rev5四种已发布颜色：surface.0（42）、surface.1（70）、border（14）、surface.input（21）。具体UUID、原目标ID及实际resolved值见[完整前像](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-shared-surfaces-r01/before.json)和[消费回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-shared-surfaces-r01/receipt.json)。只写原147 fills及7条明确localAssignments的域journal，无创建/删除对象、改main或另建主题；原historical theme-source与canonical source metadata保留。

实际update-file保存**rev21/154changes**，原始request/response均保留，生产`verifySaveResponse`按file/session/full changes/revision验证；随后关闭、fresh context重开，同file/page仍rev21/2978objects/8 local components，saved/reopened完整data相等。只移除这147 fill ref及7 journal后，整个data与rev20深比较相等；2831个非目标对象、12 Shared heads/99 shapeRefs的完整子树及所有原文字/font/position-data/source/几何不变。147个SDK几何亦相同；Shared完整数据及插件注册表未变。它证明本域显式颜色引用持久化，不是主件默认继承或全局A/B通过。

[七张实际图及逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-shared-surfaces-r01/visual-review.json)已全部打开：1280空文档/搜索空结果/焦点/anchor夹具、900空态/中文和640 opened Dock均保持。640搜索文字及中文长查询仍右裁切，窄页签省略、More/独立滚动/焦点返回未验证；UE chrome/图标/密度仍有差异。蓝框、No matching widgets、16预设和空Designer分别是设计状态/夹具，不替代实际输入、筛选、Popup单undo、实时Preview/Graph/动画。真实色引用不解决这些缺口；沿公共Search/Tree/typed slot与源码codec owner，保留稳定实例，不加私有覆盖来关闭依赖。

[源与身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-shared-surfaces-r01/source-inputs.json)、[本轮摘要](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-shared-surfaces-r01/summary.md)记录当前22只读/保护输入、3自有JS语法检查及原服务出生/EXE/deadline。原产品v2/97唯一controls/6events、两原测试identity保持，未运行新QueryOnly/Cargo/受管插件或正式导出；旧四字节null失败和未应用PNG分支提案不变。本轮PNG为1600×1100/dSF1、fit zoom71/98/134%的真实Penpot浏览器画面，非官方Shape PNG、目标客户端、正常EXE或实际150%DPI。正式导出/产品加载、双端逐图差异与≤1DIP均open，整族passed/accepted=false。


## 锚点移出保留键盘焦点，实际rev22

22:40:57–22:41:35Z沿原分配file/page完成一个原生editable状态板，使用既有Picker/Preset实例和flow，未复制main。新board `392bc57b-e3cc-80a2-8008-d748e7191288`是原hover64板的pointer-leave/rest48终态；16个preset、原min/max、glyph及键盘焦点保持。原始保存HTTP200/rev22/706changes，随后关闭authoring context、新context重开，完整data hash同为`5237f1916db12c853f047fe5500c506e6a66dc20c392198ab6d0dceaab408229`。[原始报文、身份及保护比较](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/receipt.json)保留：3314对象、336新增、2977原非根对象完整不变、8原main、334新sourceRefs、12旧Shared头及子树不变；仅根children追加和本次journal为允许的旧数据变化。Shared rev5及插件注册表完整保持。

[原hover64图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/hover-before-browser.png)与[新return48图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/return-state-browser.png)均已实际打开；68焦点边界、48内框、15glyph和Shift/Ctrl提示可见，分别是settled设计状态，不是鼠标动画或实时preview。[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/review.json)保留两PNG SHA/1600×1100/212% fit及来源界限。官方PNG的旧null失败不被这些浏览器图替代。

资源响应200仍不足以判ready：本次源Roboto400 WOFF76344B/SHA `b71db71a9568c29f6aa3efb4bf25f25efde84c659930dfcc3851f90e09014144`匹配原注册资源；TTF/WOFF内部PostScript名均Roboto-Regular，而新标题CDP为Roboto/custom/34glyph，exact font check=false。十条normal400 CSS含注册WOFF和Google子集，尚未确认最终URL→字形来源。[两次只读probe及分析](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/font-source-analysis.json)保留该缺口与本地分析helper的两次失败；没有安装fontTools、注册新字体或改公共CSS。两图是设计状态诊断，fontsReady/passed/accepted均false。

本轮仅追加3份本域文档及证据；原product v2/97唯一control IDs/6events、r03三Rust、Runtime Bool sampler四源、两原Jenkins identity保持。[源清单](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/source-inputs.json)和[静态验证](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/validation.json)提供精确范围；未运行Cargo/受管公共插件测试/新QueryOnly/桌面/服务生命周期。源导出、实际popup/焦点、运行预览、Graph/动画、正常EXE/150%DPI及双端≤1DIP继续open。


## Roboto 实际字形来源隔离对照 r05

2026-10-24T23:04Z与后续一次独立headless对照沿原file/page只读重开，Widget读到rev22；两次前后完整data均为`5237f1916db12c853f047fe5500c506e6a66dc20c392198ab6d0dceaab408229`，Shared完整data亦保持。没有改原字体、样式、节点或position-data。临时FontFace别名和屏外SVG仅供测量，均显式删除并关闭context，不保存为Penpot样式/主件或产品字体系统。

标题为原`392bc57b-e3cc-80a2-8008-d748e719128a` / `Keyboard focus after pointer leave`，源font-id仍`custom-495ff285-7c71-49bc-bc4e-df20017e2d66`、normal400；实际CSS字号13.3333px。CDP与同文同字号自然advance结果如下，**不是正常EXE或整窗DIP验收**：

| 对象 | CDP PostScript name / glyph数 | 自然宽度 | FontFace检查 |
| --- | --- | --- | --- |
| 原生Penpot标题，操作前后相同 | Roboto / 34 | 202.7745819091797 | false |
| 原注册engine WOFF的临时独占别名 | Roboto-Regular / 34 | 204.47984313964844 | true |
| 页面既有Google Latin WOFF2的临时独占别名 | Roboto / 34 | 202.7745819091797 | true |

本标题原生字形与Google对照相同、与原注册engine face不同，自然宽差为**1.70526123046875**。注册WOFF的真实响应200/76344B、SHA `b71db71a9568c29f6aa3efb4bf25f25efde84c659930dfcc3851f90e09014144`与原文件匹配；Google Latin为200/43136B、SHA `1404ca348bd75ef836f4dd8b6f2cc719458642d1237c368296b2fc652dca47dc`。因此资源已下载和font-id正确不能证明实际使用了引擎face。原SVG仍带textLength，不能用保存框宽或缩放截图抵消这个自然度量差。

实际CSS顺序先是`custom-495ff285...`的Roboto400声明，随后`gfont-roboto`的九条同名子集声明。已从实际HTTP200加载的main/shared/workspace三份JS复算SHA，均匹配原服务的现存文件；其对应source map显示`app/main/fonts.cljs:293–329`按family生成custom CSS，`styles.cljs:130–154`与`svg_text.cljs:89–95`以family设置文字。具体来源/行片段见[源码追踪](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/font-family-selection-r03/source-trace.json)。这定位了font-id选择与实际CSS字形选择之间的同名竞争边界；没有单凭注册receipt或源码推断字体已修。

[首轮原始probe](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/font-family-selection-r03/probe-receipt.json)、[Google对照](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/font-family-selection-r03/google-contrast-receipt.json)和[结论/owner提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/font-family-selection-r03/hypotheses.json)保留完整结果。每次各有1份response.body取数失败，资源未定位，原错不抹除；受检两种字体和三份JS另有完整有效buffer/hash。尚无浏览器内部URL→glyph调用栈，不把同name/advance对照外推到全部字形、CJK/fallback/atlas或EXE。原r04中文样本/clip解释继续按其范围保留。

公共font/renderer owner需沿原前端family生成、HTML/SVG、position-data与正式codec核对稳定内部CSS身份及重算；保留原source family、注册fontId/variant、engine manifest/face和Shared typography引用，不能只放宽插件selectRegisteredFont校验或做本族私有改字号。此为精确未应用提案，本聊天未改公共源码。fontsReady、正式导出、正常EXE/150%DPI与双端≤1DIP仍open，passed/accepted=false。

