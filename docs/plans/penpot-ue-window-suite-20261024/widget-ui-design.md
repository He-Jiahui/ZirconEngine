# Widget/UI 具体设计与编辑契约

## 2026-10-25：Palette 同 ID 非文本替换的实际输入退休修复

已在原37路径内仅改 `window/palette_search_input.rs`，23e7156b…/17931B→973dacf6…/20618B，LF保持。原保留焦点守卫有ID/routes/source/frame却缺实际组件族；现复用canonical `is_component_family(TextInput)`，让原state发布链先退休同ID Button/Dropdown/KeySelector/Label的旧query目标，不造新focus/clock/store或发布结构。原9测试和10保护输入保持，原fixture从真实v2产品projection携带role/component_role；新增一项四替换/合法刷新caret/晚文本拒绝/不复活回归，10定义均0执行。[实际补丁与边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-native-field-family-r03/review.md)及[当前source后继](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-native-field-family-r03/source-review.json)已固定。

初fmt的import/断行失败保留，最终fmt/reverse/diff0、精确guard0/0/0仅静态。新[最小意向](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-native-field-family-r03/validation-intent.json)仅原消费者module10项，单Editor/defaults-off/features[]/lib/test/locked/jobs1/threads1/物理E，request/operation=null，没有编译/submit/query或重放#412/#415。当前已审阅三scope为30/6/38具名定义，重叠2/去重72，不代替实际Rust发现/执行。旧source-review-r02与两原terminal输入原样保留，不更新旧资格。

本轮产品ZUI/97ID/6events、Parent两raw-whitespace源及Penpot rev51设计未写；尚无正常EXE/IME/实际150%DPI/双端1DIP通过，整族passed/accepted=false。原Resize preferred-size API未发现真实handle生产caller，不拿它关闭Canvas offset拖动缺口。帮助文件/必要失败保留，释放0B。


## 2026-10-25：原 Widget rev49→51 的可见 Details 修复与 Palette 可视边界

原file/page保持；r07五个640 Palette原copy以native AutoWidth/row/nowrap和原clip修复数据行框，rev48→49保存关闭fresh重开。10对象后继/3535非目标及完整非目标file data保持，原component/main/shapeRef、根几何、内容/字体/paint保持。四图已逐张打开，before/fresh原PNG完全相同：五目标在原Palette可视clip外，不宣称可见修复或真实滚动通过。详[实际限制与四图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-palette-nowrap-r07/review.md)。

r08只改900中文和640英文两条Details空态的原native flow/auto-height，保留换行、原字号与根几何；两次HTTP200保存到rev51，关闭fresh重开和独立API同data SHA `f9f843e2abc0edeb601b2be898c678a485eb773487664a54ca04a65a31ef23d7`。行框原底部约3 DIP越界变0；八张原图/并排/原RGB差异已逐张打开，可见变化仅说明文字。4对象后继/3541非目标（含r07十对象）、8原main/17Shared heads/Sharedrev5及registry保持。详[逐对象身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-details-flow-r08/editable-design-delta.json)、[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-details-flow-r08/visual-review.json)及[本轮差异与未完成项](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-details-flow-r08/review.md)。真实font buffers不证明引擎provider/fallback；640 tabs/Search、900中文query、scroll/caret/More/focus/IME和完整UE chrome继续open。

#415新实际终态为API FAILURE/building=false、结束16:45:59.581Z，State44/547B原archive `759ed2c63112a5e816d62f55924fbb9067d3bc4329ff63001dfea4c2633dc70e` 与原State结果一致，报capture期间HEAD/full index/complete membership变化。63原声明仍63/63；不等于最终compiler source、Rust结果或native/resource/双EOF释放。原a2164349身份/输入/失败保持，未重投；[原终态证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-pointer-managed-r01/handoff-terminal-r03.json)已queued给原执行owner，queued不当已读。

本轮产品Rust/ZUI未改，原v2/97ID/6events与97992ab7资产保持；没有新业务ID/POST/query/Cargo/capture/freeze/服务/桌面或系统DPI操作。正式导出、正常EXE/实际150%DPI/双端逐图及1DIP仍pending，整族passed/accepted=false。自有browser/API contexts已关闭，临时帮助文件与失败证据保留，实际释放0B。


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

201个目标只选择原宽度内可单行容纳的既有本地实例；选择用保守ASCII宽度门，最终由真实SDK布局和保存后几何验证。不是字体度量或引擎font-provider证明。所有目标原记录3/8逻辑像素bottom excess改为0；只是本地行框包含检查，不等于UE1DIP。20个长/非ASCII或窄文本目标保留，额外Search横向溢出也未关闭。

单次201项发送曾在45秒等SDK回执超时，0保存、rev30 data未变；原失败保留在 `penpot-toolkit-caption-flow-r02`。只读reopen/audit r03成功；单例r04的真实setter/layout/save闭环成功。随后r05按<=20实例分14批，每批真实update-file ACK及累计全data delta读回后才继续。大批超时根因仍未证明，不把调整批次称SDK修复。


Hierarchy 当前四路径修订已接真实 document.root 的 Wrap/单子项 Unwrap：完整原文档上先预留 node/control ID，再更新原根及原 v2 serializer/history/selection；原根和后代、绑定与组件引用保留。新增3项回归，原1项 root 可用性测试两条期望同步调整，原测试11326B前缀保持。四路径格式/guard/补丁反查通过，Rust执行仍0；原中断 Wrap identity 不扩写/重投。详见[实际实现与消费者复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/hierarchy-root-wrap-r02/review.md)。仅 document root 源码分支有后继；组件根/多选/named-slot、native enabled投影及正常EXE/几何/焦点/150%DPI/双端仍open。此前“root gap”记录保留为旧时点。

原 Penpot file/page 已由 rev29 真实保存、关闭并 fresh 重开到 rev30：七块 Toolkit 板中40个旧实例改用 native row/nowrap/center 与 Text fill/auto/auto-height，原23/28高固定框及3/8向下越界收敛为16高原生行框。原字体/文字/样式、实例尺寸、source/shapeRef保持；3545对象、8主件、17 Shared heads不增加，3465非目标对象及Shared rev5完整data不变。六张1280/640/900中文 before/fresh图已逐张打开。640窄页签/搜索、中文query裁切及UE chrome仍open；原ZUI未改，正式导出/EXE/DPI/双端/1DIP及passed/accepted仍false。见[实际复核记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-hierarchy-designer-flow-r01/review.md)。 本次没有重建公共 Button/Tree/PropertyRow，也没有删除原8个本地主件；使用原实例增加局部flow覆盖，原metadata保留，正式codec必须在原owner链核对。

Palette clear生产链新增精确发现：原v2已声明has_clear_action，但ViewTemplateNodeData缺字段、retained转换固定false，且原生pane只有Press没有clear配对Release。已形成[19路径完整未应用consumer候选r02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/palette-native-clear-wiring-r01/review.md)，含typed source metadata→DTO/materializer/converter、原pointer/source/frame的Down/Up、原query/焦点链及5拟增回归；14额外公共路径只交Parent。32307B/SHA aaa219dd3178c74a66f8e7602e331d3b21544eaf9ac05932e66615e7eea07efd，19stdin fmt/apply-check0，未覆入源做guard/类型或执行。实际checkout仍为G09/actual-field r02的27项定义；候选32项不扩旧身份。IME/popup/ACK、公共left geometry与EXE/DPI/双端仍open，原Penpot rev29保持。

当前字段行为 r02 已落入原 G09 两路径：实际 `PaneData::template_nodes()` 决定 query 控件是否仍可接收旧焦点输入；同源字段移除、禁用、路由改变或无效几何拒绝，重新发布旧布局不复活原目标。沿原结构/焦点原语处理，未建新输入或时钟系统。Penpot rev29 原板与三态身份保持，本轮未重放保存。 [当前字段资格r02静态回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/palette-current-field-r02/review.md)固定73保护输入/6旧identity-control保持、guard 0/0/0及87源依赖指纹。格式/精确补丁检查通过，类型与27项测试未执行；实际EXE/150%DPI/官方导出/双端1DIP及整族passed/accepted仍false。此前25项和各旧revision只代表原时点。

