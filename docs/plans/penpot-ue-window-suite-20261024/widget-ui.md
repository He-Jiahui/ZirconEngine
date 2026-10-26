# Widget / UI 窗口族

## 2026-10-25：Palette 原生键盘空白/清空回归已落源，X消费者提案重基到Parent后继

本轮实际只增原 `app/tests/ui_asset_builtin_palette.rs` 两项应用消费者回归：36836B/72c5d097… → 44365B/7b6d0837…，原四测试及全部原字节完整前缀、CRLF保持。沿真实asset/EditorManager/pane hit和现有native key/text入口，断言ASCII与Unicode空白原query、Ctrl+A/Delete清空和caret/焦点保留、中文/emoji scalar删除、blur晚输入拒绝及重新focus后旧opaque query target拒绝；读取实际Session query与文档/undo状态，不手填clear flag或另造Surface producer。[实际改动与限定](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-native-keyboard-whitespace-r01/review.md)及[最小验证意向](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-native-keyboard-whitespace-r01/validation-intent.json)保留。guard0/0/0、fmt/reverse/diff0仅静态，新增2项执行0；无正常OS窗口、X点击、IME预编辑/提交次序或present通过。

Parent已落raw-whitespace两源62074cdd…/7e5350a2…未改。旧19路径候选17前像保持、2为已核Parent后继，纯内存重建全部原proposedAfter一致，现[r03替代提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/palette-native-clear-wiring-r01/proposal-r03-parent-whitespace-rebased.json)剩18拟写、31411B补丁SHA449ce3ca…：已落search_clear_action完全无delta，不重套；19 stdin fmt和apply-check0，公共源写0，未以未应用candidate跑guard。metadata→DTO→converter与clear配对release/X功能仍待Parent公共集成，placeholder trim/always-glass/leading native glyph及32px夹具边界各自open。

实际builtin文件4→6定义是透明新后继；原G09 #412/27+4的输入不改，声明现50/51，仅此文件后继，不借旧结果验证新增2项。#415原63声明均保持，其Canvas过滤不验证本2项。新意向request/operation=null，本轮无submit/query/Cargo/服务/桌面/freeze。原v2/97ID/6events、rev25–29失败与其后设计均保留；EXE、字体、150%DPI/双端与1DIP继续pending，整族passed/accepted=false，释放0B。

## 2026-10-25：当前 Canvas 38 项已唯一投递，#415 等待真实资源

当前源码修订以新业务 `widget-ui-canvas-native-pointer-r01` / `validate-editor-canvas-native-pointer-01` 唯一正常POST至 queue19911 / zircon-management#415，input `a2164349bde38a6dca564f120ca8b950be15381817fdb3e0dcedd3cdff340099`；客户端6176已wait exit0，durable marker为submitted。1 owned +62 readonly及当前Canvas47逐字节核对；单 `zircon_editor` / defaults-off / features[] / Windows native lib / test profile / locked / jobs1 / threads1 / filter `canvas_anchor`，38静态定义=25域+5painter+2open+6input，尚无执行结果。详[当前命令及实际投递](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-pointer-managed-r01/handoff.json)。

首客户端8840在submit前因externalRoots路径字符串非Windows规范形式exit2；原输入与失败完整保留。原marker/operation精确缺失并核对pinned调用顺序后，仅修externalRoots/buildRoot格式，同一新业务实际POST一次，不改源码、测试/argv或旧请求。[路径修正证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-pointer-managed-r01/canonical-path-reconciliation.json)不当作服务端准入。

16:02Z本#415实际State已记录 `resource_capacity` / sourceDigest `ffad7c6cf6ad0daa95e8f0a98b1b5994a0ba1d7aa502836841498b7758df9c44`，仅是捕获时点观察，非最终compiler输入。16:05Z[同快照只读资源证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-pointer-managed-r01/resource-readonly-observation01.json)为CPU1/RAM2147483648B/disk4294967296B/物理E，4 active heavy / max4，reservation=null、prelaunch unlaunched、ownedNativeLaunchClaimed=false；本次未查询writer/maintenance hold，不由缺席推断释放。无已证Rust执行或native Job/双EOF，不扩写或重放#383/#389/#411、旧Layout/Canvas或G09#412；不清锁、调policy、freeze或占桌面。当前Penpot rev48及v2/97ID/6events保持。正式导出、引擎字体、正常EXE、150%DPI/双端逐图及1DIP继续open，整族passed/accepted=false，释放0B。

## 2026-10-25：Canvas 原生鼠标 / 指针归属已落源，slot #411 原终态保持

本次只改已移交Canvas47内原 `window/canvas_anchor_popup/input.rs`：17554B/e18466f8… → 36123B/bf7766c4…，LF保持，其余46条前像不动。普通原生mouse translator保留pointer ID=None，原native dispatcher按默认指针处理；popup原先因此未保存按压且ClickAway没保留待吞Up。现统一默认指针，先验证实际window/source与pointer owner；外来Down/Move/Up/Cancel和Touch/Pen不退另一指针状态，旧tail释放保留新popup/ticket/另一pointer的按压。原一次ticket、焦点/关闭、Session/history和v2 schema保持，未新增业务事件或runtime。

[实际实现与责任边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-pointer-ownership-r01/review.md)、[逐路径application](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-pointer-ownership-r01/application.json)和[当前验证意向](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-pointer-ownership-r01/validation-intent.json)保留4新增回归：真实winit鼠标无ID的点击/CursorLeft、外来pointer隔离、旧tail与新popup、ClickAway释放只吞一次。当前38静态定义（原34+4）/执行0；原2项测试完整保留。fmt/diff/补丁反查/guard0仅静态，不称类型、EditorManager、正常OS窗口或present已过。G09声明51/51保持，本轮不代投/查询#412、不扩旧Canvas/Layout身份。

原slot #411本轮只读API已FAILURE，543B原archive SHA3d05ecbe…为unknown/management_operation_interrupted，原因HEAD/full index/source membership捕获变化；两状态分别保留，不归责、不重投原5846ba4d。Rust/native/resource/双EOF无证明，新slot a38e源码不借旧输入验收。原Penpot rev48/ZUI97ID/6events保持；官方导出/字体/真实EXE/150%DPI/双端与1DIP仍open，passed/accepted=false、释放0B。

