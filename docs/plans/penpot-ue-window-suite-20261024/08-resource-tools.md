# 08 资源抽屉与资源工具

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

## 最新接续：真实叶子隐藏箭头并保留槽，原页保存重开93

原08页93/1666对象仍保留4业务main、151可编辑SVG与972refs；34个真实WOC/Vampire目录子目录名单确认85个叶子，已仅将其path隐藏，31展开/35折叠分支继续显示真实箭头。16×16槽、gap4、原缩进/文字/选择及其余1581对象精确保持。独立native保存关闭重开93/hash不变、0额外保存/registry变更、Chrome退出0，六板及树特写14图均实际打开。见[设计30](08-resource-tools-design.md#30-真实目录的叶子箭头可见性保存重开93)与[实际预览](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-leaf-disclosure-01/directory-tree-ue-branches-leaves.png)。这不关闭640选中不可见/900尾部裁切/CJK压缩及RT-S19-r02公共输入/state/selection缺口；产品/EXE/DPI仍pending。

## 最新接续：目录箭头已换成真实SVG，原页保存重开92

按用户截图要求，目录中的字符箭头已拆成真实UE caret-right/caret-down矢量；原08 file/page现 **rev92/vern0、1666对象、4业务main**。151行含31展开/120折叠，保留原目录文字/行ID、缩进、选择与旧832 refs，新增70份实例各两条source refs，**972/972引用有效**。图标是16×16 s档容器和原生可编辑PathData，文字与图标以row flow组合；未新建公共TreeRow/主题。见[保存关闭重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-arrow-vectors-01/final-reopen-13.json)、[精确源保护](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-arrow-vectors-01/checks.json)、[20图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-arrow-vectors-01/image-review.json)。

此前AssetTile类名框14及root vertical Hug已在13主件/实例保存重开87；三尺寸网格及真实长名name/type分离，未缩字。[设计第27/28节](08-resource-tools-design.md#27-assettile的ue类名区14及流式高度)保留同步附带改写和逐字段恢复。当前92相对87只原151目录行及151文本变化，1062无关原对象、其它三份组件定义和SoundWave99对象保持。真实展开state生产端仍缺，见RT-S19/RT-G34；640选中目录不可见、900尾部裁切与CJK压缩继续open。

20:52Z只读Shared已为 **rev5/4main/34colors/3typographies**，原4主件UUID不变；Button主件新增一个真实color ref pair，其余默认继承/reset未通过。消费者synced-at仍18:07:13Z，本轮未sync或改Shared。原rev2/零非空引用保留为当时历史。原四ZUI/source hash保持；官方导出/产品加载、真实provider、托管Rust、正常EXE/实际150%DPI及双端≤1px仍pending，**passed/accepted=false**。未重跑QueryOnly、占桌面或启停服务；释放0B，原失败及两份空临时目录保留。

## 最新接续：J01有限入口已部署，Audio执行仍待准入

原owner的19:50:05Z [J01部署回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-j01-intake-01/receipt.json)已接续：capsule 0b594015… 的Sound单包有限recipe已贯通，早期build_scope_policy拒绝保留为历史。原Audio request/operation/identity未改；本域核原4源码+18参考及14份引用回执全部匹配，没有重跑QueryOnly。

J01的335条JUnit/9项PowerShell属于管理入口验证，Audio仍0提交/0 Cargo/0测试。原owner唯一QueryOnly为ordinary_entry_unproven / Current session authorization is pending，无server receipt；writer/target/cache lock/reservation/capture均未取得。所报E盘12.16GiB低于v11 reserve+单job预算39GiB，不能据此开始捕获或Cargo。原owner沿原身份接自动收件及E根真实准入；本域不请求用户grant、不冻结文档、不改盘或清共享缓存。全族passed/accepted=false。

## 最新接续：原四owner闭包的消费输入

19:44:46Z只读API确认原08 file/page仍rev85/1364对象，Shared链接rev2、原83插件注册；本页官方source-owned asset计数0，与现有业务main/18板分开。[当前源码观察](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-closure-consumer-01/source-readiness.json)确认原public源码已新增通用owner、existingPlugin/append与保护既有内容的候选，不再把早期固定LevelEditor实现当当前全部状态；本域没有运行或改写这些公共代码。

[RT-S18/RT-G32](08-resource-tools-design.md#25-原四owner的source-closure与分配页追加前置)保留两项实际前置：Transit权限经原runner成为Set而verifier要求Array，注册fingerprint尚不可用；served bundle已348732B/SHA6ded0449…，未取得对应managed UI build/index/worker回执。[消费输入](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-closure-consumer-01/consumer-inputs.json)列四原owner的ID/events/import/hash，先以原Browser接续，实际append须fresh revision，不借LevelEditor edit或用同一85并发导入。没有插件安装/服务/桌面/新Jenkins/产品源修改，原Rust、正式导出/加载、provider与EXE/DPI验收仍open，passed/accepted=false。

更新时间：2026-10-24（下文带Z的证据时间为UTC）。Owner chat：`01a16d53-60db-7d02-b03e-0042f277783d`。Parent：`codex://threads/01a16d4e-0491-7353-84fa-038bd7706546`。可分配依赖与执行现场见 [Parent 交接](08-resource-tools-parent-handoff.md)。

## 当前事实与交接

## 最新接续：SoundWave原生copy的离线逐字段对账

已收Parent [followup-05](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/asset-tools-followup-05.json)，保留其6张rev80图实际复核、rev85的832引用/27标签及Shared样式纠正口径。本域复用原始保存响应，新增[rev80→85离线比较](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-copy-integrity-01/offline-copy-comparison.json)：三个SoundWave子树99对象/2,890字段全部相同，6个PathData直接tag/原始bytes及96关键字段相同，27标签和63文本对象连缓存保持；未新查API、创建图片或操作服务/桌面。

[可靠PCM基线对照](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-copy-integrity-01/known-failure-detection.json)补出重要边界：r02前后PCM bytes不变仍可承接r01污染。相对原rev76三条PCM，失败rev78/79均仅1/3匹配，恢复rev80及已保存rev85均3/3匹配。main同步必须同时核可靠来源与每个copy，不凭保存ACK或局部前后无差异完成。[设计第24节](08-resource-tools-design.md#24-soundwave保存响应的逐copy完整性)列范围与差异；原失败全部保留。

Shared的21组refId/file共42字段均null，非空0；Import的8 authoring+2缓存字段同属显式本地override，非传播/reset。G01仍为上轮实际代码候选与既有ACK，不重复派发或运行；原Audio3/scroll5、provider/source closure/正常EXE/DPI与双端误差验收继续open，passed/accepted=false。

## 最新接续：RT-AIREF01-G01只读adapter代码候选

Parent已把G01分配为原manager内的最小实现；实际checkout预检查本次7个产品路径diff为空，先登记精确写集/27输入hash再编辑。[单一查询合同](../../../zircon_editor/src/ui/host/assets/api.rs)新增 `readonly_asset_snapshot`，沿原publish_gate→state→index捕获真实activation、input Arc和catalog Arc；精确GUID/sub确认后读取原registry边、同一catalog的asset/details并释放所有锁。不新增index/store/service，不调用refresh/import/preview/save或AI回调，不改runtime/core/records/Cargo/catalog/主题/原ZUI。

[设计第23节](08-resource-tools-design.md#23-rt-airef01-g01原manager只读adapter候选)列实际API、scope失效规则和12项测试定义。known/missing/unknown/stale独立；空边仍coverage unknown，metadata digest不当SHA256，caller提供的source policy不当IO或保存准入。Vampire原Data/TOML样例仅只读磁盘对账，不注册AI schema或打开Toolkit。候选尚未编译或执行测试，真实host/provider接入及G02～04继续open；本轮无Jenkins请求/QueryOnly/锁生成、服务/桌面/EXE操作，passed/accepted=false。

## 最新接续：动画r02与试听provider的终态关联

已消费动画[资源第四次接续r02](animation-shared-controls.md#资源第四次接续r02)及[机器回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/resource-consumer-r02.json)，沿原RT-S08/09、RT-TYPE01接续。其rev76是历史观察；本域最后实际验证仍为18:40:19Z rev85，本轮无API/新图片。SoundWave业务main、原MP3/诊断、未应用的instance-scope diff和六route/六按钮+Fill保持；没有新operation/caller、DTO、schema或播放器。

新增独立核对：设备退役也产生Stopped；原private output_generation从0开始，Finished/公开output status未带原代次。须关联原命令返回、playback lease和终态来源，不能仅凭Stopped或后续UnknownPlayback判断，更不能给旧队列事件套drain时的当前代次。原host的ControlFailure保留typed Err，沿原registry/journal处理。[21源/10事实](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-provider-handoff-03/source-capability.json)、[RT-TYPE01-P04提案](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-provider-handoff-03/provider-host-contract.json)及[设计第22节](08-resource-tools-design.md#22-动画r02与原试听provider的终态关联)列精确出口和六个未执行场景。

Parent原provider/host接位置来源/失败诊断、终态原代次、唯一完成队列消费和实际caller；08仅消费本资产/实例。asset UUID/catalog admission/runtime position/capability未交，不重解码或用PCM启用Play/Record/step、转CurveKey或套FPS。Standalone Curve可无Timeline/Transport。无新Jenkins/capture/桌面/服务请求，原Rust/真实音频/导出/EXE/DPI验收仍open，passed/accepted=false。

## 最新接续：Browser类型行测量缓存已修正

18:40:19Z新进程重开原08 file/page为 **rev85/1364对象/4本域main/26copy**，832/832 source refs有效，上一轮Shared body/primary消费的8个ID/file字段保持。Browser长文件名与Texture重叠已追到原类型行的旧position-data：实际SVG字形重叠4.99658逻辑px，原geom已经下移而测量缓存未跟随。只让原类型文本重新测量，保存ACK200；最终仅此一对象的position-data变化，文字、字号、几何、原source/touched/guard及其它1363对象均保持。新进程重开同全部保存对象一致、0写请求，实际glyph gap6.53270px，六张原生full/crop已逐张打开。[最小修复](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-text-metrics-01/native-metrics-r01.json)、[新进程重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-text-metrics-01/fresh-reopen-r02.json)、[逐图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-text-metrics-01/individual-review-r02.json)及[设计第21节](08-resource-tools-design.md#21-browser类型行的旧测量缓存)保留原始证据。

RT-G29现在是“Penpot单板缓存已修正／产品与三尺寸待验”，不是全族accepted。UE新样式ClassNameMaxHeight14px与当前Tile类型框12px的完整对齐、Shared Search响应式、canonical master样式引用/默认传播、CJK/引擎字体、official导出→原ZUI加载、真实provider、托管Rust、正常EXE及150%DPI仍open。Shared/public source、原四ZUI、Audio候选和Parent滚动代码未改；没有桌面或服务生命周期操作，passed/accepted=false。

## 最新接续：原对象恢复及Shared样式实例消费已保存重开

2026-10-24T18:30:27Z，原08 file/page经新Chrome/context关闭重开为 **rev84/1364对象/4本域main/26copy（21本域+5 Shared）**；832/832来源引用有效。r03同步造成的25处变化已通过原native对象恢复，r05新增的一条geometry-group覆盖标记也按原API选项去除。相对原rev80，最终只有受控Button的原Import文本显式消费已发布body/primary样式；其余1363对象及4组件定义保持，未detach或复制main，原source refs/文字/已有覆盖保持。[恢复与保存ACK](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/native-consume-r06.json)、[新进程重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/fresh-reopen-r08.json)及[八图逐图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/individual-review-r08.json)已保留。

本轮源保护检查实际发现Parent持有的catalog已由最初4c2f849c…变为 **e4bba09c3ba16d55b90eabb5a821ab3e76eb0606f9afc17e9f656e2d16fe0a99**；原r08全文件不变检查保留为失败。只读复核NumberField10props/defaults、PropertyRow4slots与原ZUI/前次合同完全一致，另8份合同源保持；[r09范围对账](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/catalog-contract-reconciliation-r09.json)不声称整份新catalog或新master验收。本域未改公共文件。

Shared仍实际 **rev2/4main/34colors/3typographies**，但master实际非空样式引用为0；原21条null字段不是绑定。原linked Button文本已保存实际body及primary的UUID/refFile（段落和文本run共8个非空ID/file字段）；此为显式本地覆盖试接，不是master默认传播/reset通过。原自定义Roboto/700改为发布body的gfont-roboto/400，原guard未改；CJK、引擎字体和official source codec均未证明。Shared、83条插件registry在操作前后hash保持，无公共master/主题/服务/桌面写入。

八张原生Penpot full/crop已实际打开；均71% canvas缩放，不是产品或DPI截图。Browser长文件名与Texture类型重叠记RT-G29，Search的832/240/204响应式缺口仍RT-G19；没有缩字、隐藏目录或用空Property.value当完成。当前Jenkins所选e0635d5d…胶囊11/11源hash匹配，仍拒绝原Sound聚焦配方（build_scope_policy），未运行Cargo/未重投请求。原错误、r04服务拒绝、r05覆盖标记和r07只读脚本失败保留；**official导出/原ZUI加载/真实provider/原Rust测试/EXE与150% DPI/双端比较继续pending，passed/accepted=false**。

## 历史接续（18:12Z）：Shared rev2同步异常与新增字段合同

已接Parent followup-05；最后成功API读回的Shared仍为rev2/41对象/4原main，**34colors+3typographies确已发布**，catalog新SHA4c2f849c533f1f227903018eca860faadd7fa3138fdc265300913e3833ad36e5匹配。当前不再用rev0/零styles作阻塞。原5个linked instances的原生sync-file已HTTP200保存到资源 **rev81/1364对象**，但检测到25个几何/测量变化，其中10个非Shared目录文本框也变化；已保留native请求/响应及精确恢复清单，尚未恢复。

新增独立校核发现，Parent原回执所列21条style-reference字段的 **refId/refFile全部为null**；同一Shared API master实际非空style ID=0，不能把字段存在数当绑定。资源5 copy/40节点身份、原sourceRef、文字和已有touched flags保持，实际非空style ID仍0。发布37个styles与主件是否引用它们分别记录。[设计第20节](08-resource-tools-design.md#20-shared-rev2同步事实与字段合同接续)、[RT-S15及RT-C06](08-resource-tools-shared-contracts.md#rt-s15shared-rev2的同步与非空样式引用)已接续，没有改Shared/主题或detach/复制main。

恢复/受控消费r04打开页面即19001 ERR_CONNECTION_REFUSED，native browser exit0，**未执行恢复或应用任何样式、未关闭重开验证或新截图**。最后实际保存仍81，上一轮rev80六图只作历史。没有启动、重启或续期服务；恢复25个原对象及原linked Button显式消费body/primary的计划已准备，待Parent/原owner实际readiness后沿同身份继续，不拿失败当通过。

NumberField10props和PropertyEditorRow4slots经6项针对性静态校核与原ZUI匹配，限公开源候选；没有NumberField新master或原字段runtime准入。RT-C06已交真实schema/value/source/reset绑定提案：保留DataTable原Version Field及两原events，不自动套42/0..100，不经f32保证u64/i64，不用Label填required value。官方导出、CJK、Rust与EXE/DPI等仍pending，passed/accepted=false。

最新SoundWave标尺接续：同一08 file/page已保存关闭重开为 **rev80/1364对象/4业务main/26copy**；UE MidSplit depth3的9条电平线及9文本沿原主件同步到两copy，共27新增原生标签，832/832 sourceRefs有效。3grid和3PCM投影校核<0.01px，仅为源投影校核；原1265非SoundWave对象和原波形/时间标签恢复后保持。两个自动同步/检查失败及恢复均留存；六张full/crop已实际逐图打开。见[设计第19节](08-resource-tools-design.md#19-soundwave的ue电平标尺与原生同步校正)、[RT-S14](08-resource-tools-shared-contracts.md#rt-s14soundwave的ue电平标尺)和[新browser重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-value-grid-02/fresh-reopen-r04.json)。本轮实际Shared已rev2/41对象/4main/**34colors/3typographies**，实际UUID与token-source已只读记录，旧rev1零styles保留历史；本轮未自动消费或编辑公共palette。插件registry与Shared在各有界操作前后hash不变。没有Rust/ZUI/服务/桌面/编译，official source/真实播放provider/EXE/实际DPI和双端比较仍open，passed/accepted=false。

Parent AI第五次交接已收：资源域已提交[RT-AIREF01只读provider提案](08-resource-tools-design.md#18-ai黑板消费的资产只读provider提案)，限既有AssetRef/UUID/URI/type/catalog/DocumentToolkit及依赖状态。33份源码核查、真实Vampire Texture UUID的5项磁盘输入指纹保持；持久依赖列表虽空，live catalog/索引覆盖/AI语义关联未知，完整引用总数仍null。没有新建索引/schema/history、未改公共core/catalog或AI语义、未调用save/runtime。AI负责BT→schema/node/parameter/provider，Parent负责原host及跨文档history/原子提交/保存准入，现缺口下AI继续拒绝rename/type/delete。此轮文档提案不提升全族passed/accepted。

最新接续已按 Parent followup-04 核对：17:13:27Z～17:13:38Z，在隔离 headless 的原账户官方 Plugins manager 中 **Open 已安装的 19002 插件**，实际出现 ZUI Asset Bridge。83 条插件 registry、08 rev76/1337对象及 Shared rev1/41对象的前后哈希均一致，未点击 Install/Allow/导入/导出、未发生拦截到的写请求；browser PID28172 native exit0。四个 canonical 记录的 version1 仅是记录 schema，零 color/typography IDs 和必填 Property.value 缺合规输入组件的限制保持。[真实 Open、32份源码核查及 typed 合同](08-resource-tools-design.md#17-已安装插件open与typed资产合同)已落盘；Audio 四类与独立曲线三类均为文档/接入提案，没有创建 Engine 资产或注册新 type/codec。当前不再等待首次文件/库分配，official source-closure/产品应用、原 Jenkins 聚焦测试与真实 EXE 验收继续 pending，**passed/accepted=false**。

本轮SoundWave真实源内容已保存到同一08 file/page **rev76/vern0：1337对象、551flow、18设计板、4本域main、26copy（5 foreign）**。WOC原始ui_click.mp3在内存解码为512组包络，形成可编辑原生向量及900/640原生实例；19次保存ACK、48/48实例source refs和关闭后新browser重开核对通过，旧1261非根对象/3组件定义保持。三尺寸图已逐张查看，640完整URI在原字体下换行。当前仅是有真实来源的设计内容，完整UE工具/播放/provider/官方导出与Engine仍open；本轮没有产品ZUI/Rust/Shared修改、编译或桌面操作，**passed/accepted=false**。身份、实际MP3 importer、样式差异及失败接续见[设计第16节](08-resource-tools-design.md#16-soundwave真实源内容与原生组件)，精确公共需求见[RT-S13](08-resource-tools-shared-contracts.md#rt-s13soundwave源内容与公共消费边界)。

本轮最后管理入口实查为 2026-10-24T17:01:30.283472+00:00：实际installed已选中 **d2873ff353ea93d6b76f7d7caa12878761146e326070fd4c4526b73f3dc23112**，11/11 capsule/client和197/197 immutable driver匹配，原Audio identity/driver/runtime保持。只运行该真实pinned entry的纯parse_cargo，原精确Sound recipe仍返回 **build_scope_policy**；当前增加particles，仍无sound editor/mixer_authoring_。没有重放QueryOnly、提交、source capture或Cargo，[实际只读回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/jenkins-scope-refresh.json)保留。原Jenkins owner的新事实消息ACK **01a16edc-be0a-7843-82ad-2aab36e5f432**，仅排队、无已读/准入；原消息17:06Z文案笔误已用实际17:01:30证据[明确校正](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/jenkins-owner-time-correction.json)，校正ACK **01a16edd-85ec-78d0-ab8b-afff4782c15f**。原文和历史2790/QueryOnly结果均不覆盖。

历史15:53Z Jenkins r2接续：按Parent原回执确认恢复，不能作本域测试通过。本次实际选中capsule **2790fb51…**，11/11文件及197/197 immutable driver逐项匹配，原Audio identity未改；原命令加QueryOnly得到native exit2 / **build_scope_policy**，当前配方含材质/Prefab/AI/两项glTF，但仍无sound editor的mixer_authoring_。这不是Parent历史查询的authorization pending，Cargo/测试执行均0。RT-AUDIO01-J01已成功排队给原Jenkins owner，尚无已读。隔离API独立确认08仍rev57、Shared rev1/四main/零styles、原Search7/7引用及整份资源raw响应保持，未改页面。真实provider源链、缓存缺口和回执见[设计第15节](08-resource-tools-design.md#15-jenkins-r2原身份查询与真实texture-provider接续)。

历史15:43Z接续：Parent followup-03的13:56Z/rev47读回已收件，保留为历史；本域已继续至 **rev57/1262对象/524流式对象/15板/3本域main/24copy（5 foreign）**。修正原Import草稿把所有来源写成复制到models的错误，并给Picker保留真实UUID/type未解析原因；仅4个既有text改变，其身份/几何/文字样式/metadata与另1258对象保持。原生字形缓存也已按实际字体重测，独立新browser读回和逐板可见SVG核对完成。Dialog实查只有一个root、无body/actions槽，Picker/Import仍是内容研究，非可复用modal；动画合同和Parent四列表源码写回已接收，不重复等待或套用旧r02。详见[设计第14节](08-resource-tools-design.md#14-parent回执接续与importpicker真实能力修正)。本轮产品ZUI/Rust/共享源未改、未编译、未占桌面；**passed/accepted仍false**。

公共库最新接续（2026-10-24T15:00:32.993Z）：本域实际 **rev55/vern0、1262对象、524流式对象、15设计板、3本域main、24实例（5个foreign）**。在专属受控板原生试接Shared rev1的Button、SearchInput、TreeRow、PropertyEditorRow四主件，保存、关闭并由全新browser/context重开；4个copy的33个shapeRef及源元数据保持，原1224个非根对象/3业务main/旧SearchInput未变。Button本地文字覆盖→原生reset→再次覆盖已真实落盘；新增5对原始请求/ACK逐项匹配，但初始建板/实例ACK漏捕仍保留，不称55次全覆盖。共享主件/主题未改，Search内部宽度、TreeRow props、Property必填slot及官方导出/EXE仍open。见[四主件接续](08-resource-tools-design.md#13-四个已发布公共主件的受控消费)及[校核回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-components-trial-01/validation-r02.json)。**passed=0、accepted=0**；下述rev47及更早为历史。

Audio源码阶段（历史14:35Z）：已在sound editor域内实现[真实Mixer snapshot只读适配器](../../../zircon_plugins/sound/editor/src/authoring/mixer_snapshot.rs)，保留重复轨道ID、parent与并行send、原效果顺序/controls及未知meter；不创建第二份音频模型。四Rust路径及三个测试见[设计第12节](08-resource-tools-design.md#12-audio最小只读适配与真实资产文档提案)。格式/diff/change-guard通过；一次正式托管入口在dispatch前拒绝harness参数，随后只读parser核实当时20b3配方仅准material_graph_/prefab_/两条glTF，不含sound editor。未再次提交或编译，Rust测试保持未执行；精确准入提案交Parent/公共owner。Cue/Class/Submix typed资产、播放、host/history和本域Penpot/EXE仍open。Audio全族作者业务归08，没有另一待分配Audio会话。

RT-S10当前状态明确为 **`source-fixed / static-passed / managed+GUI-pending`**，修复producer为Parent。原scroll owner五处源码指纹匹配，已将树/内容/引用/被引用四state的clamp结果经原UI写回；Parent的rustfmt、diff与五路径change-guard通过，不能记为Rust执行。五个测试候选和原请求仍归Parent，两次capture在Cargo前失败；selected reveal/HomeEnd/focus-return/旧async结果拒绝和EXE仍open。动画RT-S08/09已交精确来源与迁移提案，见[当前接续](08-resource-tools-design.md#parent第三次接续滚动写回与动画合同)，不再请求重复交付；原六events/routes保持，公开domain/session slot、Stop/Seek新参数、typed曲线/声音资产及真实消费仍缺。

此前Parent角色CS-XR接续已纳入[单Browser合同](08-resource-tools-design.md#cs-xr-接续真实跨域导航及焦点返回)和[RT-S06补充](08-resource-tools-shared-contracts.md#rt-s06-补充cs-xr跨域导航)：真实registry/command/host/domain-ready与当前generation焦点结果分开；Static Mesh/Texture仍08，Knight九绑定和既有SkinN Data引用完整保留。1+9源绑定独立核对，9场景未运行、31项live字段null；该CS-XR轮未改Penpot/生产源/公共registry，当前Browser优先级不变。

历史接续（2026-10-24T13:41:30Z）：同一08 file/page实际 **rev47/vern0、1225对象、521 flow、14板、3本域main、20实例（含1个公共foreign实例）**。新增真实Vampire registry/UUID输入板；复用原AssetTile并修复长名行框；已链接Shared rev1并消费原生SearchInput，保存关闭fresh重开及1×PNG逐图核对完成。原1163非根对象及3原main保持，四份生产ZUI未改。live catalog/resource/provider、官方source closure、托管测试/Engine/144DPI仍open；SearchInput内部宽度未随832外框伸展。具体身份、原121控件/11事件、滚动提案及证据见[单Browser接续](08-resource-tools-design.md#11-单个真实browser的身份与接入合同)。**passed=0、accepted=0**。下文rev40及更早为保留历史。

历史接续（2026-10-24T12:51:47.854Z）：分配文件实际 **revn40/vern0、1164对象、495流式容器、13板、3个本域原生业务组件、18链接副本**。BrowserContent、AssetTile之外新增TextureSourceContent draft v2；三尺寸纹理源图/Details使用真实WOC base.png，已保存、关闭、另一新browser/context重开，240个新增ID、27项实例文本覆盖、Fit 61%/44%/34%及源格式自动换行保持。纹理网格和源图各三张原板1×PNG已由原生Export UI生成并逐张查看；六次浏览器下载均canceled，只读取得原始PNG Blob，不计下载成功，更不计官方ZUI导出。40对保存ACK严格核对；早期比较脚本错取同一输入的失败保留，现用三份独立修订/哈希重新证明旧几何和文字未变。公共库同次仍revn0/零组件样式；640目录选中可达性、共享引用、官方源导出、真实业务及托管/正常EXE双端验证仍open，**passed=0、accepted=0**。见[最终接续](08-resource-tools-design.md#最新接续revn40纹理源图与原生png)。下文revn28及更早数据为保留历史。

本会话负责 Content Browser、Content Drawer、资源/路径选择、导入与重导入，以及适用的静态网格、纹理、音频、数据表和独立曲线窗口。用户本次明确分工优先于 [overview](overview.md) 中早期把 Texture/Static Mesh 放在材质域的初始入口。材质、动画、Widget、AI 等专用资产通过其 owner 注册的 toolkit 跳转，不在本域重复实现。

此前阶段已完成源码入口与当时能力核查，具体设计接续于 [源布局与接入设计](08-resource-tools-design.md)，共享需求见 [消费者合同](08-resource-tools-shared-contracts.md)。2026-10-24T12:13:15.014Z独立认证读回实际Penpot **revn26/vern0：684对象、261流式容器、1本域原生业务组件及5链接副本**。七板覆盖Browser三个尺寸、Drawer内容面、中文/实际长路径、Model选择器和模型导入默认内容；九个模型文件名/大小和十四个目录来自真实WOC资产。正常保存、关闭、新browser/context重开及七张浏览器图逐图核对已经发生；原六区域/六text UUID保留，rev22原546个非根对象全部属性在rev26保持。共享组件/样式、官方ZUI源metadata与导出、业务交互和正常Engine EXE仍缺；**完整设计、产品修改、原生捕获均未完成，passed=0、accepted=0**。12:13Z公共文件仍revn0、单根、零组件/样式。公共主题/框架归Parent，曲线/时间轴/播放控件归动画chat `01a16d51-741b-73b3-a61a-2189cd38f81e`；新增窗口收件见RT-E01…11。本域未占用共享桌面。Parent的rev22通知因队列100条上限拒绝，未送达；rev26新事实写入本域交接，不反复重投。

前阶段曾观察到本机无 18080、Penpot 常用服务端口监听，未发现活动 Zircon/Java/Node/PostgreSQL 窗口或进程；当时正式 Jenkins `status` 返回 `degraded`、`ownerLost=true`、`serviceReady=false`、`canStart=true`。通过正式入口尝试恢复一次，入口因 `JenkinsError: Original runtime evidence changed` 拒绝；没有启动成功或取得产品测试回执。原 driver、runtime、恢复请求及历史证据保留，未修改它们来通过准入。这些是历史观察，不能代表 Parent 统筹恢复后的当前状态。

随后读到新到的 [主工作台交接](main-workbench-handoff.md)：真实 Penpot P21/Runtime11 原进程已缺席、原退出码未知，15432/16379/16060/19001 已不接受连接；P21 有历史保存/重开证据但缺官方导出。按该交接，服务恢复与桌面操作等待 Parent 统一协调；不再独立尝试启停，不把旧 ready/旧保存记录当作当前本域可用或通过。

本阶段只读指纹核查发现 `.jenkins/state/deployment/driver.json` 已在 10:06:53Z 改变；未将基线哈希覆盖成新值，也未探测/操作服务。34项输入中33项匹配，变化项单独登记在生命周期清单，当前服务状态交Parent核实。本域尚无新的Jenkins执行回执。

- 当前 checkout：`E:/Git/ZirconEngine`，分支 `main`，读取时 HEAD `75a76ee963792d5a316f738519e966937f770036`。已有大量主工作台、插件、Runtime 和原生证据改动，全部保留。当前修改限本域四份文档及sound editor只读适配器四个Rust路径，不做Git写操作；公共runtime/registry/capability/validator及四份原ZUI未改。真实Penpot草稿及隔离浏览器证据登记在本任务metadata。
- 生命周期清单：`E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/artifacts.json`。未分配编译目录、未创建源码副本，实际释放 0 字节。
- 不把本文、TOML 可解析、bridge JSON、mock-backend 截图、注册的空窗口或旧主窗口证据当成本窗口族验收。

## 参考身份

只读参考根为 `E:/Git/ZirconEngine/dev/UnrealEngine`。该目录没有自己的 `.git`；`git -C dev/UnrealEngine rev-parse` 实际回到 Zircon 仓库，不能把上述 Zircon HEAD 写成 UE 上游 revision。`Engine/Build/Build.version` 实际为 `MajorVersion=6, MinorVersion=0, PatchVersion=0, BranchName=UE5, Changelist=0`，不足以确认上游发布或提交身份。当前以实际文件 SHA-256 固定引用，关键文件哈希登记在任务清单；上游 revision 仍待 Parent 的参考树 provenance。

以下表格中的 `E/` 表示 `dev/UnrealEngine/Engine/Source/Editor/`，`P/` 表示 `dev/UnrealEngine/Engine/Plugins/`。行号为本轮读取位置，后续修改需复核。

| 参考边界 | 实际源码入口 | 读取到的合同 |
| --- | --- | --- |
| 通用资产 Toolkit | `E/UnrealEd/Private/Toolkits/AssetEditorToolkit.cpp:79,150,235,265,370` | 资产 major tab、独立宿主、菜单/工具栏、关闭请求和资产编辑对象生命周期；跨族 owner 为 Parent |
| Workflow TabFactory | `E/UnrealEd/Private/WorkflowOrientedApp/WorkflowTabFactory.cpp:18,35,87` | PanelTab、singleton 策略、标签/图标/tooltip、可选内边距；本域各 Toolkit 多为直接 RegisterTabSpawner，不假称均使用同一个 TabFactory |
| Browser/Drawer 注册 | `E/ContentBrowser/Private/ContentBrowserSingleton.h:41,210`；同目录 `.cpp:110,161,174,181,188,209,243` | 最多四个 Browser nomad tab；资源/路径/集合 picker；Drawer 复用同一 `SContentBrowser(IsDrawer=true)`；Open/Save 对话框 |
| Browser 组合 | `E/ContentBrowser/Private/SContentBrowser.cpp:350,420,791,877,905,3670` | Sources/Collections、拆分区、面包屑/导航历史、搜索过滤与资产视图；Drawer 拖出时的关闭行为。`E/UnrealEd/Private/UnrealEdGlobals.cpp:140` 的 NewStyle 默认 true，源码保留新旧布局，首版按新版，不混用 |
| Browser 数据表面 | `E/ContentBrowser/Private/SPathView.cpp`、`SAssetView.cpp`、`AssetViewWidgets.cpp`、`SFilterList.cpp`、`AssetContextMenu.cpp`、`PathContextMenu.cpp` | 目录树、多种视图、行/缩略图、过滤器、资产和目录不同上下文菜单；须继续按具体交互核对，不以文件存在判为 Zircon 已实现 |
| Picker/Dialog | `E/ContentBrowser/Private/SAssetPicker.cpp:81,126,388,770,982`；`SAssetDialog.cpp:109,116,121,129,508,945`；`SPathPicker.cpp`、`SCollectionPicker.cpp` | 初始焦点、后端类型过滤、激活回调、路径/资产上下文、历史与 Open/Save 结果 |
| 内容样式 | `E/ContentBrowser/Private/ContentBrowserStyle.cpp:48,56,106,124,129,144`；`E/EditorStyle/Private/StarshipStyle.cpp:7009` | 当前局部 ContentBrowser 样式已接管部分旧 Starship 条目；源树保留迁移注释，不能只读旧 Starship 段落猜样式优先级 |
| 静态网格 | `E/StaticMeshEditor/Private/StaticMeshEditor.cpp:172,179,277,319,333`；`E/StaticMeshEditor/Private/SStaticMeshEditorViewport.cpp`、`E/StaticMeshEditor/Private/StaticMeshEditorActions.cpp` | Viewport、Properties、SocketManager、Collision、PreviewScene、SecondaryToolbar；事务、重导入与布局。命令实际在 Actions.cpp，修正前轮误写的 Commands.cpp |
| 纹理 | `E/TextureEditor/Private/TextureEditorToolkit.cpp:75,235,281,369`；`E/TextureEditor/Private/Widgets/STextureEditorViewport.cpp`、`E/TextureEditor/Private/Models/TextureEditorCommands.cpp` | Viewport/Details/Oodle/Platforms；通道、mip、曝光、纹理类别及重导入事务。视口/命令并非 Private 根文件 |
| 音频源 | `E/AudioEditor/Private/AssetTypeActions/AssetDefinition_SoundWave.cpp:72`；`SoundFactory.cpp`、`ReimportSoundFactory.cpp` 位于其 `Factories/` | 有已注册 SoundWave opener 才进入自定义编辑器，否则真实 `FSimpleAssetEditor`；不能把 UE 的所有音频资产都当成 Waveform Editor |
| 波形编辑 | `P/Editor/WaveformEditor/Source/WaveformEditor/Private/WaveformEditor.cpp:44,1159,2138`；同目录 `WaveformEditorCommands.cpp` | Properties/Transformations/Display；播放、暂停、停止、选区/loop、缩放、导出和重导入方式 |
| 音频图形族 | `E/AudioEditor/Private/SoundCueEditor.cpp:40,52,112`、`SoundClassEditor.cpp:42,75,136`、`SoundSubmixEditor.cpp:57,94` | Cue 的 Graph/Properties/Palette；Class 的 Graph/Properties/Find；Submix 的 Graph/Properties；不能用普通声音源窗口替代 |
| 数据表 | `E/DataTableEditor/Private/DataTableEditor.cpp:94,127,154,200,277`；`SRowEditor.cpp`、`SDataTableListViewRow.cpp`、`CompositeDataTableEditor.cpp` | DataTable/DataTableDetails 页签在上方栈，RowEditor 在下方栈；撤销/重做刷新及复合表只读限制 |
| 独立曲线 | `E/CurveAssetEditor/Private/CurveAssetEditor.cpp:159,195,218` | 普通独立曲线为单栈；LinearColor 曲线另有 0.8/0.2 水平分区的 ColorCurveEditor；动画曲线仍归动画域 |
| 曲线表 | `E/CurveTableEditor/Private/CurveTableEditor.cpp:89,373,423`、`CompositeCurveTableEditor.cpp` | CurveTable tab、表格与曲线树/编辑、复合数据来源 |
| 导入/重导入 | `E/UnrealEd/Private/Fbx/FbxOptionWindow.cpp:101,113,119,164,178`、`FbxReimportDialogs.cpp`、`E/UnrealEd/Private/SCSVImportOptions.cpp:186,222`、`EditorReimportHandler.cpp` | Import/Import All/Cancel/Reset、初始焦点；CSV 数据/曲线类型与 schema；现有来源的重导入，不等同于再次外部导入 |
| Interchange 关联弹窗 | `P/Interchange/Editor/Source/Pipelines/Private/SInterchangePipelineConfigurationDialog.cpp`、`SInterchangeTranslatorSettingsDialog.cpp`、`SInterchangeGraphInspectorWindow.cpp` | 依赖具体 translator/pipeline/冲突模型，不能以 Zircon 普通文件选择器宣称已复刻 |

## 适用范围

“适用”表示本任务目标，**不表示目前可运行或通过**；“待定”保留目标与缺失能力，由 Parent/用户决定范围；“不适用”只排除明确不在本域或无对应目标的部分。

| ID | 窗口/关联页签 | 分类 | Zircon 依据、边界与理由 |
| --- | --- | --- | --- |
| RT-01 | Content Browser 主实例 | 适用 | `editor.asset_browser`、真实项目 catalog、目录、搜索、筛选、List/Thumbnail、选择已存在；UE 菜单/导航/完整操作仍需验证 |
| RT-02 | Content Browser 2–4、锁定与跨实例同步 | 待定 | UE 明确注册四个实例；当前 ViewDescriptor 为 ExclusiveMainPage。由 Parent 处理宿主多实例合同，不自行更改全局窗口注册 |
| RT-03 | Content Drawer | 适用 | 原始 `unreal/components/content_drawer.zui` 与真实 workspace 适配器；目录树/面包屑/上下文/完整缩略图与双击链尚未还原 |
| RT-04 | 资源选择器/属性资源弹层 | 适用 | 复用注册类型、AssetUri 和 workspace generation；须有过滤、None/取消/确认结果与焦点返回，不能复制静态资产列表 |
| RT-05 | 路径选择器、Open/Save Asset Dialog | 适用 | 项目资产 roots 和 source authority 已有；仍需实际 modal/result/save-path 实现与覆盖中文、冲突、越界路径 |
| RT-06 | Collection picker、Collections/Favorites 管理 | 待定 | UE 有独立集合模型；本轮未发现对应的持久化集合 owner。先保留设计目标，不使用写死列表 |
| RT-07 | Filter/View Options、资产与目录上下文菜单 | 适用 | 依托真实类型/索引及既有通用弹层；Grid/List/Column 中 Zircon 目前仅 List/Thumbnail；Column、缩略图尺寸等为明确缺口 |
| RT-08 | 静态网格导入选项与批量导入 | 适用 | 现有模型 importer、import flow、project scope、场景 settlement；现有 Quick Import 不是 UE 导入对话框 |
| RT-09 | Reimport、来源替换、冲突/覆盖/取消 | 适用 | 复用 source authority 和当前 import job 生命周期；必须保留资产 UUID、locator 和引用；尚无本轮真实证据 |
| RT-10 | 纹理与声音源导入选项 | 适用 | texture/audio importer 插件存在；仅把实际后端支持的选项接入；当前普通窗口未证明可操作 |
| RT-11 | CSV/JSON 数据表及独立曲线导入 | 适用，后端待实现 | UE SCSVImportOptions/CSVImportFactory 为依据；Zircon 通用 Data 类型不能代替有 schema 的 DataTable/Curve 资产 |
| RT-12 | Interchange Pipeline/Translator/Graph Inspector | 待定 | 当前未核实等价管线模型；不能把 FBX 选项机械套到所有 importer，也不复制整套 UE backend |
| RT-13 | Static Mesh Viewport/Details/LOD/材质槽 | 适用 | Model/Mesh、实时场景渲染存在；无内建 Mesh document toolkit；材质槽跳转交材质 owner，不实现材质编辑器 |
| RT-14 | Mesh SocketManager、Collision、PreviewScene/SecondaryToolbar | 待定 | 须核实对应 socket/collision authoring 与共享 preview owner；UE 隐藏/打开 tab 状态需要保留，不放无效按钮 |
| RT-15 | Texture 2D Viewport/Details/通道/mip/缩放 | 适用 | source image 解码与 texture runtime 存在，`texture.authoring` 扩展入口存在；缺 asset-bound toolkit 与完整编辑命令 |
| RT-16 | Texture Cube/Volume/Array 与 Platforms | 待定 | 依赖具体 runtime 资源类别、显示 renderer 和平台 cook 信息；仅有普通 Image 不能算完成 |
| RT-17 | Texture Oodle 专用面板 | 不适用当前目标 | 未核实 Zircon Oodle 编码后端；UE 专属编码选项不做无效果 UI，保留该排除理由 |
| RT-18 | SoundWave 属性/真实试听 | 适用 | Sound importer、runtime sound、sound editor output controller 存在；Mixer 窗口不能冒充绑定选中声音源的资产编辑器 |
| RT-19 | Waveform Display/Transport/Transformations | 基础波形/transport适用；破坏性变换待定 | 需真实 PCM/采样长度/播放头/输出及撤销；裁剪/淡入淡出/循环导出需要可持久化编辑模型，不画装饰波形 |
| RT-20 | SoundCue/Class/Submix 图形窗口 | 作者目标适用；真实模型/编译/执行open | Parent 已确认 Audio 全族归资源域；材质域只提供共享 graph 表面。Zircon Mixer 不等价于 Cue/Class/Submix 资产图，仍缺模型、编译/执行、文档保存与撤销证据 |
| RT-21 | DataTable、DataTableDetails、RowEditor | 适用 | 现有 DataTable workspace 只是 preview；实际 typed table/schema、保存事务、行编辑与 undo 仍缺 |
| RT-22 | Composite DataTable、CurveTable、Composite CurveTable | 待定 | 未核实父表解析、行覆盖和资产类型；不能用平面样例表代替 |
| RT-23 | CurveFloat、CurveVector、CurveLinearColor 独立资源 | 适用 | 已有 renderer-neutral CurveModel/Selection/Transform；尚无独立序列化资产、document route、保存及 undo owner |
| RT-24 | 动画时间轴曲线、材质/动画/骨骼等专用资产窗口 | 不适用本域实现 | 由对应会话负责，本域只验证双击按注册类型打开正确 owner，不改为通用预览 |

## Parent 新增收件的窗口组

Parent本次明确：资源08为Audio全族作者业务owner（含Cue/Class/Submix），同时牵头OVW-12/22/24/25/26/35/37/38/41/44的盘点、设计与业务适配；OVW-36映射既有Audio收件，共11项，不计成11个新增完成窗口。Font/FontFace资产文档归资源，Widget只消费字体；StringTable的key/reference事务适配归资源，Localization入口归设置07。Renderer/neural/provider/跨文档事务与Jenkins生命周期由Parent接续；该分工不授予公共runtime/core写权。Actor类/组件的资产跳转沿真实registry/toolkit，缺后端保持open。

以下接续 [Parent 其它窗口归属](parent-window-coverage.md)，不是把原 RT-01…24 重新计数为已完成。11 条收件中 OVW-36 对应既有 RT-20；一个条目可能包含多个窗口/页签，不能用行数充当整族窗口总数。均未创建产品入口、导出 ZUI 或通过 EXE 验收。Content Bundle Browser 仍由 Parent 牵头，本域只提供资源消费协作。

| 本域接续 / Parent ID | 适用性与 UE 只读入口 | Zircon 真实基础、缺口及边界 |
| --- | --- | --- |
| RT-E01 / OVW-12 Actor Merge | 待定；[MergeActorsModule.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/MergeActors/Private/MergeActorsModule.cpp) 注册 MergeActors nomad tab | Mesh/Model 导入和场景渲染不证明合并作者工具。缺真实 mesh 合并、材质/碰撞处理、目标路径、引用更新和撤销/保存事务；不能只放 Merge 按钮 |
| RT-E02 / OVW-22 Asset Loading Insights | 适用诊断；[LoadingProfilerManager.cpp](../../../dev/UnrealEngine/Engine/Source/Developer/TraceInsights/Private/Insights/LoadingProfiler/LoadingProfilerManager.cpp) 注册 major tab | 已有 import job、project lease/commit 和错误模型；需 Parent 诊断 provider 提供同一 project/session、asset UUID、阶段时间与请求关联。任务进度不能代替真实加载 trace，UE trace 协议不直接移植 |
| RT-E03 / OVW-24 Reference Viewer | 适用只读依赖定位；[SReferenceViewer.cpp](../../../dev/UnrealEngine/Engine/Plugins/Editor/AssetManagerEditor/Source/AssetManagerEditor/Private/ReferenceViewer/SReferenceViewer.cpp) 与 [模块注册](../../../dev/UnrealEngine/Engine/Plugins/Editor/AssetManagerEditor/Source/AssetManagerEditor/Private/AssetManagerEditorModule.cpp) | [AssetSelectionSnapshot](../../../zircon_editor/src/ui/workbench/snapshot/asset/asset_selection_snapshot.rs) 已有 references 与 used_by；[AssetReferenceSnapshot](../../../zircon_editor/src/ui/workbench/snapshot/asset/asset_reference_snapshot.rs) 有 known_project_asset。现有直接引用不是完整递归图、hard/soft 分类或循环策略；只读图表面交材质 owner，打开/定位命令归资源域 |
| RT-E04 / OVW-25 Size Map / Asset Audit / disk size | 适用分析；[SSizeMap.cpp](../../../dev/UnrealEngine/Engine/Plugins/Editor/AssetManagerEditor/Source/AssetManagerEditor/Private/SSizeMap.cpp)、[SAssetAuditBrowser.cpp](../../../dev/UnrealEngine/Engine/Plugins/Editor/AssetManagerEditor/Source/AssetManagerEditor/Private/SAssetAuditBrowser.cpp)；模块注册 AssetAudit/AssetDiskSize/AssetDiskSize2/SizeMap | 可以读取真实 source/artifact/manifest，但尚无这些分析窗口或完整统计 provider。源 bytes、解码内存、打包大小、共享依赖逻辑大小、唯一物理占用分别记账；不能把源文件 Length 冒充显存/独占磁盘或总和热图 |
| RT-E05 / OVW-26 Derived Data Resource Usage / Cache Statistics | Zircon 自身缓存诊断适用；UE DDC 服务管理不适用本域；[DerivedDataEditor.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/DerivedDataEditor/Private/DerivedDataEditor.cpp) 注册两个独立 tab | 现有 artifact/cache 生命周期是数据基础；窗口只读显示真实来源、配置/版本和物理路径/占用。清理、复用及生命周期写命令仍归 Jenkins，不在 UI 另建删除/扫描后端或操作 UE Zen/DDC |
| RT-E06 / OVW-35 Struct Viewer/类型资产 | 只读类型发现/选择适用，Struct作者资产待模型/编译合同；[StructViewerModule.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/StructViewer/Private/StructViewerModule.cpp) 注册 StructViewerApp | [reflection schema catalog](../../../zircon_runtime_interface/src/reflect/schema_catalog/mod.rs) 提供有准入/代次/指纹的中立类型目录；Parent 持有 catalog。类型过滤、返回类型身份和焦点须实际接入；不等于可编辑 User Defined Struct 或 Blueprint 类型编译 |
| RT-E07 / OVW-36 SoundCue / SoundClass / SoundSubmix | Audio作者目标适用、运行后端open，映射RT-20；[SoundCueEditor.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/SoundCueEditor.cpp)、[SoundClassEditor.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/SoundClassEditor.cpp)、[SoundSubmixEditor.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/SoundSubmixEditor.cpp) 各有 Toolkit/tab | Parent 已确认 Audio 业务归本域，材质只拥有共享图表面。Mixer、输出控制和 SoundWave 试听均不能证明图资产、编译、路由/执行及保存/撤销；不再把 owner 当作未知决策 |
| RT-E08 / OVW-37 Font / FontFace | 适用；[FontEditor.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/FontEditor/Private/FontEditor.cpp) 五类 tab；[FontFaceEditor.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/FontEditor/Private/FontFaceEditor.cpp) Preview/Details | ResourceKind::Font 与 [FontAssetMetadata](../../../zircon_runtime/src/asset/assets/font.rs) 的来源格式/face_count/faces/cooked_blob 已有；不证明作者 Toolkit。字体源、解析后的 SFNT/TTC、atlas/glyph、fallback 与 Renderer admission 分别验；不把 Penpot custom-font UUID 当引擎资产 UUID。Widget 消费字体，Parent 统筹字体/媒体 readiness |
| RT-E09 / OVW-38 String Table | 适用本地化资源目标；[StringTableEditor.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/StringTableEditor/Private/StringTableEditor.cpp) 的 StringTableEditor_StringTable 页签 | UI i18n/content 合同不等于有 schema 的可编辑 StringTable 资产。尚缺 typed rows/key identity、导入/持久化、key rename 引用迁移和 undo；设置域只协作 Localization 入口 |
| RT-E10 / OVW-41 User Asset Tags | 管理模型待定；[UserAssetTagsEditorModule.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/UserAssetTagsEditor/Private/UserAssetTagsEditorModule.cpp) 注册 ManageTagsTabId | [AssetMetaDocument/Entry](../../../zircon_runtime/src/asset/project/meta.rs) 和 registry 已有 tags: BTreeSet<String>，不能写成完全没有标签基础。仍缺 editor provider、词汇/重命名/批量事务、权限及 undo；meta tags、gameplay tags、临时 UI filter 不共享错误的写命令 |
| RT-E11 / OVW-44 NNE Model Data | 具体模型检查范围待定；[NNEEditorModelDataEditorToolkit.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/NNEEditor/Private/NNEEditorModelDataEditorToolkit.cpp) 的 NNEModelDataTab | [neural editor plugin](../../../zircon_plugins/neural/editor/src/plugin.rs) 已注册 neural.model、neural.model.onnx importer 和 neural.model.import（有 required capability，remote=false）。这证明注册源码，不证明当前 editor_host 加载、可用模型检查器或 UE NNE 执行兼容；模型格式/执行与 Parent runtime owner 对账 |

新增分析/类型/标签窗口分别保留独立数据与命令权威。通用 Dock、菜单、树/属性行、图表面和主题继续复用现有 owner，不因接收窗口而新增平行框架；新增设计与能力矩阵保持 pending。

## Zircon 实际接入与缺口

资产窗入口是 [asset_browser_view_desc.rs](../../../zircon_editor/src/ui/host/activity_windows/asset_browser_view_desc.rs)，模板为 [asset_window.zui](../../../zircon_editor/assets/ui/editor/windows/asset_window.zui)，面板布局为 [asset_browser.zui](../../../zircon_editor/assets/ui/editor/asset_browser.zui)，投影在 [asset_browser.rs](../../../zircon_editor/src/ui/layouts/views/asset_browser.rs)。旧面板含普通标题/行的 Overlay+position 布局，应在真实 Penpot 设计后迁移为流式布局，保留节点/控件/事件身份。

Drawer 根为 [content_drawer.zui](../../../zircon_editor/assets/ui/editor/unreal/components/content_drawer.zui)，开关与焦点返回在 [unreal_content_drawer.rs](../../../zircon_editor/src/ui/retained/dispatch/workbench/unreal_content_drawer.rs)。真实行适配器在 [unreal_content_drawer_adapter.rs](../../../zircon_editor/src/ui/dispatch/workbench/unreal_content_drawer_adapter.rs)，通过 `#[path]` 可达；默认 rg ignore 会漏掉该目录，已用 `rg -uuu --files` 和 `git ls-files` 确认实际文件，未误当缺文件。

当前 Drawer `prepare_content_drawer_assets` 同步 search/view-mode；`reconcile_content_drawer_assets` 从同一 `AssetWorkspaceSnapshot.visible_assets` 按 UUID 回收虚拟行；选择 binding 同时核对 UUID 与 locator；滚动回收时把焦点交给 AssetView owner。这些是应保留的真实能力。**当前被调的 reconcile 仍是固定高度的文本/图标虚拟行；view-mode 值改变不能证明网格缩略图布局已经改变。** 文件顶部另有 `unreal_content_drawer_asset_browser_data`，不能仅凭其函数存在就推断实际挂载路径已使用它。

真实资源工作区权威为 [asset_workspace_snapshot.rs](../../../zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_snapshot.rs)、`ui/project/asset_workspace_state/**`、`ui/host/assets/**`。沿用不可变行代次、UUID/locator、source scope、preview generation；不把行号/过滤后索引当作跨代次资产身份。Find-in-Content-Browser 已有清空文本过滤、必要时清除类型过滤的真实实现，后续复用。

双击当前 Browser 已有 [asset_browser.rs 测试](../../../zircon_editor/src/ui/retained/app/tests/drag_sources/asset_browser.rs)，覆盖 AnimationGraph opener 和 stale projection；它不能证明 Mesh/Texture/Sound/Data/Curve opener。实际 [asset_event.rs](../../../zircon_editor/src/ui/host/editor_event_execution/asset_event.rs) 在没有 toolkit 时只返回状态行 `No toolkit is registered`。内建 [builtin.rs](../../../zircon_editor/src/core/asset/type_registry/builtin.rs) 目前仅为 UI 和部分动画类型提供 Toolkit，插件新增 route 仍通过同一个 registry。已检索插件贡献：材质、动画、AI 等已有各自 owner，未发现本域 Mesh/Texture/Sound/Data 专用 asset toolkit 贡献。texture 扩展只注册 authoring surface，sound 扩展注册 Mixer/Acoustic Debug/inspector，不能冒充选中资产的编辑器。

缩略图实际 [generate_preview_artifact.rs](../../../zircon_editor/src/ui/host/assets/preview_refresh/generate_preview_artifact.rs) 只对 `SourceImage` 解码真实 source；内建 Texture 走此路径，其余内建类型主要为 Placeholder。Mesh 实时缩略图与 Sound 波形必须补真实 provider 并保留 source hash/generation；不能用 kind placeholder 冒充渲染结果。

旧 [extension_data_table_workspace.zui](../../../zircon_editor/assets/ui/editor/modules/data/extension_data_table_workspace.zui) 包含 DT_Items 等固定展示；实际 [feedback/data_production.rs](../../../zircon_editor/src/ui/retained/dispatch/workbench/feedback/data_production.rs) 的 open/validate/save 明确仍为 `Not implemented (preview)`。本轮不计为已实现 DataTable。`ui/curve` 的 [mod.rs](../../../zircon_editor/src/ui/curve/mod.rs) 也明确只定义 renderer-neutral 的选择与坐标词汇，资产评估/修改权威属于 runtime；不能直接复制成独立 Curve 编辑器。

## 共享组件提案与身份

遵循 [component-reuse](component-reuse.md)，公共组件、菜单/页签/停靠注册、全局主题及自定义样式继续由 Parent 统筹。Shared rev1已有四个真实主件及本域链接试接；下列完整框架/业务依赖仍是提案，不能由这四个已发布身份推导其它组件或样式就绪。

| 提案 | 复用 owner 与合同 | 本域需要的能力/交接 |
| --- | --- | --- |
| RT-C01 窗口/文档/弹层 | catalog 中 UiHostWindow、DocumentHost、ActivityDrawerWindow、MenuBar；现有 Unreal Dock/状态栏 | Parent 确认 Browser 多实例、asset document 唯一身份、modal/focus-return、Drawer pin/dock；本域只提供内容和命令 |
| RT-C02 数据行与共享输入 | WorkbenchTreeRow、WorkbenchListRow、WorkbenchTableRow、WorkbenchField/SearchInput、WorkbenchDropdown、WorkbenchButton/IconButton | 树节点展开/选择、类型 filter、虚拟行稳定身份、列表与缩略图状态、键盘多选；不注册重复组件 |
| RT-C03 资产 tile/面包屑 | 优先用既有 Button/Icon/Label/flow 与实际 breadcrumb primitive | Parent 确认是否提升为共享 AssetTile/Breadcrumb 组件；需要 image readiness、类型条、截断/tooltip、selected/focused/drag-target 状态与 slot |
| RT-C04 真实预览/曲线/播放 | WorkbenchPreviewViewport、既有Runtime scene/image/sound；curve/timeline/transport共享owner为动画chat | Parent对齐preview跨域合同；本域提供mesh/texture/sound/独立曲线业务。曲线key/tangent、声音transport消费请求见RT-S08/09；不另建共享表面或把静态Image/模板时钟算生产能力 |
| RT-C05 样式 | `editor_tokens.zui`、`unreal/starship.zui`、实际 ContentBrowserStyle | Parent 给共享 Penpot library/style 身份与版本。需要明确菜单/row/tile/drawer 的语义 token，不在每窗复制颜色/字体/尺寸 |

保留现有 `UnrealContentDrawer`、`ContentDrawerHeader`、`ContentDrawerControls`、`ContentDrawerAssetBrowser`、`SearchEdited`、`SetViewMode`、`ImportModel`，以及 `AssetBrowser*`、`WorkbenchExtensionDataTable*` 的可用稳定身份。出现相同 control ID 时以 `(sourcePath, sourceNodeId, instancePath)` 全元组定位导出 owner，不按标签或全局字符串替换。

实际Penpot file `b4a1e3ce-9e6a-81fe-8008-d69d60ef2172`，page `b4a1e3ce-9e6a-81fe-8008-d69d60ef2173`，直接入口及Parent分配回执见共享消费者合同；分配时revn0、vern0、只有初始页。原结构根board `6474610e-e630-805b-8008-d6a355a690ba`保持；当前rev55的业务/公共component、main、copy、library rev与sourceRef已逐项记录于[设计第13节](08-resource-tools-design.md#13-四个已发布公共主件的受控消费)。shared style、canonical sourceHash/slot/官方导出闭包及EXE加载指纹仍**未取得/未验证**。不把已发布库、结构board或旧主窗口证据当成完整设计/产品加载证明。

## 布局与交互目标

此段是源依据与接入约束，**不是已经完成的可编辑 Penpot 文件**。按用户指定 UE 为视觉/行为权威；项目参考映射为 [File Manager](https://pro.reactbits.dev/docs/app-ui/file-manager)、[Filtering](https://pro.reactbits.dev/docs/app-ui/filtering)、[Data Table](https://pro.reactbits.dev/docs/app-ui/data-table)、[App Dialog](https://pro.reactbits.dev/docs/app-ui/app-dialog) 的对应区域/流程。尚未开始新布局实现或读取其嵌入 DOM。

- Browser/Drawer 目标内容相同：上部 New/Import/Save 与导航/路径；左部 Sources（目录/收藏/集合按真实支持）；内容上部 query/type/filter；中央 Grid/List/Column；下部条目数、选择与 view settings。普通内容按 VerticalBox/HorizontalBox/FlowBox 组合；目录和资产区域各一个滚动 owner。Drawer 是锚定主窗口底部的真实 overlay，其 anchor、clip、z-order 和焦点由宿主管理。
- 1280/900/640 宽度保持 UE 信息层次；窄窗通过 split 最小值、Sources toggle 与文本截断处理，不能把静态样本挤到固定坐标。新版 UE toolbar 源为 clip、不 wrapping；Zircon 窄窗 overflow 若采用须作为待 Parent 确认的设计差异，不能称原样复刻。未确定 breakpoint 或 metric 不写成测量事实。字体与行框须用真实 Roboto/中文 fallback 实测，不用截图坐标补偿。
- 资源激活按类型 registry 打开正确 document，单击只选择，双击/Enter 激活；快速过滤/项目切换/虚拟行复用后拒绝 stale identity。Material/Animation 等跳转复用对应会话的 route。资源 picker 的激活是回传选择，不能误开编辑器。
- UE `ContentBrowserCommands.cpp` 明确：Enter/Ctrl+E 打开、Space 预览、Ctrl+Shift+N 建目录、Ctrl+Backspace 上级、Ctrl+S 保存、F4/Ctrl+L 编辑路径、Alt+F 按选择类型过滤。波形视图中的 Enter/Space/Shift+Space 属于 transport，不让 Browser 全局快捷键抢占；通用 Rename/Delete/Undo 与 Parent 命令路由对齐。
- 弹层需记录开启者、默认焦点、Tab/Shift+Tab、Esc、确认/取消与焦点返回；点击另一 pane 时遵循真实焦点转移。目录右键、资产右键、空白区右键分别使用实际允许命令。
- 拖放分别验证导入外部文件、目录间 move/copy、Browser 到属性/场景、Drawer 拖离关闭及撤销；项目 roots/source authority、UUID、引用及保存路径不因 UI 行重排而改变。
- mesh 是真实 mesh renderer 与属性/LOD；texture 是真实 source/mip/channel；sound 是真实采样/输出；DataTable 是真实 schema/行编辑/持久化；curve 是真实资产 key/tangent/undo。未接后端的按钮保持未完成登记，不能以通知文本作为效果证据。

## 证据与验证矩阵

复用 `tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-review-contract.ts` 的 `LayoutReviewCase`、`LayoutRenderEvidence`、`LayoutCaseReview`、`CatalogReview`，不另造通过格式。Case 由真实正常窗口状态与 source identity 生成；设计 fixture 只用于设计，不能提升为实际项目数据。

所有适用窗口均须覆盖以下四个 viewport/DPI 组合；表内全部 pending：

| 逻辑客户区 | 实际 DPI | 物理客户区 | 当前 |
| --- | --- | --- | --- |
| 1280×800 | 96 / 100% | 1280×800 | pending |
| 900×620 | 96 / 100% | 900×620 | pending |
| 640×520 | 96 / 100% | 640×520 | pending |
| 1280×800 | 144 / 150% | 1920×1200 | pending；必须由 GetDpiForWindow/捕获回执确认 |

| Case 组 | 必须覆盖的真实状态/效果 | 当前证据 |
| --- | --- | --- |
| RT-V01 公共状态 | default、无资产/过滤无结果、selected、多选、focused、中文/长文本、scroll-start/end | 无 |
| RT-V02 弹层 | filter、view settings、目录/资产/空白 context、picker、import/reimport；open/关闭/取消/焦点返回 | 无 |
| RT-V03 Browser | 树展开/导航、面包屑跳转、历史、query/type 组合、Grid/List/Column、缩略图大小/失败/ready、双击正确编辑器 | 无 |
| RT-V04 Drawer | Ctrl+Space/按钮开关、search focus、Esc/外部点击、滚动回收焦点、reveal selection、拖出与 dock/pin | 无 |
| RT-V05 路径/导入 | 活跃项目真实 roots、源与目标、中文/长路径、同名覆盖/取消、失败/重试、重导入 UUID/引用保持、切项目拒绝旧作业 | 无 |
| RT-V06 Mesh/Texture/Sound | 真 mesh 视角/选中/LOD；texture RGBA/mip/缩放；sound 播放/暂停/停止/真实时长与播放头 | 无 |
| RT-V07 Data/Curve | schema/行筛选/编辑/保存/重开/undo/redo；curve 新增移动删除 key、tangent、frame/zoom、save/reopen | 无 |
| RT-V08 修改生效 | 同 file/page/component 的真实 Penpot 样式改动→保存关闭→fresh reopen→官方 ZUI 导出→原始叶子精确应用→正常 EXE 加载→前后截图变化 | 无 |

参考正常窗口入口为 `tools/analysis/visual/capture-editor-normal-frame.py`、`record-editor-normal-window-evidence.py`、`normal-workbench-case-producer.mjs`；保留 HWND/PID/birth、EXE/DLL、资产依赖、nonce/hold/readback、source/generation、实际 DPI、输入游标、release/ACK 和自然关闭结果。不启动替代 capture binary 或 offscreen 图来通过正常 EXE 验收。脚本读取到可用不等于本轮已经调用捕获。

每图交付官方 Penpot PNG、完整原生 PNG、并排图、差异图、几何/文本行框测量及逐图 review。`LayoutRenderEvidence.status=passed` 需要真实字体/图标/媒体就绪和有效捕获；双端同状态、输入/dependency hash 一致且逐图复核通过后才 `accepted`。关键边界/文本行框目标 ≤1 logical px；未测量写 unknown，不能写 0 或 accepted。

真实项目候选为 `examples/woc/zircon-project.toml`：存在 `res://m8/models/props/crate_wooden.glb` 和 `res://m8/audio/sfx/ui_click.mp3`；另有 `examples/vampire/assets/textures/jungle_ground_albedo.png`。它们不能直接混成一个活跃项目清单。WOC 的 sound selection 当前只标 client_runtime，Vampire 的主要 selections 也只标 client_runtime；实际 Editor 预览前需要核验所选项目 editor_host 能力，不能假设插件已加载。没有修改真实示例或创建假项目/夹具。

## 待执行的聚焦检查

当前是实际Penpot API/native作者与源码/文档核查，尚无本域托管Rust、插件功能执行或产品捕获通过。早期start拒绝保留为历史；Parent最新公开capability为ready，最小托管验证授权延续，仍需对账原session/source capture和受管family输入。不自行重启、不用本地Cargo/生成或更换capture请求身份替代；下面按实际变更选择范围，不是已提交/已通过记录：

- 插件：复用其受管 workspace，只选择 zui-review-case、zui-source-export、zui-export-apply-sources、组件 identity/flow 与实际改动涉及的 spec；真实会话用现有 `zui-layout-real-session.ts`，保留 save response 和 fresh authenticated reopen。render-only/mock backend 不能替代真实保存。
- 资产状态/路由：`zircon_editor` libtest，关闭 default features；过滤 `tests::editing::asset_workspace`、`browser_double_click_`、`ui::layouts::views::` 中 asset_browser owner、`unreal_content_drawer_adapter::tests`（执行前以 --list 核实真实模块路径和匹配数）。
- 导入持久化：同 package 的 `registered_project_scope`、asset import flow、模型 settlement/项目切换、relocation/delete/reference guards；只在相关行为被改动时提交对应批次。
- 后续专用 toolkit：依次核实 owning plugin package，再选择其精确 libtest 与 runtime 叶依赖；不把整个 Editor/Runtime 所有插件当作测试范围，不运行全仓矩阵。

支持入口示例为 `tools/dev/dev-module-build.ps1 -Package zircon_editor -Action test -LibTests -TestFilter browser_double_click_ -Features @() -Jobs 1 -TestThreads 1`；默认 test profile、无默认 features、目标仅 libtest。**尚未执行此命令**；实际执行前仍须向用户声明模块、features、profile、filter、命令与覆盖理由，并确认物理输出位于项目批准的 `E:/cargo-targets`。最终产品、符号、共享构建产物和 incremental cache 分开计量。

## 开放差异与依赖

最近已保存并独立重开的原页为rev107，共24个设计板；各板覆盖范围与失败见原分阶段记录，新增OVW窗口仍未形成完整可编辑设计。所有本轮Editor图、并排/差异图均未取得；每项继续open，已有设计图和源码合同不关闭运行/验收缺口。旧rev57/14—15板与旧容量观察按历史保留于[本次台账校正原行](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/delivery-readiness-01.json)，不得作为当前readiness。

| 差异 ID | 目标/状态与 UE 参考 | 实际差异与已证实原因 | 依赖 owner / 下一步或待设计决定 |
| --- | --- | --- | --- |
| RT-G01 | 全族/全部尺寸，真实Penpot保存/重开链 | 最近实际API及fresh reopen为原页107/2002对象/698flow/24板、6本域main、36实例（25本域+11 Shared）、1200有效sourceRefs；Shared5/4main/34色/3字样。rev57/15板仅为历史，不代表当前；本轮只核既有证据，未新查API或保存 | 继续原对象/原source/原事件；canonical owner/slot/style codec与官方闭包未完成，24板数量不代表24板或全族通过 |
| RT-G02 | 全族/全部状态，受管插件/Rust验证 | RT-AUDIO01-CURRENT-CLIENT-MANIFEST-01的02:54Z回执已独立读核：c1a5a7ca…/entry3c9eba5a…、12pins与8组local/Home匹配；#448/queue15428实际128管理case（含16 Sound）全过，native root0/Job0/双EOF/readers/waiter闭合。旧2d2debd7…/57153be4…缺manifest仅保留历史。原Audio identity不变，3产品Rust仍0执行，无intake/source capture/reservation/pool-cache准入 | 客户端manifest这项缺口已修复并有管理检查；资源/source/provider/产品native及EXE仍独立open。保持原request/operation，不重放未知提交、不新attempt/QueryOnly；见[本次收件](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-current-client-manifest-intake-01.json)，其余托管范围沿原owner |
| RT-G03 | Browser/Drawer 主体；SContentBrowser/SAssetView | Browser 旧布局含普通内容绝对定位；Drawer 当前被调适配器为固定虚拟行，不等于 Grid/Column | 本域在真实 Penpot 设计后改原始 ZUI/adapter；共享 AssetTile/Flow/virtualization 提交 Parent |
| RT-G04 | Browser 多实例、dock/pin/锁定 | UE 四实例，当前入口 ExclusiveMainPage；未有全族宿主证据 | Parent 确认实例/停靠合同；本域不更改全局注册 |
| RT-G05 | resource/path/open/save picker，焦点返回 | 已保存Model picker业务内容及第五BrowserContent副本；确认因Runtime UUID未知禁用。尚无真实modal/result、类型限制、取消不改调用者或保存路径验证 | 本域提供candidate UUID/locator/type/project/revision消费；Parent共享modal/focus/body-actions；不把设计副本当已绑定全局Browser状态 |
| RT-G06 | 目录/面包屑/历史/集合/上下文/快捷键/多选 | RT-S10为source-fixed/static-passed/managed+GUI-pending：Parent四列表clamp写回、五源码读回匹配，rustfmt/diff/guard通过；五测试未执行。selected reveal/HomeEnd/focus-return/旧async结果拒绝仍open。保留单primary和Find-in-Browser多UUID/行selected；通用Ctrl/Shift、lead/anchor、批量命令/Collections持久化及Column仍缺 | 沿原state/owner接命令与真实选择，不建第二scroll/selection store；clamp不作完整首尾/选中可达性证据，字段/边界见设计第9、11节 |
| RT-G07 | Mesh/Texture/Sound/Data/Curve 双击 | 当前内建无Mesh/Texture/Skeleton等专用Toolkit，当前Engine的enabled插件registry未取得；无Toolkit的源码路径返回状态行。ActivateReference只导航，不等于OpenAsset/文档ready | 本域真实document/command就绪后由Parent沿原registry登记；Static Mesh/Texture归08，Skeletal/Skeleton/Physics归角色，Material归材质；见RT-S06补充 |
| RT-G08 | Mesh thumbnail/preview | 当前非纹理内建 provider 主要为 Placeholder | runtime mesh preview owner + 本域 provider；禁止将 placeholder 算完成 |
| RT-G09 | Texture viewport/Details/通道/mip | TextureSourceContent draft v2与三尺寸原生PNG显示真实1024² RGBA8源图、源字节/opaque alpha；GPU格式/资源大小/mips仍未知，Save/Reimport/通道/mip/Fit均未接业务。texture.authoring注册不是asset-bound toolkit | 本域接真实文档/保存/preview adapter，Parent接宿主/公共属性；Cube/Volume/Platforms待定，Oodle排除；源图不能冒充GPU预览 |
| RT-G10 | SoundWave/Waveform | Mixer/output controller及真实轨道图配置/校验/部分Kira编译已有源码；选中声音文档、真实PCM波形/播放头/保存变换未验证 | 资源08负责Audio作者业务，Parent接现有Sound runtime/provider与生产输出；动画只供共享transport/curve；破坏性变换需真实格式/事务 |
| RT-G11 | DataTable/RowEditor/Undo | 原生Tab内容已保存重开100；仍只有四行原ZUI样例，实际feedback保存/校验为Not implemented (preview)，真实typed文档/schema/transaction/persistence未接 | 08负责业务语义/codec提案；Parent负责原registry/document/host及跨文档准入，公共Table/Fields/source closure由原owner接续，见设计33/RT-S22 |
| RT-G12 | 独立Curve typed资产/Surface保真 | 原CurveModel为每curve插值/可选slope，UE逐key插值/切线模式与权重、default unset/外推和颜色7调整无法据此无损表达；原动画Curve主件rev274仍private。Data原文/JSON载体已存在，但Browser只按ResourceKind返回类型 | RT-S25五项精确出口：域codec/semantic subtype与registry、动画lossless Surface/发布、Parent原文档history/CAS/save、同源RGBA/gradient/typed输入；12定义0执行，不建第二曲线/clock，见设计36 |
| RT-G13 | import/reimport 对话框/来源与目标 | Quick Import/后台 flow 有效接口存在，但 UE 弹窗、真实目标及重导入一致性未验证 | 本域与 importer owners；reuse job/project scope，不将再次导入当重导入 |
| RT-G14 | 全族原生尺寸/DPI/中文/状态 | 没有本轮正常 EXE/capture/双端图片/逐图 review | Parent 桌面队列、Jenkins 当前产品、真实 GetDpiForWindow=144 的新窗口；不能引用125%为150% |
| RT-G15 | 样式修改→导出→加载→变化 | Shared已发布34色/3字样；原Button101—104的单消费者native sync、surface覆盖、完整reset和关闭重开已留真实设计证据。该有限消费不证明整库样式传播或完整链；官方导出、原产品加载、EXE指纹及前后图仍缺 | Parent/public owner交受管通用owner runner、source/style codec与正常family producer；本域沿原资产落实，不另建主题、不把Penpot前后变化称产品通过 |
| RT-G16 | UE 来源版本 | Build.version 非上游 commit，参考树无独立 .git | Parent 提供 provenance；当前逐文件 hash 固定，只读树保持 |
| RT-G17 | Audio全族，Cue/Class/Submix图与运行；UE三Toolkit及各GraphSchema | 已实现域内只读SoundMixerSnapshot适配器；格式/静态guard通过，三个Rust测试尚待托管执行。现有SoundMixerGraph/validator/Kira compile可复用；validate_m1_surface仍拒绝效果链/特定高级controls/pre-effect sends，停机configure成功不证明可播放。Cue程序图、SoundClass继承、typed资产/保存/history和本域对应Penpot/EXE仍open | 资源08按[RT-AUDIO01](08-resource-tools-shared-contracts.md#rt-audio01cueclasssubmix图消费合同)交原命令/schema与真实资产提案；材质只供中立surface，Parent接runtime/provider/生产输出/公共host/跨文档事务。不翻capability、不绕validator，三类资产分别验收 |

| RT-G18 | 单Browser当前project/catalog/asset/provider | Vampire GUID/563条持久registry、目标UUID/meta/源PNG/.zasset已固定；current editor activation/catalog_revision/ResourceMetadata.source_hash/state/revision/provider ready未取得。新板只是设计选择意图，无EXE图 | [单Browser合同](08-resource-tools-design.md#11-单个真实browser的身份与接入合同)交Parent准入真实project/插件/snapshot/provider；格式号/旧ready/媒体UUID不作live身份 |
| RT-G19 | Shared SearchInput的fill/原source导出 | 旧Browser foreign外框832、新验证copy外框1232，内部均surface≈240/text≈204；原生/诊断图可见未伸展。原source元数据保持，placeholder文字覆盖尚未证明正式property反导出；当前Shared已34+3styles，本Search尚无实际样式引用 | Parent/pipeline沿原field/layout刷新与guard补内部响应式、SearchEdited及placeholder映射；本域不改公共main/伪造codec，见RT-S10/11及RT-SH4-01 |
| RT-G20 | 跨域静态/蒙皮类型、关联打开与focus返回 | asset_type_id_for_locator仅kind→type，Mesh不可区分；原route仅locator/operation，host changed/effects不是domain-ready/输入焦点。CS-XR九项、31项live结果未运行 | Parent收RT-XR-D01…04精确提案，沿单一catalog/registry/host/PreviewSession；角色1+9源绑定/修复CPU前置不冒充资源ready，不启用未知UUID确认 |
| RT-G21 | Shared TreeRow作为资源目录行 | 原生链接/20 shapeRef保持；catalog未公开text/depth/selected等props，源主件含可见性/锁定等操作装饰。未替换现有目录树或伪造业务状态 | Parent/public owner交批准的内容/state/affordance合同及源闭包，沿原scroll/selection owner消费；见RT-SH4-02 |
| RT-G22 | Shared PropertyEditorRow的value消费 | 原生链接/4 shapeRef保持；required单个linear value槽无已发布合规Field/NumberField等主件，保持空槽、未用占位值完成属性面板 | Parent/public owner发布真实允许组件及slot/export闭包后接实际资源属性；见RT-SH4-03 |
| RT-G23 | Picker/Import可编辑modal与真实确认 | WorkbenchDialog原source/catalog仍无公开slots；public已交未应用提案：复用原生content/actions，内容为单Container，actions保留原Button/IconButton事件。设计中的body区域映射content，不能传未声明body别名。当前两板仍是业务内容草稿，WOC crate无已准入live UUID/type/能力 | Parent沿原modal/focus/render/source owner实现并发布content/actions及默认焦点/返回；本域在原Picker/Import内容中消费，不先改公共root/catalog/主题，见[新source合同收件](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-closure-consumer-01/public-source-contract-intake-01.json) |
| RT-G24 | SoundWave实际导入及播放位置来源 | 原MP3包络与原生内容已保存重开；Engine UUID/admission/session仍null。SoundPlaybackStatus丢失Kira/fallback来源，seek为f32秒→round/clamp；共享finished drain会排空所有播放 | 沿RT-S13/RT-TYPE01接真实importer/provider；只显示来源未知的manager位置，不造timer/FPS，不由资源窗口drain共享队列或关闭共享输出 |
| RT-G25 | Audio四类及独立曲线typed资产 | 32份实际源码指纹、7种文档提案已落盘；DataAsset不证明语义验证，Browser仍kind→type；CurveView整条插值不足以表达UE逐键模式/权重 | 08交业务模型/codec提案；Parent接原registry/document/host，动画补共享表达能力。未注册type/格式/命令，未启用无效果写入，见RT-TYPE01 |
| RT-G26 | AI资产只读provider/语义引用闭包 | G01已在原manager形成单API候选，真实activation/input/catalog匹配、原registry边及12项回归定义已落码；空边coverage仍unknown，实际live查询0，Rust未编译/执行 | G01沿原Jenkins准入待验；G02真实asset→semantic/document/Toolkit、G03覆盖和G04跨文档原子事务仍open。不新建索引或接管AI语义，rename/type/delete保持拒绝，见设计第23节 |
| RT-G27 | Shared原生同步/覆盖/reset | rev84恢复保留历史；当前Shared5，原Button单消费者sync101保留Import、覆盖102为surface.0、完整reset103恢复surface.1/Button；两说明104，fresh reopen104保持。1147总refs有效 | 单consumer/source/persistence已核；整库Updates/全局A/B、CJK/Engine字体、official codec/export继续open，不能把完整reset说成保留文字覆盖；见设计34/RT-S23 |
| RT-G28 | NumberField10props与Property4slots消费 | 新catalog真实hash匹配，6项只读源检查通过；真实schema/value/typed codec/history未交，新输入master未发布 | 依RT-C06保留原DataTableVersionField及events，不套模板42/0..100，不经f32保证u64/i64，不用Label填value。只读/未知能力保持 |
| RT-G29 | 真实Browser长名与类型行框 | 原缓存修正85后，13个AssetTile类名框已14、root vertical Hug；独立保存关闭重开87，三尺寸/长名均分离 | Penpot尺寸/flow已核；正式导出、Engine字体/产品/EXE待验，原失败保留 |

| RT-G30 | Sound Stop终态与来源代次 | 显式Stop与设备退役均可Stopped；Finished/公开output status缺来源代次，private output_generation=0有效。后续Unknown/空队列不能定成败 | 沿RT-TYPE01-P04由Parent原provider/host供产生时原lease/authority与命令返回关联；不在drain时补当前代次、不抢共享队列。仅提案，无新请求/实现/运行验收 |
| RT-G31 | SoundWave main/copy的可靠来源及PathData | 原保存rev80→85的99对象/2,890字段、6原生二进制路径与27标签/63文本缓存均相同；rev78/79的PCM各2条偏离可靠rev76，rev80/85已3/3恢复 | 离线保存证据已逐字段核；public owner后续main/主题更新仍须核每copy、可靠PCM、transform/bounds/缓存/时间标签。当前服务、官方codec/producer和EXE/DPI不由旧响应通过 |
| RT-G33 | 真实目录箭头及公共source映射 | 原页93保留151个UE native SVG/16×16槽+gap4；34真实目录确认85叶子path隐藏，31展开/35折叠分支可见，972refs有效 | Penpot可见性与保存重开已核，14图已看；原public source codec/slot导出及产品load仍pending，见设计30 |
| RT-G34 | 真实目录展开、命中与祖先选择生产链 | Workspace无展开集合，folder_tree始终递归；pointer仅Folder/TreeSurface，整行均SelectFolder。UE折叠含已选后代时选择祖先；叶子隐藏但留槽。仅补expanded不闭环 | Parent原primary owner扩真实state/可见preorder/独立disclosure及同源paint-hit几何；保留view/publication、selection与单scroll owner。见[设计29](08-resource-tools-design.md#29-目录展开状态与命中路由的真实接入边界)与RT-S19-r02；公共产品仍open |
| RT-G35 | 中文字体测量与文字缓存 | rev94原生重测11缓存，自然宽度/DOM textLength相同；18静态检查、6图实际复核，原字体/框/refs及其余1655对象精确保持 | Penpot缓存压缩已修；Engine字体/行框、官方source导出与真实EXE/144DPI/双端仍pending，见设计31/RT-S20 |
| RT-G36 | Browser真实历史导航与CircleArrow | 原页95的12原ID/24 native图形已按UE SVG保存重开，Shared24 stroke refs/992来源有效；当前state/command/ZUI缺导航history，不能视图标即功能 | Parent沿原state/host/binding/focus/source closure接真实历史、capability及多实例，10项定义未执行；原产品、EXE/144DPI和邻板thumbnail刷新仍open，见设计32/RT-S21 |
| RT-G37 | DataTableTab原生设计与共享源 | 三板native业务main/copies及3 Shared Search copies、1147总refs已核；原生flow顺序和选中序号修正，四行样例非真实表资产。Search内部240/204、偶奇image brush、typed输入/RowEditor/toolbar均open | 原源事件不变；Parent接RT-S22五项公共依赖，08交typed业务合同；12验收定义0执行，不能用设计/source静态核对关闭RT-G11或EXE/DPI |
| RT-G38 | 原19002浏览器CORS | 同一manifest GET200/278B/SHA不变，但19001真实Chromium报无ACAO并拒绝；无plugin Open/Install/host或registry变更 | 原public owner沿原host/release处理RT-S23-P01；HTTP200不作plugin ready，official source/export仍open |
| RT-G39 | Static Mesh源设计与真实typed/runtime | 原页107新增main/2copy/三板、GLB真实JSON及两primitive/两material来源；Engine UUID/catalog/type/PreviewSession未知，Mesh0为Model而非primitive Mesh | 08供RT-S24业务合同；Parent沿原registry/host/PreviewSession，public给新owner闭包。源内容不是已接通Static Mesh编辑器，作者/导入/渲染/EXE仍open |
| RT-G40 | 嵌套Shared Search原生同步偏移 | 原生filtered sync-file以原105生成24changes，24/24匹配实际106请求，含12嵌套后代及field delta280/471.578947；4组pure reflow不引入该偏移，107未保存 | 已定位复现producer；canonical flow/嵌套root实际修复、240/204 stretch与完整resize/sync/reopen仍open，不按copy改geometry/touched/refs或放宽guard，见设计37及RT-S24-P03 |




接续顺序：使用已分配真实Penpot文件，复用Parent/动画owner实际组件，完成设计和save/close/fresh reopen，再落实原始资产与真实runtime业务；Jenkins和桌面由Parent/既有owner统筹。当前完成了盘点、部分真实Penpot内容设计与四公共件受控试接；原始资产落地和整个窗口族的产品验收仍未完成。

## 2026-10-25 恢复执行：RT-S19 目录展开实际源码候选

按 Parent 01:02 PDT 的恢复授权继续推进，goal 保持 active；不再把共享 Field/导出/Toolkit 或受管测试等待作为本族停工理由。本轮直接改原 checkout 的资源树链路，精确 26 路径、输入哈希及他人前置改动见报告目录 `tree-disclosure-runtime-01/source-inputs.json`。未改原四份 ZUI、catalog、tokens、主题、其它族或服务。

RT-S19 现为 **source implementation available / static-passed / managed+GUI-pending**：真实目录分支状态进入 AssetWorkspaceState 和不可变快照；分支使用现有 SVG 箭头，叶子不画箭头但保留槽位；点击箭头通过 typed SetFolderExpanded 调用真实 workspace，整行选择与拖放仍走原 Folder route。收起包含当前目录的分支时将选择移到父分支；SelectFolder、导航及 Find in Content Browser 展开真实祖先。旧 publication 请求拒绝。没有第二 scroll store，也没有修改资产文档/history。

10 个聚焦回归已定义，rustfmt、diff 和 26 路径 change-guard 为 0/0/0；r02 补查 Find-in-Browser 和项目根切换，受影响测试 guard 亦为 0/0/0。候选 `candidate-r02.patch` SHA256 `44b047bcd46954e9427198a2b5d4214021a68217e9ee1e86a2fe3f8cb7b33cd6`。旧 r01 补丁及其原身份均保留。

受管验证确已 POST：`ue-assets-tree-disclosure-20261025-01 / validate-asset-tree-disclosure-01`，queue15791 → Jenkins #384。模块 wrapper 实际默认了 D 根和 profile=dev，违反本任务 E 根要求；资源域核验队列三键后中止自己的 #384，08:24Z API 为 ABORTED。同参数唯一 QueryOnly 仍为 running/pending，原 owner 已收到对账请求。原 input/marker/DB 未改，未重投；E/test 正式参数修订提案已落盘。没有 Rust 执行结果可记通过，也不把 job ABORTED 当 native root0/双 EOF。client/worker 实际把 source drift/pins/watch/admission 当 advisory 的原日志保留，本域没有修改策略。

08:25–08:35Z 真实 API 仍为资源 rev107/2002 对象/6 main，Shared rev5/34 colors/3 typographies。151 个可编辑 Path 及 31 展开/35 收起/85 隐藏叶子符合原设计；1200 sourceRef 经原主件子树核对，1186 直接、14 经 StaticMesh 主件内原嵌套 SearchInput，再指向 Shared，未知/缺失均 0。直接单跳检查的局限及纠正证据均保留。这里没有新 Penpot 写入、全历史 PathData 等价或产品加载主张。

仍 open：UE filled caret 与原共享 stroke chevron/实际槽位的双端差异；选中行滚动自动定位、Home/End、焦点返回及整宿主换代拒绝；正式导出/加载、provider、所有目标尺寸与真实 EXE/144 DPI/逐图 <=1px。原 RT-S10、Audio3、G01 的受管与画面门仍按各自原身份接续。`passed=false / accepted=false`，实际释放 0B，必要失败证据和候选保留。


2026-10-25 RT-S25 实作更新：原 DataAsset 上的严格 JSON Curve codec、typed source 和原 CurveModel 借用 adapter 已落七路径；原 G01 exports、四份 ZUI/ID/import/binding/event 和公共树候选未改。13 个内存回归已定义；格式/空白/七路径 guard 0/0/0，五参考 pins 一致，Rust 0 执行。unknown/duplicate/精度丢失、非有限/乱序/旧键/批量失败不写入；默认值/混合模式可保留但当前表面不能表达时明确拒绝。无新资产/UUID/fixture producer/store/history/clock/backend。原 EXE/真实 Curve 资产和语义准入/宿主/保存、共享源导出/三尺寸/144DPI/双端≤1px全部保持 pending。新 codec 计划是独立合法七路径 scope，不重放原 tree 终结 op；详见设计 RT-S25 与 standalone-curve-codec-01 产物，passed/accepted=false。


2026-10-25 RT-S25 正式受管收件补记：上述“拟提交/尚未提交”是先前时刻状态；现同一独立 codec scope 已实际 POST，input66678cf4970efac75240b3b6fe53e772087101d8beb218894dbf49c050b208e9，queue16005→zircon-management#388，client exit0，SESSION/REQUEST/OP 与 queue 实读匹配。真实 INPUT_JSON 为 Editor/defaults-off/features[]/lib/profiletest/standalone_curve_/jobs1/threads1/buildRoot E:/cargo-targets。09:07:27Z API building=true/result=null，完整console有两次resource_capacity/waiting、未见Cargo启动；不将该历史观察扩成之后“确定未编译”或终态。第一launcher WinError2 发生在创建子进程前（0POST）；改用真实Codex PowerShell后仍同identity，只有一次正式入口。7源码当前hash全部匹配，未改产品候选。

Parent 实作交接已 queued01a1724e-6db4-7fe2-9793-01eebd5570e4；原Jenkins owner 资源准入/原身份收件对账已 queued01a17253-074d-7693-8656-5c9f7aae8d25，queued不算已读/准入。原runner soft source/watch/pin日志完整保留，本域没有改policy/DB/source guard、降阈值、换盘/ID、全局freeze或清共享cache；当前没有原请求新POST或QueryOnly。E OS free 76,631,924,736B仅为09:10观察，不是pool/cache/target/reservation证明。最新精确边界见 standalone-curve-codec-01/execution-observation-limits.json、managed-build-observation-r02.json 与 resource-wait-handoff.json。Rust/原生终态、真实Toolkit/provider/保存、正式ZUI/export/load/EXE/144DPI及双端≤1px仍pending，goal active；释放0B，候选/失败/受管作业待验均保留，passed/accepted=false。


## 2026-10-25：G01 r02 同身份源摘要一致性守卫

接续当前 Parent 最小合同，RT-S19 不再以旧“无 fold-set”作现状：原 AssetWorkspaceState 已有 collapsed_folder_ids/可见 preorder/depth/has_children/expanded 和 set_folder_expanded，retained tree 复用 paint disclosure rect，经独立 Disclosure 走 typed SetFolderExpanded/原 host。实际 child folder 判叶、收起选中后代时选祖先、显式选择/reveal 展开祖先沿现接口保留。只读核对两个源 SHA 前缀及完整 hash 已入 r02-handoff；不新写 fold/scroll/history store，不重投 #384。Shift 递归、实际 scroll/present、正式 provider/EXE/144DPI 仍独立待验。

本轮 G01 实改仅原 readonly_query.rs + tests/readonly_query.rs：同 GUID/sub、locator、type 的 catalog 投影若 source_hash 与捕获 registry.entry.source_digest 不同，也返回现 CatalogProjectionMismatch/Stale，不再返回 Known。生产依据为 registry/rebuild.rs 用 meta.source_digest，原 project/import ResourceRecord 用同 source_digest，经 record_projection 与 record_to_view 原样保留 source_hash；只是 opaque 同源字段一致性，不将该值当 bytes SHA256、revision 或保存准入。原 publish_gate→state→index、同 captured input/catalog Arc、锁外返回、unknown/missing/stale 和依赖 coverage unknown 合同不变，未改公共 API/core/registry/runtime/主题/原 ZUI。

新增同路径同类型旧摘要回归，保留旧投影且不触发 refresh/发布；匹配摘要正例维持 Known。G01 原12定义现13，未执行；r02 两路径 rustfmt/diff/guard 全静态通过（0 errors/0 warnings/0 suppressed），7参考 pins 匹配。增量 patch b7ebede156bec26a252ddd15198217722629abdfb5ca586092875addd1881a51，旧 candidate.patch 与全部失败保留。原 Curve 七路径未动。

本轮为 G01 首次正式受管验证准备独立合法 scope：原session01a16d53-60db-7d02-b03e-0042f277783d / ue-assets-readonly-provider-20261025-01 / validate-asset-readonly-provider-01，13个 readonly_asset_，Editor lib/defaults-off/features[]/profiletest/jobs1/threads1/显式 E 根。正常 prepare 本身取得 source/resource/writer/pool/cache/native 回执，不要求先取得 reservation 才提交；parse/capability 或 queue 均不算产品通过。此处仍是提交前记录，后续正式原件以 r02-managed-* 为准；没有复用或重投 Audio/Scroll/tree/Curve 原身份。

原 Curve #388 09:19:46Z API building=true/result=null，同 queue16005/原request/op；不把早前无 Cargo 观察当当前永久状态，本轮仅一次 GET 现有 job 未 QueryOnly/重POST。source、动态测试、Toolkit/实际资产/provider/保存、正式ZUI/export/load、正常EXE/实际DPI/双端≤1px仍各自取真实回执；passed/accepted=false，goal active，释放0B。证据同原 E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-readonly-adapter-01/r02-*，保留 r01 与他人改动。


G01 r02 09:23:15Z 正式入口结果：同 ue-assets-readonly-provider-20261025-01 / validate-asset-readonly-provider-01 的一次 wrapper 实际 exit2，ordinary_entry_unproven，原文为“HEAD, full index or complete source membership changed during capture”。未取得 queue/server receipt；不把该消息改成 authorization pending/resource_capacity，也不从当前原件推断全历史无 POST。保存的消息没有区分 HEAD/index/membership 中具体哪项，不能擅称只是文档变化。未运行 QueryOnly/重复提交/换身份或改 gate/DB/grant/freeze。七份本 G01 已声明产品输入在失败后仍7/7匹配，13测试没有 Rust 通过回执；候选2文件静态通过与动态失败分记。原件 r02-managed-submit-output/result/reconciliation 保留。当前 goal active、passed/accepted=false，自有client已回收，释放0B；原Curve #388和其它原身份另行接续，后续只按合法业务revision/原受管收件。


### RT-DT-SOURCE01 r01 — 真实 DataAsset 只读类型投影候选（2026-10-25）

原 `ui/host/assets/data_table/` 新增只读 `DataTableSourceAdapter::read(&DataAsset, Option<&ReflectTypeInfo>, key_field)`；仅在原 assets/mod.rs 加模块/导出，G01、Curve 原导出及全部原 ZUI/ID/binding/event 保留。调用方须交真实 schema 关联的 struct 行元数据及明确 key 字段；文件名、Data/JSON 格式及 metadata 构造器均不作语义准入。返回原始源行键/顺序、原稳定 ReflectFieldId/元数据及既有 ReflectFieldValue/ReflectedValue，不创建资产、schema、索引、history/store/backend 或 UUID。

首片支持 bool、i8/i16/i32/i64、u8/u16/u32/u64、f32、alloc::string::String。整数从原 token 按实际宽度解析；浮点在任何 f64 投影前检查原 decimal，拒绝精度丢失。未知/缺少/重复属性、重复精确行键、不明确字段 ID/名称、unsupported 列及 populated canonical cache 不一致均拒绝，成功/失败均不改 URI/text/cache。`[]` 可作只读空视图；非 serializable 字段不生成列；editable/range/editor_visible 仍是原 metadata，不产生写权限或可见性授权。

边界：此处是精确 source key，不宣称 UE MakeValidName/FName 大小写/导入去重规则；alias/default 填补/CSV/TOML/嵌套/ref/enum/f64 未实现，不静默转换。真实 semantic subtype/schema association、同代 provider、Document/host/history/CAS/保存准入归原 Parent 合同，当前无真实表资产或行 schema 身份可验。新 adapter 尚无产品 caller，不将其列作窗口已接通。

12 项纯内存回归定义（未注册任何项目/schema/fixture producer）；rustfmt、diff check、7 路 change-guard 静态 0 errors/0 warnings/0 suppressed，9 个只读引用 hash 一致。Patch SHA256 `a992f2a8f1671b837bab57790411f0507907bf3aeedf7c13435184460e1a7269`。Jenkins 初次 DataTable 精确业务请求 `ue-assets-data-table-source-20261025-01` / `validate-data-table-source-01` 将按 Editor/defaults-off/features=[]/lib/profile=test/filter=data_table_/jobs1/threads1/E 根 prepare；此处尚无 Rust/native/product 通过。不是 RT-S19 #384 或 Audio/Scroll/Curve/G01 的换身份重投，所有旧回执和 pending 原链保留。

证据 `E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-codec-01/{source-inputs,candidate-review,static-validation,change-guard,handoff}.json` 与 `candidate.patch`。所有 product passed/accepted=false，桌面/服务/实际 EXE/144DPI/双端 <=1px 尚未执行。


RT-DT-SOURCE01 首次正式提交接续（2026-10-25）：原 `ue-assets-data-table-source-20261025-01` / `validate-data-table-source-01`，input `cf6433f82493a2455b566e00cee8fd52a6ba952828e36a2e71015b875763df8c`，真实 queue16284 → zircon-management#392，client exit0/已回收。实际 INPUT_JSON 核对 Editor/defaults-off/features=[]/lib/test/data_table_/jobs1/threads1/物理E，7产品源当前hash全部匹配。原捕获有9真实并行改动（4动画文档、3canvas-anchor-popup Rust、2prefab Rust）；HEAD/index/membership/cache未变是本次 drift 事件的事实，不倒推旧G01失败。原管理策略仍实际提交，未改gate/源pin/旧receipt/marker或新增freeze/grant/QueryOnly。09:42:34Z #392 building=true/result=null，所保存完整console的9个尝试均为 management_git_busy；此为已观察等待，不断言后续 compileStarted=false。当前 Rust/native/State/source writer/resource reservation/target 均无成功回执。仍 passed/accepted=false，goal active。详见 data-table-codec-01/managed-{queue-receipt,capture-drift,observation,reconciliation}.json。Curve/G01原7输入各6吻合，唯 assets/mod.rs 因本次明确新模块/导出改变，旧pin保留，不将旧scope结果当新全源通过。


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


### RT-S19 filled-caret 共享精确提案（2026-10-25T10:18:35.922261+00:00）

当前原资源页 API 读回 rev107/vern0/2002 objects，真实 closed Path 保留实心 caret-down/right。产品共享 painter 仍指向 toolbar 的描边 chevron。现已交 5 路径、6525B 的只读补丁 `tree-caret-shared-proposal-01/proposal.patch`（SHA256 `d2e0e31f71362ca46875de88ee480cf0befc8091d72f3d0309ce8f47a4a70b37`）；本域未写共享 painter/icons、未修改原树 request/marker。原 UE Starship 源指定 caret-right/down、Icon16x16；参考树未包含对应 SVG 原图，不宣称已逐字复用 UE 图形。当前 Path 平移归一化保留自身几何，原 paint/hit/token/事件不变。

3 份候选 Rust 的 stdin rustfmt、2 SVG XML、git apply --check 均为 0；最终 checkout change-guard 保留 5 项 input.worktree-mismatch，因为补丁尚未由公共 owner 集成；未 suppress/套用补丁求绿。新实心内部像素测试仅定义 1 项、0 执行。产品图标加载、公共 source closure、原始 EXE/实际 144DPI 和 <=1px 比较仍 open，passed/accepted=false。


### RT-SW-SOURCE01 实写与原托管终态（2026-10-25T10:32:15.166619+00:00）

原 owner 内 6 路径已实现借用真实 decoded SoundAsset 的 SoundWaveSourceAdapter 和惰性逐声道包络；fmt/diff/reverse patch check/6路径 change-guard 均通过，10 条内存回归只定义、0 动态执行。patch `0276242a616099b7010ac31fa5c574dfd598ac8e24490d37ca0713e54f9467a5`，完整合同和 pins：E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/sound-wave-source-adapter-01/handoff.json。原 decoded source 不代表 MP3 已导入、asset UUID/catalog/provider/Toolkit、设备输出或 transport ready，未造播放器/时钟/后端。

10:29:29Z 实际 GET 原 Curve #388 / DataTable #392：building=false/result=FAILURE，session/request/operation 与原输入逐项一致；原 console-linked control receipts SHA9f2a79b0… / d24154d3… 均精确匹配。两份原文均为 management_operation_interrupted、HEAD/full index/complete membership changed during capture、status unknown。不得把此前 building/null 当当前或把 FAILURE 改写为单纯 authorization pending；未取得 Rust/native State/资源终态，不重投/修改原 marker 或 input。保留失败，原 Jenkins owner 收真实 lifecycle 对账和透明支持的后继需求。全族 passed/accepted=false，goal active；不因共享缺件整族停工。


### RT-S25 消费实作：资源 authoring signed-zero 无损边界（2026-10-25T10:42:26.657620+00:00）

已独立读原动画 handoff SHA4660fe8d128517735dd46562c56afc8076f3c37d690c41455e5b109c2ee5be74，核两 private 源与 patch acadc92d…均匹配，消费原 animation-shared-controls.md RT-S25 同一 minimum matrix；未修改动画/公共/runtime/host/ZUI/Penpot。动画 queue16509/wrapper0/QueryOnly ordinary_entry_unproven 为其原快照，资源未查询/重投/代验，不当 Rust 结果。

本域另发现并修复原 StandaloneCurveKey/Channel 派生浮点 PartialEq 折叠 ±0，导致 full expected/no-op/candidate/codec 无法保留作者符号位的问题。5 路径补丁 `9de2c7f7d435a8c482273ac25b54800994b4734ccb413f4d800253d7434c53b1`（12403B），一套原类型的位保真相等规则贯穿原 Source/Delta；无新字段/schema/公开签名/store/history/clock。6 新内存定义+原13=19，fmt/diff/reverse patch check0、guard0/0/0，9 read refs及上游身份匹配；Rust 0 verified / actual unknown。真实asset/provider/Document/保存、rich fields/official export/EXE/DPI仍 open，passed/accepted=false。证据 curve-lossless-consumption-01/handoff.json、consumption-matrix.json。


### RT-S25 r03：当前 source / view / mutation 精确边界（2026-10-25T10:58:21.013818+00:00）

本轮在原 standalone_curve 四路径写入共置 [源合同](../../../zircon_editor/src/ui/host/assets/standalone_curve/README.md)、模块文档入口及 4 条纯内存边界回归；不改 adapter/codec/source/equality 的算法、schema 或公开签名。补丁 SHA256 30706a175124d47f2c5aa99c804428b0bb079ce47a1979ee50b22397957983f0（13695B）；2 个已有 raw 前像精确还原，2 个新增路径此前不存在；12 个只读引用 hash 均匹配。原 r02 五源目前 3/5 匹配（mod 文档、tests 新 hook 有明确后继），原 #388 七源仍仅 codec/adapter/error 3/7 匹配，旧输入不回写。

完整源编码与编辑命令现已分开公开：codec 能写合法整源快照，existing-key Delta 仍不能增删/改名；未接产品菜单。各通道独立模式/键可投影，单通道 mixed 和显式 default 保源但投影拒绝；rich tangents/weights/nonconstant extrapolation/f64/gradient/七调整继续拒绝。expected kind 仍须同一真实 catalog/类型/Toolkit publication，没有资格就 unsupported，不据 Data/JSON shape/后缀启用 Curve。

当前共 23 条内存定义，Rust 0 verified / actual execution unknown；原 #388/#392 捕获失败及 r02 证据保留。真实 provider/Toolkit/history/CAS/保存、正式 source closure/load、EXE 三尺寸与实际 144DPI 仍 open，goal active，不以共享缺件整族停工。

静态证据：rustfmt、scoped diff check、reverse patch check 均 0；未改 checker 的有界诊断在 63.54 秒 exit0，guard errors/warnings/suppressed 均 0。首次 55 秒超时、空 stdout/stderr 和已回收进程原件完整保留；诊断栈落在原 Rust 模块可达性扫描，不改 guard 或抑制规则。所有回归只定义、未执行；无新编译/桌面/服务/捕获。产品 passed/accepted=false。证据目录 E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-lossless-consumption-01/，r03 使用 boundary-r03- 前缀，旧文件不覆盖。


### RT-S25 r04：从不可变 DataAsset 准备键编辑候选（2026-10-25T11:13:51.465286+00:00）

本域已在原 codec 增加 StandaloneCurveCodec::prepare_edit(&DataAsset, expected_kind, StandaloneCurveDelta) -> Result<Option<DataAsset>, StandaloneCurveError>。它复用原 decode → StandaloneCurveAdapter::apply → encode_into，不新增 DTO/store/history/clock、decoder、资产或 UUID，不安装文档、不 dirty、不保存、不 I/O/callback。完整 source/kind/cache、全部 expected key 和最终可投影 candidate 校验后才返回一个替换值；任何后项失败均不返回部分候选，原 DataAsset 在所有路径保持不变。no-op 仅在原 adapter 完整预检后返回 None，不能以 delta.is_noop 绕过 stale/default/mixed 拒绝。

原 apply 返回的 inverse 在本次临时准备中用于确认真正 no-op；此 API 返回完整 DataAsset 候选。原 Document 须保留真实 before/after 源值以获得精确文本撤销，再沿同一 document/view/activation/source revision、一次 permit/retention/history/CAS 提交；typed 字段相等、内存候选或 JSON codec 都不产生语义资格/写入许可。SaveCtx 继续管 dirty/锁/I/O/失败/durable cancel。changed 候选保持原 URI/format，原 codec 重开校验并重建 cache，JSON 排版可以变化。

四路径仅 standalone_curve/codec.rs、tests.rs、README.md、tests/prepared_edit.rs；补丁 SHA256 0ab156ac3488c1c9bce3baf480333cc8f53b157b04af7a148a9d25b94c8746cc（13598B），3 个已有 raw 前像精确还原，1 个新增此前不存在；12 个 readonly 引用匹配。新增 8 项纯内存定义，当前13+6+4+8=31，不是项目 producer 或已执行测试。原 #388 七 pin 当前仅 adapter/error 2/7 同版；r03 四 pin 仅 mod/boundaries 2/4 同版，codec/tests/README 的后继明示，不倒签旧13/19/23定义。

该候选入口把已存在的 typed key delta 与原 DataAsset 编码接通，尚未接真实 Editor caller。真实语义 publication、Toolkit/provider、原 Document/history/CAS/保存和正常 EXE/144DPI 仍未验证；这不是新增播放器、静态窗口或资源注册。新增8定义仅静态源；Rust0 verified/actual unknown，goal active，产品 passed/accepted=false。

本次格式、scoped diff、reverse patch check 已过。guard 第一带周期栈诊断进程 exit0xC0000005，第二标准入口110秒到界，原件均已回收并保留；当前同 hash/patch 的第三标准入口以300秒上限记录实际进程CPU，结果未到，不计 guard 通过。未改 checker/规则/断言或用旧 r03 结果代当前。最终真实结论单列后继验证回执，必要失败不覆盖。

同一真实08 file/page于11:05Z API读回 rev107/vern0/2002 objects，尚无 Curve named shape/设计板；本轮0设计保存。首次 JSON reader 错用 pages-index 的诊断已保留，按实际 pagesIndex 精确读回，未把首次误解析0当空页。没有重复 Install/host/服务恢复/桌面操作，设计共享 Curve/source closure 与产品验收继续 open。


RT-S25 r04 guard 终态接续（2026-10-25T11:15:01.437562+00:00）：同一 checker SHA e2ea2593…、同一补丁和4源，第三标准入口真实 exit0/89.89秒（native CPU48.5625秒），errors/warnings/suppressed=0/0/0，stderr空；自有进程已回收。首次原生异常原因仍未证明、第二110秒超时原件保留，未以更改源/规则/断言换通过。最终静态证据 prepare-r04-final-static.json；31定义仍0已验证Rust，实际执行未知，正式源闭包/真实caller/EXE/DPI继续open，passed/accepted=false。


## TextureSourceAdapter 01 — 原始纹理事实与显式上传预检（2026-10-25）

本轮实际源码仅 `zircon_editor/src/ui/host/assets/mod.rs` 与新 `texture/{mod,source,tests}.rs` 四路径，patch `8990b0fed9c8c04a5be0f16c67fdd04312f64431cdbfcc2cd19d297bf5e6873c`。原 assets root 前像 `a468702f…` 可逐字节反向重建；原 G01、Curve、DataTable、Sound、StaticMesh 导出与并行公共改动保留。未改 Runtime、公共 host、registry、catalog、主题或原 ZUI。

`TextureSourceAdapter::read(&TextureAsset)` 借用真实已加载资产，保留可选作者描述符、URI、完整 payload/metadata/count，单列 Runtime 派生的 `effective_descriptor`、规范化是否改变以及原始描述符的原生 metadata 诊断。描述符缺失为 `None`，与 `Some([])` 无诊断区分；非法作者值继续可检查，不在读取中修复。活动 CPU payload 字节、保留的已初始化字节数、源文件大小及 GPU 显存不混用；容器旁的非活动 rgba 不丢失。

`upload_preflight(support)` 仅将调用方明确提供的 `TextureUploadSupport` 交给现有 Runtime `upload_readiness`，保留其结果；`Ready` 是 CPU 上传方案，不是源资产有效性、真实设备、上传完成或帧可见。宿主仍须从同次捕获的真实项目/catalog/UUID/provider 取得资产，核原始诊断、设备与当前 publication，缺失不得构造空纹理或启用预览。

8 个纯内存回归定义覆盖原始引用/完整描述符、缺省与无诊断、非法值及规范化、normal map 色彩诊断、容器与非活动 rgba、显式支持的 Ready/Unsupported、array/cube/volume 维度、零 extent/NaN 保留。rustfmt、scoped diff、反向 patch check 均 0；标准 change-guard 单次 217.297s 退出 0，errors/warnings/suppressed=0/0/0，子进程已回收。测试定义不计执行通过。

独立验证身份为 `ue-assets-texture-source-20261025-01 / validate-asset-texture-source-01`，session 仍原资源08；identity SHA `a0ed8629967c4b786fc98d7dc36f6b677ba94bd9ba66ce1ca3e4deeee5e0e7a9`。当前 selected `53172128…`、12 pins、公有 transport、client parse 与 server validation_recipe 均已核；正式范围固定 Editor 单包/defaults-off/features[]/lib/显式 profile=test/filter=`texture_source_`/jobs1/threads1/物理 E。至本段写入时尚未 POST，后续执行结果以本轮 E 报告原生回执为准；不重投旧 Curve/Audio/Scroll/tree/G01 身份。

交付根 `E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/texture-source-adapter-01`；8 项 Rust、真实 Toolkit/provider、正式导出及正常 EXE/144DPI/逐图比较仍 pending，passed/accepted=false。


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

交付切片为真实导航矢量设计；目录 disclosure 的真实 filled caret、scroll/reveal/focus及宿主命令依旧分别验收，不能用圆形前后导航替代树展开按钮。


### RT-S22-ROWOPS01：DataTable 完整源行复制与批量删除（2026-10-25）

原 `DataTableCellCodec` 新增 `prepare_row_duplicate` 和 `prepare_rows_remove` 产品源码候选。前者复制真实原行的全部 JSON 字节，只改精确 key literal 后追加；后者按精确源键一次预检整批删除，保留存留行顺序/原始 tokens 与数组外层空白。两项均先校验全源/schema/cache，再重读核对完整 columns/keys/field IDs/typed cells（含 u64 边界和 f32 符号零）；缺失/重复选择拒绝，空删除选择仅在全预检后返回 None。不 I/O、不启用命令、不写第二 store/history。UE 依据为 DataTableEditorUtils RemoveRow:457 / DuplicateRow:562 与 DataTableEditor DuplicateSelectedRow:1012 / DeleteSelectedRow:1049。

本批8产品路径＝3原文件插入+5新文件；可精确重建3份旧字节前像，9参考pin未变。增量 patch SHA `f64d78eef0b34efdc4b890186b6d75bce69f5c89d1d476dc7adb03813b7d50c4`，rustfmt/diff/reverse均0，guard exit0/76.719秒/0 errors、0 warnings、0 suppressed。新增9项必要回归定义，当前 data_table_ 共42项；这不是已执行数。

独立新源业务批次 request `ue-assets-data-table-row-ops-20261025-01` / operation `validate-data-table-row-ops-01` / identity SHA `421c6eb896fb0c378161272f2fcba444491f33fb7156babda26f4d4288cd8e11`，当前53172128 capsule 12pins及2canonical匹配，public capability与纯 client/server recipe核对通过。Editor单包/defaults-off/features[]/lib/profile test/filter data_table_/jobs1/threads1/物理E原入口实际POST一次，client exit0/222.016秒，queue18404，input `e05f6fe41fb6fdab03d546463d7803ba77eb824b13bc029d014cfb94e4e814a7`，actualTargetDir null。同身份一次QueryOnly exit2/6.390秒：外层 phase=running/status=pending，内层 status=waiting/reason=management_git_busy，input一致、resubmitted=false。不能把阶段字段当Cargo已启动/未启动或Rust通过，verified Rust0、实际执行未知；不重投，不改原#384/#392/rename queue17885输入或未知锁。

[本批交接](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-ops-01/handoff.json)保留源码/测试/所有回执。真实 schema/toolkit/依赖准入、原 Document/history/CAS/SaveCtx 提交、菜单/快捷键/选择/焦点/撤销接线仍open；本轮无Penpot/ZUI/公共源/运行时写入、无服务或桌面操作，自有3客户端已wait，显式释放0B，产品passed/accepted=false。


### RT-S25-KEYS01：Curve 整批关键帧插入/删除源候选（2026-10-25）

原 `StandaloneCurveCodec` 新增 `prepare_keys_insert` / `prepare_keys_remove`，均接收完整 `(typed channel, authored key)` 数组，一次返回可选完整 DataAsset 候选。插入按真实 input 排序；删除比完整 expected key（含各 f32 位身份），缺失/重复/stale或末项无效均拒绝整批。全源和最终 surface 都须准入，空批次也先预检；空 channel 保留原身份，不补假键。沿原 codec 编码重开，有变化会重排 JSON 排版；typed 增删互逆不等于字节撤销，原宿主须保留完整 before/after，经一次原 Document/permit/history/CAS 提交。`CurveModel`、`StandaloneCurveDelta`、adapter、源 schema/版本及 equality 完全未改，未新增 DTO/store/clock/backend/UUID producer。

7路径＝3精确旧前像+4新增，12参考pin保持；patch SHA `9cf6742716d51e3a09852eae499df8de95379765a830363e25b6b8978b6c7fbb`。新增11项回归定义，当前 standalone_curve_ 共42项；fmt/diff/reverse均0，guard exit0/78.469秒/0 errors、0 warnings、0 suppressed。仅静态通过，verified Rust0、实际执行未知。

本批新源业务 request `ue-assets-curve-key-structure-20261025-01` / operation `validate-curve-key-structure-01` / identity SHA `78cb08a7c37b3be8cb69d9b167891e71522b81df06ee1be96cc7b6cee2c84e35`，当前53172128 capsule12pins/2canonical/public capability与纯recipe核对通过。Editor/defaults-off/features[]/lib/profile test/standalone_curve_/jobs1/threads1/物理E 原入口实际POST一次，client exit0/171.328秒，queue18663，input `17be5c55e3b42e228ee348dbb256721fbb0f2a1d6ba48e16c3c3e1d3843db61f`，actualTargetDir null。同身份一次 QueryOnly exit2/9.937秒，phase waiting / status pending / result waiting / management_git_busy / resubmitted=false，原 input一致。不据此推断 Cargo已/未启动，不重投或修改旧#388/#384/数据表请求，不释放未知锁。

[源候选与验证交接](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-key-structure-01/handoff.json)保留精确源码、回归定义和收据。真实 semantic registry/provider/Toolkit、共享结构命令、菜单/输入、Document/history/SaveCtx、正常EXE/144DPI与双端逐图仍open。本轮无Penpot/ZUI/共享/Runtime/桌面或服务操作；3自有客户端已wait，无临时脚本/文件夹具，显式释放0B；全族active，passed/accepted=false。


### 接续：静态网格真实源可编辑表格与现 Curve 构建观测

[RT-S24-SOURCE-TABLE02](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-design-02/handoff.json) 已在原08文件/原6业务main下实交：rev116/2069objects，原静态网格3板编辑两条真实GLB primitive声明，Shared native refs/共享样式及flow保留，保存关闭重开data相同、12原PNG逐图核对。数值列连读已在r04改为右对齐；原Shared Search900/640缺口、copy override/reset及完整菜单/真实preview/provider/官方export/EXE/DPI继续pending，未把source study或18/19精确override诊断当全族通过。原source/ZUI未写。

Curve原queue18663经13:37:32Z认证API关联#405，identity键完全匹配，building=true/result=null，原Rust执行结果未知；无新QueryOnly/POST/编译/桌面或服务动作。4浏览器关闭await/4Node wait，临时0B，实际释放0B；原任务active，passed/accepted=false。


### RT-S22-CSV01：原 DataTable 的 CSV 文本交换候选（2026-10-25）

原 `DataTableCsvCodec::encode` / `prepare_reimport` 已落产品源码，只处理已关联显式 row schema 的 JSON DataAsset。支持完整列、精确行键、中文/引号/换行、列重排与整表候选；不 I/O、不注册 CSV 资产、不推断 schema，不另建 store/history。无变化返回 None 保留源字节；整批最终校验前不发布候选。新增13回归定义（当前 data_table_ 共55），9源码路径的fmt/diff/guard均0；Rust验证0，实际执行未知。

新源原入口POST一次：`ue-assets-data-table-csv-20261025-01` / `validate-asset-data-table-csv-01`，queue19307→management#409；client exit0/130.125秒。一次同身份Query exit2/4.750秒，running/pending/resultnull、resubmitted=false。随后认证GET核同queue、5身份字段和INPUT_JSON SHA均匹配，#409 building=true/resultnull；不是编译/测试通过。完整接入/差异/失败在 [本批交接](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-csv-01/handoff.json)。旧#384/Audio/Curve/RowOps请求与源证据不改；本轮没有Penpot/原ZUI/服务/桌面变更，正式导出、真实宿主/正常EXE/144DPI仍open。



### RT-S22-COLUMNS02：列宽调整／中文长文本原生状态板（2026-10-25）

08原file/page实际保存至rev128/vern0/2300 objects，仍为6业务main；在原900 linked-copy board原生Ctrl+D生成3个新板（231对象），再用原生Inspector调整到1280×800、900×620、640×520。原2068非根对象与6主件定义精确不变；根仅增加新板，旧子序列顺序保留。原DataTable component dc042d85-e62f-4025-9680-10ee1619c9d7 / main a7972f31-e63f-4359-8e6d-3e871ebb2d8d不变，未新建/复制main或detach。

这是原DT_Items四行ZUI布局fixture的显式状态示例，未创建真实DataTable资产/schema/UUID/producer。只在新增实例把表头及四行Name列85→180，列宽统一为17/180/76/77/55；新增中文行名和完整长Value，原生Text Auto Width使Value保持单行，由原77宽clip cell裁切。保留原Fill、12px Roboto、Shared body typography ref；中文可见是浏览器fallback，不能据此证明Engine/CJK字体ready。原默认三板不改。

最终r06六张full/crop原PNG已逐张打开；native save-close-reopen128 data SHA 06f62581abb137833ea093a07bc11ec01414482c38eef1ad79dfd8a87d9b8e59一致，fresh GET也一致。新增201本域+21 Shared Search sourceRef全部222/222有效；453个非空style RefId及其453个RefFile逐对有效（非453个唯一style）。Shared真实rev5/4主件/34colors/3typographies、原直接库连接保留；get-profile原响应SHA前后一致，未Install/Open插件或写registry/Shared。源ZUI/catalog/tokens/Starship等8pins与CSV9源码pins仍匹配。

保留F2定位超时、X轴名称不匹配、r03把3个图片pair误比6个文件导致success=false、900单行改动的过早GET失败及随后rev126真实持久化、r05 fresh-rev拒绝、向量字符串触发的本地text walker异常和列存储反序诊断。r06等待实际发布后重开；原始失败未覆盖。12原PNG逐图记录、原控件/事件/import映射及接入缺口见 [handoff](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-columns-design-02/handoff.json)、[individual-review](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-columns-design-02/individual-review.json)、[source-checks-r02](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-columns-design-02/source-checks-r02.json)。原Not implemented (preview)反馈、真实表schema/registry/provider/Toolkit、原Document/history/CAS/SaveCtx、菜单/编辑/保存撤销/滚动焦点、正式family插件导出加载、正常Engine EXE/144DPI/双端<=1逻辑像素仍open。此次无产品ZUI/Rust/公共源写入，无Cargo/桌面/服务/DPI动作；产品passed/accepted=false，goal继续active。

CSV原queue19307/#409最新有界认证API已为building=false/FAILURE；原console terminal为management_operation_interrupted、HEAD, full index or complete source membership changed during capture、statusunknown（保留此前3次resource_capacity等待）。这不是Rust测试失败，也没有native EOF或资源释放证明；原9源码/13定义不变、验证Rust0、实际执行未知。[终态复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-csv-01/build409-terminal-review-01.json) SHA 67ad89b31ac7256f01c64b2f66dfe1cd7083973eaeb83c20c38264fee3fbaae4；原control receipt540B SHA eb28e1de273fb54d81c6120fc6d42a5b4b9adf67be106e549a7e51543da75ad1。已沿原identity253edb50…/input0f1687c0…向Jenkins原owner排入消息01a17393-17f2-77c3-bf40-5beed1bddaa8，queued不等于已读；无新QueryOnly/POST/identity/attempt/freeze/锁或State改写。

本族继续源候选和本域设计工作，缺共享件不使整族停工；当前交付只提升原生设计与静态结构事实，正式资源/输入/产品视觉门保持pending。


### RT-S22-MOVE01：原数据表整行移动源候选（2026-10-25）

已在原 DataTableCellCodec 新增 prepare_row_move：以同次真实源的 row_key/target_row_key 定位，目标为目标行原始源位置；下移落在其后，上移落在其前。复用完整 schema/source/cache 校验及原 raw spans/verify，保持整行字面量和各位置分隔字节，返回单一完整候选或经校验的自移 no-op。源不变，没有 I/O、dirty、history 或第二排序 store。四条本域路径、9 新定义（表模块现64），fmt/diff/reverse/guard 静态通过；Rust 验证运行0，实际执行未知。

原正常请求 ue-assets-data-table-row-move-20261025-01 / validate-data-table-row-move-01 真实 POST 一次，queue19857→#413，input 52143869301ba2f837413c334b761c39f74060a8aaf4a5ee370b9658ca599d21；一次同键 Query 返回 running/pending/resubmitted=false。15:30:10Z API 身份和 input 全匹配，building=true/result=null，不代 Cargo/native/资源或测试通过。原 CSV #409、Tree #384、Audio 身份及失败保留。详见 [源码与准入交接](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-move-01/handoff.json)。真实表语义/schema/provider/Toolkit、原每 view Document/history/CAS/SaveCtx、菜单拖放/选择排序/焦点、正式导出加载及 EXE/144DPI 仍 open；passed/accepted=false，goal active。


### RT-S22-MENU01：可编辑行菜单／原生三尺寸重开（2026-10-25）

08原file b4a1e3ce-9e6a-81fe-8008-d69d60ef2172/page ...2173实际rev252/vern0/2759对象、6业务main；新增459对象，旧2299非根对象、6主件定义及旧根子序列精确不变。3个新增DataTable copy+3 Shared SearchInput native linked copy，222新引用=201本域+21Shared。Shared实读rev5/4main/34colors/3typographies且本阶段数据不变，无新main/detach/并行主题。

按实际UE SDataTableListViewRow菜单实现11项/3分组的纵向flow设计，行内caption Fill、5个56宽右对齐快捷键；plus/首尾双上/双下均为原生vector，共9 path，无文本箭头。288×271.4的原生absolute popup按实际Name cell中心夹在板内，native small nudge的Y舍入-0.2，zIndex=1；普通行280×20.4/columnGap10保持flow，48段文本消费原Shared body。此为原4行ZUI layout fixture，不是就绪资产/schema/Toolkit或真实菜单执行。

save-close-reopen及6次捕获后data SHA b415e85d193aa9397d6f19d2c92ccaf696f84ccdef48e83f8d70e77c7d663dbf一致，rev252；两独立context均await关闭。六张原生100% full/crop逐张打开，crop精确1280×800/900×620/640×520，菜单末项完整，未缩字。Chrome dpr1/100%不作Windows150%DPI/正常EXE证据。25/25源pin相同，原ZUI/IDs/imports/bindings/events/Rust及公共主题均未改。

本切片完成可编辑设计、保存关闭重开及三尺寸逐图检查；保留真实生产Table/schema/Toolkit/provider准入。此前RowMove #413由running发展为API FAILURE/源捕获中断，不是Rust失败或native EOF证明；9定义verified0/actual未知、原身份保留，无新提交。资源族整体仍未完成，不因共享未齐停工。

资源未就绪，正式plugin测试/import-export/load、真实popup输入/disabled/IME/Escape/当前generation焦点返回、Document/history/CAS/SaveCtx、正常EXE/原生LayoutReviewCase/144DPI及双端≤1逻辑像素仍open，passed/accepted=false。无桌面/服务/源码副本或新Rust构建，释放0B，必要失败与交付原件保留。

[交付](<E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-menu-design-01/handoff.json>)，[逐图记录](<E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-menu-design-01/individual-review.json>)，[嵌套source核对r03](<E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-menu-design-01/source-checks-r03.json>)。


## RT-DT-CELL01-R02 进展（2026-10-25）

完成当前源的10 owned +28 read-only pins定稿，并以完整模块路径和--exact限定原26项source/cell回归；当前64项静态定义不等于26项执行。原session/request下显式新operation validate-data-table-cell-source-r02 正常POST一次，queue20950/#419，input8a8059bf…；旧#392 identity/marker未改。API #419 FAILURE、控制/State v2 unknown/management_operation_interrupted，源捕获身份变化，实际Rust/native/资源释放未证，不重投。正确2021 edition格式与diff静态通过，误用2024首次失败保留。原38输入当前均匹配，无产品源码/服务/桌面/公共库修改。范围/原始收据/剩余门见本域 data-table-cell-validation-r02。目标active；passed/accepted=false，EXE/DPI及整个资源族未完成。


## RT-SW-RANGE02 当前切片（2026-10-25）

SoundWave原source adapter已实作按可见源帧Range读取，复用原逐声道惰性包络并保留原frame坐标；明确空range/越界和非零起点语义，未新建store/clock/backend或改根mod。5路径候选、7新定义、旧10保留，当前17定义未运行；格式/diff/reverse/guard0/0/0、11参考匹配，45秒guard超时保留。完整提案/源pins见本域 sound-wave-visible-range-02。原#388/#392只消费回传，无轮询/重投/释放；新SoundWave身份仍null。宿主真实source/view/缩放输入、正式export/EXE/DPI及音频输出仍open，目标active，passed/accepted=false。