最新可编辑设计为原file/page的rev29。Palette三状态板`5b0b50ff-0901-801b-8008-d7ecdd7e3452`保留原查询全文，clear后query为空且保焦点，hide保query但无焦点；这些是明确标注的目标状态，不是已运行的引擎结果。原3SearchInput/2Button linked copies及published body/caption/colors保持，流布局修正遮挡、单框组合和Starship默认左对齐，未改Shared/main或另建主题。[完整copy/component/shapeRef/覆盖与五图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-palette-clear-focus-r01/review.md)。原3496非根对象、8main、12heads及板外完整data保持；新增48objects、5copies，无detach。公开主件默认/reset、32px验证规格到真实20px Palette规格、×正式glyph、清空命中/提交次序/焦点/IME和正式导出仍open；以下rev24表述保留为各自源码切片的历史设计输入。

当前原生Palette文本目标退休 r01沿原设计和组件消费，只在既有hide/exit/blur/确认关闭链清理旧字段，不新画界面或调整字号。原file/page/rev24、完整查询/Shared引用、Canvas ticket/history保持；[4条真实源码及5项未执行回归](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/palette-native-input-retirement-r01/review.md)。context menu、IME composition/Tab/焦点返回与旧row/page重显有效期仍分别open，不能把本轮源码清理等同UE全交互或已呈现。

G09原签认r02现已在37原路径落源，搜索完整值、同一snapshot/当前页、borrowed hit/frame和原Document/Session受检选择连到真实消费者；Ready-empty外壳不再合回旧Palette行。只按签认稿消费既有UiSurface、history和host，没有设计替换或新runtime。见[当前实施/行尾/保护输入与未运行边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/view-retained-integration-r03/review.md)。原rev24及14搜索全文/组件身份保持，本轮未改Penpot或ZUI；8新增定义执行0，真实GUI/字体/源导出/EXE/DPI继续open。以下Canvas34计数仍属独立范围，不代验G09。

当前 hover/focus r03 已在实际3文件中消费既有Button hover/focus：鼠标移动和窗口leave不再改写键盘Enter选择，点击保持原press/release语义。设计沿用原键盘/hover板 `180e9578-8294-803b-8008-d6ae833142ad`，本轮没有新Penpot保存/截图或字号/主题修改，最新实际全文件仍rev24。见[源链与原设计映射](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-popup-hover-focus-r03/source-trace.json)。UE glyph/48↔64/0.40s动画、实际焦点模态/IME/present仍未还原；静态guard通过不当作设计或GUI通过，当前34定义执行0。

当前产品消费者 r02 已落实两文件副作用次序修复：queued native opener须先按原完整地址验证，再关闭旧popup或begin原ticket。旧地址拒绝不移动当前popup/ticket/焦点；同ticket提交、取消只退原permit及解锁后sync保持。此轮没有修改Penpot或字号/主题，原rev24设计仍有效；见[真实源码及两项回归边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-popup-consumer-current-r02/review.md)。实际#389在源码捕获失败，32定义执行0，不把设计图或静态守卫当行为验收。

## 首次 Canvas slot 的域内修复 r04

原Penpot页面已追加此行为的可编辑900×620设计板 `d07f6d26-bfdd-8016-8008-d7d588c1fb7c`，真实save→close→fresh reopen为rev24/3497objects/8mains。两份原Preset实例与Shared body/caption/颜色，三列依次表达无slot、同ticket提交Centre、重开/undo；未变化轴/pivot/position与取消/无变化保持缺省都明确列出。62新增对象之外完整data保持，逐图已打开，见[设计复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-node-placement-r04/review.json)。数值为源码推导的设计预期；字体精确provider、真实输入、正式导出及EXE仍未通过。

没有显式 placement 时，runtime 使用节点定位；首次写入 slot 后，未写字段来自 canonical slot Default。原算法已正确算出重基值，但只写与旧有效值不同的字段，可能漏掉未变化的轴、pivot或position。现记录实际 placement 是否存在，首次 materialize 以 slot 默认值作比较基准；完整语义未变时保持原字段缺省。只改原 Session/geometry/域测试三文件，原 anchors/rebase→Shift pivot→Ctrl reset、一次 opener ticket 与一次 document undo 不变。见[实际小补丁与前后哈希](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-node-placement-preservation-r04/candidate.json)及[源码算式反例](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-node-placement-preservation-r04/source-counterexample.json)。这是源码级修复，三项新增回归尚未编译/执行；原Penpot rev23显式slot夹具仍有效，不能证明本次无slot路径已在GUI验证。以下47路径原回执和计数按当时状态保留，Parent接公共接线时应保留r04三文件后继。

## 当前 Canvas 接线与修饰键设计，2026-10-25

原47路径[公共接线候选](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-public-wiring-tests-r01/proposal.diff)已依监督恢复授权实际应用；[application.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-native-integration-r01/application.json)保留逐文件preimage、保原EOL的after、责任和原部分写入失败。它接入原Details flow、原生hit/keyboard/IME入口、既有Host/Manager与真实button painter；不新建runtime/history。打开只发popup绘制，不重建preview，ticket持原source/selection/inner frame。回调校验实际窗口、pane、popup和同一ticket后才操作focus/dirty/close；提交不begin，缺session不ensure。ESC、外点、hide、源替换、关闭/resize/DPI只退对应旧permit，普通hide不当Stop。

正式图形外观仍有差异：当前生产popup是16个真实secondaryButton文字项，UE 48↔64 preview glyph、Fill分隔、当前预设标识及0.40秒hover尚未接入，opener键盘再次激活未实测。按钮输入、焦点/IME/capture和整窗呈现仍须正常EXE验证；源码接线不算GUI通过。旧G09的38路径提案仍未自动合入。后续`contract/mod.rs`新增树disclosure导出已保留，不能重套原文件。

原file/page新板 `f7131a51-5260-8090-8008-d7ccf7174941`（900×620）已原生保存关闭fresh重开为rev23。4份copy均引用原Preset component `180e9578-8294-803b-8008-d6ae776be849` / main `180e9578-8294-803b-8008-d6ae75d79d3b`；源ref80、Shared颜色/排版引用字段474，非474个新样式。原3313非根对象、8mains、12Shared heads保持；没有新master或theme。

该板明确使用600×400父内容、旧anchor=.25/.25、alignment=.2/.6、position=13/-7、offset=25/30/80/40的源码夹具，选择center preset。默认/Shift/Ctrl/Shift+Ctrl四列的期望child rect依次为172/99/80/40、148/103/80/40、284/176/80/40、260/180/80/40；展示顺序严格为anchors/rebase→Shift alignment→Ctrl position/offset reset，沿同一ticket与一次Changed Anchors undo。这些数值未执行验证，见[逐图及数值边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-modifiers-r03/review.md)。

以下rev22及此前内容为历史设计记录。

最新实际状态（2026-10-24T22:41Z）：原file/page已保存、关闭并fresh重开为**rev22/3314objects/原8local mains**。本次仅新增原Anchor Picker的一个“鼠标移出后保留键盘焦点”原生设计状态板；原2977个非根对象、12个Shared linked heads、原组件定义及Shared rev5完整数据保持。默认/hover64状态已存在，本次只补return48状态，没有新建主件。两张真实Penpot浏览器图已逐张打开；字体资源字节已匹配，但标题的精确字体检查仍false，故不能通过文字/产品验收。见[本轮回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/receipt.json)、[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/review.json)与[字体来源分析](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/font-source-analysis.json)。