## 2026-10-25：slot 运行消费断言已落源，旧 #411 输入保持

已应用原单文件runtime-consumer候选，`session/tests/slot_migration.rs` 后继为15764B/SHA `a38e71958e992fd71cc64a6f3c8ae36a7670daf797539c0133c65e785b98980d`，7个原测试函数名保持。原六个生产源码未改，25保护输入及G09原37+14声明当前51/51保持。测试沿原Session/v2重建，用真实PreviewHost只读source mapping和唯一UiSlot边断言Linear标准value=2.5、Grid行列span、Canvas→Free placement、Overlay z_order及padding/alignment/order；保留原stretch_value扩展，但它不被Runtime parser消费，不再以源往返推断运行权重。没有新增runtime、fixture producer、可变Surface、store/clock或schema。

[本次实施与边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/hierarchy-slot-runtime-consumer-r02/review.md)、[逐路径application](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/hierarchy-slot-runtime-consumer-r02/application.json)固定旧93e6→新a38e与原LF；原5883B补丁b4901973…、格式/diff/guard0仅静态。7定义执行0，未声明类型或GUI通过。原slot #411/input5846ba4d保留，它不覆盖新源码；本轮仅记当前validation intent，没有新request/op、submit/query或重放。G09 #412仍由原执行owner对账，其已记录source/resource capacity阶段与较早git-busy archive分开，未代投/清锁/改policy。

原Penpot rev48及原ZUI/v2/97ID/6events保持；正式codec、真实Preview/handler/present、正常EXE、引擎字体、150%DPI/双端逐图与1DIP仍pending。原失败保留，整族passed/accepted=false，释放0B。


## 2026-10-25：原 Widget rev48 的长标签与中文行框接续

原 file `b4a1e3ce-9e6a-81fe-8008-d69d60d24223` / page `b4a1e3ce-9e6a-81fe-8008-d69d60d24224` 从rev45经3次真实HTTP200保存到rev48，关闭fresh重开与随后认证API同data SHA `ab3278a5e2d23b04c6dcf4f109e8a6f30f7517d8f64d932bc2622eab82ade3b6`。20个候选用原字体声明实际加载并测量，13个完整单行advance加1px余量可容纳，才改原copy root row/nowrap/center与Text fill/auto/auto-height；7个超宽者保持原样。26对象有布局后继、3519非目标和整份板外data保持；8旧main、17Shared heads/source refs、Sharedrev5全data及插件registry保持。原文字/字号/paint/组件与style refs未改，无新对象/主题/detach。

详[本轮实作与差异](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-intrinsic-captions-r06/review.md)、[逐对象身份与前后hash](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-intrinsic-captions-r06/editable-design-delta.json)及[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-intrinsic-captions-r06/visual-review.json)。900默认/640默认/900中文各before/fresh共6原图及3并排/3原RGB差异图已全部逐张打开；只比较Penpot两个revision，不是正式PNG、EXE或1DIP证明。可视区外Grid/Canvas等仅有保存后的原生布局检查，不代替滚动验收。

640五个长Palette标签及长Details、900中文Details仍超宽；窄tabs/Search/中文query、More/scroll/caret/focus/IME和UE完整chrome继续open。本轮产品Rust/ZUI未改，原39714B/v2/97ID/6events及SHA `97992ab7cbab62665f17b550ef6ae115ea513302c48db47c44ba2c6030cec383`保持。字体buffer/加载与browser advance不证明引擎font/fallback；原#411和G09#412身份/范围不变，#412由原执行链对账，无本轮代投。正式导出/正常EXE/150%DPI/双端逐图与整族passed/accepted仍false，原失败保留。


## 2026-10-25：Hierarchy 换父级 slot 迁移已落源

实际七路径修复见 [source application](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/hierarchy-slot-migration-r01/application.json)，补丁 SHA `c2f04267e8911b68c503f71c1d75b18fbc0fe9af1566d2eba9e50c983f1fa8b4`。IntoPrevious/IntoNext/Outdent 在移除节点前复用原 native slot admission 选取真实空闲 mount；Outdent 的只读可用性同步检查祖父容器容量。迁移仍走原 Session 单文档 clone、原结构命令/v2 serializer/source buffer/selection/history，不增加 store、runtime、时钟或格式。

相同已知 native slot family 保留原边属性；跨 family 保留 padding/alignment/order，Free↔Canvas 保留定位六字段，Overlay↔Canvas 保留 z_order；不兼容字段按现有 v2 消费边界剔除。迁移清除旧 name/slot_name，使原 serializer 写入实际目标槽名。未知、动态、组件及 MUI 依赖的 family 只迁移通用字段；几何重基、typed Slot Details、拖动/GUI及 UE 视觉一致性仍待验证。UE 参考为 WidgetBlueprintOperationUtils.cpp 的旧 slot 导出/新 slot 导入路径，未改只读树。

七个新增功能回归定义覆盖 v2 写回重开、原 nodes/events/components 保留、Linear/Grid/Canvas 兼容、一次 undo/redo、named mount 和满槽无修改。当前执行0；7路径格式/补丁预检和静态 guard 0 errors/0 warnings/0 suppressed，不是类型或运行证明。root wrap/unwrap 完整方法、palette_state 原15597B前缀及25个保护输入均原样保持。产品 ZUI 仍为原 SHA `97992ab7cbab62665f17b550ef6ae115ea513302c48db47c44ba2c6030cec383`，97稳定ID/6events保持。

新源将按独立 slot-migration 身份准备最小 Editor/defaults-off/features[]/Windows lib/test/locked/jobs1/threads1 验证，精确 recipe 和当前源以本目录 preflight 为准；没有借旧 Layout/Canvas/#383/#389/#406 扩写或重投。原 #406 API FAILURE 与 operation unknown/capture interrupted 保持历史。Penpot rev45 的实际设计回执保持；正式插件导出、正常 EXE、输入/IME/focus/capture、150%DPI和双端逐图/1DIP仍 pending，整族 passed/accepted=false。


## 2026-10-25：原 Widget 文件 rev45 的实际 caption flow 接续

