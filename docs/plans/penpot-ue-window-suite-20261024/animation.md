# 动画、蒙太奇与时间轴窗口族

## AN-P04：旧 Graph 字符串命令的歧义拒绝与 lease 接续（2026-10-25）

已在原 AnimationDocumentMutation 生产入口加入最小准入：Remove/Connect/Disconnect 的目标匹配多个 named/sink 时返回 InvalidGraphTarget，连接源只核 named 唯一性；首个源写前拒绝，原 host 在事务 begin 前回传错误。合法旧 named `output` 与真实 sink 不迁移、不重命名、不批量编辑；唯一 named source 及唯一目标的原行为保持。未引入孤立 typed enum、第二历史/clock 或 Runtime DTO；完整 Named/OutputSink 选择、Additive 显式 pin 与实际菜单/画布仍待 Parent 的原 typed caller 集成。

四私有 Rust 实际增量，新增 5 回归定义，旧 4 项为完整字节前缀；覆盖同名/重复/multi-sink 的完整资产与 codec 保留、正常角色及原 Document/revision/current compile/last-good/dirty/history/Undo/Redo。rustfmt/diff/精确 change-guard 0 errors/0 warnings/0 suppressed，11 保护 pin 保持；9 项无本轮 Rust 运行证据，无新 Jenkins identity/提交/捕获。

已消费 Parent 的 attach/sync/复用 toolkit 与 dirty-generation 失败审阅：首次清退前原 close admission + exact source/初始 revision/无已接受编辑/新增或复用归属；拒绝保留 Session/source，原 restore/metadata/cleanup 诊断同时保留。toolkit 先退役而 DirtyRegistry 仍可能 generation 耗尽，故只移后 Session.remove 或 r02 active scope 都不是原子生命周期修复；公共实现仍 Parent。r02 原件保持未应用、替代 r01、不叠加；本次三域源后继使旧八前像现 5/8 匹配，整合须显式重基并复用同名 InvalidGraphTarget，不能重贴旧 patch。

[实际增量与五定义](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-legacy-admission-r01/implementation.patch)、[完整边界/调用合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-legacy-admission-r01/lifecycle-boundary-r03.md)、[逐 SHA 接续](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-legacy-admission-r01/lease-r02-current-compatibility.json)。原 Sequence/Curve/palette/Triggered 请求与失败独立保留；本域 active，真实 EXE/DPI/导出和全族 passed/accepted 继续 open，释放 0 B。

## AN-SQ-KEY-01：确切时间增删键与原历史回归（2026-10-25）

本切片受管结果接续：r01 `ue-animation-sequence-key-exact-20261025-01 / validate-sequence-key-exact-01` 实际 queue16722 → management #394 FAILURE；109B 控制回执 SHA085291df…明确拒绝 Control files are outside ordinary local validation。原因是本 owner 输入把不存在的 `.cargo/config.toml` 也作为 null 依赖声明，非两份 Rust 源触发；原输入/失败原件保留。已在同 request 透明登记不同 operation `validate-sequence-key-exact-r02`，只去除该不存在声明，源/recipe/完整 compiler closure/policy 未变；新 identity9d075a32…。r02 客户端 exit2/64.328s，在 pre-submit capture 报 HEAD/full index/complete source membership changed，未取得新 queue/server/native 回执，不能确定具体边界或写者。无再次重投/QueryOnly/全局停写/手工 grant；5 回归仍无 Rust 执行证据。原 intake owner按两原 operation 接续。


本回合实读 goal=active，接续监督恢复与原作者路径；不等待 Field/正式导出/Toolkit 完整。新确认 `mutation/sequence.rs` 的 AddKey 以 abs(diff)≤EPSILON 判重，RemoveKey 以 abs(diff)>EPSILON 保留；因此合法严格递增的 0 与 EPSILON/4 秒会被合并/连删，NaN 非目标键还会被 retain 的比较静默丢弃。原 Runtime `compile.rs::validate_key_time` 只要求有限、范围内、严格递增，没有这一合并规则。

已仅在原 AddKey/RemoveKey 将匹配改为请求 frame 经既有 helper 换算所得的确切 f32 秒相等；不修改 FPS/clock、compiler、key/schema/DTO、原 binding 解析、Document/history/CAS 或公共 host。邻近 key 的 value、两侧 tangent、顺序及原 target 字符串保留；缺确切 key 返回 false，原 store.prepare_mutation 继续返回 None。无效中间源的非目标时间保留给既有编译诊断，不借删除其它键规范化文档。

原 `tests/sequence_rebind.rs` 的 18 测试及整段字节作为前缀完整保留，新增 5 定义：distinct AddKey 与重复 no-op、精确 RemoveKey 和 codec 重开、缺键时原 Document/revision/编译/dirty/history 不变、既有一次事务及 undo/redo、NaN 非目标源保留。当前两源 SHA 为 `537b328506de81f2f27af5ef899f87e963bc472d46dfd16db8dd912dfaee59b7` / `e8a2c25ae9c03f428d82ccc3aef7c366bc6b2e96951966959404e12788cfd89a`；增量 patch `242373db5d842dee54dfba30e65ec41144fa180042e3f213beb5e777bf3b18aa`。rustfmt/diff/change-guard 静态 0 errors/0 warnings/0 suppressed，8 保护 pin 均保持；5 项未执行，测试定义不作通过。

有限受管验证只覆盖 root `zircon_editor` / defaults-off / features=[] / lib / filter `sequence_key_exact_` / locked / jobs1 / threads1 / 物理 E；Cargo test 使用默认 test profile，现 wrapper 未显式发 --profile。新切片正常自动 intake 独立登记，不扩张任何既有 identity。旧 curve-lossless `ue-animation-curve-lossless-20261025-01 / validate-curve-lossless-01 / queue16509` 的 submitted 和一次 authorization pending 原件保留；本轮仅通过已验证公共 transport 对该精确 queue 做一次 GET，10:12:47Z HTTP404，不推断已删、未入队或未执行，不重复 QueryOnly/POST。原 palette15804、capabilities/Triggered/Runtime Bool 仍各自独立。

[本切片报告](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-key-exact-r01/summary.md)、[精确增量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-key-exact-r01/candidate.patch) 与 [静态检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-key-exact-r01/static-validation.json)。真实菜单/拖动/typed Graph 目标、完整 rich Curve、官方 ZUI/源导出、正常 EXE/GPU/150%DPI/逐图1DIP及全族 passed/accepted 继续 open；本轮没有 Penpot/桌面/服务操作，释放 0 B。



## 2026-10-25：Curve TreeContent 接续与 palette 准入次序校正

原三张Curve打开态板接续到rev560/4959对象；补Shared SearchInput/三通道TreeRow/过滤状态，并把同一原Transport实例移至左树底部，保留六event/route/refs。新增210对象、12原生实例，当前54copies/522refs全解析、三个main与4179个作用域外对象和Shared数据保持；三对save/close/fresh-reopen板内RGB差0。[本轮设计与完整open差异](animation-design.md)，[当前范围](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-curve-tree-18/board-application-r01.json)，[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-curve-tree-18/image-review.json)。外wrapper不等于Curve正式源；640树视口28高，滚动首尾、Search/filter/输入、真实clock/ACK/GPU和EXE/DPI仍未验证。

原Jenkins owner已澄清actual-checkout Cargo没有独立prepare-only接口：[精确回执与后继对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-curve-tree-18/preparation-order-reconciliation.json)。先源审阅/声明→独立identity→normal automatic intake，服务端完整闭包、readguard/pool/E资源与cache writer全部准入后才Cargo；不把静态空闲/pin或不存在的prepare-only回执当门。本域既有queue15804/ue-animation-palette-20261025-01不重POST/query、原一次pending保留；source/resource/native产品回执仍null，verified Rust0/实际执行未知。旧supported-interface及旧三类请求身份不改；没有新Cargo/服务/桌面/freeze/公共源码/ZUI动作。必要证据与原失败保留，passed/accepted=false，释放0B。

AN-SQ-V01/V03/V04沿原范围接续，TreeContent设计的排列子差异已形成新证据；搜索/过滤/滚动/手势和真实播放仍未验证。

## 2026-10-25：Curve 打开态与独立 palette 正式提交

继续active并行推进：原Penpot页新增三尺寸Curve打开态，保留默认Closed板、原三main与六条transport事件。rev525的保存/关闭/fresh重开data hash一致；42原生实例/321 refs、570新增对象、4178原对象保护通过，r05三对板内RGB差0。已修打开页签造成的Transport出界和端点上空态提示遮盖，失败与原图均保留；[本轮结果](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-curve-toolkit-17/validation.json)、[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-curve-toolkit-17/image-review.json)。设计源是现有实体属性Sequence夹具，真实曲线求值/clock/预览、UE树搜索/左侧Transport/工具栏、小尺寸可达性和正式ZUI/正常EXE/DPI仍未交付，不升级族passed/accepted。

