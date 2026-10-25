# 08 资源工具：共享组件消费者合同

## 最新接续：嵌套Search偏移由真实原生同步producer复现

原105版中900/640的Search field与各自head对齐；在真实部署的Penpot中，仅观察过滤至StaticMeshSourceContent的原生sync-file WatchEvent输出，生成24项change，逐项完整字段与实际106保存请求中的24项一致，其中12项为嵌套Search后代。两个field X分别增加280和471.57894736842127；纯native reflow的4个对照不引入该偏移。[精确比较](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/nested-search-reflow-01/propagation-comparison.json)将原生同步producer确定为复现入口，具体root rebasing修复仍由原public owner/Parent处理，见[设计37](08-resource-tools-design.md#37-嵌套search偏移的原生同步producer复现)。

本轮只用原105/107不可变真实输入收集生成事件，未dispatch、执行commit/layout事件或保存；08仍107/2002对象/6业务main，Shared5及registry保持。16项静态检查、原42产品/参考pins不变，6个自有Chrome均原生exit0；4次入口/编码失败保留。本域不改copy子图形坐标/宽度/touched或refs，不改Shared/插件/主题。原240/204 stretch、native resize→sync→reopen实际修复、official export/load/provider、Rust/EXE/144DPI/双端≤1px均open，passed/accepted=false。本轮无新增图，复用107原18张已打开证据；2项0B临时目录待原Jenkins清理，释放0B/cleanup未关闭。

## 历史接续：独立Curve无损字段合同及原生嵌套构造核对

本轮原08 file/page仍为107/2002objects，0设计保存；只读API核当前原件、Shared5与动画原文件rev274。动画Curve Surface原main真实存在，但文件isShared=false，未复制/发布或在08仿造曲线表面。新[RT-S25具体合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-source-review-01/curve-target-contract.json)补齐逐key插值/切线权重、默认未设/外推、RGBA/七个颜色调整字段、真实typed payload与宿主保存边界，见[设计36](08-resource-tools-design.md#36-独立curve字段保真与原生嵌套构造核对)。现有DataAsset可作经真实域codec验证的载体候选，format/JSON本身不作语义准入；Browser当前按ResourceKind返回类型，仍需Parent精确subtype/registry接口，不新增schema/index/store/公共runtime写入。

同一真实headless工作区调用纯make-component-instance，53个未持久化生成部件与现有两business copy/原Shared nested copy逐part近引用匹配，0事件/0保存/0registry变化。900/640 field-root偏移与左侧缩窄量仅在浮点减法舍入范围内相同，内部field X精确等于未缩放原生copy位置；这是查因范围，不是布局/同步根因证明或已修复。[构造比对](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-source-review-01/constructor-comparison.json)保留全部字段差异、原native导出及失败。15静态检查/42产品pins保持，12新增验收定义0执行；原18图证据沿107复用，本轮无新增截图/Engine/DPI/官方导出。全族passed/accepted=false。

## 历史接续：Static Mesh真实源内容保存重开107

原08 file/page现rev107/2002对象、6业务main、1200有效sourceRefs。新增StaticMeshSourceContent和三尺寸业务main/copies，复用原Shared Search的3份嵌套linked copies；读取真实crate_wooden.glb头及JSON，未导入、解码或创建Engine UUID。原1920非根对象及5组件记录精确保持，42产品/参考输入同hash。[设计35](08-resource-tools-design.md#35-static-mesh源内容与原生三尺寸保存重开107)、[18项源检查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/source-checks.json)、[重开640预览](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/reopened-640x520-crop.png)记录边界。

三次保存完整ACK对应105/106/107，独立关闭重开107同hash、0额外保存/registry不变；18张原PNG均实际打开。640材质标题由native Hug修复，900/640嵌套Search偏移280/471.578947逻辑像素仍open，不能计整体设计通过。真实importer将Mesh0产为Model、两个PrimitiveN产为Mesh，材质关系沿原SceneMeshInstanceAsset.primitives保留，[RT-S24](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/runtime-contract.json)给具体接入与5公共依赖；不按.glb/名称/0skin判Toolkit。原ZUI及events、公共source/主题未改。原Rust、官方export/load、provider/正常EXE/144DPI和双端≤1px仍pending，passed/accepted=false。4自有Chrome退出0，1个0B临时目录保留待原Jenkins清理，释放0B/cleanup未关闭。

## 历史接续：原Shared Button同步、覆盖、完整重置及关闭重开104

原08 file/page现rev104，1921对象、5业务main及1147 sourceRefs保持。仅在原受控验证板的同一linked Button执行原生单组件sync→本地fill覆盖→完整reset：sync保留“Import”，覆盖为真实Shared surface.0；完整reset恢复surface.1和默认“Button”，有意清除原文字覆盖。两处本域说明随后由原生文字测量更新。[设计34](08-resource-tools-design.md#34-单个shared-button原生同步覆盖完整重置与关闭重开104)、[17项源检查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-button-native-01/source-checks.json)及[重开预览](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-button-native-01/reopened-full.png)记录精确范围。

四次完整document save ACK对应101—104；另一次sync-status RPC单列，不能充作保存。关闭重开104同hash、0额外保存/registry不变；其余1917对象、5组件记录及42源码输入精确保持。16张原PNG逐图实际打开，含一张空workspace失败诊断，不能把全部图计为成功设计。Shared当前实际rev5/4main/34色/3字样；此单消费者验证不等于整库更新/全局A/B/official export。原19002 manifest GET200但缺ACAO，19001浏览器实际CORS拒绝，[RT-S23](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-button-native-01/runtime-contract.json)交原public owner处理；本域未改host/服务/registry。正式ZUI/Engine provider、原Rust、EXE/实际144DPI及双端≤1px仍open，passed/accepted=false；自有7浏览器退出0，保留2个0B临时目录项，释放0B/cleanup未关闭。

## 历史接续：DataTableTab原生内容保存重开100

原08 file/page现rev100/1921对象、5业务main、1147有效sourceRefs。新增三尺寸DataTableTab内容及DataTableSourceContent本域主件/两份native copies，复用3份Shared Search linked copies；原1690对象仅zero-root追加三板，其余原对象和4组件记录精确保持。四行来自原ZUI固定样例，显式标注未加载Engine资产/schema，不是已就绪表格编辑器。[设计33](08-resource-tools-design.md#33-datatabletab内容原生设计与真实业务缺口)、[19项源检查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-source-01/source-checks.json)及[真实Penpot预览](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-source-01/data-table-number-fixed-640x520-crop.png)给出精确范围。

已由原生layout owner修正30个新增容器的存储子序；源检查又发现选中行序号应同用UE Orange，修正3个新增文字的paint并保留首失败。关闭重开100同hash、0保存/registry不变，Chrome24692退出0，当前6图均实际打开；42源指纹匹配。Shared消费依据归档rev5，900/640全图出现更新提示，未点击、未宣称同步到当前最新版。RT-S22交12项未执行业务验收与5项具体公共依赖；正式源导出/typed文档/provider/RowEditor/输入/Undo/save、原Rust、EXE/实际144DPI及双端≤1px仍open，passed/accepted=false。

## 历史接续：Browser后退/前进使用UE真实矢量，保存重开95

原08 file/page现rev95/1690对象、4业务main、992 refs。Browser本域主件及5个linked copies中的12个原字符对象已保留ID/16×16框及位置，改为实际UE CircleArrowLeft/Right的原生PathData和circle；新增24子图形、20来源引用，并绑定Shared rev5真实editor.text.disabled色样，维持旧灰色能力未知状态。其余1654原对象、151目录路径和rev94中文缓存精确保持。独立native关闭重开95同hash、0保存/registry不变、Chrome35152退出0；六板与导航特写14图均实际打开。[设计32](08-resource-tools-design.md#32-browser历史导航真实ue图形与生产合同)、[23项源检查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-history-icons-01/source-checks.json)及[真实预览](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-history-icons-01/history-640x520-062f92fc-crop.png)记录精确范围。

新RT-S21只交实际历史能力缺口：UE存ContentSources+selection、300项、Back/Forward更新当前记录并恢复来源；当前primary Workspace无导航history/cursor，AssetCommand/原ZUI也无Back/Forward，不能把图标当功能。Parent沿原state/binding/host/focus/source owner接续，不复制store或混入Editor Undo。[10项验收定义](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-history-icons-01/runtime-history-contract.json)0执行；640 reveal、900 tail、邻板缩略图刷新、正式export/load、原Rust和EXE/144DPI仍open。0产品源码/Cargo/服务桌面，全族passed/accepted=false。

## 最新接续：中文压缩由原生测量修复，保存关闭重开94

原08 file/page现rev94/1666对象、4业务main、972 refs。640中文板11处position-data已在原Noto Sans SC字体加载完成后，交Penpot原生DOM测量重建；“列表”宽度由9.73999恢复26.671875，“资源浏览器”由24.34009恢复66.65625。字号、文本框、文字、sourceRef、override与151原生箭头均未改；其余1655对象和4组件定义精确相同。独立全新Chrome重开94同hash/0保存/registry不变，保存与重开crop字节相同。见[设计31](08-resource-tools-design.md#31-中文缓存宽度由原生字体测量重建保存重开94)、[18项源检查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/chinese-text-metrics-01/source-checks.json)、[6图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/chinese-text-metrics-01/individual-review.json)与[真实预览](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/chinese-text-metrics-01/reopen-640x520-crop.png)。

本次仅关闭该板11处Penpot缓存压缩；Engine字体/行框、640选中目录reveal、900尾部scroll、公共展开/state/hit、正式导出/load、原Jenkins聚焦验证和真实EXE/144DPI/双端误差仍open。此前rev93中文open及旧入口状态按各自时刻保留；本轮0产品源码/0 Cargo/0服务桌面，passed/accepted=false。

## 最新接续：J01有限入口已部署，Audio执行仍待准入

原owner的19:50:05Z [J01部署回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-j01-intake-01/receipt.json)已接续：capsule 0b594015… 的Sound单包有限recipe已贯通，早期build_scope_policy拒绝保留为历史。原Audio request/operation/identity未改；本域核原4源码+18参考及14份引用回执全部匹配，没有重跑QueryOnly。

J01的335条JUnit/9项PowerShell属于管理入口验证，Audio仍0提交/0 Cargo/0测试。原owner唯一QueryOnly为ordinary_entry_unproven / Current session authorization is pending，无server receipt；writer/target/cache lock/reservation/capture均未取得。所报E盘12.16GiB低于v11 reserve+单job预算39GiB，不能据此开始捕获或Cargo。原owner沿原身份接自动收件及E根真实准入；本域不请求用户grant、不冻结文档、不改盘或清共享缓存。全族passed/accepted=false。

SoundWave逐copy消费的可靠来源约束新增于[RT-S17](#rt-s17soundwave原生copy与可靠pcm基线)，对应[设计第24节](08-resource-tools-design.md#24-soundwave保存响应的逐copy完整性)。只补原保存响应的离线字段/bytes证据，不改公共实现或组件。

本轮G01新增的是原asset manager的单一只读查询候选 `EditorAssetManager::readonly_asset_snapshot`；[精确合同](#rt-airef01-g01代码候选)及[设计第23节](08-resource-tools-design.md#23-rt-airef01-g01原manager只读adapter候选)承接Parent授权。无公共UI组件、主题、registry或AI语义扩写；测试仅定义，未执行。

本域：`01a16d53-60db-7d02-b03e-0042f277783d`。接续 [盘点](08-resource-tools.md)、[具体设计](08-resource-tools-design.md)、[Parent交接](08-resource-tools-parent-handoff.md)。归属依据为 [Parent最新分工](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)，不改动其它域或公共实现来代替交接。

## 实际文件与操作边界

已读取 Parent 的 `penpot-assignments.json`，其 `verifiedAtUtc=2026-10-24T10:11:47.184Z`，包含第二个独立认证上下文的读取回执。本域实际文件 [08 Asset Tools](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60ef2172&page-id=b4a1e3ce-9e6a-81fe-8008-d69d60ef2173)：

- project `b4a1e3ce-9e6a-81fe-8008-d69d606f6160`；file `b4a1e3ce-9e6a-81fe-8008-d69d60ef2172`；page `b4a1e3ce-9e6a-81fe-8008-d69d60ef2173`。
- 分配时rev0/单根；历史12:51:47Z实际读回rev40/vern0：1164对象、495flow、13板、3本域原生业务component、18链接copy，40对ACK及fresh browser/context核验；见[最终设计](08-resource-tools-design.md#最新接续revn40纹理源图与原生png)。共享库仍rev0/零component/color/typography，跨文件primitive/style/官方codec及ZUI导出未交付，不以本域组件冒充公共复用。
- 历史13:41:30Z读回rev47/1225对象/521flow/14板、3local main/20copy。公共file rev1/isShared=true/4组件/零color、typography；本域真实library link与首个SearchInput foreign copy已fresh保持。见[RT-S10](#rt-s10单browser的当前消费与滚动接续)，旧rev0观察不覆盖新事实。
- 15:00:32.993Z独立fresh重开的历史rev55/1262对象/524流式对象/**15设计板**、3local main/24copy（5 foreign）；专属验证板原生消费四已发布主件、保留33 sourceRef，Button本地文字override/reset已核。Shared最近实查仍rev1且原始响应hash前后一致、styles0，未复制main或替换旧Browser映射。精确源字段/slot/实例/覆盖、保存原始缺口和三个公共差异见[RT-S11](#rt-s11四个已发布公共主件的受控试接)。
- 历史15:43Z本域rev57，计数与rev55相同；只修正Import/Picker四个既有text的能力文案及其原生字形缓存，另外1258对象、全部公共实例、几何和文字样式保持。两个新保存请求/lagged ACK与独立新browser逐板可见SVG已核。Dialog实查无body/actions槽，内容研究不等于modal复用完成，见[RT-S12](#rt-s12pickerimport的modal与真实能力边界)。本轮没有重读或修改Shared，不将本域rev57当公共库新版本。
- 最新16:47Z独立新browser重开实际rev76/1337对象/551flow/18板/4本域main/26copy（5 foreign）。SoundWaveSourceContent草稿v1及两native copy48refs、19对ACK已核，Shared同次rev1/4main/零styles、原library link保持；只新增源内容75对象和3板引用，旧1261非根对象/3main保持。见[RT-S13](#rt-s13soundwave源内容与公共消费边界)，数据几何与URI显示覆盖仍为本地业务草稿，不是公共发布或ZUI闭包。
- 动画共享file `b4a1e3ce-9e6a-81fe-8008-d69d60a13f99` / page `b4a1e3ce-9e6a-81fe-8008-d69d60a13f9a`，专责身份/合同保留，本轮未核实其新发布或复制其main。
- 可进行不影响共享桌面的 API/隔离无头浏览器设计。当前没有本域桌面时段；不得操纵系统 DPI、鼠标、键盘、Editor 原生捕获。服务由 Parent/既有 owner 管理，不自行恢复 Jenkins/Penpot。

本域于2026-10-24T10:26:55.450Z用既有授权会话及现有官方`transit-js@0.8.874`独立读取`get-profile`/上述本域及公共`get-file`，均HTTP 200。两个文件均`revn=0, vern=0, is-shared=false`、单页单根对象，components/colors/typographies均0；没有Penpot写入或浏览器/桌面输入。该核查不是保存重开验证；原始响应及工具哈希见[Parent交接](08-resource-tools-parent-handoff.md)。

下表是请求的最小消费合同。`RT-Sxx` 是文档工作项，不是已存在的 Rust 类型、ZUI component ID 或 Penpot UUID。

## Parent 公共框架与数据组件

历史12:30Z消费者新增AssetTile本域组合草稿v1，component `30625d51-64a3-469f-84c5-79e156375f8c` / main `a87b3a71-49e3-432b-9b59-be99063789b2`，11linked copies、三个真实纹理grid板；当时全文件revn28/924对象/381flow/2business components/16copy heads。它不是已注册/发布公共main或另一个列表/主题。RT-S03输入需区分Penpot media UUID与Engine asset UUID/provider hash/ready；当前四真实源文件无.zmeta，后者null。现有Runtime SourceImage provider可复用，未证明本域运行时已加载。完整媒体/板身份、wrap/clip及保存重开证据见[设计接续](08-resource-tools-design.md#最新接续revn28真实纹理网格与assettile)。公共promotion、primitive跨文件实例/style和虚拟化准入仍由Parent统筹；原RT-S01…09继续有效。

本域rev40新增TextureSourceContent draft v2（component `02db8ec6-f26e-482e-9df2-1c0a02753b02` / main `7f5ecf4a-fb69-4710-b1ad-0384d19e2cbc` / 两copy），只用源Image/Label/flow组合，未新增通用属性行/主题。RT-S07必须区分sourcePNG字节/尺寸/alpha、Engine asset UUID、document dirty、PreviewSession GPU ready及mip/resource统计；本稿后四项未知/未接，通道/保存/重导入禁用。RT-S02/S04/S06及正式导出需保留本地文本与颜色覆盖；本轮按Penpot真实文本同步子组修复27项copy，fresh保持Fit44%/34%，不能把content-group单独存在当文字覆盖有效。完整身份/证据见[rev40接续](08-resource-tools-design.md#最新接续revn40纹理源图与原生png)。六个原生PNG Blob已真实导出但download均canceled，原始下载链由Penpot/公共pipeline owner核对，消费者没有改公共实现。

现有身份以 [catalog.toml](../../../zircon_editor/assets/ui/editor/components/catalog.toml) 为准。公共路径、组件和主题只提交提案；资源域只负责原始 Browser/Drawer/Picker/Import 与资产工具内容及其业务适配。

| 请求 | 已存在的身份/实际表面 | 本域需要的具体输入、输出与边界 | 首次可验证的消费者 |
| --- | --- | --- | --- |
| RT-S01 文档/Dock/Drawer | `UiHostWindow`、`DocumentHost` 来自 `unreal/editor_shell.zui`；DocumentHost slots 为 `tabs`（DocumentTabs）、`content`（PaneSurface）；`ActivityDrawerWindow` 具有原七个 activity/content slots | document identity 按实际资产UUID/类型+project activation；打开同资产聚焦原文档；dirty/save/close/cancel、tab历史和关联子页生命周期。Drawer dock复制浏览设置并保留临时入口；Browser多实例由宿主决定。不在资源域新增第二套DocumentHost | Browser、Drawer与首个实际Mesh/Texture document |
| RT-S02 树/表/列表 | `WorkbenchTreeRow`、`WorkbenchListRow`、`WorkbenchTableRow`；当前catalog未声明这些数据行的通用slot/选择模型 | 本域提供folder ID、asset UUID/locator、row revision/代次、真实文本/图标；共享表面负责hover/selected active/inactive/focused/disabled、缩进与键盘导航。沿既有state保留Find-in-Browser同步集合及单primary，再补通用Ctrl/Shift集合、lead/anchor和范围排序；不能只有多项高亮或新增平行selection store。逐字段依据见[设计第9节](08-resource-tools-design.md#9-真实字段项目资产与路径准入) | Browser目录树、列表；DataTable真实行 |
| RT-S03 Tile与虚拟化 | Drawer已有`UnrealContentAssetPrototype`与virtual_rows、item_extent=28；catalog没有核实到`AssetTile`身份 | 优先组合现有Image/Icon/Label/Button与共享样式，若提升公共组件由Parent登记。需要稳定实例ID、真实thumbnail image/provider hash/ready/error、name/type/tooltip、selected/focus/drag。Grid虚拟化需真实tile extent/columns/overscan/scroll-into-view，paint/hit-test/drag必须同代次；禁止在资源域复制并行list系统 | Drawer真实Thumbnail与Browser Grid |
| RT-S04 搜索/路径/数值输入 | `WorkbenchSearchInput(query,placeholder)`；`WorkbenchField(value,placeholder)`；`WorkbenchNumberField(value_text,placeholder)`；现有dropdown和icon按钮 | 保留IME草稿、change/submit/cancel、文本Undo作用域、非法值与只读状态。路径crumb复用Button/Icon/flow；child popup/文本路径编辑返回folder identity。数值编辑适配mip/curve值等各域类型，公共输入不解释资产语义 | Browser search/filter；Import destination；Curve key输入 |
| RT-S05 Modal/上下文菜单 | `WorkbenchDialog`原ZUI只有一个Dialog root（WorkbenchDialogRoot），catalog公开props只有popup_open/title/message；源和catalog均无body/actions槽。`WorkbenchConfirmDialog`有confirm_text/cancel_text/severity/destructive；`WorkbenchContextMenu`与`WorkbenchPopupMenu`有options/menu_items/context_target等 | Picker/import所需可编辑body/actions尚未交付；Parent须沿现有overlay owner核实际内容/动作挂载，再公开合同。本域不传虚构slot/props。默认focus、trap/Tab/Esc/关闭提交后的当前generation焦点返回仍待真实验证。菜单项目来自实际capability/selection与既有格式，不另造menu schema；确认仅一次、取消不提交、子popup先消费Esc | 类型受限picker、Save冲突、Import确认、资产/目录/空白右键；详见RT-S12 |
| RT-S06 真实Preview表面 | `WorkbenchPreviewViewport`当前源码是非交互Image，指向`animation_mannequin_viewport.svg`；不代表Parent生产session | 与角色CS-D02消费同一个Parent PreviewSession/image/input/frame/lifecycle。Static Mesh/Texture归08，Skeletal Mesh/Skeleton/Physics归角色，关联Material归材质；按真实registry/command/host/domain-ready及当前generation的focus返回，见本节补充。Runtime负责真实renderer/资产，不复制backend/clock或用SVG作ready | 当前单Browser与CS-XR九项；实际session/provider/资源ready仍pending |
| RT-S07 共享主题/导出 | 复用`editor_tokens.zui`、`starship.zui`及原strict样式依赖；沿用现有Penpot source ownership/metadata与LayoutReviewCase | 提供shared library/component ID、版本、实例和style/token refs、局部override/reset策略；跨文件实例更新/导出必须保留源node/control/import/event/slot。Browser strict与Drawer Starship的优先级统一交Parent，不能在本域新建配色。首次导出依赖闭包与正常EXE加载hash由公共协议承接 | 同一共享输入/行样式在Browser、Drawer的传播；再与其它族交叉验证 |

`WorkbenchPanelHeader`已声明title/actions slots，可直接作为区域头的首选；`AssetSurfaceControls`已有controls slot（ComboBox/IconButton/InputField），保留binding namespace `AssetSurface`。这些可复用身份不等于已验证完整业务行为，也不意味着可以越过slot接受类型或无记录地替换组件。

## 动画 owner：独立曲线与声音播放

共享时间轴/曲线/播放控件 owner 为 `01a16d51-741b-73b3-a61a-2189cd38f81e`。共享源修改需先交接；本域不直接改`ui/curve`、timeline/transport原资产或动画投影。[动画设计](animation-design.md)可复用其选择/事务/预览原则，本域不把动画Sequence或模板时钟当作独立资产/音频后端。

owner 的 [实际消费合同](animation-shared-controls.md) 已提供下列 UUID。本域于 2026-10-24T11:48:08.846Z 独立官方 API 读回动画 file：revn8/vern0、302 对象、3 components、colors/typographies 均0；与 owner 文档相符。下列版本是草稿合同版本，**不是公共库发布版本**；没有在资源文件复制这些 main，也没有把 file ID 当组件 ID。

| 专责组件 / 草稿版本 | component UUID | main instance UUID | 本域消费边界 |
| --- | --- | --- | --- |
| Timeline Surface / 2 | `8cf70bfd-9275-4bac-849b-b79148b8a4fd` | `4db0829c-abc4-482d-b7b3-155a5d8d75c2` | 声音可选时间变换，不强制动画 FPS；整个组合尚无 ZUI wrapper，只有内层映射 WorkbenchTimelineStripRoot |
| Curve Surface / 1 | `78492c86-3bb4-47dd-97c6-752d6ff37e3d` | `599963dc-b91b-4740-926b-d38154aa8f3a` | 独立 Float/Vector/LinearColor adapter；不用动画 track 或模板播放时钟冒充独立资产 |
| WorkbenchTransportControls / 1 | `982ef940-2239-4167-a24a-56c80ec35fbf` | `44eb5ccd-8ac2-44f2-a4ad-d0d618daa096` | 保留原 8 个 control ID 和下列六 event/route；真实 Sound session/capability 尚待接入 |

跨文件库引用、共享样式、官方 ZUI 导出及真实 preview/session 均仍 pending。Parent 已把最小基础组件发布/导出闭环派给原 pipeline owner `01a0f04d-9d54-7fe2-9adf-120186804cfb`（其调度 ACK `01a16d95-5ffc-73f3-971a-e3676fb2b6b8`）；该 owner 的共享文件写权限不传递给本域。

### RT-S08 Curve消费合同

实际 [ui/curve/model.rs](../../../zircon_editor/src/ui/curve/model.rs) 已提供`CurveModel { CurveId, Delta, Error; curves(); apply(delta)->Result<Delta,Error> }`，`CurveView`、`CurveKey`、`CurveSelection`和`CurveElementRef {curve_id,key_id,kind}`。`kind`可区分Key/InTangent/OutTangent；插值为Step/Linear/Hermite；切线是可选标量斜率。坐标变换见 [canvas.rs](../../../zircon_editor/src/ui/curve/canvas.rs)，曲线轴value向上、屏幕向下，不重复实现变换。

| 边界 | 请求的共享能力 | 本域保留的业务权威 |
| --- | --- | --- |
| Curve身份与投影 | 泛型CurveId/稳定key ID；外层project/document/revision用于拒绝旧结果，选择来自CurveSelection；不使用当前可见索引作为持久key | 独立Float/Vector/LinearColor的真实资源类型、channel ID/名称、key序列化、source UUID/locator与revision由本域/runtime adapter提供；当前还缺这些独立资产 |
| 横轴/纵轴/视图 | 共享bounds、pan/zoom/frame、snap配置、tick/label与curve_to_screen/screen_to_curve；有限值验证、最小viewport/bounds处理 | model字段叫time；独立曲线的时间/输入域单位与范围由资产声明，不能硬绑动画FPS、AnimationTrackPath或强制播放时钟。Vector和LinearColor按真实channel映射，颜色空间/alpha由typed asset决定 |
| Key/tangent/选择 | 点选/框选/增删、Key与In/OutTangent独立命中；readonly/capability、focus与context菜单；连续拖动begin/update/commit/cancel | 域delta执行、合法性、反向delta、dirty/history与持久化；`CurveModel.apply`的可逆协议需复用，不能在UI中直接修改runtime源或给独立曲线复用动画delta |
| 事务与快捷键 | Escape取消本次gesture；一次拖动只形成一次Undo；文本框先处理IME/编辑Undo；失焦/关闭有明确取消或提交 | 本域asset document的Undo/Redo/Save/关闭决策和重新求值；runtime保存成功后才清dirty，fresh reopen证明落盘 |
| 可选关联 | 共享表面允许无timeline/无transport的独立Curve；可选外部cursor只作观测 | 独立曲线默认没有动画播放对象，不制造定时游标。颜色曲线关联ColorCurveEditor与真实RGBA值；动画曲线仍完全归动画owner |

现有 [animation curve_foundation.rs](../../../zircon_editor/src/ui/animation_editor/session/curve_foundation.rs) 从选中Sequence track投影scalar/vector，明确Bool/Integer/Quaternion留在非scalar lane；这可以证明共享词汇有消费者，不能证明独立Curve资产已支持。模型只表达Hermite斜率，没有核实weighted tangent/weighted handle数据；UE相关项保持待定，不能只画手柄而丢权重。

### RT-S09 Timeline/Transport消费合同

实际 [timeline_strip.zui](../../../zircon_editor/assets/ui/editor/workbench/primitives/data/timeline_strip.zui) 是`Canvas`，props为duration/current_time/tick_interval/track_label/timeline_keys，输入标志均false；不等于可拖scrub的时间轴。[transport_controls.zui](../../../zircon_editor/assets/ui/editor/workbench/animation/transport_controls.zui) 保留以下既有稳定身份：

| control / event | 当前route | 本域消费要求 |
| --- | --- | --- |
| `WorkbenchTransportRecord` / `WorkbenchExtension/AnimationTransportRecord` | `workbench.extension.animation_transport.record.toggle` | 当前资源SoundWave只需试听；录制没有真实capture/model时不提供有效操作，不把切色当录音 |
| `WorkbenchTransportPlay` / `WorkbenchExtension/AnimationTransportPlay` | `workbench.extension.animation_transport.play.invoke` | 适配到选中真实声音资产/session的play请求；playing由实际输出确认，不能用模板时钟冒充 |
| `WorkbenchTransportPause` / `WorkbenchExtension/AnimationTransportPause` | `workbench.extension.animation_transport.pause.invoke` | pause确认后位置稳定，resume从实际playhead续播；输出失败回真实error |
| `WorkbenchTransportPrevious` / `WorkbenchExtension/AnimationTransportPrevious` | `workbench.extension.animation_transport.previous.invoke` | 既有语义不能直接改名声称单帧。声音需要明确seek start；动画owner需区分start/end与step，不令声音使用硬编码动画端点 |
| `WorkbenchTransportNext` / `WorkbenchExtension/AnimationTransportNext` | `workbench.extension.animation_transport.next.invoke` | 同上；声音是否需要seek end/step sample由真实backend capability决定 |
| `WorkbenchTransportLoop` / `WorkbenchExtension/AnimationTransportLoop` | `workbench.extension.animation_transport.loop.toggle` | loop/play range由实际声音session响应；无range支持时不能只让图标保持按下 |

动画owner已在[资源消费合同RT-S08/09](animation-shared-controls.md#资源消费合同-rt-s0809)交出现有binding/slot/参数来源表与Stop/Seek迁移提案，本域已读回，不再等待重复交付。component仍无公开domain/session command slot，TimelineStrip是input=false的显示参数来源；catalog/root参数存在不等于跨实例可透传。Parent须明确实例project/document/view/source/backend authority后分流，不按全局focus猜域或覆盖六原handler。新增Stop/Seek需参数/payload/slot/catalog/dispatch及消费者迁移具体diff与正式导出，不另建TransportControls。

独立Curve复用泛型CurveModel/typed selection/CurveCanvasTransform和delta inverse；asset/channel/key身份、Float/Vector/LinearColor序列化及document history归08。输入域/单位不需AnimationTrackPath/FPS或clock，f32视图/finite fallback不是高精度源validator。当前Step/Linear/Hermite按curve，尚无已核实的逐key/weighted handle持久存储；缺能力禁用并给原因。

声音playback_status先查Kira，失败可回退本地cursor，不能混称已确认输出位置；seek接f32 seconds并按sample rate取整/夹range，requested与actual须分开，不宣称精确sample API。per-playback Stop/关闭释放与全输出设备Stop不同，关一份资产不停止其他消费者；无真实capture/result的Record没有有效录音能力。本次只消费原owner合同/源码，不改公共route/clock/backend。

SoundWave/Waveform所需反馈为asset UUID、project activation、playback session/revision、duration、position、sample rate、真实状态（loading/ready/playing/paused/stopped/failed）和各操作capability。以实际声音输出/解码为权威，单位为秒/采样位置；不能用动画FPS假定采样帧。UE Waveform的Space播放切换、Enter停止、Shift+Space回首由声音document局部命令处理，公共按钮不注册抢占全局的快捷键。stop后输出释放、切项目/切资产/关闭后拒绝旧playhead返回，必须保留可验证结果。

波形显示由本域PCM/sample数据投影，不能把音频样本强行装成可编辑CurveKey；波形缩放/选区可消费同一时间变换思想，破坏性裁剪/淡入淡出仍需要本域真实变换模型、导出与撤销。未具备时登记缺口，不复制动画曲线或渲染固定正弦线。

## 材质 owner：只读依赖图消费提案

共享节点图表面owner为`01a16d50-6c7f-7370-853d-4581184b66a1`。本域材质槽、动画等专用资源引用双击只走实际registry/toolkit到对应owner；不重复材质/动画编辑器。Parent已将Reference Viewer与Audio全族业务交本域；真实SoundMixerGraph/validator/Kira compile已存在，Cue程序、Class继承及三类持久typed资产/作者文档仍缺，不能用节点外观关闭缺口。

Reference Viewer 请求复用只读 graph surface：本域提供 project activation/catalog revision、节点 asset UUID/locator/known-project 状态及有方向的 references/used_by 边；共享层处理 pan/zoom/frame、稳定命中、选择、焦点及上下文菜单锚点。双击/Find-in-Browser/打开正确编辑器仍回资源域命令中心。节点不具备材质端口/连线编辑、编译或音频执行语义；未知外部引用可显示但不冒充可拖动项目资产。hard/soft、循环、深度/宽度限制、依赖种类尚须 provider 给真实信息，不能从一组 UUID 猜出分类。该提案先交 Parent/graph owner；本域未修改共享图源码或创建第二套图表面。

## RT-AUDIO01：Cue/Class/Submix图消费合同

J01执行入口依赖已由原owner完成有限部署；精确身份、管理验证与Audio未执行/资源不足的分界见[设计第26节](08-resource-tools-design.md#26-rt-audio01-j01有限入口修复的接续边界)。本轮只消费该回执；RT-AUDIO01业务schema、原六transport routes、producer来源和typed资产缺口保持原责任与状态。

当前托管入口接续：本域实际2790fb51… capsule的11/11文件和driver197/197匹配；原identity QueryOnly返回build_scope_policy，允许material/prefab/AI/glTF，尚无sound editor mixer_authoring_。RT-AUDIO01-J01已向原Jenkins owner成功排队，未获已读/准入；20b3及首次harness拒绝保留为历史。不能用Parent的authorization pending代替本域实际结果，详见[设计第15节](08-resource-tools-design.md#15-jenkins-r2原身份查询与真实texture-provider接续)。

接续Parent明确Audio全族收件和[MAT-SHARED-001](shared-node-graph-contract.md)。资源08持有Audio资产格式/作者语义、选中资产adapter、命令与文档持久化提案；材质owner `01a16d50-6c7f-7370-853d-4581184b66a1`仅提供中立GraphModel/Canvas/Delta/输入和呈现；Parent接现有Sound runtime/provider/生产输出、公共宿主与跨文档事务。下文共享字段/行为仍为待接入合同，未注册公共DTO、route、组件或持久资产；域内已实现只读Rust投影见本节末及设计第12节，不扩大公共写域。

实际来源：[SoundMixerGraph/Snapshot/Track/Send](../../../zircon_runtime/src/core/framework/sound/graph.rs)、[SoundMixerGraphManager](../../../zircon_runtime/src/core/framework/sound/manager/mixer_graph.rs)、[SoundEffectDescriptor](../../../zircon_runtime/src/core/framework/sound/effects.rs)已有Serialize/Deserialize、显式轨道/效果ID、parent/sends/effects、sample-rate/channel-layout和meters。插件[authoring_bindings](../../../zircon_plugins/sound/editor/src/authoring_bindings.rs)的Mixer/Output操作注册不等于宿主已实现asset-bound命令；[plugin](../../../zircon_plugins/sound/editor/src/plugin.rs)注册的是Mixer/Acoustic Debug与组件Inspector。不能报告“完全没有音频图模型”，也不能把现有Mixer配置改名为Cue/Class/Submix资产。

UE的[SoundCueGraphSchema](../../../dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/SoundCueGraphSchema.cpp)在339…393校验方向/同节点/环，单input替换，成功后CompileSoundNodesFromGraphNodes；[SoundClassGraphSchema](../../../dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/SoundClassGraphSchema.cpp)72…124同样单input替换，并LinkSoundClasses；[SoundSubmixGraphSchema](../../../dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/SoundSubmixGraphSchema.cpp)213…350则单output替换/多input，另有dynamic、endpoint、soundfield兼容性。共享表面不得把三者都硬编码成“同类型pin即可连”或固定单input策略。UE pins中方向与业务父子关系不能仅按Input/Output名称猜测，adapter输出实际端点和合法关系。

| 目标模型 | 可消费的当前基础 | 资源域必须给出的schema/runtime语义与未实现边界 |
| --- | --- | --- |
| Cue | 真Sound资源、source/clip与动态事件的独立运行基础 | Cue文档/根Output/节点稳定ID、节点kind/schema版本、可变input slot稳定key、SoundWave引用/随机或选择等已支持行为、实际lowering/编译/执行结果。当前Mixer graph不能表示任意Cue程序，palette/Connect/Play必须按真实支持集；不造默认完整UE节点列表 |
| Class | 轨道ID和parent可作已有Mixer关系参考 | Class资产ID、显式parent/child与属性覆盖/继承规则、跨Class引用、与SoundWave/source的绑定及有效值回读。Mixer track不自动等价于SoundClass，缺实际typed资产/继承求值时保持不可编辑目标 |
| Submix | SoundTrackDescriptor(parent/sends/effects/controls)、SoundEffectId、当前graph validator/compile可复用 | 明确持久Submix资产与runtime track ID映射、主输出与send/sidechain边种类、layout/声道/采样率、效果有序链、支持的控制/输出代次。普通Mixer只读投影可先用真实snapshot；不能据此宣称UE endpoint/soundfield/dynamic或效果运行已实现 |

| 接口 | 域→共享表面/宿主的具体数据 | 返回/失效/验收要求 |
| --- | --- | --- |
| target与版本 | project activation/asset UUID+locator/type、document/graph/subdocument、source/model revision；独立surface/layout generation；真实read/edit/connect/compile/play/save能力 | 沿GraphEditContext的Target，重导入、切项目/文档或revision过期取消旧手势；asset UUID未知不可用media或SoundTrackId替代。公共层不拥有第二份Audio模型 |
| node/port/edge schema | NodeId用实际作者key；Mixer只读node可用当前snapshot中的SoundTrackId，effect用(track ID,effect ID)；port沿GraphPortRef(node,direction,stable pin schema key)，显示名另投影。可变Cue slot须由真实格式提供稳定身份 | 保持图、Details与列表选择同一authority；禁止用显示名、Vec index、数值gain或当前时间作持久key。parent/send/sidechain/effect order是不同关系；send现无独立edge ID，需完整source/target/role及域唯一性，未有可逆key不提供歧义删除 |
| verdict/替换 | 域对候选图作whole-graph校验，返回实际将替换的边、conversion/布局限制和原因；区分Cue/Class单input与Submix单output语义 | 共享Allowed/Rejected只是呈现；候选先移旧边再验，重连一次delta提交旧+新+inverse。禁止先断开再失败，禁止共用材质Scalar/RGB转换 |
| mutation/history | 真实palette/schema/defaults、expected revision、域delta/inverse与retention proof、owning document/history context | 使用现有GraphDeltaCommand的proof→permit→prepare→apply，一次gesture一次可撤销提交；Esc/capture-lost/关闭取消。pan/zoom/选择不进入资产Undo；runtime graph rollback不是Editor Undo |
| compile/发布/last-good | 当前源/请求generation、lowering结果与node/pin/source map、实际runtime能力、候选compiled revision、活动output generation及发布回执 | [compile_graph](../../../zircon_plugins/sound/runtime/src/kira_bridge/graph_compile.rs)183…254先validate_graph再validate_m1_surface；553…589当前明确拒绝所有非空效果链、特定高级controls与pre-effect send。[configure_mixer_impl](../../../zircon_plugins/sound/runtime/src/service_types/mixer_graph/configuration.rs)仅输出active时编译并执行图更新；inactive只通用校验。接收配置≠输出可执行；不可跳过guard或翻capability，失败保留真实last-good/报告失效 |
| 播放/meters/诊断 | 实际source/clip/session/输出device及output generation、clock/position、sample-rate、layout、真实meters和request/source revision | 运行active样式不能由点击/选择或假计时生成；meters无数据应未知而非模拟电平。动画transport复用原6route，Sound秒/采样adapter不复制clock；仅选择Mixer轨道不证明Cue在执行 |
| 保存/关闭/重开 | 真实文档dirty/saved revision、目标路径与hash、完整引用依赖；发布与磁盘保存分别回执 | 已有MixerGraph serde不是项目资产保存/Toolkit；新格式/registry与跨文档patch先交Parent具体路径。关闭最后consumer释放本域预览source/session，不能停全局音频设备或别域消费者 |

最小只读域投影已落实在[SoundMixerAuthoringSnapshot](../../../zircon_plugins/sound/editor/src/authoring/mixer_snapshot.rs)：借用调用者的原snapshot，track行含snapshot内source_index，parent与每条send独立保留；重复ID/未知target不消失，effect顺序、controls、meters及完整source保持。source_index和send index只在当前snapshot有效，不能作为持久节点/边ID；项目、source revision、provider和output generation仍须由真实调用者提供。未连接UI/graph surface、未接写命令，不能称Submix资产完成。

原[SOUND_MIXER_OPERATION_PATHS与schema_id](../../../zircon_plugins/sound/editor/src/authoring_bindings.rs)仍为唯一20项Mixer/Output命令描述，要求`editor.extension.sound_authoring`。track create/update_controls/delete、send upsert/delete、effect add/update/delete/reorder、preset list/apply、sidechain set_source、automation bind/unbind、dynamic_event registry.open及output refresh/configure/start/stop/debug沿原operation/payload schema，不另发明Cue/Class/Submix command ID。特别保留`track.controls.v1`、`sidechain.source.v1`、`acoustic.layer.v1`三个非机械同名schema；描述符注册不等于asset-bound handler、文档Undo或真实执行。

SoundWave先消费真实SoundAsset PCM、SoundClipInfo及SoundPlaybackStatus，不从播放按钮/计时器构造波形、position或readiness；clip/playback/track ID均不替代asset UUID。Submix提案要明确持久资产↔runtime track映射，不能把普通track改名后保存成已支持Submix。Cue须有版本化程序文档、稳定节点/slot、Wave引用、输出根及实际lowering；Class须有显式parent、override/继承求值和资源绑定。当前这些typed文档和持久化均未创建，详细字段/失效/命令边界见[设计第12节](08-resource-tools-design.md#12-audio最小只读适配与真实资产文档提案)。

精确四Rust路径格式/diff/change-guard通过，首次格式和模块可达性失败原样保留。一次正式Jenkins入口在parse、identity load/dispatch之前拒绝harness参数；随后仅调用固定client的纯parser，确认当时20b3bf85… capsule只准material_graph_/prefab_/两条glTF，不含sound editor，未再次提交/编译。请求公共owner接RT-AUDIO01-J01：单包zircon_plugin_sound_editor、原plugin manifest、lib/test、mixer_authoring_、无默认features/直接features为空、locked/jobs1/testThreads1；同时核实际sound/editor路径依赖闭包及server检查，不单改client放行。见[原拒绝](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-mixer-jenkins-run-01.json)与[真实parser/三处合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-mixer-route-parser-01.json)。不换manifest/包名绕规则，不改capsule/服务，不作local Cargo fallback，原请求身份保持。

三个测试未执行；后续仍需真实输出、失败last-good、Editor undo/redo/save/重开、跨项目/设备丢失的旧请求拒绝。六条transport route、公共capability和validator未改，无伪波形/录音；三尺寸/实际150%、正常EXE及双端比较pending，passed/accepted=0。

## Parent 插件合同：本地组件归属与资产闭包

对现有插件做了只读调用链核查，未修改公共源码，也未直接运行生成器。源码当前能力与接入请求如下；不是已经证明所有其它导入入口都没有能力。

| 已核对的路径/边界 | 当前实际代码 | 资源域需要的最小公共接续 |
| --- | --- | --- |
| [real-session](../../../tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-real-session.ts) 的 `--source-closure` | 与 `--full-workbench` 调用相同的 `prepareRealWorkbenchInput`；不带开关则使用原始未展开文档 | 明确 Browser/Drawer/AssetWindow 的受管 source closure 入口，保留 root asset ID 和跨 owner 的 source/slot/binding/event/fingerprint。不能把该开关的通用名字当成任意资产已可用 |
| [real-workbench-input](../../../tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-real-workbench-input.ts) | `owner` 固定为 `unreal/level_editor_window.zui`，函数先 assert sourcePath 为此 owner；依赖原完整宿主准备与 fingerprint 校验 | 复用现有 `LayoutDependencies`/`prepareComponentReviewHost` 支持的 AssetWindow 与 Drawer 挂载能力，Parent 决定如何公开最小 family 入口；本域不复制另一个工具链或把 Browser 冒充 LevelEditor 根 |
| [native-components](../../../tools/penpot/apps/zircon-zui-plugin/src/penpot-native-components.ts) 与 [ownership](../../../tools/penpot/apps/zircon-zui-plugin/src/penpot-native-component-ownership.ts) | full/leaf 模式在当前文件 local library 创建 main component，并按本 asset board 的 shelf/source/structure 缓存；查找使用 `penpot.library.local.components`。semantic 模式显式 deferred，设计用，不满足原生实例要求 | Parent 给实际共享 library/component/version/source→instance 解析与 owned shelf 复用合同，处理跨文件依赖/局部override/export；消费者不在本域产生重复公共 main。不能用 semantic 绕过实例验收 |

`prepareComponentReviewHost` 中已有 AssetWindow 七个 mount 及 UnrealContentDrawer 的实际 source 入口，可作为复用起点，但其附加运行时数据和正常 EXE 证据仍需核实。请求 Parent 统筹上述公共路径，资源域提供四个原始 `.zui v2` owner、具体消费者和逐项缺口，不直接覆盖原工具。源码核查不等于当前已部署 bundle 具有这些能力；部署身份仍需原 owner 的真实构建/服务回执。

四个原始 owner 的**静态 authored imports 递归**已用只读 TOML 检查；仅沿 `imports` 中的 `res://…zui`，不执行生成，不展开动态 slot、catalog 或正常 EXE 的媒体/字体依赖，所以不能充当完整运行时依赖闭包。

| 原始 owner | 含根文件数 | 唯一 widget/style 引用数 | 缺失文件 | 本次静态闭包指纹 |
| --- | --- | --- | --- | --- |
| `asset_browser.zui` | 10 | 7 / 2 | 0 | `d659366385cc35a97161b7baee6a8b283c24c8b9bddfcf6885abe9334be28554` |
| `unreal/components/content_drawer.zui` | 4 | 1 / 2 | 0 | `7434f3655473eabdf931c12b7e57091975295c8ca9d28a7a39509c6e03a55754` |
| `windows/asset_window.zui` | 3 | 1 / 1 | 0 | `e92698e359ef5579dae4cb50a9115972010dea78c6e232f08aec9bc4fed200aa` |
| `modules/data/extension_data_table_workspace.zui` | 10 | 8 / 1 | 0 | `72ada4253bba8b0345220f2287e961172f920b9850e195a6b92b4faae8cc6234` |

静态闭包指纹算法为：相对 `zircon_editor/assets` 的路径与原文件 SHA-256 按路径排序，逐行以一个空格连接、行间 LF、无末尾 LF，再计算 SHA-256。原始 imports 不变；其中 Browser→strict→editor_tokens 与 Drawer→Starship→editor_tokens 的统一仍归 Parent，不因本次遍历而新建样式。

## 最小集成顺序与状态

1. Parent给出公共Penpot组件/共享样式的实际UUID/版本和跨文件实例使用入口；动画owner给出Curve/Timeline/Transport的真实消费合同。上述文件身份已经存在，无需再次询问用户服务/页面。
2. 本域在已分配文件用这些实例构成Browser/Drawer/picker及独立工具内容，保存关闭fresh重开；不在自己的文件复制公共main component/平行token来绕过依赖。
3. 官方导出回原始ZUI v2，核对现有绑定/事件/导入和实例身份，再接真实asset/document/runtime；缺的backend逐项保留。
4. Jenkins聚焦插件/Rust检查与正常EXE捕获分别取得实际回执；桌面时段请求给出窗口族/file/page、状态/尺寸/DPI、预计时长和已ready的EXE/资源hash，Parent明确授予后才输入/捕获。完成后报告进程保留/退出、输入释放、证据路径与失败。

Parent最新记录已确认RT-C01…05与RT-S01…09的责任边界，并允许本域以现有Image/Icon/Label/Button及flow组合Tile/Breadcrumb；提升公共组件时仍由Parent登记catalog、样式及实际实例身份。该确认是责任与接入方向，不表示公共组件/插件/后端已经实施。

当前revn26本域业务组件`BrowserContent`为`6dbdeee0-4fe3-80ec-8008-d6afd543b430`，main保留原split `30578a6c-27a0-80d5-8008-d6a477785097`，草稿业务合同版本2。五个链接副本消费同一组件，新增Model picker copy `95ebe941-228c-4316-b1ab-688f048fc335`；独立导入内容板登记原source字段/ImportModel事件业务合同，确认仍禁用。原六frame/六text UUID保留，原546个非根对象全部属性保持。684对象/261flow和七个实际画面已经保存重开/逐图核对；窄窗/Drawer滚动、MDL占位、共享控件/样式、Modal业务与真正预览仍欠缺。`zircon-ue-assets`不是官方ZUI codec。12:13共享文件仍零组件/样式；跨文件公共实例、官方导出、正常EXE及双端review未完成，本域`passed=0`、`accepted=0`。


## RT-S10：单Browser的当前消费与滚动接续

补充RT-S02/03/04/07，不替换authority或新建通用组件。真实Vampire输入、61个新增ID、原121控件/11事件和图证据见[设计第11节](08-resource-tools-design.md#11-单个真实browser的身份与接入合同)；[input packet](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-real-input-01.json)中live project/catalog/resource/provider revision/ready仍null，format1和旧preview_state不作准入。

| 接续 | 实际消费/身份 | 具体责任与缺口 |
| --- | --- | --- |
| Shared SearchInput | library `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` / component `6ddc248b-bd35-80c5-8008-d6c8c3ce892e` / main `6ddc248b-bd35-80c5-8008-d6c8c0f9c196` / 08 copy `753e5eeb-6e53-8014-8008-d6cb5445fc1c`；native实例化/flow/autosave、关闭fresh重开及7对象sourceRef/link保持，local main仍3 | Parent/pipeline保留query/placeholder props，原asset_browser.zui:toolbar_search_field / SearchEdited / Change→workbench.asset_browser.search.edit；尚无运行时接线、可逆root export |
| Field响应式/guard | 外框832，surface≈240/text≈204，图中内部未伸展，未覆盖额外色块掩盖 | Parent沿penpot-field-controls refreshFieldDecoration/layoutFieldText、renderer refreshAssetTextLayout，绑定完整canonical source/实例/font/allocation后更新parts/text；不放宽guard或据发布映射v1宣称style传播/reset完成 |
| Tile/provider | 原AssetTile component/main保持，本板copy `ff9db784-1073-47f3-b33f-9864612eed7f`；136×184/name32为本实例geometry override | 须从原producer取得同UUID的实际metadata.source_hash/current publication及image decode回执，当前live均unknown，未传虚构props。Grid extent/columns/hit/drag及正式props/slots/catalog登记由Parent接闭包，不用线性28行冒充tile |
| 单scroll owner / RT-S10 | **source-fixed / static-passed / managed+GUI-pending**。Parent仅改原tree/content/reference三个bridge、asset_surfaces/sync.rs并新增tests/scroll_sync.rs；沿原AssetSurfacePointerState将树/内容/references/used_by的snapshot clamp结果交paint owner并经原UI写回，没有第二store。原[三文件r02候选](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-real-candidate-scroll-writeback-proposal-r02.patch)及失败保留，未原样套用 | Parent的rustfmt/diff/五路径guard通过，digest0b465ecc…；五个纯内存回归覆盖缩短/空、两profile/resize、手动scroll与geometry代次保持、当前thumbnail hit，但尚未执行Cargo。原request/operation/identity保持，两次capture失败保留。selected reveal/首尾/HomeEnd/focus-return/旧async结果拒绝及正式GUI另验，08不重叠修改五路径；见[完整状态](08-resource-tools-design.md#parent第三次接续滚动写回与动画合同) |
| 原source/正常产品 | 原四ZUI v2的ID/import/binding/event/slot/hash不变，完整映射已交，不改catalog/tokens/Starship/plugin/producer | 公共owner交Browser/Drawer/AssetWindow受管source closure、当前bundle/EXE/review-host/normal producer；Parent准入真实project/snapshot/provider并排桌面/DPI；保留原验证授权和capture请求身份 |

copy sizing override由原生update-layout-child记录为实际`layout-item-h-sizing/layout-item-v-sizing`，不猜造枚举。首次辅助检查错选runtime编译符号、无事件保存；只读探明真实导出后接续原native入口，失败保留。业务metadata用本域`zircon-ue-assets`，公开source metadata原样保留；不混充官方codec。

rev47轮只证明一项真实跨文件原生消费；后续四主件试接见RT-S11。公共styles/完整框架/全族设计、托管插件/Rust、正常Engine/144DPI、双端逐图及1 DIP比较仍pending，passed/accepted=0。

最新只读API再核本域rev57/Shared rev1、正式link及原Search7/7身份；整份own raw SHA与既有fresh读回一致，没有新写入/视觉验收。RT-PV01…06的真实provider消费细化见[第15节](08-resource-tools-design.md#15-jenkins-r2原身份查询与真实texture-provider接续)：已有Texture SourceImage/192²缓存及原refresh job源围栏；generator目前builtin-only、Operation未派发，item snapshot缺source_hash/preview_state，非空path不证明image-loaded。这些是原provider/projection/host的精确依赖，不是新公共props/DTO或第二registry/store；SourceImage缓存、GPU预览及可打开Toolkit分开验收。

## RT-S06 补充：CS-XR跨域导航

消费角色[第15节](character-skeleton-design.md#15-九族归属与真实资源关联打开合同)和原CS-XR01-01～09，具体当前Browser链见[设计第11节的接续](08-resource-tools-design.md#cs-xr-接续真实跨域导航及焦点返回)。[本域回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-cross-domain-navigation-01.json)是静态源/业务合同，不是新公共schema、registry或运行API；9项全部未运行、31项live结果null、passed/accepted=false。1+9个实际GLB源绑定已独立核对，角色原13指纹与joint顺序保留，不把源label合成runtime UUID。

| 精确提案 | 既有owner/入口 | 需要的接续；本域不实施公共写入 |
| --- | --- | --- |
| RT-XR-D01 单一indexed authoring type | Parent现有catalog；`asset_workspace_state.rs::asset_type_id_for_locator/asset_type_projection`、`AssetTypeProjectionSnapshot`；既有`AssetCatalogRecord.meta/direct_references` | 将真实导入元数据/关联解析出的语义AssetTypeId由同一catalog authority发布，供Browser展示、picker资格及OpenAsset复用。现有开放AssetTypeId已可承载；不新增ResourceKind分支、显示名/.glb判断或窗口私有lookup。未知/多义保持不可确认；具体canonical ID由Parent登记 |
| RT-XR-D02 Registry/Toolkit贡献 | Parent与域owner沿原`AssetTypeContribution/AssetToolkitDescriptor`、`enabled_asset_types_for_shell`及现有field owner检查 | 真实document/command就绪后登记对应view_id/open_operation/required_capabilities，保留唯一类型字段owner。08与角色不各自抢通用mesh Toolkit。实际enabled registry generation、definition owner、descriptor、command ID/when/能力及host结果需来自当前Engine；不猜不存在的view ID |
| RT-XR-D03 导航结果与当前focus | Parent沿原`AssetToolkitOpenRoute/execute_asset_event/open_asset_document_view`、EditorManager/host焦点与native source window | 原route严格两字段不变，不塞未定义props。沿现有host/event证据返回真实instance、metadata写入、domain document/load和focus阶段结果；来源上下文由原host持有，close后按当前project/document/reimport generation解析stable control/对象。focus_view bool是changed而非输入焦点成功；partial-host失败如实报告，不另建focus manager |
| RT-XR-D04 node/skin/primitive关联 | 角色供真实subject；Parent资源/Runtime authority；既有SkinN Data和`SceneMeshPrimitiveBindingAsset` | 原revision内消费skeleton_asset/joints/inverse_bind_matrices与逐primitive材质；Knight九项完整。MeshSkinAsset矩阵与per-mesh首skin不足以绑定多实例，具体持久DTO缺口交Parent，不在08另存映射、改CPU importer或复制preview |

ActivateReference保留Find/选择导航语义，显式编辑关联资产才走真实OpenAsset。missing/unindexed/unregistered/no-toolkit/disabled/capability失败必须保留OpenAsset前来源document/selection/preview及Browser状态并显示真实诊断；不预先Find再失败，不用状态文字假装打开正确编辑器。已有初次Browser点击选择不等于OpenAsset额外改动，前后状态按明确边界记录。

关联Character→Material→Texture由原host保存来源并按实际close提交返回当前有效来源；cancel/dirty-save未决不算关闭，不按旧行号/旧generation抢焦点。Parent同一PreviewSession保留subject/consumer/frame身份，关联编辑不新建角色播放world；最后消费者解绑/关闭和设备失败由Parent生命周期接续。meshopt的两文件最小CPU修复归角色原授权，不能提升任何一项资源ready/preview/EXE验收。

## RT-S11：四个已发布公共主件的受控试接

Parent当前发布Shared file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` / page `b4a1e3ce-9e6a-81fe-8008-d69d607aac13` / rev1。正式库链接及四原生linked copy只落08受控board `fb52acb4-3e9c-47cd-98ef-d6d6f1540b11`；源file/component/main/copy/33条shapeRef/覆盖明细见[设计第13节](08-resource-tools-design.md#13-四个已发布公共主件的受控消费)与[实际源/slot/ref回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-components-trial-01/validation-r02.json)。未写Shared/其它域/公共catalog/tokens/Starship、未detach或另建本地main；旧3业务main、旧Browser公共Search及1224个非根对象全部保持。

| 消费范围 | 已实际证明 | 原public owner/Parent仍需交付 |
| --- | --- | --- |
| Button | catalog有text/button_variant；native文字覆盖、原生reset清除touched、再次覆盖Import，独立fresh保存保持源身份 | canonical源闭包、正式property/export映射和真实Import命令；未用文字reset充当global-style reset |
| SearchInput / RT-SH4-01 | query/placeholder源props存在；中文文字覆盖及1232宽root fill保持，surface仍240 | 沿原field renderer/guard修复内部响应式、SearchEdited/query及placeholder导出；不改baseline掩盖source差异 |
| TreeRow / RT-SH4-02 | 原main20条sourceRef、默认文字/可见性/锁定装饰完整；当前catalog无公开props/slots | 提供Browser所需text/depth/expanded/selected/focus及适用actions的正式合同，复用单一scroll/selection owner；不能静默删除decorations或发明props |
| PropertyEditorRow / RT-SH4-03 | required单个linear value槽及4 sourceRef保持，未放入不合规内容 | 发布允许的Checkbox/Dropdown/Field/NumberField/RangeSlider/Slider/Toggle之一及真实slot/export闭包；当前4主件均不合要求，空槽不作已完成属性面板 |

五对实际请求/ACK按完整file/session/changes匹配lagged条目，最新独立重开rev55全部1262对象相等；初始建板/实例ACK漏捕和早期路由/locator失败均保留，不称连续55次原始保存全覆盖。四根canonical-record均未见sourceHash；磁盘源hash是读取证据，不能自行补成owner sourceHash。Shared colors/typographies仍0，尚无两族全局style A/B、canonical slot/override/export及真实Engine加载。三张headless Penpot诊断图已逐图核对，当前71%画布图不是原生PNG导出/正常EXE/DPI/双端差异证据；passed/accepted仍false。本阶段没有Rust变化或编译，原Audio/scroll受管请求不重投。

## RT-S12：Picker/Import的modal与真实能力边界

消费Parent followup-03对RT-S05的实际源核查。[WorkbenchDialog源](../../../zircon_editor/assets/ui/editor/workbench/primitives/feedback/dialog.zui)和[组件catalog](../../../zircon_editor/assets/ui/editor/components/catalog.toml)均无body/actions槽，只有Dialog根及popup/focus声明、popup_open/title/message三个公开props；声明不证明实际焦点行为。当前两个本域可编辑内容板不构成可复用modal、原ZUI导出或产品集成。读取指纹、真实来源及取消边界见[source-capability](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/import-picker-capability-01/source-capability.json)与[设计第14节](08-resource-tools-design.md#14-parent回执接续与importpicker真实能力修正)。

| 消费需要 | 当前事实 | 精确依赖及责任 |
| --- | --- | --- |
| 内容/动作承载 | Dialog只有一个root；没有已发布body/actions或允许类型 | Parent沿原overlay/runtime挂载能力核实内容/动作如何承载，再交源字段、允许类型、slots和导出合同；保持根ID及popup/focus声明。08不新增并行modal、复制公共main或传未声明字段 |
| Picker确认/取消 | WOC crate_wooden.glb实际318100字节、有源URI；此次没有当前项目catalog/typed资格/runtime UUID | 原registry/type authority提供真实project/activation、asset UUID/locator/revision和allowed type；未知/disabled保持不可确认。调用者属性在确认事务前及取消后保持；实际close提交后由原host解析当前generation来源并返回焦点 |
| Import来源/目的地 | 项目内源保留实际res:// URI；仅外部.obj/.glb到首root的models目录，同名拒绝 | 复用submit_model_source和source_plan；不得显示任选目的地/Import All/FBX options为现有能力。当前project/index activation/model backend/job ticket真实准入后才能提交 |
| 取消/持久化/history | job进入Runtime前检查取消；Runtime调用未接收该cancel token；成功durable receipt不能被迟到取消抹去 | 原job/Runtime owner提供真实阶段/结果。不能声称Runtime中途已停或已回滚；成功仍刷新catalog。Editor Undo与Runtime rollback分别接原事务合同，未交付继续open |

本轮只在原Import/Picker的四个text写入上述限制，原有确认按钮继续不可用，没有把磁盘文件名/扩展名/Penpot media当runtime UUID/type。动画[RT-S08/09](animation-shared-controls.md#资源消费合同-rt-s0809)的泛型Curve、typed资产/codec/history及per-playback Sound分工已收件；公开context/slots/StopSeek仍待实施，六条原route与无capture Record保持。树/内容/引用/被引用四state滚动写回继续由Parent原owner承接，不套用旧三文件r02。

RT-TEXT01给原公共pipeline owner的可复现观察：仅用native update-text-with-function改content时，服务端rev56已是新文字，但旧position-data仍让实际SVG显示旧字形。此次依Penpot现有await-html-font路径，先对一个text通过native update-shapes取消派生position-data，确认真实字体重排与SVG后再处理余下三个；rev57保存并在新browser复核。没有修改renderer/guard、手填字形坐标或更改字体/几何；全部公共实例保持。公共导出应沿原source/parts/guard核文本缓存与真实字体重排，不能只读content即判视觉通过。原失败图、修正图、两对严格保存ACK及r04离屏locator失败/r05逐板可见核对见[校核回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/import-picker-capability-01/validation.json)。这只是本域native设计修正，未实施公共pipeline修复。

官方插件导出/加载、真实provider/命令、托管Rust与正常EXE/144DPI/双端逐图仍pending，passed/accepted=false；不新增桌面时段或重投原受管请求。


## RT-S13：SoundWave源内容与公共消费边界

原file/page实际rev76，SoundWaveSourceContent草稿v1 component 6ff61186-040c-804f-8008-d6f2f4d0ed80 / main 98a02ddc-755b-4643-b6c4-fae1699875c4；900/640 copy48/48 refs、19次保存、新browser重开已核对。[第16节](08-resource-tools-design.md#16-soundwave真实源内容与原生组件)记录三board/copy、75新增对象和真实MP3能力，**不是公共发布或产品source codec**。

| 提案 | 原owner与精确消费 | 当前边界 |
| --- | --- | --- |
| RT-S13-C01 | Parent主题/source-owner沿已有tokens/Starship发布waveform/plot/major-minor grid/ruler text的canonical映射 | UE White/thickness1、linear(.02)背景/Blackgrid/白.9文字10字号，与草稿C0/303030/101010不同；Shared rev1零styles，不造style UUID或本地平行主题 |
| RT-S13-P01 | Parent原registry/importer/provider；08适配SoundAsset uri/rate/channel/samples/frame/duration | MP3等真实Symphonia importer存在，但有效Editor admission/Engine UUID/catalog revision未观测。按实际project/source hash/revision/generation绑定只读snapshot；重导入/关闭后的旧结果拒绝，不启用未知UUID |
| RT-S13-P02 | 动画中立Transport与Parent sound output/provider；08提供业务asset/session | PCM不是CurveKey。直接消费RT-S08/09真实位置/capability，保留6events/routes、非文本快捷键、无capture Record及close/current-generation。Stop/Seek公开参数/slots须正式diff，无ready不造播放控件/停全局设备 |
| RT-S13-Z01 | 原public source codec/producer、Parent Toolkit/host；08仅业务内容 | sound_wave.zui仍提案，原drawer/mixer不是该文档，本地metadata非public props；需正式source/component/version/main/copy/shapeRef、原bindings/events/style/slots/export与load回执，不改原四ZUI/ID/events或复制公共main |

copy保留本地数据几何及640 URI显示覆盖、raw URI未变；自然path与六条向量已验证，任意后续main/style传播/reset仍未接受。dB ruler/Processing/Toolbar、全状态/history/reimport/Engine播放与正式双端≤1逻辑像素仍open；未知runtime/capability保持null，不当false或ready。

分工保持RT-AUDIO01：08是SoundWave/Cue/Class/Submix业务owner，材质供中立graph，Parent接sound runtime/provider、公共host/跨文档，动画供共享Curve/Transport。本轮不扩大Runtime/core/公共源码写权或桌面时段；原Audio Jenkins recipe仍等原owner收件，passed/accepted=false。


RT-AUDIO01-J01最新管理入口只读实查为17:01:30Z的d2873ff3…（11/11+197/197一致），增加particles但仍拒原Sound recipe/build_scope_policy；原请求与历史QueryOnly保持，不重放或执行Cargo。新证据已排队原Jenkins owner、尚无实际准入，见[本轮回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/jenkins-scope-refresh.json)；本域真实provider/正式导出/EXE验收继续open。

## RT-TYPE01：typed文档与真实provider消费

按Parent followup-04实际Open既有19002插件，83条registry/资源rev76/Shared rev1保持。四main/零styles与Property.value合法输入限制仍在，canonical version1只是记录schema。[设计第17节](08-resource-tools-design.md#17-已安装插件open与typed资产合同)、[32份源码能力](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-capability-typed-02/source-capability.json)和[7种typed提案](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-capability-typed-02/typed-contract.json)可读回；没有注册类型/格式/command或生产写入。

| 合同 | 精确需求与owner | 原源边界及待验证项 |
| --- | --- | --- |
| RT-TYPE01-P01 | Parent原sound provider保留真实Kira/fallback位置来源与session/source/backend代次；08消费 | 当前DTO丢来源，保持unknown；不自行改公共DTO、造设备位置或timer。需真实分支/seek/backend-loss回执 |
| RT-TYPE01-P02 | Parent原完成事件消费者按原session/playback lease分发，08只消费本实例并停止拥有的ID | 原drain排空全局；不得新建manager/queue或抢drain。Stop返回/终态分别核；设备退役也可Stopped，UnknownPlayback不能定成败。manager无unload_clip，不清共享cache/output |
| RT-TYPE01-P03 | f32 seek如实保留请求/round-clamp/readback与integer源frame | 不承诺任意长度sample-exact，不用动画FPS/覆盖六routes；Stop/Seek仍待正式公共diff |
| RT-TYPE01-P04 | Parent原provider/host关联原命令返回、playback/clip/track lease与终态产生时authority/generation | Stopped不唯一归因于显式Stop；private output_generation从0开始，Finished/公开output status不带它，不能用drain-time当前代次补旧事件。保留typed error/实际终态与当前opener，不新建DTO/queue或猜focus |
| RT-TYPE01-T01 | 08牵头Audio四类/独立Curve typed模型/codec提案，Parent接单一registry/document history/toolkit host | kind→type/DataAsset不作typed准入；没有格式/type ID，不按后缀/显示名路由，不新建伪资产/平行registry/跨文档writer |
| RT-TYPE01-T02 | 复用实际PersistedAssetReference与原resolver | URI/文件存在非Engine UUID/admission；missing/wrong-kind/disabled保留来源，关联Material/角色正确owner，需真实打开/host/focus结果 |
| RT-TYPE01-C01 | 动画owner负责共享表达，08保留逐键rich源文档、1/3/4通道与typed history | 现整条插值/非加权切线不保真逐键mode/weight/default/extrap；无法表达时只读，不复制primitive/丢字段 |
| RT-TYPE01-D01 | 08表格业务与Parent原document host接真实schema/row/reference事务 | 当前validate/save反馈Not implemented (preview)，sample controls不作完成；保留原ZUI/IDs/events |
| RT-TYPE01-Z01 | public owner复用已安装19002 entry和四原owner，交source-closure/font/icon/media/codec/正常producer | 本次只有Open；不运行旧runner的listen/Install/Allow，不把raw metadata当family闭包 |

动画instance-scope r01 diff已实际读并固定SHA、未应用，proposed operation/context不冒充公开props/API。沿原journal核current publication/node/binding、真实project/document/source/backend/capability及request sequence，不建player/clock、不停共享设备。单gesture inverse/Editor Undo、durable之后Cancel与current-generation focus均待真实事务验证。公共修改仍由Parent/owner实施；本域本轮Rust/ZUI改动0，托管Rust与正常EXE/DPI/双端逐图未通过，passed/accepted=false。

## RT-AIREF01：AI消费的资产只读观察

以下保留第五次交接的原提案事实；G01后续已获Parent授权并形成[单API代码候选](#rt-airef01-g01代码候选)，因此下文“没有新API”只指原提案时刻。真实type/document关联、覆盖和原子写入仍未就绪。

Parent AI第五次交接已收。详见[设计第18节](08-resource-tools-design.md#18-ai黑板消费的资产只读provider提案)、[source-capability](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-readonly-provider-01/source-capability.json)与[provider提案](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-readonly-provider-01/readonly-provider-proposal.json)。33份源/8项事实与真实Vampire样例可复核；没有provider实现、新API、引用索引/schema/history/core/catalog写入。

- 资源08只提供既有GUID/AssetRef/sub→locator/runtime kind、实际semantic type及来源策略、原catalog/input观察代次和已有UUID依赖边。known/unknown/missing/stale与coverage、observed rowcount、完整total分别报告；当前coverage和完整total未知，裸空Vec不能证明零引用。
- Parent原DefaultEditorAssetManager已有project/index/catalog一致发布与锁顺序先例，公开catalog Arc没有activation/input proof；G01在原owner接只读adapter，不能用Model import专用receipt冒充AI查询。G02由Parent原host交真实asset↔document关联，DocumentToolkitSnapshot generation/descriptor不含asset UUID。
- G03由原index/provider声明资产边scope，AI负责BT→schema资产、node/parameter语义及provider闭包。未绑定schema、unknown provider、只读/缺失/stale依赖均拒绝rename/type/delete；不按同名key、title、payload或运行schema业务ID推导关联。
- source policy-writable不等于OS权限/文档锁/AI引用/保存准入；SaveDirtyViewCandidate两个默认true不作事实。只读provider不调用save/validate/autosave/refresh/import，不构造save request。
- G04由Parent接真实跨文档history/原子提交与保存准入，AI供业务inverse。SaveDirtyViews仅队列准入与逐文档完成，单源writer无多文件原子保证；不循环单文档save、热更schema或清所有dirty。

该提案没有接管AI语义或运行schema，没有新增桌面/服务需求。本轮12个验收场景未运行，不触发Rust编译；沿原任务保留official source/provider/EXE/DPI门，passed/accepted=false。


## RT-S14：SoundWave的UE电平标尺

[设计第19节](08-resource-tools-design.md#19-soundwave的ue电平标尺与原生同步校正)已将UE MidSplit depth3九条水平网格与dB标签落实到原SoundWave业务main和两份native copies，原component/main/sourceRefs保持。当前08 rev80、1364对象、27新增文本，832/832 refs有效；关闭后新browser重开一致，3grid+3PCM投影<0.01px。数值源和label2px距离来自实际UE源码，不是运行中SoundAsset或播放位置。

原native自动同步曾压缩copy路径并移动时间标签，两个失败和恢复保留。最终通过原保存对象恢复原包络/时间标签，原1265非SoundWave对象保持；今后主件结构/样式变更需重验全部copy的PathData和text geometry，不能只核main或object数量。

Shared已实际读到rev2/34colors/3typographies，UUID/源字段在[只读观察](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-value-grid-02/shared-style-observation-r04.json)，旧rev1零styles只作历史。本轮仅保留当前palette，不修改公共库；RT-S13-C01的UE paint/text-measure、实际source-slot/codec、全局传播和EXE仍open。动画generic Curve/Transport及六routes不变，无新公开props或播放器。逐图仅Penpot canvas诊断，原托管Rust、EXE/144DPI、双端1px未通过，passed/accepted=false。


## RT-S15：Shared rev2的同步与非空样式引用

已接Parent发布37个styles及原4mains。详见[设计第20节](08-resource-tools-design.md#20-shared-rev2同步事实与字段合同接续)。原5 linked copies原生sync-file已保存rev81；40节点的component/main/sourceRef、文本及原覆盖flags保持，但触发25个几何/测量变动，含10个非Shared目录文本框，检查未通过。21个master ref字段实际refId/refFile均null、非空style UUID=0；不将字段存在数当绑定，不把已发布styles重新报空。

[原对象恢复清单](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/native-restoration-plan.json)及[原linked Button的body/primary显式消费计划](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/consumer-style-trial-plan.json)已准备；r04打开页面即19001拒绝连接，0恢复/消费写入，无新截图/重开。原Shared/主题/源guard未改；真实服务readiness由Parent/原owner接续，本域不启停服务或抢桌面。native-sync的失败不改变官方codec/EXE门，passed/accepted=false。

### RT-S15接续：已恢复且单实例样式持久化

18:30:27Z新进程重开原08 rev84/1364对象/4业务main/26copy，832 source refs全部有效。原25处同步重算已恢复，旧失败保留；相对rev80只原Button Import文本显式应用发布body/primary的真实UUID/refFile（段落/run共8字段），原copy/component/main/shapeRef/已有覆盖/guard保持。全部对象与r06保存相同、0新写请求，Shared rev2及83-entry registry前后hash一致。[原生保存](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/native-consume-r06.json)、[新context重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/fresh-reopen-r08.json)、[八图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/individual-review-r08.json)。

Shared master实际非空引用仍0，这次是本地显式消费，不是默认传播/reset。发布body使该受控Button由700变400、provider改gfont-roboto；Roboto DOM可见不证明CJK/Engine字体或UE Button默认一致。公共owner仍需补原master/source/正常producer；本域不改Shared/guard或复制main。Browser长文件名/类型重叠RT-G29与Search fill RT-G19继续open。官方codec、真实产品加载、Rust及EXE/实际DPI均未验收，passed/accepted=false。

## RT-C06：资源字段的真实值与四个属性slots

Parent catalog新SHA4c2f849c533f1f227903018eca860faadd7fa3138fdc265300913e3833ad36e5实际匹配，NumberField10props和PropertyRow4slots已做6项有界静态读回，见[原字段合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/catalog-contract-readback.json)。这是新增公开源候选，未发布新输入master、无Rust/插件或产品验收。

[业务消费提案](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/field-slot-consumption-proposal.json)列9组真实绑定：精确原文/typed值、schema范围及单位步进、可编辑准入、required value的真实输入、description/source和真实reset/history。旧DataTable Version Field及两events保留，不把12或42/0..100当当前项目值，不经f32保证u64/i64；缺typed codec或scope则保留原文本/只读。value仍required single linear及原7accepts，Label不合规。公开字段≠公共master或backend能力，无新registry/schema/history库，应用仍pending。


RT-C06补记：4c2f849c…为本轮最初读回；末尾实际catalog已e4bba09c3ba16d55b90eabb5a821ab3e76eb0606f9afc17e9f656e2d16fe0a99。r08全文件不变检查失败保留，r09只读对账仅证实上述10props/defaults及4slots/required value和另外8源保持，不把其它公共合同变动或新master当已验收。见[精确范围对账](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/catalog-contract-reconciliation-r09.json)。


## RT-S16：Browser名称与类型的测量一致性

原AssetTile copy保留全文/原字体/136×184几何和所有main/copy/source refs。RT-G29实际SVG先证实4.99658逻辑px重叠，原type的position-data比当前frame滞后；仅经原Penpot update_shapes失效该缓存，让原生重新测量，保存rev85。新进程重开全部1364对象一致，832refs有效，0写请求，实际glyph gap6.53270px，六图已实际复核。[设计第21节](08-resource-tools-design.md#21-browser类型行的旧测量缓存)和[新进程原记录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-text-metrics-01/fresh-reopen-r02.json)不作为UE/EXE双端或DPI验收。原4业务main/26copy和Shared样式消费保持，无公共master/主题/原ZUI/Rust修改。UE Class14px对当前type12px及正式三尺寸仍open，测量缓存修正不替代字体/source codec/产品布局闭包。


### RT-TYPE01接续：动画资源r02

[设计第22节](08-resource-tools-design.md#22-动画r02与原试听provider的终态关联)已消费r02。25输入24源保持、仅本域合同变化；旧catalog pin差异单列，不重签/应用原diff。位置DTO缺来源、Stop/Unknown及全量drain边界保持；新增核实设备退役亦Stopped、私有output_generation=0有效且未随Finished公开，必须保留产生时来源，不在取队列时补当前代次。沿原host typed Err与真实event/journal关联，不建路由/播放器/clock/DTO/schema。六route/六按钮+Fill保持，Standalone Curve无需Timeline/Transport，typed语义仍08。[提案与六个未执行场景](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-provider-handoff-03/provider-host-contract.json)不是新Jenkins/capture请求；运行UUID/admission/position/capability未交，无新API/图片/解码/桌面/服务/编译，passed/accepted=false。

## RT-AIREF01-G01代码候选

Parent把原§18提案的G01明确分配给08，实际[单API](../../../zircon_editor/src/ui/host/assets/api.rs)及[query实现](../../../zircon_editor/src/ui/host/assets/manager/assets/readonly_query.rs)已落原owner，7产品文件写集之外无扩写。输入为AssetRef、可选expected scope和实际source policy；输出保留requested、可选真实scope及Known/Missing/Unknown/Stale。scope按原gate→state→index核activation/input/registry并捕获catalog，expected以真实activation与Arc匹配；两投影来自同captured catalog，原incoming不被UI row过滤。

调用方须释放自己的外部锁后按实际host上下文调用，query返回前会释放本manager全部锁；此API不接收回调、不执行IO/refresh/import/preview/save。scope不是写lease，异步结果仍须核Parent当前document/关闭状态。None policy不是read-only或writable的实测结果；有policy也只是策略计算，实际semantic type/registry generation、bytes SHA256、IO、锁及save admission独立。依赖边端点Known/Missing不声明extractor完整性，两种coverage始终Unknown，空行不证明零引用。

[设计第23节](08-resource-tools-design.md#23-rt-airef01-g01原manager只读adapter候选)与[candidate-contract](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-readonly-adapter-01/candidate-contract.json)列12项未执行回归定义和真实Data/TOML候选。没有AI关联/schema/runtime注册、真实host接入或live查询；G02/G03/G04继续原分工。无Jenkins新请求/QueryOnly/锁生成或直接本地编译，无Penpot/API/桌面/服务/EXE，本域passed/accepted仍false。

## RT-S17：SoundWave原生copy与可靠PCM基线

[Parent followup-05](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/asset-tools-followup-05.json)已消费。本域新增的[离线比较](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-copy-integrity-01/offline-copy-comparison.json)严格比较原rev80与原rev85两个保存响应：99对象/2,890字段、6条原生tag/bytes及96关键字段、27标签和63文本对象/缓存相同；不声称新API、Parent原hash协议匹配或实际EXE/DPI。没有改动main/copy/sourceRef/local override或Shared。

[原失败对照](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-copy-integrity-01/known-failure-detection.json)说明仅前后diff不足：r02的PCM局部未继续变化，但相对原可靠rev76仍有2/3漂移；恢复rev80和原保存rev85均3/3匹配。公共样式/master更新需逐copy核可信PathData、transform/bounds、时间/来源文字和position-data，保留来源/覆盖；合法业务修改须给准确允许差异和新来源证据，不降低guard或只验main。该消费约束不代公共source codec/family producer实现。

Shared42字段null与Import authoring8+缓存2口径保持；发布34/3不证明默认绑定/传播/reset。G01原候选/12未执行测试保持，不重复派发；原Audio3/scroll5/provider/session/official source/EXE/150%DPI/双端逐图门仍open，passed/accepted=false。


## RT-S18：原owner闭包和已分配页面追加的消费前置

详见[设计第25节](08-resource-tools-design.md#25-原四owner的source-closure与分配页追加前置)及[消费输入](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-closure-consumer-01/consumer-inputs.json)。19:44:46Z真实只读API仍rev85/1364对象、Shared链接rev2、83条原插件注册；官方role=asset/document计数0仅对应append predicate，不表示现有18设计板/4业务main不存在。当前原源码已有通用prepareRealOwnerInput、四资源owner的contract-test定义、existingPlugin/append及原内容保存/重开保护候选；早期固定LevelEditor读数留作历史，本域未运行这些候选或改公共文件。

RT-G32最初19:44Z发现的Transit Set/Array权限与fingerprint缺口保留为历史。原public owner已在原三源修复：权限比较严格接受Set/字符串数组，manifest仍须数组；penpot-plugin-registry-v2保留Set内容/类型、数组及ordered IDs顺序和完整registry，原plugin-id/URL/唯一注册/权限精确核验不变。v1 pin、JSON pluginId别名或marker不回退；必须由同配置受管runner通过原Transit decoder重新只读取得entry及full registry的v2 pins，本域未生成替代指纹。见[限定收件](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-closure-consumer-01/rt-s18-registration-intake-01.json)及[原owner交付](E:/cargo-targets/zircon-local/reports/penpot-unreal-01a0f04d/ue-penpot-resume-20261023/local-execution/public-component-closure-01/source-colors-and-copy-preservation-01/resource-rt-s18-registration/handoff.md)。固定penpot-public-canonical-tests-r03含registration共6spec，但测试/TS/browser/export仍未执行，owner real-session编译配方未列；旧compiled26d2e4fc…不视为三源已部署。05:37:59Z的19002 manifest/worker GET仅为原时点HTTP身份，不证明6ded0449…/348732B与旧442bed70…兼容。原资源file/page append仍须fresh revn/vern，rev85/107均为旧读回；保持prepareRealOwnerInput/existingPlugin/append与非空页/foreign-content guards，不回退LevelEditor。四原ZUI不变，正式import/export/产品加载/样式A/B、原G01/Audio/scroll及EXE/DPI继续open，passed/accepted=false。

## RT-S19：真实目录箭头与现有TreeRow接入

用户截图要求已在原08页落实为真实UE caret-right/down原生SVG PathData，见[设计第28节](08-resource-tools-design.md#28-用户要求的真实目录展开箭头)及[机器合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-arrow-vectors-01/parent-contract.json)。最终92/1666对象保留原4业务main；151行含31展开/120折叠，16×16 s档+gap4 row flow，70实例各新增两条真实shapeRef，972/972 refs有效。未新建公共TreeRow/main/theme，四原ZUI及原IDs/events/bindings未写。关闭重开92与151原生对象/19可见path已实测，20张设计图逐图打开，产品/EXE/实际DPI仍未验收。

Parent/public owner接两项精确缺口：RT-G33保持原Icon/expanded、control/hit身份，将公共SVG paint/state映射到原source closure与命名s档；不用raw metadata或字符前缀替代。RT-G34当前AssetFolderSnapshot没有expanded/has_children，source_tree_row_node只投影depth/selected；真实展开须原tree state按folder_id/current publication接入，不复制store或猜显示名。已有TreeRow painter会选真实打包SVG；设计状态不算运行展开通过。

Shared20:52Z实际rev5/4main/34colors/3typo，Button一个fill ref pair现非空；原UUID保持，其余默认继承/reset仍待验。消费者synced-at仍18:07:13Z，本轮只读来源不sync。原rev2/零非空绑定是历史。首次native重排、缓存/图层名标准化、固定DOM数量与ink-vs-frame断言以及恢复均保留；最终仅原302目录对象变化，1062无关原对象和SoundWave99完整保持。官方source/export guard未改，export/load、provider、Rust及EXE/DPI仍pending，passed/accepted=false。

### RT-S19-r02：目录展开不是图标字段替换

本次[真实源码合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-disclosure-contract-r02.json)修正r01“接现有tree state”的过宽措辞：已有primary AssetWorkspaceState，但没有展开集合；当前folder_tree无条件递归所有后代，pointer layout只有folder_ids，所有Folder命中经原SelectFolder，没有独立disclosure路由。Parent需在原owner接真实展开意图、可见preorder及当前publication，不能仅新增expanded字段、图标旋转或第二store。

UE SExpanderArrow是非focusable的MouseDown按钮，Shift递归，无child隐藏但保留槽位；SPathView 2278—2303在收起包含已选后代的祖先时把该祖先加入selection，避免Content Browser因空选而显示全部。普通展开/无关收起保持selection，选中后代被收起的情形另按此规则处理。完整引用与边界在[设计第29节](08-resource-tools-design.md#29-目录展开状态与命中路由的真实接入边界)。

Parent/public owner仍持有Workspace/snapshot、原SelectFolder及兼容event、canonical TreeRow/paint/hit几何、真实modifier/focus与source闭包。tree_disclosure_rect目前仅paint可见；命中须复用同一几何或实际发布rect，不复制算法。真实project activation/view实例/folder_id/current publication由原owner提供，surface_mode字符串和行号不作业务身份。leaf须由当前父子关系判定，不能用资产计数或设计31/120状态猜测。保持RT-S10单owner和clamp写回，reveal须显式意图，旧异步结果不抢焦点。

RT-TD01—14是必要回归定义，0执行；原Audio3/Scroll5和G01十二项也未新增运行。rev92可编辑SVG设计、原4ZUI/IDs/events/imports/bindings、G01七产品hash和Parent滚动源保持，未扩公共写权或创建验证请求。source/common实现、正式导出、EXE和实际144DPI/双端仍pending。

### RT-S19-r03：已知磁盘叶子在Penpot隐藏图形并保留槽

原页实际保存重开93，151原生SVG路径保留，其中31展开/35折叠分支可见，85叶子path隐藏；16×16父frame、gap4、目录文字/缩进/选择与972 sourceRefs未改。叶子依据WOC/Vampire当前34个真实目录的child-folder名单；完整[设计第30节](08-resource-tools-design.md#30-真实目录的叶子箭头可见性保存重开93)、[源保护](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-leaf-disclosure-01/source-checks.json)、[14图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-leaf-disclosure-01/individual-review.json)保留精确字段与native ACK。BrowserContent只有原生modified-at元数据更新，其余组件定义和PathData不变，首个过严timestamp检查失败保留。

这是原业务main及copies内的可编辑设计状态，不是新增公开has_children/expanded prop、正式codec或Engine registry接入；后端必须使用当前真实catalog的父子关系及publication，不能把本轮目录列表/151图层写入第二store。公共RT-S19-r02/RT-TD01—14仍open，640 reveal、900 tail、中文、source导出及正常EXE/144DPI均未通过；原事件与ZUI未改，不新增Jenkins请求或桌面需求。

### RT-S20：中文原生测量缓存与作者字段边界

原08页保存关闭重开94，仅11个中文shape的position-data由原Penpot DOM measurement owner重建。原Noto Sans SC font ID/normal-400确实loaded，缓存宽度与同字体自然测量及DOM textLength一致；字号、文字、框、sourceRef、touched、4组件定义和151原生箭头不变，不新增公共prop/style/master。[设计31](08-resource-tools-design.md#31-中文缓存宽度由原生字体测量重建保存重开94)、[原保存](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/chinese-text-metrics-01/save-receipt.json)、[独立重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/chinese-text-metrics-01/reopen.json)及[6图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/chinese-text-metrics-01/individual-review.json)保持原file/page，当前对象1666/refs972。

公共source codec/字体导出owner后续须保留真实content/font、相应位置缓存和原source身份；不应按缓存错误反向缩小字号/文本框。此次仅证明已加载Penpot字体下的缓存修复，未证明引擎字体、producer/style传播/reset、官方导出、EXE/DPI或双端行框误差；不修改公共源码，不用raw metadata或18项静态检查充当family通过。原导出和Host准入责任保持。

### RT-S21：Browser历史状态能力与原宿主接入

[原生设计32](08-resource-tools-design.md#32-browser历史导航真实ue图形与生产合同)只把12个原对象改为实际UE CircleArrow path+circle，保持原ID/框/parent和来源；原file/page保存重开95，4业务main、992 refs和24个真实Shared stroke refs已核。此图形没有新增或假装已发布的history prop/route，原ZUI121 controls/11events保持。

[真实接入点及10个未执行场景](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-history-icons-01/runtime-history-contract.json)固定Parent拥有原AssetWorkspaceState及snapshot、AssetCommand/codec、AssetHostEvent/dispatch、editor_event_normalization、asset_event执行、builtin bindings与原ZUI/source closure；08不写这些公共路径。当前没有history/cursor或后退/前进意图。UE目标是ContentSources+selection的300项历史；先更新当前entry再移动/恢复，分支导航截断future；来源描述用于tooltip，ThumbMouseButton/2也走相同真实owner。它不等于asset/document Undo。

Parent需先交实际activation/view实例隔离与当前history publication/canBack/canForward/target描述，再定义受控意图及应用回执。旧代次/同根重开、缺失或重用locator/UUID必须返回真实状态；失败保留来源，成功后经原selection/single-scroll-owner/reveal及当前native focus generation。文本/IME/popup优先，不让旧ACK抢焦点，不按同名/字符串或单全局state冒充多Browser历史。当前单Optional UUID选择不足UE集合，不能偷降多选。原图库颜色只沿灰色能力未知状态，不能据此启用Back/Forward。

RT-NH01—10是验收定义，0执行；SVG source精度及23静态检查不是产品Rust。已有RT-S10/19、modal、registry/provider、原source/export/load、EXE/144DPI与≤1px双端门保持open。Shared版本5的显式stroke refs只是此本域消费，不宣称公共传播/reset/主题闭环；邻板低细节thumbnail旧字符也记录为未闭合producer差异。


### RT-S22：DataTableTab业务内容与原共享输入/宿主

[设计33](08-resource-tools-design.md#33-datatabletab内容原生设计与真实业务缺口)在原08文件保存重开100；唯一新增本域业务component dc042d85-e62f-4025-9680-10ee1619c9d7/main a7972f31-e63f-4359-8e6d-3e871ebb2d8d，2native本域copies和3真实Shared Search copies的sourceRef保持。四原ZUI样例非真实typed资产，未建通用表格/输入主件，没有新增全局theme/样式/registry/command/slot或改原events。

[具体接入及12未执行场景](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-source-01/runtime-contract.json)区分五项公共依赖：P01原Table/list/column/hit/scroll合同；P02 Search全parts响应式及SearchEdited正式导出；P03 Property required value与已发布合规Field/NumberField/dropdown/typed binding；P04 CellText12px与UE选中Orange/linear0.075及真实偶奇brush源映射；P05原Toolkit/Dock/RowEditor per-document/view/publication宿主、菜单IME/focus及source闭包。Shared归档rev5的body/caption及颜色是真实引用，本域显式font/size覆盖及两个源paint不作公共发布；最终全图的库更新通知未执行。Search内部240/204缺口仍交原owner，不能局部伸展冒充闭环。

08负责真实typed table文档/codec/行字段业务语义提案；Parent保留原registry/toolkit/document/host及跨文档history/CAS/atomic save，公共原owner供组件/源producer。身份须真实GUID/sub/type/catalog+document source，schema关联未知时只读；行号/过滤排序索引非持久身份，row-key/rename方案待真实codec，不造UUID序列。保留精确类型/未知字段，i64/u64不能经f32保证；source policy不是I/O/lock/save admission。能力不明不启用写入，popup/文本/IME优先，晚到ACK不复活已关闭view。未建新store/index/history或runtime服务。

源feedback仍Not implemented (preview)，RT-DT01—12全部0执行；原Rust、官方export/load、实际EXE/144DPI、双端比较和≤1px均open。当前可编辑Tab内容不关闭RT-G11/RowEditor/Undo/persistence，也不声称新字段或runtime已经发布。

### RT-S23：原Shared Button有限消费证明与manifest CORS

[机器合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-button-native-01/runtime-contract.json)只覆盖原验证板同一Button consumer。Shared file...ac12/rev5，原component **6ddc248b-bd35-80c5-8008-d6c8c01096f3**、main **6ddc248b-bd35-80c5-8008-d6c8bd9b057f** 不变；本域root **5698dd90-efc9-8032-8008-d6dd4c845807** 与label ...5808，均保留原shapeRef、source/几何身份，没有detach或main复制。

真实sync保存101保留Import，local fill覆盖102绑定surface.0并真实touched；完整reset103恢复surface.1/Button并清原文字覆盖；两处说明104由原生测量更新。四完整ACK、17静态/source检查及fresh reopen104/0保存/registry不变均在[设计34](08-resource-tools-design.md#34-单个shared-button原生同步覆盖完整重置与关闭重开104)。这是单linked consumer的持久化/覆盖/reset证据，不证明整库Updates、全局样式A/B、官方codec/export、Engine样式或已接命令。

| ID | 原owner与具体差异 | 状态 |
| --- | --- | --- |
| RT-S23-P01 | Parent/public owner核原19002部署host：实际19001浏览器CORS拒绝，独立Origin GET200/278B/原manifest SHA但无ACAO；沿原registered entry修正支持的origin并给真实浏览器回执，不新host/Install/关闭安全策略 | open，本域只交精确观察 |
| RT-S23-P02 | 原public owner完成4原ZUI owner白名单/source closure、原source/IDs/imports/events/bindings和native style/reset codec，stale/unsupported应拒绝，不以raw metadata或单consumer检查关闭family gate | open |
| RT-S23-P03 | canonical SearchInput内部stretch、原PropertyEditorRow required typed value slots，保留原4master/source guard；不造Label输入、实例像素覆盖或平行主题 | open |

全部16张PNG已实际打开，其中一张空workspace失败图保留为失败；DPR1/71%不是EXE/144DPI或双端≤1px。没有新增运行测试、Jenkins create/QueryOnly、host/服务/桌面操作；原typed/provider/audio/scroll/G01和正式export/load/EXE验收不变。passed/accepted=false，释放0B，cleanup未关闭。

### RT-S24：Static Mesh源分组、嵌套Search与原宿主合同

原08文件保存关闭重开107，新增本域StaticMeshSourceContent component c2b1f915-4203-4a85-b433-152c4b6f13f5 / main 37a2a346-caa1-4820-b1f2-5de79d8f8095，两own copies及3个原Shared Search nested copies。1200 refs全部有原来源，near-instance refs保留，不detach/复制公共main或编辑Shared。详见[设计35](08-resource-tools-design.md#35-static-mesh源内容与原生三尺寸保存重开107)及[具体合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/runtime-contract.json)。

实际GLB只读诊断没有asset UUID/catalog admission。现有gltf importer明确#Mesh0是Model、#Mesh0/Primitive0与1是Mesh，node scene instance原primitives vector分别绑Material0/1；顶层第一material不能替代两条。ModelPrimitiveAsset本身不含material，08保留源node/mesh/primitive/revision与原SceneMeshInstanceAsset绑定，不造新模型或按.glb/显示名/0skin判static/skinned Toolkit。候选URI非实际registry AssetRef；未知保持来源，相关Material/Texture跳转仍须真实registry/command/Toolkit/host/focus回执。

| ID | 原owner及精确依赖 | 状态 |
| --- | --- | --- |
| RT-S24-P01 | Parent原G01/provider/registry：同activation+captured catalog，精确GUID/sub/type，真实Static Mesh Toolkit/host；builtin ResourceKind不足类型裁定 | open |
| RT-S24-P02 | Parent原PreviewSession：Model0/两primitive/material与node transform/source SHA及当前runtime revisions，真实renderer frame/失败；socket/collision/LOD独立能力 | open |
| RT-S24-P03 | 原生filtered sync-file在真实105输入生成24项操作，24/24字段匹配106保存；12嵌套后代与field delta280/471.578947复现，4组pure reflow不引入偏移。原240.0000095/204.0000032及无flow源保持 | open，复现入口已证实为原生同步producer；原public owner/Parent修canonical flow/guard并验证nested sync→layout/reopen，必要时评估root rebasing；无本域像素/touched/ref workaround |
| RT-S24-P04 | Parent Dock/DocumentToolkit：UE水平14/19:5/19与右7:3、同SocketManager TabID归一、真实菜单/焦点/Undo/save；不把preview dirty当document dirty | open |
| RT-S24-P05 | Public原producer/Parent：原四ZUI source/IDs/imports/bindings/events不变，新Static Mesh资产路径只是未创建提案，先给owner/source closure和typed slots/codec，再正式接入 | open |

原native Hug修正的是本域3个新材质标题，不改变Shared geometry或其source guard。18项静态检查/18图已看/三次完整ACK105—107和重开107仅为有限设计证据。RT-SM01—12定义0执行，原Rust/official export/load/EXE/144DPI/双端≤1px、CORS与Source closure仍pending；不扩公共代码写权、不启动服务/插件/桌面或Jenkins，passed/accepted=false。

### RT-S25：独立Curve typed文档与原Surface损失边界

动画原文件已实际只读核rev274/isShared=false，Curve Surface原component **78492c86-3bb4-47dd-97c6-752d6ff37e3d** / main **599963dc-b91b-4740-926b-d38154aa8f3a** 存在；未在08/Shared复制或发布。[设计36](08-resource-tools-design.md#36-独立curve字段保真与原生嵌套构造核对)与[RT-S25](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-source-review-01/curve-target-contract.json)接续原RT-S08，不重做共享组件、曲线renderer或播放clock。

UE每key保存插值/切线模式/权重、曲线保存默认未设与前后外推；现CurveView per-curve interpolation/可选slope不能无损代表混合rich keys。资源域codec需保留全部字段、unknown extension和独立channel/key身份；动画沿原model/surface给真实lossless能力或明确逐操作拒绝，不能先压成global Hermite/Linear。Float/Vector3/LinearColor4与输入单位不依赖AnimationTrackPath/FPS，颜色原始/调整/clamped/HDR及7属性同一document。

| ID | 精确原出口 | 待接内容 |
| --- | --- | --- |
| RT-S25-P01 | Parent原DataAsset/AssetTypeContribution/AssetTypeId与Browser asset_type_id_for_locator | DataAsset已有原文+canonical_json载体，不必先建新payload；真实域codec/schema与同代次semantic subtype仍缺，当前只按ResourceKind返回data。不得给所有Data套Curve或按文件名/JSON猜类型；不扩本域G01写集 |
| RT-S25-P02 | 动画原ui/curve/model.rs与Curve Surface | 逐key interpolation/tangent/weight/extrapolation保真或明确unsupported；未交接口前只读保留来源，不新增并行曲线实现 |
| RT-S25-P03 | Parent/public/动画原发布和source闭包 | 原private组件真实库发布/linked实例/source props/slots/codec，不造公开UUID或复制main；当前没有资源Curve消费者通过 |
| RT-S25-P04 | Parent原DocumentToolkit/host/history/CAS/save/focus | 实际activation/source UUID/sub/document revision，gesture一次commit/inverse、durable save回执，关闭旧ACK拒绝与当前opener返回，不造新栈 |
| RT-S25-P05 | 资源颜色语义及原typed输入/颜色求值owner | RGBA/gradient/七调整同document，保真raw/adjusted/clamped/HDR，真实schema/value/精度与正式input导出后启用 |

RT-CV01—12定义0执行；原六transport routes不动，standalone不装播放控件。原08仍107，42产品pins/原四ZUI/events/G01与Parent公共源保持；official export/load、真实资产/runtime/EXE/144DPI和双端≤1px继续open，passed/accepted=false。

RT-S24-P03补充只读纯构造事实：53parts逐近ref匹配原生make-component-instance，两业务窄版内部Search X等于宽版整块平移位置，delta约等于左栏缩窄量。构造没有原生resize/sync/render生命周期，因此仍不推定根因owner；不作局部geometry/touched/ref修正，原Search stretch与placement均未关闭。

RT-S24-P03最新有限证据见[设计37](08-resource-tools-design.md#37-嵌套search偏移的原生同步producer复现)及[24项精确对账](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/nested-search-reflow-01/propagation-comparison.json)。前节53part纯构造仍为历史事实；本轮真实producer核查推进了原因定位，仍未修复source/设计/产品或取得native resize/EXE验收。

## RT-S19（2026-10-25）：原资产树 disclosure 候选合同

已在原资源宿主接入候选，不再只等待 Parent 完美共享交付；没有新公共控件、theme、registry、store 或 runtime 写入。

- 业务 owner：资源08。`AssetWorkspaceState.collapsed_folder_ids` 为现有 UI 状态的集合扩展，默认保留原展开结果；snapshot 新增 `has_children/expanded`，visible tree 用真实 parent 关系裁剪。刷新清掉失效/无子分支；不同 project_root 清空该 UI 集合。相同根的新 catalog publication 使用 Arc 身份使旧 stamp 失效，未冒充 ProjectActivationIdentity。
- 新 typed event：`EditorAssetEvent::SetFolderExpanded { folder_id, expanded, expected_catalog_revision }`。expected 字段沿现 AssetWorkspaceSnapshot 的 publication/invalidation stamp；不是源资产、document revision 或可保存 activation。执行层在同一 shell state 验证后修改，锁内无 I/O/import/preview/save/外部回调。
- 现有 ViewTemplateNodeData 不扩 props：深度保留现 value_number；资源专属 component_variant 三态映射为 host tree_depth/expanded。叶子隐藏 disclosure，保留缩进；资源行不画没有 route 的 eye/lock/more。公共其它树行不改语义。
- 中立共享几何扩展：原 `tree_disclosure_rect` 调用 `tree_disclosure_rect_for_indent(depth, indent, FrameRect)`，经原 host_contract 有限 export 给 hit。指针以同一当前 HostControlMetrics scale 把逻辑行框映到物理，再调用 painter 的同一函数。未新增 metrics/theme 常量。
- 保持原 Parent `pointer_state` clamp→owner 写回，handle_move/route_at/drop 仍用 Folder。只有 Down 的 disclosure 子区域产生新 typed route；边缘/叶子/文本保持原选择。现 4 ZUI/121 controls/11 events 和 6 transport routes 没有改动。

Parent/共享 owner 后续：原共享 SVG 的 stroke chevron 与 UE rounded filled caret、16 槽及字体行框需沿统一源/guard/parts 对齐；不在本域堆 pixel override。selected reveal/HomeEnd/focus/整 retained generation 的合同继续 open。公开 geometry 的准确候选和 3 个共享前置 hunk 在 `tree-disclosure-runtime-01` 已分离保留。完整 26 路径静态 guard 和 r02 受影响测试 guard 都 0/0/0，10 定义无 Rust 通过回执。

新 tree 受管原键：session01a16d53-60db-7d02-b03e-0042f277783d / request ue-assets-tree-disclosure-20261025-01 / op validate-asset-tree-disclosure-01。queue15791→#384 已因 wrapper 默认 D/dev 被本域中止，job ABORTED，原 QueryOnly 仍 running/pending。需要原 Jenkins owner 的 native/State 对账及显式 E/test 参数业务修订；不重写原 input，不用新 ID 掩盖，不复用 Audio/scroll 资格。原日志中的 source advisory 保留，本域未弱化任何策略。


## RT-S25-CODEC01：本域最小产品合同（2026-10-25）

七路径候选公开 `StandaloneCurveCodec::{decode,encode_into}`、document-owned `StandaloneCurveSource` 及借用它的 `StandaloneCurveAdapter: CurveModel`。实际参数是原 DataAsset + 调用者真实语义选择的 StandaloneCurveKind；不能把 ResourceKind::Data、locator 后缀、schema/display 名或媒体 UUID 当准入。source.version=1 为域编码版本；无新 runtime/core 类型或 catalog 注册。原 codec、typed source、error、adapter 分文件，root 只布线。

Parent 接入要求：在原 DocumentToolkit/retained view 的当前 activation/document generation 内取得源，codec 成功且 surface 可表达才启用编辑；同一原 history 事务提交 key delta/inverse，原 CAS/SaveCtx 决定 dirty、写锁、保存和 durable cancel。完整 expected key 只防本键陈旧，不等价全 document CAS。无共享控件或未知 registry 时保留来源并拒绝相关命令，不伪造 ready。

Capability 有限：Float/XYZ/RGBA、f32、seconds/unitless；源支持空/显式默认和混合插值，但当前共享 CurveView 无这些表示时 adapter 拒绝，原值不丢弃。表面支持同通道统一 Constant/Linear/联动手工 User Cubic（双方斜率相等、无权重）。auto/break/weighted、非 constant extrapolation、f64、颜色调整和未知字段拒绝。梯度/七调整、插入/删除/改键 ID、真实资产 decoder admission 和正式 source round trip 仍 open。无 Timeline/Transport/播放时钟或新 store。

13 回归定义、七路径 guard 0/0/0 与五参考 pin 一致仅静态结论。新 codec scope ue-assets-curve-codec-20261025-01 / validate-asset-curve-codec-01 的显式 E/test 管理计划见原 artifacts；旧 Audio/Scroll/G01/tree 原 identity 不动。资源 typed semantics 的实现不授予本域修改共享 Curve、主题、registry 或 host；parent 原树 state/hit/history 接续分工不变。


## 2026-10-25：G01 r02 同身份源摘要一致性守卫

接续当前 Parent 最小合同，RT-S19 不再以旧“无 fold-set”作现状：原 AssetWorkspaceState 已有 collapsed_folder_ids/可见 preorder/depth/has_children/expanded 和 set_folder_expanded，retained tree 复用 paint disclosure rect，经独立 Disclosure 走 typed SetFolderExpanded/原 host。实际 child folder 判叶、收起选中后代时选祖先、显式选择/reveal 展开祖先沿现接口保留。只读核对两个源 SHA 前缀及完整 hash 已入 r02-handoff；不新写 fold/scroll/history store，不重投 #384。Shift 递归、实际 scroll/present、正式 provider/EXE/144DPI 仍独立待验。

本轮 G01 实改仅原 readonly_query.rs + tests/readonly_query.rs：同 GUID/sub、locator、type 的 catalog 投影若 source_hash 与捕获 registry.entry.source_digest 不同，也返回现 CatalogProjectionMismatch/Stale，不再返回 Known。生产依据为 registry/rebuild.rs 用 meta.source_digest，原 project/import ResourceRecord 用同 source_digest，经 record_projection 与 record_to_view 原样保留 source_hash；只是 opaque 同源字段一致性，不将该值当 bytes SHA256、revision 或保存准入。原 publish_gate→state→index、同 captured input/catalog Arc、锁外返回、unknown/missing/stale 和依赖 coverage unknown 合同不变，未改公共 API/core/registry/runtime/主题/原 ZUI。

新增同路径同类型旧摘要回归，保留旧投影且不触发 refresh/发布；匹配摘要正例维持 Known。G01 原12定义现13，未执行；r02 两路径 rustfmt/diff/guard 全静态通过（0 errors/0 warnings/0 suppressed），7参考 pins 匹配。增量 patch b7ebede156bec26a252ddd15198217722629abdfb5ca586092875addd1881a51，旧 candidate.patch 与全部失败保留。原 Curve 七路径未动。

本轮为 G01 首次正式受管验证准备独立合法 scope：原session01a16d53-60db-7d02-b03e-0042f277783d / ue-assets-readonly-provider-20261025-01 / validate-asset-readonly-provider-01，13个 readonly_asset_，Editor lib/defaults-off/features[]/profiletest/jobs1/threads1/显式 E 根。正常 prepare 本身取得 source/resource/writer/pool/cache/native 回执，不要求先取得 reservation 才提交；parse/capability 或 queue 均不算产品通过。此处仍是提交前记录，后续正式原件以 r02-managed-* 为准；没有复用或重投 Audio/Scroll/tree/Curve 原身份。

原 Curve #388 09:19:46Z API building=true/result=null，同 queue16005/原request/op；不把早前无 Cargo 观察当当前永久状态，本轮仅一次 GET 现有 job 未 QueryOnly/重POST。source、动态测试、Toolkit/实际资产/provider/保存、正式ZUI/export/load、正常EXE/实际DPI/双端≤1px仍各自取真实回执；passed/accepted=false，goal active，释放0B。证据同原 E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-readonly-adapter-01/r02-*，保留 r01 与他人改动。


### RT-DT-SOURCE01 r01 — 真实 DataAsset 只读类型投影候选（2026-10-25）

原 `ui/host/assets/data_table/` 新增只读 `DataTableSourceAdapter::read(&DataAsset, Option<&ReflectTypeInfo>, key_field)`；仅在原 assets/mod.rs 加模块/导出，G01、Curve 原导出及全部原 ZUI/ID/binding/event 保留。调用方须交真实 schema 关联的 struct 行元数据及明确 key 字段；文件名、Data/JSON 格式及 metadata 构造器均不作语义准入。返回原始源行键/顺序、原稳定 ReflectFieldId/元数据及既有 ReflectFieldValue/ReflectedValue，不创建资产、schema、索引、history/store/backend 或 UUID。

首片支持 bool、i8/i16/i32/i64、u8/u16/u32/u64、f32、alloc::string::String。整数从原 token 按实际宽度解析；浮点在任何 f64 投影前检查原 decimal，拒绝精度丢失。未知/缺少/重复属性、重复精确行键、不明确字段 ID/名称、unsupported 列及 populated canonical cache 不一致均拒绝，成功/失败均不改 URI/text/cache。`[]` 可作只读空视图；非 serializable 字段不生成列；editable/range/editor_visible 仍是原 metadata，不产生写权限或可见性授权。

边界：此处是精确 source key，不宣称 UE MakeValidName/FName 大小写/导入去重规则；alias/default 填补/CSV/TOML/嵌套/ref/enum/f64 未实现，不静默转换。真实 semantic subtype/schema association、同代 provider、Document/host/history/CAS/保存准入归原 Parent 合同，当前无真实表资产或行 schema 身份可验。新 adapter 尚无产品 caller，不将其列作窗口已接通。

12 项纯内存回归定义（未注册任何项目/schema/fixture producer）；rustfmt、diff check、7 路 change-guard 静态 0 errors/0 warnings/0 suppressed，9 个只读引用 hash 一致。Patch SHA256 `a992f2a8f1671b837bab57790411f0507907bf3aeedf7c13435184460e1a7269`。Jenkins 初次 DataTable 精确业务请求 `ue-assets-data-table-source-20261025-01` / `validate-data-table-source-01` 将按 Editor/defaults-off/features=[]/lib/profile=test/filter=data_table_/jobs1/threads1/E 根 prepare；此处尚无 Rust/native/product 通过。不是 RT-S19 #384 或 Audio/Scroll/Curve/G01 的换身份重投，所有旧回执和 pending 原链保留。

证据 `E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-codec-01/{source-inputs,candidate-review,static-validation,change-guard,handoff}.json` 与 `candidate.patch`。所有 product passed/accepted=false，桌面/服务/实际 EXE/144DPI/双端 <=1px 尚未执行。


### RT-SM-SOURCE01 r01 — 原 Model/Scene 实例的只读对应（2026-10-25）

原 `ui/host/assets/static_mesh/` 实作 `StaticMeshSourceAdapter::read(&ModelAsset, Option<&SceneMeshInstanceAsset>)`，只在 assets/mod.rs 增模块和导出。沿 Runtime 原 ModelAsset::overview 给统计；提供实例时逐 primitive 比较完整 AssetReference（UUID+locator）并保留原 SceneMeshInstanceAsset，包括全部材质对、fallback、LOD、morph weights 和队列字段。不同模型 subasset、缺/多/legacy-first-only binding、缺 mesh reference、错序或同 locator 不同 UUID 均拒绝。没有实例时明确 Unknown，不当零材质；真正空实例与 Unknown 分开。

ModelAsset 实际只含 URI/primitives，没有原 node/skin/persistent-model-UUID；因此本 adapter 不推导 static/skinned，不将 Mesh kind/.glb/显示名/无权重作为静态准入。model locator 对应也不证明 UUID、当前 revision 或同代 publication；仍由 Parent/角色 source/provider 给真实 node/mesh/skin/primitive/revision、opaque activation 和 registry/Toolkit/host。geometry 是源统计，非 validation/GPU/PreviewSession ready；Known 是 base primitive 引用对应，非材质/LOD已解析或 OpenAsset 完成。未新增 preview/backend/store/clock/history/注册/producer/写操作，未改原产品ZUI/事件/共享组件。

真实 WOC crate_wooden.glb 当前原 bytes318100/SHA7a5f880c07a60adbc9b50b92557e47a1fa35a7dfa0e3a9fe4fe9f42216d31131 与原设计输入一致；未执行导入、Typed Model 加载或入库，真实 engine UUID/registry admission 仍null。8项纯内存契约回归定义：无上下文、全字段/多材质/LOD保留、Mesh4不降Mesh0、partial/extra/错序/缺引用、同locator不同UUID及9条binding、known-empty。测试构造只在内存用原引用构造器，不产生项目/文件/资产或发布身份。rustfmt/diff/6-path change-guard 0/0/0，7源引用hash吻合；patch `a59880a7820cdf82f3fdff50eab56782e148361d52b340dba86704c16c530537`。

本片尚未提交新 Jenkins request/QueryOnly/POST或本地Cargo，原所有 operation 保留；Editor lib/defaults-off/features[]/test/static_mesh_source_/jobs1/threads1/E 最小托管范围已登记供正常批次/合法新scope接续。原DataTable#392当前build仍pending，本次新增接线再次改变 assets/mod.rs；Curve/G01/DataTable原各7pin均为6/7，仅此root变化，旧pin不改，不假称旧scope仍完整匹配。静态 passed 不计 Rust 或产品，passed/accepted=false。证据 `E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-adapter-01/{candidate-review,static-validation,handoff,managed-validation-plan,cross-stage-source-change}.json`。


### RT-DT-CELL01 — DataTable 原文本的单元格候选编码（2026-10-25）

新增本域 `DataTableCellCodec::prepare(&DataAsset, Option<&ReflectTypeInfo>, explicit key_field, &[DataTableCellEdit]) -> Result<Option<DataAsset>, DataTableCellError>`。命令仅按原 source row key +稳定 ReflectFieldId，保留原 Read adapter；两端共用借用原文本的 RawValue row decoder，重复属性仍拒绝。整批核旧 typed value、重复 target、实际 editable/editor_visible/serializable、字段类型/位宽/finite f32及实际schema min/max；失败不返回部分候选，原 DataAsset 在所有路径均不变。成功只替换原 UTF-8 target value span，倒序处理变长字符串，不重排或重写未编辑字节；保留原 URI/format，重建canonical cache后经原读取适配完整重开并逐项核新值。

no-op返回None，保留1.2500/转义/空白/unset cache；有意signed-zero修改按f32位识别。i64/u64始终整数，边界在i128中比较f32的ceil/floor（极限饱和仅在完整i64/u64域之外），不经integer→f32/f64。step/precision只作编辑提示，不用默认42/0..100替代真实schema或自动舍入。

这只是候选编码，未安装document/写文件/调用callback/新建UUID、store、clock或history。实际asset/schema关联、同代provider/opaque activation、view/document/revision、permit/retention预算及一次原history/CAS仍由原host执行；cell旧值相等不防ABA。精确撤销应由原Document记录真实before/after DataAsset，不以重新编码typed inverse冒充源字节恢复。新增UI caller/实际Field/normal EXE/保存/DPI仍未接通，行增删/改名/alias/default/复合/ref/enum/f64/CSV/TOML仍open。

10路径候选增量 `d50a893a3622215a75288a5ec4a4d086936b20daa99a82e94d9b493f9a596752`；4个已有文件的真实前像hash反建一致，StaticMesh/G01/Curve/原ZUI/IDs/events保持；10个引用hash一致。14项新增内存回归定义+原12读取定义=当前data_table_26项；rustfmt/diff/10path guard 0/0/0，仅静态通过，Rust结果null。原prepare报告parser去掉末LF的小失败已留preimage-check-r01.json并修复；未改产品断言或旧源码收据。

原Jenkins `ue-assets-data-table-source-20261025-01` / `validate-data-table-source-01` / queue16284/#392 / input cf6433f8…不改，未QueryOnly/新POST/新ID/替换marker或直接本地Cargo。新source需原execution owner支持的透明业务scope successor，精确10源和Editor/defaults-off/features[]/lib/test/data_table_/jobs1/threads1/E范围见managed-scope-successor.json，当前26定义不能倒签到旧12定义请求。保留旧失败及native/State差异，goal继续active，passed/accepted=false。证据 `E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-edit-codec-01/`。


### RT-S19 共享 caret source closure 精确提案（2026-10-25T10:18:35.922261+00:00）

Parent/原 public owner 收 5 路径只读 proposal：`paint/tree_row_glyphs/disclosure.rs`、`paint/tree_rows_tests/tests/paint.rs`、`paint/tests/runtime.rs`、`assets/icons/zircon_editor_shell/controls/tree-caret-down.svg`、`tree-caret-right.svg`（均在 zircon_editor 原树内，完整路径与 preimage SHA 见 source-inputs.json）。只换树图标源名并扩展原测试；新增 filled-interior/transparent-margin 测试定义 1 条。原 TreeRow ZUI、ID/imports/events/bindings、共享 metrics/hit rect、hover/selected tint 均不动。

提案 `E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-caret-shared-proposal-01/proposal.patch` SHA256 `d2e0e31f71362ca46875de88ee480cf0befc8091d72f3d0309ce8f47a4a70b37`，可只读 git apply --check；未由本域 apply。原 owner 集成时须按真实 current preimage 和 canonical guard/source closure 包含两新图标，再执行最终 checkout guard/原最小托管测试。现 guard 的 5 项未套用 mismatch 原件保留，不算静态全过。不得把本补丁或 Penpot filled Path 当 official export/产品 loaded/原树 Jenkins 修订或 EXE 通过。


### RT-SW-SOURCE01 与 provider/Transport 边界（2026-10-25T10:32:15.166619+00:00）

08 新本域入口：SoundWaveSourceAdapter::read(&SoundAsset) -> Result<SoundWaveSource, SoundWaveSourceError>；SoundWaveSource 只借用元数据和 waveform(channel, NonZeroUsize) 逐声道源帧包络。原 upstream AudioChannelLayout 验证器和 WAV decoder 不改。6 路径写集仅 assets/mod.rs + assets/sound_wave/{mod,source,envelope,error,tests}.rs；Runtime/core/原4Audio生产源/公共组件不改。完整 SHA 见 source-inputs.json/candidate-review.json。

Parent provider 仍须交真实 registry UUID/locator/revision、同 opaque activation/input/catalog publication 的已准入 SoundAsset，原 retained view 处理当前发布和关闭。source facts 不能证明 SoundClipId/输出设备/Kira位置来源/Stop关联终态/current generation 或启用 Play/Record；这些仍消费 RT-S08/09 和 Parent 真实 runtime 合同。无独立播放器、重复解码器、第二 snapshot/history/store/clock。Cue/Class/Submix业务模型、图 adapter、真正作者保存另列 open，不用本 readonly source 宣称图工具完成。

最小拟验证 Editor/zircon_editor/defaults-off/features[]/locked/lib/test/filter sound_wave_source_/jobs1/threads1/E 根；此片未创建 request/operation/POST/QueryOnly。原 plugin Audio3项和 editor scroll5项身份不借用。当前 assets/mod.rs 因结构接线由 d90c64af…到 a468702f…；旧 DataTableCell10pins现9/10、StaticMesh6pins现5/6、Curve7pins现6/7，均仅原 root 改变，旧输入原件不回写。原 owner 正常 producer 必须取得透明受支持的当前 scope 后继。


### RT-S25：已消费同一合同，未重建公共类型（2026-10-25T10:42:26.657620+00:00）

上游 [原 RT-S25](animation-shared-controls.md#rt-s25逐键语义的最小无损准入2026-10-25) 的 private preflight 与本域 Source/View/Mutation 分开：原两动画源 SHA c0272cc8…/2533f9ed…、交付patch acadc92d…匹配；bad tangent/partial component 拒绝由动画 owner 持续验证。本域仅 standalone_curve/{mod,source,equality,tests}.rs + tests/signed_zero.rs 五路径，schema version1/序列化字段/adapter/codec/公共 CurveModel/三共享件/六路不动。

PartialEq 行为有明确后继：原 authoring key/channel 的浮点字段按 to_bits 判等，Option 保留 None/Some 与 ±0；这样原 full expected/replacement/candidate/inverse 不丢 sign-only 变更。字段级 equality 仍不证明 document ABA/project activation/catalog publication/写锁/source bytes SHA 或原 CAS。原缺失 provider/Field/focus/close generation 的菜单继续禁用，未知 asset UUID 不启用确认；无 rich DTO stub或第二 store。

patch `9de2c7f7d435a8c482273ac25b54800994b4734ccb413f4d800253d7434c53b1`；最小拟验证原 Editor/zircon_editor/defaults-off/features[]/locked/lib/default Cargo test（不传 --profile）/standalone_curve_/jobs1/threads1/E，19定义未执行。原资源 #388 的 unknown/capture interrupted 身份/旧7pins保持；当前仅3/7旧pin匹配，3本域修改与此前root接线明确登记，不能把19项回签到旧13项或借动画8定义/queue16509。只给原 Jenkins owner 透明合法 source/scope successor，不创建/重投 request/operation/QueryOnly，不改旧marker。


### RT-S25 r03：当前 source / view / mutation 精确边界（2026-10-25T10:58:21.013818+00:00）

本轮在原 standalone_curve 四路径写入共置 [源合同](../../../zircon_editor/src/ui/host/assets/standalone_curve/README.md)、模块文档入口及 4 条纯内存边界回归；不改 adapter/codec/source/equality 的算法、schema 或公开签名。补丁 SHA256 30706a175124d47f2c5aa99c804428b0bb079ce47a1979ee50b22397957983f0（13695B）；2 个已有 raw 前像精确还原，2 个新增路径此前不存在；12 个只读引用 hash 均匹配。原 r02 五源目前 3/5 匹配（mod 文档、tests 新 hook 有明确后继），原 #388 七源仍仅 codec/adapter/error 3/7 匹配，旧输入不回写。

Parent 语义准入仍是必要真实合同：decode(asset, expected_kind) 的 expected 必须来自同一次 captured catalog/AssetTypeDefinition/Toolkit 资格；它只是 caller 的断言，codec 不签发资格。Data/扩展名/JSON shape/板名/焦点均不能推导 Curve，没有语义 publication 时原宿主保持来源并明确 unsupported。这项公共接线尚未实现，不能据本域解析成功把全 Data 更换 Toolkit。

资源源格式的 User-linked/unweighted 子集不因动画 private projection 可表示独立斜率而扩权。per-channel 模式可不同；单通道 mixed/default 保存原源但表面拒绝，未知字段直接拒绝。完整 codec 能写合法键列表，existing-key Delta 不能 insert/delete/rename，原结构命令仍 open。expected-key/inverse 不替代 document incarnation/revision/activation/publication/CAS；原 Document 一次 permit/history 承接整批，SaveCtx 管 dirty/锁/I/O/失败和 durable cancel。未写动画、公共、Runtime、host、ZUI 或 Penpot。

静态证据：rustfmt、scoped diff check、reverse patch check 均 0；未改 checker 的有界诊断在 63.54 秒 exit0，guard errors/warnings/suppressed 均 0。首次 55 秒超时、空 stdout/stderr 和已回收进程原件完整保留；诊断栈落在原 Rust 模块可达性扫描，不改 guard 或抑制规则。所有回归只定义、未执行；无新编译/桌面/服务/捕获。产品 passed/accepted=false。证据目录 E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-lossless-consumption-01/，r03 使用 boundary-r03- 前缀，旧文件不覆盖。


### RT-S25 r04：从不可变 DataAsset 准备键编辑候选（2026-10-25T11:13:51.465286+00:00）

本域已在原 codec 增加 StandaloneCurveCodec::prepare_edit(&DataAsset, expected_kind, StandaloneCurveDelta) -> Result<Option<DataAsset>, StandaloneCurveError>。它复用原 decode → StandaloneCurveAdapter::apply → encode_into，不新增 DTO/store/history/clock、decoder、资产或 UUID，不安装文档、不 dirty、不保存、不 I/O/callback。完整 source/kind/cache、全部 expected key 和最终可投影 candidate 校验后才返回一个替换值；任何后项失败均不返回部分候选，原 DataAsset 在所有路径保持不变。no-op 仅在原 adapter 完整预检后返回 None，不能以 delta.is_noop 绕过 stale/default/mixed 拒绝。

原 apply 返回的 inverse 在本次临时准备中用于确认真正 no-op；此 API 返回完整 DataAsset 候选。原 Document 须保留真实 before/after 源值以获得精确文本撤销，再沿同一 document/view/activation/source revision、一次 permit/retention/history/CAS 提交；typed 字段相等、内存候选或 JSON codec 都不产生语义资格/写入许可。SaveCtx 继续管 dirty/锁/I/O/失败/durable cancel。changed 候选保持原 URI/format，原 codec 重开校验并重建 cache，JSON 排版可以变化。

四路径仅 standalone_curve/codec.rs、tests.rs、README.md、tests/prepared_edit.rs；补丁 SHA256 0ab156ac3488c1c9bce3baf480333cc8f53b157b04af7a148a9d25b94c8746cc（13598B），3 个已有 raw 前像精确还原，1 个新增此前不存在；12 个 readonly 引用匹配。新增 8 项纯内存定义，当前13+6+4+8=31，不是项目 producer 或已执行测试。原 #388 七 pin 当前仅 adapter/error 2/7 同版；r03 四 pin 仅 mod/boundaries 2/4 同版，codec/tests/README 的后继明示，不倒签旧13/19/23定义。

Parent 原 caller 只应在同次真实 captured catalog/AssetTypeDefinition/Toolkit 资格下取 expected_kind，并传入同个 admitted DataAsset/原 revision 的完整 delta；无语义 publication 时保持来源 unsupported，不按 Data/后缀/JSON shape/焦点启用。prepare_edit 的返回 None 表示已检验 no-op；Some 是一个待原 document 提交的完整替换值，Err 不能安装部分结果。实际 per-view incarnation/current source/CAS、retention 和保存准入仍原宿主负责，不复制 host 或增加公开 registry。这个资源方法不扩大动画或公共 Curve 权限。

本次格式、scoped diff、reverse patch check 已过。guard 第一带周期栈诊断进程 exit0xC0000005，第二标准入口110秒到界，原件均已回收并保留；当前同 hash/patch 的第三标准入口以300秒上限记录实际进程CPU，结果未到，不计 guard 通过。未改 checker/规则/断言或用旧 r03 结果代当前。最终真实结论单列后继验证回执，必要失败不覆盖。

同一真实08 file/page于11:05Z API读回 rev107/vern0/2002 objects，尚无 Curve named shape/设计板；本轮0设计保存。首次 JSON reader 错用 pages-index 的诊断已保留，按实际 pagesIndex 精确读回，未把首次误解析0当空页。没有重复 Install/host/服务恢复/桌面操作，设计共享 Curve/source closure 与产品验收继续 open。


RT-S25 r04 guard 终态接续（2026-10-25T11:15:01.437562+00:00）：同一 checker SHA e2ea2593…、同一补丁和4源，第三标准入口真实 exit0/89.89秒（native CPU48.5625秒），errors/warnings/suppressed=0/0/0，stderr空；自有进程已回收。首次原生异常原因仍未证明、第二110秒超时原件保留，未以更改源/规则/断言换通过。最终静态证据 prepare-r04-final-static.json；31定义仍0已验证Rust，实际执行未知，正式源闭包/真实caller/EXE/DPI继续open，passed/accepted=false。


## TextureSourceAdapter 01 — 原始纹理事实与显式上传预检（2026-10-25）

本轮实际源码仅 `zircon_editor/src/ui/host/assets/mod.rs` 与新 `texture/{mod,source,tests}.rs` 四路径，patch `8990b0fed9c8c04a5be0f16c67fdd04312f64431cdbfcc2cd19d297bf5e6873c`。原 assets root 前像 `a468702f…` 可逐字节反向重建；原 G01、Curve、DataTable、Sound、StaticMesh 导出与并行公共改动保留。未改 Runtime、公共 host、registry、catalog、主题或原 ZUI。

`TextureSourceAdapter::read(&TextureAsset)` 借用真实已加载资产，保留可选作者描述符、URI、完整 payload/metadata/count，单列 Runtime 派生的 `effective_descriptor`、规范化是否改变以及原始描述符的原生 metadata 诊断。描述符缺失为 `None`，与 `Some([])` 无诊断区分；非法作者值继续可检查，不在读取中修复。活动 CPU payload 字节、保留的已初始化字节数、源文件大小及 GPU 显存不混用；容器旁的非活动 rgba 不丢失。

`upload_preflight(support)` 仅将调用方明确提供的 `TextureUploadSupport` 交给现有 Runtime `upload_readiness`，保留其结果；`Ready` 是 CPU 上传方案，不是源资产有效性、真实设备、上传完成或帧可见。宿主仍须从同次捕获的真实项目/catalog/UUID/provider 取得资产，核原始诊断、设备与当前 publication，缺失不得构造空纹理或启用预览。

8 个纯内存回归定义覆盖原始引用/完整描述符、缺省与无诊断、非法值及规范化、normal map 色彩诊断、容器与非活动 rgba、显式支持的 Ready/Unsupported、array/cube/volume 维度、零 extent/NaN 保留。rustfmt、scoped diff、反向 patch check 均 0；标准 change-guard 单次 217.297s 退出 0，errors/warnings/suppressed=0/0/0，子进程已回收。测试定义不计执行通过。

给 Parent 的最小消费出口为借用 `TextureSource<'a>`，不携自造资产 UUID/revision/capability。真实 caller 持有实际 TextureAsset 生命周期，明确给上传支持，分别核当前 activation/catalog/source/provider/device；`None`、诊断、Unsupported 与 GPU/present 失败保持不同状态。只读适配不授权 source mutation、导入/重导入或保存；这些继续走原 Document/history/CAS/SaveCtx 与 Parent retained host。

本轮无共享组件/main/样式写入，也不复制 Search/Field/Tree。原 filled caret 与命中/展开合同不因该 adapter 被宣告通过。旧 Curve r04 的源4路径保持，公共 assets root 由 a468→ada7 是本轮显式导出后继；旧请求原 pin/receipt 保留，不重签历史。


## 后继实际回执：纹理准入与导航矢量（2026-10-25 11:57Z）

TextureSourceAdapter01 已沿独立原身份 `ue-assets-texture-source-20261025-01 / validate-asset-texture-source-01` 正式单次提交：wrapper 335.313s/exit0，queue17519，input `bb6357b9207b0b52450855e81aca128349795e844beed8a5f879c913b849474f`。显式 E/defaults-off/features[]/lib/profile=test/filter=`texture_source_`/jobs1/threads1。唯一同身份 QueryOnly exit2 返回 phase=running/status=pending/result=null，resubmitted=false；并非 authorization pending。actualTargetDir仍null，编译启动、源/资源/pool/cache/native终态及8项Rust执行未知，verified0；匿名queue API403与早期本地observer参数断言失败原件保留，未重投原未知链。两个自有客户端均退出，源码4/4与参考14/14匹配已交Parent/Jenkins owner，queued不等于已读或正式执行通过。

真实 Penpot 原08 file/page 从rev107→108，仅8处旧面包屑 `<`/`>` 文本改为本项目现有 Ionicons SVG几何对应的原生可编辑path。shape ID、parent/frame、12×16槽位、flow布局均保留，确实删除positionData/growType等旧文字字段；其余1994对象、4业务组件与样式记录逐项摘要未变，无新增组件或Shared写入。目录展开箭头原已是矢量，本轮不重写其component/main/copy。UE源码的导航目标是16px `Icons.CircleArrowLeft/Right`，参考树实际缺其SVG内容；当前chevron复用仍是明确外观差异，不宣称UE≤1px通过。

单次实际API保存HTTP200后，以隔离无头Chromium打开原页、逐板选择、关闭context并以新context重开；8/8 path内容与旧缓存删除持续，1280首图与重开图逐字节相同。五张实际PNG均已用view_image打开并在 `navigation-vector-icons-01/review.json` 逐图记录；三板画布zoom分别71/98/134%，浏览器DPR1，不是实际产品150% DPI。900目录尾部裁切、640选中knight目录不可见继续open。raw Transit get-profile首尾摘要一致，0 UI写命令/0插件Open或Install/0桌面输入/0服务变化。首次无头捕获脚本误用Browser.process()失败保留，浏览器已关闭；修正metadata读取后的r02完成，原生browser PID32852出生身份已核且关闭后PID消失。

证据根 `E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/navigation-vector-icons-01`；正式plugin/source-closure导出、原ZUI产品加载、Engine正常EXE/144DPI/原生nonce证据、双端差异图及其余交互仍pending，passed/accepted=false。未删交付/失败证据或共享缓存；自有浏览器临时根已空，自动回收量未测，未声称全局清理关闭。


## RT-S19 原 #384：实际 Cargo 启动与受理边界更正（2026-10-25）

已实读 Jenkins 原 owner `asset-tree-disclosure384-input-revision-reconciliation-20261025-r01` 的 result/parameter-interface/summary 三份原件，SHA逐项匹配；D盘原 `started.json` 607B/SHA44905af0…、stderr69B/SHA7365fec1…也按原路径实读匹配。原 session/request/op/queue15791/#384/input be680232…/State key22cdf92d…不变。新增确证为：原Cargo PID31296/birth134373901772070199于08:22:57.2070199Z启动，stderr仅Updating git repository zr_vm。不能再将该原链描述为Cargo未启动；编译器、harness和10条定义的执行仍无验证结果。原DTO profile=dev，但实际argv无显式profile；不把DTO或Cargo默认值推定成已记录的有效环境。历史stop302、08:24/08:27 QueryOnly与原失败完整保留。

原 owner 11:18Z观察的API ABORTED与State v1/running/resultnull分记，reader active/pinsReleased=false/continuousReadProven=false、launchClaimed及execution running、pool writer与D reservation active均未收口。缺native-terminal/native_job完成记录；当前PID无row不能签root0/children/双EOF/readersJoined或释放锁/资源。本轮只读原报告/源码/已存在文件，不重复认证GET、QueryOnly、stop、capture、提交或资源控制。

现行 `local-cargo.ps1 -RepoRoot E:\Git\ZirconEngine -BuildRoot E:\cargo-targets -JenkinsIdentityFile <source-owner-reviewed-legal-business-revision-input>` 已可表达 `test --package zircon_editor --no-default-features --locked --jobs 1 --profile test --lib asset_tree_disclosure -- --test-threads 1`；原 owner 纯内存client parse/output-policy/runner配方及53172128/12pins是参数合同证明，不是formal public qualification或准入。已受理同键换D/dev→E/test仍明确拒绝 `Original identity already has different action/input/source/runtime`；single recipe不能借durable-batch ResumeOriginal换参。本轮未选后继ID/attempt、未生成或提交业务input，不改c020模块wrapper、原payload/marker/State/sealed或门禁。合法业务修订仍需原source owner与受支持正常intake定稿，不把模板当已受理，也不借Audio/Scroll/Texture身份。

本轮r02源码26/26实读匹配；旧r01仅25/26、唯一测试叶后继和10定义保持，旧收件不验r02。独立DataTable/Curve/Texture现有源码继续保留，未为消除旧native缺口覆盖它们、重造store/clock或停工。当前精确消费记录在 `E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-disclosure-runtime-01/original384-reconciliation-consumption-r03.json`。本段仅新事实对账与文档更正，0Cargo/新请求/资源释放/手工清理，显式释放0B；产品、EXE/实际DPI及全族passed/accepted仍false。


### RT-S22-ROW01：DataTable 精确行键候选与独立托管批次（2026-10-25）

本域在原 `DataTableCellCodec` 增加 `prepare_row_rename`：全源/schema/cache 预检后，仅替换精确原行键的借用 JSON literal 范围，重读检查列、行序和全部 typed cells，保留未改动字节、u64 与 f32 符号零。空目标、已占用目标和缺失原行拒绝；原键相同也须完成预检，返回 None 时不改源。只返回不可变 DataAsset 候选；不安装文档、不写 history/dirty、不 I/O、不改引用。当前键语义仍为精确、非空、区分大小写 UTF-8 JSON 字符串，不自动采用 UE FName、NAME_None 或 Unicode 归一规则。UE 参考为 `DataTableEditorUtils.cpp:587-607`；完整重命名/引用事务与原 Document/CAS/SaveCtx 准入继续由宿主提供。

产品4路径增量 patch `33ecaebd1b2f448a01822faa2597c6fce4b903a88116b696b2ada360ca792794`，两份旧文件的原始字节前像精确可还原、两新增文件；原 cell batch、assets root、公共源码与 ZUI 未改。新增7项回归定义，当前 data_table_ 共33项；rustfmt/diff/reverse-check均exit0，change-guard exit0/61.719秒/0 errors、0 warnings、0 suppressed。仅静态通过，已验证 Rust 0、实际执行未知。

独立本批 request `ue-assets-data-table-rename-20261025-01` / operation `validate-data-table-rename-01` / identity SHA `6c702bd7833ba5b6cecd8c89c44d12e4629cb31423aa822896878df8fd87b202`，Editor单包/defaults-off/features[]/lib/profile test/filter data_table_/jobs1/threads1/物理E，通过原 local-cargo 正式入口实际POST一次，client exit0/148.719秒，queue17885，input `54ca73d17ef46d4e046da5726e8bf069fc820ee8727d6d539ad9a134345461a5`。实际 targetDir null，不能据请求的E根声称已分配。首次同键Query因提交规范化 E:\cargo-targets 与调用 E:/cargo-targets 的原字符串比较拒绝，原失败保留；按原回执的同一规范路径修正一次 QueryOnly 后，得到 waiting/management_git_busy，input一致、resubmitted=false。没有新POST/换身份/改原#392或#384输入/释放锁；编译、33测试及native/资源收口均无通过证据。

[候选及精确接入合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-rename-01/handoff.json)保留原 `WorkbenchExtensionDataTableRowField`、`WorkbenchExtension/DataTableRowEdit`/`RowCommit` 和原 routes；当前handler仍preview，不把codec当已接编辑器。引用覆盖未知时不能宣布零引用或启用跨文档rename；同代次成功后才换选择/焦点，no-op/失败不产生文档事务。正式Field/master/source closure、typed registry/provider、正常EXE/144DPI/双端误差继续open。

本轮Penpot只读实际资源rev108/2002对象/6本域main，Shared rev5/4main/34 colors/3 typographies，动画rev588/原Curve main仍private；0设计保存、不复制未发布main。全部4个自有静态/提交/查询客户端已退出并wait；不由客户端终态推服务端EOF或资源释放。无自有临时脚本/源码副本，显式删除及释放0B，候选/所有失败与收据保留；全族仍active，passed/accepted=false。


### RT-S17-NAV02：真实 UE 圆箭头与原生流式布局更正（2026-10-25）

更正上轮“UE 圆箭头 SVG 缺失”的结论：只读源码树实际有 `Starship/Common/circle-arrow-left.svg`（SHA256 `29b6362d6a36f3ff42b92d38b855209f0a19c73cc78dd0983a8e685636c3feb0`）和 `circle-arrow-right.svg`（`b964c3a12761d1d478df3ec104fd9509725b06ee7f1d96734d3d01cbb9f8e8e5`）。早先 browser-history-icons-01 已用这两源完成另外12处；本轮处理的是剩余8处本域未引用的旧导航草稿。旧误报与失败原件保留，不将文件视为新出现。

本域原 file/page `b4a1e3ce-9e6a-81fe-8008-d69d60ef2172` / `b4a1e3ce-9e6a-81fe-8008-d69d60ef2173`，实际 rev108→112、2002→2018对象。8个原root ID/parent保留，改成16×16 frame，每个含按真实 UE 源创建的可编辑 path+circle；未新建main或复制Shared主件。通过原生Width属性完成12→16及一次16→17→16往返，四条URI由原生flow自动右移8px/宽减8px，不手写位置。保存、关闭、fresh重开后data SHA稳定，rev110和最终112的data SHA逐字相等；1986个保护对象、6个主件、原colors/typographies以及Shared数据未变。

16个stroke实际引用Shared `editor.text.secondary`（`7c88169d-5f57-51e1-98d0-a76d6d08c0eb`，#c0c0c0）；这是本地consumer显式覆盖，不是全局main传播或官方source闭包验收。两次观察器失败均保留：第一次仅监控RPC别名而漏掉实际 `/api/main/methods/`；第二次只允许update-file而误判原生缩略图缓存。逐请求核对实际2个本文件design write+5个本文件derived thumbnail操作，0外域/未分类目标；不修改源保护范围来消除失败。

[本轮证据与交接](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/navigation-ue-circles-02/handoff.json)含7张原PNG及逐图复核。900目录尾裁切、640选中Knight目录不可见仍open。Penpot的71/98/134%画布缩放不是系统DPI；未取得正式ZUI导出/产品加载、正常EXE/144DPI、双端并排/差异或<=1px整窗验收。本轮0Rust源码/请求/Cargo；四个自有隔离浏览器均已await close、8个记录PID当前无row、tmp为空，自动profile清理字节未测，显式释放0B。全族active，passed/accepted=false。

共享边界：Shared保持rev5/4main/34colors/3typographies且data SHA未变；8root均无component/copy引用，6个原业务main完整保留。仅消费真实已发布stroke token。公共canonical owner/sourceHash/slots/正式export与Engine布局仍由原owner接续，本轮不编辑公共main/catalog/theme或插件registry。


### RT-S22-ROWOPS01：DataTable 完整源行复制与批量删除（2026-10-25）

原 `DataTableCellCodec` 新增 `prepare_row_duplicate` 和 `prepare_rows_remove` 产品源码候选。前者复制真实原行的全部 JSON 字节，只改精确 key literal 后追加；后者按精确源键一次预检整批删除，保留存留行顺序/原始 tokens 与数组外层空白。两项均先校验全源/schema/cache，再重读核对完整 columns/keys/field IDs/typed cells（含 u64 边界和 f32 符号零）；缺失/重复选择拒绝，空删除选择仅在全预检后返回 None。不 I/O、不启用命令、不写第二 store/history。UE 依据为 DataTableEditorUtils RemoveRow:457 / DuplicateRow:562 与 DataTableEditor DuplicateSelectedRow:1012 / DeleteSelectedRow:1049。

本批8产品路径＝3原文件插入+5新文件；可精确重建3份旧字节前像，9参考pin未变。增量 patch SHA `f64d78eef0b34efdc4b890186b6d75bce69f5c89d1d476dc7adb03813b7d50c4`，rustfmt/diff/reverse均0，guard exit0/76.719秒/0 errors、0 warnings、0 suppressed。新增9项必要回归定义，当前 data_table_ 共42项；这不是已执行数。

独立新源业务批次 request `ue-assets-data-table-row-ops-20261025-01` / operation `validate-data-table-row-ops-01` / identity SHA `421c6eb896fb0c378161272f2fcba444491f33fb7156babda26f4d4288cd8e11`，当前53172128 capsule 12pins及2canonical匹配，public capability与纯 client/server recipe核对通过。Editor单包/defaults-off/features[]/lib/profile test/filter data_table_/jobs1/threads1/物理E原入口实际POST一次，client exit0/222.016秒，queue18404，input `e05f6fe41fb6fdab03d546463d7803ba77eb824b13bc029d014cfb94e4e814a7`，actualTargetDir null。同身份一次QueryOnly exit2/6.390秒：外层 phase=running/status=pending，内层 status=waiting/reason=management_git_busy，input一致、resubmitted=false。不能把阶段字段当Cargo已启动/未启动或Rust通过，verified Rust0、实际执行未知；不重投，不改原#384/#392/rename queue17885输入或未知锁。

精确方法签名和7步原宿主顺序见 [scope-contract.json](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-ops-01/scope-contract.json)。Parent原宿主须验证实际asset/schema/publication/source/document版本及依赖覆盖，保留原完整DataAsset作inverse，经一次Document/permit/history/CAS提交，成功且generation仍当前才更新选择/焦点。键/值相等不替代ABA保护；观察边为空不等于无引用。既有 WorkbenchExtensionDataTableRowField、RowEdit/RowCommit 与 routes完全保留，无新假按钮或公共catalog/theme写入。


### RT-S25-KEYS01：Curve 整批关键帧插入/删除源候选（2026-10-25）

原 `StandaloneCurveCodec` 新增 `prepare_keys_insert` / `prepare_keys_remove`，均接收完整 `(typed channel, authored key)` 数组，一次返回可选完整 DataAsset 候选。插入按真实 input 排序；删除比完整 expected key（含各 f32 位身份），缺失/重复/stale或末项无效均拒绝整批。全源和最终 surface 都须准入，空批次也先预检；空 channel 保留原身份，不补假键。沿原 codec 编码重开，有变化会重排 JSON 排版；typed 增删互逆不等于字节撤销，原宿主须保留完整 before/after，经一次原 Document/permit/history/CAS 提交。`CurveModel`、`StandaloneCurveDelta`、adapter、源 schema/版本及 equality 完全未改，未新增 DTO/store/clock/backend/UUID producer。

7路径＝3精确旧前像+4新增，12参考pin保持；patch SHA `9cf6742716d51e3a09852eae499df8de95379765a830363e25b6b8978b6c7fbb`。新增11项回归定义，当前 standalone_curve_ 共42项；fmt/diff/reverse均0，guard exit0/78.469秒/0 errors、0 warnings、0 suppressed。仅静态通过，verified Rust0、实际执行未知。

本批新源业务 request `ue-assets-curve-key-structure-20261025-01` / operation `validate-curve-key-structure-01` / identity SHA `78cb08a7c37b3be8cb69d9b167891e71522b81df06ee1be96cc7b6cee2c84e35`，当前53172128 capsule12pins/2canonical/public capability与纯recipe核对通过。Editor/defaults-off/features[]/lib/profile test/standalone_curve_/jobs1/threads1/物理E 原入口实际POST一次，client exit0/171.328秒，queue18663，input `17be5c55e3b42e228ee348dbb256721fbb0f2a1d6ba48e16c3c3e1d3843db61f`，actualTargetDir null。同身份一次 QueryOnly exit2/9.937秒，phase waiting / status pending / result waiting / management_git_busy / resubmitted=false，原 input一致。不据此推断 Cargo已/未启动，不重投或修改旧#388/#384/数据表请求，不释放未知锁。

两新入口精确签名与原宿主7步序列见 [scope-contract.json](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-key-structure-01/scope-contract.json)。这是本域DataAsset源候选，不是共享CurveModel结构delta或已注册菜单；原Delta仍只替换既有键，不另发公共props/slots/main UUID。expected_kind须来自同一真实captured semantic catalog资格；新键ID必须由原真实命令/文档提供。完整expected-key相等不替代revision/activation/ABA/CAS，old ACK不更新选择/焦点或复活关闭view。无Timeline/Transport/FPS/新clock。


### RT-S24-SOURCE-TABLE02：实际源表格与原typed provider消费边界

本域原静态网格main新增可编辑primitive列/行flow，实际身份和34near-ref映射见[E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-design-02/design-contract.json](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-design-02/design-contract.json)；没有新增公共master/list/Field/主题。源GLB material0/1仅local ordinal，不是Engine UUID。后续只从同一admitted publication的原 `StaticMeshSource.geometry.primitives[].primitive_index/vertex_count/index_count` 投影Runtime行；`.mesh`保留完整AssetReference，材质只有原 `instance=Known` 的逐primitive/fallback真实引用可交registry/toolkit/host。当前GLB POSITION声明不得静默改称Runtime vertex_count；Unknown、缺语义static subtype、catalog generation/activation仍禁导航/修改，URI/同名/扩展名不补资格。

Parent原provider/host/PreviewSession/Document/history/CAS/SaveCtx责任不变。900/640原Shared Search位置、显式copy的新增content/layout overrides之native sync/reset、official family codec/export和真实EXE仍pending；Shared实际rev5/4masters/34colors/3typographies原raw未变。原4ZUI及公开源未写。[交付](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-design-02/handoff.json) passed/accepted=false。


### RT-S22-CSV01：真实源/Document/保存消费合同（2026-10-25）

公开原 `ui::host::assets::DataTableCsvCodec` 与 `DataTableCsvError`，不改已有DataTableCell/Row候选入口。`encode(&DataAsset, Option<&ReflectTypeInfo>, key_field)` 返回CSV文本；`prepare_reimport`再收CSV文本，返回一个 `Result<Option<DataAsset>,...>` 整表候选。没有生产默认构造、index/store、文件I/O或命令授权。未改token保留，13定义覆盖全批次失败、header/quotes/中文、整数极值、signed zero、readonly/hidden、旧值超范围、cache/schema等，均未验证运行。

调用方须来自同次真实captured catalog/语义表类型/row schema及当前activation/document/view/request/source revision，不能用ResourceKind::Data或JSON形状自动推断。整批候选由原Document一次permit/history/CAS安装，保存完整before/after作字节inverse；key/字段值相同不能拒绝ABA。SaveCtx负责dirty/锁/I/O/取消/实际保存结果。源损坏恢复、UE FName/默认值/部分错误导入、readonly初始化与未知字段仍明确unsupported。

原 `extension_data_table_workspace.zui` 的Validate/Save/RowEdit/RowCommit、源码 `specs/data_production.rs` 和 `feedback/data_production.rs` 保留；当前feedback仍 `Not implemented (preview)`，没有CSV Import/Export的正式route或handler。此批仅可供Parent接真实per-view host，不把codec或模板按钮当真实编辑器。9路径静态guard0/0/0，queue19307/#409已实投、运行结果null；[消费矩阵](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-csv-01/scope-contract.json)。全族仍active，passed/accepted=false。



### RT-S22-COLUMNS02：列宽调整／中文长文本原生状态板（2026-10-25）

08原file/page实际保存至rev128/vern0/2300 objects，仍为6业务main；在原900 linked-copy board原生Ctrl+D生成3个新板（231对象），再用原生Inspector调整到1280×800、900×620、640×520。原2068非根对象与6主件定义精确不变；根仅增加新板，旧子序列顺序保留。原DataTable component dc042d85-e62f-4025-9680-10ee1619c9d7 / main a7972f31-e63f-4359-8e6d-3e871ebb2d8d不变，未新建/复制main或detach。

这是原DT_Items四行ZUI布局fixture的显式状态示例，未创建真实DataTable资产/schema/UUID/producer。只在新增实例把表头及四行Name列85→180，列宽统一为17/180/76/77/55；新增中文行名和完整长Value，原生Text Auto Width使Value保持单行，由原77宽clip cell裁切。保留原Fill、12px Roboto、Shared body typography ref；中文可见是浏览器fallback，不能据此证明Engine/CJK字体ready。原默认三板不改。

最终r06六张full/crop原PNG已逐张打开；native save-close-reopen128 data SHA 06f62581abb137833ea093a07bc11ec01414482c38eef1ad79dfd8a87d9b8e59一致，fresh GET也一致。新增201本域+21 Shared Search sourceRef全部222/222有效；453个非空style RefId及其453个RefFile逐对有效（非453个唯一style）。Shared真实rev5/4主件/34colors/3typographies、原直接库连接保留；get-profile原响应SHA前后一致，未Install/Open插件或写registry/Shared。源ZUI/catalog/tokens/Starship等8pins与CSV9源码pins仍匹配。

保留F2定位超时、X轴名称不匹配、r03把3个图片pair误比6个文件导致success=false、900单行改动的过早GET失败及随后rev126真实持久化、r05 fresh-rev拒绝、向量字符串触发的本地text walker异常和列存储反序诊断。r06等待实际发布后重开；原始失败未覆盖。12原PNG逐图记录、原控件/事件/import映射及接入缺口见 [handoff](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-columns-design-02/handoff.json)、[individual-review](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-columns-design-02/individual-review.json)、[source-checks-r02](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-columns-design-02/source-checks-r02.json)。原Not implemented (preview)反馈、真实表schema/registry/provider/Toolkit、原Document/history/CAS/SaveCtx、菜单/编辑/保存撤销/滚动焦点、正式family插件导出加载、正常Engine EXE/144DPI/双端<=1逻辑像素仍open。此次无产品ZUI/Rust/公共源写入，无Cargo/桌面/服务/DPI动作；产品passed/accepted=false，goal继续active。

CSV原queue19307/#409最新有界认证API已为building=false/FAILURE；原console terminal为management_operation_interrupted、HEAD, full index or complete source membership changed during capture、statusunknown（保留此前3次resource_capacity等待）。这不是Rust测试失败，也没有native EOF或资源释放证明；原9源码/13定义不变、验证Rust0、实际执行未知。[终态复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-csv-01/build409-terminal-review-01.json) SHA 67ad89b31ac7256f01c64b2f66dfe1cd7083973eaeb83c20c38264fee3fbaae4；原control receipt540B SHA eb28e1de273fb54d81c6120fc6d42a5b4b9adf67be106e549a7e51543da75ad1。已沿原identity253edb50…/input0f1687c0…向Jenkins原owner排入消息01a17393-17f2-77c3-bf40-5beed1bddaa8，queued不等于已读；无新QueryOnly/POST/identity/attempt/freeze/锁或State改写。


消费边界：只新增本域linked copies，不发布公共主件/新theme。Shared Search每copy7/7 refs属于原main 6ddc248b-bd35-80c5-8008-d6c8c0f9c196，component 6ddc248b-bd35-80c5-8008-d6c8c3ce892e，三板合计21；DataTable每copy67/67属于原main，合计201。原content/geometry override保留并新增本次本域显式覆盖，未验同步/reset，不传虚构slot/property或组件语义版本。当前body ref 99eb86f9-12ff-5935-96ad-3d1a985e61f0只证明原生字体样式引用有效，不证明Engine fallback。Parent仍接semantic subtype/registry、真实表viewport/每view宿主、history/CAS/SaveCtx；public owner接原family source-closure/codec/export，资源提供真实JSON/schema typed adapter/整批候选及完整inverse。Data不能按扩展名/fixture板名自动认Table。


### RT-S22-MOVE01：原宿主一次事务消费（2026-10-25）

原 DataTableCellCodec::prepare_row_move(&DataAsset, Option<&ReflectTypeInfo>, key_field, row_key, target_row_key) -> Result<Option<DataAsset>, DataTableCellError> 是不可变整表候选接口。Data/JSON/Name 字段或布局 fixture 都不能授予 Table 语义；调用方须捕获同次真实 catalog/表类型+schema 与当前 activation/document/view/source revision。保留确切 key，不从显示行号重建身份。无效未选行、重复属性或 key、未知格式、stale cache 在自移前也拒绝。

成功候选由原 Document 一次 permit/CAS/history 安装，完整原 DataAsset 作 inverse，SaveCtx 负责 dirty/持久 I/O/锁/取消；相同 key/值不代 incarnation/revision 或 ABA 判断。当前成功才保留 moved key、重建源 RowNumber 升序，并在同代 layout 下 reveal；旧 ACK 不抢焦点或重开已关闭 view。None/Err 不发布 selection/dirty/history。源码四路径不碰公共 host/registry/source closure；[精确合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-move-01/integration-contract.json)。9 定义0验证运行，queue19857/#413 不等于 Rust/native/产品通过；真实 per-view producer、语义注册及正式导出/EXE 仍 open。


### RT-S22-MENU01：可编辑行菜单／原生三尺寸重开（2026-10-25）

08原file b4a1e3ce-9e6a-81fe-8008-d69d60ef2172/page ...2173实际rev252/vern0/2759对象、6业务main；新增459对象，旧2299非根对象、6主件定义及旧根子序列精确不变。3个新增DataTable copy+3 Shared SearchInput native linked copy，222新引用=201本域+21Shared。Shared实读rev5/4main/34colors/3typographies且本阶段数据不变，无新main/detach/并行主题。

按实际UE SDataTableListViewRow菜单实现11项/3分组的纵向flow设计，行内caption Fill、5个56宽右对齐快捷键；plus/首尾双上/双下均为原生vector，共9 path，无文本箭头。288×271.4的原生absolute popup按实际Name cell中心夹在板内，native small nudge的Y舍入-0.2，zIndex=1；普通行280×20.4/columnGap10保持flow，48段文本消费原Shared body。此为原4行ZUI layout fixture，不是就绪资产/schema/Toolkit或真实菜单执行。

save-close-reopen及6次捕获后data SHA b415e85d193aa9397d6f19d2c92ccaf696f84ccdef48e83f8d70e77c7d663dbf一致，rev252；两独立context均await关闭。六张原生100% full/crop逐张打开，crop精确1280×800/900×620/640×520，菜单末项完整，未缩字。Chrome dpr1/100%不作Windows150%DPI/正常EXE证据。25/25源pin相同，原ZUI/IDs/imports/bindings/events/Rust及公共主题均未改。

本轮消费不发布菜单master：继续复用现有ContextMenu/PopupMenu，等待原owner正式typed slots/source-closure/producer，不能把原生frame草稿当公共组件或把无效果菜单启用。Shared5真实body ref在48新菜单文本中有效，root背景沿原editor.surface.0；泛型图标及small keybinding typography仍有差异，不新建主题。全1678source refs含14条原生nested两段链，r03要求真实outer-main membership/对应nested组件file+id/父ref/Shared canonical后代和type，未改official guard。现source与组件语义version不混写。

资源未就绪，正式plugin测试/import-export/load、真实popup输入/disabled/IME/Escape/当前generation焦点返回、Document/history/CAS/SaveCtx、正常EXE/原生LayoutReviewCase/144DPI及双端≤1逻辑像素仍open，passed/accepted=false。无桌面/服务/源码副本或新Rust构建，释放0B，必要失败与交付原件保留。

[交付](<E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-menu-design-01/handoff.json>)，[逐图记录](<E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-menu-design-01/individual-review.json>)，[嵌套source核对r03](<E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-menu-design-01/source-checks-r03.json>)。


## RT-DT-CELL01-R02：有限受管后继合同

- 源责任 owner 仍为资源08。正式 request=ue-assets-data-table-source-20261025-01、原 session 不变；本次显式新 operation=validate-data-table-cell-source-r02，不是恢复或重投原 #392。原 identity/marker SHA401aaa2b…逐字节未变，原 unknown/native/resource 边界继续保留。
- 当前源 revision 944a2dddbe872852e51fd763df7f91fdc138f28abe218327d203e347dd59d593：10份源声明、28份只读依赖。原 CELL01 为6/10匹配、旧依赖10/10；四个已记录后继不回滚。当前全部38 pin 再读匹配。revision 是报告身份，不向协议加入不存在的 sourceRevision。
- 有限测试采用 --exact 和完整 ui::host::editor_asset_manager::data_table::tests::... 名称：原 source adapter 12 + editing 14 =26项静态定义；当前已接线 DataTable 子树64项，其余38项未列为执行。目录 assets 不等于实际 Rust 模块 editor_asset_manager；不使用宽前缀来虚称26项。
- current selected 5317212844a5cd6ff9117849c8bb01a93b8e97f5ae614778c445b34f005b678e 12/12 pins、public transport、client parse→server validation_recipe 均已核。单 zircon_editor、defaults-off/features=[]、Windows host lib、显式 profile=test、locked/jobs1/threads1、E:/cargo-targets；正常 prepare 自行取得真实 source/resource/writer/native 准入，不额外索要 grant/freeze。
- 本次正常 wrapper 仅调用一次，客户端exit0/reaped，resubmitted=false；真实 queue20950 → zircon-management #419、input 8a8059bffe1bf3fa7662448ff941592c88453d42eddd57f126e13950ffb96e82。17:31:36Z认证API已building=false/result=FAILURE；控制receipt SHA3babac39…、精确State key a0a18027d651a5e62c5f1c70c7d255b7ed229076f3af3e8aed9939e7c6d652c6 v2/unknown，reason=management_operation_interrupted，原文HEAD/full index/complete source membership changed during capture。该通用错误不能识别实际变化分量/写者，不能写成Rust失败、确认未启动或native/资源已释放。
- 只读格式首试误用2024 edition，失败保留；按实际Cargo.toml 2021 edition复查，10路径rustfmt与diff均exit0，无生产源码改动。0 formal QueryOnly/重复POST/服务/桌面/清锁/清缓存；verified Rust0，actual Rust/native/resource结果未证，passed/accepted=false。产品schema/Toolkit/Document一次history-CAS/正式导出/EXE/150%DPI继续open，目标active。
- 完整范围、26项名称、38 pin、argv、原始stdout/stderr、真实API/console/控制与只读State见 E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-cell-validation-r02/。旧#392与新#419分别保留，后续仅交原执行owner对账本新链，不盲重投。

本次未改变 DataTableCellCodec::prepare 的源/DTO/错误/事务合同，也没有借新operation接管或更改公共源、schema/registry/history、Cargo、主题或其它域。Shared、Penpot、插件导出与真实引擎状态没有新增观测，既有证据按原时刻保留。


## RT-SW-RANGE02：可见源帧消费合同

- 输入：原 SoundWaveSourceAdapter::read(&SoundAsset) 获得的借用source，channel、半开 Range<usize> 源帧区间和非零bin预算。0<=start<=end<=frame_count，channel错误优先；错误InvalidFrameRange保留实际start/end/frames，不夹紧或产生部分结果。read仍对整个源验证布局、帧对齐和finite样本，包括区间外数据。
- 输出：同一SoundWaveEnvelope的逐声道惰性ExactSize/Fused包络；原始源帧坐标、合法空range零bin、巨大预算有界，无新波形buffer/downmix/source mutation。原整段入口继续复用，原URI/layout借用与解码owner不变。
- 宿主：Parent原source provider与per-view publication负责真实project/asset/type/catalog/source/document/view和当前generation；保活原被借用SoundAsset后消费。old range在新source中碰巧仍合法不能替代代次检查，不新增viewport store/history/clock。整数源帧不得用动画FPS或播放cursor猜测；UI ratio映射、min-two-frame、pan/zoom/focus/present仍为原host/control责任。
- 能力：这是只读可见区间，不是trim写命令、注册资产、SoundClip/Playback身份或输出ready。六Transport routes及动画共享合同不变；Kira位置来源/Stop终态/原finished队列owner仍open，组件不能drain全队列、停设备或从波形启用Play/Record。
- 当前5路径源码候选与7新增定义已落地；格式/diff/反向patch/guard0/0/0，旧10+新7共17定义但verified Rust0。原#388/#392及新SoundWave未分配身份保持；后续如正常受管准备，须按当时revision和精确17项有限提案走原入口，不复用旧unknown来换源。原Doc/history/CAS、Toolkit、正式export、EXE/DPI均不借本候选关闭。

详见本域 sound-wave-visible-range-02/integration-contract.json 与 managed-validation-proposal.json（仅范围提案，request/op/attempt=null；无prepare/validate/POST/QueryOnly）。