原 file `b4a1e3ce-9e6a-81fe-8008-d69d60d24223` / page `b4a1e3ce-9e6a-81fe-8008-d69d60d24224` 已从 rev30 经单例 rev31、14 批真实保存到 rev45；关闭原 context 后 fresh 重开，同数据 SHA `4761ad43117f7acdc10be2b8542ba1d40d85aabbb68ab2f9fda3f078102fb282`。本轮总计201个既有本地 copy 的 root/text 共402对象；其余3143对象及整份非目标 file data 不变，8个既有 main、17个 Shared heads/source refs、Shared rev5 全 data 和插件 registry 不变。未创建组件、平行主题、字体或新对象。

报告根：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-caption-batched-r05`，详 `editable-design-delta.json` / `receipt.json` / `visual-review.json` / `image-comparisons.json`。实际6张 Penpot before/fresh workspace 图及3并排、3原始RGB差异图均已逐张打开复核。它们只证明本轮真实可编辑设计及保存变化，不是正式 shape PNG、Penpot/EXE 双端比较或1DIP验收。

修复范围是菜单、Save/Undo/Redo、Palette短条目、Details短标签和动画页脚的原生行框。root 使用 row/nowrap/center，Text 使用 fill/auto/auto-height，原横向 inset、root/Dock尺寸、文字、字号、paint、component/shapeRef和源 metadata 保持。窄窗 tabs/Search、长文本/CJK、More/滚动/焦点/IME及UE完整chrome仍 open。

本轮没有改产品 Rust/ZUI。原资产 SHA `97992ab7cbab62665f17b550ef6ae115ea513302c48db47c44ba2c6030cec383`、v2/97唯一control IDs/6events保持。独立 root-wrap 原请求 #406 已实际API FAILURE，533B archive SHA `9ef368e43c4019012ecd6ab6fbff73a69bf8dbc83841ead78d2664575116513a` 的 operation 为 unknown/management_operation_interrupted（capture期HEAD/index/membership变化）；8定义0已证执行，未重投。下一域切片是父级切换 slot 属性迁移；当前仅完成源契约盘点，未声称已实现。整族 passed/accepted=false。


Hierarchy 当前四路径修订已接真实 document.root 的 Wrap/单子项 Unwrap：完整原文档上先预留 node/control ID，再更新原根及原 v2 serializer/history/selection；原根和后代、绑定与组件引用保留。新增3项回归，原1项 root 可用性测试两条期望同步调整，原测试11326B前缀保持。四路径格式/guard/补丁反查通过，Rust执行仍0；原中断 Wrap identity 不扩写/重投。详见[实际实现与消费者复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/hierarchy-root-wrap-r02/review.md)。仅 document root 源码分支有后继；组件根/多选/named-slot、native enabled投影及正常EXE/几何/焦点/150%DPI/双端仍open。此前“root gap”记录保留为旧时点。

原 Penpot file/page 已由 rev29 真实保存、关闭并 fresh 重开到 rev30：七块 Toolkit 板中40个旧实例改用 native row/nowrap/center 与 Text fill/auto/auto-height，原23/28高固定框及3/8向下越界收敛为16高原生行框。原字体/文字/样式、实例尺寸、source/shapeRef保持；3545对象、8主件、17 Shared heads不增加，3465非目标对象及Shared rev5完整data不变。六张1280/640/900中文 before/fresh图已逐张打开。640窄页签/搜索、中文query裁切及UE chrome仍open；原ZUI未改，正式导出/EXE/DPI/双端/1DIP及passed/accepted仍false。见[实际复核记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-hierarchy-designer-flow-r01/review.md)。

Hierarchy Wrap With 已修一个稳定身份缺陷：原实现先移除子树再分配容器 node/control ID，可能重用原节点或后代的身份；现在在完整原文档上先分配两项 ID，再走原 v2 写回、选中与单次 history。[本次两路径实现与复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/hierarchy-wrap-identity-r01/review.md)保留旧测试5736B前缀，新增两项 v2/绑定/组件引用/重开/undo/redo 回归，当前0执行。格式、guard、补丁反查通过；15依赖/保护输入及6旧身份与终态控制未变。UE root/multi-selection/named-slot、正常EXE/150%DPI/双端仍open；不改变97ID/6events，也不应用Palette clear公共候选。


Palette clear生产链新增精确发现：原v2已声明has_clear_action，但ViewTemplateNodeData缺字段、retained转换固定false，且原生pane只有Press没有clear配对Release。已形成[19路径完整未应用consumer候选r02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/palette-native-clear-wiring-r01/review.md)，含typed source metadata→DTO/materializer/converter、原pointer/source/frame的Down/Up、原query/焦点链及5拟增回归；14额外公共路径只交Parent。32307B/SHA aaa219dd3178c74a66f8e7602e331d3b21544eaf9ac05932e66615e7eea07efd，19stdin fmt/apply-check0，未覆入源做guard/类型或执行。实际checkout仍为G09/actual-field r02的27项定义；候选32项不扩旧身份。IME/popup/ACK、公共left geometry与EXE/DPI/双端仍open，原Penpot rev29保持。

当前 G09 实作 r02：已在原转交的两个消费者核实际绘制模型中的 query 控件、edit/commit 路由、启用状态及有限正尺寸；同源 pane 不再单独授权已移除或被 paint override 替换的字段。旧 query 目标退休，同源连续输入保留 caret，selected/page 原 snapshot/frame 检查保持。 [当前字段资格r02静态回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/palette-current-field-r02/review.md)固定73保护输入/6旧identity-control保持、guard 0/0/0及87源依赖指纹。格式/精确补丁检查通过，类型与27项测试未执行；实际EXE/150%DPI/官方导出/双端1DIP及整族passed/accepted仍false。此前25项和各旧revision只代表原时点。

最新Penpot实作（2026-10-25）：原file/page已保存至rev29/3545objects/8local mains/17Shared heads，并两次只读fresh重开。新增900×620 Palette clear/focus/hide可编辑板，使用原Shared SearchInput×3、Button×2和已发布样式；原3496非根对象、8main、12旧heads及板外完整data保持。查询、clear→返回焦点、hide只退休文本目标三态均已逐图复核；按实际SSearchBox/Starship将glass与clear按空值切换并左对齐。所有早期遮挡、右对齐和辅助核对失败保留，未重放成功保存；[真实保存链与身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-palette-clear-focus-r01/source-mapping.json)、[五图复核及未完成项](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-palette-clear-focus-r01/visual-review.json)。×仍是原Button裸caption覆盖，32px共享验证规格不是密集Palette正式度量；原生输入、官方导出、正常EXE、150%DPI和双端1DIP仍open，整族passed/accepted=false。另交[两路径raw-whitespace clear未应用diff](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-palette-clear-focus-r01/public-clear-whitespace-proposal-r01/proposal.json)，未越权写公共painter/dispatch；原ZUI/v2/97ID/6events不变。

当前 Palette 原生文本目标退休 r01 已在转交的4个现有消费者落源：hide/exit、原生失焦及确认关闭清理旧字段，拒绝关闭保留；不重置查询、不调用 preview Stop/Cancel/finish，也不改原文档/history。5新增定义执行0，静态格式/精确guard/diff通过；[实施、逐路径哈希与实际边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/palette-native-input-retirement-r01/review.md)。G09当前25定义及原builtin4、关联Canvas34分别准备，不能借旧operation代验。旧row/page在hide/show后的有效期、IME/多pane/文本视口、正常EXE/DPI与整族验收继续open，以下旧“未应用”和计数按历史保留。

G09现已依Parent明确授权落实原r02的37条路径（35既有＋2新模块），38前像及全部原EOL后像精确匹配；finder可见性未加第二次修改。完整SearchField输入→同源snapshot/当前页→borrowed hit/frame→注册native window/active pane/source→原Document guard/Session受检选择已接入源码。见[实际落地和保护输入](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/view-retained-integration-r03/review.md)。10条授权Canvas重叠按签认稿修改，43条未写保护输入、原完整Canvas方法后缀、Tree helper、产品v2/97ID/6events及主题保持。37条rustfmt/精确guard（0/0/0）/diff检查通过；8新增定义和原相关12项共20，另原4个builtin Palette消费者均执行0。当前执行范围已交原Jenkins owner正常准备，尚无新request/operation或准入回执，旧Layout/Canvas/#383/#389身份不动。真实GUI过滤、native生命周期/焦点/IME/横向滚动、正式导出/EXE/150%DPI/双端仍open。

当前产品切片 hover/focus r03：已修真实 Canvas popup 的 Move 把键盘选择改成鼠标所在项的问题。现在 Move/Leave 仅更新 hover/press，End 后移鼠标再 Enter 保留键盘预设；点击仍要求原 pointer/同一格 Up，并消费释放时 Shift/Ctrl。复用原 Button hover/focus 样式及原 ID/ticket/source/frame，不建 clock/store/theme。仅3条当前47路径有改动，另44条本轮前像字节保持；共享92B树几何导出、ZUI/97ID/6events及G09保持。见[实际diff/逐路径哈希和未执行边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-popup-hover-focus-r03/review.md)。新增2输入消费者定义，当前25域+5公共Button painter+2queued-open+2本轮＝34定义/0执行；静态guard0/0/0、fmt和diff通过。未新建或重投验证身份，#383/#389失败及native未知边界保留；原Penpot rev24及所有正式导出/EXE/DPI门仍open。下文r02等计数属于各自历史修订。

当前消费者修复 r02（2026-10-25）：已沿 Parent 移交的43路径审查实际接线，仅改原 app opener 与 native popup 合同2文件。旧 opener 在 Details 重排后可能仍有同一 inner Surface Arc/node ID；现在先校验原捕获地址的 active pane、源身份和完整矩形，再允许关闭现 popup 或 begin ticket，避免较晚的 show 拒绝已造成副作用。另41条当前路径保持，G09及原ZUI/97ID/6events未写入；原ticket提交、精确ID取消及解锁后sync保持。见[两文件真实diff/哈希与边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-popup-consumer-current-r02/review.md)。新增2项消费者回归，连同r04后继共32定义；静态guard 0/0/0、格式和diff检查通过，类型及测试未通过。

本修订已通过正常公共入口实际投递 `widget-ui-canvas-consumer-r02 / validate-editor-canvas-consumer-02`，queue16085→[Jenkins #389](http://127.0.0.1:18080/job/zircon-management/389/)，durable payload明确E根/显式test profile。#389现为 **FAILURE**；同身份只读结果为pending/unknown、`management_operation_interrupted`，原因是源码捕获期间HEAD/full-index/完整成员集合变化，Cargo未到、32项执行0。[实际对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-popup-consumer-current-r02/jenkins-build-389-observation-r04.json)保留早期management_git_busy等待与最终失败；不推断变化路径/写入者，不以Pipeline结束证明native Job/children/双EOF关闭。已沿原身份交Jenkins owner，不重投旧#383或这笔终结请求。整个窗口族仍open，passed/accepted=false。

当前设计后继（2026-10-25T08:56Z）：原file/page真实保存、关闭并fresh重开为**rev24/3497objects/原8local mains**。新增900×620“首次slot保留源定位”板 `d07f6d26-bfdd-8016-8008-d7d588c1fb7c`，仅62对象、两份原Preset native copy；3434原非根对象、12 Shared heads、全部主件、Shared rev5整data和registry保持。实际截图已逐图打开，三列及页脚没有裁切；全部数值明确为源码推导目标，未作为产品运行结果。见[保存重开回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-node-placement-r04/receipt.json)和[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-node-placement-r04/review.md)。原rev23修饰键板及所有旧失败继续保留，未做正式导出/EXE/DPI验收。

当前域内修复 r04（2026-10-25T08:51Z）：首次写入 Canvas slot placement 时，省略字段改用 slot 默认值，不能继续假定继承节点。已在原 Session/geometry 修复未变化 anchor 轴、pivot、position 的保留，整份 no-op 则不生成 slot；原 ticket、修饰键次序及一次 undo 不变。仅3条本域路径实际写入，公共 host/G09 没有新增修改。见[逐路径前后哈希及小补丁](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-node-placement-preservation-r04/summary.md)。原22项测试的26360B前缀完整保留，新增3项真实 Session/v2/preview/history 回归，当前25域+5 painter定义，执行0；rustfmt、精确guard 0/0/0和diff检查通过。#383仍是原27项capture失败，不重投、不扩大原identity；[本revision验证意向](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-node-placement-preservation-r04/validation-intent.json)未提交且request/operation=null。以下47路径匹配数和22/27项计数保留为原实施时点，不替代r04后继。类型、正常EXE、DPI和整族验收仍open。

当前接续（2026-10-25T08:24Z）：监督恢复后，原 Canvas 公共接线候选已按47个准确前像落入实际 checkout，见[逐路径实施记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-native-integration-r01/application.json)。真实 Details opener→注册窗口/active pane→原 document/session guard→一次原 ticket→16项预设/Shift/Ctrl→同一事务→按 popup 身份关闭和焦点返回的源码已接通；没有改原ZUI/97ID/6events、catalog/tokens/Starship/Runtime，也未合入旧G09的38路径提案。格式、精确diff guard（0错误/警告/抑制）和diff检查通过。随后回查46/47原输出一致，`contract/mod.rs`另有保留的树disclosure re-export，Canvas导出仍在；不覆盖并行改动。

原 Penpot file/page 已保存、关闭并fresh重开到**rev23/3435objects/原8local mains**。新增900×620修饰键板 `f7131a51-5260-8090-8008-d7ccf7174941`，复用4份原Preset native copy和实际Shared rev5样式；3313个原非根对象、12个Shared heads及Shared整份数据保持。图已打开复核，数值明确标为源码夹具预期，未声称运行结果。见[真实回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-modifiers-r03/receipt.json)及[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-modifiers-r03/review.md)。

既有未提交popup意向现已具体投递为 `widget-ui-canvas-popup-r01 / validate-editor-canvas-popup-01`，范围为单包zircon_editor/defaults off/features=[]/lib/test profile/filter `canvas_anchor`，22域+5真实button painter消费者定义。真实queue15787→Jenkins #383 **FAILURE**，原operation为unknown：源码捕获时HEAD/full-index/membership变化，Cargo未到、执行0。原payload只读查询与[原始529B回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-native-integration-r01/control-receipt.raw.json)已保留，不更换身份重投。原Layout/Canvas两请求保持。正式源导出、UE glyph/hover动画、正常EXE、实际150%DPI及双端1DIP验收仍open；整个窗口族passed/accepted=false。goal保持active，继续独立工作。

G09旧提案已另做[当前Canvas上的精确重基r02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/view-retained-canvas-rebase-r02.md)：38路径核对，其中11处Canvas重叠；原pane finder可见性前置已具备，实际拟写集收窄为37。Manager保留原Canvas方法，其他新增/删除行与旧提案一致；38份rustfmt stdin及git apply --check通过，公共源码写入0，8拟增回归执行0。它仍未应用，不称GUI搜索已修；原popup #383未知操作未扩大或重投。

以下rev22及此前段落保留为历史时点，不覆盖上述当前源码、设计或提交事实。

最新实际状态（2026-10-24T22:41Z）：原file/page已保存、关闭并fresh重开为**rev22/3314objects/原8local mains**。本次仅新增原Anchor Picker的一个“鼠标移出后保留键盘焦点”原生设计状态板；原2977个非根对象、12个Shared linked heads、原组件定义及Shared rev5完整数据保持。默认/hover64状态已存在，本次只补return48状态，没有新建主件。两张真实Penpot浏览器图已逐张打开；字体资源字节已匹配，但标题的精确字体检查仍false，故不能通过文字/产品验收。见[本轮回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/receipt.json)、[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/review.json)与[字体来源分析](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/font-source-analysis.json)。

本会话 `01a16d52-d54d-7ac0-b0d5-21864f383a1e`，父会话 `01a16d4e-0491-7353-84fa-038bd7706546`。用户明确分工为 Widget/UI；原 overview 的 Widget 与 AI 合并入口不意味着本会话接管 Behavior Tree / Blackboard。

历史进展（rev20及此前）：已记录 stock Toolkit/TabFactory 盘点和21项窗口/关联面板，动态extenders未核实。原七张完整边界目标覆盖三个尺寸、空数据、搜索空结果、焦点、16锚点弹层及中文长文本。[Shared受控消费板](widget-ui-shared-consumer.md)已连接4个公共主件、12份原生copy，覆盖/reset及保存关闭重开成立。revn11的六处Resize Slot flow、revn12的28个被动页签标题量测/省略保留。17:10Z原14个Palette/Hierarchy搜索已保存关闭重开为revn13/2978objects/原8local components：完整查询、字体及component/sourceRef保持，只调整28个既有root/text的单行flow与字段裁剪，其余2950对象及Shared全哈希保持。七图已逐张复核，640及中文长查询仍右侧裁切，真实caret横向滚动/IME/过滤/清空/切文档继续G09 open，见[revn13逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-search-shared-r01/review.md)。Shared SearchInput原240×32/free与本页20高字段的覆盖/导出合同未闭合，未替换旧引用或复制主件。真实tooltip/overflow/tab导航仍未验收。验证板不替代七板语义区域的正式接入，Dock/Tree/Search/属性行源导出、Graph、动画和引擎交互仍未完成。Shared当前独立读回为rev5，原34colors/3typographies保持，Button main新增1组有效surface.1引用；旧rev2/0绑定是历史。原12copy的真实Update保存rev18，本域随后在两个原Button验证已发布颜色/独立reset，关闭fresh重开为rev20/2978objects/8local mains/99实例sourceRefs；相对rev17仅4对象变化，板外与正文/字体/行框保持。1条等值token metadata命名空间移动、原SDK绘制缓存、旧状态ACK及rev14报文缺口均保留。原19/16已核实为保存位置框与文字层差，不能直接判裁切；r04两条中文ink高13、四级clip包含完整SVG BBox，Roboto400长文本与原engine-face自然宽度差3.734375，公共字体消费、全局A/B及七板正式消费仍open，见[当前共享消费](widget-ui-shared-consumer.md#文字度量与字体消费复核-r04)。canonical sourceHash/slot/export合同及官方PNG超时/null保留。原产品资产仍仅有上一轮正式导出的gap6实验；域内源码候选尚未托管编译/接GUI。正常EXE、实际DPI、双端逐图验收和整个窗口族继续open。

G09新增真实目录上的Session查询候选：保留完整query和Native/Component/Reference原身份，返回带源缓冲、目录、查询及页面分配守卫的不可变四项页快照，再以可见行解析真实目录索引调用原选择/插入事务。无结果清除Palette选择和拖放目标，查询不写源或history；目录重建和实际文档插入/undo/redo保留查询及分页钳制。12项新增回归尚未托管执行，公共输入/焦点、retained collection与Ready-empty shell尚未接通；原pane仍输出完整目录，不能将本候选当作GUI过滤已修。精确路径、基础文字查询与UE表达式差异见[源码候选及验证边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/summary.md)。原layout/canvas验证身份、filter和失败不变，不借旧请求验证扩展后的范围。

Parent第五次G09接续已形成[view/retained精确提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/view-retained-proposal-r01.md)及[未应用diff](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/view-retained-proposal-r01.diff)：原生SearchField完整输入→同一snapshot/可见页→真实borrowed hit/frame→受检Session选择，并检查实际注册窗口/active pane/source/focus；Ready-empty更新外壳且排除旧Palette行回流。38个拟改路径均未实际写入，8项拟增回归执行0；38文件rustfmt stdin与git apply --check通过，仅证明语法/hunk适配。原11域源11/11、产品97ID/6events和两原identity哈希复核保持。公共callback/DTO/focus需Parent合并审查；native hide/IME/popup/Tab、多pane焦点绘制、hover/capture及GUI仍open，未用该提案覆盖原Jenkins范围或宣称G09已修。

- [完整窗口盘点、适用性与开放差异](widget-ui-inventory.md)
- [控件映射、接入链和验证输入](widget-ui-validation.md)
- [原 layout 正式对账、九根捕获计时与有限窗口条件](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/widget-ui-penpot-r01-layout/summary.md)
- [具体区域设计、焦点命令与 Canvas 锚点契约](widget-ui-design.md)
- [实际 Penpot 身份、导出修改与未通过门槛](widget-ui-penpot-evidence.md)
- [Shared原12份copy/覆盖/reset/重开与rev2同步边界](widget-ui-shared-consumer.md)
- [Actor Graph 交接、预览事件来源断点与实际宿主消费合同](widget-ui-graph-consumer.md)
- [七张完整边界布局的真实图像、逐图差异与保存回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-widget-toolkit-r01/widget-toolkit-design-review.md)
- [G01 Starship 最小待审 diff 与现有导出边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/starship-class-adoption-r01.md)
- [Canvas 预设事务源码候选、限定范围及托管测试输入](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-domain-r01.md)
- [生产 visible 属性的域内 Host 候选、过期守卫与真实失败边界](widget-ui-preview-visibility.md)
- [已核对源输入指纹](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/source-inputs.json)
- [实际服务检查与失败记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/runtime-observations.json)
- [Parent 最新分工与实际文件回执入口](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)

## 源资产与所有权

- 接续 `zircon_editor/assets/ui/editor/ui_asset_editor.zui`，身份 `res://ui/editor/ui_asset_editor.zui`，v2，根节点 `ui_asset_editor_root`。它是现有 UI Asset Editor bootstrap 与 retained projection 的布局权威；不得另建同功能资源替代它。
- `zircon_editor/assets/ui/editor/ui/extension_ui_asset_editor_workspace.zui` 是另一个 workbench extension 入口，需核实其真实数据和命令，不能以其示例面板代替完整 UI 编辑器。
- 复用 `components/catalog.toml`、`theme/editor_tokens.zui`、`unreal/starship.zui` 和既有控件。公共 token、catalog、Dock/菜单及跨族共享组件保持父会话所有权，本会话只登记具体提案。
- UE 参考树只读。已检查 UMGEditor 根到源码路径的本地指导入口，根 `dev/UnrealEngine/AGENTS.md` 不存在；子目录指导仍按实际存在情况核对。
- 产物清单：[artifacts.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/artifacts.json)。未创建源码副本；当前释放 0 字节。