AN-P04叶包现有修复首次有独立实际提交：`ue-animation-palette-20261025-01` / `validate-graph-palette-01`，[queue15804](http://127.0.0.1:18080/queue/item/15804/)，默认特性关闭、features[]、独立plugin manifest、Windows host、lib/显式test profile、locked/jobs1/threads1、唯一palette filter。后续一次QueryOnly仍authorization pending；verified产品Rust测试0、server/native/source/resource回执未取得，原两Editor身份不变，不重投/换身份。见[独立identity与原始输出](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/palette-product-validation-r01/handoff.json)。不因等待共享完备交付停做设计或域内工作。

## 最新 typed Graph source lease r02 提案

已补 [AN-P04-ID02 r02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-typed-target-proposal-r01/lease-r02.md)：8文件未应用组合以原edit lease+active TransactionScope保护checked prepare、原CAS与dirty source检查，原r01保留。新确认restore rollback先删Session的旁路，Parent公共生命周期仍需处理；现有RemoveTrack/RebindTrack及Undo/Redo已有selection reconciliation，缺实际UI回执。格式/只读apply-check通过，源写与测试执行0；未启用UI/新clock或新Jenkins请求，本族验收继续open。

## 最新 Sequence 空轨道设计 r04

已在原Penpot文件新增1280×800、900×620、640×520三张可编辑空轨道Toolkit板，真实保存关闭重开rev498/4179对象。450新对象/39原生copies/246有效sourceRefs，原3728非Root对象、3main及Shared5保持。按既有RemoveTrack源语义显示1空binding/0轨道/0keys，0.5秒仍为本地编辑器游标。

修正新板窄宽属性和空态分配、旧key注解及15处Noto本地行框；27行名义框越界0，18原PNG已逐张打开、9对同阶段保存重开板内RGB差0。r01预写失败/r02截断/r03行框失败保留，未把SVG外溢group包围盒当画板。见 [具体设计与未完成项](animation-design.md#sequence-toolkit-空轨道设计-r04an-sq-v01v03v072026-10-25) 和 [新交接](animation-parent-handoff.md#新交付sequence-空轨道三尺寸-r04--rev4982026-10-25)。正式export/typed slot/Canvas consumer、真实clock/GPU/undo/focus、托管测试、EXE三宽/实际150%DPI仍open；本族passed/accepted=false。

## 最新Sequence曲线源修复r01

已修复同实体多binding时Session选轨与Curve投影查找不一致的问题，完整路径唯一匹配才投影，跨目标歧义明确诊断。仅2域内Rust文件，原7测试前缀保持、新增3项内存codec/history/undo回归；格式/补丁/guard通过，Rust执行0。见[具体实现与剩余接线](animation-design.md#sequence选轨曲线完整路径解析r01)。生产caller/正式导出、稳定key与typed target、真实clock/pose/GPU/正常EXE和144DPI仍open；未创建或扩大Jenkins身份，全族未通过验收。

## 最新BlendSpace 1D作者区状态rev483

已在原动画file/page完成三尺寸1D作者区，保存关闭fresh重开rev483/3729对象；原3350非Root和3main保持，新增24native copies/147设计源refs。真实源码fixture显示Idle、Scalar speed及0/1两样本，权重仍未求值、无subject，未声称真实预览。21 Shared属性行的readonly文字尚不符合正式typed value槽，4part已发布设计main也不等于现7node源闭包，交原public owner接Field/NumberField，不复制master。

[三board身份、投影与未完成项](animation-design.md#blendspace-1d作者区接续rev483)、[1280实际设计图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-blend-space-1d-15/blend-space-1d-1280-reopened-r05.png)、[检查及逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-blend-space-1d-15/validation.json)已更新。18张Penpot原图逐张打开，九对保存/重开板内RGB差0；640短Graph及84个文字框修正仅为设计证据。正式ZUI/稳定sample身份/真实参数权重与GPU/完整Toolkit输入/正常EXE/实际150%DPI/双端1DIP和样式产品闭环仍open。无新Cargo/Jenkins身份、服务或桌面操作；五个自有actor退出、释放0B，passed/accepted=false。

## AN-P04当前验证入口状态r04

限定palette叶包路线已由原管理owner正式交付：当前两Rust和四canonical源、c2806705胶囊12成员均匹配。支持独立plugin manifest/defaults off/features=[]/lib/显式test profile/唯一palette filter/locked/jobs1/threads1/Windows host/物理E；管理293checks及部署回执不代替Rust。旧nested receiver拒绝保持历史，当前待完整源/external/writer/工具/E资源与pool/cache准入，产品身份尚未登记、唯一测试执行0。见[精确路线与后续模板](animation-design.md#an-p04叶包固定路由实际交付r04)及[本轮消费](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/palette-leaf-route-delivery-r02.json)。无Cargo/query/capture/freeze/新identity，产品passed/accepted仍false。

## 最新Montage候选状态rev405

本域真实file/page在原rev383三个默认Toolkit板外新增四张900/640 Details、Sections打开候选，保存关闭fresh重开到rev405/3351对象。它们复用原3main与60native copies/324叶引用，原默认板与2658非Root对象、Shared数据均保持。抽屉宽438/308逻辑像素，来自真实Dock-to-Sidebar availableWidth的50%上限示例；不作为自动断点或替代UE默认布局。完整四值与三命令可见，但仍是readonly源码语义夹具。详见[具体设计与MT-D01～05](animation-design.md#montage侧栏抽屉候选rev405)。

24张原PNG逐张复核、12对可见画板RGB差0，只是Penpot持久性；r01/r02及ROI/字体测量失败保留。没有正式Montage模型/codec/真实clock/GPU、ZUI导出、Dock输入、正常EXE或实际150%DPI证据。全族passed/accepted=false；0Rust/插件功能执行、0服务/桌面/新Jenkins身份，释放0B。

## 最新Montage设计状态rev383

本域已在原file/page完成[Montage Toolkit三尺寸接续](animation-design.md#montage-toolkit三尺寸接续rev383)：fresh rev274→372→379→381（保留辅助检查失败）→383，2659对象、原3main；新501对象/45 native copies/243叶refs，原2157非Root对象与Shared rev5保持。原rev14空作者区、Sections和Sequence板全部保留。18张PNG逐图打开、九对保存fresh重开板内RGB差0；[最终1280图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-toolkit-13/toolkit-1280-reopened-r04.png)/[检查与未完成项](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-toolkit-13/source-validation.json)。

这次推进可编辑UE源码语义布局，段落/Slot/未解析segment/点与状态notify是明确夹具，不是已支持的Montage资产或真实预览。时间/ruler/gutter/文本clip均是设计投影；900/640侧栏操作/值/tabs及完整名称访问仍缺，Timeline外wrapper和PropertyRow值尚无正式源导出/typed绑定。当前catalog71项/三原登记与Shared34+3已实际存在；下列旧阶段的rev2/缺登记/旧palette结论保留历史，不能据此描述当前源。

本轮无Rust/ZUI/公共源/主件写入、无新请求/编译/桌面/服务操作，插件功能和Rust执行0；真实Montage后端/clock/GPU、正式codec/export/产品加载、正常EXE/原生证据/1DIP/实际150%DPI和样式往返仍open。产品passed/accepted=false、captureReady=false；实际释放0B。

## 范围、归属与当前状态

- 本会话：`01a16d51-741b-73b3-a61a-2189cd38f81e`；Parent：`01a16d4e-0491-7353-84fa-038bd7706546`。
- 用户授权的实际 checkout：`E:/Git/ZirconEngine`。只读参考：`dev/UnrealEngine`。不创建源码副本，不修改参考树，不接管角色/骨骼资产编辑。
- 用户此前答复“等待parent告知”；最新 [Parent 分工记录](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md) 已明确本聊天负责共享 timeline/curve/transport，材质聊天 `01a16d50-6c7f-7370-853d-4581184b66a1` 牵头节点图表面。公共 catalog、tokens、Starship、窗口框架及跨域集成继续归 Parent；具体共享消费合同见 [animation-shared-controls.md](animation-shared-controls.md)。
- Parent 已实际分配 `02 Animation and Timeline`：file `b4a1e3ce-9e6a-81fe-8008-d69d60a13f99`、page `b4a1e3ce-9e6a-81fe-8008-d69d60a13f9a`；[打开真实文件](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60a13f99&page-id=b4a1e3ce-9e6a-81fe-8008-d69d60a13f9a)。2026-10-24T10:33:07Z 的revn0/单页1对象和公共空文件是初始历史观察。rev146阶段的本域原生消费曾独立读回公共文件 `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` 为rev2/isShared=true/41对象/四原main/34颜色/3typographies；21个master引用ID字段均null，发布集合与默认源绑定分开。rev1/0样式观察保留历史，不能再作当前阻塞；[rev146消费与覆盖边界](animation-shared-controls.md#shared-rev2原生同步与覆盖修复rev146)。
- 已接续 [主工作台历史交接](main-workbench-handoff.md) 和最新 Parent readiness：原 Runtime11/P21 缺失终态保持原记录，不用新服务补造旧退出码。Parent 新运行时已就绪；本域可在分配文件内使用隔离无头浏览器/API，不启停共享服务、不操纵桌面或系统 DPI。尚未获得桌面时段。
- 已完成本轮注册入口及现有实现的源码盘点；下表是发现范围，不代表适用窗口已实现，也不涵盖尚未检查的 UE 第三方插件扩展。
- Shared同步阶段已保存关闭fresh重开到rev146/1411对象/原3main；[Sections源状态复核板](animation-design.md#montage-sections源状态复核rev140)复用9个Shared原生Button，九标题的text-content-text覆盖经重复原生full sync保留，14处中文font覆盖已恢复；未来full typography sync仍会覆盖该font，缺口保留。原控制板Button新增真实共享颜色/strong引用是消费者显式覆盖，不冒称master源继承；本轮未改公共库。原23本域copy保持，外链Shared共25个；Transport draft2、Timeline2、Curve1不变。此前[rev105源边界/flow修复](animation-design.md#transport源边界与流布局rev105)的15张原PNG及150图形矩形读回证据保留，640状态机Loop/树actions、值编辑器/长文本等差异仍open。历史revision不替代较晚证据。Parent公共Bool/Step出口已落实，Widget sample/finish候选已接但17+3项回归未运行，无真实clock/Stop/present。正式导出/公共发布、正常EXE/GPU、原生捕获/实际DPI、同状态双端diff及样式到产品闭环仍pending，产品passed/accepted=false。
- 当前[状态机实例尺寸接续](animation-design.md#状态机treerow实例几何接续rev148)已保存关闭重开到rev148/1411对象：八个原TreeRow copy内部painter依现tokens修正，根/组件UUID与Shared/三个专责main未变。三尺寸最终board重开区域像素相同，13张本轮图均已打开，仅是headless设计证据。长GraphRef auto-height实验未换行，已恢复四个原fixed框；640仍有Transport需180而左栏158、文字行框越界6.225 DIP两项约束。八copy继承主件部件ID、缓存240宽的painter metadata原样保留，正式sourceRef映射及guard/reflow/导出交原public owner，不据本次原生设计几何宣称可导出或业务能力。Eye/lock无状态机事件，SM-R01～08与EXE/真实DPI/全族验收仍open；未重查Jenkins/编译/source capture或新建请求。
- Parent原AN-P04两文件palette候选已写入实际checkout，Graph只登记Blend/Output，StateMachine保留GraphRef State；原schema/compiler/capability和已存在资产读取保持。新增关系回归、格式/差异静态检查已形成，Rust未执行；[精确源码与受管scope](animation-design.md#an-p04-两文件修复r02)。
- Montage已接续[源、事务与边界核查r01](animation-design.md#蒙太奇源与事务边界r01)：47份实源指纹、14项操作合同、31节点/29 control/15 events及实际安装payload已对账。菜单有有效工作区映射，但字段只写UI字符串，Preview/Apply只写未实现反馈；当前没有Montage文档kind、codec、作者事务或真实播放实例。本轮只改现有四份记录，不改原资产、公共Runtime或capability，不将这份核查算功能完成。
- 原Jenkins两笔r05 QueryOnly的exit2/pending/ordinary_entry_unproven/Current session authorization is pending保留，不重查。原owner18:31:35Z[只读收件对账](animation-parent-handoff.md#原两请求r05的owner收件对账接续)实见installed e0635d5d…/11pins匹配，同fc553228/current runtime1792836412672596900；15:00的2790及更早部署为历史。capabilities仍旧runtime1792418123320632400且无跨代映射/原交付，mutation runtime匹配却无实际准入；两笔server/source/job/native receipt仍null，不以自动context分支或服务恢复推定执行。三mutation精确回归仅静态存在，0执行；E低于35GiB reserve是前置观察而非原请求资源拒绝，captureReady=false/完整writer边界及资源时段未交付。现有full-tree enforcement=false原样单列，未改guard、runtime、DB或grant，不新建请求/attempt、不代提Cargo/capture，产品验收不升级。
- 产物清单：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/artifacts.json`；实际释放空间为 0 字节。

## 参考身份

`dev/UnrealEngine/Engine/Build/Build.version` 实际声明 Major=6、Minor=0、Patch=0、Changelist=0、BranchName=UE5，不能仅根据目录或分支名声称某个 UE5 发布版本。参考树没有独立 `.git`；在其中执行 `rev-parse` 得到的是外层 Zircon 仓库 HEAD，不能当作 UE commit。当前使用以下实际文件哈希固定本轮依据：

| 文件（相对 `dev/UnrealEngine/Engine/`） | SHA-256 |
| --- | --- |
| `Build/Build.version` | `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6` |
| `Source/Editor/AnimationEditor/Private/AnimationEditorMode.cpp` | `d9a11416cfd72f785def4e915b254189a42d225c961876773f05844777acffd3` |
| `Source/Editor/AnimationEditor/Private/AnimationEditor.cpp` | `bb0b327ae5c7bdc11dbe908cdbf743e2f67edf6e4458a9f794ac400d3047af63` |
| `Source/Editor/Persona/Private/PersonaModule.cpp` | `dab96ae0e7fc19a17f172660f1a1eb76339732e1ce789882e819523a56cac5bd` |
| `Source/Editor/Persona/Private/TabSpawners.cpp` | `3548621a5d9b03c1dac574a2c067766750ca2779244f1994deedafe15631ea44` |
| `Source/Editor/AnimationBlueprintEditor/Private/AnimationBlueprintEditorMode.cpp` | `ffe5f55166069e07df69a6939086abcd28dbae61424fe8ec0eef74f9bbeff730` |
| `Source/Editor/EditorStyle/Private/StarshipStyle.cpp` | `1c91e6767a76ba82505c16d815fb3530b622b59a8d13fc97c18662cba2abd2ca` |
| `Plugins/Animation/PoseSearch/Source/Editor/Private/PoseSearchDatabaseEditor.cpp` | `4c99550123125fb141ee78d6f1850886e3ae4e8d183b56a1b978657ffffd007a` |
| `Plugins/Animation/PoseSearch/Source/Editor/Private/PoseSearchInteractionAssetEditor.cpp` | `d90004dff4fc6167e60821bc1efa0ee0007307e0082335ced3aea550028c8562` |
| `Plugins/MovieScene/TemplateSequence/Source/TemplateSequenceEditor/Private/TemplateSequenceEditorToolkit.cpp` | `8cc39fc748e2c8c2f35964b592b004b26a3306f4783ff23272af2a9b77c9e943` |

以下 `UE:` 路径以 `dev/UnrealEngine/Engine/Source/Editor/` 为根。Plugins 路径单独写全。视觉与交互以 UE 源码为本任务目标；流式布局、稳定身份、i18n 和原生证据遵守 Zircon 合同。公共框架的 ReactBits 区域映射沿用 [App Shell](https://pro.reactbits.dev/docs/app-ui/app-shell)；属性及弹层沿用 [Forms](https://pro.reactbits.dev/docs/app-ui/forms) 与 [App Dialog](https://pro.reactbits.dev/docs/app-ui/app-dialog)，不从这些参考引入另一套外观。

## Toolkit、TabFactory 与关联窗口盘点

“适用”表示属于本轮设计和实现目标，不能据此推断后端已经具备。“待定”需要实际能力/owner 或产品范围决定。“本会话不适用”保留已有实现并交给相应窗口族。

| 编号 | 窗口或文档 | UE 实际入口 | Zircon 对应及分类理由 |
| --- | --- | --- | --- |
| AN-W01 | Animation Sequence | `UE:AnimationEditor/Private/AnimationEditor.cpp:465`；`UE:Persona/Private/PersonaModule.cpp:404`；`UE:Persona/Private/SSequenceEditor.cpp:18` | **适用**。UE 此处是骨骼动画；Zircon 已有的 Sequence 文档是实体/属性序列，不能当作同一功能。骨骼动画对应 `AnimationClipAsset`，其作者文档、轨道编辑与真实预览尚待接入。 |
| AN-W02 | Montage | 同一 Toolkit 的 `PersonaModule.cpp:435`；`UE:Persona/Private/SMontageEditor.cpp:39`；`UE:Persona/Private/SAnimMontagePanel.cpp` | **适用**。已有 `extension_montage_editor_workspace.zui`，31节点/15事件均仍是模板链；菜单能映射原工作区，Preview/Apply仅未实现反馈。真实资产、Slot、分段/跳转与撤销缺口见[14项边界](animation-design.md#蒙太奇源与事务边界r01)。 |
| AN-W03 | BlendSpace 1D/2D | `UE:Persona/Private/PersonaModule.cpp:465`；`UE:Persona/Private/SAnimationBlendSpace.cpp:35`；`UE:Persona/Private/SAnimationBlendSpaceGridWidget.cpp` | **适用**。已有嵌入状态机的 `AnimationStateKindAsset::BlendSpace1D/2D` 持久模型与编译/求权重链，样本引用 Graph。现成 UI 用静态 sample profile，Editor 创建状态仅支持 GraphRef；独立文档、真实 sample 编辑和参数/权重反馈尚未接通。 |
| AN-W04 | AnimGraph | `UE:AnimationBlueprintEditor/Private/AnimationBlueprintEditor.cpp:481`；`UE:AnimationBlueprintEditor/Private/AnimationBlueprintEditorMode.cpp:71` | **适用**。已有 Graph 文档、连接/参数事件及编译产品；editor capability 对 Clip/Additive/Mask 和 semantic compiler 仍标为不可用，需逐项查明和修复。 |
| AN-W05 | 动画状态机与转换规则 | `UE:AnimationBlueprintEditor/Private/AnimationStateNodes/SGraphNodeAnimState.cpp`；`SGraphNodeAnimTransition.cpp`；`StateMachineConnectionDrawingPolicy.cpp` | **适用**。已有 state/entry/transition/condition 文档命令和历史；真实节点、连接命中、规则跳转与运行状态高亮未验收。 |
| AN-W06 | Graph 内嵌 BlendSpace 文档 | `UE:AnimationBlueprintEditor/Private/BlendSpaceDocumentTabFactory.cpp:88` | **适用**。须共享同一资产身份、选择与预览上下文；不得复制一个独立假 BlendSpace 编辑器。 |
| AN-W07 | Sequencer 的资产绑定、属性轨道和事件 | `dev/UnrealEngine/Engine/Plugins/MovieScene/LevelSequenceEditor/Source/LevelSequenceEditor/Private/LevelSequenceEditorToolkit.cpp:178`；`UE:Sequencer/Private/SSequencer.cpp`；`Sequencer.cpp` | **适用的子集**。已有 `timeline_sequence` 插件与 Sequence 数据模型；现有 `extension_sequencer_workspace.zui` 示例 Camera/Audio 行不证明相应类型可执行。先核对真实 transform/property/event 轨道，再逐轨道开放。 |
| AN-W08 | Animation Curve Editor | `UE:AnimationEditor/Private/AnimationEditor.cpp:582`；`UE:Persona/Private/SAnimSequenceCurveEditor.cpp`；`UE:CurveAssetEditor/Private/CurveAssetEditor.cpp:171` | **适用**。已有选中 Sequence track 到 scalar/vector component curve 的投影；离散/Quaternion 不得伪装成 scalar curve。交互编辑及真实求值待接入。 |
| AN-W09 | Notify / NotifyState / Sync Marker 轨道 | `UE:Persona/Private/AnimSequenceTimelineCommands.cpp`；`UE:Persona/Private/SAnimNotifyPanel.cpp`；`UE:Persona/Private/AnimTimeline/AnimModel_AnimSequenceBase.cpp` | **适用**。Runtime timeline event-track 能力不等于 UE NotifyState/Sync Marker 生命周期；需核对 begin/end、跨段、循环、scrub 触发规则及历史。 |
| AN-W10 | Montage Sections 管理与预览跳转 | `UE:Persona/Private/SAnimMontageSectionsPanel.cpp:27`；`UE:AnimationEditor/Private/AnimationEditor.cpp:545` | **适用**。独立页签；当前/下一段、循环、预览全部/单段与 default/clear 是业务操作，不能用表格选中代替。 |
| AN-W11 | Asset Details / 选择 Details | `UE:AnimationEditor/Private/AnimationEditorMode.cpp:43`；`UE:Persona/Private/TabSpawners.cpp:737`、`:820` | **适用**。复用公共属性行和实际资产字段；资产字段与瞬时选择详情分开，修改走文档事务。 |
| AN-W12 | 实时 Preview Viewport / Preview Scene Settings | `UE:AnimationEditor/Private/AnimationEditorMode.cpp:35`；`UE:Persona/Private/TabSpawners.cpp:283`、`:680` | **适用**。本域负责播放时间与选中动画；角色域提供 skeleton/mesh/preview subject，主工作台提供原生视口承载；必须实际渲染。 |
| AN-W13 | Animation Asset Browser | `UE:Persona/Private/TabSpawners.cpp:231` | **适用**。过滤真实可兼容资产，打开时保留/更新 Toolkit 文档及预览；资产浏览公共组件归资源族/主工作台。 |
| AN-W14 | Curves 元数据 / Curve Debugger | `UE:Persona/Private/TabSpawners.cpp:179`、`:206` | **适用**。区分持久曲线元数据与 evaluator 调试值。调试值需要真实运行态；无运行态不得显示虚构权重。 |
| AN-W15 | Animation Attributes | `UE:Persona/Private/TabSpawners.cpp:849`；`UE:Persona/Private/SAnimAttributeView.cpp` | **待定**。需明确 Zircon attribute 类型与预览输出的对应；当前只确认 UE 注册入口。 |
| AN-W16 | 动画 Find/Replace | `UE:Persona/Private/SAnimAssetFindReplace.cpp:35`；`UE:AnimationEditor/Private/AnimationEditorMode.cpp:48` | **适用**。先实现受支持 track/curve/parameter 目标；跨资产事务、撤销与失败反馈仍须真实验证。 |
| AN-W17 | Anim Preview Editor / 参数预览 | `UE:Persona/Private/TabSpawners.cpp:522` | **适用**。参数必须传给真实 graph/state-machine 实例；仅修改属性文本不算预览。 |
| AN-W18 | Pose Watch | `UE:Persona/Private/TabSpawners.cpp:654`；`UE:Persona/Private/SPoseWatchManager.cpp` | **待定**。依赖 evaluator 的节点姿态观测能力及角色域的 pose 载体；不能用静态节点标记替代。 |
| AN-W19 | Asset Override Editor | `UE:Persona/Private/TabSpawners.cpp:632` | **待定**。需要确定 Zircon 是否支持 graph 继承及 clip override，避免引入 UE Blueprint 类继承假合同。 |
| AN-W20 | My Blueprint / Compiler Results / Find Results | `UE:AnimationBlueprintEditor/Private/AnimationBlueprintEditorMode.cpp:37`、`:123` | **适用的动画子集**。图/状态/参数导航和动画编译诊断适用；完整 Blueprint/K2 系统不由本域复制。 |
| AN-W21 | Aim Offset 1D/2D | `UE:Persona/Private/PersonaModule.cpp:470`、`:484` | **待定**。UI 与 BlendSpace 共用，但要先确认 additive pose 与 aim-offset evaluator；不因同一控件可渲染就开放。 |
| AN-W22 | Anim Composite | `UE:Persona/Private/PersonaModule.cpp:425`；`UE:Persona/Private/SAnimCompositeEditor.cpp` | **待定**。和 Montage 共用分段基础；当前 authoring asset enum 只有 Sequence/Graph/StateMachine，需确定复用 Montage 文档或新资产合同。 |
| AN-W23 | Anim Streamable | `UE:Persona/Private/PersonaModule.cpp:448`；`UE:Persona/Private/SAnimStreamableEditor.cpp` | **待定**。UE 专用流式动画资产；未发现对应 Zircon 编辑文档，不替换成普通 Sequence 冒充支持。 |
| AN-W24 | Pose Asset / Pose Library | `UE:Persona/Private/PersonaModule.cpp:458`；`UE:Persona/Private/SPoseEditor.cpp` | **待定/跨域**。角色域负责姿态资产；动画域仅消费选择和预览，最终编辑归属等 Parent。 |
| AN-W25 | Skeleton Tree / Skeleton Editor / Skeletal Mesh / Physics Asset / Retarget | `UE:AnimationEditor/Private/AnimationEditorMode.cpp:28`；`UE:Persona/Private/TabSpawners.cpp:372` | **本会话不适用（由角色域负责）**。动画窗口可挂入共享树实例和选中 subject，不复制资产编辑实现。 |
| AN-W26 | Skeleton Notifies 名称库 / Anim Slot Manager | `UE:Persona/Private/SSkeletonAnimNotifies.cpp:81`；`UE:Persona/Private/SSkeletonSlotNames.cpp:105` | **共享边界待定**。通知/Slot 的时间行为属于本域；骨骼上的名称与分组资产归角色域，需统一 ID/删除影响确认合同。 |
| AN-W27 | Control Rig / Modular Rig | `dev/UnrealEngine/Engine/Plugins/Animation/ControlRig/Source/ControlRigEditor/Private/Editor/ControlRigEditorMode.cpp`；角色域已核对当前及 legacy 模式 | **本会话不适用（角色域负责编辑）**。动画域仅接时间轴、预览实例和播放选择；复用角色域资产身份，不接管其原始 workspace。 |
| AN-W28 | Pose Search Database / Motion Matching | `dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Editor/Private/PoseSearchDatabaseEditor.cpp:246`、`:359` | **待定**。UE 实际注册 Viewport、Database Details、Preview Settings、Tree、Selection Details、Statistics、Data Details、Asset Browser 八个页签；Zircon 有固定数据的 `extension_motion_matching_workspace.zui`，本轮对 plugins/runtime animation 的搜索未找到 pose-search/motion-matching 后端，须确认能力与路线。 |
| AN-W29 | Pose Search Interaction Asset | `dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Editor/Private/PoseSearchInteractionAssetEditor.cpp:850`、`:884` | **待定**。UE 是独立资产 Toolkit，Details/Viewport 初始比例 0.4/0.6；不能由单人动画预览推断多角色 interaction 资产和同步能力。 |
| AN-W30 | Template Sequence | `dev/UnrealEngine/Engine/Plugins/MovieScene/TemplateSequence/Source/TemplateSequenceEditor/Private/TemplateSequenceEditorToolkit.cpp:65`、`:180` | **待定**。UE 复用 Sequencer 单一主页签，另有 template playback context；Zircon 当前 EntityPath/target_id 绑定不等于可复用模板绑定/重绑定合同。若支持应复用同一 Sequencer 编辑器。 |
| AN-W31 | Deformer Graph / MLDeformer / Cloth | `dev/UnrealEngine/Engine/Plugins/Animation/DeformerGraph/Source/OptimusEditor/Private/OptimusEditor.cpp:132`；`OptimusEditorTabSummoners.cpp` | **待定/交接变形与角色相关 owner**。UE 有独立 Graph/Explorer/Shader 等工作流；角色域已登记为待确认的变形/布料边界。动画域不因共用时间播放就新建求解、训练或变形作者系统。 |

## UE 布局与操作合同

### Sequence、Montage、BlendSpace

`AnimationEditorMode.cpp:51` 的布局名为 `Standalone_AnimationEditor_Layout_v1.5`。三列初始比例为 0.2 / 0.6 / 0.2；中列 Preview/Document 为 0.6 / 0.4；右列 Details/Asset Browser 为 0.6 / 0.4。比例是可停靠初始关系，不是硬编码屏幕坐标。左侧 Skeleton Tree/Asset Details 同栈；右上 Details/Advanced Preview 同栈；右下 Browser/Montage Sections/Curve Names/Slot Names 同栈。Document/Curve Editor/Find Replace 的默认 closed-tab 由实际打开操作激活。

`AnimationEditor.cpp:465` 复用动画文档页签，切换资产时更新预览 subject、关闭旧 Curve Editor；关闭动画文档也关闭其关联曲线页签。打开 Montage 时请求 Sections 页签，复用 Toolkit 时重新激活 Asset Browser；切回其他类型关闭 Montage Sections。Zircon 要保留这些上下文关系和返回焦点。

`SAnimTimeline.cpp` 将 outliner、track area、标尺与 transport 绑定同一 model/view-range；Display Frames/Seconds、次格式、snapping、轨道过滤和横向范围不得各有一个时钟。`SAnimTimelineTransportControls.cpp:19` 区分前/后播放、前/后单帧、首/末帧、录制和循环；现有 Workbench 的 Previous/Next 实际跳首/末，不能当作单帧操作。

Montage 的 section 边界、Slot、segment 和 notify 属于不同可选对象。`SAnimMontagePanel.cpp` 使用事务执行 section/slot/segment 修改，右键菜单与详情修改必须经过同一文档命令；`SAnimMontageSectionsPanel.cpp` 的下一段/循环和预览单段需要影响实际执行图。

BlendSpace 的采样点移动/添加/移除/替换、双击打开动画与预览参数由 `SAnimationBlendSpace.cpp` 回调。`SAnimationBlendSpaceGridWidget.cpp:1435` 的实际分支为鼠标悬停时按住 Ctrl 开始预览，Ctrl 释放停止；注释仍写 shift，实施以代码为准。Alt 切换高级预览 tooltip，PageUp/PageDown 导航，Escape 清除选点。`:2785` 绑定通用 Cut/Copy/Paste/Delete，paste 先检查资产/graph 各自剪贴板头；右键与键盘走同一选择和可执行条件。参数变化触发实际 runtime resample 和 pose 更新，采样权重由真实 evaluator 返回。

### AnimGraph、State Machine 与 Sequencer

`AnimationBlueprintEditorMode.cpp:83` 的布局名为 `Stanalone_AnimationBlueprintEditMode_Layout_v1.6`，三列 0.25 / 0.55 / 0.2，中列 graph/results 为 0.8 / 0.2。左上预览、左下 My Blueprint/Pose Watch；右上 Details/Advanced Preview/Overrides、右下 Anim Preview/Asset Browser。状态及过渡节点的形状、连线箭头和状态色来自实际 graph widgets 与 Starship；画布坐标仅用于图的空间语义。

`SequencerCommands.cpp` 明确规定 Space 播放切换、Shift+Space viewport 播放切换、K 暂停、左右箭头单帧、Up 起点、Ctrl+Up 终点、逗号/句号前后关键帧、方括号设置范围。各窗口必须按其实际命令表接入；文本编辑焦点优先消费文本键盘，不能全局劫持。保存/撤销由已聚焦文档 owner 处理，弹层关闭后恢复原控件身份。

## 原始资产与可复用组件

2026-10-24 静态读取 `zircon_editor/assets/ui/editor`：239 个 `.zui` 文档 TOML 解析成功，无重复 asset ID；下列七个根的直接导入 ID 均能在实际资产身份索引中解析。逻辑 `res://.../components/...` 与物理文件路径可能不同，不能仅用文件名推断导入失效或重写稳定 ID。

| 物理路径（相对 `zircon_editor/assets/ui/editor/`） | 稳定身份/现状 | 本轮 SHA-256 |
| --- | --- | --- |
| `host/animation_sequence_body.zui` | `AnimationSequencePaneBody`；6 节点、0 authored events；`animation_timeline_slot` | `14de402c8bad19cc963d9b068c35cc77bb6bbce5651400084ca92211307c2467` |
| `host/animation_graph_body.zui` | `AnimationGraphPaneBody`；6 节点、0 authored events；`animation_graph_canvas_slot` | `f0391dc745151f821eafc4b2fe1c2106fb283cec5f50bc9cfc882a0e20e67dab` |
| `animation/extension_montage_editor_workspace.zui` | `WorkbenchExtensionMontageEditorWorkspace`；31 节点、15 events；示例表内容 | `422445449969ae037746028e3a61d75825b366a2f856889f95831a31ecc35977` |
| `animation/extension_blend_space_workspace.zui` | `WorkbenchExtensionBlendSpaceWorkspace`；59 节点、7 events；静态 sample profile | `e5a058b9668c0dda05b878d2232599245f4db6e14421a81daa4955ab1776578b` |
| `animation/extension_sequencer_workspace.zui` | `WorkbenchExtensionSequencerWorkspace`；31 节点、15 events；示例轨道表 | `c0db37877318971fa08aaaffecbf99c5d1c9033c8758a7d6405c007a1fcf40d8` |
| `workbench/animation/transport_controls.zui` | `WorkbenchTransportControls`；8 节点、6 events；初盘无登记，现已在71项catalog中集成 | `eec15fa9d009c5a67b5ab2a0a6505e9882fe355f604c4287b6286a0711dcff7b` |
| `workbench/primitives/data/timeline_strip.zui` | `WorkbenchTimelineStrip`；Canvas，`input_interactive/clickable/focusable = false` | `75f2fa403f609e4f7815d489946642d703f02b617842666c69b9e11681c683fd` |

已登记 catalog 的重复控件：`WorkbenchPanelHeader`（title/actions slots）、`WorkbenchPropertyEditorRow`（value slot）、`WorkbenchPreviewViewport`、`WorkbenchSampleGrid`、`WorkbenchWeightHeatmap`、`WorkbenchTimelineStrip`、两个 Animation host body。`WorkbenchPreviewViewport` 当前根是静态 `Image`，默认 `animation_mannequin_viewport.svg`；不得计为实时预览。

初盘时这些组件的catalog登记尚未发现；当前public owner已集成`WorkbenchTransportControls`、`WorkbenchBlendSpaceDetails`、`WorkbenchSampleWeights`三项，catalog SHA9556e66d…/71项。动画扩展工作区登记另按具体身份核对，不扩大三项集成结论。组件库/canonical owner/正式导出仍未因此完成；本域保留原ID、binding namespace、route和slot，不重加18行或新建组件。

初盘共享依赖指纹（历史；当前catalog为9556e66d…/71项）：

| 文件 | SHA-256 |
| --- | --- |
| `zircon_editor/assets/ui/editor/components/catalog.toml` | `357e63b22d6203029c3a10980f2cf072e28ca584074d5f14863fd181f04d05cb` |
| `zircon_editor/assets/ui/editor/theme/editor_tokens.zui` | `dc23135b1e349773266e09cfb354c01bf047774ac0beb70a6a129e5ab4f66743` |
| `zircon_editor/assets/ui/editor/unreal/starship.zui` | `1e9140bd0b234271d13a575bf5c983c180f86bd395135afa84c261a2bc05f470` |

## 持久资产与运行态的实际边界

`zircon_runtime/src/core/framework/animation/asset/sequence.rs:71` 的 `AnimationSequenceAsset` 以 `EntityPath`/可选 `target_id` 绑定实体，各轨道指向 `ComponentPropertyPath`，并带 duration/FPS。它适合 Sequencer 的属性时间序列子集。`zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs:34` 会按资源 revision 和 World 身份重新编译并应用属性值；本轮尚未验证 Editor 的预览路径调用该流水线。

`zircon_runtime/src/core/framework/animation/asset/clip.rs:40` 的 `AnimationClipAsset` 才引用 skeleton，并包含逐骨骼 translation/rotation/scale 通道和点事件 `event_tracks`。`ui/host/animation_editor_sessions/lifecycle.rs` 的现有创建路由与 `core/editing/animation_document/{kind,asset}.rs` 只支持 Sequence/Graph/StateMachine；本轮未在这些作者入口找到 Clip/Montage/独立 BlendSpace 文档。Clip 已有运行时采样器，不等于有 UE Animation Sequence 的可编辑窗口。

`AnimationEventTrackAsset` 只有事件名、时间、payload 和可选 target_id。`clip_event_sampling.rs` 已有正反区间、loop、预算和续读游标合同；这些事实不证明 NotifyState 的 begin/end 或 Sync Marker 同步语义。`AnimationSequenceAsset` 本身没有事件轨道字段，channel value 只有 Bool/Integer/Scalar/Vec2/Vec3/Vec4/Quaternion；timeline 插件声明的 event-track capability 还需沿实际执行链核实，不能把 Clip 点事件或 UI 的 Event Cues 行当作 Sequencer 事件支持。

本域保留现有文档/历史和 Runtime 资产类型。后续用实际 Clip 资源身份建立作者入口，并与角色域共享 skeleton/mesh subject；不得把 Clip 字节按 Sequence 解码，也不得为“看起来同名”改写资源 kind。PreviewScene 的生产 backend、secondary session 和 GPU image 承载由 Parent 协调具体 owner；动画域负责当前文档 revision、时间、播放范围、混合参数和图/状态选择。关闭、切资产、编译失败 last-good、撤销后失效以及 GPU 资源就绪必须有同一生命周期与原生帧证据。

### 状态机与 BlendSpace 的新增核对

`asset/state_kind.rs` 已定义 Clip、GraphRef、SubMachine、BlendSpace1D/2D。1D 参数是 Scalar，2D 参数是一个 Vec2，样本的持久字段是 position + graph reference；没有独立 sample GUID 或 UE 风格两套轴 metadata。本轮不能把“缺独立 BlendSpace Toolkit”写成“引擎没有混合空间持久模型”。共享编译器校验空样本、非有限坐标、重复坐标及 2D 共线；Runtime 下沉为实际混合权重和 graph samples。Editor 的 `CreateState` 却只有 graph_locator，mutation 使用 `AnimationStateAsset::graph_ref`，没有对应的 state kind、sample、layer、exit/interruption 修改命令。

`AnimationStateTransitionAsset` 已持久化 duration_seconds、exit_time、interruption 和条件列表；Runtime `TransitionDesc` 把 exit_time 解释为归一时间并限制在 [0,1]。公共源编译器只拒绝负值/非有限值，因此大于1的已保存值会在 runtime 下沉时被夹取，这是需要显式设计/验证的差异。当前 Editor `CreateTransition.duration_frames` 固定按30 FPS换算；条件按 parameter 名称查找并覆盖，不支持同一参数的多个独立约束。更完整的 Runtime `All/Any/Not` 运行时表达式也不能推断为当前条件 Vec 的可保存规则图。

另有一处已定位的静态合同冲突：`ui/binding/animation/command.rs:88` → `binding_dispatch/animation/dispatch.rs:155` → `host/animation_editor_sessions/editing.rs:253` → `animation_document/mutation/state_machine.rs:99` 的 `SetTransitionCondition` 原样传入 operator/literal；mutation 的 `:131` 总写 `Some(value)`，而共享 compiler `state_machine/compile.rs:407` 要求 `Triggered` 不携带比较值，否则产生 `ZR-ANIM-COMP-STATE-008`。空 literal 在当前解析器中也无法生成无值条件。此项尚未动态复现、修复或运行回归；不是可通过的 trigger 编辑能力。 这条是修复前库存；当前最小 mutation 已修复并补三条回归，实际 Rust 执行仍为 0，见[后续修复记录](animation-parent-handoff.md#triggered-修复与-parent-最新接续)，不能继续将旧观察当作当前源码。

初盘时Graph plugin palette曾声明`blend_space_1d/2d`，但Graph持久enum/schema只有Clip/Blend/Additive/Mask/Output，Editor resolver未接受这两项。后继AN-P04两文件r02已让当前Graph palette只登记Blend/Output，状态机保留GraphRef State；未改Runtime/schema/compiler，实际Rust与GUI仍未执行。真实 BlendSpace 状态类型应沿现有 StateMachine 模型复用；不得仅把调色板存在当作 Graph 节点已可创建或运行。具体字段、公共提案和回归要求见设计合同。

## 已核实的能力边界与未完成项

共享空态控件已有Penpot工作区实图与保存重开回执；下面各业务窗口仍缺正式同case Penpot PNG、产品图、并排图和差异图。状态为open，几何/文本误差尚未测量，不以缺图推算误差为零。

| 差异编号 | 参考与实际差异 | 已证实原因/依据 | 依赖与下一项具体工作 |
| --- | --- | --- | --- |
| AN-G01 | UE 实时动画预览；Zircon 本轮只找到通用预览 trait 和静态 mannequin image | `ui/preview_scene/preview_scene.rs` 定义 `PreviewSceneBackend`；对 `zircon_editor/src` 的实现搜索只找到 tests 的 Fake backend；`animation_editor/capabilities.rs` 标记 runtime preview=false | 动画域+角色域+原生 viewport owner：以 secondary runtime session、真实 subject、pose/evaluator 输出和 rendered image 接通，不抹掉 capability 门槛。 |
| AN-G02 | 播放与循环未驱动实际动画 | `retained/dispatch/workbench/blend_space_transport.rs:65` 明确声明仅模板预览时钟；前后端点硬编码 0/3 秒 | 本聊天已负责共享transport；按 [域消费合同](animation-shared-controls.md) 接实际asset范围/单位、实例状态和backend clock，区分step与endpoint，不强制其它域使用动画FPS。 |
| AN-G03 | 混合参数与 sample 权重是固定资料 | `retained/dispatch/workbench/blend_space_selection.rs` 的 `SAMPLE_PROFILES`/`ASSET_PROFILES`；真实 BlendSpace 模型已嵌入 StateMachine，但当前创建命令仅 GraphRef | 复用 `asset/state_kind.rs` 的 position/graph/Scalar或Vec2字段和已有编译/evaluator；补状态类型、sample 修改与实际反馈。rev17已做源码三点fixture设计，发现样本Vec无稳定ID、轴范围/插值不属该持久模型，且`ui/sample_grid/generation.rs::format_tick`将有效小数标成0；具体差异/未应用公共提案见[设计记录](animation-design.md#blendspace作者区rev17)，不以设计证明运行。 |
| AN-G04 | Montage/Sequencer 轨道是示例表，缺少真实时间标尺与对象 | 两份 `extension_*_workspace.zui` 的 Intro/Combo/Camera/Audio 固定行；authoring asset enum及host loader仅 Sequence/Graph/StateMachine；Montage 15个binding可静态解析但无domain mutation/runtime | [Montage r01](animation-design.md#蒙太奇源与事务边界r01)已给具体事务/引用修复/循环/预览边界及候选路径。Parent确认规范资产/ID/codec及跨骨骼依赖后扩展原文档/history；Sequencer轨道集、实际播放与双端验收仍open。 |
| AN-G05 | Graph capability 与已有编译产品存在边界差异 | `animation_editor/capabilities.rs` 拒绝 Clip/Additive/Mask/semantic；`core/editing/animation_document/compilation.rs` 已保留 current/last-good product | 核对真实 compiler/evaluator 可执行拓扑，不仅改 available=true；补齐拒绝诊断、非法中间态与 last-good 行为。 |
| AN-G06 | Timeline strip 不接收输入，view foundation 尚未成为实际编辑通路 | `workbench/primitives/data/timeline_strip.zui` 三个 input flags=false；`TimelineModel`/`CurveModel` 是中立协议，对 Editor/plugins 的符号搜索未发现生产 adapter；foundation 方法在已查产品范围仅定义/测试调用 | 本聊天已负责共享timeline/curve；复用既有模型/变换/selection/inverse delta，不重复建框架；补实际surface、稳定key身份、Move/value/tangent/interpolation命令和反馈。 |
| AN-G07 | 布局、字体、状态、中文、DPI 及 Dock 生命周期未对齐 | 当前无本窗口族的正式双端证据；不能继承主工作台历史通过 | 完成真实 Penpot 及产品同 case 捕获后逐图登记；关键边界/文本行框目标 ≤1 DIP。 |
| AN-G08 | 专责main和跨文件公共实例已保存重开；公共样式、有效value组合、官方导出和产品加载仍缺 | rev8/17/31是历史，当前rev105/1358对象/原3mains、23本域copy和16 Shared linked copy；Shared rev1四主件已真实link/instance/override及Button reset。长文本及窄窗裁切负例保留；Timeline外层仍是组合草稿。现有 `--source-closure` 仍硬编码LevelEditor owner，native实例不证明ZUI codec | 依[当前源边界](animation-shared-controls.md#源边界与widget消费接续rev105)和[公共消费](animation-shared-controls.md#公共库四主件消费-r03)交Parent原UUID/draft/slot/event及逐图记录；公共owner接续value提供者、overflow与正式导出，不detach或复制main。 |
| AN-G09 | 两个原请求均无Rust执行证据 | 旧capabilities与新trigger-condition-01的r04 QueryOnly均exit2/pending/ordinary_entry_unproven；此前新请求已推进intake，曾先被多余Cargo字段拒绝、同身份更正后又遇source capture drift，均未起Cargo | 原owner/monitor先回capture-ready/ETA/参与路径，Parent再协调有界写稳定窗。保留原身份及所有失败；不全checkout停写、放宽drift守卫、本地Cargo或拿旧capability代验三条mutation回归。 |
| AN-G10 | 原生 EXE、实际 DPI、双 nonce 与本域捕获缺失 | 本轮未启动任何 Editor 或桌面输入/捕获进程 | 排队取得桌面使用权后从正常 EXE 启动；记录 EXE/DLL/资源闭包、进程 birth、nonce/generation/source/hold/release。 |
| AN-G11 | 现有 Sequence 作者文档与 UE 骨骼动画 Sequence 的资产语义不同 | `asset/sequence.rs` 是 entity/property bindings；`asset/clip.rs` 才是 skeleton/bone channels；当前作者 enum 没有 Clip | 本域接 Clip 作者入口与真实通道/点事件，角色域提供 subject；保持两种资源 kind、保存、历史和编译边界，不借同名窗口冒充支持。 |
| AN-G12 | NotifyState、Sync Marker 与 Sequencer Event Cues 未取得执行证据 | Clip 已有点事件采样合同；Sequence asset 没有 event 字段；timeline event capability 只确认声明 | 明确实际可执行事件源、scrub/逆向/loop/跳段规则与撤销；缺少的持久化/生命周期合同作为具体依赖，不先开放无效果轨道。 |
| AN-G13 | `triggered` 条件原来总写比较值；私有修复已落地，执行/界面验收仍缺 | compiler要求None，否则STATE-008；mutation已按operator生成None trigger/合法Some comparison，未放宽compiler | 新增源码/编译/二进制/历史回归覆盖比较↔trigger、无效值拒绝、no-op、current/last-good和undo/redo；rustfmt通过，Jenkins执行及真实UI链仍pending，不关闭整项差异。 |
| AN-G14 | 状态机 Details/规则编辑覆盖不足，单位和选择身份尚未对齐 | CreateState仅GraphRef；CreateTransition固定30 FPS且按端点对查找；无 exit/interruption/layer/priority 编辑；条件按参数覆盖；source exit_time>1 可下沉夹取 | 复用现有持久字段与文档历史，先确定秒/归一/帧单位、转换排序、多条件身份；公共 binding/注册变更向 Parent 交接。UE更完整的规则图/事件/自定义blend保留差异。 |
| AN-G15 | 原Graph palette不支持类型已按AN-P04收敛，生产创建/身份接入及完整BlendSpace工具仍缺 | 两插件源r02现仅Blend/Output，唯一palette Rust回归执行0；限定nested receiver已按固定leaf recipe交付，完整source/resource/pool准入仍缺；本轮私有创建命令另拒绝命名output与sink新冲突，旧源合法同名不自动迁移 | [AN-P04-ID01](animation-design.md#graph输出节点身份创建保护an-p04-id01-r01)四项新回归执行0；旧资产typed node/sink寻址、UE默认Root/NodeGuid、真实palette点击/gesture/undo链待接，不能以删除不支持条目或只翻标志称工具完成。 |

## Penpot 设计与产品接入的交接合同

具体flow树、原始控制身份、数据/事务职责和失败反馈见 [动画设计接入合同](animation-design.md)。共享空态设计已进入真实Penpot，业务窗口、官方导出及产品接入仍按各自证据单独推进。

在已经分配的真实文件中使用实际可用的library/component实例和共享tokens，建立Sequence、Montage、BlendSpace、AnimGraph、StateMachine、适用Sequencer与Curve页面/board；本域已先落地专责共享控件的空态组件并交Parent集成公共库。保留UE目标区域；缺后端的区块保留为未实施设计，不能当作产品完成。共享组件的file/page/board/component/instance身份已登记，业务窗口及跨文件样式/实例身份仍待实际落地。

普通区域使用 HorizontalBox/VerticalBox、stretch/min/max 和一个长列表滚动 owner。真正的 timeline lane、curve graph、BlendSpace 采样平面和 node graph 可使用各自数据坐标系；不要用画布排列普通工具栏、标签或详情。Panel title 起始对齐，时间/数值列末端对齐，独立 transport 图标按钮按共享度量居中。宽度不足时复用公共 Dock/tab 折叠机制，保留中心编辑表面和返回路径；不直接丢失侧栏操作，也不为截图复制第二套布局。

原文件导出前后核对 asset ID、node/control ID、import/component identity、binding、event、slot 和源 hash。产品加载需出具相关依赖闭包身份，不能只比较根 `.zui`。一次真实 Penpot 共享样式或本域受支持局部样式修改，必须完整经过保存、关闭、fresh 认证重开、官方 ZUI 导出、原资产应用、产品加载与截图变化；数值由实际组件可导出的属性决定，避免未映射属性绕过 guard。

## 验证矩阵与验收

复用 `tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-review-contract.ts` 的 `LayoutReviewCase`，保留其 id/sourcePath/host/viewport/dpi/locale/state/data；真实产品输入使用对应 presentation/source identity，不向 case 填造“已就绪”。复用主工作台 `tools/analysis/visual/record-editor-normal-window-evidence.py` 及严格 normal-frame evidence 协议，不另造简化验收。

| 覆盖 | 必须取得的事实 | 本轮状态 |
| --- | --- | --- |
| 1280×800、900×620、640×520，100% | 相同逻辑 client、内容、字体、state、资源 closure 的双端图和逐图复核 | pending |
| 1280×800，实际 150% | 窗口本身的 DPI/scale=1.5，物理 client 1920×1200；资源及业务状态匹配 | pending |
| 默认/空数据/选中/焦点 | 空数据由真实空文档和资源状态得到；焦点有稳定 control identity | pending |
| 菜单/右键/弹层及焦点返回 | 正常输入触发、可用性、命令执行及关闭后的焦点归还 | pending |
| 滚动首尾/中文/长文本 | 同一个滚动 owner 的实际前后位置、文本行框/裁剪/overflow | pending |
| Sequence/curve | 真 track/key/curve 选择、scrub、单帧、range、snap、插值、保存与撤销/重做 | pending |
| Montage | section/segment/Slot 修改、重排、next/loop/jump、notify 生命周期及撤销 | pending |
| BlendSpace | 参数与点拖动、三角剖分/权重、无效样本反馈、双击动画、真实 pose 改变 | pending |
| Graph/state-machine | 节点/边/参数与转换编辑、规则跳转、编译失败/last-good、运行态、撤销/重做 | pending |
| Sequencer | 真实资产绑定、可执行轨道类型、关键帧、播放/范围/事件、跳转与保存 | pending |

资源就绪且原生捕获协议有效才可写 passed；双端同 case 比较和逐图复核通过才可写 accepted。每图记录原图、side-by-side、difference、关键边界/文本行框的 DIP 误差及未还原项。当前所有窗口族状态保持未验收。

Jenkins 恢复后的首个基线检查仅选择 `zircon_editor` library、关闭 default features、显式空 features、test profile，过滤 `ui::animation_editor::capabilities::tests::`；覆盖当前对不可用能力的真实拒绝合同。后续按实际变更增加 Sequence/Graph/StateMachine 文档与历史测试及插件的相应过滤器，不运行全仓构建。每次真正提交 compilation-capable 命令前重新向用户声明包、features、target、filter、profile 和命令。物理 buildRoot 显式为 `E:/cargo-targets`，不继承当前部署规格的 D 盘默认值。

本轮已经按上述范围声明并提交一次 `tools/dev/dev-module-build.ps1 -Package zircon_editor -Action test -Features @() -LibTests -TestFilter 'ui::animation_editor::capabilities::tests::' -Jobs 1 -TestThreads 1 -JenkinsIdentityFile 'E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/jenkins-capabilities.request.json'`。原 request/operation ID 均为 `ue-animation-20261024-capabilities-01`，selected driver 为 `fc55322804fb53ab3e39f996c848ef5e5f3907497d3f38167b7095244b2a7399`，runtime 为 `start-1792418123320632400`。入口退出 1，返回 `status=pending`、`reasonCode=ordinary_entry_unproven`、WinError 10061 连接拒绝；`formalAcceptance=false`。没有发生 Cargo 编译或测试执行，不计两项回归为通过。恢复时先核对原请求身份/状态，不用新 ID 覆盖这次 pending。

另已声明并实际提交新私有修复回归：`zircon_editor` lib/default features off/features空/test profile，过滤 `core::editing::animation_document::tests::state_machine_conditions::`，由同一 `dev-module-build.ps1` 执行，request/operation `ue-animation-20261024-trigger-condition-01`。首个 `jenkins-trigger-condition.request.json` 带当前Cargo schema不支持的ownerThreadId/allowCommit而在编译前拒绝；原文件和拒绝保留，随后同ID的 `jenkins-trigger-condition.cargo-input.json` 仅移除这两个字段。入口进入正式只读source capture（主树52122文件、外部zr_vm8011文件及嵌套依赖），两次捕获间其它域计划文档变动，退出1/pending：`Source changed during pre-submit capture`。本域七个canonical文件在捕获期间没有再编辑，未发生Cargo/test执行；具体可见变动和原请求记录在任务产物清单。Parent已收到 `01a16da9-54fc-7672-92fe-f70f348e4ac1`，等待稳定准备时段或原入口owner处理，未重复提交。

本轮静态格式和七路径change-guard通过只证明该补丁的格式/静态边界；Penpot实际保存重开由独立回执证明。两者均不证明Rust测试、真实动画、原始ZUI回写或双端视觉验收。


### 2026-10-25 active 接续：Curve RT-S25 拒绝状态与作者源边界

原动画file/page保持不变，实际已从rev570推进到rev589/5889对象，原三专责main未改未发布。完成显式default、mixed interpolation各三尺寸的六个原Curve实例，150refs有效；12caption使用Shared5实际secondary color引用、保留NotoSC。本次不新建Timeline/Transport/clock或资源adapter。r01行框重叠实际失败保留；r02仅6text leaf的1.5行高覆盖修正，保存关闭重开稳定、12原图逐张复核、六对未缩放画板RGB差0。详见 [实际设计/逐图与限制](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-curve-rejection-20/summary.md)。这是owner表面设计，正式ZUI/host/正常EXE/150%DPI和消费者行为验收仍open。

RT-S25在既有shared-controls原位新增实际资源r04 prepare_edit合同，整批expected/candidate/inverse与UnsupportedSurface保持，expected kind来自真实语义准入；codec候选不取得Document/history/save权。Sequence非法FPS偷偷fallback30的最小diff/3回归候选已准备，未应用/未执行，不并旧请求。

原Bool `ue-animation-sequence-bool-insert-20261025-01 / validate-sequence-bool-insert-r02` 的queue17016→management397现管理build结果FAILURE；原console240次状态均waiting，末行资源准入pending，没有Cargo/test/native执行行可核实。原source/input/失败保留且未重投；已向原Jenkins owner排队单次新失败通知，排队ACK不代表已读或准入，native/readguard释放仍待真实证据，verified Rust=0。原palette、capabilities、Triggered、Curve与Runtime Bool scope保持分开。goal active，整体passed/accepted=false；本轮无编译、服务或桌面操作，释放0B。


### 2026-10-25 Sequence FPS 作者修复实际后继 r02

已把非法FPS fallback30修复从未应用候选落实到原两私有路径：AddKey/RemoveKey先按当前compiler同一finite/>EPSILON边界准入，错误不查改track，不静默换算作者时间。mutation源当前f6c8adb0…，新回归leaf cbf0bc0e…；原Bool/精确时间/binding修复、原Document/history/CAS保留，Runtime/compiler/host/Session显示fallback/clock/schema不变。三必要测试已接literal cfg(test)，rustfmt/diff/reverse-check/精确guard通过，Rust执行0。 [实际应用与待验收](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-frame-admission-r01/summary-r02.md)。

原palette queue15804→management385 FAILURE已据109B原服务器archive核实；其readonly config误拒已由管理owner修复发布，管理通过不改产品原失败。原单recipe不存在可用同键resume/reconstruct接口，不retry/造替代ID，不要求用户grant。原Bool397/其旧source、palette原identity/delivery及新FPS源后继分时保留；本轮无新的产品request、QueryOnly、Cargo、服务/桌面/Penpot动作。goal active，原受管Rust/正式导出/真实后端/正常EXE/150%DPI及全族passed/accepted仍open/false。登记释放0B，失败和交付证据保留。


### Sequence FPS 拒绝与预览投影接续（r03，2026-10-25）

本轮在原owner路径实际落实公开timeline_foundation及内部project_sequence_timeline的非法FPS拒绝；不再将显示fallback的30FPS换算成可交给PreviewPlayback的源秒值。有效FPS、源key秒值、UI帧选择及原Document/CAS/history保留；这是foundation投影准入，不是生产clock/播放器或GPU接通。原mutation f6c8adb0…及三项frame_admission回归cbf0bc0e…未再修改。

同步修正上一轮遗漏的旧invalid-FPS成功断言，并新增同一文档经原CAS/history撤销重做后重检源revision的回归。当前8项Sequence frame回归仅定义、0执行；3路径静态检查通过，normal EXE/实际clock/GPU/150%DPI仍缺。精确源/差异/限制见[summary-r03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-frame-admission-r01/summary-r03.md)和[test scope](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-frame-admission-r01/test-scope-r03.json)。原palette终态/修复与旧请求保持，不创建新验证identity。


- 2026-10-25 phase21：Sequence900/640各Details/Asset Browser可编辑drawer候选完成真实保存关闭fresh重开rev616；原1280/default与5888旧对象/3main保持。52copies/328refs，10原图及5对内部RGB0差异只证明Penpot设计持久。640左tab/center遮挡、实际输入/asset/preview/官方导出/EXE/150%DPI仍open；[逐项证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-sequence-drawers-21/summary.md)，passed/accepted=false。


- 2026-10-25：资源RT-S25-KEYS01整批结构候选已实源消费；旧Delta与codec专用insert/remove分开，真实资产/Host/undo/save仍open。动画私有倍率stored-change回执最小修复及3回归已落地，最终静态0/0/0，Rust0；[patch/范围/失败保留](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-playback-ack-r01/summary.md)。无新Jenkins身份/服务/桌面，族未通过。


### 状态转换 duration 精确事务接续（2026-10-25）

已在原 mutation/state_machine.rs 修正既存转换 duration 的变更回执：原 f32 EPSILON 比较会将合法极小秒值改0、符号零或相邻 finite 值写入 clone 后报false，store.prepare_mutation据此丢弃候选。现只比较实际存储位值，旧frame→30FPS秒换算及compiler有限非负约束保留；UE AnimStateTransitionNode.h:38-39 的 CrossfadeDuration 是 seconds浮点量，保留旧frame合同不代表该时间差异已对齐。前像2b67e8d9…为原条件修复后像，现e0afe5aa…是本批后继；set_transition_condition全文和原state_machine_conditions.rs逐字保持，不重套Triggered或把旧条件请求当本批验收。

新增私有 state_machine_duration.rs 三项回归覆盖完整source/codec、Document prepare不写源、单History/CAS事务的dirty/undo/redo/当前编译与last-good、真实no-op与缺目标清洁性。补丁07a88aac…，fmt/diff/reverse/guard全0且guard0/0/0，26保护源保持；Rust执行0。精确[后像/保护检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/state-machine-duration-r01/applied.json)、[静态检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/state-machine-duration-r01/static-validation.json)、[必要聚焦范围](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/state-machine-duration-r01/test-scope.json)保留。未创建或提交新的验证身份；正在等待的播放回执queue18979/#408仍只有其原3测试filter及immutable输入，本批是后继checkout源，不借其结果。

另已确认非Bool AddKey取末键值：现有Linear(0s,1)/(1s,9)在0.5s的末值9与真实采样点值5不同，UE SequencerChannelTraits.h:198-206 EvaluateAndAddKey会先求值。已有raw/compiled sampler仍private，公开纯查询仅Bool。编译产物只读出口可复用sample_compiled，但必须关联当前document revision/唯一target，不能拿last_good当当前源，也不能以单点采样声称Hermite切分或极短interval形状保留；无效中间文档仍须可编辑。具体[原调用点/公开出口边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/state-machine-duration-r01/source-followup.json)交Parent，Runtime/公共host/原6events/共享3main及RT-S25 codec未改。

本批没有Penpot/服务/桌面或新编译动作；正式导出、真实clock/GPU/EXE/实际150%DPI与完整窗口族验收继续open，passed/accepted=false，goal active，释放0B。

### Curve 原请求终态与 compiled-track 只读出口（2026-10-25）

原 curve-lossless queue16509 已精确对上 management #393 FAILURE，109B archive 的拒绝为 `.cargo/config.toml=null` 只读依赖在自动 context 前触发 control-path 检查。两源+十依赖12/12匹配，原 filter 静态8项（7普通+1ignored），Rust发现/执行未知、verified0。旧 pending/输入/marker/失败全部保留，不删依赖或重投；palette窄例外不覆盖本recipe。[本域终态消费](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/curve-lossless-admission-r01/original-terminal-consumption-r01.json)。

已形成公共Runtime四路径**未应用**的 `sample_compiled_animation_track` 精确diff，复用原sample_compiled，原Bool出口/Step边界/private traits不变；rustfmt解析与git apply --check通过，两回归定义未执行，源码未写。当前非Bool AddKey仍取末键，当前revision/唯一真实target、一次Document/CAS/history与invalid中间态策略须接入；last-good不能充当前源，单点值不保证Hermite切分形状。[补丁及范围](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/compiled-track-query-r01/summary.md)、[consumer准入](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/compiled-track-query-r01/consumer-boundary.json)。RT-S25不引入Timeline/FPS/clock；公共整合交Parent。无新身份/编译/query/capture/服务/桌面动作，goal active，真实EXE/DPI/完整族验收open，passed/accepted=false，释放0B。

### Sequence 跨实体重绑的非拥有空 binding 隔离（2026-10-25）

已在原作者 `mutation/sequence.rs::rebind_track` 修复新发现的数据隔离错误：合法移动唯一源track时，原retain_mut会顺带删除同entity下未拥有该track的空binding及其target身份。现仅实际移走track的binding依原规则判断是否清退，其他空binding/target_id/原顺序完整保留。源compiler允许空tracks和同entity多binding；UE MovieScene操作也按明确binding对象/GUID定位。原同实体重绑、目标选择、缺源/冲突/歧义、旧时间与Bool修复及源owner最后一轨清退规则保持，未宣称解决typed目标选择或骨骼Montage。

新私有两回归定义覆盖12组直接重绑（owner target Some/None、有无余轨、目标新建/既有Some/None）和6组原Document/store prepare→一次command/history→codec重读→undo/redo，连同完整target/顺序/compile/dirty/no-op。23项保护与原sequence_rebind.rs逐字保留；rustfmt/diff/reverse及guard均0，guard 0/0/0；Rust类型检查/执行0。真实wrapper DryRun已0退出，仅证明root zircon_editor/lib/defaults-off/features[]/locked/filter sequence_rebind_isolation_/jobs1/threads1配方文字；metadata dev/无--profile/Cargo默认test分列。该slice向既有受管milestone lane交独立scope，未创建新request/attempt/identity，不扩旧请求。[精确补丁与验证边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-rebind-isolation-r01/summary.md)、[有限测试scope](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-rebind-isolation-r01/test-scope.json)。

另原播放回执 queue18979/#408于14:44:16.363Z成为Pipeline FAILURE，console终句为Resource admission remains pending；最后archive仍waiting/checkout_source_busy，不能当Rust失败/通过或native EOF/释放证明。[本次真实终态GET](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-playback-ack-r01/validation-server-408-terminal-r01.json)。原identity/input/marker与等待历史均保留，不重投/扩大filter。本次新私有源是该旧完整checkout观察的后继，不冒充旧输入已覆盖。公共sampler四源/未应用proposal、6events/3main/主题/原ZUI均未改；无新Cargo/capture/Penpot/桌面/服务，真实预览/导出/EXE/150%DPI/全族验收open，goal active，passed/accepted=false，释放0B。


### Sequence path-only 选择准入 r01（2026-10-25）

实际 Session `select_timeline_span` 先核当前完整 entity/property 路径的唯一真实轨道；缺轨 false、多 binding 同路径明确拒绝且保留旧选择及 transport。`reconcile_source_change` 在原 focused Document undo/redo 后清理缺失/歧义选择，源恢复不复活已退休的 transient selection。原 host `editing.rs/history.rs` 与 `pane_presentation.selection_summary` 链路保留；未造 target ID、绑定位置 identity、第二历史或 clock，正式 typed binding/revision 接线仍 pending。

新增3项回归定义，原 Curve 歧义回归强化入口拒绝且保留旧选择的投影拒绝；Rust执行0。最终 fmt/diff/reverse-check/guard通过（0错误/0警告/0抑制），10保护依赖不变。原 playback setter 与测试字节保持，但 Session 文件为后继，旧 filter 不覆盖本切片；原 palette/Curve/playback 身份及失败不改、不重投，无新 Cargo/Query/POST/capture/freeze/桌面/服务。Penpot rev616/phase21 原图为历史，非本修复 EXE 证据；正式导出/typed target/实际窗口/144DPI及全族验收继续open，goal active，passed/accepted=false，释放0B。[源差异与检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-selection-admission-r01/summary.md)。


### Montage Sections 三尺寸双状态原生流式板（2026-10-25，phase22）

原动画file/page现rev642/6831对象，新增1280×800、900×620、640×520的Next=None/自环六板，均从原53对象Sections板原生复制并保留引用。原6512个非根对象、原3main、Shared rev5未改；新增54 native heads/108 refs，36 heads可见，隐藏比较状态不算新消费。命令/链使用原生流式布局和换行；Preview All修正保留。

14张原工作区PNG已逐图打开。r02六对画板内RGB差0仍发现1280 None缓存箭头：r03仅重建该叶positionData，content/盒子/shapeRef不改，保存关闭fresh重开后为Next方块；其余五板子树不变，明确复用rev641图。r03最终对有194像素差≤2 RGB色阶，零差断言失败保留。最终87个可见文本记录行框在自身/父盒内，7/7限定源pin匹配；不是UE/EXE 1DIP或native DPI验证。r01辅助脚本失败及同身份续作过程保留。

这六板是Sections源夹具；真实Montage模型/clock/authoring/history、精确FontAwesome/Combo/menu输入与焦点、滚动长名、官方ZUI/style/export、完整Toolkit/normal EXE/GPU/150%DPI仍open（MT-S01..07）。未改原6events/业务ZUI/public源，未新建main/clock/store/producer。manifest readiness不算插件测试，无新Cargo/Jenkins身份/query/replay、桌面或服务动作。3个自有browser已退出，释放0B，passed/accepted=false，goal active。[实际产物与差异](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-sections-responsive-22/summary.md)。


### Sequence 新建轨道 binding 准入 r01（2026-10-25）

已修复原 CreateTrack 在缺失属性时按同entity第一个binding插入的问题。现写源前只接受零或唯一候选，多binding明确InvalidTrackPath；现有唯一完整轨道仍false/no-op，重复轨道沿原规则拒绝。原target_id完整十进制字符串、空binding和源顺序均保留，零候选新建None/唯一候选复用原target。没有增加typed DTO、迁移旧源或修改compiler/capability/Runtime/公共host/history/clock。UE UMovieScene::AddTrack:1173以明确ObjectGuid查FindBinding后才Modify；本修复不等同typed选择已交付。

改动只在原mutation/sequence.rs、原tests/sequence_rebind.rs及新增tests/sequence_create_admission.rs。旧lookup回归的已有路径no-op/Document/history断言保留，错误的首binding插入期待改为拒绝并保完整source/codec。新3定义覆盖16直接拒绝、16原store.prepare拒绝及5个合法prepare→一次原command/history→codec→undo/redo循环，校验revision/compile last-good/dirty/undo及完整target/顺序；未来同私有module过滤器静态37定义，Rust发现/执行未证明（verified0）。最终fmt/diff/reverse/guard通过，guard 0错误/0警告/0抑制，29保护路径未变；首次新文件格式失败及修正保留。

现production codec→dispatch→host→store链保持，prepare错误在开启transaction前传回，不把它报Created成功；这是实源审阅而非新GUI/host-lifecycle验证。实际typed binding/document/view/revision交接、官方源导出/真实GPU/normal EXE/144DPI/双端验收继续open。根zircon_editor/lib/defaults-off/features[]/locked/单jobs单threads的37定义候选交原Jenkins milestone lane；未调用wrapper/DryRun/Cargo、未创建identity/query/重投或扩旧请求。原palette#385、curve#393、playback#408及其旧输入/markers/失败完整保留。goal active，passed/accepted=false，释放0B。[增量补丁与精确验证边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/sequence-create-admission-r01/summary.md)。


### Graph 默认参数作者位事务 r01（2026-10-25）

已在既有Graph mutation落实浮点作者位变化回执，保留原Document/history/CAS与全部legacy节点保护；3新回归静态定义、fmt/diff/reverse/guard通过，Rust0。原 r02/#416 只验证原5项 key-exact，不借其身份验收本片；无新Cargo/提交/桌面/服务。typed目标、正式导出与真实EXE/DPI仍open，goal active，passed/accepted=false，释放0B。[实际两源与验证范围](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/graph-parameter-bits-r01/summary.md)。


### Montage 时间轴独立页三尺寸与行框实测（2026-10-25，phase23）

原文件当前已实际保存关闭fresh重开rev661/7067；新增3独立Timeline板/236对象，6 owner copies/141refs，原3main/6830非根/Shared5不改。最终三对RGB0，57行框最大溢出0.00634765625px，字体/文字保持；14原图+3最终diff逐张打开。夹具不代表Montage模型/clock，默认Toolkit、正式ZUI/normal EXE/150%DPI/完整交互与产品验收open，passed/accepted=false，goal active。原#416实际FAILURE/Resource admission remains pending，Rust/native EOF/资源未知，不重投。详[交付与失败边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-montage-timeline-focus-23/summary.md)；4客户端退出，释放0B。