以下保留截至rev20的布局与Shared消费历史记录。实际file `b4a1e3ce-9e6a-81fe-8008-d69d60d24223`、page `b4a1e3ce-9e6a-81fe-8008-d69d60d24224` 已保存局部样本、锚点组件及七张完整边界目标；七板原revn9、[共享消费验证板](widget-ui-shared-consumer.md)原revn10、Resize Slot修订revn11及页签量测/省略revn12保留为历史。14个搜索的完整查询/原字体未变，单行flow与字段裁剪已保存、关闭重开到revn13；文件仍2978objects/8本地主件，4个Shared主件的12份linked copy、七板拓扑及旧source映射均保持。七板尚无正式导出或运行后端，不能代替共享Dock/Tree/属性组件、真实预览、Graph或动画。实际身份、七图及差异见[证据记录](widget-ui-penpot-evidence.md)和[revn13复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-search-shared-r01/review.md)。Shared原34颜色/3排版保持，当前独立读回rev5的Button main有1组surface.1颜色引用；旧rev2/0绑定及rev17文字重算保留为历史。原12linked copy实际Update后，本域两份原Button完成颜色覆盖/独立reset并保存关闭fresh重开为rev20；七板、8local mains与99实例sourceRefs保持。1条等值token metadata命名空间移动已逐字段记录，原19/16是保存位置框/文字层差，r04实际ink高13且四级clip包含BBox；Roboto400长文本自然宽度差3.734375，公共字体消费、原ACK/报文缺口、公共自动重算、完整槽/导出仍open，见[当前共享消费边界](widget-ui-shared-consumer.md#文字度量与字体消费复核-r04)。本域窄样式消费已有真实结果，产品运行和正常EXE验收仍pending；窗口范围见[盘点](widget-ui-inventory.md)，原生矩阵见[验证输入](widget-ui-validation.md)。服务、公共组件、主题和桌面由Parent统筹。

## 原资产身份和区域接续

唯一产品布局入口仍为 `zircon_editor/assets/ui/editor/ui_asset_editor.zui`，v2、`res://ui/editor/ui_asset_editor.zui`、根 `ui_asset_editor_root`。当前 97 个节点和 97 个 control IDs 是导出前的身份基线；允许在真实设计后增加有来源的节点，不能通过删除或重命名原节点绕过投影合同。原有引用、绑定、事件、未知字段继续保留。

| 原节点 / control ID | 设计中的职责 | 接入限制 |
| --- | --- | --- |
| `header_panel` / `HeaderPanel` | 资产身份、dirty/诊断摘要和保存/撤销/重做 | 沿用实际 session 状态；模式栏与全局菜单接 Parent 的文档宿主。 |
| `left_column` / `LeftColumn` | Palette/Library 上栈、Hierarchy/Bind Widgets 下栈 | 需共享 Dock 的独立 panel slots；当前 `LeftScrollRegion` 同时滚动两面板，差异 G03 保留。 |
| `palette_panel` / `PalettePanel` | 类别、搜索、真实控件/外部组件集合 | `PaletteComponentGrid` 的四个原型节点目前被 projector 按四项分页重写，不能只在 Penpot 中增加列表后假定宿主会消费。 |
| `hierarchy_panel` / `HierarchyPanel` | 同一文档的树、选择和结构编辑 | 行绑定真实 node ID、parent/mount；显示索引只用于投影寻址，不能成为持久身份。 |
| `designer_panel` / `DesignerPanel` | 工具栏、真实画布、选择/拖放/锚点叠层 | `DesignerCanvasPanel` 是创作画布，可用 CanvasBox；工具栏、面板和弹层内容使用 flow。 |
| `action_bar_panel` / `ActionBarPanel` | 现有可执行文档/树命令 | 复用 `WorkbenchUiAssetActionBar` 的实际 availability；后续按 UE toolbar/menu 分组迁移仍保留命令身份。 |
| `source_panel` / `SourcePanel` | Split/Source 模式和实际诊断/预览测试输入 | 保留 source outline/text、mock workspace 等节点；Source 不能改名为 Graph，mock 不能当作生产数据。 |
| `right_column` / `RightColumn`、`inspector_panel` / `InspectorPanel` | Details 的 Widget、Slot、Layout、Events/Bindings 分组 | 仍消费 session inspector；样式作者面板可成为同一右栈的另一个 tab，须保留 `StylesheetPanel` 和实际入口。 |
| `designer_diagnostic_overlay_panel`、`emergency_shell_panel`、`render_stack_panel` | 已有诊断和失败恢复信息 | 不删除节点掩盖错误。默认可见性只能由真实状态与已核实的宿主规则决定。 |

六条原事件必须逐条保留：`UiAssetEditor/HeaderSave` → `save`，`HeaderUndo` → `undo`，`HeaderRedo` → `redo`，`DesignerSelect` → `designer.tool.select`，`DesignerResizeSlot` → `designer.tool.resize_slot`，`DesignerPreviewInteract` → `designer.tool.preview_interact`。后三条事件 ID 的完整前缀同为 `UiAssetEditor/`。

## Dock、flow 和尺寸

UE 的 `WidgetDesignerApplicationMode.cpp` 是拓扑和默认状态依据。主区左栈 coefficient 为 0.15，其 Palette/Library 与 Hierarchy/Bind Widgets 两组各 0.5；剩余 0.85 的上区 0.7 依次含 Tool Palette 0.05、Designer 0.80、Details 0.35，下区 0.3 含默认关闭的 Animations/Compiler Results。Tool Palette 默认关闭，Designer 隐藏 tab well，独立 `SequencerGraphEditor` 也默认关闭。coefficient 由对应 splitter 在实际 opened children 中解析，不能将 0.80/0.35 当作屏幕百分比或预留关闭面板的空白。

Designer 的所有非画布区域使用共享 density、字体和可测文本；沿用 `editor_tokens.zui` 的 `editor.density.*`、控件高度与字段组件，不新增 Widget 专用颜色或像素补丁。Starship 的角色类映射由 Parent 确认后应用到原资产。每个面板只有自己的内容滚动 owner，标题、搜索和 tab 不随内容离开；画布 pan/zoom 不消费 Details 的滚轮。

字体资产作者入口归 Parent OVW-37 指定的资源 owner；本域只消费实际字体、atlas、fallback 和 readiness，并保留中文/长文本验证，不扩成 Font/FontFace 编辑器。OVW-45 的 Utility UI 内容适配与 runtime Widget Graph 分开：Actor owner 提供 Editor Utility 脚本/图/宿主操作合同，Parent 对接 editor host/VM/文档；完整 Utility 后端尚未确认，不能据现有 runtime 预览或图表面宣称 Utility 执行。Actor Timeline 子文档继续复用动画 owner 的表面。此处只接续[覆盖归属](parent-window-coverage.md)，公共组件、字体、导出及 Capture 的交付缺口保持 open。

| profile | 设计约束和检查 |
| --- | --- |
| 1280×800，96 DPI | 按 UE 默认 opened stacks 形成首个真实设计；记录 dock 内容矩形、标题/搜索固定区和 Designer 剩余区域。 |
| 900×620，96 DPI | 保留 pane resize/切换与完整操作；工具栏依据内容最小宽度使用共享 overflow，Details 字段标签和值按组件约束排列，分别检查滚动首尾。 |
| 640×520，96 DPI | 先用相同 Dock 合同验证最小宽度之和。若不能容纳，向 Parent 提交“侧栏进入同一可切换 tab stack，Designer 保留独立内容区”的具体紧凑布局提案，记录与 UE 默认拓扑的差异；未经确认不静默自动折叠或隐藏操作。 |
| 1280×800，144 DPI | 仍为同一逻辑布局，实际物理 client 1920×1200；文字、边界和输入坐标只转换一次。不能用缩放 PNG 代替新 HWND 的实际 DPI。 |

窄尺寸仍须通过菜单/tab 找到 Palette、Hierarchy、Details、事件、Source 和真实动画入口；关闭浮动面板后 focus 返回原文档。Dock/popup/overflow 的行为由共享宿主提供，本域不另写平行窗口系统。上述侧栏合栈是独立候选；本轮没有将其画为默认或自动折叠行为。

`widget-toolkit-r01` 已在原页保存三个尺寸的 opened Dock 目标和四个状态目标，早期结果见 [revn9逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-widget-toolkit-r01/widget-toolkit-design-review.md)。Palette、Hierarchy、Details 各有独立裁剪/flow 内容区，Designer 为独立画布区；这只证明设计结构，滚轮、首尾滚动、tab persistence 和焦点仍待真实宿主。640 左栈约 95.7 逻辑像素，900 约 134.7：页签及搜索出现裁剪/换行，G03 保留实测设计差异，未通过入口可达或 1 DIP 比较。15:50Z原字体实测Resize Slot自然框为67×16；在保留96×28按钮、原字体和8逻辑像素padding的前提下，将六个copy根改为居中flow，文字改auto-width，未改主件、主题或产品。关闭重开后六处文字均为67×16且几何居中，首张1280两行问题已在设计端消除；[revn11逐图记录及测量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-text-flow-r01/review.md)保留全部七板复核与其余裁切缺口，不据此关闭UE/EXE比较。

七板的 15 条 Palette 条目来自现有 editor showcase descriptor 的源码标签，不代表 GUI catalog 的发布顺序或实际搜索结果。锚点板使用现有回归的 `CANVAS_SOURCE` 数据作为明确标注的设计夹具；画布网格没有绘制运行界面，不计实时预览。Graph 采用不可用颜色及能力标注；640 的 More 只描述共享 overflow，三条 Designer 操作分别列出，没有伪装成 Preview 点击绑定。焦点蓝框与无匹配文字均为设计终态，不能关闭 G09 或键盘/执行验收。

16:15Z起接续修订被动页签，依据实际`SDockTab`默认Ellipsis、左对齐/垂直居中及`SDockingTabWell`均分宽度。直接调用既有`penpot-text-overflow.ts::measuredEllipsis/cachedTextMeasure`，使用当前注册Roboto700/13.333333字体的24条native测量；保留完整原标题、原copy/sourceRef、30高父框及Dock尺寸，28处copy使用单行auto-width/row-nowrap、既有8像素左右inset。1280/900/640的可用文字宽分别约79.85/51.35/31.85，28处文字框及SDK textBounds均高16（保存position-data矩形，非墨迹）且在宽度内；实际保存/重开和七图见[revn12复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-tab-overflow-r01/review.md)。这些是原Button prototype的受控覆盖，未发布或声称复用了公共Dock/Tab主件。

完整标题保留在原root.name和本页journal.plan.source，display仅为该尺寸的设计显示值，不得导出成丢失全文的canonical属性。真实tab tooltip、溢出选择、快捷键、焦点与无障碍标题由Parent公共Dock合同接续，G03仍open。revn12只修订被动页签，14个搜索shape当时逐项哈希未变。未自动合栈、移动或隐藏面板。

17:10Z搜索修订已在同页保存、关闭重开为revn13。`SSearchBox.cpp:26`构造真实SEditableTextBox并保留change/commit与焦点行为；因此14个Palette/Hierarchy搜索保留完整内容，不套页签省略，只将原copy根设为row-nowrap/垂直居中/裁剪、文字auto-width，左右inset沿用原分配差所得8像素。原20高字段、字体、paint、component/sourceRef及source-contract不变，文字框均16高；28个root/text之外2950对象及Shared保持。[七图与保存请求核查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-search-shared-r01/review.md)保留首次post-ACK回读断言失败及同rev13后续只读重开对账，没有重放修改。640/中文长查询现在单行右裁切；真实caret横向滚动、IME、clear及切文档尚无GUI证明，G09仍open。当前Shared SearchInput为240×32/free分配，canonical dense覆盖/导出尚未闭合，保留旧copy和现有12份验证板linked实例，不detach、复制main或改公共主题。

## Palette、Hierarchy 和选择

Palette 从真实 `UiAssetPaletteCatalog` 和 reference imports 取项；目标区分原生控件、外部组件、类别和搜索无结果，当前GUI输入到过滤结果的接线尚缺。Library 使用真实资产发现/组件引用语义，不能复制 Palette 充数。重复项复用已登记 TreeRow/ListRow；键值使用稳定 descriptor/reference，虚拟行重新绑定后不得留下上一项的 selected/disabled/drag state。

本域 `palette-search-domain-r01` 已将查询视图接到真实catalog/Session，未改公共host、retained DTO、产品ZUI或共享组件。`set_palette_search_query`仅更新导航，保留完整query；基础文字按空白分词、Unicode lowercase后，在显示标签和真实Native类型/Component key/完整Reference中逐项AND匹配。UE `SPaletteView.cpp:202`使用TextFilter并显示语法错误，当前候选没有UE高级表达式、category tree、关键字/tooltip搜索或无限滚动，均保留差异，不以四项页替代最终UE树目标。

`palette_search_snapshot`保留原catalog与query/page的Arc、实际asset ID和既有`UiAssetSourceBuffer`分配/revision快照；`rows()`给出原目录索引与真实entry，`select_palette_search_row`先检查源/目录/query/page仍当前、当前源有效，再把本页行号映射到原`select_palette_index`。查询或分页A→B→A、源编辑/undo、导入重排和重新打开文档均不得使旧行重新有效。Host仍须提供自身instance/input context，Clone共享分配不等于跨host事件准入。Parent不可将过滤行号继续发给旧的无快照collection路径。

无匹配时快照为0行、Previous/Next不可用、Palette选择/拖放目标清除；查询和清空不改源资产、不加undo。Source/catalog重建和有导入的preflight命令路径保留query、按新结果数钳制分页并重协调真实选择。实际插入仍用原v2引用与文档history。12项回归覆盖错误索引、同名不同ref、中文/完整query、空结果/清空、极端分页、源/查询/页ABA、invalid/reopen、导入重排及真实插入/undo/redo，尚未编译或执行，见[本轮源码/验证证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/summary.md)。原pane仍保留完整labels，原NodeProjection空labels/default bootstrap及retained空集合早退未改；公共SearchInput→Host→快照/结果DTO→Ready-empty shell、hover/hit清理、焦点与滚动由Parent沿既有合同接续，G09仍open。

Parent第五次要求的[G09 view/retained提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-search-domain-r01/view-retained-proposal-r01.md)现已补齐，公共源码仍未应用。一次pane snapshot同时驱动全量/scoped投影、Ready-empty和页面行；原完整palette_items保持兼容。原四项control保留真实目录身份，callback传页内row与原snapshot/frame，在真实注册窗口、可见active pane、源及命中帧当前后进入原Session受检选择。生产native buttons从borrowed hit直接转owned，提案因此在borrowed route取得引用，覆盖这条实际入口，未用测试包装器替代生产路径。scoped合并还排除旧Palette行，避免把被过滤行当Details加回。

输入复用SearchField及原文本编辑/Unicode/IME通道，保留完整query/value_text。连续query只在同一有效source和当前field allocation延续，行/分页则严格要求原query/page和当前发布帧；换源/隐藏的结构发布退休本field，不复活旧ticket，不触发Preview Stop/Cancel。8项回归仅在提案中定义，0执行；38路径语法/hunk静态通过不能代替编译。Parent须审查公开callback/DTO/焦点hook并接原native hide/IME/popup/Tab及多pane绘制；原control-id-only focus painter、hover/capture、640裁切和真实EXE仍open。11原域源、97ID/6events及原Layout/Canvas identity实际hash保持，未重apply或扩展旧请求。精确路径、preimage、8测试和剩余边界见同一报告，不再创建另一份域合同。

拖放先解析现有 target chooser，展示可插入/拒绝原因；目标包括 child/sibling、合法 mount 与原容器的 slot 类型。释放执行一次实际插入/改父事务；取消不改变文档。当前 `node_projection.rs::palette_document` 固定四个源节点及步长 4，后续采用 UE 列表/过滤时须同步投影及输入身份，不能把过滤后行号派发为原 catalog 索引。空 labels 保留原 disabled 原型也是默认 bootstrap 入口的现有合同；retained scoped pane 的空集合早退则作用于新 DTO，不直接沿用旧 pane。真实 catalog 先含内建项，空文档/清 imports 并不造成空目录；搜索接线未找到，因此当前没有 GUI“搜索为空后残留”的复现。[源码审计和具体核验输入](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/palette-empty-boundary-r01.md) 分开记录这些边界；后续区分未准备、Ready-empty、真实失败，并保留搜索、标题、Details 及正确 generation/hit artifacts。

Hierarchy 保留父子层级、展开和过滤，搜索结果不改变实际父链。单选/多选通过现有 `UiDesignerSelectionModel` 的 primary/sibling/parent/mount 表达；该结构已有 `is_multi_select`，不等于当前 GUI 的 Ctrl/Shift 选择与批量编辑已验证。选中树节点同步 Designer 和 Details；Designer 命中同一源节点同步树，Source 定位使用现有 outline/cursor 路径。

右键菜单以实际 command availability 驱动；目标包含 Rename、Cut/Copy/Paste/Delete、Move/Reparent、Wrap/Unwrap、Open Reference、Extract/Promote。目前已定位 Move/Reparent/Wrap/Unwrap/Reference 等 session 合同；剪贴板、重命名、锁和可见性仍是逐项未核实能力，不能仅凭 UE 菜单文字启用。多选字段混合值必须有明确状态，批量修改应一次撤销，未接入前不展示可执行批量动作。

## Designer、焦点和命令

| 触发 | 目标行为 | 已有合同 / 待接入 |
| --- | --- | --- |
| Designer Select / ResizeSlot / PreviewInteract | 选择源节点、编辑父子 slot、把输入送入实际 runtime 控件 | 三条原 event route 保留；PreviewInteract 使用已有 preview host 和 input dispatcher。 |
| UE W / E | Layout Transform / Render Transform | 依据 `DesignerCommands.cpp`；现有 ResizeSlot 不可冒充完整 Render Transform，只有真实能力与命令映射核实后绑定。 |
| UE G / L | Outlines / Respect Locks | 同源命令；outline/lock 状态与命中规则尚待核实。 |
| 保存、撤销/重做 | dirty/源文本/选择/preview 与相关外部效果一致 | 使用现有 source buffer、command journal/undo stack；保存错误保留实际诊断。共享快捷键由 Parent 路由，避免重复注册。 |
| 搜索/字段输入焦点 | 字符输入、组合输入、选文与本地撤销正常 | 键盘焦点位于输入时不将 W/E/G/L/Delete 等穿透到画布。组合输入和未提交字段不触发文档命令。 |
| 右键、Anchor popup、下拉、drawer | 合法 anchor/z-order、键盘选择、Escape/外点关闭和焦点返回 | 记住 opener 的稳定 focus identity；关闭后若 opener 已不存在则回到所属有效面板，不能留在已销毁弹层。 |
| Designer 中拖拽 | 逻辑坐标命中、拖动预览、一次事务、释放/取消 | 使用当前 source revision 与有效 parent frame；拒绝旧 generation 输入，不根据截图坐标直接改 slot。 |

UE Hierarchy 的 `OnKeyDown` 先处理 DesignerCommandList，再处理自己的 Rename command；Zircon 需在共享 focus owner 的前提下实现相同行为目标，并验证输入控件消费事件。UE 代码还留有 undoable selection TODO，不能把单纯导航写入源文档作为“UE 一致撤销”；实际结构/属性编辑才进入文档事务。

## Details 的父容器语义

Details 的 Slot 是父子边，Layout 是节点自身。选中根无 parent slot 时显示根属性，不伪造 CanvasSlot。按父容器提供对应 Linear、Flow、Grid、Overlay 或 Canvas/Free 字段，切换选择时不能沿用上一个容器的数值。字体/颜色等仍走已有 widget prop/state、style inspector 和父会话主题，不在本域另造 schema。

源码明确存在：`UiCanvasSlotPlacement` 保存 anchor、optional anchor_max、pivot、position、offset 和 auto_size；v2 `surface_tree/slot.rs` 仅在父容器为 Free/Canvas 时解析该 placement。`child_frame.rs` 已根据 point/stretch 两轴计算布局。当前 `inspector_semantics.rs` 的两条字段投影仅覆盖 HorizontalBox/VerticalBox/Overlay/GridBox/FlowBox，没有 Canvas/Free 分支。Overlay 的九点放置及 `layout.z_index` 不能当作 Canvas 的完整 min/max 模型；Canvas builder 的层级字段是 `layout.z_order`。

| Canvas UI 字段 | 原 v2 child mount slot 路径 | 条件 |
| --- | --- | --- |
| Anchors Minimum X/Y | `layout.anchor.x` / `.y` | 缺省 max 解析为同一 point anchor。 |
| Anchors Maximum X/Y | `layout.anchor_max.x` / `.y` | 与 min 不同的轴为 stretch，显式 max 不能在导出时丢失。 |
| Alignment X/Y | `layout.pivot.x` / `.y` | 对应 UE alignment，point 轴参与位置，stretch 轴不再扣除 pivot。 |
| Position X / Offset Left | `layout.offset.left`，叠加已有 `layout.position.x` | point 显示 Position；stretch 显示 Offset Left。 |
| Position Y / Offset Top | `layout.offset.top`，叠加已有 `layout.position.y` | 同上，按 Y 轴独立切换。 |
| Size X / Offset Right | `layout.offset.right` | point 为尺寸，stretch 为右边距；auto_size 的 point 轴使用 desired size。 |
| Size Y / Offset Bottom | `layout.offset.bottom` | 同上，按 Y 轴独立切换。 |
| Size To Content / Z Order | `layout.auto_size` / `layout.z_order` | 使用已有运行时属性；零尺寸与 constraints 的分支仍需实际对照，不假定所有边界已等价。 |

这是面向既有运行时的编辑器补全提案，无需另建 UI runtime。已有 `layout.position` 与 offset.left/top 同时非零时，Details 的 Position 必须显示它们的有效和；普通 Position 编辑可保留 position 并改 offset。保持几何的重基也要计入该分量。用户显式使用 Ctrl 重置位置时，两个分量都归零；此时属性结构变化属于该操作的明确效果，不在无操作的 roundtrip 中自动归一化资产。

## Anchor popup 与重基

`CanvasSlotCustomization.cpp` 实际含 16 个预设，组成四行四列的语义格；最后一列/行与九点组有分隔。以下 min/max 是设计目标，不是预造的 Penpot 组件身份。

| Y 预设 / X 预设 | Left | Center | Right | Fill |
| --- | --- | --- | --- | --- |
| Top | (0,0) → (0,0) | (.5,0) → (.5,0) | (1,0) → (1,0) | (0,0) → (1,0) |
| Center | (0,.5) → (0,.5) | (.5,.5) → (.5,.5) | (1,.5) → (1,.5) | (0,.5) → (1,.5) |
| Bottom | (0,1) → (0,1) | (.5,1) → (.5,1) | (1,1) → (1,1) | (0,1) → (1,1) |
| Fill | (0,0) → (0,1) | (.5,0) → (.5,1) | (1,0) → (1,1) | (0,0) → (1,1) |

选择预设按一个 Changed Anchors 事务完成。顺序严格对应 UE：先设置 anchors，以操作前的 alignment/pivot 重基；再由 Shift 将 point 轴 alignment/pivot 改为 min，stretch 轴改为 0；最后由 Ctrl 把左/上位置归零，stretch 轴的右/下边距也归零，point 轴保留尺寸。普通选择保持可见矩形，Shift 单独改 pivot 后 point 轴的位置可以变化，不能为了保持矩形再补一次 offset。Shift+Ctrl 同时生效。UE 操作后 dismiss menus，Zircon 使用共享 popup focus-return 合同。

保持矩形的重基推导基于父内容区的逻辑布局：某轴父长度 `P`，旧 widget 在父局部的起点 `r`、有效尺寸 `s`，新 min/max 为 `a/b`，该步使用的 pivot 为 `q`，已有 position 分量为 `p`。popup 操作中的 `q` 是 Shift 修改前的值；显式执行其它保持矩形操作时才采用该操作指定的 pivot。只在有效布局与无并发 source revision 变化时运算：

- anchors未变：保留原offset，跳过几何重基；Shift/Ctrl仍按后续步骤执行。
- anchors已变且point (`a == b`)：leading offset = `r - P*a - p + s*q`；旧轴为stretch时trailing size取测量`s`，旧轴仍为point时保留原right/bottom尺寸字段（含0的intrinsic fallback）。
- stretch (`a != b`)：leading offset = `r - P*a - p`，trailing margin = `P*b - r - s`。

Y 轴同式对应 top/bottom。上述是对现有运行时公式的推导，并非已运行的测试。Free 父容器还会经过约束求尺寸；auto-size、零尺寸、负边距、min/max constraints 和嵌套缩放须单列回归，不把 Canvas 的常见正尺寸推导直接当作全部等价证明。`CanvasPanelSlot.h` 的 UE `RebaseLayout` 默认 `PreserveSize=true`，不能误用其中显式 false 的 point→stretch 归零分支作为普通预设行为。

拖动锚点参照 `CanvasSlotExtension.cpp`：begin 保存原 anchors/geometry 并 capture pointer，move 以父画布局部坐标除以父尺寸形成 delta，边/角分别修改 min/max；拖动范围保持 min≤max 且在 0..1，零父尺寸拒绝操作。未按 Shift 时使用 UE 的 0.1 主分隔附近吸附（proximity 为该间隔的 0.1），Shift 暂停吸附。Ctrl 对受拖动的边/角重置相应 positional offsets。drag 的 Shift 含义与 popup 的 Shift 不同，必须分开验证。数值字段不因拖动限制而静默重写已有合法源值。

当前 `style_state.rs::set_selected_slot_semantic_field` 每次 clone 文档并调用一次 `apply_document_edit_with_label`。连续调用十多个单字段 setter 会生成多个撤销项；预设、alignment 联动和重基应在一个 clone 中校验全部字段，然后一次 apply，形成一个命令。拖动中间帧可用于真实预览，释放只提交一个最终事务；Escape/capture lost/文档切换取消并恢复起始源，具体 gesture staging 接现有命令生命周期，不创建第二个 undo 系统。

本轮已把预设事务的限定候选写入 `session/canvas_anchor_state.rs` 与子模块 `geometry.rs`，公开 session 方法为 `apply_selected_canvas_anchor_preset(index, shift, control)`，index 按上述四行四列从 0 到 15。当前 preview 的 source-node/parent/source-path/空实例路径必须唯一一致，使用父内容框而非外壳框，并拒绝旧/无效几何；在同一clone内只写实际变化的slot语义字段，再沿原 `Changed Anchors` source/undo一次提交；缺省anchor_max随min解析，首次stretch才补齐max两轴。早期无条件12字段写入已由下述r03取代。它不改 schema、auto_size/z_order 或原节点 props/events。限定 v2 单选、正尺寸、auto-size 关闭的原生 Canvas；Free 约束、组件实例、零/折叠几何、auto-size 尚待对照，当前明确拒绝。早期八项回归保留，当前r03共22项定义，编译/执行均0；公共 popup/字段/键盘/焦点、拖动及正式组件导出仍未接通，候选不是完整 G04 修复。源 diff、指纹和真实验证状态见[域内交接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-domain-r01.md)。

## Canvas Details 已有字段的投影接续

`style/inspector_semantics.rs`现按真实Canvas父边建立`Canvas Slot`语义组，并将既有slot kind投影为`Canvas`。使用现有列表/选中路径/值输入与Session编辑、删除、undo/redo，不增加DTO、控件、事件、主题或运行时。投影只显示源中已有的min/max anchors、pivot、position、四offset、auto_size、z_order，保留原literal与缺失属性；Canvas不走Overlay字段，Free不借用Canvas几何语义。UE依据仍为`CanvasSlotCustomization.cpp`的anchors/offsets/alignment和`CanvasPanelSlot.h`的Size To Content/ZOrder。

这是原Details数据链的实现增量，不能代替UE专用typed行、默认值/继承来源、点/伸展下Position/Size与Offset动态标签、16preset弹层、drag/focus/IME或正式Penpot导出。原锚点Penpot设计保持；源资产97IDs/6events、公共host/catalog/tokens/Starship及原Canvas预设事务保持。新增3项真实Session回归覆盖投影无写入、非Canvas隔离、offset源编辑到真实几何/保存重开/单undo与no-op，以及删除属性的缺失/undo/redo；源码已编写、执行0，详见[域内候选](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-slot-details-r01/candidate.json)。G04与整族验收继续open。

## Events、Graph、动画和运行实例

Events/Bindings 复用现有 binding ID、event option、action kind、route/target/payload 编辑器；修改后必须写回原事件并保存重开。事件跳转使用实际 route/源定位；打开 Widget Graph 需要真实 Graph 文档/执行合同。Designer 切 Graph 要保留 node/variable 选择，Source 模式继续表示文本。Graph、My Blueprint、Bookmarks、Replace References 的能力和共享图宿主依赖仍为 G02/W11/W12/W20/W21。

事件链已按下面四层分开核实，不能把其中一层完成当作全链完成：

| 层 | 已定位的真实合同 | 仍缺的证据 / 设计处理 |
| --- | --- | --- |
| 源声明与编辑 | v2 `asset.rs` 的 `UiV2NodeDefinition.events: Vec<UiBindingRef>`；`binding_state.rs` 的 add/delete/id/event/kind/target/payload 通过文档事务应用。 | 保存重开和真实控件消费仍待验证；切换 None/Route/Action 要保留合法字段并显示实际校验结果。 |
| 候选与定位 | `binding_inspector.rs` 有 16 类事件；`payload_editing.rs` 根据事件和控件关键字生成 route/action suggestions。 | 这些建议不是已注册 handler 目录，不能显示“已连接”或直接算可执行。Goto Definition/事件图跳转尚无本域真实调用链。 |
| 绑定检视 | `designer_state.rs::dispatch_preview_interact_at_preview_index` 选择节点并返回含 route/action/payload 的摘要。 | 此方法本身只构造/保存摘要，没有执行宿主操作；其返回值不能作为按钮效果或事件执行证据。 |
| 原生预览输入 | retained `asset_events/preview.rs` → host session → `dispatch_preview_input` → `preview_host.rs::dispatch_input`，使用当前 SurfaceFrame 和真实 pointer/navigation dispatchers，重建 dirty surface 并发布 input result。 | 已定位通用 `dispatch_template_action_invocation`，但 preview 的 bool/result-presentation 路径尚未接它；Runtime pointer 报告未保留原 binding ID/compiled handle，通用 consumer 又把 record 压成 effects。需按 [WG-E01～06](widget-ui-graph-consumer.md) 接来源、同次结果、实际 record/error 和失效边界。当前没有 EXE 外部操作或跳转证据。 |

具体事件验收分成：输入/焦点等本地控件状态、已绑定属性赋值、宿主 Route/Action 完成结果、跳转到真实事件/源码。每一类独立记录源 binding ID、目标、source generation、输入回执及实际效果；失败保留原错误。默认候选中包括项目保存等真实动作，不能为了截图随意触发无关副作用，优先复用任务范围内的 preview/selection/属性操作。

Actor [AB-WG-01～07](actor-blueprint-shared-contracts.md#widget-graph实际可复用边界与未接通项) 已接续。Widget 不把 `UiBindingExecutionReceipt` 的限长诊断标识或执行计数当精确来源/外部完成，也不复用 GeneratedBottomPane 专用 canonical slots 反查任意 binding。Route 具名 JSON object 与 remote UiBinding array、Action 空 payload 保持既有边界；VM/Editor ID grammar 差异交 Parent/VM owner，不静默改名。共享 report/consumer/SourceLocator 修改由 Parent/Runtime 原 owner 接续，本轮只补 [来源与最小接入合同](widget-ui-graph-consumer.md)，未改 `.zui v2`、Graph 模式或执行后端。

Animations list、Sequencer drawer、keyframe 按钮和曲线窗保持 UE 目标：真实动画创建/重命名/删除、widget 属性轨道、关键帧、scrub、播放/停止、撤销和 drawer focus-return。`WorkbenchTimelineStrip` 仅为已登记的布局候选；未确认 widget animation schema、时钟和属性应用合同前不把它接成假动画工具。该依赖由 Parent 指定动画 owner，本域不复制动画 runtime。

进一步核对已有基础：`zircon_editor/src/ui/timeline/model.rs` 提供 `TimelineModel`、typed track/key/selection/range，以及返回 inverse delta 的 `apply`；可作为 Widget 轨道 UI 的共享基础。`animation_editor/session/timeline_foundation.rs` 已把现有 sequence 投影到该基础，但其运行时 `AnimationTrackPath` / sequence bindings 使用场景 `EntityPath`、`ComponentPropertyPath` 和 scene target ID，不能将 widget node ID 直接拼成 entity path。当前动画编辑器 capability table 把 `animation.preview.runtime` 标为 false；此表只说明该编辑器自身能力，不代表引擎全部动画运行能力缺失。

另一个同名 `zircon_runtime/src/ui/surface/timeline.rs` 是 `UiDebugTimelineStore`：保存 Reflector 调试帧、选择旧 snapshot，不参与 runtime 帧调度或 surface 恢复。它不能用于 UMG 关键帧/播放验收。当前 v2 文档类型未声明专门的 animation collection，UiAssetEditor session 也未定位到 widget 轨道持久化/采样接口；保留模型缺口，不能把现有 `props/state/slots` 的可扩展 map 当作已存在的动画 schema。

给动画 owner 的最小输入是：稳定的 `.zui` 资产/组件/源 node/property 定位，目标属性类型与实例覆盖规则，轨道存储归属与导入/导出保持方式，采样时钟和写入现有 Surface 的接口，scrub/stop/reset 如何恢复值，编辑事务与源 generation 失效规则。优先复用已有 `TimelineModel` 和动画采样能力；只有 owner 确认该适配合同后才决定资产表示和接口修改，本稿不新增 `.zui v2` 字段或第二套播放器。

Preview 复用 `UiAssetPreviewHost::new_v2` → `UiV2SurfaceBuilder`，pointer/navigation 使用现有 dispatcher。源编辑使旧 frame 失效，重建后的输入与当前 source generation 对应。预览运行状态与源文档分开：按钮、输入和导航实际发生；Reset 真实重建实例，失败有诊断。独立 preview 资产/Scene Settings 的产品形态仍待 W16/W17 决策，不添加无 scene 后端的装饰字段。

## 向 Parent 提交的具体依赖

| 提案 | 所需共享合同和本域消费 | 状态 |
| --- | --- | --- |
| P-UI-01 | 现有 Starship panel/header/tree/field/toolbar 的 class 角色映射及中文字体；本域只在原资产接入其 imports/classes。 | 待 Parent；不改 catalog/tokens/Starship。 |
| P-UI-02 | 共享 Dock 独立 pane slots、tab persistence、Window 菜单、overflow；本域提供 Palette/Hierarchy/Designer/Details/Source/结果/动画身份。 | 待 Parent；G03 和 640×520 紧凑布局仍开放。 |
| P-UI-03 | 现有 PropertyEditorRow/NumberField/Dropdown/IconButton/Popup primitives 组成 Anchor popup 与两轴数值组；只有共享目录确缺可复用组合时再由 Parent 决定公共组件。 | 本域负责 Canvas 字段/原子事务提案，无新 runtime 需求。 |
| P-UI-04 | 共用 graph/document、动画模型/轨道、导航与 Reflector 入口；要求真实类型/事件/属性接口、focus/undo owner。 | 图表面由材质聊天牵头；时间轴/曲线/播放由动画聊天维护；Parent 协调跨域合同，Widget 业务适配仍未实现。 |
| P-UI-05 | 原资产 importer/exporter 保留非视觉字段，支持真实 native instances；LayoutReviewCase 和 normal native evidence producer 按真实 host 生成。 | Parent 所有；本域不改插件或捕获协议。 |

后续在已分配的真实 Penpot 文件中按本稿编辑原资产导入结果；保存、关闭、重开和官方导出前不修改产品 ZUI。真实局部样式实验优先改变一个本域 panel 的既有样式属性（如 padding 的共享 token 引用），再导出、加载普通 EXE 并比较同状态画面；全局 token 实验须由 Parent 安排至少两个窗口族。未完成链路继续保留 [验证输入](widget-ui-validation.md) 的 pending，不能以本设计稿关闭窗口族。

## 共享 owner 的具体消费合同

本节向已指定 owner 交接 Widget 消费需求，不授予本聊天修改公共路径的所有权。复用的源码基础已只读核实，接口存在和 Penpot 文件已分配均不构成可见组件或后端接入验收。公共目录、token、导出及原生捕获合同仍由 Parent 集成。

2026-10-24 已接续动画 owner 的真实交付；本聊天 12:32:28Z 认证只读复核文件已到 revn12，3 个 main UUID 保持一致。组件/main/copy 身份、内层 Canvas 与外组合的源边界，以及 Widget 属性类型、事件、采样/恢复和历史接入见 [动画消费者合同 r01](widget-ui-animation-consumer.md)。公共文件仍为该时点revn0/仅根节点；本域没有复制主件或新建共享控件实例。原 `UiSurface::mutate_property` 可复用但没有本域动画宿主适配，反射 SetProperty 不能代替真实属性写入；WG01～07 保持 open。

早期[受控预览API提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/preview-property-adapter-r01.md)的三处公共 Surface identity 可见性 patch 仍未应用。现已沿 Parent WG-C01/C04/C05 在本域写入 [visible 窄 Host 候选](widget-ui-preview-visibility.md)：当前有效 v2 原生 Panel/Label、无 import/style/binding/event/repeat；使用该 Host 创建且 Clone 不继承的单次 gesture lease，加实际 frame/incarnation/源 revision/sequence 校验，无需修改公共 Runtime。原 preview builder 使用真实 route URI 登记 root source path；Cancel 保留有效起点/attribute presence，Reset 明确重建当前源，builtin 拒绝。检查真实 Accepted/Unchanged/Rejected、binding/dirty 与属性回读，发布和 present 分层。13 个 Rust 文件格式通过、11 项回归源码未运行，GUI/持久动画未接，不关闭 WG 或 W10。

| 合同 ID / owner | 复用的实际入口 | Widget/UI 需要的表面合同与域内边界 |
| --- | --- | --- |
| UI-SH-TIME-01；动画 `01a16d51-741b-73b3-a61a-2189cd38f81e` | `zircon_editor/src/ui/timeline/model.rs` 的 `TimelineModel` / `TimelineTrackView` / `TimelineSelection`；ruler/keyframe/section lane | 同一 playhead 和横向时间范围驱动轨道、标尺、曲线；轨道列表与 lane 纵向滚动同步。共享层接受稳定 TrackId/KeyId、时间单位/显示 FPS、类型与 selection，发出 seek、选择、拖动/复制/删除关键帧、缩放和平移意图。Widget 域提供 `.zui`/组件/源 node/property 映射、动画选择与可撤销 delta；不能依赖 scene EntityPath。KeyId 不应只用可修改的时间值作为稳定身份。 |
| UI-SH-CURVE-01；同一动画 owner | `zircon_editor/src/ui/curve/model.rs` 的 `CurveModel` / `CurveView` / `CurveElementRef`；`curve/canvas.rs` 的 `CurveCanvasTransform` | 支持 Step/Linear/Hermite、key/in/out tangent 选择，标尺时间与 timeline 对齐；曲线变换只消费本地 viewport，实际 DPI 由宿主转换一次。共享层提供多选/框选、拖动、数值提交、范围 fit 和取消；Widget 提供有效数值与 vector/color/margin 分量映射。离散和 Quaternion 不伪装 scalar，未支持类型保留 lane 与诊断。 |
| UI-SH-PLAY-01；同一动画 owner | `zircon_editor/src/ui/preview_scene/preview_scene.rs` 的 `PreviewPlayback` 值对象和现有 toolbar primitives | 需要 playing/looping/rate/time_seconds 绑定，Play/Pause/Stop、Seek、单步与区间首尾命令，以及实际可用/失败状态。共享控件不持有第二个采样时钟或播放器；Widget 的现有 UiAssetPreviewHost/Surface 接收经 owner 确认的采样结果。scrub 不写入持久源；key 编辑才形成文档事务；stop/reset 恢复策略由 Widget 域明确。UI 预览不因此被迫新建 scene/world。 |
| UI-SH-GRAPH-01；材质 `01a16d50-6c7f-7370-853d-4581184b66a1` | `zircon_editor/src/ui/graph/model.rs`、`canvas.rs`、`commands.rs`；`GraphModel` / `GraphCanvasState` / `GraphClipboardModel` / `GraphDeltaCommand` | 单一坐标变换和 viewport，复用节点、typed pins、连线、选择/框选、平移/缩放、拖动、连接预览/断开/替换、搜索和上下文菜单。Widget 提供节点/事件/变量/类型、`can_connect` verdict、delta、编译/执行诊断及 source locator；共享表面不拥有 Widget Graph 资产，不把绑定摘要生成器当图执行器。 |
| UI-SH-HOST-01；Parent | 现有 Dock、DocumentHost、ActivityDrawerWindow、属性行、菜单与主题；原导出/normal-frame producer | 文档/面板稳定身份、选中跨模式定位、弹层关闭/焦点返回、Window 菜单/持久布局、640×520 约束，以及真实资源/frame/source generation 证据。domain surface 只提交内容与事件，不复制 window host 或给捕获器设置成功标记。 |

时间轴和曲线的一次拖动需共享 gesture begin/update/commit/cancel 语义：预览帧更新不积累多条 undo，commit 由域 owner 返回逆向 delta；Escape、capture lost、切换文档/动画或 source revision 失效都必须取消并恢复有效起点。键盘快捷键在所在 surface 聚焦时生效，输入框保留自身文本编辑，drawer 关闭后回到 opener。曲线和轨道必须使用同一真实选中动画/属性，不各自维护平行 selection。

节点图表面的编辑需保留现有 `GraphDeltaProofV1` / `PreparedGraphDeltaV1` 的 revision、retention bound 与一次 permit 合同，失败保持域模型不变；不能以直接调用简单 `apply` 绕过已存在的命令生命周期。节点 geometry、pin anchor、连线命中和 selection 来自同一 transform/当前帧；缩放、滚动和 150% DPI 后仍与实际绘制一致。域提供稳定 SourceLocator，使从 Designer 所选 widget 到变量/事件节点以及从节点回 Details/源位置可真实导航。

每个 owner 的具体交付输入包括：真实 Penpot file/page/component UUID 与版本、实例/slot/参数/事件合同、共享样式继承与 override/reset、canonical ZUI/component source paths、导出支持边界、后端适配 API/焦点与事务 owner，以及至少一个真实消费者的行为与图像证据。中立Graph及时间控件尚无已证明发布的共享组件 UUID；已有基础Shared四主件与rev2共享样式按[受控消费](widget-ui-shared-consumer.md)原身份接续，不推导尚未发布的其它主件，也不把基础 Rust trait 的存在记为消费者接入。

独立 Penpot 已复用实际 plugin dist/bundle、source closure、认证和隔离浏览器导入局部源样本；完整窗口因 TreeRow 的 SDK nil 差异失败，公共修复由 Parent/原 pipeline owner 接续。后续仍用同一 file/page、正式源 owner 和严格 save/export 校验，不另建文件/发行物。凭据只传路径，不把内容写入计划或消息。

Canvas Anchor Preset / Picker 已在真实专属文件形成可编辑业务组件 `canvas-anchors-r01`；16 项 min/max、48 个实例链接、默认/焦点和 hover 终态/中文长文本三板已保存关闭重开并逐图查看。实际组件与 board UUID、flow、依赖、源/主题指纹、官方 PNG 返回 null 和所有差异见 [证据](widget-ui-penpot-evidence.md)。这些状态设计不证明键盘/动画/预设事务执行；engineComponentId 仍为空，未通过正式 ZUI codec 写回新业务组件，也不赋予新 runtime 或私有全局样式。

## Canvas 预设弹层目标凭据

原同步`apply_selected_canvas_anchor_preset`继续沿一次source/undo事务执行。弹层适配新增crate内`begin_canvas_anchor_preset`→私有字段`UiAssetCanvasAnchorPresetTicket`→`apply_canvas_anchor_preset`/`cancel_canvas_anchor_preset`：复用原UiAssetSourceSnapshot的buffer allocation/revision，记录原route、last-valid generation、完整selection与已发布UiSurfaceFrame；只在Design/Split且当前原生Canvas几何可重基时打开。提交不重新猜测当前目标，旧source/选择/preview、新弹层及已消费凭据返回RetiredCanvasAnchorPreset；选择离开再返回、mode往返、关闭重开均不复活旧操作。无效preset index不消耗当前有效凭据，已接纳的no-op仍结束该弹层。

Session只保存一个轻量lease。显式选择、command selection/undo和set_mode退休它；旧cancel/submit不能退休新弹层。begin/cancel不写源或history，accepted提交仍调用原anchors/rebase→Shift pivot→Control reset及Changed Anchors一次事务，不建第二套history或运行时。几何求解器将position计入拉伸local坐标，原right/bottom公式已核对相符，未改重基公式。

八项新增真实Session/preview/undo回归已写源码，原11项测试字节前缀保持，执行0。原layout/canvas identity/filter/失败不变，不沿旧request运行扩展后的模块；新范围仅为未提交的`validate-editor-canvas-popup-01`/revision canvas-anchor-popup-r02，request/attempt/serverReceipt仍null。Parent仍须在真实popup打开时持document/session guard捕获ticket，在原菜单提交/ESC/关闭消费对应API，并处理真实active pane、hide、capture/IME/opener/focus；本轮未改公共host/menu/focus、产品ZUI或Penpot，不称GUI已接。来源、增量及最小托管范围见[本域回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-popup-r02/candidate.json)。正式编译、popup/focus实测、正常EXE/DPI/双端及G04保持open。


## Canvas 锚点的缺省源字段保全 r03

本次从UE `CanvasPanelSlot.cpp::RebaseLayout` 和 Zircon `layout/pass/child_frame.rs` 核实一个具体源码差异：旧适配对当前预设和点→点切换都以测量宽高覆盖right/bottom，且无条件写12字段。Zircon点偏移尺寸0使用原intrinsic fallback，因此这会固化现尺寸、补出缺省字段，并可能让当前默认预设形成无意义编辑。[源级反例](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-source-preservation-r03/source-counterexample.json)明确未执行Rust/GUI，不写成已现场复现。

原域内[geometry](../../../zircon_editor/src/ui/asset_editor/session/canvas_anchor_state/geometry.rs)和[Session事务](../../../zircon_editor/src/ui/asset_editor/session/canvas_anchor_state.rs)已改为锚点不变保留offset、点→点保留原尺寸字段，只写实际语义变化；implicit max继续跟随minimum，首次stretch才显式写完整max两轴。anchors/rebase→Shift pivot→Ctrl reset顺序、原source/frame/lease守卫及一次`Changed Anchors`提交保留；默认预设即使无源变化也消费一次popup。原生Canvas/单选/正尺寸/auto-size关闭及Free/组件实例限制不变，未改公共Runtime或host。

[原测试文件](../../../zircon_editor/src/tests/ui/ui_asset_editor/canvas_anchors.rs)原19项字节前缀保持，追加3项使用真实Session/v2/runtime/history的回归：缺省预设四组修饰键无源/历史修改；点→点保留intrinsic sizing/缺省max/pivot/position/尺寸、往返和一次undo/redo；首次stretch补齐max两轴且重开矩形保持。**22项定义，编译/执行均0**。[当前闭包及检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-source-preservation-r03/validation.json)记录3文件格式检查、r02九源的6同+3本轮更新、23只读/保护输入及两原身份保留；静态检查不证明GUI已修。

[未投递验证意图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/canvas-anchor-source-preservation-r03/validation-intent.json)沿既有`validate-editor-canvas-popup-01`提案标明确切r03/22项，request/attempt/serverReceipt仍null；没有改原layout/canvas请求、filter/ownedPaths或重投。执行前继续由原Jenkins owner取得真实源/资源/writer/target准入并声明最小zircon_editor/defaults-off/features[]/lib/test-profile/locked/jobs1/threads1范围。正常EXE锚点操作、焦点返回、DPI和双端验收仍open，整族passed/accepted=false。


## Canvas 锚点指针移出保留焦点状态 r02

UE `CanvasSlotCustomization.cpp` 的初始preview viewport为64、content padding为2，cell68；内框由48到64，源码curve为0.40秒，Fill分隔1。原default和hover end frame组件已表达这些度量。本次仅从原hover板`180e9578-8294-803b-8008-d6ae833142ad`原生clone为board `392bc57b-e3cc-80a2-8008-d748e7191288`，位于(1080,1820)，320×440，保留原column flow和16个preset native copy。选中Center/Center的内框回到48、居中inset10，15×15 glyph及68焦点边界保持。它是移出后的可编辑设计终态，未演示或接入真实计时/hover动画。

Picker component `180e9578-8294-803b-8008-d6ae7d94d33b`、main `180e9578-8294-803b-8008-d6ae7788b380`沿用；新copy `392bc57b-e3cc-80a2-8008-d748e7191289`有原shapeRef。Preset component `180e9578-8294-803b-8008-d6ae776be849`及main `180e9578-8294-803b-8008-d6ae75d79d3b`不变；本次336个新对象中334个保有sourceRefs，8个旧main和2977个旧非根对象完整保留。新板背景消费Shared原surface.0颜色，不复制theme或Shared主件。[真实ACK和fresh重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/receipt.json)均为rev22，data hash `5237f1916db12c853f047fe5500c506e6a66dc20c392198ab6d0dceaab408229`；Shared rev5完整data hash未变。

两张浏览器画面为1600×1100/dSF1、212% fit，非官方Shape PNG、正常EXE或实际150%DPI。[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/review.json)记录64→48及焦点保持；不据此声称UE整窗≤1DIP。实际标题仍为源Roboto400/13.333333，未改字体或私有尺寸；注册WOFF字节真实200且匹配原engine-face，但WOFF内部PostScript名Roboto-Regular与CDP实际Roboto不同，十条normal400同名CSS并存，实际标题font check=false。[来源证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-anchor-hover-r02/font-source-analysis.json)将同名选择冲突列为待公共字体owner核定的推断；精确字形来源、文字ready、源codec和运行交互均open。本轮未改Rust、原ZUI或测试请求，r03的3个Rust哈希保持；其旧5文件闭包因本设计文档追加成为历史，不能再称整组未变。整族passed/accepted=false。