Parent [覆盖对账](parent-window-coverage.md) OVW-37/45 已收件：Font/FontFace 资产文档和作者入口归资源会话 `01a16d53-60db-7d02-b03e-0042f277783d`；Widget 只消费字体、atlas、fallback 和 readiness，不另建字体资产编辑器。Editor Utility Blueprint/Utility Widget 的脚本、图及宿主操作合同归 Actor 会话 `01a16d53-8965-7472-931c-73cba748fe8e`，本域负责真实 Utility UI 内容适配；Parent 接 editor host/VM/文档。完整 Utility 后端尚未确认，runtime Widget Graph 不充当 Editor Utility。Actor Timeline 子文档继续消费动画 owner 表面。此为归属对账，字体、共享组件/导出/Capture 的未交付状态保留，不扩大公共源码写权或桌面时段。

本轮接 Actor AB-DEF-01～08，已在原 Widget 域源码补入 [binding 本地源定位候选](widget-ui-binding-source-locator.md)：同一解析文本的 binding 声明与字段范围、真实 root/component 归属、current / last-valid 区分及旧修订拒绝；沿原 selection / source cursor 接入 crate 内 API。12 项回归仅已编写，尚未编译、执行或接 GUI；实际 SourceLocator provider、目标 URI/文档/历史/焦点、外部 Definition 与 Graph 仍由 Parent/Actor 接续。原两份验证 identity 保持，captureReady=false；未启动服务、桌面或新请求，整族继续 open。

## 已确认的参考与能力边界

`Engine/Source/Editor/UMGEditor/Private/BlueprintModes/WidgetDesignerApplicationMode.cpp` 注册 Designer、Details、Compiler Results、Animations，以及由设置启用的 Palette、Library、Hierarchy、Bind Widgets、Navigation Simulation。默认左栏占 0.15，Palette/Hierarchy 各 0.5；右侧 0.85 内上部 Designer/Details 占 0.7，底部 Animations/Compiler Results 占 0.3 且默认关闭。Tool Palette 和独立 `SequencerGraphEditor` 默认关闭。

`WidgetGraphApplicationMode.cpp` 使用 My Blueprint/Details 左栏 0.15、Document/Compiler Results/Find Results 中栏 0.70、Palette 右栏 0.15。激活 Graph 时会选择 Designer 中所选 widget 对应的变量。不能把 Zircon 的 Source 文本模式改名为 Graph 来声称实现了此行为。

`WidgetPreviewApplicationMode.cpp` 注册 `WidgetPreview` 与 `DebugDetails`；`WidgetBlueprintApplicationModes.cpp` 的 `UMG.EnablePreviewMode` 初值为 false，因此该模式属于显式启用的关联能力。

现有 Zircon `UiAssetEditorSession` 保有 v2 文档、source buffer、树选择、undo stack、slot/layout/binding 编辑和 runtime preview host；`preview/preview_host.rs` 使用现有 `UiV2SurfaceBuilder`，并不需要另一套 UI 运行时。`UiAssetEditorMode` 实际为 Design/Split/Source/Preview，未声明 Widget Blueprint Graph 模式。接口存在仅为源码能力，不是本轮 GUI 验收。

Canvas 能力已进一步核实：运行时 `UiCanvasSlotPlacement` 和 v2 builder 已支持 min/max anchors、offset、pivot、position、auto-size；Canvas已补既有语义字段列表与kind投影；专用typed字段、默认值/继承和Free分支仍未实现。剩余包括UE专用字段布局、16预设/重基的GUI接线、拖动与原子撤销实测，详见具体设计；不再将整个 Canvas 运行模型视作缺失。

本轮在自有 `ui/asset_editor/session` 新增 `apply_selected_canvas_anchor_preset` 候选：同步读取当前原生 Canvas 的唯一源映射几何，扣除父 padding，以旧 pivot 重基，再执行 Shift/Ctrl，一次调用已有 source/undo 提交。限定 v2 单选、正尺寸、auto-size 关闭；Free、组件实例和其它未支持几何返回错误且不改源。八项回归覆盖实际 v2 preview/roundtrip/undo 链，格式和夹具解析仅为静态检查，未执行 Rust 或 GUI 验收。未改公共 host、组件、主题及新的产品资产。

接续 Parent WG-C01～08，新增 [visible 属性受控候选](widget-ui-preview-visibility.md)：只在当前有效、无动态 writer 的原生 Panel/Label View 中使用生产 `mutate_property`，核对实际 source/context、Host gesture lease、frame、sequence、节点 incarnation 和 binding/invalidation 回执；Cancel 保留原 override presence，Reset 从当前有效源重建。原 v2 builder 同步传入真实 owning route URI，未补造 imported source path。13 个域内 Rust 文件格式检查通过，11 项消费者回归仅已编写，未编译/运行或接 GUI；没有修改共享 Runtime/动画 schema，也未关闭动画、外部事件或 EXE 证据门槛。

接原owner15:13Z只读对账后，已[刷新visible候选r02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/preview-visibility-domain-r01/revision-r02/summary.md)：两个索引的变化精确对应本会话后续binding定位登记，保留所有注册及原03616d65…patch；r02的9cdcbcfa…patch仅更新两处hunk行号，当前13源码/22依赖/14binding上下文/4保护资产闭包匹配，Rust0写入。原11项测试仍0执行，新增operation仅为`validate-editor-preview-visibility-01`/r02的离线意图，无request或提交；原Layout/Canvas身份、filter、ownedPaths及失败保持。E空间原观测低于35GiB保留线，资源/pool/cache-writer、真实intake及capture-ready均未取得，本轮无查询/捕获/编译/静写/服务动作，不要求用户手填grant。

Parent 已澄清既有本域源码/最小 Jenkins 验证授权持续有效，无需再向用户申请。2026-10-24T14:13Z 通过正式入口实核服务 public capability 为 ready；两份原 request 的 QueryOnly 均 exit2、`Current session authorization is pending`。这是管理工具在查询原操作前的会话状态检查，不是撤销用户授权，也不能据此判断原服务端是否收件。保留两份 identity/hash/filter，沿原 Jenkins owner 对账 automatic intake、历史 capture 和实际缓存锁准入；本轮未重投或启动 Cargo。精确回执见 [验证接续 r02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/managed-validation-resume-r02/supported-route-review.json)。

新收原 layout 对账 JSON 的 SHA256 `d4bb8116…139b8` 已完整核验。原 owner 的 State/受保留构建参数检查未发现本请求匹配，删除及间接历史仍未知，serverReceipt=null；不解释为“已有 job 等待人工授权”。首轮九根 timed pin 为195.930秒，完整双轮/总时长未知，8分钟只是低置信 capture-only 排程预算。14:22Z 当前 installed 已为20b3bf85…，区别于 operator 观察时点的cdd14500…；仅因该变化对原 layout 做了一次正式QueryOnly，仍exit2/pending，身份未变。未 capture、重放、编译或发静写指令；[完整时点与边界](widget-ui-validation.md#原-layout-对账与捕获预算) 保留。

接Parent恢复r2后，15:38Z实际installed已为 `2790fb5199bd06ca7291ac197f03545eaeae05c56b125117fad268ce19d4b0c6`，当前胶囊5个server/6个client哈希全部匹配，driver/runtime保持原值。两份Widget原identity各执行一次正式QueryOnly，均exit2/pending/`ordinary_entry_unproven`；没有源码捕获、Cargo或server准入回执。原identity字节保持，实际输出已通过官方队列交原Jenkins owner和Parent（仅投递确认），详见[恢复后原身份对账r03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/managed-validation-recovery-r03/summary.md)。服务恢复不增加产品passed/accepted。

G01 按 Parent 用途映射已形成原资产的最小增量审查补丁，仅增加既有 Starship import、shell/Designer canvas/toolbar classes，并收窄两条遮住默认的局部 rule；未应用。只读调用现有 `isSupportedVisualPath` 确认 source-owner visual-delta envelope 拒绝 classes/imports/stylesheets，公共 owner 需交付明确的结构迁移合同后才能形成正式导出。未把布局三列当 Dock、普通 Details 分组当属性行，也未复制共享样式值。

## 桌面和服务协调

本聊天未提供 `read_thread` / `send_message_to_thread` / 桌面 MCP 工具。父聊天链接已保留；未读取或编辑聊天数据库，也未接管其它聊天进程。

Parent 已明确独占协调 Jenkins/Penpot 生命周期、公共组件/主题、file/page 分配和桌面截图。本会话不独立启动或恢复服务、不占用桌面输入、不重复询问服务/页面信息。真实 file `b4a1e3ce-9e6a-81fe-8008-d69d60d24223`、page `b4a1e3ce-9e6a-81fe-8008-d69d60d24224`：[05 Widget and UI](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60d24223&page-id=b4a1e3ce-9e6a-81fe-8008-d69d60d24224)。本聊天已独立认证读回 revn 7、1434 objects、8 个本地 native components，完成下述局部设计的保存重开；官方 ZUI 导出只覆盖前一轮源样本的 gap delta，不包含新锚点业务组件。

本聊天于 `2026-10-24T10:18:29Z` 对前端 `19001/readyz` 与后端 `16060/readyz` 独立只读检查，均为 HTTP 200。原连接拒绝与正式 Jenkins start 失败继续保留为历史观测；之后公开 Jenkins capability ready 也不代表本请求或原生生命周期通过。r01时公共库rev1/shared四主件已由本域有限消费；Parent新观测已为rev2/34colors+3typographies，本域跨文件样式同步尚未执行；本域真实 component/instance/board UUID 见 [实际证据](widget-ui-penpot-evidence.md)。桌面时段仍未授予。

共享时间轴/曲线/播放控件 owner 为动画聊天 `01a16d51-741b-73b3-a61a-2189cd38f81e`，共享节点图表面由材质聊天 `01a16d50-6c7f-7370-853d-4581184b66a1` 牵头。Parent 负责跨域合同、公共主题/框架及集成；Widget/UI 负责事件、widget 属性轨道及 Designer 业务语义，重叠路径先交接。具体需求见 [共享 owner 合同](widget-ui-design.md#共享-owner-的具体消费合同)。

已通过 `codex queue --thread 01a16d4e-0491-7353-84fa-038bd7706546 --message` 交接所有权、现有计划、共享提案和真实能力缺口；CLI 返回 queued message `01a16d5d-3c46-7831-b927-bd14b91706ae`（exit 0）。这是投递确认，不是 Parent 完成依赖或服务恢复的回执。

Jenkins 初次实测：18080 连接被拒绝；正式 `tools.jenkins.tray --formal ... status` 返回 `state=degraded`、`ownerKnown=false`、`ownerLost=true`、`canStart=true`。历史 driver 为 `fc55322804fb53ab3e39f996c848ef5e5f3907497d3f38167b7095244b2a7399`，runtime generation 为 `start-1792418123320632400`；历史 `health=true` 不能充当当前 readiness。

历史正式 start 实际失败（exit 1）：`Original runtime evidence changed`，调用链在 `_submission_idle` → `interrupted_request` → `_lineage_context` 校验历史 `release.deploymentSpec` 时拒绝；同阶段 Penpot readyz 连接被拒绝。后续 Parent 的真实启动及上述 HTTP 200 是新观测，不覆盖旧失败。本聊天未改历史证据或恢复服务；原托管 layout 请求又在 pre-submit 双次源捕获因 digest 漂移被拒，尚未启动 Cargo，query 为 authorization pending。沿原身份与 Parent/既有实现 owner 对账，不重投或私开替代构建。

真实产品入口已进一步定位为 `ui/host/asset_editor_sessions/open.rs`：`view.editor.ui_asset.open` 打开实际 source，创建 session，hydrate imports，sync instance，register document toolkit 并 focus view。下一阶段复用该入口，不使用 extension 的示例工作区替代。

## 验收范围

Graph接续已核对材质UI-SH-GRAPH-01原合同和10个未应用公共逐文件提案，6个canonical源保持原hash；本轮编辑前30条来源29同，唯一差异是本域设计文档的历史hash，当前值与上一轮Palette候选一致。随后本消费者文档自身更新、AI两处并行重导出变化及catalog更新分别留痕；10提案的初读匹配不代替当前状态。已在[原Widget消费者记录](widget-ui-graph-consumer.md#接材质-ui-sh-graph-01-的源码与输入边界)明确checked Current binding定位已实现为源码候选、宿主/Details/Source/focus仍缺，并定位通用UiSurfaceFrame绘制/命中/输入的原路径。真实Graph geometry producer/业务文档和native owner尚未交付；不能用通用模板hit frame、Source或Material/AI helper关闭Graph缺口。额外追到正常winit pointer入口保留PhysicalPosition数值、normalize不换算坐标，原Preview fit scale也不等同DPI；由Parent在原native owner核定一次坐标转换，本域未改公共输入。此次仅两份本域文档和只读核查，无Cargo/插件/Penpot/桌面/EXE操作，W11/W12/W20及整族仍open。

1280×800、900×620、640×520、1280×800 实际 150% DPI，分别覆盖默认、空数据、选择、焦点、弹层及返回、首尾滚动、中文长文本。另覆盖 Palette 拖放/插入、Hierarchy 重排/改父、Designer 选择/缩放/锚点/对齐、Details 提交与撤销、事件跳转、动画轨道/关键帧/播放、运行时预览输入、保存重开及运行产品消费。

所有 case 复用 LayoutReviewCase 和普通原生证据协议；资源未就绪或捕获无效不记 passed。真实双端 PNG、匹配状态、逐图复核和不超过 1 DIP 的关键几何/文本行框目标全部满足后，才对指定范围记 accepted。

Canvas弹层域适配补入[打开时的目标凭据](widget-ui-design.md#canvas-预设弹层目标凭据)：复用原source snapshot、selected edge和当前preview frame，begin/apply/cancel不将过期弹层重定向到新选择；新弹层、selection/mode往返、源/preview变化或一次消费后拒绝旧ticket，仍使用原一次undo事务。八项新Session回归0执行，原11项测试前缀及两原Jenkins身份保持；新验证operation仅离线提案，未提交。公共popup/active-pane/focus/hide由Parent接续，本轮未改公共源码、Penpot或ZUI，G04及整族继续open。
