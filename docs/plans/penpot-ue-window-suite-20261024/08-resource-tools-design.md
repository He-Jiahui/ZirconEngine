# 08 资源工具：源布局、交互和接入设计

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

## 历史接续：J01有限入口已部署，Audio执行仍待准入

原owner的19:50:05Z [J01部署回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-j01-intake-01/receipt.json)已接续：capsule 0b594015… 的Sound单包有限recipe已贯通，早期build_scope_policy拒绝保留为历史。原Audio request/operation/identity未改；本域核原4源码+18参考及14份引用回执全部匹配，没有重跑QueryOnly。

J01的335条JUnit/9项PowerShell属于管理入口验证，Audio仍0提交/0 Cargo/0测试。原owner唯一QueryOnly为ordinary_entry_unproven / Current session authorization is pending，无server receipt；writer/target/cache lock/reservation/capture均未取得。所报E盘12.16GiB低于v11 reserve+单job预算39GiB，不能据此开始捕获或Cargo。原owner沿原身份接自动收件及E根真实准入；本域不请求用户grant、不冻结文档、不改盘或清共享缓存。全族passed/accepted=false。

Owner：`01a16d53-60db-7d02-b03e-0042f277783d`。2026-10-24。范围和差异主表见 [08-resource-tools.md](08-resource-tools.md)，统筹入口见 [Parent 交接](08-resource-tools-parent-handoff.md)。

这是接续真实Penpot authoring的设计合同，**完整设计和产品验收尚未完成**。最新SoundWave源内容见[第16节](#16-soundwave真实源内容与原生组件)，实际rev76/vern0、1337对象、551flow、18板、4本域main、26copy（5 foreign）；19次保存与独立关闭重开已核对，仅设计内容通过内部检查。公共styles、official source codec/export、真实provider/播放及正常EXE仍open。以下rev57记录保留原时刻事实：本域当时实际rev57/vern0、1262对象、524流式对象、15设计板、3本域业务main、24实例（5 foreign）；原Import/Picker四text的真实能力文案及原生字形重测/新browser逐板SVG核对见[第14节](#14-parent回执接续与importpicker真实能力修正)。Shared rev1四主件的受控链接、Button文字override/reset见[第13节](#13-四个已发布公共主件的受控消费)，原Browser/source映射保持。公共styles、canonical sourceHash/slot/export闭包、原ZUI导出及Engine EXE仍缺。Parent统一公共实现/服务/桌面；本轮未改产品ZUI/Rust或共享源。以下各带时间的rev7…55与Audio记录保留各自历史，不覆盖最新事实。passed=0、accepted=0。

最新Parent followup-04及已安装19002插件Open见[第17节](#17-已安装插件open与typed资产合同)。此次未执行资源source导入/导出；32份源码核查和7种typed文档提案未发布新schema/type/command，原ZUI/Rust保持。

AI第五次交接的原只读provider提案保留于[第18节](#18-ai黑板消费的资产只读provider提案)。Parent新分配的G01最小代码候选见[第23节](#23-rt-airef01-g01原manager只读adapter候选)：只新增原manager的一个查询合同，不新增索引或AI语义；Rust动态验证和真实host接入待验。

SoundWave原始保存响应的99对象/6个原生PathData逐字段比较和可靠PCM基线见[第24节](#24-soundwave保存响应的逐copy完整性)；此为离线证据补全，没有新API/桌面/EXE验收。

## 1. 固定参考分支和尺寸来源

当前参考树包含 Content Browser 新旧两套实现。实际 [UnrealEdGlobals.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/UnrealEdGlobals.cpp) 第 135–189 行注册 `ContentBrowser.EnableNewStyle`，默认 `true`，命令行显式 `=0` 可关闭；[ContentBrowserStyle.h](../../../dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/ContentBrowserStyle.h) 第 11–25 行缓存读取该值。因此本域以**源码默认的新版分支**形成首版设计。没有启动参考 UE，不能声称已测得其实际 CVar 或窗口像素。若 Parent 的参考现场显式关闭此值，须整体调整同一布局分支，不能混合旧工具栏与新 tile 状态。

| 源依据 | 源值/行为 | 设计落实与约束 |
| --- | --- | --- |
| [SContentBrowser.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SContentBrowser.cpp) 第 3670–3867 行 | 新版主体是 VerticalBox → 树/资产 splitter；工具栏位于资产区顶部；导航容器随来源布局组合 | Sources 为独立可收起树区域，资产区为纵向流：操作栏、query/filter、结果、状态/导航；不继承当前 Browser 的 112 高标题卡片作为 UE 目标 |
| 同文件第 3672–3675、3715、3740、3780 行 | 工具栏注释总高 36；独立按钮高 24；来源 section padding 2；顶部 separator thickness 2；资产 splitter coefficient 0.75 | 这些是 Slate 源值，不是已测产品坐标。split coefficient 不直接当作整个窗口 75%；最终边界要结合父 padding、tab chrome、宿主约束和真实字体测量 |
| 同文件第 1047–1072 行 | 搜索延迟 change notification、历史、向上展开建议；NavigationBar 有 crumb click、子目录菜单、recent path、文本编辑和补全 | query 的 IME/输入不触发全局快捷键；路径显示用真实 roots 和目录，编辑提交后解析为已存在/可写目的地；不存在的路径不能仅更新标签 |
| 同文件第 788–801、877–880、905–920、1005 行 | 纵向 filter 最小宽 95；filter/search 在横向 filter 布局会一起 wrap；splitter handle 2；初始 FilterBarLayout 为 Vertical，随后 LoadSettings 可恢复用户配置 | 默认保留纵向 filter 的源状态和“无 filters 时收起”行为；用户可切到横向组合，此时一起 wrap。窄窗自动切换尚是待 Parent 确认的差异。不能把 95 用作 Sources 最小宽 |
| [ContentBrowserStyle.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/ContentBrowserStyle.cpp) 第 27–59、83–95、348–385 行 | 局部 style 继承 AppStyle；新版来源 header/body 为 Panel/Recessed；toolbar icon 16、背景 margin(4,2,4,2)、block margin(2,0)，不 wrap 且不显示 wrap button | 由 Parent 共享主题承接语义，实例引用同一 style。源 toolbar 在空间不足时 clip；若 Zircon 为 640 窄窗增加 overflow 菜单，必须作为设计差异审批/记录，不能说是 UE 原样行为 |
| 同文件第 106–155 行及 [AssetViewWidgets.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/AssetViewWidgets.cpp) 第 2904–2909 行 | List 名称字体为 Regular 12，Tile 名称基准 Regular 9；Column 行区分 active/inactive/focus；TileTableRow 背景为空，tile 自己绘制 selection/hover | 字体为 UE FontInfo 源值，不直接认定为 Zircon 逻辑 px。tile 状态由 tile composition 绘制，不能同时套满行选择背景，避免双重高亮。中文 fallback 与行框由实际字体检查 |
| [SAssetView.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SAssetView.cpp) 第 6175–6213、6281–6312 行 | thumbnail 档位 Tiny…Huge；插值范围 64–80、80–96、96–112、112–128、128–136、136–160；新版 tile 基础高=基础宽，再加类型/文本区域 | 尺寸选择必须影响真实 tile extent、行列数和滚动计算；当前 Drawer 固定 28 的虚拟行不能冒充这些档位。每种视图独立保留档位，不能把源值全部塞成同一固定卡片 |

公共 token 的名字、值、Penpot style ID 和版号由 Parent 确认。原 Browser 导入 `res://ui/theme/editor_workbench_strict.zui`，Drawer 导入 `res://ui/editor/unreal/starship.zui`；迁移前须让 Parent 对账这两条依赖，不能在本域新建第三份配色或凭观感逐控件覆盖。

## 2. Penpot 的具体可编辑对象

以下 `RT-Dxx` 是文档中的**设计工作项**，不是 Penpot UUID。实际身份逐项登记；没有实例ID时不填造UUID。目标是消费Parent的window/menu/tab/field/tree/list/table/viewport实例和共享样式，当前这些公共身份尚未交付；下列业务组件内的叶子仍需替换，不能当公共复用完成。普通布局使用可编辑vertical/horizontal/wrap flow；overlay仅用于Drawer、popup、viewport内容和必要层叠。

| 工作项 | 默认板应可编辑的内容 | 必须另存的状态板/交互 |
| --- | --- | --- |
| RT-D01 Browser | 真实项目标题/roots、Sources/目录树、可操作面包屑、New/Import/Save、query/type filters、Grid/List/Column 结果、计数和 view settings | 空项目、过滤无结果、单选/多选、失焦选择、首尾滚动、中文长名、目录/资产/空白右键；有后端缺口的状态标为待实现 |
| RT-D02 Drawer | 复用同一 Browser 内容组合；底部锚定、宿主高度控制；Dock in Layout 与关闭入口 | 初开 search focus、Ctrl+Space 再关、子菜单 Esc、焦点返回、点击外部、拖出关闭、停靠复制设置；不能另维护一套资产数组 |
| RT-D03 Asset picker | 参数化类型/来源过滤、搜索、同一结果视图、允许时 None、确认/取消 | 受限类型、空结果、选中后返回调用者、取消不修改属性、多次开关、不误开资产 editor |
| RT-D04 Path/Open/Save | 真实目录树、路径/crumb、文件/资产名、当前目标、合法性错误、确认/取消 | 中文/长路径、只读目的地、名称冲突、覆盖/拒绝/返回、外部不可写路径；Open 和 Save 使用不同确认条件 |
| RT-D05 Import/Reimport | 来源、当前 project destination、实际 importer 支持的选项、任务结果；reimport 显示当前 UUID/source | Import/Import All/Cancel/Reset；无效来源、重复提交、冲突、批量失败/取消、项目切换、成功后定位；不同 importer 不共享虚假的 FBX 参数 |
| RT-D06 Mesh | Toolkit 菜单/工具栏、Viewport、Details，LOD 与材质槽引用；待支持页签独立列出 | 真 mesh 旋转/缩放/框选/LOD、材质槽跳正确 owner、重导入、dirty/save/undo；socket/collision 不放有效外观的无操作按钮 |
| RT-D07 Texture | Viewport + Properties 栈；RGBA、mip、缩放/fit/100%、背景显示；属性滚动 | 真 source 与不同 channel/mip 图像、alpha、加载失败、metadata 与源一致、reimport、save/undo；Platforms 依后端，Oodle 当前排除 |
| RT-D08 Sound | 资产属性；有真实选中资产 PCM/输出后才启用 waveform/transport | 播放/暂停/停止、真实时长与 playhead、输出失败、关闭释放、项目切换；破坏性变换独立待定，不绘制样例波形 |
| RT-D09 DataTable | 上方 DataTable/DataTableDetails 栈，下方 RowEditor；真实 schema 列与行键、搜索、属性 | 真空表、长列名/横滚、选择行与编辑、类型错误、save/reopen、undo/redo；旧 DT_Items/Potion 不是验收数据 |
| RT-D10 Curve | Float/Vector 单曲线编辑页；LinearColor 的 Curve/ColorCurveEditor 水平分区 | key 选择/新增/移动/删除、切通道、tangent、数值提交/取消、frame/zoom、dirty/save/reopen/undo；动画曲线只转其 owner |

保存链：对实际共享组件实例作最小样式改动 → save ACK → 关闭工作区 → 新认证上下文重开 → 对账实例、共享 style、flow 和依赖 → 官方导出 → 对账原源 ID/事件 → 正常 EXE 加载和前后图。样式实验选择**已分配给本域的实例可覆盖叶子**，或由 Parent 改公共 style；本域不自行修改整库。此链没有执行，不能以本文或离线文件代替。

### 历史记录：revn7…13结构与来源文本

本域通过隔离的真实 Chromium 无头上下文、官方 Penpot 工具栏/图层/属性输入操作，建立了下列可编辑对象；未安装新plugin或启动额外服务。2026-10-24T10:56:59.787Z，实际`get-file`读回`revn=7, vern=0`，含页面根共7个对象；components/colors/typographies/ZUI metadata均0。公共组件文件同时读回仍为revn0、单根对象、上述计数均0。

| 实际 board UUID | 当前内容/父级 | 实际保存值及未完成项 |
| --- | --- | --- |
| `6474610e-e630-805b-8008-d6a355a690ba` | `RT-D01 Content Browser · layout draft · shared instances pending`，页面根下 | 1280×800，Flex column；是结构草稿，不是UE视觉完成稿 |
| `175c4e58-29af-80a5-8008-d6a41e51ee4c` | Document tabs，Browser根下 | 1280×32，Flex row；现有原生Content Browser文本，待Parent DocumentHost实际实例 |
| `30578a6c-27a0-80d5-8008-d6a477785097` | Sources and Asset View split，Browser根下 | 1280×744，Flex row；scroll与响应式flow伸缩约束尚未补齐 |
| `5a083d37-7a08-807b-8008-d6a5f1e30ba0` | Sources，split主体下 | 256×744，Flex column；树/集合及共享行尚未加入，前版row方向已在revn7修正 |
| `2e3ed126-21ac-80da-8008-d6a77373c850` | Asset View，split主体下 | 1024×744，Flex column；工具栏、搜索/筛选与真实结果内容仍缺 |
| `2e3ed126-21ac-80da-8008-d6a778de1700` | Status，Browser根下 | 1280×24，Flex row；实际计数与view settings组件及业务仍缺 |

上述32/744/256/24为当前结构草稿的试排值，不是已测UE最终像素或三个尺寸的验收参数。根板的`x=182,y=200`只是Penpot画布放置位置，不能用于产品窗口布局或截图偏移补偿。revn7时六区为空，早期文本等待失败未落盘；后续revn8…13加入下表原生文本，仍无真实业务绑定、交互、组件实例或共享样式。没有在本域建立重复公共main component或平行主题。

首版根板在2026-10-24T10:38:01Z前完成HTTP保存→关闭原context→fresh已授权context重开；最终六区结构在10:57:40.054Z前另一次新浏览器/新context中全部按UUID/名称读回。七份实际保存request/response（revn0→1→2→3→4→5→6→7）已用项目原有`verifySaveResponse`校验file/session/changes和后端持久变更；无请求重放。后续区域操作保留了图层容器/工具激活/异步选择/折叠状态、Column控件及文本标签等待失败，不能把这些失败改记为成功；Sources方向的后续修复另由revn7证明。诊断浏览器图不是官方board PNG或正常Engine EXE图，也不是实际150%DPI证明。

历史只读核验2026-10-24T11:21:02.573Z为revn13/vern0，共13对象（页面根+六frame+六text）；当时公共文件仍revn0/单根。两文件原生components/colors/typographies、实例对象及ZUI metadata计数均0。13对真实请求/响应已按项目现有shape-aware Transit合同解码、复用`verifySaveResponse`重核file/session/changes/lagged记录，实际ACK覆盖1…13。frontend请求revn可落后于最新ACK，依原lagged协议核验，不重放请求。

| 实际原生text UUID | 父区域 | 实际保存文字 |
| --- | --- | --- |
| `318ddd39-5da4-8095-8008-d6abf25b9036` | Sources | World of Claudecraft |
| `7fd0d19a-fb6b-8097-8008-d6acd10b3af0` | Sources | res://m8/models/props |
| `7fd0d19a-fb6b-8097-8008-d6acd5a44db2` | Asset View | res://m8/models/props |
| `7fd0d19a-fb6b-8097-8008-d6acda113693` | Asset View | crate_wooden.glb |
| `7fd0d19a-fb6b-8097-8008-d6acde35a49f` | Asset View | 318,100 B |
| `7fd0d19a-fb6b-8097-8008-d6ace262dbed` | Document tabs | Content Browser |

原context和browser关闭后，另一新browser/context于11:19:49.898Z前重开同file/page，逐项核对六text UUID/parent/content；正文来源见第9节，未伪造Runtime UUID。文字仍为Penpot默认Source Sans Pro 14/黑字白底，未引用UE共享样式，不作为视觉完成稿；路径文字还没有交互式Breadcrumb，文件名还没有Tile/缩略图/选择/命令。

已目视复核[当前重开诊断图](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/source-labels-r05-fresh-reopen.png)：六段文字可见且分属标题/Sources/Asset View，主体仍大片空白；缺少UE颜色、字体行框、树、工具栏、搜索、缩略图和全部业务状态。该图含1920×1200浏览器chrome、画布约94%缩放，不可量最终1DIP、作为正常Engine窗口、150%DPI或双端比较证据，review结论为草稿核查、pending。

实际证据目录：[penpot-headless-layout-01](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01)。本历史阶段核心回执为[penpot-draft-current-verified.json](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/penpot-draft-current-verified.json)、[source-labels-r05.json](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/source-labels-r05.json)；原revn7回执、全部失败和原始诊断图保留。r03漏解shape标签造成“文本未保存”的错误断言，实际revn8文字与保存ACK由修正解码后证实；该失败不改记成功。完整保存/重开验收还要求同一真实组件实例、shared styles、source metadata与官方导出，当前未满足。

### 当前记录：revn22本域业务组件与五个内容板

本节固定Browser/Drawer最近内容变动时的rev22证据；全文件后续rev26及新增两板见下节。后续已逐属性确认原546个非根对象完全不变，复用这里五张原图，不声称重新捕获了五个未变状态。

实际file `b4a1e3ce-9e6a-81fe-8008-d69d60ef2172` / page `b4a1e3ce-9e6a-81fe-8008-d69d60ef2173`未另建。11:58:17.647Z的[独立读回](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-content-final-r22-readback.json)确认547对象、205 flow、一个native component、四个copy heads及412个链接后代（连同heads共416个链接对象），全部parent/child图有效。公共文件同次读回仍revn0、单根、零components/colors/typographies；本域也没有原生color/typography共享样式。现有颜色只是已有token的草稿投影，不新增主题。

本域业务组件为`Zircon / Asset Tools / BrowserContent`，component UUID **`6dbdeee0-4fe3-80ec-8008-d6afd543b430`**，main为原split **`30578a6c-27a0-80d5-8008-d6a477785097`**，草稿业务合同版本2，未发布library。原区域在真实UI以Ctrl+K建立native component，四个副本保留component-file/component-id/shape-ref和局部尺寸、中文内容override。它组合资产来源/结果内容，不充当公共TreeRow、Button或SearchInput主件。`zircon-ue-assets` metadata仅登记业务合同/依赖；不是官方`dev.zircon.zui`源codec，不能据此声称可回写产品。

| 工作项 / 实际板尺寸 | board UUID | BrowserContent实例head |
| --- | --- | --- |
| RT-D01 Browser 1280×800 | `6474610e-e630-805b-8008-d6a355a690ba` | main `30578a6c-27a0-80d5-8008-d6a477785097` |
| RT-D01 Browser 900×620 | `4488c08b-8e05-4626-98fb-b97edbd5663d` | `1bd7c136-e28f-4b38-9bb9-5ba4e71b7bfc` |
| RT-D01 Browser 640×520 | `409e039b-26b9-4b4c-95a3-4380147320b1` | `97e4e0d6-50d8-4fad-8f61-e7381999f81c` |
| RT-D02 Drawer内容面1280×520 | `d303e393-e852-4531-9554-3d850d8a4c5a` | `29678a17-4461-4a29-90a3-8b4494439c67` |
| RT-D01 中文/真实长路径640×520 | `062f92fc-81a7-4d8e-bedd-c437c3762444` | `d746513d-7228-4a8f-8fef-9c1f32f8c833` |

Drawer只是1280×520内容面，不是完整Editor 1280×800或已工作的overlay/Dock。主组合为Sources与AssetView横向flow；SourceHeader/Search/Tree/Location和Breadcrumb/FilterSearch/ListHeader/Rows/SelectedDetails各自纵向flow，标题32、状态24、来源列256（640板160）、资产行40、目录行28均为草稿值，不是UE测量通过值。目录树的14行、props的9个GLB与实际磁盘核对；列表选择crate，文件大小318,100 B。没有生成缺失的.zmeta或假Runtime UUID。

原六frame及六text UUID保留，来源文字现已移动到实际业务容器：project→`5e2674f4-2e25-4cf9-94ee-eb2d62063474`，Sources路径→`8eff32d3-e59e-4e46-a4d8-f8fe51b214c0`，breadcrumb→`306560d9-c1b0-4fb2-8edf-255201f4924a`，crate/大小→`d00ce552-1642-45d6-b908-271a547e803c`；Content Browser仍在原tab。旧表中的父级是rev13历史位置，当前精确parent/text以独立读回为准。新副本曾由原生同步重映射六个新增文字ID，记录保留；没有更换原六text身份。

保存历史接续rev14…22。原生UI创建业务组件后，经官方RPC authoring和正常前端autosave保存完整树；rev15/17/19/21的原始请求/响应以既有`verifySaveResponse`验证。JSON请求与后端schema的比较副本仅归一化shape缺省flip-x/flip-y=null、touched集合和typed rect/point/matrix；原始证据不改写、不重放，也不是ZUI导出。实际front-end正常保存与两个新browser/context重开确认当前component/main/copy/sourceRef、全部新增ID、目录及文字。最终[review-r05](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-native-review-r05.json)保存到rev22，两个自有headless PID9372/36184均native exit0，关闭均await；11:57:15Z同任务Chrome为0。

字体复用现有注册Roboto400 `custom-495ff285-7c71-49bc-bc4e-df20017e2d66`和NotoSansSC400 `custom-2ddd793a-5f89-40b0-aec1-fbe879e37a57`。隔离工作区显式`wasm=true`，WASM ready且两font上传值均1；默认SVG渲染未初始化WASM的早期probe失败保留，不误判为font损坏。中文图可见“资源浏览器/搜索路径/筛选/搜索资源/名称/类型/大小”；实际绝对源路径可见，未把它当极端溢出、ellipsis/tooltip或IME证明。

| 已逐图打开的最终浏览器画面 | 所见与仍欠缺 |
| --- | --- |
| [Browser1280×800](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-native-review-r05-1280x800-6474610e.png) | 14目录/9行完整，crate选中、名称和大小对应实际资产。MDL只是文字，占位缩略图仍待provider；菜单/命令、共享行/输入实例未接 |
| [Browser900×620](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-native-review-r05-900x620-4488c08b.png) | 流式伸缩后仍可见完整14目录/9行；没有因此证明键盘、焦点或命中 |
| [Browser640×520](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-native-review-r05-640x520-409e039b.png) | Sources收至160；选中props及尾部目录被裁，末尾结果不可达。真实ScrollBox/首尾/scroll-into-view待接，不可accepted |
| [Drawer1280×520](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-native-review-r05-1280x520-d303e393.png) | 与Browser复用同组件，Dock in Layout只是设计文字；来源选中目录及末尾结果被裁。外部点击/Ctrl+Space/Esc/停靠/焦点返回未验 |
| [中文/实际长路径640×520](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-native-review-r05-640x520-062f92fc.png) | 两种字体可见，无缺字；真实绝对路径在详情区可见。继承640滚动缺口；极端长名/路径、tooltip、输入法和文本Undo仍缺 |

全部PNG为1920×1200隔离浏览器工作区，画布缩放分别94%/131%/178%，deviceScaleFactor=1；没有官方Shape.export或正常EXE对应图，不能从浏览器缩放推出实际DPI，也不能测双端1逻辑像素。[逐图记录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-design-individual-review-r22.json)引用现有LayoutReviewCase/正常捕获协议，各项native capture/semantic resources/side-by-side/diff仍pending，误差为null，不生成passed案例。[22对原始保存重核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/penpot-saved-ack-recheck-r22.json)已复用原插件`verifySaveResponse`确认ACK连续1…22；不替代Jenkins托管插件/Rust测试。早期修复保留：native布局children顺序按实际Penpot flex算法处理，未给个别对象加截图补偿；r04曾被新增中文board的画布摆放覆盖Drawer，r05仅整体平移中文board+400x并验证内部相对几何不变，失败图未删。

当前Search/Filters/List/crumb/Dock等均为可编辑设计叶子，未有产品效果；缺New/Import/Save等完整工具栏、真实Grid/thumbnail、通用选择、空态/弹层/焦点/首尾滚动与业务菜单。其它RT-D03…10及RT-E窗口仍待设计和真实后端。官方导出需公共pipeline的family source-closure/native source metadata及共享身份；不能把此业务组合直接覆盖四个原始资产来伪造完成。

### 最新接续：revn26选择器与模型导入内容

12:13:15.014Z的[独立读回](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/resource-tools-final-r26-readback.json)确认684对象、261flow、5个BrowserContent副本及515个链接后代（含heads共520）；7板和所有parent/child/sourceRef有效。公共文件同次仍revn0/单根/零组件样式。rev23创建以下两个原生可编辑内容板，rev24前端正常保存；rev25修正来源字段合同，rev26正常保存并另一新browser/context重开，所有137个新ID、业务metadata和路径placeholder保持。最后两个自有PID38220/36904均native exit0、关闭await、无page error。

| 业务内容草稿 | 真实board / 组件实例身份 | 实际画面与边界 |
| --- | --- | --- |
| RT-D03 Model asset picker，900×620 | board `34f72199-8ba0-4e33-affb-cc45bc69b2be`；BrowserContent copy `95ebe941-228c-4316-b1ab-688f048fc335` | [实际PNG](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/resource-picker-import-review-r02-900x620-34f72199.png)：九个真实模型/选中crate、候选文件名、Cancel和禁用Select；目录尾行仍裁切。Runtime UUID未知，不填随机UUID或启用假确认 |
| RT-D05 Import Model默认态，640×520 | board `e02dc17d-fde0-4bba-8bd0-ba9d6f1e7e80`；内容frame `e8f39f41-fb28-47fe-8010-c66dba1aa7ed` | [实际PNG](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/resource-picker-import-review-r02-640x520-e02dc17d.png)：Project、空来源文本字段、固定models目的地、复制语义、Cancel和禁用Import。没有凭空添加FBX/Import All/任意目的地/成功状态 |

Picker的`allowedKind=Model`只是业务合同，不证明当前已有类型限制或回传逻辑。确认必须从真实catalog取得UUID/locator/type并重核同一project activation/revision；取消不写调用者属性，双击/Enter回传而不是打开asset editor。复用同一内容组件不表示产品已把picker绑定到全局Browser选择，也不新增资产数组/selection store；临时candidate与调用者上下文需沿Parent既有选择/Modal owner承接。None、其它类型、多选、默认focus/Tab/Esc及焦点返回仍待实现。

Import来源提示已从“Choose file”改为`Paste an .obj or .glb source path`。原始Browser `import_path_field` / `AssetBrowserImportPathField`、`AssetsView/MeshImportPathEdited`→`workbench.asset.mesh_import.path.set`写`mesh_import_path`；原ImportModel事件/route保持。真实[retained side effects](../../../zircon_editor/src/ui/retained/app/dispatch_effects/side_effects.rs)调用[model_import_settlement](../../../zircon_editor/src/ui/retained/app/assets/workspace/model_import_settlement.rs)的`import_model_into_project`，读取`build_chrome().mesh_import_path`并走[canonical_model_source_path](../../../zircon_editor/src/core/editing/paths.rs)→`submit_model_import_source`→原EditorAssetImportFlow。未证明原生文件选择器，不以Browse外观冒充已接通。

同一宿主已有pending时，若处于settlement则ImportModel重开该尝试的决策，不隐式Retry/新建作业；其它pending返回busy。提交还需真实active scene identity和当前项目lease，move/delete/import并发互斥；这些不是仅凭有project标题就已满足。取消是advisory，durable commit后不能承诺回滚。此导入内容板还不是现存Modal或保存事务，原Quick Import输入/命令保留；接入必须明确request/confirm/job/settlement阶段、错误和焦点恢复，不分裂成无效果表单。

新增两个内容板仍以普通可编辑叶子标注Parent Field/Button/Modal依赖，没有建立重复公共main/style。其`zircon-ue-assets` metadata读回包含真实原control/event/route业务映射，**不是官方源codec或产品绑定**。完整共享实例/样式、官方导出/原ZUI回写、实际输入/作业与保存、Reimport、专用资源窗口和正常EXE验收仍open。

[七图逐项记录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/resource-tools-design-individual-review-r26.json)包含每图缺陷和null误差；Picker第二轮PNG与已打开首轮SHA相同，导入新placeholder图另行打开复核。[保存重核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/penpot-saved-ack-recheck-r26.json)确认ACK连续1…26、原546个非根对象全部属性不变。两个共享执行脚本的基线漂移另见[只读对账](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-source-drift-reconciliation-r26.json)：strict `verifySaveResponse`函数与原HEAD逐字相同，新函数仅处理已知空滚动捕获标记；保留先前exit1基线检查，不把它抹成全通过。托管插件/Rust和双端视觉仍pending。

### 最新接续：revn40纹理源图与原生PNG

2026-10-24T12:51:47.854Z的[原始读回](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/resource-tools-final-r40-readback.json)为revn40/vern0：1164对象、495flow、13板、3本域业务component、18copy heads，704个链接后代（不含head）。[最终fresh-context核验](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/texture-source-final-identities-r40.json)另起两个隔离browser，前/后/重开均rev40，未产生新保存；原生退出0，无page error，未用桌面。共享库仍rev0/单根/零component/color/typography、无已交付跨文件引用；本域三业务组件不等于公共组件复用。

新增业务组件 **TextureSourceContent draft v2**，component `02db8ec6-f26e-482e-9df2-1c0a02753b02`、main `7f5ecf4a-fb69-4710-b1ad-0384d19e2cbc`。只组合Image/Label/flow及现有样式值，不创建公共Button/Tree/Property主件，不登记平行theme/list；正式共享实例/样式和codec metadata仍待Parent。BrowserContent v2（`6dbdeee0-4fe3-80ec-8008-d6afd543b430`、5副本）与AssetTile v1（`30625d51-64a3-469f-84c5-79e156375f8c`、11副本）保留，新增TextureSourceContent两副本：

| 逻辑板尺寸 | board UUID | content head UUID | 原生PNG |
| --- | --- | --- | --- |
| 1280×800 | `9e6aa0d5-6a16-487d-b543-687a301b6433` | `7f5ecf4a-fb69-4710-b1ad-0384d19e2cbc`（main） | [源图/Details](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/texture-source-native-export-r02-1280x800-9e6aa0d5.png) |
| 900×620 | `da29931e-d802-4557-8d03-56a1a067b5b1` | `8bf3798a-0612-47d9-9db8-41cd454df92b` | [源图/Details](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/texture-source-native-export-r02-900x620-da29931e.png) |
| 640×520 | `88f34a85-47dd-4300-a909-c08392fbedc6` | `04853481-3648-4809-9110-c29beb945715` | [源图/Details](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/texture-source-native-export-r02-640x520-88f34a85.png) |

源图复用已上传的base.png/media `15e0edbb-d0a4-4abc-bc41-8ecfba752043`，没有重传或拷贝源树。[实际PNG审计](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/texture-source-image-audit-r01.json)确认1024×1024、12914B、RGBA 8-bit、alpha范围255…255；源SHA `028db4d1cd8b3471e6ba21686515abd4e36ae582f8b314b71ad2ab1fb5453b12`。无相邻.zmeta，Engine asset UUID仍null；Penpot media ID不代替asset/document/PreviewSession身份。GPU format、resource size、mip count、compression、LOD均显示未知；Save/Reimport/Find、通道/Mip/Fit文案是禁用设计状态，未连接真实引擎能力，不算功能完成。

UE依据仍为[TextureEditorToolkit.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/TextureEditor/Private/TextureEditorToolkit.cpp)的`Standalone_TextureEditor_Layout_v7`（369…395）：primary横向分割，viewport隐藏tab well，右侧Properties/Oodle/Platforms子stack、foreground Properties。viewport下层vertical stack的0.9不能误当primary横向系数；Details的0.33才在primary层。本稿Details宽320/280/240、源图Fit方形628/448/348是待实窗比较的设计约束，未宣称UE逐像素量测。当前[texture plugin](../../../zircon_plugins/texture/editor/src/plugin.rs)仅注册`texture.authoring`/`view.texture.authoring.open`与`plugins://texture/editor/authoring.zui`；未找到本域typed资产Toolkit/该URI实体ZUI，后者需原owner核对解析/打包，不能推断已观察运行加载错误。

本轮真实保存暴露并修正两项设计问题：640的Source format改为native auto-height，值框22、行高30、Quick info高152，完整显示`PNG · RGBA 8-bit`；正常float布局有小于0.001逻辑px的数值差，原严格相等检查失败被保留。修改main后native同步曾把900/640的Fit覆盖回61%；沿Penpot真实`libraries.cljc#text-change-value`为27项本域实例文本补`text-content-text`，必要的本地rich-text再补`text-content-attribute`，保留content-group，恢复44%/34%。[修复和重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/texture-source-review-r03.json)及rev40最终核验确认原文、覆盖组、图片缓存/字体上传仍保持；不修改公共Penpot实现，也不把这些同步标记冒充ZUI绑定/事件。

六张实际1×PNG（网格三尺寸、源图三尺寸）经原Penpot sidebar `Add export` / `Export 1 element`产生；[原生导出回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/texture-source-native-export-r02.json)保存尺寸/字节/哈希、settings ACK35…40及每次下载失败。六次native浏览器download均`canceled`；只读监听原`URL.createObjectURL`产生的实际PNG Blob字节，未改返回值/revoke、未另画/裁图/换codec。原导出源码在anchor click后立即revoke是待owner核实的原因假设，未确认或修复；首次失败回执不覆盖。PNG可用不等于浏览器下载成功、官方ZUI源导出或EXE捕获。

[40对原保存ACK严格重核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/penpot-saved-ack-recheck-r40-r02.json)沿当前`verifySaveResponse`核file/session/changes/lagged，未发送/重放请求。原R28比较脚本误取rev26两次，首次R40又错用旧输入，均是检查错误并保留；[修正记录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/penpot-save-comparison-repair-r40.json)固定三个独立Transit文件的revn/object count/SHA。真实26→28的683个旧非根对象全属性不变；28→40的923个旧非根对象仅25项touched及3项exports改变，几何和文字未变。不能引用早期自比较当证明。rev40原Transit SHA `bc1565fbc940de717dd1113c0066d6ce5fc6c85761686ff49c31a394fc6e9b7f`。

[逐图复核rev40](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/resource-tools-design-individual-review-r40.json)覆盖13板；原7图沿旧几何/文字不变的真实比较复用，网格3图改用原生PNG，源图3图逐张查看。640网格的选中目录仍被裁，实际滚动/自动定位未接；工具栏图标、共享menu/focus/drag/dock及真实命令继续open。全部normal EXE、side-by-side、difference、boundary/text-line误差仍null；源图只证明真实PNG内容，**不证明GPU预览、运行交互、实际150%DPI或≤1DIP双端通过，passed=0、accepted=0**。插件/Jenkins Rust检查与Penpot样式→官方源导出→产品加载→截图变化链仍待交付。

## 3. 原始资产和稳定身份映射

### 最新接续：revn28真实纹理网格与AssetTile

此节为rev28历史记录；最终rev40及原生PNG/比较修正见上一接续。

12:30:17.731Z的[认证读回](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/resource-tools-final-r28-readback.json)确认同一file/page现为revn28/vern0、924对象、381flow、10板、2本域native components、16copy heads/548链接后代。BrowserContent及原七板保持；[连续ACK重核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/penpot-saved-ack-recheck-r28.json)覆盖1…28，rev26的683个非根对象全部属性无变化，支持复用此前七图，不伪称重新捕获。

Parent已允许本域用既有Image/Icon/Label/Button/flow组合业务Tile。新增 **AssetTile component `30625d51-64a3-469f-84c5-79e156375f8c` / main `a87b3a71-49e3-432b-9b59-be99063789b2` / draft contract v1**；三板合计1main+11个原生链接副本，未创建泛型List/Tree/Button公共main、另一selection store或平行主题。136×172是本轮草稿extent，128×128源图、name/type和selected状态位于同一原生column；网格为row/wrap、gap8、padding8，不能把这一档位冒充已实现UE全部thumbnail档位、运行时虚拟化或键盘导航。正式提升/版本由Parent公共catalog统筹。

| 真实纹理网格板 | board UUID / 实际图 | 逐图复核与未完成边界 |
| --- | --- | --- |
| 1280×800 | `9ae8f0ff-c158-415e-b49e-13527ce18a47`；[PNG](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/texture-grid-review-r02-1280x800-9ae8f0ff.png) | 四真实源图一行；base.png蓝色选中；knight选中目录和全部19目录可见。仍缺真实工具栏/crumb控件、shared style/实例和运行时输入 |
| 900×620 | `e6247927-146e-4d11-9dea-9453bd7a0796`；[PNG](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/texture-grid-review-r02-900x620-e6247927.png) | 四列，名称/类型/选中和真实URI可读；目录尾行裁切，首尾滚动尚未实现或验证 |
| 640×520 | `a0d4c1e2-10f7-4177-8b46-949074b67cc6`；[PNG](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/texture-grid-review-r02-640x520-a0d4c1e2.png) | 原生wrap为3+1，第四项base.png及状态完整；树高304而12th选中行在可视区之外，knight选中行未出现。必须沿共享树scroll owner自动定位，不能靠藏目录或缩字掩盖 |

四张源图均来自真实`examples/woc/assets/m8/textures/skins/knight`，1024×1024、project GUID `28be8619-648f-45e4-9513-e02fb909ea9c`；当前同路径.zmeta缺席，Runtime asset UUID保持null，未生成假身份。完整源SHA和上传请求/响应见[media回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/texture-grid-media-r01.json)：alt_a.png→`eefa0967-63a7-4254-aa4c-96b1b3a08612`、alt_b.png→`630400b4-9303-4643-b229-90060da38fc6`、alt_c.png→`39a66c39-85ca-4427-a937-24102a4f561d`、base.png→`15e0edbb-d0a4-4abc-bc41-8ecfba752043`。这些是Penpot file-media UUID，不能作为Engine asset UUID。原生rect `fills[].fill-image`沿真实Penpot `image-uploaded`/`upload-file-media-object`合同，12个图像fill实际引用这四个媒体对象，不是示意色块或伪造缩略图。

[新browser/context重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/texture-grid-review-r02.json)核对全部240新ID、Tile shape-ref/媒体覆盖、现有BrowserContent及Picker/Import路径合同；三次图和另一fresh context的四图WASM full/thumbnail cache均为1，两字体上传为1。PID33260/2768均native exit0、close await、无page error。首次[失败检查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/texture-grid-review-r01.json)在实际ACK28后因检查代码把compiled registry导出名末尾`$$`误写为`$`而报available=false；仅修正检查端符号名，保留原失败及严格cache/fresh断言，未重放保存或降低准入。

真实UE [SPathView.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SPathView.cpp)的1539、1923、2170行明确RequestScrollIntoView；[SourcesViewWidgets.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SourcesViewWidgets.cpp)第137–161行使用真实folder image和SourceTree字体、root额外top padding3，并非证明本草稿28行高精确。当前`>`/`v`文字箭头、缺folder icon、静态path/search/filter叶子、未接焦点/菜单/拖放及shared style均继续open；不以读到cache就绪代替实际Engine SourceImage provider/thumbnail cache或GPU资源ready。正式缩略图应沿[现有provider](../../../zircon_editor/src/ui/host/assets/preview_refresh/generate_preview_artifact.rs)以真实asset UUID/source hash生产，不新建并行缓存。

本轮全部图含Penpot chrome、浏览器1920×1200@DSF1及94%/131%/178%工作区缩放；不是官方board export、正常EXE截图或实际150%DPI。每图误差仍null，没有双端并排/差异，passed=0、accepted=0。四个原始ZUI及其IDs/imports/bindings/events未改。共享库12:30Z仍revn0/零组件、colors、typographies；`zircon-ue-assets`仍只是业务依赖合同，不冒充官方source codec。

资产依据：[asset_browser.zui](../../../zircon_editor/assets/ui/editor/asset_browser.zui)、[asset_window.zui](../../../zircon_editor/assets/ui/editor/windows/asset_window.zui)、[content_drawer.zui](../../../zircon_editor/assets/ui/editor/unreal/components/content_drawer.zui)。现有 `SearchEdited`、`ImportModel` 在 Browser 和 Drawer 重名，定位必须用 `(sourcePath, sourceNodeId, instancePath, controlId)`；不能全局搜索替换。

| 原始 owner / node | 稳定 control / event / route | 接续设计要求 |
| --- | --- | --- |
| Browser `asset_browser_root` / `main_panel` | `AssetBrowserRoot` / `AssetBrowserMainPanel` | 保留根与子区域身份；用流式 split 组合替换普通内容的 position。横向空间由真实树/结果/可收起详情协商 |
| Browser `toolbar_search_field` | `SearchEdited`；`Workbench/AssetBrowserSearchEdited` → `workbench.asset_browser.search.edit` | 绑定真实 `search_query`；输入刷新与指针命中引用同代次。保留 IME 文本态，不能每帧重建输入实例 |
| Browser `toolbar_kind_filter_dropdown` | `AssetBrowserKindFilterDropdown`；`Workbench/AssetBrowserKindFilterChanged` → `workbench.asset_browser.kind_filter.change` | 复用 registry 可用类型；未启用插件不显示为可创建/可打开能力。filter 与 query 组合，不只改按钮文案 |
| Browser `toolbar_view_mode_list_button` / `toolbar_view_mode_thumb_button` | `AssetBrowserViewModeListButton` / `AssetBrowserViewModeThumbButton`；`Workbench/AssetBrowserViewModeList` / `…Thumbnail` → `workbench.asset_browser.view_mode.list` / `.thumbnail` | 保留 `dispatch_kind=asset:browser`、`action_id=workbench.asset.view_mode.set` 和真实值；缩略图必须有不同布局。Column 当前没有对应枚举/route，需明确新增并走公共选择合同 |
| Browser `toolbar_locate_button` | `LocateSelectedAsset`；`Workbench/AssetBrowserLocateSelected` → `workbench.asset_browser.locate_selected` | 复用已有 navigate-to-asset：定位真实 folder、清文本过滤，必要时清 type filter；不制造重复资产 |
| Browser `import_path_field` / `import_button` | `AssetBrowserImportPathField`；`AssetsView/MeshImportPathEdited` → `workbench.asset.mesh_import.path.set`；`ImportModel`；`Workbench/AssetBrowserImportModel` → `workbench.asset_browser.import_model` | 现有 Quick Import 来源值不能丢弃。引入 modal 时给同一 command 明确 request/confirm/job 阶段，避免按钮与原字段分离成无效果表单 |
| Browser utility 四 tab | `AssetBrowserPreviewTabButton` / `…References…` / `…Metadata…` / `…Plugins…`；`Workbench/AssetBrowserUtilityPreview/References/Metadata/Plugins` → `workbench.asset_browser.utility.preview/references/metadata/plugins` | 保留事件/值与真实选中资产投影。它们是 Zircon 附加能力，不是 UE 默认 Browser 的强制右侧卡片；可收进工具区，并逐项记录布局差异 |
| Browser result/tree/detail targets | `AssetBrowserSourcesPanel`、`AssetBrowserSourcesScrollBody`、`AssetBrowserContentPanel`、`AssetBrowserAssetTableScroll`、`WorkbenchAssetBrowserTableHeader`、`AssetBrowserDetailsPanel`、`AssetBrowserUtilityPanel` | 现有源中的样例行文本不是数据。实际 painting/selection 继续使用 `asset_browser_pane_data` 的 immutable generation；迁移时须消除外层与结果内层竞争同一 wheel 的嵌套纵滚 |
| Activity shell `root` 和七 slot | `AssetWindowRoot`；`AssetWindowTreeActivity/CollectionsActivity/DetailsActivity/PreviewActivity/ImportLogActivity/DependencyActivity/BrowserContent` | 保留 `ActivityDrawerWindow` 引用与原 slot。当前 entry 为 ExclusiveMainPage；不能仅改 ZUI 标签宣称新增 UE 四实例/Dock 能力 |
| Drawer `drawer/header/controls/content` | `UnrealContentDrawer` / `ContentDrawerHeader` / `ContentDrawerControls` / `ContentDrawerAssetBrowser` | 保留 overlay 身份；只有根的 bottom anchor 是允许的定位。高度、z-order、点击外部与 focus-return 归宿主 |
| Drawer `search/view_mode/import/close` | `SearchEdited`：`AssetSurface/SearchEdited` → `workbench.asset.set_search_query`；`SetViewMode`：`AssetSurface/SetViewMode` → `workbench.asset.set_view_mode`；`ImportModel`：`AssetSurface/ImportModel` → `workbench.asset.import_model`；`ContentDrawerClose`：`Unreal/DismissContentDrawer` → `workbench.content_drawer.dismiss` | 保留原事件/命令语义。更换共享组件实例要验证 imported port 和输入捕获，而不只是旧 route 字符串仍存在 |
| Drawer `asset_prototype` | `UnrealContentAssetPrototype`；`Unreal/SelectContentAsset` → `workbench.content_drawer.select_asset`；虚拟 prefix `UnrealContentAssetRow`，namespace `v2` | 按 UUID 和 locator 同时核验；保留 prototype source ID、稳定实例 identity 与回收焦点逻辑。Grid 虚拟化要按行/列及 tile extent 计算，不能继续复用 item_extent=28 冒充 |

[DataTable 原资产](../../../zircon_editor/assets/ui/editor/modules/data/extension_data_table_workspace.zui) 是 component `WorkbenchExtensionDataTableWorkspace`，不是根 view。保留 `data_table_workspace` 与现有 `WorkbenchExtensionDataTable*` 源身份；`…SaveButton` / `…ValidateButton`、`…RowField`、`…TypeDropdown`、`…VersionField` 的原 event ID 和 edit/commit 路由需在真实 schema/transaction 接入时逐项迁移核对。现有 `items/schema_file/localization`、`row.potion/sword/armor/debug` 是固定样例命令，不能作为真实行键；不得用它们命名真实资产或据此宣称编辑完成。新增动态行的 prototype/instance 和旧调用者兼容方式在实际设计/导出时给出明确映射，不随机重建 ID。

Mesh/Texture/Sound/独立Curve没有核实到现成的asset document ZUI/toolkit。本文不捏造“原始文件”或已登记route；接入应复用已有插件/registry/module owner，必要新增资产先由Parent确认边界，再在已分配Penpot文件创建并官方导出。独立Curve的共享表面由动画owner维护，本域仅提供typed asset/model、业务命令、保存与Undo adapter；不得直接改动画共享路径。

## 4. 选择、激活、菜单、快捷键与焦点

选择语义依据 [SListView.h](../../../dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h) 第 592–730、1251–1282 行和 [STableRow.h](../../../dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/STableRow.h) 第 469–500、602–628 行。Browser 的资产视图通过 [SAssetView.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SAssetView.cpp) 第 3044–3085 行连接选择、双击和 context menu。

| 输入/状态 | 目标结果 | 当前接入约束/验收观察 |
| --- | --- | --- |
| 单击、Ctrl 单击、Shift 单击 | 单选、切换集合、按当前排序扩展范围；focus/lead 与 selection 分离 | [AssetWorkspaceSnapshot](../../../zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_snapshot.rs) 有单个 `selected_asset_uuid` 和单份详情；现有 Find-in-Browser 同步集合可经行 `selected` 呈现多项（第9节），并非完全没有集合。通用 Ctrl/Shift 选择、lead/anchor、集合命令/context payload 仍缺，需沿同一 owner 补齐 |
| 右键未选项 / 已选项 | 未选项先成为选择再开菜单；已选项保留当前集合；右键拖滚不误开菜单 | menu payload 使用同代次 UUID 集合，执行前再核验。当前测试与实际新命令待实现，不借右键文案证明 mutation |
| Home/End/PageUp/PageDown、方向键、Ctrl+A | 导航可见结果，自动 scroll-into-view，范围和全选尊重选择模式；虚拟化不丢 lead | Grid 按实际列数导航；分页使用真实可见行/列。项目/排序/query 变更后按 UUID 重定位，不能跨代次沿用 row index |
| Enter / Ctrl+E / 双击 | Browser 打开所选真实资产或目录；双击只激活被击项；picker 回传选择 | [ContentBrowserCommands.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/ContentBrowserCommands.cpp) 第 18 行及 SAssetView 第 5323、5788 行。临时项/thumbnail editing mode 不激活；类型 registry route 是唯一 opener，没有 toolkit 就保持未完成 |
| Space | 资产预览；编辑器内部 transport/canvas 自己处理 | Browser quick-jump 会吞掉用于查找的字符避免穿透视口 W 等命令（SAssetView 第 2930 行）。输入法组合/文本输入先于 Browser 命令；不能让 Browser Enter/Space 抢走音频/Curve 的作用域 |
| Ctrl+Shift+N、Ctrl+Backspace、F4/Ctrl+L | 在单个可写目录新建、到上级、编辑路径 | UE 新建要求一个允许创建的真实目录；上级要求单个非根目录（SContentBrowser 第 4474–4514 行）。本域缺的 create/history/path-edit 命令分别登记，不绕过 source authority 写磁盘 |
| Ctrl+S、Alt+F、复制路径快捷键 | 保存选中资产；按选择类型筛选；复制真实路径 | UE Ctrl+Shift+C 是 object path、Ctrl+Alt+C 是 package name；Zircon locator/UUID/物理来源与 UE package 不同，菜单名称/结果必须按实际资源模型给出，不伪造 UE package |
| Ctrl+Space / 状态栏 Drawer 按钮 | 开 Drawer 并聚焦 search，再触发关闭；返回原 focus | Windows 参考 [GlobalEditorCommonCommands.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Toolkits/GlobalEditorCommonCommands.cpp) 第 71 行；SContentBrowser 第 2495–2508 行搜索框内也处理同一 chord。复用现有 Drawer owner，不另立热键控制器 |
| 弹层 open → Esc/确认/取消 | 默认焦点在可操作输入；Tab/Shift+Tab 留在 modal；关闭回到仍有效的开启者 | 子菜单 Esc 先关子菜单，再关 picker/modal/Drawer；源 opener 已回收时回到结果 scroll owner，不能恢复到新 UUID 的复用虚拟行。点击别的 pane 已形成新焦点时不强抢回来 |
| Drawer drag leave / Dock in Layout | 拖出后 Drawer 关闭，拖动 payload 仍有效；Dock 复制所有浏览设置，临时 Drawer 仍可用 | SContentBrowser 第 420–427、1347–1375 行。drag payload 绑定 project activation/UUID/locator，不依赖被销毁 tile 的内存。Dock 由 Parent 负责真正宿主迁移，不能替换成关闭 Drawer |

资产/目录/空白 context 分开组装。可用项来自 registry、只读策略、选择与真实 command capability；没有后端的删除、重导入、创建等动作登记缺口，不能显示可用后仅发 toast。Rename/Delete/Undo 的公共菜单和快捷键由 Parent 统一命令路由，本域提供实际 asset payload、admission 与效果。

## 5. 真实能力链与业务窗口

| 操作/窗口 | 可复用的真实入口 | 尚需形成的可验证效果 |
| --- | --- | --- |
| 浏览数据、query、type、单选与定位 | `ui/project/asset_workspace_state/**`、`ui/layouts/views/asset_browser.rs`、Drawer adapter、`host/editor_event_execution/asset_event.rs` | 单一数据权威；paint/hit-test/drag/selection 同代次。活动项目、UUID、locator 和索引修订进入 case；旧帧/旧项目结果拒绝 |
| 拖放移动/重命名 | [project_relocation.rs](../../../zircon_editor/src/ui/host/assets/manager/assets/project_relocation.rs) 已提交 Runtime filesystem/registry/resource transaction，mutex 为 `project_asset_mutation`；retained app 有 request/poll | 从 UI 到 ticket、终态与磁盘/registry 一致性；冲突/只读/失败返回与焦点；文件移动后引用仍正确。已有 transaction 不是已经具备 editor undo 的证据，撤销反向事务/恢复策略仍待核实 |
| 删除 | runtime registry 真实 `asset_delete_preflight`，UI 已有请求/阻挡/轮询链 | 引用阻挡详情、确认/取消、成功后刷新与失败保留；不能把 preflight 通过当删除成功。是否支持可撤销删除由 runtime/history owner 提供合同 |
| 导入 | `core/asset/import_flow` project activation/lease/job 与 model settlement | modal 输入转真实 request，确认只提交一次；显示来源/目标/进度及终态。`import_flow/job.rs` 明确 durable commit 后迟到 cancel 不能撤销，UI 不得报“已取消并回滚” |
| Reimport | 现有 registered source、项目 authority、UUID/lease 可复用 | 必须使用原 UUID 和引用；替换 source 与源选项有明确 transaction。再次 ImportModel 导出另一个 UUID 不算重导入。Mesh 的 source/LOD 选项和 Texture/Sound 来源需由各后端声明，不机械复制 UE 所有开关 |
| Mesh | 真实 Model/Mesh resource、scene renderer；[StaticMeshEditor.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/StaticMeshEditor/Private/StaticMeshEditor.cpp) 的 Layout_v6，viewport coefficient 0.7，右侧 Properties/Socket/扩展栈 | asset-bound document、独立 preview scene/camera、真实 LOD/material slot/数据统计、save/dirty/history。命令真实源在 [StaticMeshEditorActions.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/StaticMeshEditor/Private/StaticMeshEditorActions.cpp)，不是不存在的 StaticMeshEditorCommands.cpp；Nanite/DistanceField 等无 Zircon 后端的项逐项不适用/待定 |
| Texture | SourceImage 真解码；[TextureEditorToolkit.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/TextureEditor/Private/TextureEditorToolkit.cpp) Layout_v7：Viewport 与 Properties/Oodle/Platforms 栈，Properties 前景 | 真实 GPU/source preview、RGBA mask、mip/sampling/fit/100% 可见结果，属性修改/重导入/保存/撤销。命令在 [Private/Models/TextureEditorCommands.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/TextureEditor/Private/Models/TextureEditorCommands.cpp)，视口在 Private/Widgets；Oodle 排除，压缩/平台参数按实际后端 |
| Sound | sound runtime/output controller；UE SoundWave 默认可走 SimpleAssetEditor，Waveform 是独立可选 opener | 实际选中声音解码、PCM、设备/播放头与停止释放；基础属性 editor 不能冒充 waveform。Waveform 源 Layout_v2 的属性/变换区 coefficient 0.2，显示区 0.8；Space切播放/Enter停/Shift+Space回首，由该 document 处理 |
| DataTable | [DataTableEditor.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/DataTableEditor/Private/DataTableEditor.cpp) Layout_v6 的上部 Table/Details tab stack + 下部 RowEditor | typed schema/row-key、合法性、批量原子提交、dirty/history/save/reopen；本地旧 preview handler 不承接假保存。CSV/JSON 导入须确定 schema/curve 类型并在真实路径持久化 |
| CurveFloat/Vector/LinearColor | renderer-neutral `ui/curve` + [CurveAssetEditor.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/CurveAssetEditor/Private/CurveAssetEditor.cpp) 普通 Layout_v2，Color 特例 curve/color 0.8/0.2 | runtime 序列化资产、eval/mutation、key identity/tangent/selection、事务与save；拖 key 从 begin/update/commit/cancel 一次 undo。颜色分区依真数据，不制造样例 gradient |

Mesh/Texture 只在已实现字段上允许写回；显示状态（镜头、缩放、通道、滚动）与资产 mutation 分清 owner。保存成功需 durable write + registry/version 刷新并重开证明；按按钮后状态栏出现文字不足以通过。文档关闭的 dirty/取消/保存后关闭和全局 undo/history 由 Parent 通用宿主合同承接。

## 6. 三种尺寸和真实 DPI 的排布合同

Parent“资源第二次接续”已确定首版默认：`EnableNewStyle=true`，toolbar clip/no-wrap/no-wrap-button；不因640尺寸自动启用overflow、切filter方向或收起详情。下面是同一默认分支的约束，不是已测UE像素或已完成的breakpoint。以 `1280×800`、`900×620`、`640×520` 的**逻辑 client viewport**建可编辑板；加 `1280×800`、scale 1.5 的实际 native 对照。window outer rect、系统边框与 client rect 分开登记。

| 尺寸 | Browser/Drawer | Modal/专用资源窗 |
| --- | --- | --- |
| 1280×800 | Sources + 结果为主split；树与结果各沿现有单一scroll owner。filter保留默认Vertical/已保存设置；toolbar不wrap。详情/References沿真实utility状态，不添加新的自动显示规则 | 沿Toolkit tab tree保留默认前景页；属性纵滚、preview fill。工具栏按源clip/no-wrap，不能用auto height暗中换行 |
| 900×620 | 保持同一默认分区和字号；允许源码已有的手动Sources/filter布局入口及已保存状态。长名/路径需真实elide/tooltip；当前尾部目录裁切保持open | 参数正文滚动、确认/取消可见；DataTable保留横滚与行键。默认改变须另登记差异 |
| 640×520 | 不自动隐藏目录/详情或改变filter方向；沿稳定folder ID/asset UUID请求选中定位、首尾和键盘可达。knight选中目录不可见仍open，新增Vampire较短树不关闭旧case。Drawer服从实际host content height | 保留源Toolkit布局/最小宽冲突；需要切tab/dock时走真实手动入口。自动折叠/overflow仍为未批准候选，不能据候选宣称默认通过 |
| 1280×800 @150% | 同一逻辑布局与文字测量合同；物理 client 期望1920×1200，必须实际 GetDpiForWindow=144 | LayoutReviewCase `dpi=1.5` 是 scale，不能填写144；原生 OS DPI单独为144。禁止图片缩放、伪造窗口尺寸或复用DPI120截图充数 |

已知设计差异：窄窗overflow、filter自动转横向和详情自动收起没有获准为默认，只保留候选；源已有的手动布局切换也须双端同状态比较。文本边界覆盖glyph/line box与框边，不用缩字、隐藏目录、截图偏移或局部pixel-snapping关闭差异。

## 7. 业务数据、case 和逐图复核

复用 [LayoutReviewCase 合同](../../../tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-review-contract.ts)，不新建平行 case schema 或绕过 native 协议。`sourcePath` 使用上表原始资产；standalone 新资产尚未创建时该 case 仍为 pending。`host=editor`、locale/state/scrollPosition/真实数据入现有合同；共享 themeSourcePath、reviewHost、EXE/runtimeAsset/dependency hashes 由 Parent 当前实际产物提供。

| 业务 case 工作项 | 最小真实操作链 | 必须记录/比较 |
| --- | --- | --- |
| RT-I01 Browser 身份 | 活跃项目 → 搜索/类型组合 → 选择 → filter/排序/滚动 → 双击相同UUID | 同代次 paint/hit/drag；过滤无结果与空项目区别；跨域 toolkit route；首尾真实计数与选择可见性 |
| RT-I02 Drawer 焦点 | 真实焦点 → Ctrl+Space → query → popup → Esc → Drawer关闭 → 回原焦点；另跑拖出/外部点击 | popup和Drawer分别截图；回收虚拟行后的focus；拖payload不失效；Dock待Parent宿主 |
| RT-I03 Picker/Save | 从实际属性调用 → 类型过滤/选中/取消/确认；再实际 Save path、同名冲突、重开 | 返回值只一次，取消不改原值；真实目标/UUID；path合法性和中文长名。无实际调用者不算完成 |
| RT-I04 Import/Reimport | 实际项目路径导入 → job终态 → Locate → 打开；原UUID重导入 → 再打开 | 来源/目标实路径、importer/options、UUID、引用、source hash/版本；失败/重试/取消/项目切换；不破坏示例项目原资产 |
| RT-I05 Mesh/Texture/Sound | 同一真实资产打开正确toolkit → preview ready → 窗口特有控制 → save/关闭/重开 | 真mesh姿态/LOD，texture通道/mip前后像素，sound真采样时长/playhead/输出；placeholder必须失败或pending |
| RT-I06 Table/Curve | 真资产/schema/key → 编辑提交 → undo/redo → save → 关闭/重开 | 字段/曲线改变、dirty/version、一次事务、磁盘hash；不能用固定Potion或装饰曲线 |
| RT-I07 样式生效 | 同真实file/component style变更 → save/close/fresh reopen → export → 产品加载 → 前后捕获 | Penpot/source/export/dependencies/EXE哈希和原生入口；变化区域定位到该属性；不得截图后处理模拟 |

真实项目候选保持分离：`examples/woc` 具真实 glb 与 mp3，sound selection 当前只在 client_runtime；`examples/vampire` 有真实源纹理。先让当前 editor_host 证明相应 capability，再选择项目与源路径。不能把多个项目的目录混成一份假 catalog；测试导入的目标和生成资产先由 Jenkins 产物/夹具协议登记，不能直接把已有示例文件重写成“测试样本”。中文/长文本优先用实际文件/显示名；若需受管夹具，数据与图必须标明来源，不能替代真实项目验证。

每个尺寸和对应状态保存 Penpot图、正常EXE图、并排图、差异图与独立逐图记录。记录包括 case与选择器、焦点/scroll、ready资源/字体代次、原生捕获身份、边界最大逻辑像素误差、文字/可见性结果、差异解释和待决项。目标关键边界/文本行框 ≤1逻辑px；只通过结构检查或某张图片不能把全族 accepted。资源未ready或原生捕获无效时不写passed；双端缺图或未逐图复核时不写accepted。

## 8. 现阶段独立工作与移交边界

已完成：默认样式分支核对、实际 Slate 选择/键盘/菜单语义、四份原资产的稳定 identity/event 对照、DataTable preview迁移限制、十项编辑板工作项、七条业务验证链，以及窄窗差异提案。

该早期设计阶段已开始实际Penpot结构草稿并保留保存/重开与失败证据；当时完整设计、共享实例/样式、可逆source metadata、官方导出、产品接入及Jenkins/正常EXE/150%DPI验收均未完成。后续已消费Shared SearchInput，当前又新增域内Mixer只读Rust适配器；具体事实分别见第11、12节，不把早期“公共组件仍未发布/无Rust改动”当成当前状态。原Jenkins失败保持，没有对公共主入口/catalog/tokens/Starship/export/capture协议提交修改。

收到实际资源后先实现RT-D01/D02的共享内容设计和最小可见样式生效链，再沿真实能力实现picker/import及各资产document；不能把完成Browser当作完成整个族。每个Rust/插件变更前重新核对共享checkout差异、模块依赖和现有相关验证，向用户声明精确package/features/target/filter/profile/command，通过Parent受管Jenkins提交；文档结构检查不触发Rust构建。

## 9. 真实字段、项目资产与路径准入

本节是对现有代码与磁盘的只读核查及消费合同，不是新增公共API或已运行的picker/import。普通窗口字段必须来自完整workspace发布；[pointer_projection](../../../zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_snapshot.rs) 为降低指针发布开销，会清空项目元数据、目录与creation menu，不能把它的空project_root当成“未打开项目”。

| 显示/操作 | 已存在字段和权威 | 消费与拒绝条件 |
| --- | --- | --- |
| 项目标题、Sources根 | `project_name/project_root/assets_root/cache_root/default_scene_uri`；[ProjectManifest](../../../zircon_runtime/src/asset/project/manifest/project_manifest.rs) 的project_guid和ordered asset_roots | project_guid是持久身份，不替代当前ProjectActivationIdentity。多根完整列表须由真实project manager投影，单个assets_root字符串不代表已经呈现全部roots |
| 目录树与crumb | [AssetFolderSnapshot](../../../zircon_editor/src/ui/workbench/snapshot/asset/asset_folder_snapshot.rs) 的folder_id/parent_folder_id/display_name/depth/count；selected_folder_id | folder_id做命令身份，显示名只作文本；不靠字符串截断重建package/子资源路径，不把未核实Collections作为真实数据 |
| 结果代次 | catalog_revision、[AssetWorkspaceItemGeneration](../../../zircon_editor/src/ui/workbench/snapshot/asset/asset_workspace_item_generation.rs) 的不可变chunks及UUID/locator索引 | catalog_revision是workspace发布代次，含preview/resource更新，不能当成唯一源目录修订。渲染、hit-test、drag和菜单须同代次；跨代次按UUID/locator重定位 |
| Tile/List内容 | [AssetItemSnapshot](../../../zircon_editor/src/ui/workbench/snapshot/asset/asset_item_snapshot.rs) 的uuid/locator/display_name/file_name/extension/kind/asset_type/preview_artifact_path/diagnostics/resource_state/resource_revision | 类型badge/icon/color来自当前注册类型投影，不能由扩展名推导写权限；资源状态缺席是允许状态，不能补成Ready。preview路径还须文件/解码/provider与代次核验 |
| 选择集合与主项 | [AssetWorkspaceState](../../../zircon_editor/src/ui/workbench/project/asset_workspace_state.rs) 的selected_asset_uuid、synced_asset_uuids；[content_browser_sync](../../../zircon_editor/src/ui/project/asset_workspace_state/content_browser_sync.rs)；行selected与crate内selected_indices | Find-in-Browser能同步多UUID，按BTreeSet首项选primary，跨目录回res://，清search并按需清kind。select_asset/select_folder/navigate会清同步集合。此现有能力必须保留；它尚无通用Ctrl/Shift、lead/anchor及批量mutation合同，不能重建第二份集合store |
| Details与正确打开 | [AssetSelectionSnapshot](../../../zircon_editor/src/ui/workbench/snapshot/asset/asset_selection_snapshot.rs) 的uuid/locator/asset_type/meta_path/toolkit_view_id/toolkit_open_operation/context_commands/subassets/references/used_by | 基础state先产空toolkit/commands，后续真实注册/operation投影才能赋予业务入口；点击时回操作中心重核，不能把快照字符串当执行授权。Mesh子资源与top-level Model UUID分别保留 |
| dirty、保存与关闭 | [project_catalog_record](../../../zircon_editor/src/ui/host/assets/manager/project_sync/record_projection.rs) 当前把PreviewState::Dirty投影为asset.dirty | 这个dirty表示预览待更新，不是通用document mutation/history状态。页签星号、Ctrl+S、dirty关闭确认须接真实文档权威；不得用它模拟资产已修改/已保存 |
| 可写来源 | [AssetSourceAuthority](../../../zircon_editor/src/core/asset/source_authority.rs) 同时解释注册类型write policy与locator scheme | 仅ProjectOnly类型且res来源才可写；lib/package/builtin/mem或解析失败采用只读。picker可按用途读这些来源，Save/Import目标准入另核物理project root，不以按钮enabled替代后端检查 |
| 新目标与现有源 | [source_path_for_uri](../../../zircon_runtime/src/asset/project/manager/source_path_for_uri.rs) 与[ProjectPaths](../../../zircon_runtime/src/asset/project/paths.rs) | 现有URI要求配置roots中唯一匹配；新res目标明确选首个root。拒绝跨根歧义、`..`/绝对片段和物理junction/symlink逸出；发布前重核物理身份，不在UI另写Path::join解析器 |
| 模型导入输入与目标 | [submit_model_source](../../../zircon_editor/src/core/asset/import_flow/model_submit.rs) 仅收source_path；[source_plan](../../../zircon_runtime/src/asset/project/manager/scan_and_import/source_plan.rs) 决定实际destination | 已在配置roots内的源保留原URI；外部obj/glb固定res://models/文件名，存在则DuplicateProjectAssetUri拒绝，不覆盖。当前没有任意目标目录、Import All、FBX选项参数；设计这些控件须保持缺口，不显示已能提交 |
| 导入任务与提交终态 | [submit](../../../zircon_editor/src/core/asset/import_flow/submit.rs) 的indexed URI/UUID/source_digest代次、flight与lease；[Runtime scoped commit](../../../zircon_runtime/src/asset/manager/service_contracts/asset_manager_contract.rs) | 普通导入只接受已索引URI，缺失返回AssetNotIndexed；模型提交返回job ticket，不是完成。Runtime重核activation/catalog/preparation epoch，durable commit后才publish并返回receipt，亦可能触及default material；取消与失败按真实终态显示，不能由进度1/1直接报保存成功 |

### 实际项目与只读资产身份

两份manifest均未显式asset_roots，依据[默认root](../../../zircon_runtime/src/asset/project/manifest/validation.rs)及[RelPath::project_assets](../../../zircon_runtime_interface/src/project/rel_path/value.rs)采用assets。以下只证明文件/元数据存在与所记哈希，不证明当前Editor已经激活、加载插件、播放或生成生产画面；不把两项目合成一个catalog。

| 真实项目/资产 | 已核验身份与源 | 验证前提/限制 |
| --- | --- | --- |
| [WOC manifest](../../../examples/woc/zircon-project.toml) | project_guid `28be8619-648f-45e4-9513-e02fb909ea9c`；World of Claudecraft | rendering/texture/animation/gltf_importer声明editor_host；sound只声明client_runtime。声明不是当前加载/能力证明 |
| [crate_wooden.glb](../../../examples/woc/assets/m8/models/props/crate_wooden.glb) | `res://m8/models/props/crate_wooden.glb`；318100B；SHA-256 `7a5f880c07a60adbc9b50b92557e47a1fa35a7dfa0e3a9fe4fe9f42216d31131`；GLB JSON有1mesh/0skin/0animation | 同路径.zmeta不存在；资产UUID尚未从实际Runtime catalog取得，不用路径hash或随机UUID冒充。仅解读GLB结构，不是GPU preview |
| [ui_click.mp3](../../../examples/woc/assets/m8/audio/sfx/ui_click.mp3) | `res://m8/audio/sfx/ui_click.mp3`；13835B；SHA-256 `98598833b3360cd0ad24c1e409b21434b0b822ecaa0482de9c6342cca13cd242` | 同路径.zmeta不存在，UUID未取得；没有PCM时长/采样和editor输出session证据，不画拟造波形 |
| [Vampire manifest](../../../examples/vampire/zircon-project.toml) | project_guid `837f1b65-bbcf-4b72-af59-1317f512a8fc`；Zircon Vampire Roguelite | 当前这些rendering/texture/gltf等selections只声明client_runtime，不能直接假定editor_host可用；不改示例manifest补证据 |
| [jungle_ground_albedo.png](../../../examples/vampire/assets/textures/jungle_ground_albedo.png) / [.zmeta](../../../examples/vampire/assets/textures/jungle_ground_albedo.png.zmeta) | `res://textures/jungle_ground_albedo.png`；256×256；SHA-256 `9d567dde6505b7a77b3cc42c397076fe97a5ed55d59e5ba333da8fc56662d28d`；UUID `e0cca9ad-973a-4dfd-a306-8116b9930c93` | meta preview_state=ready和对应cache artifact存在均是静态观察，不能直接算当前preview ready。meta source_digest与本表SHA-256算法不同，不混作同一指纹 |
| [hero_capsule.model.toml](../../../examples/vampire/assets/models/hero_capsule.model.toml) / [.zmeta](../../../examples/vampire/assets/models/hero_capsule.model.toml.zmeta) | Model UUID `33ac1631-ed92-46a1-b819-9706331eba91`；Mesh0/Primitive0 UUID `49eb3126-4fcd-4b3b-af56-858fd53dfafe`；源SHA-256 `bcef313eb78a5d9c2ea1384109a120c13e3dd16b56514d2ad90e4568b8369fe3` | 两个artifact文件存在；Browser top-level过滤label的行为与正确Mesh document跳转须分开核验，不能把Model和Mesh身份合并 |

未执行导入、meta重写、缓存生成、播放或项目保存。路径/导入验收还须实际覆盖：已存在内部源、外部obj/glb、.gltf明确拒绝、同名冲突、首root新目标、跨root歧义、中文长路径、只读来源、项目切换/旧ticket拒绝、durable提交后迟到cancel、真实磁盘/registry/UUID结果；这些都仍pending。

## 10. Parent新增收件的业务适配与差异

Parent本次明确：资源08为Audio全族作者业务owner（含Cue/Class/Submix），同时牵头OVW-12/22/24/25/26/35/37/38/41/44的盘点、设计与业务适配；OVW-36映射既有Audio收件，共11项，不计成11个新增完成窗口。Font/FontFace资产文档归资源，Widget只消费字体；StringTable的key/reference事务适配归资源，Localization入口归设置07。Renderer/neural/provider/跨文档事务与Jenkins生命周期由Parent接续；该分工不授予公共runtime/core写权。Actor类/组件的资产跳转沿真实registry/toolkit，缺后端保持open。

原RT-E01…11均继续open；下表的Penpot/EXE/双端图均未取得，不引用rev40的Browser/Texture图代替新窗口。UE入口与适用性沿[原始逐项盘点](08-resource-tools.md#parent-新增收件的窗口组)，本次补业务数据、命令与依赖边界：

| 原收件 | 本域适配/目标设计 | 已证实差异、真实依赖与下一项独立工作 |
| --- | --- | --- |
| OVW-12 Actor Merge | 选中实体/源model与material集合、变换/LOD/collision选项、真实目标asset路径及结果引用；preview与执行分开 | 未核实mesh合并executor/作者命令，导入与渲染不能代替。Parent接场景/资产跨文档事务和renderer；本域先给source selection/revisions、输出/替换预检与失败不改原场景合同，再定合并/保留源对象策略，不给静态Merge按钮算完成 |
| OVW-22 Asset Loading | 资产入口携带UUID/locator→真实trace session/request/stage/时间区间→事件列表/详情→定位资源 | 现有import job不是加载trace；等待Parent诊断provider的真实时间、单位/clock、丢事件/范围及correlation。无trace与零事件区别显示，不推算时间条；本域只牵资产过滤/跳转消费 |
| OVW-24 Reference Viewer | 当前selection snapshot的references/used_by/known_project_asset投影；方向/深度/过滤与定位 | 真实直接引用可用，hard/soft/recursive/incomplete边界仍缺。材质中立surface只读复用，不用MaterialAsset/编译器；本域供asset target/catalog revision与域verdict，Parent provider接完整索引。未知目标保留locator，不按同名文件跳转 |
| OVW-25 Size Map/Audit | 已选资产及依赖统计表/treemap；source bytes、artifact bytes、runtime decoded/GPU、shared/unique物理占用分列 | 未有完整度量provider/窗口；每个数值携source/hash/config/unit/measurement basis/completeness，unknown不作0，shared依赖不重复累加。Parent接provider/renderer；本域先给只读audit行和真实资产入口，不把源Length当显存 |
| OVW-26 Cache | 只读来源/配置/版本、真实物理root、大小/复用状态与原请求回执视图 | Jenkins持有扫描/清理/复用生命周期；本域不新建删除命令、服务或“清理缓存”按钮。数据age/错误/无法归属可见，未知不当垃圾，UE Zen/DDC管理不适用 |
| OVW-35 Struct | 已接纳reflection catalog的类型发现/筛选/返回结果；Struct作者资产另行审查 | [ReflectSchemaCatalogEntry](../../../zircon_runtime_interface/src/reflect/schema_catalog/entry.rs)是类型注册+显式dependencies，不含对象adapter/实时实例。Parent供catalog generation/fingerprint、canonical type identity与实际调用者；筛选/取消/focus归共享picker。没有typed Struct资产/schema migration/compiler，不把Viewer当可编辑类型资产 |
| OVW-36 Audio全族 | Wave/波形与Cue/Class/Submix各自Toolkit目标，不同图schema；同一资产route/文档/播放输出 | 三图的具体字段/接线/编译发布合同见[RT-AUDIO01](08-resource-tools-shared-contracts.md#rt-audio01cueclasssubmix图消费合同)。当前Mixer有真实graph/校验/有限编译；Cue程序图、Class继承、typed持久文档和可用高级Submix仍open；资源负责业务，材质仅表面，Parent接runtime/provider |
| OVW-37 Font/Face | 真实FontAsset文档的source/face_index/family_members/variable_instances/fallback_families/render_strategy；Font预览/Details与Face信息分工 | [FontAsset](../../../zircon_runtime/src/asset/assets/font.rs)已有上述字段、FontAssetMetadata；text feature下另有CompositeFontDescriptor(default_family/sub_fonts/scripts/ranges/cultures)。作者Toolkit/保存/undo、glyph/atlas/字体ready仍未接。资源供稳定asset+face key/解析revision/source/cooked hash；Widget消费字体/回读，Parent renderer/provider接真实glyph与admission。Penpot字体UUID不是资产ID |
| OVW-38 StringTable | 资产table/key/源文本与reference列表、导入/编辑/重命名、真实预检→提交→undo/保存 | [UiLocalizedTextRef](../../../zircon_runtime_interface/src/ui/asset/localization/localized_text_ref.rs)已有key/table/fallback且只检查key非空；不证明StringTable作者资产或跨文档rename。资源牵table/key identity、引用定位/patch+inverse适配；Parent主持跨文档revision/锁/原子提交，设置只Localization入口。显示文案变更不迁key；key rename必须保持真实引用或报告冲突，不能全局字符串替换 |
| OVW-41 Tags | 真实metadata/index中的tag集合，选择、过滤、添加/移除的预检和脏态/保存反馈 | 当前meta/registry已有BTreeSet<String>，非完全无模型；作者管理/跨资产batch/history未证明。遵循原tag字符串与索引语义，不另建标签store/大小写归一化；Parent接metadata原子保存与revision发布，本域设计合并/替换/取消语义 |
| OVW-44 NNE模型 | 真实neural.model选择、source/import信息、模型I/O与验证诊断的只读详情；能力允许才执行 | [Neural editor plugin](../../../zircon_plugins/neural/editor/src/plugin.rs)已有neural.model.onnx importer/neural.model.import操作工厂及.onnx→.znn路径约束；不能写成无导入能力。尚无asset toolkit/模型详情窗口或UE NNE后端等价证明；Parent持有neural执行/部署provider，本域仅模型资产适用性、入口及业务投影，不跑合成推理/造性能数值 |

Font/Face与StringTable沿同一个registry/toolkit打开原资产，重复打开应聚焦原文档；引用跳转带project+asset/source locator+revision并重新解析。来源或generation失效时保留错误而非跳同名对象。当前没有为这些目标新增ZUI文件或注册虚构route，仍保留四个原生产owner及稳定身份。

新增验证工作项（均pending）：RT-I08 Audio：支持/拒绝连接与一次history、真实compile/publish/output回读、失效/last-good、save/reopen；RT-I09 Font/StringTable/Struct/Tags：真实字体fallback与中文glyph、key/reference预检/原子提交/undo、类型picker取消返回、metadata与索引同代次；RT-I10 Analysis：真实trace关联、未知/部分统计、共享占用不双计及只读缓存无删除副作用。只有对应真实数据/命令、官方导出、托管聚焦检查和正常EXE逐图证据取得后才能改passed/accepted。


## 11. 单个真实Browser的身份与接入合同

接续Parent“资源第二次接续”，本轮只固定一个真实项目输入，不重做全族盘点。[browser-real-input-01.json](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-real-input-01.json)是磁盘registry候选和完整原控件/事件映射，**不是当前运行Engine的catalog/provider回执**。全部live revision/ready字段明确为null，不能用持久化文件的格式号或Penpot媒体身份补齐。

| 身份/输入 | 实际来源与值 | 准入边界 |
| --- | --- | --- |
| 项目 | [Vampire manifest](../../../examples/vampire/zircon-project.toml)，Zircon Vampire Roguelite，GUID `837f1b65-bbcf-4b72-af59-1317f512a8fc`；`examples/vampire/assets`为真实目录 | manifest format3/library3不是catalog revision。现有rendering/texture等selections仅client_runtime，未改示例manifest、未证明editor activation |
| Registry | [持久化asset-registry.json](../../../examples/vampire/.zircon/registry/asset-registry.json)，format1、563 entries；res://textures下该Texture恰为1条 | 当前完整AssetWorkspaceSnapshot、activation generation及catalog_revision尚未取得；不能把1或563填为revision |
| 资产 | `e0cca9ad-973a-4dfd-a306-8116b9930c93`，`res://textures/jungle_ground_albedo.png`，Texture；registry、邻接.zmeta及entries的UUID/locator一致 | 当前ResourceMetadata.id/state/revision、文档revision和selection authority未知；本板蓝色只是设计选择意图，不能声称Engine已选中 |
| 源图 | [jungle_ground_albedo.png](../../../examples/vampire/assets/textures/jungle_ground_albedo.png)，256² RGBA、86085 B，SHA-256 `9d567dde6505b7a77b3cc42c397076fe97a5ed55d59e5ba333da8fc56662d28d` | 原文件未修改；Penpot显示其真实像素，未作为GPU/当前thumbnail provider ready证据 |
| 导入结果 | 邻接.zmeta format7，texture_importer.image v1、config_hash `bd60acb658c79e45`、source_digest `ae73852a65e9c03c`、preview_state=ready；实际.zasset 162485 B、SHA-256 `b3bfc1d965d1bde92c6d9db73da686cd0948ef6fda3cee6479add6ffebbf353c` | artifact为`lib://textures/f792b3a7-a64e-c0bc-a422-1109d0c75813.zasset`，磁盘在项目`.zircon/cache/assets/textures`；存在/旧ready不等于当前resource/provider ready |
| Thumbnail provider | [generate_preview_artifact](../../../zircon_editor/src/ui/host/assets/preview_refresh/generate_preview_artifact.rs)的Texture SourceImage分支；[record_projection](../../../zircon_editor/src/ui/host/assets/manager/project_sync/record_projection.rs)使用`ResourceMetadata.source_hash`建立record | [PreviewArtifactKey](../../../zircon_editor/src/ui/host/assets/preview.rs)使用asset_uuid+实际source_hash，目标cache_root/editor-previews/UUID-thumbnail-source_hash.png。不能把source_digest或SHA-256假定为同算法；本轮没有创建PreviewCache/重导入/生成缓存 |

### 原始source、controls、events和导出映射

原[asset_browser.zui](../../../zircon_editor/assets/ui/editor/asset_browser.zui)仍v2、129 nodes、121 unique controls、11 events；root=`asset_browser_root`/`AssetBrowserRoot`，7个widget imports及strict style import、所有原binding/slot值保留。上述JSON登记全部121控件映射和11条原事件，并另存Drawer/AssetWindow/DataTable三owner的source指纹/事件合同。下表是首个Browser接线位置，**是原身份对应提案，不是已附官方codec或已导出**。

| 设计角色 | 原node / control | 原事件/生产路径及接入限制 |
| --- | --- | --- |
| 项目/树 | `sources_panel` / `AssetBrowserSourcesPanel`；`sources_scroll_body` / `AssetBrowserSourcesScrollBody` | 来源为完整snapshot.folder_tree和selected_folder_id；行模板`AssetBrowserSourcesRowPanel`及Icon/Name/Count保留。生成row_XX/TreeRowXX是临时可见索引，不作folder ID |
| 搜索 | `toolbar_search_field` / `SearchEdited` | `Workbench/AssetBrowserSearchEdited`，Change→`workbench.asset_browser.search.edit`；原WorkbenchSearchInput的query/placeholder与IME/文本undo作用域保留。公共native实例见下节，尚未接入产品 |
| 类型过滤 | `toolbar_kind_filter_dropdown` / `AssetBrowserKindFilterDropdown` | `Workbench/AssetBrowserKindFilterChanged`，Change→`workbench.asset_browser.kind_filter.change`；用真实支持类型，不按显示label猜kind |
| 视图 | `toolbar_view_mode_list_button`、`toolbar_view_mode_thumb_button` / 对应AssetBrowserViewMode…Button | 原List/Thumbnail两route保留；Column和通用多选set/lead/anchor仍缺，不用Penpot按钮把它们写成实现 |
| 定位 | `toolbar_locate_button` / `LocateSelectedAsset` | `Workbench/AssetBrowserLocateSelected`，Click→`workbench.asset_browser.locate_selected`；请求真实stable identity定位，不能只改选中颜色 |
| 结果与tile | `content_panel` / `AssetBrowserContentPanel`；`content_asset_table_scroll` / `AssetBrowserAssetTableScroll`；`content_preview_card` / `AssetBrowserContentPreviewCard` | 结果沿已有visible_assets和pointer route/selection authority。AssetTile属于本域业务native组合，改版/原template替换及虚拟化由Parent source-closure准入；不把媒体UUID传给SelectAsset/双击 |
| 导入 | `import_path_field` / `AssetBrowserImportPathField`；`import_button` / `ImportModel` | 原path Change→`workbench.asset.mesh_import.path.set`和Import Click→`workbench.asset_browser.import_model`保留；此入口仍是模型导入，不将新Add/Import/Save All文字当已接通通用命令 |

[retained click](../../../zircon_editor/src/ui/retained/app/asset_content_pointer/events/click.rs)先取committed布局再以当前UUID、locator、resource_revision、shared visible items、catalog_revision、view_mode核对双击；当前500ms条件保留。资产激活仍走真实registry/toolkit；本轮没有让缺专用Toolkit的Texture假装打开编辑器。drag/context/保存路径/Editor undo及durable后cancel边界仍按原差异表open。

### 单一scroll owner的具体修改提案

[完整提案与源码指纹](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-scroll-owner-contract-01.json)只读核对现有owner，没有应用Rust补丁。每个Browser/activity pane沿`AssetSurfacePointerState.tree_state/content_state`及现有bridge/dispatch/UI writeback，禁止另建scroll或selection store。

以下为r02形成时的修复前诊断，当前源码状态以[Parent四列表接续](#parent第三次接续滚动写回与动画合同)为准：[tree bridge](../../../zircon_editor/src/ui/retained/asset_pointer/tree/bridge.rs)原41…66与[content bridge](../../../zircon_editor/src/ui/retained/asset_pointer/content/bridge.rs)原59…85在sync内clamp内部offset；[snapshot sync](../../../zircon_editor/src/ui/retained/app/pointer_layout/asset_surfaces/sync.rs)原32/36行忽略返回bool，未在该路径将clamped state写回outer；[ui_writeback](../../../zircon_editor/src/ui/retained/app/pointer_layout/asset_surfaces/ui_writeback.rs)原17/19行却读取outer offset。**目录/结果缩短后paint与hit-test状态可能分歧，当时尚未在Engine复现/修复**。原pane-size路径已经消费updated state，提案复用其写回方式并沿同一outer owner，禁止复制第二个滚动模型。原候选/失败与这一历史诊断保留，不据此将现已修复源码重新记为未修改。

选中定位按完整snapshot中的stable folder ID或(UUID,locator)解析当前index，使用原tree/list/grid metrics及测得viewport作最小滚动并clamp；不把pointer_projection丢弃的project/folders当空项目。仅显式Locate/Find/导航或键盘lead变化请求一次reveal，不在每次preview/catalog发布时强制拉回，保留用户wheel。Browser Thumbnail需按实际columns计算row；[Drawer旧reveal](../../../zircon_editor/src/ui/dispatch/workbench/unreal_content_drawer_adapter.rs)272行只按authored线性item step定位，不能直接复用为Grid/目录树算法。

首/尾、Home/End/arrows、popup Esc/焦点返回和selected active/inactive/focused由同一shared focus owner接线；当前tree row focused=false、pointer state只有hover/offset，未证明键盘链。原tree metrics是header48+1、row28/gap4、两端8；本域草稿row28/gap0不同，不把现有生产值认作UE测量通过。640 WOC的knight选中不可见、900尾部裁切继续保留，新Vampire十行短树不关闭这两个case。

### 当前可编辑板与公共库消费

本小节是rev47首个Browser消费的历史记录；四个公共主件的后续受控验证见[第13节](#13-四个已发布公共主件的受控消费)，既有Browser对象与原source映射未替换。

实际board=`a64c5b04-bff7-401d-9f58-e04e4f5c26ec`，1280×800，放在原08 file/page；真实目录Content+九个一级目录，选择textures/唯一Texture。复用AssetTile component `30625d51-64a3-469f-84c5-79e156375f8c`、main `a87b3a71-49e3-432b-9b59-be99063789b2`，本板copy `ff9db784-1073-47f3-b33f-9864612eed7f`，未改main。真实PNG的file-media ID是`61207385-5831-4633-aa06-e8ca4a6501aa`，上传后取回原字节SHA相同，明确不同于Engine UUID。

rev41 authoring保存，rev42通过原生Add export保存设置；rev43只把本实例name行框20→32、tile172→184，保留12px字号，修复原图两行文件名压到类型行。原[失败图r01](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-real-candidate-native-export-r01-1280x800-a64c5b04.png)与[修复后r02](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-real-candidate-native-export-r02-1280x800-a64c5b04.png)均保留并已逐张打开；r02原生1×PNG为71557 B、SHA-256 `2a2523f1d35e2574b39aa7d3b7075d33e2059f47bf3636facfdb759b638d95d8`。两个浏览器重开证明32行框、184tile、原main/copy及54新增ID保持，旧1163非根对象不变。它是local variant草稿，不偷偷提升公共AssetTile版本；正式props/slots和统一tile extent仍交Parent登记。

这两次浏览器download均canceled；只读原生PNG Blob，不改变URL返回/revoke，不能称下载成功、官方ZUI导出或EXE。首次longname ACK检查把assign.value.touched数组误转Set而失败；独立复核按原始请求原样通过现有verifySaveResponse，42→43，无重放/放宽guard；失败回执保留。author receipt的boards=15误含页面零根，独立fresh inventory排除零根后是14板，数量更正在此，不改原回执。

13:24:46Z独立API发现Shared file已rev1/isShared=true、4组件、零color/typography，覆盖此前rev0历史空库存。真实组件为Button `6ddc248b-bd35-80c5-8008-d6c8c01096f3`、SearchInput `6ddc248b-bd35-80c5-8008-d6c8c3ce892e`、TreeRow `6ddc248b-bd35-80c5-8008-d6c8c7a79944`、PropertyEditorRow `6ddc248b-bd35-80c5-8008-d6c8cab1e7d0`。canonical-record v1是发布映射格式，不能当Rust组件或ZUI版本。共享theme/style/完整family closure仍未证明。

本域用官方link-file-to-library链接此已发布库，native实例化SearchInput至copy `753e5eeb-6e53-8014-8008-d6cb5445fc1c`，main=`6ddc248b-bd35-80c5-8008-d6c8c0f9c196`，43→44原生autosave已严格ACK；7个后代保留foreign shape-ref，本域local components仍3，未复制公共main或改共享文件。它对应上述原SearchEdited消费合同；后续44→45→46通过正常native update-shapes/update-layout-child完成query栏顺序与fill；47补本域业务映射，fresh保持library/copy/全部7个sourceRef。旧占位文字ID保留，hidden+absolute退出flow，由真实实例替换显示；没有隐藏目录或缩字。复用不等于可逆source export或运行时绑定接通。get-file-libraries只给关联摘要，其缺data导致旧link receipt中的components=0是解析默认值，不是推翻get-file读回的4组件。

首个真实Browser的后续准入顺序：Parent提供实际editor project/activation及当前snapshot/resource/provider回执；公共owner按原source/control/event/import/slot完成受管family source-closure和normal producer；本域落实原ZUI及业务adapter后提交现有最小Jenkins插件/Rust过滤测试，再排正常EXE三尺寸和实际144 DPI。没有source metadata/producer/桌面时段时不从固定LevelEditor绕过，不换capture请求身份。当前没有product apply、托管测试终态或双端图/1 DIP测量，passed/accepted仍0。


最终[原生PNG r03](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-real-candidate-native-export-r03-1280x800-a64c5b04.png)是1280×800/1×、72085 B、SHA-256 `2812e2db01cf74afc674885c9969cab8abd044fc0c30bae73e95780e62a762e9`，已实际打开并完成[三图逐图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-real-candidate-individual-review-r01.json)。fresh raw SHA为`77fecf6f7b480b10f62bc0ac7df4ea7cdef62474fdc50d88167e398a7fe67e06`；3原main及此前1163非根对象全部属性不变，新增61个ID全保留，20copy含1个foreign。三图download仍canceled。47对ACK沿原guard，旧80份原字节哈希一致，新7对严格核对，未重放或改旧失败。

公共SearchInput copy外框832.00004，内部surface239.99998/text203.99998，图中仅240宽有底色；外框fill不替代内部响应式通过。本域不覆盖公共main。请Parent/pipeline沿[refreshFieldDecoration/layoutFieldText](../../../tools/penpot/apps/zircon-zui-plugin/src/penpot-field-controls.ts)及[refreshAssetTextLayout](../../../tools/penpot/apps/zircon-zui-plugin/src/penpot-asset-renderer.ts)，在当前canonical source/实例/字体/guard下更新parts、文本与source allocation，保持无关改动可检出。shared color/typography仍0；跨族样式传播/reset仍open。

交[3文件写回补丁r02](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-real-candidate-scroll-writeback-proposal-r02.patch)：两bridge提供现有state回读，snapshot sync写回clamped tree/content state，无新store。`git apply --check` exit0，**未应用/编译/Engine复现**；不含selected reveal/keyboard/focus。首版patch格式失败和一次错误原因解释保留，[r02核对](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-headless-layout-01/browser-real-candidate-scroll-writeback-check-r02.json)更正为：源码LF，初版Windows文本输出变CRLF，r02仅修产物格式，没有ignore-whitespace。Parent协调这3个共享路径后补shrink/empty回归，不能把apply-check当Rust测试或关闭RT-G06。

### CS-XR 接续：真实跨域导航及焦点返回

Parent已接收角色域[第15节](character-skeleton-design.md#15-九族归属与真实资源关联打开合同)与[原9项合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/cross-domain-navigation/source-and-navigation-contract.json)。本域将其并入当前单Browser、RT-S06及RT-S10；不新增第二PreviewSession，不改变本轮真实Texture Browser优先级。[本域消费回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-cross-domain-navigation-01.json)保存25项实际生产源指纹、原13项指纹的逐项对照、31项尚未观察的运行结果字段和全部9个pending case。旧指纹保留，文档/统筹记录的变化不回填成旧基线。

归属固定：Static Mesh/Texture仍08，Skeletal Mesh/Skeleton/Physics归角色，Material归材质；Parent统筹type registry、Toolkit host/focus和唯一PreviewSession。角色只提供实际source revision、node/mesh/skin/primitive与既有SkinN Data引用；材质/纹理关联打开按目标真实registry进入对应owner。Vampire的jungle_ground_albedo是本Browser持久化候选，不因此成为Wolf/Knight关联材质的纹理；未解析引用不替换成现成测试图片，未知UUID的picker确认仍禁用。

| 链路 | 当前源码实际行为 | 当前闭环要求与实测缺口 |
| --- | --- | --- |
| Browser激活 | [click.rs](../../../zircon_editor/src/ui/retained/app/asset_content_pointer/events/click.rs)重核catalog_revision/view_mode、可见item identity及UUID/locator/resource_revision后派发OpenAsset；Scene另走真实场景入口 | 使用当前真实snapshot和project activation，不给GLB根Model或首Mesh猜Toolkit。当前Vampire Engine激活/资源revision仍未取得 |
| indexed type与registry | [asset_type_id_for_locator](../../../zircon_editor/src/ui/workbench/project/asset_workspace_state.rs)仅按kind转AssetTypeId；[enabled_asset_types_for_shell](../../../zircon_editor/src/ui/host/editor_extension_registration.rs)组合builtins和实际enabled contributions并缓存 | Mesh不足以区分static/skinned。实际registry generation/definition owner/能力、Texture与Mesh Toolkit均须读当前Engine；内建无相应Toolkit不等于所有插件均无贡献 |
| command | [OpenAsset](../../../zircon_editor/src/ui/host/editor_event_execution/asset_event.rs)取得descriptor的open_operation，查实际command后调用[ensure_enabled](../../../zircon_editor/src/core/commands/registry.rs) | 记录真实command ID、when/能力结果及诊断。ensure_enabled通过不是命令执行、文档加载或preview-ready回执 |
| host与domain | 同一OpenAsset链依次open_view、update_view_instance_metadata、focus_view；规范[AssetToolkitOpenRoute](../../../zircon_editor/src/core/asset/toolkit_route.rs)只有asset_locator/open_operation且deny_unknown_fields | 保留原route，分别取得ViewInstanceId、metadata写入、域文档/加载/资源ready结果，不把Opened状态文字或非空instance当完成。open后metadata/focus失败可有部分host状态，须如实记录，不能自动关闭复用/dirty实例 |
| 关联Reference | [现有reference click](../../../zircon_editor/src/ui/retained/dispatch/shared_pointer/asset_reference.rs)调用ActivateReference，当前只navigate_to_asset | Find导航保留。显式编辑关联Material/Texture才发目标OpenAsset；不能先ActivateReference改变Browser目录/选择后把失败称为来源未变 |
| focus结果 | [focus_view](../../../zircon_editor/src/ui/host/layout_commands.rs)返回布局diff或focus identity是否变化；[原native observer](../../../zircon_editor/src/ui/retained/contract/window/focus_observer.rs)另观察native输入窗口 | false可能只是已聚焦，不作失败；true也不证明实际窗口/控件得到输入。分别核当前session.focused_view、source window、稳定control key与当前generation |
| 关闭/返回 | 原EditorManager关闭链已有save-in-progress、DocumentCloseRevision与dirty save/discard边界 | 实际关闭提交后才由原host恢复当前有效来源；cancel/未完成保存不算关闭。来源已关闭/重导入时按稳定文档/对象键重新解析或报告过期，不按旧行号恢复、不抢其它modal焦点 |

来源保持的边界是**显式OpenAsset派发前**的来源文档/selection/preview与当前Browser query/folder/scroll；Browser首次点击造成的合法选择与之后打开失败分开记录。locator非法、未indexed、无definition/toolkit/command或disabled的已见guard都在open_view之前；运行验证仍需证明没有额外清选/换subject/新空窗口。嵌套Character→Material→Texture使用原host的导航来源上下文，按实际关闭次序返回；不能在08另建focus stack或preview player。

独立重读相同GLB的JSON chunk（不解压accessor、不导入）核对：Wolf Node48→Mesh0/Skin0共1绑定；Knight Node23～31→Mesh0～8/Skin0～8共9绑定，每项primitive/material、joints顺序及inverse-bind accessor与角色原回执相同。源SHA仍Wolf `655da121afa8768c13a3b2aa172e6875c716d06e352110bf68eaa4f34c9fec6e`、Knight `ce0d0a5ec1d67594629ecac20a7576fde22a44456e7628e8eb100891bc6dda97`。这些index只在该source revision内有效；SkinN/Skeleton等只是源label预期，运行UUID/locator仍null。静态crate源码0skin也不能授予未知UUID确认或证明已路由Static Mesh工具。

[MeshSkinAsset](../../../zircon_runtime/src/asset/assets/mesh/metadata.rs)当前只有逆绑定矩阵；plugin的per-mesh map保留首个node skin，不能用于消除同mesh多skin歧义。复用[SkinN Data](../../../zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs)已有`skeleton_asset / joints / inverse_bind_matrices`及[逐primitive材质引用](../../../zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs)。需要持久化的多义关联由Parent资源/Runtime owner收具体DTO提案，08不复制映射或修改导入器。Parent交角色的meshopt白名单与negative测试最小修复只是CPU前置；其源码变化、正负例和托管回执由原owner核验，不推断resource ready、GPU帧或EXE验收。

| 原场景 | 08最小消费与断言 | 执行状态 |
| --- | --- | --- |
| CS-XR01-01 static/skinned | 同一实际indexed type用于Browser/picker/OpenAsset，不按.glb/显示名分流 | pending；未运行 |
| CS-XR01-02 node/skin/skeleton | source revision内解析既有SkinN Data、node/mesh/skin/primitive；同mesh多skin不取首项 | pending；只核源JSON |
| CS-XR01-03 Knight九部件 | 九项绑定全部参与，不以Mesh0代表角色 | pending；只核1+9实际绑定 |
| CS-XR01-04 Material关联 | selected primitive的实际Material AssetReference→当前registry/command→材质owner的真实host/加载/focus结果 | pending；运行locator未知 |
| CS-XR01-05 Texture关联 | 实际Material纹理引用→Texture UUID/locator→08工具；保留嵌套来源 | pending；不借用Vampire图片代替 |
| CS-XR01-06 missing/disabled | 返回实际诊断；来源文档选择/preview与OpenAsset前Browser状态保持，未知picker确认不启用 | pending；缺真实前后状态 |
| CS-XR01-07 generation/focus return | 实际close提交后重核当前来源文档、对象和控件generation，过期重新解析/报告 | pending；无native输入证明 |
| CS-XR01-08 PreviewSession释放 | RT-S06与CS-D02同一Parent session；关联工具不复制world/clock，最后消费者由Parent释放 | pending；无真实session/frame |
| CS-XR01-09 正常Engine证据 | 原验证请求/normal producer、真实ready/三尺寸/144DPI、双端差异与逐图review | pending；passed/accepted=false |

精确共享差异见[RT-S06补充](08-resource-tools-shared-contracts.md#rt-s06-补充cs-xr跨域导航)。本次仅改本域四份文档与证据；未修改公共type/registry/route/host/Runtime、四原ZUI、Penpot或服务，未编译或占桌面。rev47是13:41:30Z的既有真实设计读回，本次未新读或修改页面；31项运行结果均null，不能把本合同当产品已接线。

### Parent第三次接续：滚动写回与动画合同

RT-S10当前状态：**`source-fixed / static-passed / managed+GUI-pending`**。source-fixed/static-passed对应Parent的五路径实际修复和静态检查，不对应五个Rust测试或GUI通过。本域只消费既有修复，不重叠修改这些公共路径；源码与本域状态对账见[状态回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/rt-s10-status-reconciliation-01.json)。

已按Parent的[asset-tools-followup-03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/asset-tools-followup-03.json)独立读回原tree/content/reference三个bridge、[asset_surfaces/sync.rs](../../../zircon_editor/src/ui/retained/app/pointer_layout/asset_surfaces/sync.rs)及[五项回归候选](../../../zircon_editor/src/ui/retained/app/pointer_layout/asset_surfaces/tests/scroll_sync.rs)，5/5源码hash匹配。Parent沿原AssetSurfacePointerState同步snapshot后发布树/内容/references/used_by的clamp结果，再走原UI writeback，无第二scroll store。本域r02提案原样保留；Parent同层统筹五路径修复，并非逐字应用该提案，08未重写共享源。

Parent源patchDigest为`0b465ecc460bd029ad0beb1e74abded0eafa9a646c37c17dbfa69244f8be72f0`，rustfmt/diff与五路径change-guard passed，原guard 0 error/warning/suppression可复用。五个无文件/桌面的纯内存回归覆盖缩短/空、两surface profile/resize、保留手动scroll与geometry代次、当前thumbnail hit，**均未执行**。原`ue-asset-scroll-writeback-20261024-01 / validate-asset-scroll-writeback-01`及两次capture失败保留：第一次仅Actor文档漂移；确认真实四分钟窗口后Actor稳定，第二次仅四份角色文档变化；HEAD/index/membership/cache一致，没有Cargo/queue URL，08不重新提交。Parent回执closeout显示先前窗口已结束、无当前持续写入限制；下一次capture由原owner按两owner实际交集ACK/sha/时段接续，不从投递或静默推断窗口、不全局停写、不放宽source/doc guard。selected reveal/HomeEnd、focus-return/旧async结果拒绝、多选/Column及旧900/640可达性仍open，不以clamp关闭。

动画[资源消费合同RT-S08/09](animation-shared-controls.md#资源消费合同-rt-s0809)已交精确参数/slot/注册/handler与Stop/Seek迁移提案。本域复用CurveModel/Selection/Transform与inverse，负责独立Float/Vector/LinearColor typed资产/channel/key/codec/history，不强制动画TrackPath/FPS或transport。声音消费真实PCM/per-playback/session/generation与输出回读，区分Kira位置/fallback cursor、f32 seek请求/夹取实际位置，关一个试听不Stop整个设备。公开domain/session command slot与共享发布/正式导出仍缺；六原route/event保持，不暗改Prev/Next/Pause，不将Record切色当录音。动画三组件尚未发布，不复制为资源main。

独立核查与8项合同来源hash见[消费回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-parent-clamp-receipt-01.json)。该读回轮未改公共Rust/Penpot/服务/桌面或编译；正常EXE/144DPI和双端逐图pending，passed/accepted=0。

## 12 Audio最小只读适配与真实资产文档提案

Parent最新分工继续有效：SoundWave及Audio全族（OVW36/RT20 Cue/Class/Submix）作者业务、模型与文档提案归资源08；材质只供中立graph，Parent接sound runtime/provider/生产输出、公共host和跨文档事务。没有另一Audio实现chat或待用户分配owner。当前仍优先一个真实Browser；本节只实现现有声音模型的域内最小适配，不启动另一预览或改公共入口。

实际修改限[sound editor lib](../../../zircon_plugins/sound/editor/src/lib.rs)、[authoring入口](../../../zircon_plugins/sound/editor/src/authoring.rs)、[Mixer只读投影](../../../zircon_plugins/sound/editor/src/authoring/mixer_snapshot.rs)及[三个回归测试](../../../zircon_plugins/sound/editor/src/tests/mixer_authoring.rs)。`SoundMixerAuthoringSnapshot::new(&snapshot)`借用真实SoundMixerSnapshot；tracks保留源顺序和原对象引用，routes区分Parent与每条Send，同端点并行send及pre/post标记不合并。重复track ID、未知parent/meter、原effect顺序、非默认controls和完整source均保留；不替validator过滤错误图，不补零meter或ready状态。

source_index/source_send_index只定位当前snapshot内对象，不能作持久NodeId/EdgeId。snapshot没有project/source revision/output generation字段，消费方须携带真实provider的scope/fence，不能从track ID、Vec位置、hash或counter伪造。重复ID需要snapshot内身份或显示冲突，不按ID覆盖并声称可编辑；没有稳定send key时不启用歧义删除。该投影没有mutation、持久化、编译、播放或UI消费者，不能据此关闭Submix缺口。

| 作者文档提案 | 真实输入与本域最小字段 | 仍缺的模型/操作与验收 |
| --- | --- | --- |
| SoundWave | 原SoundAsset的uri/sample-rate/channel-count/layout/interleaved PCM；SoundClipInfo的frame_count/duration；SoundPlaybackStatus的clip/playback/cursor/实际状态。外层绑定实际project、asset UUID、资源revision及当前输出代次 | 先只读PCM/metadata；clip/playback ID不当asset UUID，不用动画FPS或timer造播放头。破坏性变换/导出/保存/Undo尚缺，六条共享transport route不覆盖 |
| Cue | 提案为版本化程序文档、稳定node/slot key、实际Wave AssetReference、Output root及已支持节点kind/schema | 须有真实lowering/source map/compile/execute与失败last-good。Mixer graph不是任意Cue程序；未注册新type/command ID，未创建伪完整palette或假资产 |
| Class | 显式Class资产引用、parent/child、属性override/继承求值、Wave/source绑定与有效值来源 | track.parent不等价Class继承；需原格式/真实解析和变更命令、循环/缺失引用诊断及Undo/save。缺模型时不启用无效果编辑 |
| Submix | 持久asset UUID↔runtime track ID的真实映射；主输出/aux send/sidechain/effect order分型，sample-rate/layout与source/output代次；先复用只读snapshot | 持久typed文档、稳定边身份、asset-bound命令和事务仍缺。不能将普通track改名为已支持Submix；endpoint/soundfield/dynamic与效果运行以backend为准 |

命令沿[authoring_bindings](../../../zircon_plugins/sound/editor/src/authoring_bindings.rs)的20项`SOUND_MIXER_OPERATION_PATHS`和既有payload schema/capability，不建平行Audio路由。track/send/effect/preset/sidechain/automation/dynamic_event与output动作仍需实际manager/controller承接；目前仅核源码与descriptor，未取得选中资产→host handler→运行/文档history回执。[SoundEditorLiveOutputController](../../../zircon_plugins/sound/editor/src/live_output/controller.rs)已直接持有真实backend/device managers，Refresh/Configure/Start/Stop返回实际失败和snapshot，不复制controller/backend。

编译范围按原`validate_m1_surface`：非空effects、pan/左右gain/delay/invert/solo/bypass等特定高级controls及pre-effect send被拒绝；不把所有gain/mute误报为被该gate拒绝。inactive configure只做通用validate，不证明Start能compile；runtime rollback不等于Editor Undo。只读视图保留不支持字段供诊断，不把它们变成可播放或翻capability制造通过。

精确四文件rustfmt/diff通过；change-guard r01因子模块隐式路径未解析报`rust.unreachable-module`，改为实际显式`#[path]`后r02为0 error/0 warning/0 suppression，未改或抑制检查器。r01 patch/失败receipt保留。三个测试覆盖重复ID/未解析parent、parent与并行pre/post sends、effect顺序/controls/未知meter。当时正式Jenkins入口在dispatch前拒绝harness参数，纯parser核实当时20b3bf85… capsule只含material/prefab/glTF配方、无sound editor。原失败保留；r2恢复后的当前2790fb51…及原身份QueryOnly见[第15节](#15-jenkins-r2原身份查询与真实texture-provider接续)，不把静态检查计成Rust通过。见[Audio源码合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-mixer-source-adapter-01.json)、[原入口拒绝](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-mixer-jenkins-run-01.json)及[原recipe缺口](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-mixer-route-parser-01.json)。RT-AUDIO01-J01保持原单包/闭包/测试范围，不改公共准入、不换请求身份或运行本地Cargo。

Parent已追到native PNG404的共享frontend `/api/export` route/exporter缺失，原public owner接续；本域不重复点击或重启。此前九次download canceled与原始PNG Blob成功读取各保留原事实，不倒推全由404/revoke造成。正式ZUI source-closure/导出和正常Engine仍分别pending；本轮没有Penpot页面修改、新截图、桌面/DPI或三尺寸验证，passed/accepted仍0。

## 13 四个已发布公共主件的受控消费

接续Parent最新四主件授权，仅在08专属file `b4a1e3ce-9e6a-81fe-8008-d69d60ef2172` / page `b4a1e3ce-9e6a-81fe-8008-d69d60ef2173`新增验证板 **`fb52acb4-3e9c-47cd-98ef-d6d6f1540b11`**，名称`08 Shared controls · native linked-instance validation`，1280×800、位置(0,5760)。原生flex、padding24/gap20按实际子对象堆叠设为column-reverse；首图倒序已修正并保留。板上三个可编辑说明文本标注实际Vampire输入和开放限制，不接管Browser来源、控件、事件或项目激活。该板是公共源兼容性验证，不能替代资源工具内容面或正常产品。

Shared源file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` / page `b4a1e3ce-9e6a-81fe-8008-d69d607aac13` / **rev1 / isShared=true**。正式`get-file-libraries(file-id=08 file)`返回HTTP200且链接rev1。通过当前Penpot实际native `instantiate_component`创建以下foreign copy；未detach、复制main、改Shared或其它文件。

| 原主件 | component UUID | main / 根shapeRef UUID | 本板copy UUID | 根与后代sourceRef |
| --- | --- | --- | --- | --- |
| WorkbenchButton | `6ddc248b-bd35-80c5-8008-d6c8c01096f3` | `6ddc248b-bd35-80c5-8008-d6c8bd9b057f` | `5698dd90-efc9-8032-8008-d6dd4c845807` | 2/2保持 |
| WorkbenchSearchInput | `6ddc248b-bd35-80c5-8008-d6c8c3ce892e` | `6ddc248b-bd35-80c5-8008-d6c8c0f9c196` | `5698dd90-efc9-8032-8008-d6dd4d5bb281` | 7/7保持 |
| WorkbenchTreeRow | `6ddc248b-bd35-80c5-8008-d6c8c7a79944` | `6ddc248b-bd35-80c5-8008-d6c8c452bf0c` | `5698dd90-efc9-8032-8008-d6dd4e455377` | 20/20保持 |
| WorkbenchPropertyEditorRow | `6ddc248b-bd35-80c5-8008-d6c8cab1e7d0` | `6ddc248b-bd35-80c5-8008-d6c8c84d848f` | `5698dd90-efc9-8032-8008-d6dd4f1323ee` | 4/4保持 |

源根`dev.zircon.zui`的canonical-record及sourcePath/sourceNodeId/controlId/instancePath/baseline保持原样，33条shapeRef均解析到各自原main子树。原生canonical-record实际version1，含source/library/component/main/assetBoard身份，**未观察到canonical sourceHash字段**；回执中的磁盘SHA-256只固定本轮读取的源文件，不能补写成原owner的完整源闭包。原本域3个业务component定义、旧Browser公共SearchInput `753e5eeb-6e53-8014-8008-d6cb5445fc1c`及rev47原1224个非根对象逐项完全相等。Shared前后原始响应SHA均`ab087754c3ffa24332a793b04d0e71a662a407ebc79e055083b029c34e4eaf8d`；公共catalog/tokens/Starship未变。

### 实际源字段、覆盖与限制

| 主件与原source | catalog/slot合同 | 本轮实际操作与仍缺的消费 |
| --- | --- | --- |
| [Button](../../../zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/button.zui) | props=text/button_variant，无slot | native text先覆盖`导入 · Import`，server rev50保存；原生reset后独立server rev51读回`Button`、text touched消失，主源/shapeRef不变；再次覆盖`Import`并在fresh rev55保持。只验证本地文字override/reset，未验证全局style传播 |
| [SearchInput](../../../zircon_editor/assets/ui/editor/workbench/primitives/inputs/search_input.zui) | props=query/placeholder，无slot | text覆盖`搜索资源 · res://textures`；root使用原生fill/h-sizing与v-sizing覆盖，fresh宽1232，内部surface≈240/text≈204未伸展。未把原source baseline改签，未证明placeholder正式反导出、query/Change业务绑定 |
| [TreeRow](../../../zircon_editor/assets/ui/editor/components/workbench/primitives/data/tree_row.zui) | 当前catalog未声明可公开props/slots；源内部仍有实际TreeRow状态 | 保留默认`Tree item`及source可见性/锁定/操作装饰，不臆造目录label/depth/selected props，不替换Browser单一树owner |
| [PropertyEditorRow](../../../zircon_editor/assets/ui/editor/workbench/inputs/property_editor_row.zui) | required=true、multiple=false、linear value；允许Checkbox/Dropdown/Field/NumberField/RangeSlider/Slider/Toggle | 四个已发布主件均不在允许类型中。value保持空槽，未填Label/假值/重复Field；属性名/default和source identity保持。依赖原owner发布允许组件并闭环slot/export后才能消费真实属性 |

三个精确公共差异为 **RT-SH4-01 Search内部fill**、**RT-SH4-02 Tree内容/state/affordance公开合同**、**RT-SH4-03 Property合规value组件及slot闭包**。均交Parent/public owner，不授予本域公共main/catalog/theme写权。本域继续保留原真实Browser输入/121控件/11事件、CS-XR导航和滚动/focus合同，未以区域占位关闭任何窗口。

### 保存、独立重开和逐图结果

最新[校核回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-components-trial-01/validation-r02.json)读取rev55：1262对象、524带flex/grid布局的对象、**15实际设计板**（另有零UUID页面根，不计板）、3本域main、24个copy head，其中5个foreign；colors/typographies仍0。初版计数将页面根算为第16板的`validation.json`保留，r02按实际板身份纠正，不改Penpot数据。

[保存ACK核查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-components-trial-01/save-ack-validation.json)对实际`/api/main/methods/update-file?id=…`五对请求/响应逐项匹配file、session、完整changes；匹配各响应lagged中的51/52/53/54/55条目。响应顶层revn及第一条lagged不直接当本次保存证明。早期观察器误用旧路由、漏query导致初始建板/实例ACK漏捕和超时，原始失败保留；不声称本轮8个修订或累计55次均有原始ACK。

原author浏览器关闭后，[最终新browser/context重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-components-trial-01/fresh-reopen-r03.json)认证读取同file/page，全部1262个已保存对象与独立保存后读回完全一致，四实例/覆盖/shapeRef和正式library link保持。该浏览器PID37680已native exit0；前次重开r02因同board DOM节点匹配两项导致crop失败、库请求最初误用id参数返回400、默认Playwright浏览器缺席及window全局入口探测失败均保留，没有安装/补造成功或改前端bundle。

已实际打开并复核[初版倒序图](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-components-trial-01/controlled-board-crop.png)、[修正后完整chrome图](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-components-trial-01/controlled-board-full-r02.png)和[新重开板裁图](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-components-trial-01/controlled-board-crop-r03.png)，逐图记录见[individual-review.json](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-components-trial-01/individual-review.json)。标题/输入→Import→Search→Tree→Property→限制的阅读顺序已正确；Search内部宽度与空value槽清楚保留。图为隔离headless Penpot画布诊断，zoom0.7088235294，viewport1600×1100/deviceScaleFactor1；1280×800板在图上约907×567，**不是1×原生导出、Engine客户区或实际150%DPI**。没有重试公共native PNG exporter。

本阶段未改Rust/四原生产ZUI、未编译、未用桌面或新增捕获请求；原Audio精确托管recipe拒绝与Parent原scroll请求继续各自pending。原source/slot/ref静态核查不冒充官方插件功能检查；4213 host/bundle、canonical sourceHash/slots/export、两族全局style A/B、真实产品加载/截图变化、正常EXE三尺寸与144DPI、双端逐图及≤1 DIP仍未完成。passed=false、accepted=false。

## 14 Parent回执接续与Import/Picker真实能力修正

已实际读取[Parent followup-03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/asset-tools-followup-03.json)。其13:56:59Z的rev47及7/7 SearchInput引用核对是有效历史，不回退本域后续rev55/57。四列表滚动写回的五源码/原请求、两次Cargo前capture失败和source窗口已结束的记录继续由Parent负责；不直接套旧三文件r02、不复制scroll owner/store。动画[资源合同RT-S08/09](animation-shared-controls.md#资源消费合同-rt-s0809)已收件，泛型Curve+资源typed资产/codec/history与Sound per-playback/session/真实位置/capability分工保持，公开context/slots/StopSeek仍是提案，原六route与无capture Record不变。

### Dialog的实际合同与内容板边界

[WorkbenchDialog源](../../../zircon_editor/assets/ui/editor/workbench/primitives/feedback/dialog.zui)只有一个`Dialog` root、control ID=`WorkbenchDialogRoot`，有popup/focus声明；[catalog](../../../zircon_editor/assets/ui/editor/components/catalog.toml)只公开`popup_open/title/message`，源与catalog均没有body/actions slots。声明focus属性不证明实际trap/Tab/Esc/焦点返回。现有RT-D03 Picker板`34f72199-8ba0-4e33-affb-cc45bc69b2be`和RT-D05 Import板`e02dc17d-fde0-4bba-8bd0-ba9d6f1e7e80`仍是可编辑内容研究，未挂真实modal、未接命令；其旧按钮/Field组合也不充当已完成公共复用。

精确依赖见[RT-S12](08-resource-tools-shared-contracts.md#rt-s12pickerimport的modal与真实能力边界)：Parent沿原overlay/focus owner交实际可承载内容/动作的公开源与导出合同，保留根ID和原focus声明；先核runtime真实挂载能力，再决定允许类型/slot，08不传未声明props。Picker从当前registry取得typed资格及UUID/locator/revision才可确认；取消保留调用者属性，close提交后按当前generation恢复实际来源。Import由当前project/activation/backend与真实job ticket准入，不能仅凭源文件存在启用。

### 源能力与四项原生文字修正

[只读能力回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/import-picker-capability-01/source-capability.json)固定9项输入。实际WOC crate为`res://m8/models/props/crate_wooden.glb`、318100 B，SHA-256=`7a5f880c07a60adbc9b50b92557e47a1fa35a7dfa0e3a9fe4fe9f42216d31131`；相邻.zmeta及WOC持久registry本次均不存在，live UUID/activation/catalog revision/type verdict/provider ready均null。磁盘Model候选不能按扩展名确认Static Mesh，更不授予未知UUID确认。

[submit_model_source](../../../zircon_editor/src/core/asset/import_flow/model_submit.rs)只接PathBuf，并核当前index activation/model backend/job admission。[source_plan](../../../zircon_runtime/src/asset/project/manager/scan_and_import/source_plan.rs)保留项目内来源的实际URI；仅项目外.obj/.glb复制到首root的`res://models/<file name>`，同名以DuplicateProjectAssetUri拒绝，不是可任选目标/Import All/FBX options接口。[job](../../../zircon_editor/src/core/asset/import_flow/job.rs)进入Runtime前检查取消；Runtime调用中未交付真实中止回执不能声称已停，成功durable receipt后的迟到cancel不能回滚或隐藏catalog刷新。Runtime事务rollback也不等于Editor Undo。

本轮保留board、frame、文本ID及其布局，只改四个既有text：

| text UUID | 修正结果 |
| --- | --- |
| `32e369fc-c069-4193-b100-e14ba68f36e1` | Destination分成项目内保留res URI、项目外res://models/<file name>两行 |
| `118f8548-dad2-4cff-b4d7-64f901b55e99` | 移除所有来源都会复制的错误，明确项目外.obj/.glb及重名拒绝 |
| `4df9a5ee-604d-4e09-9972-14286c1f8ec1` | Import空来源状态显示当前项目准入要求，未填假来源/成功结果 |
| `8b8a5ec0-02f0-4a1d-97ac-6acff680cc51` | Picker候选显示`crate_wooden.glb · runtime UUID/type unresolved`，Select继续不可确认 |

### 字形缓存失败、原生重测与重开

原生文字事件先保存到rev56，API内容正确，但[第一张实际Import图](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/import-picker-capability-01/import-crop.png)仍显示旧文案，故该轮画面不通过。实际SVG使用旧position-data；[原生text renderer](../../../dev/penpot/frontend/src/app/main/ui/shapes/text.cljs)按position-data绘字，[viewport HTML测量](../../../dev/penpot/frontend/src/app/main/ui/workspace/shapes/text/viewport_texts_html.cljs)在position-data缺席时强制calc-position-data。按[texts.cljs既有await-html-font路径](../../../dev/penpot/frontend/src/app/main/data/workspace/texts.cljs)，先仅对Destination通过native update-shapes去除派生position-data，观察原生测量生成新SVG后，再处理另三项。没有修改frontend/renderer、手写字形坐标、改字体/外框或放宽source guard。

最终rev57的四text除content/派生position-data及native取消重复顶层fills外，身份、父关系、几何、metadata均不变；去除文字后完整content样式相等，另外1258个对象/三个业务component定义/所有公共实例/file features保持。[两对真实保存ACK](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/import-picker-capability-01/validation.json)按file/session/完整changes分别匹配lagged56/57；不拿响应顶层55/56当最终保存修订。

原作者关闭后，新browser实际读回全部1262对象相等且四项content/position-data一致，并产生更新后的两板图；该r04脚本末尾在Picker可见时查询离屏Import节点超时，[原失败](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/import-picker-capability-01/cache-fresh-r04.json)保留。随后[只读新browser r05](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/import-picker-capability-01/fresh-visible-r05.json)在各板可见时逐项核四个真实SVG，全部正确，API原始响应hash与上次独立重开完全相同、own-file writes=0，browser35120 native exit0。

已实际打开[修正后Import](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/import-picker-capability-01/cache-import-r04-crop.png)及[修正后Picker](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/import-picker-capability-01/cache-picker-r04-crop.png)，与rev56旧字形图对照；[逐图记录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/import-picker-capability-01/individual-review.json)保留失败与新图、完整chrome图入口。Import两行分支/说明与来源准入提示可见；Picker真实九文件/318100 B与未解析原因可见，目录尾裁切及modal/focus仍open。图片为headless Penpot画布诊断：Import板640×520在图上约857×696，Picker板900×620约885×610；不是1×导出、EXE尺寸或实际DPI。

最初无后缀原生文字export不可用、辅助替换误处理`$$`而改变arity03名字的写前失败均保留；没有编辑Penpot bundle来解决。本阶段只改本域设计/文档，不改产品ZUI/Rust/共享源，不编译或重投原Jenkins/capture，不用桌面。Dialog真实body/actions与焦点、typed registry/provider、正式插件/源导出/产品加载、EXE三尺寸/144DPI及双端≤1 DIP继续pending；全族passed/accepted仍false。

## 15 Jenkins r2原身份查询与真实Texture provider接续

已读Parent [Jenkins r2恢复回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/jenkins-recovery-handoff-02.json)。原request/attempt/runtime的恢复终态与healthy只证明服务恢复；P21/Runtime11、产品Rust和画面验收没有提升。本域未启停服务、写数据库或要求用户grant。

### 当前入口与本域原请求

本次installed实际指向 **`2790fb5199bd06ca7291ac197f03545eaeae05c56b125117fad268ce19d4b0c6`**，不同于历史20b3。实际逐项核11/11 capsule/client及197/197 immutable driver；driver仍fc553228…、runtime仍start-1792836412672596900，与原Audio identity一致，见[当前读回](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/jenkins-r2-resource-intake-01.json)。这是文件/绑定核查，不是本域重新验收API/Home安装，Home差异仍由原Jenkins owner接续。

原session `01a16d53-60db-7d02-b03e-0042f277783d` / request `ue-assets-audio-mixer-20261024-01` / operation `validate-audio-mixer-source-01`和identity SHA `20c1952b404ef721953299ce5d806410ca6b1425d0f4dfc60dc81bcd39729d41`保持，不发明attempt。仅对原local-cargo正式wrapper增加QueryOnly：plugin manifest、zircon_plugin_sound_editor单包、defaults off/features=[]、lib/mixer_authoring_、test profile、locked/jobs1/test-threads1、物理E根。[实际查询](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-mixer-jenkins-query-01.json)native exit2/pending/**build_scope_policy**，stdout仅准bounded material/prefab/AI或两项glTF配方，未到Parent历史查询的authorization pending阶段。Cargo/测试执行均0，无source capture或新请求，不能据查询推断server已收件。

RT-AUDIO01-J01的client/runner/plugin member catalog一致准入、原身份及真实path闭包已交原Jenkins owner `01a12af6-7511-70d1-a452-61b80678eb56`。[官方排队回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/jenkins-r2-resource-owner-handoff-01.json)消息 `01a16e9e-7207-7a21-a349-4dfecd5ae6af`，无已读/准入/编译回执。未改capsule/validator、借其它filter或改包/manifest/身份绕过；Parent scroll原请求保持独立。

### Shared Search当前只读确认

原认证路径直接用于隔离API context，未复制/输出认证、启动浏览器或触桌面。own/Shared/get-file-libraries实际HTTP200；[当前读回](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/penpot-api-search-recheck-01.json)确认own **rev57/1262对象/3local mains/24copies**、Shared **rev1/isShared/41对象/4mains/0colors/0typographies**，正式库链接保持。own原始response SHA与rev57 fresh证据完全相同，复用原raw，不称本轮又做保存重开或新视觉验收。原Search7/7 shapeRef/source file/component身份有效，832/240/204缺口仍交公共owner，本域未像素覆盖。日期通用解码没有保留libraries.syncedAt，本回执不将空对象当新时间证明。

### 单Texture provider的实际链路

沿15项现有源码核provider，没有新建registry/provider/store。Vampire原project GUID、Texture UUID `e0cca9ad-973a-4dfd-a306-8116b9930c93`及locator保持，manifest/源PNG/meta/.zasset四指纹匹配旧证据。此次未见该项目`.zircon/cache/editor-previews`目录，不创建缓存或运行生成器；[能力包](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-provider-source-capability-02.json)记录实际路径和全部live unknown字段。源256²PNG、已存preview_state=ready、lib://导入artifact不能证明当前Editor thumbnail已生成/加载。

| 实际入口 | 当前能力/限制 | 消费合同 |
| --- | --- | --- |
| [builtin](../../../zircon_editor/src/core/asset/type_registry/builtin.rs)、[generator](../../../zircon_editor/src/ui/host/assets/preview_refresh/generate_preview_artifact.rs) | Texture SourceImage经真实project resolve/image::open写缓存；其它builtin主要为Placeholder。generator查builtin definition，Operation descriptor只报需operation host派发 | 当前enabled registry/自定义provider由Parent原owner接执行；本域不复制目录、不以占位图关闭Mesh/Audio preview |
| [PreviewCache](../../../zircon_editor/src/ui/host/assets/preview.rs)、[record projection](../../../zircon_editor/src/ui/host/assets/manager/project_sync/record_projection.rs) | key为真实UUID+ResourceMetadata.source_hash；path为editor-previews/<uuid>-thumbnail-<source_hash>.png，SourceImage写192² PNG。record可先投影尚未生成的path | 不以Penpot UUID/meta.source_digest或推测hash构造key；非空path不等于存在/decode/Ready，源码unversioned fallback也不作版本证明 |
| [refresh job](../../../zircon_editor/src/ui/host/assets/preview_refresh/request_preview_refresh.rs) | jobs/cache/project准入；完成核scheduler token/catalog revision/Arc row/source_hash/meta path，再核sidecar UUID/URI/source_digest/完整meta后才publish PreviewChanged/epoch | 复用原token/代次；这里已有thumbnail源围栏，运行回归与导航旧async拒绝分别待验，不概括为都无围栏或都已验证 |
| [item snapshot](../../../zircon_editor/src/ui/workbench/snapshot/asset/asset_item_snapshot.rs)、[workspace](../../../zircon_editor/src/ui/workbench/project/asset_workspace_state.rs)、[thumbnail nodes](../../../zircon_editor/src/ui/layouts/views/asset_browser/thumbnail_nodes.rs) | item有path/resource state/revision，无source_hash/preview_state；workspace代次输入有catalog_revision/publish_epoch/resource_sequence。节点has_preview_image只检查path非空 | 沿原projection/image加载结果取状态，不将path判断当provider完成；额外source/DTO由原owner核，不传虚构ZUI/Penpot props |

RT-PV01…06是适配/验收合同，不是新props/DTO/存储：未准入不发请求；真实job准入才显示pending；真实失败/解码结果显示unavailable与诊断；同UUID/source hash/current publication的缓存经原UI图像路径decode后才记image loaded；过期完成沿原job围栏拒绝；自定义Operation由原host实际派发。Resource Ready、GPU预览、toolkit可打开与thumbnail loaded分别记录，当前activation/catalog epoch/resource sequence/source_hash/admission/decode均unknown/null。

本轮只更新四本域文档与最小查询/能力证据，生产ZUI/Rust、五公共scroll路径、Shared、原六events/routes未改；RT-S08/09和canonical Field gap责任不变。无桌面/编译/缓存生成/源码副本/服务动作，释放0字节；原失败、请求、writer/资源准入和全族passed/accepted=false保持。


## 16 SoundWave真实源内容与原生组件

2026-10-24T16:47:27Z起，在原08 file b4a1e3ce-9e6a-81fe-8008-d69d60ef2172 / page b4a1e3ce-9e6a-81fe-8008-d69d60ef2173实际保存、关闭并由新browser/context重开 **rev76/vern0、1337对象、551flow、18板、4本域main、26copy（5 foreign）**。Shared同次仍rev1/isShared=true、41对象/4main、零colors/typographies，原library link保留。没有公共文件、主题、原产品ZUI、Rust或桌面写入。

### 真实源与UE边界

源为[WOC ui_click.mp3](../../../examples/woc/assets/m8/audio/sfx/ui_click.mp3)，URI res://m8/audio/sfx/ui_click.mp3，13835B/SHA256 98598833b3360cd0ad24c1e409b21434b0b822ecaa0482de9c6342cca13cd242；项目GUID 28be8619-648f-45e4-9513-e02fb909ea9c。沿现存FFmpeg/FFprobe2.2.14在内存解码：44100Hz/mono、24192frames、0.5485714285714286s、512组min/max，未创建PCM、源文件或WAV副本。[源分析](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/source-analysis.json)固定工具/源哈希、命令和退出码。这是源诊断，尚非Symphonia产物、当前Engine SoundAsset或播放位置；跨decoder比对和实际导入仍缺。

UE [SoundWave opener](../../../dev/UnrealEngine/Engine/Source/Editor/AudioEditor/Private/AssetTypeActions/AssetDefinition_SoundWave.cpp)仅在自定义delegate绑定且允许时进入WaveformEditor，否则进入真实FSimpleAssetEditor。[WaveformEditor.cpp](../../../dev/UnrealEngine/Engine/Plugins/Editor/WaveformEditor/Source/WaveformEditor/Private/WaveformEditor.cpp)44–46/66–78/115/1159+/2136–2164固定Properties/Transformations/Display、非零源及enabled device门、Undo注册、Standalone_WaveformEditor_v2的20%左Details/Processing与80%右Waveform。本轮仅实现源内容20/80分区，完整Toolkit、Processing、Dock、命令、dB标尺和Transformations仍缺。[STransformedWaveformViewPanel](../../../dev/UnrealEngine/Engine/Plugins/Editor/WaveformEditor/Source/WaveformEditorWidgets/Private/STransformedWaveformViewPanel.cpp)依据PCM采样，不用泛型CurveKey替代或复制动画Transport。

实际[audio_importer注册](../../../zircon_plugins/audio_importer/runtime/src/plugin.rs)和[Symphonia解码](../../../zircon_plugins/audio_importer/runtime/src/lib.rs)已存在：WAV priority120，MP3/OGG/FLAC/AIF/AIFF codec priority90，走import_symphonia_audio。不能再报告“仅能导WAV/无MP3 backend”。WOC显式selection只见sound client_runtime、未见audio_importer，但**有效Editor registry/admission未观察**，不能推断全局禁用。Engine UUID/catalog revision/imported SoundAsset/playback/session/capability仍null。[16份源指纹与能力合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/source-capability-contract-r14.json)保留提案时刻，不当实际接入回执。

### 原生身份与流式内容

本域SoundWaveSourceContent草稿v1用可编辑vector/text、既有palette/font和原生flex组合，没有另建通用按钮/树/属性行/播放组件。

| 内容 | 实际UUID |
| --- | --- |
| component/main | 6ff61186-040c-804f-8008-d6f2f4d0ed80 / 98a02ddc-755b-4643-b6c4-fae1699875c4 |
| 1280×800 main board | cdb99d7c-c846-4d14-b995-1e1167f79957 |
| 900×620 board/copy | 769d6eae-8613-4e8b-be3f-d885ef8b02d7 / 6ff61186-040c-804f-8008-d6f2fb6297a1 |
| 640×520 board/copy | 4075b090-47ea-4bbc-9cf6-b4e475d7ffd6 / 6ff61186-040c-804f-8008-d6f2fefcbb10 |

Details宽256/180/128、plot1000×672/696×492/488×392。两copy及后代48/48 shapeRef属于原24对象main子树，未detach/复制main。五个时间标签各64逻辑像素，锚定0/.25/.5/.75/1刻度。640的URI按实际10px Roboto测宽分成res://m8/audio/sfx/与ui_click.mp3，宽90.1367/56.1133、可用104；拼接仍等于raw URI，没有缩字或隐藏Details。[身份与覆盖](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/native-identities-r19.json)记录自然path边界、本地copy数据几何和URI显示覆盖。

本地plugin-data只存源分析/业务提案，**不是official source codec、公共props/slot或可导出ZUI声明**。独立plugins://sound/editor/sound_wave.zui仍是RT-AUDIO01文档提案、磁盘不存在；原audio_source.drawer是组件inspector、mixer_console是Mixer，不冒充SoundWave文档。正式源资产/绑定/事件及Toolkit/registry归Parent接续，原四ZUI/四Audio Rust保持。

### 失败、保存与逐图复核

r01/r02的CLJS arity/schema、r06缺最终JSON的helper失败、r07/r09符号转义失败均保留。r04过早读到57，随后原27事件已实际提交58，按原ACK与读回对账、未再次重建。r13误将business metadata内层key写为keyword导致500、状态停66，改用原生string key后另存67。r14保留width已改而派生几何未更新和无变化等待超时；r15/r16保留窄图失真及时间锚偏差。

实际路径被强设为整个plot bbox，main同步按参考bbox再次缩放content；r17沿原native结构改为content自然bbox，先完成main再核copy。没有改公共renderer/library算法或放宽source guard。最终6条保存path均从原512bins独立重算，max坐标差0.00024411逻辑像素、时间锚误差0；这只证明源向量保存一致，**不是UE/EXE≤1px或任意后续main/style传播/reset通过**，本地copy数据覆盖仍是限制。

[严格校核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/validation.json)逐项匹配19对file/session/完整changes的lagged58…76 ACK，并保留一次HTTP500。数组顺序/所有字段保持，只规范化Transit map键序；首次guard误判同响应中的先前rev61和键序，失败原因保留，不用顶层revn推断ACK。旧1261非根对象、3组件定义、旧root关系/顺序保持，仅新增75对象及3板引用。[fresh r20](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/fresh-reopen-r20.json)全部1337对象/4组件定义相等、原raw响应一致、own writes=0、browser native exit0。

已实际查看最终三个full和三个crop：[1280图](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/soundwave-1280-r20-crop.png)、[900图](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/soundwave-900-r20-crop.png)、[640图](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/soundwave-640-r20-crop.png)，[逐图SHA/尺寸/观察](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/individual-review.json)完整保存。canvas约70.88%/98.36%/133.89%，含Penpot chrome/选择框，非official export/1×/DPI/EXE。旧900目录尾裁切、640选中不可达、modal/focus/scroll/Column等不由本轮关闭。

### 精确差异与公共依赖

[UE WaveformEditorWidgetsSettings](../../../dev/UnrealEngine/Engine/Plugins/Editor/WaveformEditor/Source/WaveformEditorWidgets/Private/WaveformEditorWidgetsSettings.cpp)规定White waveform/thickness1、背景linear(.02,.02,.02,1)、Black major/minor(alpha.5)、白.9刻度文字/10字号及dB ruler。草稿现有C0波形/303030网格/101010背景与源不同，**RT-S13-C01交Parent canonical role/style与source闭包**，不另造palette或把linear值误当sRGB字节。Processing/Toolbar、菜单停靠、采样选择/缩放、中文长名、变换/history/reimport/save及实际字体行框仍open。

RT-S13-P01沿原asset registry/importer/SoundAsset给当前project/catalog/UUID/revision、有效admission、只读采样snapshot与stale-generation拒绝；RT-S13-P02直接消费动画RT-S08/09真实per-playback session/capability/位置，六events/routes保持，Stop/Seek公开参数/slots仍缺，不据null造播放控件或停止共享设备。RT-S13-Z01由原public owner交official source/guard/normal producer与文档host，未授予公共写权。[消费边界](08-resource-tools-shared-contracts.md#rt-s13soundwave源内容与公共消费边界)列精确责任。

本轮没有Rust变化或编译；原Audio request的build_scope_policy与原Jenkins owner排队ACK保留，r2健康不是本域测试通过。official ZUI/export/load、样式修改→产品截图变化、provider/播放、正常EXE/144DPI/状态矩阵/双端逐图均pending，**passed=false、accepted=false**。必要失败证据保留，整个窗口族未完成。


### 本轮管理入口与投递事实

本轮最后管理入口实查为 2026-10-24T17:01:30.283472+00:00：实际installed已选中 **d2873ff353ea93d6b76f7d7caa12878761146e326070fd4c4526b73f3dc23112**，11/11 capsule/client和197/197 immutable driver匹配，原Audio identity/driver/runtime保持。只运行该真实pinned entry的纯parse_cargo，原精确Sound recipe仍返回 **build_scope_policy**；当前增加particles，仍无sound editor/mixer_authoring_。没有重放QueryOnly、提交、source capture或Cargo，[实际只读回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/jenkins-scope-refresh.json)保留。原Jenkins owner的新事实消息ACK **01a16edc-be0a-7843-82ad-2aab36e5f432**，仅排队、无已读/准入；原消息17:06Z文案笔误已用实际17:01:30证据[明确校正](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/jenkins-owner-time-correction.json)，校正ACK **01a16edd-85ec-78d0-ab8b-afff4782c15f**。原文和历史2790/QueryOnly结果均不覆盖。

2026-10-24T16:59:15Z向Parent官方queue投递本轮新事实，native exit1/code-32600，因100 submissions上限拒绝；[原消息与失败](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/parent-notification.json)保留，没有Parent ACK/送达/已读证明，不清队列或重投。最新记录可从本域四计划读回。自有profile当前活动进程0、20份已记录browser exit0；r06原终态未知，另发现Chrome旧unpacker20文件131212B归属未证，均[保留路径/哈希](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-source-design-01/lifecycle.json)，不以目录或年龄删除。本轮释放0B，必要设计/保存/重开/失败证据保留，cleanup和全族验收未关闭。

## 17 已安装插件Open与typed资产合同

已接续 [Parent followup-04](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/asset-tools-followup-04.json)。原资源 file/page为 **rev76/1337对象/4业务main/26copy（21本域、5 Shared）**；Shared rev1/isShared=true/41对象/4main，colors/typographies为0。Parent核814 source refs全部有效，此轮没有重复独立验证814项。四个 canonical 记录的 **version1 是记录schema版本**，不是组件语义版本或 sourceHash。完整component/main UUID、sourcePath及实际源码SHA已记[收件](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-capability-typed-02/parent-receipt.json)。Property.value仍required/single/linear，不能用Label代替尚未完整发布的7类合法输入。

### 原插件Open的实际结果

17:13:27.281Z～17:13:38.098Z，隔离headless原账户经Main menu→Plugins→Plugins manager→**既有条目Open**，实际打开19002的ZUI Asset Bridge；pluginId **cce74285-44c0-8011-8008-d6ea3345bdbe**。manifest 278B SHA dbcb22b4ab92d61c43f40e3a1ffb035df8d22285151e38d2c7571ee85e436ac5，bundle 485232B SHA 442bed70a38d28c86a702002658c8d77598ac6205fe325b1696aedbd27b7b47d，均匹配Parent原host。Open前逐字段确认已登记manifest与HTTP manifest一致，避免Open自动更新manifest的分支。没有Install/Allow、新host或服务操作。

[原始回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-capability-typed-02/plugin-open-readonly-r01.json)证明83条registry、资源file、Shared file前后hash全部相同，未遇到需guard拦截的写请求。实际iframe显示 **No ZUI asset selected / Ready**，Import可用、Export Selected及Canvas PNG禁用，没有点击导入/导出。browser PID28172 native exit0，未占共享桌面。[完整原图](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-capability-typed-02/existing-plugin-open-r01-full.png)、[插件裁切](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-capability-typed-02/existing-plugin-open-r01-crop.png)及[逐图记录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-capability-typed-02/individual-review.json)只证明既有插件可Open，不证明source识别、官方导出、Engine加载或实际DPI。

原real-session runner未运行：无条件server.listen(0)/Install/Allow副作用不会因WORKSPACE_URL消失。public owner仍需沿原host/entry和四个原owner交source-closure、字体/图标/媒体、codec guard及正常producer。四原ZUI的IDs/imports/events/bindings未动，业务metadata不是正式codec。r00 stdin helper在browser/API启动前括号语法失败退出1，证据保留；修正后Node语法检查通过，再以r01执行。

### 精确源码能力与缺口

[32份源码指纹、8项发现及符号行号](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-capability-typed-02/source-capability.json)覆盖SoundAsset/playback/graph/validator、持久引用/registry、CurveModel、DataTable反馈与UE模型。此次没有新增Rust实现或执行测试。

| 发现 | 资源消费边界与责任 |
| --- | --- |
| 位置来源丢失 | [status实现](../../../zircon_plugins/sound/runtime/src/service_types/playback_status.rs)对Kira位置取.ok()后回落active cursor，公共DTO不保留来源分支。当前只能称manager报告的位置，来源unknown；不称设备实测，不用timer/FPS。RT-TYPE01-P01交Parent原provider |
| 完成/停止 | 同实现drain排空共享finished队列；[stop](../../../zircon_plugins/sound/runtime/src/service_types/playback.rs)须真实control/stop成功才移除指定playback并发Stopped。原Parent/provider按真实session/playback分发；资源UI不能自行drain，失败不作已停止回执 |
| Seek精度 | [seek](../../../zircon_plugins/sound/runtime/src/service_types/playback_controls/seek.rs)接f32秒，乘sample-rate再round/clamp原slice并请求Kira。integer源frame、请求、实际回读分开；不承诺任意长度sample-exact或立即到达 |
| Clip生命周期 | [SoundPlaybackManager](../../../zircon_runtime/src/core/framework/sound/manager/playback.rs)无unload_clip。关闭经原manager停止本实例拥有的playback，不声称释放PCM/cache，不清全局clip map或停止共享输出 |
| typed入口 | [Browser解析](../../../zircon_editor/src/ui/workbench/project/asset_workspace_state.rs)为catalog.kind→AssetTypeId；[DataAsset](../../../zircon_runtime/src/asset/assets/data.rs)的text/canonical_json不是typed验证。不得按后缀/显示名路由Cue/Class/Submix/Curve，媒体UUID不是Engine UUID |
| 持久引用 | [AssetRef](../../../zircon_runtime_interface/src/project/asset_ref/value.rs)以GUID为registry权威，path_hint辅助迁移、sub精确匹配；[PersistedAssetReference](../../../zircon_runtime_interface/src/project/persisted_asset_reference.rs)区分project与builtin://。复用实际resolver，不另造URI→UUID规则 |
| 曲线表达 | [CurveModel](../../../zircon_editor/src/ui/curve/model.rs)只有整条插值和可选非加权切线，不能完整表达UE逐键mode/weight/default/extrap。typed源文档保留完整字段，无法表达的编辑只读 |
| DataTable写入 | [feedback](../../../zircon_editor/src/ui/retained/dispatch/workbench/feedback/data_production.rs)的validate/save仍返回Not implemented (preview)。controls/actions/sample rows声明不是表格事务；原ZUI继续保留 |

### 七种typed文档提案

[typed-contract.json](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-capability-typed-02/typed-contract.json)是 **未实施提案**，不是公开DTO/props/slot或已注册资产格式/command。assetTypeId、扩展名、实际Engine UUID/catalog admission/provider/session均未取得；没有创建虚假资产。

| 作者语义 | 提案保留的字段与身份 | 当前必须保持的缺口 |
| --- | --- | --- |
| SoundWave | 实际AssetRef/source/importer/options；原PCM的sample-rate/channel/layout/frame-count；处理设置须真实codec/command支持 | FFmpeg包络不是imported SoundAsset/session；破坏性处理、保存/导出/Undo、实际播放缺口保持 |
| SoundCue | 稳定node/slot/edge key、Wave持久引用、validated output、节点schema与未知字段策略 | 中立graph/MixerGraph不是Cue程序；真实lowering/source-map/compile/execute和支持节点集合未交付 |
| SoundClass | parent引用、override/有效值来源、真实成员引用；children按单一权威校验并派生 | track.parent不作继承；Pitch不隐式映射speed；循环/缺失引用、跨文档事务未接通 |
| SoundSubmix | asset UUID↔track真实映射；main/aux/sidechain分型、effect顺序、稳定作者边key | snapshot索引不持久化；保留并行send/重复ID诊断，遵守M1 gate，不把inactive Configure当可播放 |
| CurveFloat | 1个value通道及逐键时间/值/modes/tangents/weights、default/pre/post infinity | typed codec/注册toolkit/history未交付，整条插值选择不能覆盖逐键源语义 |
| CurveVector | 3个x/y/z独立通道与各自keys/default/extrap；稳定通道key | 不合并为共享keys，不借用动画TrackPath/FPS |
| CurveLinearColor | 4个r/g/b/a独立通道、rich字段、实际数值域/HDR | 不强制0..1、不假造alpha键，不以颜色显示替代曲线编辑 |

UE依据：[RichCurve.h](../../../dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Curves/RichCurve.h)逐键Interp/Tangent/Weight Mode和双切线/权重，[RealCurve.h](../../../dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Curves/RealCurve.h)的default与前后外推，以及Float/Vector/LinearColor实际1/3/4个FRichCurve。MAX_flt表示无default的区别保留，空曲线不生成0键。源f32语义保持，不能因view的finite过滤静默改源；mixed/weighted等无法表达的编辑禁用并显示原因。切线重算、碰撞、评估/外推等待typed adapter/validator，未复制共享曲线控件。

### Transport、事务及未完成验证

已直接消费动画[资源合同RT-S08/09](animation-shared-controls.md#资源消费合同-rt-s0809)及[instance-scope r01 diff](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/transport-instance-scope-r01.proposal.diff)，SHA 1e5b90f22b836bfe392ed3080cffa7496d9bf1670d59d5d2ee9b367d2287e984。仍是 **unapplied proposal**，不是公开context/slot或已注册editor.transport.invoke；本域未应用公共文件。沿既有EditorOperation/Record/journal带真实project/doc/view/source/backend/capability与current publication/node binding；generation保持整数（真实0有效），不凭focus猜域，不将UIprojection.document_id当资产文档。原六events/routes保留，Stop/Seek尚未公开，无capture不启用Record。

typed提案将revision检查、单gesture inverse/history、save/reopen、外部drift、durable之后Cancel边界及旧异步结果拒绝交原document owner。runtime rollback/seek不是Editor Undo。关闭先使本view scope失效，晚到回执不得改新文档；原host仅向当前generation有效来源返回焦点。保存durable后Cancel不能声称回滚磁盘，逆向操作是另一次显式校验事务。这些是待实际实现验证的合同，不能据文档关闭。

当前仍优先真实Browser project/catalog/UUID/provider接入；08交精确typed/registry/codec/host提案，未扩大runtime/core写权。RT-S10 source-fixed/static-passed/managed+GUI-pending，selection reveal/HomeEnd/focus-return仍open。原Audio3项/scroll5项未运行，没有换request/新capture/编译。EXE、实际150% DPI、双端并排/差异及≤1逻辑像素逐图验收未完成，**passed/accepted=false**。

本轮新事实于17:28:21Z仅尝试一次官方Parent queue，17:28:22Z native exit1/code-32600，因100 submissions上限拒绝；[确切消息与输出](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-capability-typed-02/parent-notification.json)保留。没有本轮Parent ACK/送达/已读证明，不重投或清队列。四计划结构及28保护源+32核查源+4公共源指纹校核通过，自有进程0；[生命周期](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-capability-typed-02/lifecycle.json)保留未知20文件131212B、释放0B，任务及cleanup未关闭。

## 18 AI黑板消费的资产只读provider提案

本节保留第五次交接时的只读源码盘点。其“未实现/API”描述仅属当时；后续G01授权和实际代码候选见[第23节](#23-rt-airef01-g01原manager只读adapter候选)，不回写旧提案为已验证实现。

已按Parent「AI第五次接续」与AI的[AI-BB01～07](ai-design-and-acceptance.md#黑板引用事务与保存边界)收件。资源08只交实际资产身份、locator/类型/catalog revision、来源可写策略和依赖查询状态；**AI拥有BT→schema持久关联、node/parameter语义引用和provider**，Parent拥有原host、跨文档history/原子提交与保存准入。schema资产UUID、业务ID和运行注册句柄不能混用，保存不热更新活动schema。此轮没有新增索引/schema/history库或修改公共core/catalog/AI实现。

[只读provider提案](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-readonly-provider-01/readonly-provider-proposal.json)及[33份源码指纹与8项发现](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-readonly-provider-01/source-capability.json)可独立审阅。以下字段/状态是提案语义，**不是新发布的DTO/API或已接通provider**；实际live查询次数0。

### 既有能力与准确入口

| 需要的事实 | 既有入口 | 必须保留的边界 |
| --- | --- | --- |
| GUID→当前locator/kind | [EditorAssetIndex::row_by_uuid/runtime_registry](../../../zircon_editor/src/core/asset/index.rs)、[AssetRegistryIndex::entry_by_uuid](../../../zircon_runtime/src/asset/registry/asset_registry_index.rs)；当前catalog的asset(uuid)只作同代际展示 | [AssetRef](../../../zircon_runtime_interface/src/project/asset_ref/value.rs)的GUID权威、sub精确匹配；path_hint不是替代身份。缺GUID即使旧path被另一资产占用也不绑定，保留原UUID/URI/诊断 |
| Runtime/UI观察代际 | [ProjectCatalogInputGeneration](../../../zircon_runtime/src/asset/project/catalog_input_generation.rs)的sequence及resource/references revision；[EditorAssetCatalogGeneration](../../../zircon_editor/src/ui/host/assets/generation.rs)的catalog_revision/publish_epoch | [ProjectActivationIdentity](../../../zircon_runtime/src/asset/project/manager/project_activation_identity.rs)是进程内opaque Arc identity，同根重开也变化。manifest GUID/root、格式号、hash、publish epoch不替代activation或资产revision，不序列化pointer/自造counter |
| 类型/来源策略 | [AssetTypeRegistry::get/generation](../../../zircon_editor/src/core/asset/type_registry/registry.rs)、[AssetTypeDefinition::source_write_policy](../../../zircon_editor/src/core/asset/type_registry/definition.rs)、[AssetSourceAuthority::from_locator](../../../zircon_editor/src/core/asset/source_authority.rs) | 只消费实际注册semantic type，runtime ResourceKind另列；现Browser kind→type不能识别AI schema/BT业务关联。policy仅ProjectOnly+project来源可写，未测OS权限、锁、引用闭包或实际save |
| direct/incoming资产边 | [get_dependencies_by_uuid/get_referencers_by_uuid](../../../zircon_runtime/src/asset/registry/query.rs)、原input record的direct_references | 已存在索引，不另建反向索引/扫字符串。UUID依赖边不带schema/node/parameter语义，也不声明hard/soft/传递闭包或覆盖完整 |
| 原UI详情 | 同一个captured catalog Arc上的details(uuid)；[details builder](../../../zircon_editor/src/ui/host/assets/manager/catalog_generation/details.rs) | details=None不能格式化为零引用。反向投影会跳过catalog无对应行的referencer，不能用可见行数充完整数量；单独再次调用manager.asset_details可能取到另一代，需原owner保证同scope |
| 已打开文档 | [DocumentToolkitSnapshot](../../../zircon_editor/src/core/extension/toolkit/snapshot.rs) generation/descriptor_for_instance及[descriptor](../../../zircon_editor/src/core/extension/toolkit/descriptor.rs) document/history | snapshot无asset UUID/locator/source-policy，须Parent提供实际asset→document关联。ToolkitInstanceId只是受长度约束的实例字符串，不从title/instance猜资产。未打开toolkit不等于资产缺失 |

提案分开报告source policy access与save admission：policy-writable/read-only/unknown/stale带实际原因；物理源存在性、IO权限、document revision/锁、AI引用验证及跨文档提交是否允许各自保持未知，不能默认为true。原[SaveDirtyViewCandidate](../../../zircon_editor/src/core/asset/dirty/save_batch.rs)的writable/references_valid默认true是调用方断言，不是provider验证结果。只读查询不构造candidate、不调用validate_references/save/autosave，也不触发refresh/import/preview生成。

### known、unknown、missing、stale及空结果

| 提案状态 | 可证明的事实 | 空列表/边界规则 |
| --- | --- | --- |
| known | 在明确snapshot scope/fence中观察到精确UUID/locator/类型或具体依赖边 | known只针对该事实；observedRowsCount与完整引用total分开。coverage未知时空行仍不能证明零引用 |
| unknown | 无当前匹配project/index、details未提供、provider/extractor覆盖未声明、semantic type或asset-document关联未取得 | 保留null/原因；不转换为false、missing或complete empty |
| missing | 当前已识别的权威registry scope内没有精确GUID/sub，或原resolver实际报告缺源 | 保留原UUID/URI/sub/诊断；复用旧path或同名schema不能替代。无toolkit不作missing asset |
| stale | activation/input/catalog/publish/type/toolkit/source/document revision与捕获状态不匹配 | 保留旧来源并标stale，经原owner重新查；不能将旧结果清为空而允许写操作 |

现[registry query](../../../zircon_runtime/src/asset/registry/query.rs)对不存在UUID的dependency query也返回空Vec，必须先entry_by_uuid；[工作区投影](../../../zircon_editor/src/ui/workbench/project/asset_workspace_state.rs)把details=None的references/used_by默认成空，此展示不能用作引用闭包证明。原[typed extractor](../../../zircon_runtime/src/asset/registry/dependency_extractors/mod.rs)显式分派受支持模型并对其余返回空，也没有AI语义完整性标记。未知provider、未绑定schema、只读/缺失/stale依赖均不能解释为无引用；不按同名key、schema业务ID、标题、equals_string、事件payload或脚本推导关联。

真实样例沿已有Browser输入：Vampire manifest GUID 837f1b65-bbcf-4b72-af59-1317f512a8fc，Texture资产 **e0cca9ad-973a-4dfd-a306-8116b9930c93 → res://textures/jungle_ground_albedo.png / Texture**。当前磁盘563条持久registry及manifest/原PNG/meta/.zasset共5指纹均保持；原target dependencies=[]是磁盘字段观察。实际activation/runtime input/catalog revision/type registry/doc/provider/coverage未取得，完整total仍null、zeroReferencesProven=false；没有把Texture当AI schema。[真实只读样例](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-readonly-provider-01/real-asset-observation.json)保留原条目，没有调用Runtime或新建任何索引。

### 原owner内的最小接入点与生命周期

[EditorAssetManager::catalog_snapshot](../../../zircon_editor/src/ui/host/assets/api.rs)公开Arc当前缺activation/input seq和coverage；现原manager的[EditorAssetState](../../../zircon_editor/src/ui/host/assets/manager/assets/editor_asset_state.rs)已持project/index/catalog，[sync_from_project](../../../zircon_editor/src/ui/host/assets/manager/project_sync/sync_from_project.rs)沿原gate发布一致pair。其[model_import_projection_record](../../../zircon_editor/src/ui/host/assets/manager/assets/mod.rs)已有publish gate→state read→index lock及activation/input Arc对账先例；这是Model import专用回执，**不能拿来当AI查询或伪造import receipt**。

建议Parent在该原owner接最小只读adapter，复用原锁顺序取得真实scope与原registry边、同一catalog Arc，克隆所需不可变事实后释放锁。type/toolkit快照分别携带generation并由实际host映射关联，不能声称这些独立snapshot天然构成一个原子事务。释放asset manager锁后才调用AI业务provider或其它owner；不在锁内调用save/validate/外部回调。无正确scope则返回unknown/stale，不增加core/catalog存储、helper registry或服务。既有asset change subscription只用于失效/requery，通知本身不证明覆盖或activation匹配；关闭/换project后旧请求失效，不让旧结果写新文档。

| 依赖 | 原owner需要接续的最小内容 | 本域交付状态 |
| --- | --- | --- |
| RT-AIREF01-G01 | 原asset manager scope的只读adapter；actual activation/input/catalog join | Parent后续授权08实现；单API/原owner候选及12项测试定义已落码，见第23节；未编译、未执行、未接真实host |
| RT-AIREF01-G02 | Parent host的真实asset UUID↔semantic type↔document/Toolkit关联及revision | snapshot本身不足；不猜title/instance、不扩大资源到AI语义 |
| RT-AIREF01-G03 | 原importer/index声明资产边覆盖；AI交tree-schema及node/parameter/provider语义引用 | current completeness未知，不新建引用索引或按字符串补齐 |
| RT-AIREF01-G04 | Parent跨文档history/原子提交/保存准入，AI业务inverse与引用验证 | SaveDirtyViews只批量准入/逐文档完成，原source writer只单源；不循环save冒充原子 |

AI-BB02/03/04在真实schema关联、语义引用闭包或Parent原子事务未就绪时继续拒绝写操作；AI-BB05沿实际history/save token/generation与逐文档完成，不把部分成功清全dirty，不热更新运行schema。只读provider不创建schema、注册运行handle或进行rename/type/delete。已列12个未来验收场景（缺GUID旧path复用、sub、无project、未知空边、遗漏referencer、关闭toolkit、同根重开、独立代次、policy/保存边界、AI语义未知、部分保存、回调/旧结果），均 **未执行**。本轮只文档/只读审阅，不需要Rust编译、插件/服务/桌面；原Audio/资源工具责任与全族未完成门保持，passed/accepted=false。


## 19 SoundWave的UE电平标尺与原生同步校正

2026-10-24T17:58:04Z独立headless关闭重开同一08 file/page，当前 **rev80/vern0、1364对象、551flow、18板、4本域main、26copy（21本域+5 Shared）**。原SoundWave component **6ff61186-040c-804f-8008-d6f2f4d0ed80** / main **98a02ddc-755b-4643-b6c4-fae1699875c4**、900/640 copy和三board UUID均保留。新增9个原生可编辑文本，经原main的自动同步进入两copy，共27个；832/832全文件source refs按实际owner file核对有效。没有新建主件或detach实例。[身份与原始引用逐项](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-value-grid-02/source-preservation-r04.json)、[关闭重开/真实路径校核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-value-grid-02/fresh-reopen-r04.json)可复核。

来源为UE [STransformedWaveformViewPanel](../../../dev/UnrealEngine/Engine/Plugins/Editor/WaveformEditor/Source/WaveformEditorWidgets/Private/STransformedWaveformViewPanel.cpp)的SetUpValueGridOverlay（370～401行）、[WaveformEditorWidgetsSettings](../../../dev/UnrealEngine/Engine/Plugins/Editor/WaveformEditor/Source/WaveformEditorWidgets/Private/WaveformEditorWidgetsSettings.cpp)和[SSampledSequenceValueGridOverlay](../../../dev/UnrealEngine/Engine/Plugins/Runtime/AudioWidgets/Source/AudioWidgets/Private/SSampledSequenceValueGridOverlay.cpp)的水平标签定位。MidSplit默认depth3，由[SampledSequenceDrawingUtils](../../../dev/UnrealEngine/Engine/Plugins/Runtime/AudioWidgets/Source/AudioWidgets/Private/SampledSequenceDrawingUtils.cpp)递归产生0、1/8、…、1，共9线；转换20·log10(abs(2·ratio−1))，最大1位小数，中心低于−160输出-inf。标签依次为 **0、-2.5、-6、-12、-inf、-12、-6、-2.5、0**。原接口默认标签与线间距2px，上半含中心在线下、下半在线上；本设计沿原10字号/12px文本框计算，真实Slate测量仍待比较。[六个只读源码指纹与算法](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-value-grid-02/source-grid-plan.json)保留。

原main plot1000×672，900为696×492，640为488×392，按各自真实viewport定位。新水平9线加原5条时间线共28 path commands；3条grid、3条原PCM向量经Penpot原PathData→vec解码，与实际512 min/max bins及标尺投影核对，最大坐标误差见回执（均<0.01逻辑px）。这仅证明源数据投影，不是UE/EXE双端1px验收。原1265个非SoundWave对象和原全部1337 IDs保持，原512包络/5时间标签/源URI及其折行恢复后逐字段相同，公共paint未改。

原生同步缺陷与修复均保留：r01添加和同批main/copy几何更新已HTTP200保存，但检查器错误将PathData二进制当数组；同时main自动同步覆盖copy几何。r02分开校正新标签后检查发现原copy包络被压缩、部分时间标签移动。r03仅通过原native update_shapes恢复**原保存对象**的两copy包络与10时间标签，未创建对象/复制组件/改PCM；最终r04关闭后新browser重开，全部对象与rev80保存相同，0文件写请求，包络和时间标签完全恢复。四次实际save ACK均200，两个失败报告、请求/响应和恢复证据均未覆盖；4个自有Chrome均native exit0。

[六张r04逐图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-value-grid-02/individual-review-r04.json)已实际打开full及crop：1280/900标尺及URI完整，640九标签/时间刻度完整、URI两行。Penpot chrome缩放分别71/98/134%，不是实际DPI。UE默认paint与当前灰色仍记RT-S13-C01，不以标尺坐标修复关闭主题问题；未实现新generic timeline/curve/transport。

本轮实际Shared读回已是 **rev2/isShared=true/41对象/4main/34colors/3typographies**，get-file-libraries同一Shared返回rev2、synced-at仍13:29:37.404Z；rev1零styles属于历史。实际style UUID和其token-source字段已只读落盘[Shared样式观察](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-value-grid-02/shared-style-observation-r04.json)，不从列表存在推断本域已消费、global A/B传播或official codec通过。本轮未编辑Shared/公共主题，83-entry插件registry与Shared文件在各有界操作前后hash保持；公共owner仍接正式source-closure、source props/slots与导出producer。

本轮没有产品ZUI/Rust、播放provider、录音/播放器、桌面、系统DPI、服务或Cargo操作。RT-S14限上述可编辑源内容；完整SoundWave toolbar/真实输入/typed SoundAsset导入和注册、实例播放session/位置、Editor history、official导出→原ZUI加载→EXE，以及真实150% DPI和双端图仍open。原Audio3项/scroll5项未执行，passed/accepted继续false。

本轮新事实于18:01:58Z经官方queue native exit0，ACK **01a16f13-9a82-7a22-b2b9-a1f804f7228b**；[确切消息及原生输出](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-value-grid-02/parent-notification.json)仅证明排队，未证明Parent已读/验收。18:01:57Z[只读插件host字节核对](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-value-grid-02/plugin-host-readonly.json)的manifest278B与bundle485232B均HTTP200、原SHA一致；没有Open/Install/Allow/import/export或续期服务，functional codec仍pending。



## 20 Shared rev2同步事实与字段合同接续

### 两种事实：styles已发布，master字段仍未绑定

Parent最新发布已接。本轮原Shared API读回rev2/isShared=true/41对象/4原main/34colors/3typographies；body **99eb86f9-12ff-5935-96ad-3d1a985e61f0**、caption **c0224e78-f918-526f-8026-a76cd8f347af**、strong **9ac86733-8214-5a04-820a-9074b5c68310**均实际存在。37个style的token-source与当前editor_tokens源hash在上一轮逐项匹配，旧rev1/style0保留历史，不继续作为当前阻塞。

但[Parent followup-05](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/settings-plugins-followup-05.json)的publishedSharedStyles.styleReferencesInMasterShapes所列21行，refId/refFile均null；本轮Shared native Transit也确认21个字段、非空style UUID=0，typography-ref-id仍null，没有绑定的fill-color-ref-id。部分字段来自position-data/测量文本，不能凭key或含null的列表长度证明paint/style绑定。[逐条原回执审计](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/parent-master-style-ref-audit.json)与[真实native读回](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/native-sync-audit-r03.json)保存原路径/值。公共owner接真实UUID+对应style存在+refFile+正常producer/source guard；本域只读Shared，不修改公共main或补造seal。

### 原linked copies同步、几何异常和未执行恢复

原资源08 file/page不变，从rev80执行原Penpot libraries.sync-file四参数(own-file,Shared,nil,nil)，保留两个SearchInput、Button、TreeRow、PropertyEditorRow共5 heads/40节点。r01在写前因没有导出2arity入口停止，r02在写前因脚本文本替换损坏UUID函数名停止，均无update-file；r03改用实际4arity入口后update-file HTTP200，保存到 **rev81/1364对象**，component definitions保持，全部旧ID、5 copy的component/main/所有sourceRef、文字和已有touched flags均保持，没有detach、复制main、reset或改Shared。palette字典可用，但40节点实际非空style UUID仍0，不能称组件默认样式传播通过。

同步触发25个现存对象变化：10个非Shared的Directory name，其x/width/points/selrect变动；另15处为Shared copy几何、SVG PathData和文本测量。没有文字/绑定/事件/原色值变化。非目标目录变动使本轮检查失败，不用增宽、缩字或藏目录关闭旧裁切问题；原因仅确认原native sync触发了重算，尚未完成内部责任追踪。原before80和after81原始响应、update-file请求/ACK完整保留。[25对象恢复清单](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/native-restoration-plan.json)要求当前revision/原hash相符后，通过原native update_shapes返回before80原对象，保留PathData和全部ID；不用手工重画、detach、替换原文件metadata或改公共主题。

r04准备按此恢复，再在原受控Button copy的原Import文本上显式应用已发布body/primary，保留原caption/sourceRef/touched/source guards；这个显式本地样式覆盖与master默认继承分别验证。[受控消费计划](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/consumer-style-trial-plan.json)已记录真实UUID、原copy/main/shapeRef和字体provider风险，未执行。实际在打开原workspace时19001即ERR_CONNECTION_REFUSED，Chrome native exit0，没进入恢复或消费，没新截图、没关闭重开验证。全部4个自有Chrome均已真实exit0；服务未被本域启停/续期，按原Parent/owner排队协调，不能将prepared脚本或旧rev80图算rev81验收。[四次原生操作报告所在目录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/)保留全部失败。

### NumberField与PropertyRow新增源合同的最小消费

本轮实际catalog SHA **4c2f849c533f1f227903018eca860faadd7fa3138fdc265300913e3833ad36e5**。只针对两个变更合同进行了6项静态检查：68原组件保持；NumberField正好10props且默认与[原number_field.zui](../../../zircon_editor/assets/ui/editor/workbench/primitives/inputs/number_field.zui)一致；PropertyRow四slots与[原property_editor_row.zui](../../../zircon_editor/assets/ui/editor/workbench/inputs/property_editor_row.zui)匹配，value仍required/single/linear且不接Label。[逐项读回与9源指纹](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/catalog-contract-readback.json)不冒充Parent的25项/任何Rust或插件测试，也不推出已发布NumberField master。

| 合同 | 资源业务应提供的实际值 | 未就绪时 |
| --- | --- | --- |
| value_text/value | 当前typed字段原始文本、真实值及独立编辑buffer；数值投影不能作序列化权威 | 保留原Field/只读，未知值不代42或0 |
| min/max/step/large_step/precision | 当前字段schema、单位、codec和精确步进能力 | 不套0..100/precision0，不制造无穷sentinel，不套动画FPS或sample-rate |
| editable_text/layout_stepper | 当前generation的真实command/codec/history与可写准入 | source policy可写不等于save允许；缺能力则禁止编辑/步进 |
| value slot | 原accepts允许的一份真实Input实例 | 不塞Label、不用未发布UUID或模板控件冒充业务 |
| description/source | 实际schema说明、来源URI/revision和known/unknown/missing/stale | optional缺则省略；unknown索引不报零引用 |
| reset | 原字段真实default/inherited层和单事务inverse/Editor Undo | 无业务reset则不显示，不把删除Penpot override当产品reset |

[RT-C06提案](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/field-slot-consumption-proposal.json)固定了当前DataTable **WorkbenchExtensionDataTableVersionField** 的原WorkbenchField及version.edit/version.commit两个events；原字面量12不证明typed live schema，不能据catalog扩展就自动换NumberField。原[inspector_fields.rs](../../../zircon_editor/src/ui/retained/ui/data/inspector_fields.rs)保留value_text，但同时parse::<f32>().unwrap_or(0.0)，不能因此保证u64/i64步进或重新序列化精确。单独ComponentLab路径的f64/percent投影也不作DataTable业务codec。数值表示计算示例16777217→16777216、i64/u64最大值f32舍入均已记录，仅为计算，不是Rust运行证据。真实MP3 decodedFrames=24192本身已超过模板max100，仍是只读源分析、未导入SoundAsset，不得clamp为100或启用编辑。

本轮产品Rust/ZUI/catalog/theme及其它族文件改动0，无Cargo/桌面/DPI/EXE/播放/插件安装或新服务。R03真实sync已保存但失败；恢复、实际消费、重开/截图、main默认传播/reset、CJK/官方导出与正常EXE均open，passed/accepted=false。


### r05～08：原对象恢复、真实样式消费与关闭重开

18:21:48Z只读19001/readyz和19002/manifest均HTTP200后，本域沿原授权恢复独立headless工作；没有启停、延长或接管服务。r05以before81原hash为准，通过原native update_shapes返回before80的25个原对象，保存rev82；几何/文字/PathData完全恢复，但原生更新自动多加一条SVG path的geometry-group touched，精确全文件比对因这一条标记失败。[r05差异](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/restoration-diff-r05.json)保留。随后只读核实[Penpot原update-shapes](../../../dev/penpot/frontend/src/app/main/data/workspace/shapes.cljs)的ignore-touched选项，r06仅返回该原path对象并保留原touched状态，保存rev83；未关闭export guard或弱化source检查。

r06继续在原Button copy **5698dd90-efc9-8032-8008-d6dd4c845807** 的原Import文本 **5698dd90-efc9-8032-8008-d6dd4c845808** 上应用已发布body **99eb86f9-12ff-5935-96ad-3d1a985e61f0** 和primary **e022fcac-39df-51eb-8515-4723f9932060**，保存rev84。component **6ddc248b-bd35-80c5-8008-d6c8c01096f3**、main **6ddc248b-bd35-80c5-8008-d6c8bd9b057f**、文本shapeRef **6ddc248b-bd35-80c5-8008-d6c8bdb3e2f8** 均不变，原content-group/text-content-text覆盖及全部plugin-data/source guard保持。段落与run各有typography ID/file、fill ID/file，共8个实际非空字段，全部指向原Shared file；不是把raw元数据或null字段当应用。

这次是受控本地样式覆盖，**不是master默认同步**。Shared 4master中的非空style UUID仍0，原21条refId/refFile为null的校核不变。body实际将该文本custom Roboto/700变为gfont-roboto/400；重开DOM实测Import、Roboto400、13.3333px、fill rgb(192,192,192)，document.fonts.status=loaded，但不推出CJK字形、引擎同字体provider或UE按钮默认正确。保留Shared发布值及原source guard，不重签守卫、不本域改master或强制回填自定义字体；正式反导出仍需public owner正常producer/字体闭包。

r07只读重开已验证保存对象/组件/source/style后因辅助脚本漏定义walk中止，未写文件，Chrome native exit0；原失败未覆盖。r08修正只读遍历后，以新Chrome/context重新加载原页，18:30:27.456Z最终 **rev84/1364对象/551flow/18boards/4local mains/26copies**，832/832 source refs有效，保存关闭重开全部对象和组件定义一致、0 update-file写请求。与before80比较仅上述一个消费文本的content/touched/position-data变化，其余1363对象完全相同；5 foreign heads/40节点的原身份/source refs/文字及旧覆盖均保持。Shared rev2原始响应和83-entry registry前后hash一致。r06两个原生update-file ACK均200，作者和重开图共8张已实际逐张打开。[r06操作](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/native-consume-r06.json)、[r08原数据核验](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/fresh-reopen-r08.json)与[逐图记录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/individual-review-r08.json)区分持久化校核与产品验收。

八图均为1600×1100隔离headless页面中71% Penpot canvas及board crop，不是official PNG exporter、UE/EXE或系统150%DPI。真实Browser板的jungle_ground_albedo.png第二行与Texture类型重叠，新增RT-G29 open；Shared Search内部surface/text宽度缺口仍RT-G19；受控板的Published library rev1保留为原始发布注释，本次真实API为rev2。没有按缩放图宣称<=1逻辑px、全族passed或accepted。

同轮只读核当前Jenkins selected e0635d5d791bd8895cb22b84cbbad59baf20c6e76456b1dc190d42bab6a56faf胶囊，11/11实现hash匹配，driver/runtime和原Audio request/operation/identity hash保持。仅调用原parse_cargo，原zircon_plugin_sound_editor/--lib/mixer_authoring_/defaults-off/no extra features/test profile仍被build_scope_policy拒绝，未QueryOnly、未重投、未捕获或运行Cargo。[当前胶囊实际只读回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/jenkins-selected-scope-readback-02.json)不改变原Audio3项/Parent scroll5项未执行状态。无本域产品Rust/ZUI/catalog/theme修改，官方导出→原ZUI加载→正常EXE、实际provider及DPI/双端验收继续open，passed/accepted=false。


末尾范围对账：本轮结束检查发现Parent-owned catalog全文件hash已从初读4c2f849c…变为 **e4bba09c3ba16d55b90eabb5a821ab3e76eb0606f9afc17e9f656e2d16fe0a99**。原[r08检查失败](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/stage-recovery-checks-r08.json)保留；[r09只读核查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-style-consumption-02/catalog-contract-reconciliation-r09.json)确认本次消费的NumberField10props及所有defaults、PropertyRow4slots/required value与原ZUI及旧合同完全相同，另外8份合同源hash保持。仅允许复用此精确范围的证据，不称整份catalog未改或新公共master已验收；本域没有写catalog。


## 21 Browser类型行的旧测量缓存

RT-G29来自上一轮r06/r08真实PNG，原Vampire Browser板 **a64c5b04-bff7-401d-9f58-e04e4f5c26ec** 的Tile **ff9db784-1073-47f3-b33f-9864612eed7f** 保留AssetTile component **30625d51-64a3-469f-84c5-79e156375f8c**、main **a87b3a71-49e3-432b-9b59-be99063789b2**。原名称 **6f607b81-1754-4409-ad02-9f3551512d4b** 显示完整jungle_ground_albedo.png两行，type **974c6c06-4f6e-4d7b-aec6-423e17dde5f1** 显示Texture。字号12/10及Tile136×184、name128×32、type128×12和所有来源状态均保持。

先核实际源码再选最小修复：[UE AssetViewWidgets.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/AssetViewWidgets.cpp)的EnableNewStyle分支把InlineRename与Class text分开，ClassNameMaxHeight=14；名称区高度依当前thumbnail档位和真实Slate字高决定，不靠缩字藏类型。原[Penpot svg_text.cljs](../../../dev/penpot/frontend/src/app/main/ui/shapes/text/svg_text.cljs)直接使用position-data.y作SVG baseline；原[texts.cljs](../../../dev/penpot/frontend/src/app/main/data/workspace/texts.cljs)的字体加载后同样通过update-shapes dissoc position-data让文本重新测量。[4源hash与判别计划](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-text-metrics-01/source-cause-plan.json)为只读本地依据，不宣称与部署bundle等价。

before84中type model y5116，但缓存baseline5115.4702仍是原较高位置；SVG实际字形测量证实它与名称第二行重叠 **4.9965799逻辑px**，排除了仅凭低倍率图片误判。没有改变宽高、字号、line-height、全文或重新设计行框。通过原native update_shapes只删除原type的position-data，并保留原touched，正常Penpot重新测量后update-file ACK200，保存 **rev85**；全文件逐字段比较，仅这一个type的position-data改变，原内容/字体/geom/shapeRef/slot/main及其余1363对象和4组件定义完全相同。不是给所有copy写一个固定y、局部缩放或另造主件。

18:40:19.956Z第二个Chrome/context关闭后重开原08，全部1364对象/4定义与保存一致，26copy和832/832 source refs有效，Shared消费8字段及Import保持，0 update-file。原83-entry插件registry前后hash一致，两Chrome均native exit0。实际字形间距 **6.5327008逻辑px**，无交叠。前、后及新进程共6张原生full/crop已实际逐张打开，新进程crop与保存后crop字节相同。[原操作与字形测量](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-text-metrics-01/native-metrics-r01.json)、[fresh重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-text-metrics-01/fresh-reopen-r02.json)与[逐图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-text-metrics-01/individual-review-r02.json)保留前后实际状态，不覆盖旧失败图。

该修复只关闭本地1280 Browser板的陈旧测量原因。UE固定14px类名区与当前12px仍有差异，完整字体/动态名称高度/三尺寸实际flow和原产品仍未对齐；不能凭本地glyph gap宣称UE/EXE的<=1px验收。Shared Search的外832/内240/204、900/640旧目录裁切、真实多选/scroll reveal/keyboard/focus-return、provider/typed资产和official source export/loading仍沿原未完成清单。没有改任何产品Rust/ZUI、共享主题/catalog/master/引用索引或服务，没有占用桌面。RT-G29仅为Penpot缓存已修正，产品与全族passed/accepted继续false。


## 22 动画r02与原试听provider的终态关联

已消费动画[资源第四次接续r02](animation-shared-controls.md#资源第四次接续r02)及[机器核对](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/resource-consumer-r02.json)，不重复首次分工/组件/播放器。原SoundWaveSourceContent component **6ff61186-040c-804f-8008-d6f2f4d0ed80** / main **98a02ddc-755b-4643-b6c4-fae1699875c4**为08业务内容，不替代三动画共享件。r02的资源rev76是16:50历史观察；本域最后已验证为18:40:19Z rev85，本轮没有API/设计改写/捕获。

原ui_click.mp3仍13835B/SHA98598833b3360cd0ad24c1e409b21434b0b822ecaa0482de9c6342cca13cd242；复用16:11:22Z ffmpeg诊断44.1kHz/mono/24192frames/0.5485714286秒/512bins，未重解码/写PCM/调用声音服务。asset UUID、catalog admission、runtime position/capability未交。原instance-scope diff SHA1e5b90f22b836bfe392ed3080cffa7496d9bf1670d59d5d2ee9b367d2287e984未应用，当前原25输入24份保持，仅资源合同更新；额外catalog pin的4c2f→e4bba属已登记公共变化，不改r02历史或重签proposal。[收件与源对账](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-provider-handoff-03/incoming-r02-reconciliation.json)记录准确范围。

[playback_status_impl](../../../zircon_plugins/sound/runtime/src/service_types/playback_status.rs)用.ok()丢弃Kira位置错误后fallback local cursor；[SoundPlaybackStatus](../../../zircon_runtime/src/core/framework/sound/playback.rs)没有来源/该读取失败字段。仅此DTO，08来源必须unknown。Parent原producer须在丢弃分支前交可核来源/原失败诊断，关联真实asset/source/playback lease及manager/output authority；requested seconds/desired integer frame与实际round-clamp/readback分开。现f32 Seek返回unit，不证明精确sample或DAC已播放。

[stop_playback_impl](../../../zircon_plugins/sound/runtime/src/service_types/playback.rs)同锁依次检查control、poll完成、Kira stop、移除active、追加Stopped并返回Ok。[Kira stop](../../../zircon_plugins/sound/runtime/src/kira_bridge/manager/playback.rs)移除句柄后调用Tween stop，Ok只证明控制路径。后续UnknownPlayback不是失败，也不能单独证明本请求成功；playback_empty只查active与仍未drain的finished，队列取走后没有持久终态索引，不补造一个。

本轮新增判别：[prepare/reconcile_kira_deactivation](../../../zircon_plugins/sound/runtime/src/engine/state/playback.rs)同样产生Stopped；[stop_output_device_impl](../../../zircon_plugins/sound/runtime/src/service_types/output_device/lifecycle.rs)退役Kira后递增原[private output_generation](../../../zircon_plugins/sound/runtime/src/engine/state/storage.rs)。它从0开始，0有效；不是全局manager唯一ID。Finished只有playback/clip/reason/completion_action/output_track，公开[SoundOutputDeviceStatus](../../../zircon_runtime/src/core/framework/sound/output.rs)也不含该代次。因此Stopped不能一律归因于某资源按钮，不能在drain时给旧事件补当时current generation。原producer/现有owner须保留产生时的真实来源代次或原lease关联，未交则unknown，不新增manager/完成队列/索引。

| 原请求与观测 | 08/原host保留的结果 |
| --- | --- |
| Stop返回Ok且原authority/lease匹配Stopped | 本playback API停播确认；不宣称最终样本已听到或全部输出静音 |
| 只有后续UnknownPlayback | 成败未知，保留原命令返回，由原owner核对，不重投 |
| Stop错误与自然Completed交错 | 分别保留typed error及真实Completed，不改记Stopped |
| 设备退役/重配置发Stopped | 仅终态观测；无原来源关联不作该view Stop成功 |
| 完成队列已被取空 | 回执可见性不足，不等于零结束/仍活动或允许再drain |
| 关闭/换project/source/generation后旧ACK | 原owner留诊断，不改新view、复活资产或抢焦点 |

Parent沿原[EditorHostEventController](../../../zircon_editor/src/ui/host/editor_operation_dispatch.rs)的registry/descriptor/执行入口接线；ControlFailure记日志后返回原typed Err。原[EditorEventRecord](../../../zircon_editor/src/core/editor_event/types.rs)的event_id/sequence/binding_path/operation metadata/result可关联原命令，但序号、资产revision、UI publication及output_generation不能混作一代，事件日志也不是asset inverse或audio终态。新transport operation/caller未交，当前literal检索0，不据proposal/JSON arguments称已注册。

RT-TYPE01-P01/P02/P03维持，新增P04补Stop/终态原来源：Parent原provider供可核来源/失败及产生时authority；Parent原host关联实际命令、asset/session和current publication/node binding，唯一生产owner收集/分发finished。08只消费本实例、关闭自己owned playback，不共享drain、执行completion_action ECS意图、清共享cache或停输出设备。若需公开DTO/producer/slot修改，仍由原owner交真实diff；本域未写Runtime字段/schema或新路由。

opener保留实际project/document/view/instance/control/publication，弹层关闭重新解析；文本/IME/局部Undo/popup先处理，focus不猜域。旧ACK不抢焦点/重开资产，drawer dismiss与tab close分开。六route/canonical六按钮+Fill保持，Clock/Status是历史外wrapper。Standalone Float/Vector/LinearColor可仅用generic Curve，不强制Timeline/Transport/TrackPath/FPS；08持typed channel/key/codec/save/history，不把PCM投CurveKey。

本轮[21份源/10事实](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-provider-handoff-03/source-capability.json)与[原provider/host提案](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-provider-handoff-03/provider-host-contract.json)均非实现或新验证请求。六场景未执行，原Sound3项/Parent scroll5项仍未运行，不从本次静态核查推断Jenkins准入。产品Rust/ZUI/动画与public源/DTO/schema/registry/history写入0，无Penpot/API/图片/解码/桌面/服务/Jenkins query或提交编译。真实音频/输入/official导出/EXE/实际DPI与双端验收继续open，passed/accepted=false。

## 23 RT-AIREF01-G01原manager只读adapter候选

Parent的[AI第六次交接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/ai-followup-06.json)明确把G01分配给08实现，授权仅原manager/assets、单一api查询和必要export。编辑前本次7个产品路径diff为空；先在原[artifacts.json](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/artifacts.json)登记7产品/4文档写集及27项输入hash，保留335项并行状态记录。原四ZUI、Audio4源、Parent5滚动源和runtime/core/records/Cargo/catalog均不在写集，不以此次授权扩写。

### 实际接口与发布边界

[EditorAssetManager::readonly_asset_snapshot](../../../zircon_editor/src/ui/host/assets/api.rs)接受原 `AssetRef`、可选的旧 `EditorAssetReadScope` 和可选的真实类型 `AssetSourceWritePolicy`，返回owned `EditorAssetReadSnapshot`。原Default manager通过[原trait bridge](../../../zircon_editor/src/ui/host/assets/manager/assets/preview_trait_bridge.rs)转到[readonly_query.rs](../../../zircon_editor/src/ui/host/assets/manager/assets/readonly_query.rs)；未提供此能力的自定义manager默认返回Unknown/UnsupportedManager，未增改其它mock或业务语义。类型定义在同owner的[readonly_snapshot.rs](../../../zircon_editor/src/ui/host/assets/manager/assets/readonly_snapshot.rs)，仅经原api/root mod导出。

按原publish_gate→state→index获取：project存在、index activation与真实project相同、index input Arc与project input Arc相同、index registry Arc与project registry Arc相同，才捕获activation clone、真实input Arc和当前catalog Arc。不序列化activation、指针或自造序号。expected scope用activation和两个Arc的真实身份核对；相同root/GUID或相同sequence/catalog_revision/publish_epoch都不足以替代。preview-only新catalog同样使旧scope失效。scope仅是本次只读观察，不持锁、不承诺返回后仍当前，更不是save lease或CAS许可；异步consumer须由Parent实际host核对当前document/publication与关闭状态。

先 `entry_by_uuid(reference.guid())`，再严格比较注册locator.label与reference.sub；不从path_hint、同名资产或父GUID代领子资产。已匹配的同一个catalog Arc仅各取一次asset/details；两者如与registry身份/locator/kind或彼此不一致，返回Stale。缺UI row/details仍是未知投影，精确registry条目可Known，不把缺Toolkit当missing。返回前所有manager/index guards已释放；无回调参数和IO/refresh/import/preview/save调用，不改变change stream、source sync epoch或共享状态。

| 返回/字段 | 已实现的候选语义 | 保留的未知 |
| --- | --- | --- |
| Known | 当前匹配scope内的精确registry条目；可选的同catalog asset/details | ResourceKind不是AI semantic type或Toolkit ready；源存在/bytes/加载未证明 |
| Missing | 当前权威registry无精确GUID，或GUID的sub不匹配；保留requested完整引用及actual注册sub诊断 | 不是缺源IO诊断，不替换旧path指向 |
| Unknown | 不支持该合同、无active project、无index/input | 不当false、missing或完整空集 |
| Stale | project退役，activation/input/registry不符、expected publication过期或catalog投影不符 | 不用空行清理旧引用或启用写操作 |
| dependencies | 原registry outgoing/incoming，逐边Known(entry)/Missing(UUID)；incoming不经UI过滤 | asset_edge_coverage与semantic_coverage均Unknown；空Vec不证明零引用/闭包 |
| source_policy_access | 仅以调用方实际提供的策略和精确locator计算；None保留未知 | 未join type registry generation，未验证IO、锁、AI语义或save admission |
| scope/input/catalog | 真实immutable Arc可复核generation上下文和原input事实 | catalog/publish/格式号不当asset/document revision；原source_digest和投影source_hash不当SHA256 |

Vampire原project GUID **837f1b65-bbcf-4b72-af59-1317f512a8fc**、Data UUID **2f7f0e40-bc1b-4ee0-9a8b-5a1590ed5f31**、`res://data/enemy_behavior_tree.toml`及真实bytes SHA256 **fb2b913c9cc03a5b67be654e1749c24e98b913c3164b228d4999ad2767801b4e**作为只读磁盘输入对账。meta的`7f63a3d20dfab0ed`只作原摘要；没有创建/导入/迁移资产、把builtin TOML冒充AI schema或执行live provider查询。source bytes读取/CAS及真实resource revision仍由生产读取owner交；本query不读源文件补齐。

### 必要回归定义与后续准入

[原owner tests/readonly_query.rs](../../../zircon_editor/src/ui/host/assets/manager/assets/tests/readonly_query.rs)的12项定义已挂入真实模块树，统一filter `readonly_asset_`。下列全部**未编译、未执行**，不把定义或rustfmt解析当Rust通过。

| 场景 | 定义验证的边界 |
| --- | --- |
| 无project | Unknown、保留原引用、不伪造scope |
| 缺GUID而旧hint被占用 | Missing，不绑定占用者 |
| sub精确匹配 | 原GUID和大小写敏感label，诊断不替换 |
| 空边/缺UI投影 | Known registry可无asset/details；两类coverage仍Unknown |
| 未投影referencer/缺依赖UUID | 原incoming不被UI过滤，缺边端点显式Missing |
| 同catalog资产/详情及替换 | 两者来自同captured Arc；同数字的新Arc使旧scope stale |
| 同sequence的不同input Arc | 不用数字代替真实publication |
| 同根重开 | GUID/root不变而activation不同；半发布/旧scope均拒绝 |
| project关闭 | 带旧scope为Stale，未带scope为Unknown |
| index缺失/registry Arc断开 | Unknown与Stale分开，不当精确GUID缺失 |
| catalog与registry不符 | Stale，不交叉拼接 |
| 外部消费/无发布 | 返回后try_lock三层均可取，consumer可重查；change stream/epoch不变，policy-writable不证明源存在 |

未来fixture只在托管测试可执行文件的物理D/E/F根cargo-targets子目录创建，使用真实ProjectManager打开并取得真实activation/input；合成registry/UI内容仅属单测，未作为产品数据。递归删除前解析并核对唯一owned leaf及其parent；此轮未创建fixture或项目锁。动态验证必须沿原Jenkins准入另核package `zircon_editor`、defaults-off、原依赖target-editor-host/test-support、lib及上述filter与test profile；此处未声明或运行新的编译命令，没有create/QueryOnly/锁生成/直接本地构建。

[candidate-contract](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-readonly-adapter-01/candidate-contract.json)、[静态记录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-readonly-adapter-01/static-checks.json)与[生命周期](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-readonly-adapter-01/lifecycle.json)区分源码候选、结构检查和未执行动态门。G01不是live provider/AI写操作完成；G02真实asset→semantic/document/Toolkit，G03生产覆盖和AI语义引用，G04跨文档history/CAS/atomic save仍由原owner接续。无服务/桌面/EXE操作，原失败、原请求和全族pending均保留，passed/accepted=false。

## 24 SoundWave保存响应的逐copy完整性

Parent [followup-05](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/asset-tools-followup-05.json)已经实际打开rev80六张1280/900/640 full/crop，并在18:51/18:55独立读到rev85。其99对象摘要和6条vector基线只证明该时点观察，不倒推全历史PathData已逐字段比过。本节补做明确范围的离线比较，不复述为新在线读回或新增图片验收。

### 比较输入与实际范围

复用原 `soundwave-value-grid-02/fresh-reopen-r04.transit`（rev80，SHA256 a90b87ee649b8e99358719998d7093cd532e7c27cd32dbc6243a759e07a78c09）与 `browser-text-metrics-01/fresh-reopen-r02.transit`（rev85，SHA256 289bf736d36f3e8cdf001ec8c17aa12d204705d2aa37496d5ae9ecf548bb610c）。没有复制原响应、安装依赖、启动host/浏览器或向Penpot发送请求；使用原runtime-dependencies的transit-js@0.8.874读取已有内容。原source树的依赖路径探测MODULE_NOT_FOUND已单列保留，实际复用依赖库原路径/hash登记在manifest。

首轮通用reader把TaggedValue转为调试字符串；[该较窄记录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-copy-integrity-01/offline-copy-comparison-r01-string.json)保留，不能冒充直接原生bytes验证。最终[完整比较](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-copy-integrity-01/offline-copy-comparison.json)显式读取 `penpot/path-data` 的tag和binary rep，比较原bytes与SHA256，不改写/反导出PathData。它是离线诊断读取，不是官方ZUI codec或新的产品组件。

| 实际检查 | rev80→已保存rev85的结果 | 证据限制 |
| --- | --- | --- |
| 原main及两copy的子树 | 99对象，2,890个解码native字段全部相同，新增/移除0 | 只检查这两份真实保存响应；不声称当前在线文件或全历史版本 |
| 六个原生PathData | tag及binary bytes全部相同；三条PCM各28,672B，三条grid各784B | 未重新解码MP3、生成路径或运行official exporter |
| vector关键字段 | 6×16=96项content/transform/inverse/selrect/points/坐标/尺寸/rotation/flip/parent/frame/shapeRef/touched相同 | 不把该相等性当UI运行时或≤1DIP视觉结论 |
| 新增电平标签 | 27项ID/shapeRef/文字/坐标/尺寸与原r03映射完全匹配，全字段及position-data保持 | 不是实际DPI/CJK字体或Engine行框测量 |
| 所有文本对象 | 63项content、position-data存在性/值及全字段相同，含原时间/来源文字 | 不是样式默认传播/reset或产品输入控件通过 |

Parent保存的API raw-response SHA与本域原Transit响应SHA不相同，6个Parent content摘要也未与本次显式tag/二进制表示的canonical JSON摘要相等。两种表示没有被当作同一hash协议，更没有把这些不同摘要转成路径改动或宣称已匹配Parent实时body；本次直接bytes/字段相等性严格在自己的原rev80和原rev85响应之间成立。

### 用真实失败验证可靠基线

[失败对照](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/soundwave-copy-integrity-01/known-failure-detection.json)读取原r01/r02保存响应及原错误。r01局部前后有5个vector字段变化，含2条PCM和3条grid bytes；r02局部前后只有2条grid bytes变化，PCM前后相同。后者不能证明PCM正确：对照原失败前rev76，r02仍保留2条错误PCM。相邻版本无变化会漏掉已污染基线，所以原可靠PCM必须单独保留。

| 原响应 | 三条PCM与可靠rev76 bytes匹配数 | 原状态 |
| --- | --- | --- |
| r01保存后rev78 | 1/3 | 原Grid command count失败；两次HTTP200 ACK不作设计通过 |
| r02保存后rev79 | 1/3 | 原PCM altered失败；一次HTTP200 ACK不作恢复通过 |
| r03恢复并r04重开rev80 | 3/3 | 原恢复证据、源bin/坐标检查保留 |
| 原重开响应rev85 | 3/3 | 仅新增本次离线原生bytes/字段对账，未新查在线服务 |

grid在r03完成了有意的UE标尺调整，因此不要求它等于改版前rev76；其实际坐标/28命令仍复用原r04 source proof，不能用PCM的固定基线掩盖合法grid设计变化。原r01×2、r02×1、r03×1四个保存ACK、错误与恢复记录全部保留，没有重放作者操作或构造新负例夹具。

### 原owner消费与剩余验收

公共owner后续只改master样式时，应逐一比较每个copy的内容、原生tag/bytes、transform/inverse、bounds/points、原时间/来源文字、position-data、sourceRef和local override；同时核可靠来源，不只比较main、对象数、保存ACK或相邻响应。若是合法业务内容修改，应先给出准确允许差异及新的来源证据，不能放宽guard使未解释的PCM/时间标签漂移通过。本节只提供已观察的精确记录，不新增公共guard实现、props/slots或source codec。

Shared现在34 colors/3 typographies已发布；21组master refId/file共42字段仍全null，非空0。Import原文本的8个authoring字段另有2个positionData缓存字段，总10个，属于同一个显式override，不是主件默认绑定或传播/reset。rev85类型缓存及6.53270逻辑px gap保持原记录；没有把它提升为Engine/DPI结论。

G01仍是上轮7文件候选，实际交付ACK为01a16f54-8c9e-7be1-85c6-530295248157（仅排队，未确认已读）；Parent提到的01a16f3c-e761-7a20-90b6-b16e2ec863e2是先前分配ACK，不是新的执行/验收。此轮不重复候选派发，不提交validate/create/QueryOnly，不生成锁或编译。原Audio3/scroll5、真实provider/session、四原source closure、公共codec/family producer、正常EXE/实际150%DPI/双端逐图误差继续open。纯离线检查和文档静态检查另记，不计全族passed/accepted；两者仍false。


## 25 原四owner的source closure与分配页追加前置

本节是RT-S18／RT-G32的源合同接续。[只读源码观察](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-closure-consumer-01/source-readiness.json)区分当前候选与实际执行；原早期固定LevelEditor入口的记录保留为历史，不能再概括全部当前源码，也不能把新增分支当作已交付的运行入口。

### 真实分配页和组件来源

19:44:46Z用原P21认证只读get-file/get-profile/get-file-libraries，本域原file/page仍rev85/vern0、1364对象，响应SHA289bf736…与原重开rev85字节相同。Shared真实链接rev2；原账号83条插件注册保持，19002 manifest仅有一个匹配，pluginId仍cce74285-44c0-8011-8008-d6ea3345bdbe。没有创建账户、文件、页面或插件注册，没有保存、同步、导入、浏览器或桌面操作。[API回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-closure-consumer-01/api-readback.json)保留实际状态。

原页面的shared/dev.zircon.zui namespace存在22个对象，但role=asset且带document的官方source-owned asset计数为0。现有业务main/copy及18板没有因此消失或降为未设计；这个0只对应新append准入的精确predicate。四原owner的source/hash、asset/root/components/imports、control及events已整理到[consumer-inputs](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/source-closure-consumer-01/consumer-inputs.json)。未从缺省bindings字段推断没有宿主绑定，更没有移除源props/slot/repeat。

### 当前公共源码候选

原[real-workbench-input](../../../tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-real-workbench-input.ts)已有prepareRealOwnerInput和validateRealOwnerExportSources；--source-closure已调用前者，不再统一走只允许LevelEditor的prepareRealWorkbenchInput。原contract-test定义显式包含Browser、AssetWindow、Drawer、DataTable四路径并检查asset/root/components/nodes/imports、原source及指纹保持；本域没有执行这些定义、prepare-only或Plugin测试。

原[real-session](../../../tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-real-session.ts)新增existingPlugin及append参数，已有复用原host/注册、避免另起pluginServer的分支。append要求原分配file/page及确切revn/vern、认证storage和当前managed UI build receipt，并拒绝借用LevelEditor case/Inspector edit。原[contract](../../../tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-real-session-contract.mjs)还要求既有官方source asset为0，保存及重开后逐页/对象/组件/样式核原内容保持。资源不改这些公共路径；未把源码阅读或候选声明计作保存/关闭/重开通过。

### 尚需原public owner闭合的精确边界

1. 实际Transit get-profile的permissions是set，原runner decoder返回Set；原[registration verifier](../../../tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-real-plugin-registration.ts)却要求Array.isArray(entry.permissions)。这可能在真正Open前拒绝现存注册，属于静态调用链与真实payload类型的不匹配，未运行helper或实际Open来制造失败。JSON协议虽返回数组，但解码记录字段形态不同；本次JSON与Transit-marker摘要均仅诊断，不作为registryEntrySha256交给runner。原owner需在原规范边界给出统一表示、完整注册指纹和不增权的校验。
2. 原19002 manifest仍278B/SHA dbcb22b4…；实际served bundle已348732B/SHA6ded04494d4126e9540b6b225ff0c5ecd61b9772408d8ac2d2e80f1d6c3701f3，区别于早期485232B/SHA442bed70…版本。本域未获其当前managed build/index/worker/source closure回执，不把旧receipt套到新bundle，不接管/重启服务。
3. consumer-inputs先选择原asset_browser.zui。rev85只作这次观察，不硬编码为未来ready；实际append前须fresh核同file/page及revision，后续每个owner重新准入，不能拿同一85并发导入四份。完整UI export/apply与样式A/B还须原owner/source guards及产品加载回执；LevelEditor专用Inspector edit不借给资源。

RT-G32状态是“公共源码候选与本域消费输入已核／原公共运行交付pending”。原四ZUI、G01七源、Audio四源、Parent滚动五源未改，原12/3/5项Rust定义仍未执行。不提交validate/create/QueryOnly、不生成锁或本地编译，不操作服务/EXE/桌面。官方导出/真实provider/normal producer、EXE及实际150%DPI双端≤1逻辑像素仍open，passed/accepted=false。

## 26 RT-AUDIO01-J01有限入口修复的接续边界

[原owner summary](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/ue-assets-audio-mixer-20261024-01/summary.md)与result的SHA分别为b39453af…、7249102f…；[本域接续核对](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-j01-intake-01/receipt.json)保留完整哈希、14份引用及22项源码/参考输入，均匹配。这里只复用原owner真实证据，没有重新执行管理测试、capability或QueryOnly。

| 边界 | 本次事实 | 后继条件 |
| --- | --- | --- |
| 身份 | ue-assets-audio-mixer-20261024-01 / validate-audio-mixer-source-01，原session及SHA 20c1952b…保持，无新attempt | 自动普通收件须返回原身份的实际server receipt，不能借J01管理修复ID |
| 有限recipe | 原nested zircon_plugins/Cargo.toml，唯一zircon_plugin_sound_editor，defaults off/features=[]，lib/test profile/locked/mixer_authoring_，jobs1/threads1 | 执行前重新声明同一最小范围；不使用全仓或其它filter |
| 传递依赖 | 原owner确认2 owning workspaces、19 path包、38声明边、31元数据输入；含Runtime native-host/target-editor-host/sound-contracts | 这是含optional/dev/target的保守闭包，非Cargo已解析特征图；四业务源稳定不证明全capture稳定 |
| 准入 | 原owner唯一QueryOnly native2/pending/ordinary_entry_unproven；writer/target/cache lock/reservation/fullCaptureEOF全null，captureReady=false | 由原Jenkins owner接原自动上下文和真实准入；不例行QueryOnly、不手工grant/改库、不推断全局历史从未收件 |
| 物理资源 | 原owner所报E OS free 13,054,636,032B，reserve 37,580,963,840B+单job4,294,967,296B，静态差额28,821,295,104B | 这是OS观察/静态预算，无预约；不自动切D、不删除共享cache或启动freeze |
| 结果 | 管理#422 SUCCESS；入口4文件已在原分支commit42bb32d…，integration pending。Audio三测试仅定义，0执行 | 管理提交/335 JUnit/9PS及helper终态均不代替Audio Rust、完整host lifecycle或产品/EXE验收 |

本轮原四ZUI、Audio/G01/Parent scroll产品源码未写入；没有source copy、服务、Penpot、桌面或Cargo操作。完整资源族的provider、官方source closure/export/load、真实音频输出及EXE/150%DPI双端逐图仍open。必要失败和旧sealed bytes原样保留；本轮释放0B，未知Chrome unpacker归属仍未关闭。

## 27 AssetTile的UE类名区14及流式高度

接续RT-G29，UE ContentBrowser/Private/AssetViewWidgets.cpp:2431,2523将ClassNameMaxHeight固定为14。原AssetTile component **30625d51-64a3-469f-84c5-79e156375f8c**、main **a87b3a71-49e3-432b-9b59-be99063789b2** 在原文件更新类名框12→14，root改vertical Hug。实际13主件/实例保留字体、全文、图像、选择和source refs；普通Tile高174，真实两行长名Tile高186。四板覆盖640×520、900×620及两份1280×800，实际SVG name/type gap约6.5327～8.5174逻辑px。

[native repair r02](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-tile-class-height-01/native-repair-r02.json)误在3秒保存debounce完成前读回85而报失败；关闭后的真实保存为86。同步还改写67目录缩进与10个Tile图层名称。先以可信85建立77对象/345字段的恢复清单，再用原typed Transit值恢复到87，没有回滚整个文件或转换opaque PathData。[恢复r05](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-tile-class-height-01/collateral-restore-r05.json)和[独立进程重开r06](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-tile-class-height-01/fresh-reopen-r06.json)核87正文hash一致、0新写请求、Chrome exit0。

[八图逐图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/asset-tile-class-height-01/individual-review.json)分别打开四板full/crop。它们是71/98/134% Penpot canvas及含chrome/边框的诊断图，不是EXE或实际DPI。RT-G29现为“Penpot13 Tile/三尺寸class14和name/type分离已保存重开；source codec/字体/产品/EXE仍待验”。原preflight、root自父遍历错误及仅终止自有Node5260的失败证据保留，没有把失败进程说成正常退出。

## 28 用户要求的真实目录展开箭头

### 原生设计与源身份

UE StarshipCoreStyle.cpp:2253–2256的TreeArrow_Collapsed/Expanded使用16×16 Starship/Common/caret-right.svg及caret-down.svg；SExpanderArrow.cpp通过SButton/SImage选择普通/hover brush。两SVG的viewBox均为-2 -2 16 16，来源hash在[原生parser preflight](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-arrow-vectors-01/native-path-preflight-r02.json)。本次通过真实运行Penpot的SVG parser及PathData构造器生成17命令原生路径，没有用Unicode三角字符或位图替代矢量。

原file **b4a1e3ce-9e6a-81fe-8008-d69d60ef2172** / page **b4a1e3ce-9e6a-81fe-8008-d69d60ef2173** 保存为rev92，正文SHA **61a4b069d804cd12ceda3e0d02fb12716420d94439e484158f2e88af7cfb1f1c**。BrowserContent component **6dbdeee0-4fe3-80ec-8008-d6afd543b430** / main **30578a6c-27a0-80d5-8008-d6a477785097** 保持，没有新增或复制公共main。151行各新增一个16×16透明图标容器和一个真实path，31向下/120向右；原label移除字符前缀后仍保留原ID及文字样式。深度padding、行ID、选中填色和旧832refs保持，70份本域linked rows新增frame/path均指回原main对应新源。实际UUID/source refs/二进制hash见[151项计划](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-arrow-vectors-01/plan.json)。

布局用s档16、gap4、row/no-wrap及原深度padding。Penpot原生flex会逆转非reverse方向的子项vector，故最终children为label/icon而视觉为icon→label，不能仅写x值交付错误flow。[最终原生重开r13](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-arrow-vectors-01/final-reopen-13.json)读取151个原生PathData/17命令/16×16/4px及正确子项顺序，另逐项核当前19个SVG path。关闭后92字节hash相同、0新写请求、Chrome exit0。无遮挡特写使用原生decrease-zoom，约127% canvas，非150%系统DPI。

### 失败、恢复与逐图证据

首次重开89触发70实例错误排布和无关文本缓存/作者图层名变化。原失败及保存ACK保留；按原88 typed对象恢复178无关对象的名称/缓存，修正151行flow顺序，再恢复70实例的图标/文字geometry及原生path。翻译缓存仅移动其全局x/y，保留字体局部x1/x2/y1/y2。最终精确保护又发现一个中文label作者名称被标准化，已单字段恢复为原“v  内容”；可见文字仍为“内容”，作者名称不参与显示。92相对87只151行+151文本变化，1062无关原对象、其它三份component定义及SoundWave99对象保持。[15项字段/bytes检查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-arrow-vectors-01/checks.json)通过；[真实来源核对](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-arrow-vectors-01/shared-source-check-14.json)确认972/972 refs有效。

r04将文字ink边界等同4px布局gap导致错误断言；r06要求固定100个全页DOM path，在最后viewport只挂载90、61个离屏，也保留为失败。最终直接读取151个原生对象布局合同及当前19个可见path，未改官方source/export/native-capture guard。r11仅因未导出的zoom helper失败，0写入、关闭后92保持；随后复用真实公开decrease-zoom完成无遮挡特写。脚本/认证路径/typed decoder错误是准备或只读验证失败，不计作成功保存。

[20图逐图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-arrow-vectors-01/image-review.json)包括九板full/crop及无遮挡树特写两图：三尺寸WOC网格、Vampire长名、中文、Drawer、900/640列表及Picker，全部实际打开。九板截图源为91，92仅恢复一条作者图层名，精确源保护及最终重开独立确认后复用可见画面；未用旧错误图冒充新状态。这些均是浏览器canvas诊断，不是正常Editor原生捕获。

### 原能力和公共接入合同

[RT-S19机器合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-arrow-vectors-01/parent-contract.json)只交确切公共差异。原source_tree_nodes.rs把Browser ZUI的Space/Label原型投影为TreeRow / asset-folder；公共tree_row_glyphs/disclosure.rs按node.expanded选择现存打包SVG，因此不报告全无图标能力，也不新造字符箭头组件。当前tree_disclosure_size仍取gap_l；UE solid caret及命名s档的原owner/组件/source export映射交Parent/public owner，记RT-G33。

真实AssetFolderSnapshot只有folder_id、parent、display_name、recursive_count、depth、selected，未含expanded/has_children；source_tree_row_node也未发布expanded。RT-G34要求原tree state以folder_id和current publication接expand/hit/keyboard/focus，不能从字符或显示名猜状态。本轮31/120是设计状态保持，非实际引擎展开动作通过；不新建tree store或改Parent滚动五源。

20:52:05Z实际Shared已rev5、41对象、4原主件、34 colors/3typographies。Button main **6ddc248b-bd35-80c5-8008-d6c8bd9b057f** 的fill新增color ref **599586e3-022c-500b-ac80-5d9c490139af** / Shared file；全页44字段仅此2字段非空。旧rev2/42字段null留作历史。[当前观察](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-arrow-vectors-01/shared-current-observation-14.json)与真实source refs分开；库列当前rev5，消费者synced-at仍18:07:13Z，未sync、detach、reset或验证样式传播。

RT-G33为“本域真实箭头设计/保存重开完成；公共source映射、官方导出/产品加载pending”。RT-G34、640 selected reveal、900尾部clip、CJK压缩、Shared Search fill、真实provider、Audio/Curve typed业务及原12/3/5项Rust仍open。未新建Jenkins create/validate/QueryOnly，没有Cargo、EXE、服务或桌面/DPI操作；四原ZUI/source hash未变，全族passed/accepted=false。自有Chrome均真实退出，释放0B；必要失败、证据和两份空URL-fetch目录保留，清理未记closed。

## 29 目录展开状态与命中路由的真实接入边界

本轮沿用户要求的真实箭头追查当前生产链，形成[RT-S19-r02机器合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-disclosure-contract-r02.json)。这是新源码证据及精确公共依赖，不是新一轮全量盘点或插件/产品验收。rev92的151个真实SVG及972个sourceRef保持；本轮没有Penpot/API/browser写入或公共Rust修改。

### 仅补expanded字段不足以接通

[AssetWorkspaceState](../../../zircon_editor/src/ui/workbench/project/asset_workspace_state.rs)第63—80行没有展开集合；第689—742行的build_folder_tree/append_folder_branch始终递归输出全部后代。此前“接现有tree state”指复用这个primary owner，不能据此宣称已有可绑定的fold set。原[AssetFolderSnapshot](../../../zircon_editor/src/ui/workbench/snapshot/asset/asset_folder_snapshot.rs)和[source_tree_nodes](../../../zircon_editor/src/ui/layouts/views/asset_browser/source_tree_nodes.rs)也没有真实expanded/has_children出口。

[tree/layout](../../../zircon_editor/src/ui/retained/asset_pointer/tree/layout.rs)只有pane_size/folder_ids，[route](../../../zircon_editor/src/ui/retained/asset_pointer/tree/route.rs)只有Folder/TreeSurface；[shared_pointer/asset_tree](../../../zircon_editor/src/ui/retained/dispatch/shared_pointer/asset_tree.rs)将所有Folder点击送到既有SelectFolder。因此目前点击箭头位置同样是选择目录，没有独立展开意图。不能只让图标转向后宣称子树已收起。

### UE约束改变了选择处理要求

[SExpanderArrow](../../../dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Views/SExpanderArrow.cpp)用独立非focusable按钮在MouseDown处理箭头，Shift走递归展开；无children时Hidden保留槽位。叶子不因recursive_asset_count大于零就可展开。

[SPathView](../../../dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SPathView.cpp)第2278—2303行在收起包含已选子目录的祖先时，把该祖先加入选择，以免空选择导致Content Browser显示全部。故“所有箭头点击都不改selection”也不符合目标：普通展开/无关分支收起不改当前选择，选中后代被收起时须处理祖先选择。第2166—2189行则先展开实际祖先，再选择并请求滚入视野；不能靠每次sync强制reveal而覆盖手动scroll。多选仍需保留既有后续合同，不能把UE的加入选择偷换成任意选择Content根。

### Parent公共实现所需的最小合同

| 边界 | 所需行为及现有接入点 | 本域权限与状态 |
| --- | --- | --- |
| primary树状态 | 扩展既有AssetWorkspaceState/retained owner，按真实project activation及view实例持有展开意图；同一catalog得父子关系、expanded与可见preorder | Parent持有公共路径；不另建Browser store，不把surface_mode字符串当document身份 |
| intent/selection | 原SelectFolder稳定control/event保留；单独区分disclosure目标状态，核folder_id/current publication；在同次状态变化处理选中后代的祖先选择 | 仅提案，未声称发布新command/slot/props；不在异步回调间产生空选择 |
| paint/hit几何 | [tree_disclosure_rect](../../../zircon_editor/src/ui/retained/contract/paint/tree_row_geometry/rows.rs)目前只对paint模块可见；Parent须让命中复用同一canonical几何或其实际发布rect | 不复制坐标公式或从截图建hit map；resize/密度/144DPI同一权威 |
| leaf/输入/focus | 真child-folder事实决定箭头可见和可点击；保留槽位及文本缩进；真实Shift、text/IME/popup优先，防Down/click重复toggle | 无本轮产品输入验收，不按图层字符或资产数推断children |
| scroll及实例 | 继续原AssetSurfacePointerState、bridge和RT-S10 clamp/writeback；只在显式导航/reveal意图展开祖先后重测 | Parent五滚动源及旧request/两次失败保持；不同真实view不得串状态或由旧ACK抢焦点 |

机器合同列RT-TD01—14必要回归定义，覆盖分支/叶子、选中子目录收起、Shift、原事件、重排/旧layout、同根重开、独立view、空数据、三尺寸/144DPI与文档undo边界。定义不等于已执行，不能并入原scroll五项计通过。公共SVG painter已存在，不报告“全无箭头能力”。

G01只读adapter的七个产品文件本轮与原候选hash全部一致；已复核现有十二项scope/空引用/同根重开/锁释放回调定义，未发现需改动的本域缺陷，复用旧同源静态证据，不重复Cargo或另造测试。原Audio3、Scroll5、G01动态执行、正式source导出/加载、正常EXE/DPI及双端逐图仍open，passed/accepted=false。当前Jenkins selector变化的只读接续另见原[scope r2消费记录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/audio-scope-refresh-r2-intake-01.json)，不以parse支持代准入。

## 30 真实目录的叶子箭头可见性保存重开93

在第29节的UE行为依据上，本轮继续修正原Penpot设计，不把14项回归定义当实现。实际读取WOC/Vampire两资产根的34个目录，仅用真实child-folder名单判叶子；不按文件数量、扩展名或Engine类型推断。WOC的druid/knight、env/fonts/licenses等无子目录，audio/models/ui等有子目录。[逐目录记录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-leaf-disclosure-01/directory-membership.json)包含真实路径、child names及其SHA；这是磁盘设计依据，不是活跃Engine UUID/catalog/source admission。

原file b4a1e3ce-9e6a-81fe-8008-d69d60ef2172/page ...2173从92保存为93，正文SHA a987b41a9dfc8316dda375a5b776146acc32b74f493d95a1d2a14b363a21d358。按SExpanderArrow的Hidden而非Collapsed语义，只在85个现有叶子Path上设置hidden=true；透明16×16 icon frame仍留在原flow，gap4/label起点/缩进/选中均保持。151条可编辑Path全部保留：31展开分支向下、35折叠分支向右、85叶子不显示箭头。原4业务main、70个BrowserContent linked rows与972 refs保持，没有新建公共组件或删除目录/缩字。

[原生保存ACK](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-leaf-disclosure-01/save-receipt.json)为HTTP200/85 changes；[独立native关闭重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-leaf-disclosure-01/reopen.json)核151个真实PathData/17 commands、85个native hidden、原16-slot/gap4，并在可见19行树核真实分支SVG与叶子不渲染。关闭后同93/hash，0额外保存，原账户registry相同，Chrome18372退出0。未Open/Install插件、同步Shared更新、启动服务或占桌面。

[15项源保护检查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-leaf-disclosure-01/source-checks.json)确认85个Path只有hidden变化，其余1581对象精确相同；全部PathData字节/transform/bounds、所有row/label/icon frame、字体/选择/sourceRef无变。原生保存把本域BrowserContent component的modified-at由04:39:10更新到05:36:26（UTC+8）；只有该字段变化，其余组件字段及另外3组件定义精确相同。首次过严的包含timestamp全等检查失败已保留，不伪装成通过或泛化放宽guard；无产品修改。28份原四ZUI/G01/公共tree输入实读hash保持。

本轮[14图逐图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-leaf-disclosure-01/individual-review.json)为WOC三尺寸、Vampire1280、原BrowserContent1280、中文copy640的full/crop共12图，以及1280完整目录树full/crop两图，均实际打开。截图保留Penpot chrome/selection，canvas71/98/134或127%不是真实DPI；[树特写](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-leaf-disclosure-01/directory-tree-ue-branches-leaves.png)显示分支箭头及选中knight/空叶子槽。

900目录尾部裁切、640选中knight/props不可见、中文chrome文字压缩、Shared Search内部fill等仍open。RT-S19-r02规定的真实展开/selection/单scroll-owner/paint-hit/旧generation拒绝仍须Parent在原公共owner实现；本轮磁盘child facts和Penpot visibility不能当引擎交互通过。正式原ZUI/source closure/export/load、provider、Jenkins Rust、正常EXE/144DPI/双端≤1px未执行，passed/accepted=false。原失败及旧92图作为历史保留，释放0B；新空browser-temp目录保留待托管清理，cleanup未关闭。

## 31. 中文缓存宽度由原生字体测量重建：保存重开94

640中文Browser linked instance d746513d-7228-4a8f-8fef-9c1f32f8c833所在board为062f92fc-81a7-4d8e-bedd-c437c3762444，本轮从真实rev93开始。[plan](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/chinese-text-metrics-01/plan.json)记录12处中文，其中正常目录“内容”作不变对照，11处chrome/表头/状态缓存异常。原svg_text.cljs直接把position-data.width设为SVG textLength并使用spacingAndGlyphs，过小缓存会压缩字形；该宽度不是文本框宽度，也不是13.333333/10.666667字号的自然宽度。

[原生只读测量](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/chinese-text-metrics-01/native-metrics.json)沿原font ID custom-2ddd793a-5f89-40b0-aec1-fbe879e37a57 / Noto Sans SC / normal-400完成ensure-loaded、FontFace loaded和document.fonts ready；正常“内容”缓存与自然宽度同为26.671875。异常“列表”为9.739990234375对自然26.671875，“资源浏览器”为24.340087890625对自然66.65625。对照区分了当前缓存错误与字体尚未就绪，但不反推历史测量为何产生过小值。

修复只调用原workspace.shapes/update-shapes，在原11个shape移除position-data以触发当前Penpot DOM measurement owner；ignore-touched=true、reg-objects?=false、save-undo?=false。没有手算新width/坐标或改公共字体。原pipeline重测并产生一次HTTP200的[真实保存](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/chinese-text-metrics-01/save-receipt.json)，rev93→94；原request/response保留。11缓存与自然测量精确一致，大小列右对齐等位置也由原DOM产生。

[18项源检查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/chinese-text-metrics-01/source-checks.json)确认只11个shape的position-data变化；其余1655对象、所有content/font/box/style/touched、4业务组件定义、972来源引用、151个PathData及85叶子hidden精确保持。[28源保护](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/chinese-text-metrics-01/production-source-protection.json)覆盖原四ZUI、G01和公共tree等输入，hash全部不变。18是静态/回执检查数，不是Rust测试或official插件运行。

[独立全新Chrome重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/chinese-text-metrics-01/reopen.json)再次加载原字体，11缓存、自然宽度和DOM textLength相同；关闭后同rev94/hash、0额外保存、registry相同，Chrome23344退出0；保存Chrome16884也退出0。原file/page UUID不变，保存原文SHA256为ee14ee3a36d8f772237446474705b8dfdf7d65613d92ef3cbe7fc328f5c0668e。[6张native full/crop逐图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/chinese-text-metrics-01/individual-review.json)覆盖before、after及fresh reopen，均实际打开；after/reopen的crop字节相同。134%是Penpot canvas缩放，不是实际系统DPI。

只读诊断前四次因helper未取得正确native export而中止，均93不变/0保存；一次save preflight未取到三参数update函数也在写入前退出。首次reopen检查错误复用了原plan的旧cache值，实际DOM已正常且94不变；已改为读取当前已保存shape，不改阈值。失败原件原位保留。9个自有native浏览器均有退出0回执；[生命周期](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/chinese-text-metrics-01/lifecycle.json)记录一个0B空临时目录待托管清理，释放0B，cleanup未关闭。

640选中props/knight未reveal、900尾部裁切、Engine CJK字体/实际行框与≤1px、公共Search fill/source codec、真实registry/provider和正式产品导出/load仍open。原RT-S10/19、Audio三项、Scroll五项、G01十二项未新增Rust运行；official插件、正常Engine EXE/真实144DPI及双端比较继续pending，全族passed/accepted=false。

## 32. Browser历史导航真实UE图形与生产合同

按实际EnableNewStyle=true的[ContentBrowserMenus.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/Menus/ContentBrowserMenus.cpp)第252—280行，后退/前进为SActionButton Simple，分别绑定IsBackEnabled/IsForwardEnabled、真实tooltip、BackClicked/ForwardClicked及Icons.CircleArrowLeft/Right。[StarshipCoreStyle.cpp](../../../dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Styling/StarshipCoreStyle.cpp)371—372行指定Icon16x16与两个原始SVG。原[左图](../../../dev/UnrealEngine/Engine/Content/Slate/Starship/Common/circle-arrow-left.svg)和[右图](../../../dev/UnrealEngine/Engine/Content/Slate/Starship/Common/circle-arrow-right.svg)实际可读，均为6条move/line命令的箭头、中心8.00016/7.99998且半径6.66667的圆环、1.5描边及round端点。没有用字符或通用三角形替代这组历史图标。

[plan](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-history-icons-01/plan.json)保留原12个Browser导航文本对象ID并将其变为固定16×16的原生icon frame，父Navigation流式布局/原gap/位置/URI均不变。此角色是紧凑breadcrumb内图标，沿现有s档及UE16brush；不另调17/18等尺寸。每个frame新增一个原生PathData及一个circle，5份linked copies各个子图形回指两个原main图标的真实source部分；总新增24对象/20 refs。本域BrowserContent component/main仍为6dbdeee0-4fe3-80ec-8008-d6afd543b430 / 30578a6c-27a0-80d5-8008-d6a477785097，未创建或复制公共master，仍为4业务main。

[真实Shared来源](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-history-icons-01/shared-source.json)为file b4a1e3ce-9e6a-81fe-8008-d69d607aac12、rev5。24个新增描边显式绑定已发布editor.text.disabled，color ID 451e67f9-8821-56e5-8d91-fe7fb3fb923a、#808080；保持旧字符的灰色状态，不宣称当前CanGoBack/Forward已知。这是本域新primitive的真实stroke style ref，不改Shared master/主题，不证明全局传播、override-reset或Engine样式加载。当前库版本、UUID和token源hash均入原报告，不把schema1当组件语义版本。

[保存回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-history-icons-01/save-receipt.json)为一次HTTP200/36变更，rev94→95；[23源检查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-history-icons-01/source-checks.json)确认只有原12图标的name/type/grow-type/content/position-data/shapes/show-content七字段变化，全部原几何/parent/flow及其余1654对象精确相同。原972 refs保持、新20 refs有效，总992；151目录PathData/85叶子hidden与全部12处中文及缓存精确相同。只有BrowserContent的原生modified-at更新，另三组件定义及其它Browser字段不变。24个stroke的refId/file均实际非空，原四ZUI/G01/common等28源hash保持。原SVG命令经native float32打包的最大坐标误差0.000081953125，仅指这组向量编码，不是窗口布局≤1px验收。

[全新Chrome native重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-history-icons-01/reopen.json)逐一确认12个固定框、12个6命令DataView PathData、12个circle与真实共享stroke refs，6个当前板的DOM中均是矢量且无原字符text；关闭后同rev95/hash，0额外保存、registry不变、Chrome35152退出0。[14图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-history-icons-01/individual-review.json)覆盖Browser1280/900/640、中文640、Drawer1280×520、Picker900与导航特写的full/crop，均实际打开。71/98/134/67%是canvas缩放，不是系统DPI。900 full中的非当前Drawer邻板低细节缩略图仍显示旧字符；切至其当前可编辑板后是新路径。邻板thumbnail/source producer刷新仍open，不拿旧缩略图作为来源通过。

运行合同沿[RT-S21](08-resource-tools-shared-contracts.md#rt-s21browser历史状态能力与原宿主接入)及[精确源码/10项验收定义](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-history-icons-01/runtime-history-contract.json)。实际UE FHistoryData是ContentSources(paths/collections)+selected virtual paths，不能自行扩称已保存SearchQuery/view_mode；GoBack/Forward先更新当前数据再移动游标/apply，AddHistoryData在回退后清除future，最大300。SContentBrowser恢复Path/Favorites/Collections/AssetView并UpdatePath，SAssetView走PendingSyncItems且bPendingFocusOnSync=true；ThumbMouseButton/2也走真实history。

当前primary AssetWorkspaceState无导航history/cursor，AssetCommand/codec/AssetHostEvent/normalization/asset_event执行链及原Browser ZUI无对应意图/控件/事件。现有SelectFolder只调原select_asset_folder，ActivateReference是navigate_to_asset，不是历史Back。现有selection为单Optional UUID，与UE集合仍有差距。Parent须沿原primary owner明确真实project activation和view实例隔离、当前capability及publication，再接原binding/host/focus/reveal；不在08图标/控件建第二store、不从surface字符串/文件名推实例，不用Editor Undo冒充浏览历史。原121 controls/11 events/routes未改，新映射须走公共原source closure和原资产加载门。

本轮不修改产品ZUI/Rust或其它Editor主件；只关闭Browser这12处历史图标设计差异。官方导出/producer、真实history输入/tooltip/enabled/焦点、640 reveal、900 tail、正常EXE/实际144DPI、双端误差及原Audio3/Scroll5/G01十二项Rust仍未执行。源检查首次stdin括号错误在执行前中止，修正语法后23个断言/阈值不变，失败诊断保留。[生命周期](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/browser-history-icons-01/lifecycle.json)记录自有进程退出0，一个新增0B空临时目录保留待托管清理，释放0B、cleanup未关闭；全族passed/accepted=false。


## 33. DataTableTab内容原生设计与真实业务缺口

本轮延续原file b4a1e3ce-9e6a-81fe-8008-d69d60ef2172/page b4a1e3ce-9e6a-81fe-8008-d69d60ef2173，从95保存至100。新增231原生对象、本域DataTableSourceContent component `dc042d85-e62f-4025-9680-10ee1619c9d7`/main `a7972f31-e63f-4359-8e6d-3e871ebb2d8d`；同一67parts本域主件及两份native copies提供134引用，3份原Shared WorkbenchSearchInput linked copies保持21引用。旧992引用和原4组件定义保持，合计1147来源均存在。没有创建共享TableRow/Field/Dock主件，未改公共文件、主题、原产品ZUI/Rust/IDs/imports/bindings/events。

| 原生board | 尺寸 | 实际内容边界 |
| --- | --- | --- |
| `4451e36a-b68f-43d3-8773-c35f12beebd2` | 1280×800 | x182/y8000；内容x190/y8116/w1264 |
| `e7ba5a22-1ffd-496b-b4cf-fe966d937cbf` | 900×620 | x1622/y8000；内容x1630/y8116/w884 |
| `6e4898c7-2233-4a02-96b7-3b6616a27d02` | 640×520 | x2682/y8000；内容x2690/y8116/w624 |

UE真实[DataTableEditor.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/DataTableEditor/Private/DataTableEditor.cpp:275)的Standalone布局v6是上方DataTable/DataTableDetails栈、下方RowEditor栈，没有显式0.7/0.3比例。本轮只设计DataTableTab内容：search、单选列表、手动列的内容结构与12px外部滚动槽；未用静态空面板表示RowEditor/Dock已完成。1359—1424的SSearchBox change+commit、单选SListView、ConsumeWheelAlways/noOverscroll和外部双向scroll仍待真实输入及状态owner接入；四行无需滚动，槽不是滚动功能证明。

数据只取原[extension_data_table_workspace.zui](../../../zircon_editor/assets/ui/editor/modules/data/extension_data_table_workspace.zui:147)四行Potion_Health/Consumable/+50 HP/Selected、Sword_01/Weapon/12 DPS/Ready、Armor_Heavy/Armor/42 DEF/Ready、Debug_Item/Item/Missing Icon/Warning。标题及说明明确layout fixture/no Engine asset or schema loaded；原假统计128 rows/2 warnings/512 refs未带入。Ready等是原样例文字，不作capability。Engine asset UUID/schema/document revision/catalog admission/write capability均null，媒体/原生component UUID不代资产身份。源[feedback](../../../zircon_editor/src/ui/retained/dispatch/workbench/feedback/data_production.rs:3)的open/validate/save/若干select仍Not implemented (preview)，spec/control声明没有证明typed document/model/persistence；未据此断言整个引擎没有DataAsset。

字体来自UE StarshipStyle DataTableEditor.CellText Regular9pt，即12逻辑像素。原Penpot Roboto首次尚未loaded，Canvas fallback测量被拒绝；显式加载原customRoboto后才取真实测量，列宽17/85/76/77/55，总310。实际消费Shared归档rev5的颜色/body/caption引用并明确本域12px及custom字体覆盖；不宣称Engine字体/CJK通过。选中背景取实际白色8×8 Common/Selection×linear0.075量化为#4d4d4d，文字取GetRowTextColor的FColorList::Orange即#ff7f00。源检查发现初版序号仍primary；[SDataTableListViewRow.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/DataTableEditor/Private/SDataTableListViewRow.cpp:237)确认序号也用GetRowTextColor，故仅3个新增序号对象的fills/content/position-data改色，geometry/文字/sourceRef/组件定义保持，99→100。首轮18项检查中的选中序号断言失败已保留，修正后保持原断言并补3对象改色范围检查，共19项。偶奇行IMAGE brush尚未真实复现，当前Shared surface交替是已记差异；两局部源颜色未新增公共style IDs或主题。

[保存过程](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-source-01/save-receipt.json)保留95→96一次API POST及全232 decoded commands的真实ACK；首次transit.equals误报后离线完整对账，未重放POST。原生初次保存97在900/640暴露反序；实际Penpot flex layout按绘制序逆转child vector，30个新容器改存储顺序至98仍未触发布局。错误调用部署中不存在的update_layout arity3失败保留、源98未变；只读probe核实真实potok.v2.core/data-event :layout/update后由原layout owner重排保存99。期间没有翻转flex方向或手给像素偏移。随后序号精确修正100，五次保存的完整commands/file/session/revision ACK均已核。

[最终独立重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-source-01/selection-number-fix-reopen.json)为100/1921对象/5mains，raw SHA739da0407e1ab7f65391870273313dab8f183138c6af71358feb5f9ea72e5629；关闭前后同hash、0保存、原账户registry不变，Chrome24692真实退出0。当前6张full/crop及前99阶段12图均实际打开，[逐图记录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-source-01/individual-review.json)区分历史序号差异与当前图；71/98/134%为画布缩放、DPR1，不是实际系统DPI。900/640最终全图出现Shared更新通知，未点击；仅证明归档rev5组件消费，不作当前最新库同步结论。Search根1264/884/624而内部surface240/text204仍为原公共缺口。

[19项源检查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-source-01/source-checks.json)通过42输入指纹、原1690对象除根追加外精确、4原组件精确、新231层级/引用/flow/样例和五完整ACK。此为静态及Penpot原生证据，不是官方插件family验证或Rust执行。新[RT-S22合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-source-01/runtime-contract.json)给typed业务/provider/codec/history与5具体公共依赖、RT-DT01—12验收定义，0执行。公共producer/source闭包、实际RowEditor输入/菜单/撤销保存、正常EXE/实际144DPI/双端≤1px和原Audio3/Scroll5/G01十二项均未通过。

全部自有浏览器已退出；r03失败仅证无残留，native退出码未知。空临时目录保留交原Jenkins有界清理，释放0B、cleanup未关闭。各失败、旧raw和图片完整保留，不启停服务、不请求桌面/新Jenkins，也未更改原请求身份。全窗口族passed/accepted=false。

## 34. 单个Shared Button原生同步、覆盖、完整重置与关闭重开104

继续原资源验证板 **fb52acb4-3e9c-47cd-98ef-d6d6f1540b11**，没有新建copy、公共main或主题。原linked root **5698dd90-efc9-8032-8008-d6dd4c845807** / label **5698dd90-efc9-8032-8008-d6dd4c845808**，source file为Shared **b4a1e3ce-9e6a-81fe-8008-d69d607aac12**，component **6ddc248b-bd35-80c5-8008-d6c8c01096f3**，main/label shapeRef分别为 **6ddc248b-bd35-80c5-8008-d6c8bd9b057f** / **6ddc248b-bd35-80c5-8008-d6c8bdb3e2f8**。当前Shared真实rev5、SHA 8f2e63f900792a7eea28d36b2565132ba79a9098fbf2c5c96aadf64abef658e4；本轮前后原件精确相同。

| 原生动作 | 实际保存与行为 | 有限结论 |
| --- | --- | --- |
| 原libraries.sync-file按components/真实Button UUID过滤 | 100→101，仅root fill新增surface.1真实refId/file；label的Import/content/cache/touched精确保持 | 单原消费者同步证明；未点整库Updates，不声称全库更新 |
| 原update-shapes本地fills、ignore-touched=false | 101→102，surface.0 **5edd281a-cc68-597c-a6fa-1234e164adc8** / Shared，#151515且touched=fill-group；Import保留 | 真覆盖，与source区分；不改Shared main |
| 原reset-component | 102→103，surface.1 **599586e3-022c-500b-ac80-5d9c490139af** / #242424；清root/label touched；Import恢复source默认Button | 完整reset有意清除原文字/body覆盖，不能声称文字始终不变 |
| 两处原本域说明的native content更新/重测 | 103→104，只content与position-data改变；字体、几何、样式及source元数据精确保持 | 纠正原rev1说明并标明无Engine command/export/provider；原历史PNG不改写 |

[严格回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-button-native-01/strict-save-reconciliation.json)逐项匹配file/session/完整decoded changes与实际101—104 ACK，四次document save全部对应。native.json的旧监听以includes(update-file)采集4条，其中native-0实际是update-file-library-sync-status（file-id/library-id、update-count=1），必须另列RPC；请求revn可落后、response顶层revn也不代表对应保存，不能按request+1计数。首次离线审计把该RPC当document save产生TypeError，失败原件保留，修正分类后不放宽完整payload核对。

[17项源检查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-button-native-01/source-checks.json)确认1921 IDs不变、只按钮2对象和说明2对象变化、其余1917对象及5业务component记录精确相同。原151目录PathData/85叶子Hidden、中文缓存、历史圆箭头、DataTable231和SoundWave路径标签均通过全对象精确保护；1147 refs保持、31copy heads=23本域+8Shared。原四ZUI/G01/公共tree等42源码输入全部同hash；没有产品Rust/ZUI/catalog或Shared写入。

[独立关闭重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-button-native-01/reopen.json)实读104/SHA 1c93810b73a22ce51db0193c246ef62ddbb883770135b24882b379aacaf03317、0保存，registry SHA6104344366788cb47dc47bdb6d019658b6db6b9a08b4efe0449505ad8c0167f7不变。按钮仍linked，逻辑240×32框/原source字段精确保持；[16图逐图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-button-native-01/individual-review.json)包含sync/override/reset9图、说明2图、重开3图及2张probe诊断。override图实际#151515像素、sync/reset/reopen实际#242424像素可核；这是Penpot DPR1/71%原生浏览器画面，不是Engine/DPI或双端误差。原Property value空及Search内部240/204仍open。

三次只读probe失败原件保留：先等离屏Button、再过早按store page objects发选择/zoom，实际workspace尚未mount，第三次诊断图为空。修正为先等真实native SVG mount，再选原board、核准确Button DOM；不改源、不重启服务或放宽guard。7个自有Chrome均真实退出0、现无匹配进程；两项0B临时目录保留待原Jenkins生命周期，释放0B，cleanup未关闭。

新增RT-G38/RT-S23-P01为精确公共部署缺口：[CORS观察](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/shared-button-native-01/plugin-manifest-cors-observation.json)记录原19002/manifest.json从19001被实际Chromium拒绝“No Access-Control-Allow-Origin”；独立带Origin GET为200/278B/原SHA dbcb22b4ab92d61c43f40e3a1ffb035df8d22285151e38d2c7571ee85e436ac5，但响应无ACAO。交原public owner核对原host/release与支持的origin配置；不新开host/Install、换origin/proxy或关闭浏览器安全。HTTP可达与账户原registry存在不能充作浏览器插件就绪。

RT-S23-P02仍要求原owner白名单/source closure/codec保留source/IDs/imports/bindings/events、拒绝stale或unsupported；P03保留原Search内部stretch与Property required typed value slot缺口。没有新Cargo/Jenkins请求或QueryOnly、服务/EXE/桌面/DPI操作；原RT-S10/19/21/22、Audio3/Scroll5/G01十二项执行及正常EXE/144DPI/双端≤1px继续open，全族passed/accepted=false。

## 35. Static Mesh源内容与原生三尺寸保存重开107

本节是实际源诊断内容和可编辑设计进展；完整Static Mesh窗口/真实预览与作者操作仍未接通。遵循只读UE StaticMeshEditor.cpp第322—371行的Standalone_StaticMeshEditor_Layout_v6：水平系数0.7/0.25归一为14/19与5/19；右侧上下0.7/0.3。SocketManager在上部与extension stack重复同一TabID、Collision闭合，实际Dock归一逻辑留给原宿主核实，不造两份Socket模型。PreviewSceneSettings与Actions/Slate viewport仍按原源码盘点。补充布局映射沿已批准ReactBits [App Shell](https://pro.reactbits.dev/docs/app-ui/app-shell)的区域关系，以UE实际目标为准。

| 实体 | 真实身份与有限状态 |
| --- | --- |
| 原Penpot file/page | b4a1e3ce-9e6a-81fe-8008-d69d60ef2172 / b4a1e3ce-9e6a-81fe-8008-d69d60ef2173；关闭重开107，vern0 |
| 新本域业务组件 | StaticMeshSourceContent；component c2b1f915-4203-4a85-b433-152c4b6f13f5；main 37a2a346-caa1-4820-b1f2-5de79d8f8095；draft，不伪造语义版本/canonical sourceHash |
| 1280×800 board | aaa02205-ef63-4aa4-8e65-bbe70867d66b |
| 900×620 board | cc9924eb-d9c9-4967-8841-95f6b59b0b16 |
| 640×520 board | 0592cd54-eea6-4c9f-9a91-c4d4a557d5e1 |
| Shared原组件 | file b4a1e3ce-9e6a-81fe-8008-d69d607aac12/rev5；Search component 6ddc248b-bd35-80c5-8008-d6c8c3ce892e / main 6ddc248b-bd35-80c5-8008-d6c8c0f9c196；Shared全件/样式未写 |
| 原始产品资产 | 原四ZUI/hash/IDs/imports/bindings/events保持；拟议modules/mesh/extension_static_mesh_workspace.zui未创建、未注册，须原producer/source与Toolkit准入后实施 |

真实源为[crate_wooden.glb](../../../examples/woc/assets/m8/models/props/crate_wooden.glb)，318100B/SHA256 7a5f880c07a60adbc9b50b92557e47a1fa35a7dfa0e3a9fe4fe9f42216d31131；WOC项目GUID28be8619-648f-45e4-9513-e02fb909ea9c不是opaque activation。GLB声明node0 Crate_Wooden→mesh0 Cube.017、2primitives、0skins/0animations；POSITION数量610+816=1426，indices984+3744对应1576个声明triangle。计数不是GPU/LOD统计或唯一顶点数；POSITION为normalized INT16，声明min/max不是实际世界bounds。EXT_meshopt_compression、EXT_texture_webp与KHR_mesh_quantization已在当前importer白名单，不能重复报白名单缺失；实际解码/导入/upload本轮未发生。近邻.zmeta未见，Engine asset UUID/catalog/type verdict/model与mesh revision/PreviewSession全部null，未创建/迁移资产填身份。完整[源分析](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/source-analysis.json)和[11项能力核对](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/source-capability.json)保留原件。

现有[subassets.rs](../../../zircon_plugins/gltf_importer/runtime/src/subassets.rs)第93—143行明确：#Mesh0为ImportedAsset::Model，持两条primitive Mesh reference并列出两材质dependency；#Mesh0/Primitive0、#Mesh0/Primitive1分别为ImportedAsset::Mesh。第468—512行mesh_instance_from_gltf_node保留node的Model及SceneMeshInstanceAsset.primitives中两条mesh/material绑定；顶层material取第一条不替代该vector。ModelPrimitiveAsset本身无material字段，不能仅从Model引用或顶层第一材质恢复两slot。上述#locator由gltf_label_uri生成，是源推导的候选，不是已发布AssetRef/UUID；MI_Trim_Furniture/MI_Trim_Metal只是源名，不判MaterialInstance类型。Parent沿原PreviewSession/provider复用该结构，08不扩runtime/core或建第二模型/渲染器。

新增81对象含三板、一个本域main/两copy及三嵌套Shared Search，18个新增自有flow parent，当前698flow/24boards/36copyheads=25本域+11Shared。3个业务实例各23parts；own copies引用本域main对应part，main内Search7parts再引用Shared原part，形成原生near-instance链。嵌套component-root按实际Penpot container/library逻辑省略；没有detach、平铺refs、复制公共main或新主题。body字体仍真实Roboto13.333333/400和Shared style refs。三尺寸实际native比例已核，未声称任意手动resize已有动态比例约束。

[严格保存回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/strict-save-reconciliation.json)完整核对file/session/decoded changes与实际ACK105—107。105一次82 commands创建内容，106由原:layout/update重排18自有flow并测量；没有倒置方向或手算像素修正。106的640材质标题固定20px容不下两行，和正文重叠；随后仅3个新materials-title以native update-shapes改auto-height/auto，107由原native测量使标题为16/16/32px、正文保留4px间隔。字号/文字/样式/sourceRef不变；6个新增对象改变，其余原件不动。

本轮新暴露的RT-G40是嵌套Search布局：105三root/field对齐，106原native ACK将900/640内部field分别向右移动280/471.57894736842127，相对main delta0。原Shared root/surface均无flow，surface实宽240.00000953674316、text204.00000318180355，3份copy均精确保留源宽；root宽分别315.578947/215.578947/147.157895。900/640 Search因而在板外，native重开仍复现。完整[位置记录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/sourcecopy-placement-observation.json)、source refs与ACK交原public/native producer；near-ref正确本身不能证明最终责任归属或原因。未给实例内部坐标/宽度/touched打补丁、flatten/detach或放宽source guard。

首版900/640 crop误用整个SVG group包围盒，因离框Search把Penpot chrome包含进来，失败图完整保留；后续按实际native frame-background取边界，并同时保留full viewport原PNG，未修图。[独立关闭重开](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/reopen.json)为107/2002objects、raw SHA740986c7dab1d2f8f91bc45f0b99aa6c78bfcbafb8cc712a078e98918f83e22d、0保存/registry SHA6104344366788cb47dc47bdb6d019658b6db6b9a08b4efe0449505ad8c0167f7不变。18张[逐图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/individual-review.json)均真实打开；DPR1/71/98/134%画布缩放不是实际150%DPI或Engine证据。左侧仅标明源声明和无frame，不以黑板/静态模型冒充渲染预览。

[18项source检查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/source-checks.json)通过：原1920非根对象、原5组件定义、42产品/参考pins保持；23个本轮输入22个未变，唯一Parent活跃交接文档变化单列，不冻结他人写入。首次整数240精确比较失败保留，核原Shared真实浮点字段后改成同源字段精确比较，未增容差/改设计/重放保存。heading preparation首次因输入路径错误在浏览器前中止、随后只纠正baseline；所有[失败原件](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/preparation-failures.json)保留。

[RT-S24](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/runtime-contract.json)给5项公共依赖与RT-SM01—12必要验收定义，0执行。重点为实际admitted AssetRef/typed toolkit、原Model与两primitive/material context、Parent PreviewSession和原host/focus/Undo、canonical Search嵌套source、正式新owner闭包；不新建index/store/schema/播放器/preview。按钮未接作者模型不启用，preview dirty不当document dirty，资源/类型未知保持来源状态。原Audio3/Scroll5/G01十二项及新12定义均未新增Rust执行；official插件/export/load、正常EXE/实际144DPI和双端≤1px继续open，全族passed/accepted=false。4个自有Chrome真实exit0、现匹配0；[生命周期](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/lifecycle.json)登记1个0B空临时目录待原Jenkins清理，释放0B，cleanup未关闭。

## 36. 独立Curve字段保真与原生嵌套构造核对

本轮先核实时态：原资源file/page为rev107/SHA740986c7dab1d2f8f91bc45f0b99aa6c78bfcbafb8cc712a078e98918f83e22d，2002objects/6业务main；Shared5原件不变。只读实际动画file b4a1e3ce-9e6a-81fe-8008-d69d60a13f99/page...a13f9a为rev274/isShared=false，Curve Surface component **78492c86-3bb4-47dd-97c6-752d6ff37e3d** / main **599963dc-b91b-4740-926b-d38154aa8f3a** 与另外两动画原main存在。没有将原私有主件复制到08/Shared或伪称已发布，仍沿动画owner合同复用共享表面；独立Curve不装Timeline/Transport/FPS/自走clock。[当前读取](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-source-review-01/readback.json)保留原file/rev/SHA与registry摘要，不含认证材料。

UE实际[CurveAssetEditor.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/CurveAssetEditor/Private/CurveAssetEditor.cpp)第200—242行：Float/Vector普通独立曲线用单Curve tab；LinearColor用Curve0.8/ColorCurveEditor0.2横向分区、隐藏tab well。第299—378行使用ResolveCurveModelDelegate恢复undo/redo模型、SCurveEditorTree承载channels并默认pin至多4条；GradientView与ColorCurveDetails绑定同一UCurveLinearColor owner。因此原曲线树、曲线面与颜色梯度/属性必须共享真实文档，不造第二曲线模型或预览专用状态。普通单栈不因适配限制改成统一右属性栏；补充设计映射仍沿批准[App Shell](https://pro.reactbits.dev/docs/app-ui/app-shell)，UE实际布局优先。

| 持久源字段/行为 | 当前真实能力与必须保留的差异 |
| --- | --- |
| Float/Vector/LinearColor | UE分别1/3/4独立rich channels；stable channel/key ID分离显示名、排序与筛选索引，不能强制每channel共享同一key集合 |
| 每key插值、切线与权重 | RichCurve.h明确InterpMode、TangentMode、TangentWeightMode、Arrive/LeaveTangent和两weight；当前CurveView是每curve Step/Linear/Hermite，CurveKey仅可选斜率，不能无损承接混合segment/weighted数据 |
| 默认值/空态 | RealCurve.h以MAX_flt表示default未设置，HasAnyData另查default或keys；域codec需明确unset，不生成0默认/0key，unknown/missing/unadmitted/empty继续分开 |
| 曲线外推 | PreInfinityExtrap/PostInfinityExtrap独立；Cycle/CycleWithOffset/Oscillate/Linear/Constant/None需按真实能力保存/求值，不能默认为constant丢语义 |
| LinearColor颜色 | 原始/调整后/clamped/HDR求值接口分开；保存AdjustHue/Saturation/Brightness/BrightnessCurve/Vibrance/MinAlpha/MaxAlpha共7属性，不全局把源RGB/alpha夹至0..1 |
| 精度与合法性 | UE rich输入/值为float32；当前CurvePoint为f32，view finite_or_zero只是视图fallback。typed域须声明来源精度，拒绝非有限/越界/不可保真的变换；不将view自动修正当validator |
| 撤销与保存 | CurveModel.apply返回域inverse，不含activation/document/revision/codec。资源供delta/保真codec，Parent原history/CAS/save/close承接真实文档，不新建栈或循环SaveDirtyViews充作跨文件原子 |

现有[DataAsset](../../../zircon_runtime/src/asset/assets/data.rs)保留uri/format/text/canonical_json，可作为真实typed域codec的候选载体，无需先假定必须增加ImportedAsset变体。但TOML/JSON解析不证明曲线schema；源原文、未知字段、实际UUID/sub、source SHA与document CAS须由真实owner提供。AssetTypeId是开放标识，AssetTypeContribution已有runtime_kind/with_toolkit，然而原asset_type_id_for_locator只从catalog ResourceKind推导，Data仍返回data。Parent需交精确同代次subtype解析/注册/host，不给所有Data挂Curve Toolkit或从同名/扩展名/JSON字段猜语义，不导入新资产补ID。检查范围内ImportedAsset无独立Curve变体、catalog无curve条目不等于全局活动插件不存在；本轮未取得活动registry快照。

[RT-S25合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-source-review-01/curve-target-contract.json)明确五项跨域出口：P01原Data载体+真实domain codec/type/subtype；P02动画原CurveModel/Surface的逐key模式/weighted/extrapolation保真或明确unsupported capability；P03原native组件正式发布/source props/slots/codec；P04原DocumentToolkit/history/CAS/save/focus；P05同文档RGBA/梯度/七调整和真实typed输入。新RT-CV01—12只是必要验收定义、0执行，不把公开catalog默认数值或Label当编辑器。当前缺失表示时保留源字段并只读诊断，不画global Hermite曲线冒充逐key源，不丢权重或伪造可编辑手柄。

对RT-G40另做了实际native纯构造核对：调用部署中的app.common.types.container/make-component-instance，force-parent/frame指向现有两个业务板和本域main Details；仅生成临时内存shape，未emit/dispatch/save/render。真实[constructor回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-source-review-01/native-constructor-r02.json)和[53parts比对](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-source-review-01/constructor-comparison.json)确认两份23parts业务copy和7parts原Shared Search按近引用逐一匹配，component/file/root语义与实际原生构造一致。尺寸/layout item flags为本域flow变体；native layout的geometry-group touched与pure构造主动清touched分开，两个magnifier parent的null/false flip差异未证明是位移原因。

900与640内部field X精确等于whole business copy未缩窄时的native nested root X；相对真实flow root偏移280与471.57894736842127。main-left减variant-left分别279.9999999999998与471.5789473684211；差值仅浮点减法舍入，完整记录derived IEEE754 bound，未改source数值/视觉容差/guard。此关联说明内容停在宽版平移位置，尚不证明实际sync/layout哪一层负责；公开原生resize/sync/reopen生命周期仍需owner核查，原240/204 stretch仍open。没有按实例坐标/touched覆盖、flatten refs/detach、换main或放宽canonical source校验。

第一次pure probe误请求部署未导出的encode-str arity1，在constructor前失败；按实际export arity2加nil opts后第二次完成，原失败保留，均0保存/107同hash/registry不变。离线减法错误用strict equality也保留，改记录实际操作数及舍入界；同源字段精确比较未放宽。[15项source检查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-source-review-01/source-checks.json)与[失败记录](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-source-review-01/preparation-failures.json)区分这些诊断、纯构造及正式验证。本轮28源码输入27未变，唯一Parent活跃交接文档变化不冻结；原42产品/参考pins全部保持，无产品Rust/ZUI/共享source/主题写入。

本轮没有新增图片，沿同rev107的18张已实际打开原图复用；没有把纯生成shape或私有Curve主件读取当可编辑消费者、正常Engine或DPI证据。两Chrome35440/39044均真实exit0，当前匹配0，[生命周期](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-source-review-01/lifecycle.json)保留两项0B空临时目录待原Jenkins清理，释放0B/cleanup未关闭。原Audio3/Scroll5/G01十二项及新12定义均未新增Rust执行；official source/export/load/provider/正常EXE/144DPI/双端≤1px仍pending，passed/accepted=false。

## 37. 嵌套Search偏移的原生同步producer复现

本轮接续RT-G40/RT-S24-P03，不新建平行问题或组件。当前原08 file/page仍107，SHA740986c7dab1d2f8f91bc45f0b99aa6c78bfcbafb8cc712a078e98918f83e22d；2002对象/6业务main。Shared仍实际5，SHA8f2e63f900792a7eea28d36b2565132ba79a9098fbf2c5c96aadf64abef658e4，原native library/copy/near refs与registry保持。复用既有UE StaticMeshEditor的0.7/0.25布局目标、既有ReactBits file-manager/app-shell参考和本域原flow；本轮没有改设计/产品/公共source。

证据输入为原[105 API响应](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/mesh-01-after.transit)、[106实际保存请求](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/native-flow-0-request.transit)、[106保存后原件](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-01/native-flow.transit)及当前107。只调用部署的原生geometry/reflow函数，另通过原生filtered sync-file WatchEvent生成器取输出。生成器使用不可变真实file输入；file==library、asset-type=components、asset-id=c2b1f915-4203-4a85-b433-152c4b6f13f5，避免跨file sync-status分支。只观察commit-changes producer生成的不可变commit数据，绝不派发事件，也不执行commit/layout/update生成事件；不改bus、store、共享组件或网络写入口。

| 原板 | 105 field X / pure reflow X | native sync生成X / 实际106、107 X | 相对head偏移 |
| --- | --- | --- | --- |
| 1280×800 main | 1130.421052631579 | main Search的7个parts未生成修改 | 0 |
| 900×620 copy | 2290.421052631579 | 2570.421052631579 | 280 |
| 640×520 copy | 3158.842105263158 | 3630.421052631579 | 471.57894736842127 |

[原生producer回执](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/nested-search-reflow-01/native-sync-probe-r03.json)及[完整生成数据](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/nested-search-reflow-01/native-sync-probe-r03.transit)显示105生成24changes，24/24均按完整change字段深比较匹配实际106请求中的项；其中两copy各6个Shared后代共12项全部匹配。这里比较change内容，不把新生成commit的ID/时间/整份请求bytes冒充原请求。107生成器也有24changes，但不再包含这些已移位Search后代的修改；不能把再次sync当修复。原main Search未受此过滤同步修改。

[4个pure reflow对照](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/nested-search-reflow-01/native-propagation-r03.transit)每例返回69个实际业务part。105普通flow只归一9个frame的width/height/selrect数值，Search field保持与head对齐；当前107整体flow、单business copy与单Search root的reflow均不修复既存偏移。保持严格源字段比较，不放宽浮点、source guard或视觉容差。Modifier record没有Transit writer，成功记录保留真实native Shape输出；String(record)仅null/[object Object]诊断，不声称已解码整个modifier tree。

[源逻辑](../../../third_party/penpot/frontend/src/app/main/data/workspace/libraries.cljs)的watch-component-changes会响应影响main的local commit并启动组件同步；[递归同步](../../../third_party/penpot/common/src/app/common/logic/libraries.cljc)保留outer root-inst/root-main，update-attrs先按root reposition。它与本轮外层宽版平移位置复现一致，但具体哪项canonical flow/嵌套root重定位应改仍需owner最小修复验证，不把代码阅读当完整运行链跟踪。现已证明原生同步producer能复现原保存操作，不再仅归为纯构造猜测。

后续仍由原public owner/Parent沿原Search canonical source/guard/parts完成root/surface/text flow+fill，并在这三个原板和同一main/copies上检验native sync→layout、正反resize、保存关闭重开。若该闭包仍不能处理嵌套head的本地几何，再由owner评估公共producer的root rebasing；本域无公共源码写权，也不先改Penpot参考树。不能新增本域Search主件、detach/flatten近引用、为每个child覆盖x/width/touched或把240px surface放大后直接关闭差异。原14/19与5/19面板比例不改，不能隐藏Search、缩字或裁掉失败区域。

[16项源核查](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/nested-search-reflow-01/source-checks.json)通过：15直接输入14保持，唯一Parent活跃交接文档变化留历史、不冻结；原42产品/参考pins保持。6Chrome35192/34020/4648/34240/34604/20828均真实exit0，当前stage进程0；四次入口/record编码失败原件及原因[完整保留](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/nested-search-reflow-01/preparation-failures.json)。前后原page编码/API SHA、Shared和账户registry一致；观察流完成由原with-pending finalize负责，不冒称读取了全局pending map，不调用reset。

本轮没有设计保存、截图或Rust/plugin执行。复用原107的18张已实际打开图片，不新增画面通过结论；不把只读native producer当official插件导出/原ZUI加载/Editor绘制。原Audio3/Scroll5/G01十二项仍0新增执行，正常EXE/实际144DPI/逐图双端≤1px保持pending。两项0B空临时目录由原Jenkins清理流程接续，实际释放0B、cleanup未关闭；passed/accepted=false。

## RT-S19：可编辑目录箭头与真实展开状态的最小接入（2026-10-25）

Penpot 原 file/page 不变：`b4a1e3ce-9e6a-81fe-8008-d69d60ef2172 / ...2173`。当前实读 rev107；BrowserContent component=`6dbdeee0-4fe3-80ec-8008-d6afd543b430`，main=`30578a6c-27a0-80d5-8008-d6a477785097`。复用旧 rev93 保存/关闭/重开及原生截图；本轮只补当前结构读回，不把旧图说成本轮 EXE 图。

| 状态/操作 | 已有 Penpot 设计 | 本轮原生代码候选 | 验收边界 |
| --- | --- | --- | --- |
| 分支展开/收起 | 16 槽、gap4、UE caret 原生 Path；31 down/35 right | asset-folder-expanded / asset-folder → host expanded；原 SVG loader | 共享 stroke glyph 与 UE filled glyph、实际指标尚未逐图对齐 |
| 叶子目录 | 85 Path hidden，缩进与槽保留 | has_children 来自真实父关系；asset-folder-leaf 不绘制/命中 expander | 目录有资产不等于有子目录 |
| 箭头点击 | 箭头与 label 分离 | 与 painter 共用 physical disclosure rect；typed SetFolderExpanded | 不将箭头点击误作目录选择；drag route 保持 Folder |
| 收起当前目录的祖先 | 保留来源层级与可见选择 | 按 UE SPathView::TreeExpansionChanged 将选择移到被收起分支 | 不伪造多目录选择；资产多选原边界保留 |
| 外部导航/Find | 展示真实项目路径 | 清除真实祖先的 collapsed 状态；原目录/UUID 保持 | 自动滚动 reveal 和 focus-return 仍 open |
| 旧帧回调 | 不复活旧选择 | 现有 workspace projection stamp 比较后再改状态；换 catalog Arc 或树状态使旧 stamp 失效 | 不是 asset/document revision；整 EditorState 生命周期仍须 retained host 守卫 |

源码参考为只读 UE `SPathView.cpp:2278` 的收起选择逻辑、父链 SetItemExpansion 和 `SExpanderArrow.cpp`；StarshipCoreStyle.cpp:2253–2256 实际使用 caret-right/caret-down。共享 painter 当前复用 `toolbar/chevron-right.svg` / `dropdown.svg`，因此“真实 SVG”成立与“UE 同形、<=1px”分开记录，不用字号或文本 `v/>` 替代图标。

当前 1200 refs 中 14 条是原生 nested-instance 的两跳链：资源副本 → 原 StaticMeshSourceContent main 中的 SearchInput 副本 → Shared SearchInput main；不能误判为丢失，也不能仅凭同文件 UUID 存在宣称 source closure。r01 单跳局限及 r02 逐条祖先/main/shapeRef 链均见 `tree-disclosure-runtime-01/penpot-arrow-source-checks-r0*.json`。本轮没有 detach、复制公共 main、修改 Shared 或恢复服务。

实现/测试、实际 #384 中止与 E/test 修订边界见同目录 `validation-r02.json`、`build-root-revision-proposal.json`。原 ZUI source/ID/import/event/binding 未改；正式 source codec/export/load 仍待原 owner 闭包，不把这次 native adapter 当 ZUI 往返完成。


## RT-S25：DataAsset 严格 Curve codec 与借用适配器候选（2026-10-25）

按 Parent 最新并行授权，已在原 `ui/host/assets/standalone_curve/` 落实本域切片，不等待共享 Field、正式导出或 Toolkit 完整后才编码。现有 assets/mod.rs 只增模块和公开类型；原 G01 exports 保持。精确七路径、输入哈希与候选补丁见 `standalone-curve-codec-01/source-inputs.json`、`candidate-review.json`。

`StandaloneCurveCodec` 只接受原 DataAsset 的 JSON text；调用者必须由真实语义准入明确传入 Float、Vector 或 LinearColor，不能按 Data/文件名/同名字符串推断类型。精确 Value、XYZ 或 RGBA 通道集合、稳定原键 ID、输入单位（seconds/unitless）、f32 精度均显式声明；通道次序、独立键集合、中文名、负值/HDR 保留。version=1 只是该域编码版本，不是 asset/document/catalog revision。当前没有新增/导入任何曲线资产或 UUID，也没有把单元值记为真实项目 producer。

原始 JSON 直接 typed decode，重复/未知字段先拒绝；canonical_json 缓存非空时必须与 text 一致。数值检查原始 token，接受 canonical f32 十进制和精确值及其指数/尾零等价拼写，拒绝整数截断、额外小数精度、溢出和下溢，不先经 f64 丢精度再验证。encode_into 先完整解码旧源、验证新源及重解码结果才更新原 DataAsset.text/canonical_json；no-op 保留原文字字节、URI、format 和空缓存。无 I/O、refresh/import/save 或 durable cancel 实现。

`StandaloneCurveAdapter` 直接借用这个 document-owned typed value，消费原 CurveModel/CurveView/CurveKey；批量 expected-key 比较、全量候选校验后一次发布，返回反向差量供原 history 使用。没有持久第二 source/store、history 栈、clock、runtime evaluator 或 registry。键 ID 变更、插入/删除、重复编辑、旧 expected、非有限数或重叠/乱序输入全部失败且不写入。真正 document/activation CAS、view generation、dirty/undo 事务和 SaveCtx 仍由 Parent 原宿主接入，字段级 expected 不冒充这些守卫。

支持 Constant/Linear 和明确联动 User 的 unweighted Cubic；两侧斜率显式且相等。UE RichCurveEditorModel.cpp:566–583 的非 Break 联动逻辑是该限制依据。weighted/auto/break 切线、非 constant extrapolation、未知 precision/字段严格拒绝。codec 能保留显式默认值和逐键混合插值；原 CurveView 没有默认值/逐段模式，adapter 对这些源返回 UnsupportedSurface，保留源不动，不补零/齐键/平化。LinearColor 当前仅保留 RGBA 原值；gradient、七项颜色调整及 raw/adjusted/clamped 评估依旧 open。standalone 不添加 Timeline/Transport/FPS。

13 项内存回归已定义，包含 exact source/no-op、通道与中文/HDR、未知/重复/精度拒绝、空与默认区分、manual tangent、inverse/redo/codec reopen、stale/批量后项失败不改源。没有生成项目 fixture。rustfmt、diff 和七路径 guard 静态通过（0 error/0 warning/0 suppression）；13 项 Rust 尚未执行。新合法独立 codec scope 拟用原 session / ue-assets-curve-codec-20261025-01 / validate-asset-curve-codec-01；显式 Editor lib/test/defaults-off/features[]/filter standalone_curve_/jobs1/threads1/物理 E。现 dev-module-build 无法表达 E/test，使用原正式 local-cargo managed wrapper；parse 与 public transport 核对不是执行/准入。原 tree #384 ABORTED、Audio/Scroll/G01 身份及失败均保留，不由此新 scope 重放或关闭。

仍待：Parent semantic subtype/registry 选择、真实 Curve 文档和原 retained per-view 宿主、原 history/CAS/SaveCtx 写入、原共享 Curve 发布和正式 ZUI source closure/export/load、正常 EXE/三尺寸/实际144DPI及双端逐图≤1px。这里只交产品源码候选和静态证据，passed/accepted=false。


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


### RT-S19 filled-caret 几何与源身份后继（2026-10-25T10:18:35.922261+00:00）

读取当前专属 file `b4a1e3ce-9e6a-81fe-8008-d69d60ef2172` / page `b4a1e3ce-9e6a-81fe-8008-d69d60ef2173` rev107；down 原 Path `7805a8ca-a65a-4cd3-b3c7-b777a7b2c20e`、right `171a25b4-9ddd-4c3b-a790-dc9ea55faf51`，fill 非空/stroke 空。把绝对 M/L/C/Z 平移至 16x16 viewBox 中心，不缩放，六位小数舍入误差 <0.0000005 逻辑像素；这只是现 Penpot 几何的资源编码提案，不是 UE 原 SVG 字节或产品 <=1px 验收。完整输入和坐标身份见 E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/tree-caret-shared-proposal-01/penpot-path-readback.json 及 proposal-review.json。

树专属 controls/tree-caret-down.svg、tree-caret-right.svg 走原 icon_candidates → packaged assets/icons → source-required SVG pixels/tint，不新增 alias/theme/loader。普通 toolbar dropdown/chevron 不变；叶节点和 current folded state 仍走已存在的 owner 接线。现有树状态/paint-hit 证据与原失败身份完整保留，当前 token slot 和实际 EXE 尺寸仍须单列复核。


### RT-SW-SOURCE01 真实 decoded source 的最小内容适配（2026-10-25T10:32:15.166619+00:00）

借用原 SoundAsset，保留同一个 URI、sample_rate_hz、usize frame_count 与原 AudioChannelLayout。frame_count/sample_rate 为精确时长比，不把 f32 duration 当原始计数或运行位置。调用上游 matches_channel_count + is_valid_contract_layout，不另造扬声器规则；合法 discrete 的 positions 仍 unknown。0 rate/count、布局不匹配/非法、残缺交错帧、任意非有限 sample 都拒绝，不截帧、填充、修正或重排原样本。

waveform(channel, NonZeroUsize) 返回借用源的 ExactSize/FusedIterator。每个输出给原 frame 的 start/end、minimum/maximum，按 quotient/remainder 完整分箱；无 frame_count×index 溢出、空箱、重复或丢失。逐声道而非 downmix，反相左右声道不会互相抵消，原超出±1的有限峰值保留；巨大 bin budget 被实际 frame_count 限制且不分配 bins 缓冲。空的已供 decoded source 给已知空；缺 provider 时不得构造空 SoundAsset 代替 unknown。

该代码尚无产品 caller/波形绘制绑定。原 MP324,192frames/44.1k/mono/512bins仍是 ffmpeg 诊断，未冒充本适配实际接收过的已导入 SoundAsset。不推断原压缩格式/码率/位深/循环点，不转 CurveKey、FPS、Transport 或新 clock。图形和播放的真实 EXE/144DPI仍 open。


### RT-S25 minimum matrix 资源后继与 authoring 相等语义（2026-10-25T10:42:26.657620+00:00）

沿原 Source/View/Mutation 矩阵，不扩大格式：source 可保留 mixed/default，原 CurveView 缺表达时 adapter 拒绝；User-linked/unweighted Cubic 仍要求显式 finite 两侧数值相等，Auto/Break/weights/非 Constant extrapolation/unknown 字段继续拒绝。动画合法独立斜率/None 的 private 投影不自动成为资源语义准入；空曲线、None 与显式 default 不补假 key。独立 Curve 无 Timeline/Transport/FPS/clock。

新本域 equality.rs 给原 Key/Channel 实现存储位相等，input/value/arrive/leave/default 的 -0/+0 不再折叠；仍由原 validate 拒绝非有限和非递增输入，±0 不能占两个相同数字时间位置。原 Source/Delta 派生比较自然使用同一规则，原 apply/encode 代码不动：stale full expected、is_noop、candidate publish、codec noop/reopen 和 inverse 一致。数值上 linked 的两个零斜率可保留各自作者符号位；不靠相等反推 mode、不允许缺失/非等的 User Cubic。

6 新定义覆盖 sign-only input/value/slopes 的 apply/inverse/redo，末项 stale sign 全批拒绝，codec sign-only写入/重开，default±0/None与 surface拒绝，负零 exponent 原文本/cache-null no-op，以及单键 mixed 拒绝/完整 channel batch/inverse。定义不是执行；同原 Document/history/CAS 只提交一次 gesture，typed inverse 不证明原始文件bytes undo或 durable save，原 host仍须保留 before/after 与真实revision。完整原始text精确撤销与文本局部undo不得由本类型相等规则替代。


### RT-S25 r03：当前 source / view / mutation 精确边界（2026-10-25T10:58:21.013818+00:00）

本轮在原 standalone_curve 四路径写入共置 [源合同](../../../zircon_editor/src/ui/host/assets/standalone_curve/README.md)、模块文档入口及 4 条纯内存边界回归；不改 adapter/codec/source/equality 的算法、schema 或公开签名。补丁 SHA256 30706a175124d47f2c5aa99c804428b0bb079ce47a1979ee50b22397957983f0（13695B）；2 个已有 raw 前像精确还原，2 个新增路径此前不存在；12 个只读引用 hash 均匹配。原 r02 五源目前 3/5 匹配（mod 文档、tests 新 hook 有明确后继），原 #388 七源仍仅 codec/adapter/error 3/7 匹配，旧输入不回写。

支持矩阵：Float/Vector/LinearColor 要求精确 Value/XYZ/RGBA 通道集；各通道可以采用不同的统一 Constant/Linear/User-linked unweighted Cubic 模式及独立键输入。单通道混合键模式、显式 default（包括 ±0）可无损编码，但原 CurveView 无相应表示，投影明确拒绝。默认 unset 不造键。独立斜率、Auto/Break、weights、非 Constant 外推、f64、unknown fields、gradient 和七项颜色调整仍拒绝，不因动画 private preflight 扩大本域准入。有限负值/HDR/符号零保持。

纠正旧文字的层次：完整 source codec 可以编码合法的 changed key list；“不支持插入/删除/改名”指现有 StandaloneCurveDelta 和产品命令，没有禁止原文档合法整源快照的编码。现 delta 只编辑已存在且 ID 不变的键，完整预检 candidate 后返回 inverse；整源编码能力不产生结构菜单、权限或 history。changed JSON 会重新排版，精确字节撤销必须沿原 Document 保存 before/after DataAsset。

菜单/文本/gesture、insert-delete、gradient、七项颜色调整、真实 subtype/registry、Toolkit、provider、原 history/CAS/SaveCtx、正式源闭包和正常 EXE/DPI 逐项 open；独立 Curve 沿原 CurveModel/CurveView，无 Timeline/Transport/FPS/第二 clock/store/history。

静态证据：rustfmt、scoped diff check、reverse patch check 均 0；未改 checker 的有界诊断在 63.54 秒 exit0，guard errors/warnings/suppressed 均 0。首次 55 秒超时、空 stdout/stderr 和已回收进程原件完整保留；诊断栈落在原 Rust 模块可达性扫描，不改 guard 或抑制规则。所有回归只定义、未执行；无新编译/桌面/服务/捕获。产品 passed/accepted=false。证据目录 E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/curve-lossless-consumption-01/，r03 使用 boundary-r03- 前缀，旧文件不覆盖。


### RT-S25 r04：从不可变 DataAsset 准备键编辑候选（2026-10-25T11:13:51.465286+00:00）

本域已在原 codec 增加 StandaloneCurveCodec::prepare_edit(&DataAsset, expected_kind, StandaloneCurveDelta) -> Result<Option<DataAsset>, StandaloneCurveError>。它复用原 decode → StandaloneCurveAdapter::apply → encode_into，不新增 DTO/store/history/clock、decoder、资产或 UUID，不安装文档、不 dirty、不保存、不 I/O/callback。完整 source/kind/cache、全部 expected key 和最终可投影 candidate 校验后才返回一个替换值；任何后项失败均不返回部分候选，原 DataAsset 在所有路径保持不变。no-op 仅在原 adapter 完整预检后返回 None，不能以 delta.is_noop 绕过 stale/default/mixed 拒绝。

原 apply 返回的 inverse 在本次临时准备中用于确认真正 no-op；此 API 返回完整 DataAsset 候选。原 Document 须保留真实 before/after 源值以获得精确文本撤销，再沿同一 document/view/activation/source revision、一次 permit/retention/history/CAS 提交；typed 字段相等、内存候选或 JSON codec 都不产生语义资格/写入许可。SaveCtx 继续管 dirty/锁/I/O/失败/durable cancel。changed 候选保持原 URI/format，原 codec 重开校验并重建 cache，JSON 排版可以变化。

四路径仅 standalone_curve/codec.rs、tests.rs、README.md、tests/prepared_edit.rs；补丁 SHA256 0ab156ac3488c1c9bce3baf480333cc8f53b157b04af7a148a9d25b94c8746cc（13598B），3 个已有 raw 前像精确还原，1 个新增此前不存在；12 个 readonly 引用匹配。新增 8 项纯内存定义，当前13+6+4+8=31，不是项目 producer 或已执行测试。原 #388 七 pin 当前仅 adapter/error 2/7 同版；r03 四 pin 仅 mod/boundaries 2/4 同版，codec/tests/README 的后继明示，不倒签旧13/19/23定义。

新定义覆盖跨通道 HDR/负值/符号零完整候选、raw spelling/cache-null/缓存存在的 no-op、stale 后项和 stale no-op、非有限/越序/非法 slope/改ID/重复 target、完整 channel 模式切换、不可投影 default/mixed 源、kind/cache/unknown 源拒绝、符号零重开及旧 expected 拒绝。混合插值与显式 default 的 codec 保源/表面拒绝、User-linked/unweighted 子集和 gradient/7调整/结构编辑拒绝边界不变。此 API 尚无实际 per-view caller，菜单、insert-delete、真实 subtype/Toolkit/provider、原 history/保存/focus 和 EXE/DPI 仍分列 open。

本次格式、scoped diff、reverse patch check 已过。guard 第一带周期栈诊断进程 exit0xC0000005，第二标准入口110秒到界，原件均已回收并保留；当前同 hash/patch 的第三标准入口以300秒上限记录实际进程CPU，结果未到，不计 guard 通过。未改 checker/规则/断言或用旧 r03 结果代当前。最终真实结论单列后继验证回执，必要失败不覆盖。

同一真实08 file/page于11:05Z API读回 rev107/vern0/2002 objects，尚无 Curve named shape/设计板；本轮0设计保存。首次 JSON reader 错用 pages-index 的诊断已保留，按实际 pagesIndex 精确读回，未把首次误解析0当空页。没有重复 Install/host/服务恢复/桌面操作，设计共享 Curve/source closure 与产品验收继续 open。


RT-S25 r04 guard 终态接续（2026-10-25T11:15:01.437562+00:00）：同一 checker SHA e2ea2593…、同一补丁和4源，第三标准入口真实 exit0/89.89秒（native CPU48.5625秒），errors/warnings/suppressed=0/0/0，stderr空；自有进程已回收。首次原生异常原因仍未证明、第二110秒超时原件保留，未以更改源/规则/断言换通过。最终静态证据 prepare-r04-final-static.json；31定义仍0已验证Rust，实际执行未知，正式源闭包/真实caller/EXE/DPI继续open，passed/accepted=false。


## TextureSourceAdapter 01 — 原始纹理事实与显式上传预检（2026-10-25）

本轮实际源码仅 `zircon_editor/src/ui/host/assets/mod.rs` 与新 `texture/{mod,source,tests}.rs` 四路径，patch `8990b0fed9c8c04a5be0f16c67fdd04312f64431cdbfcc2cd19d297bf5e6873c`。原 assets root 前像 `a468702f…` 可逐字节反向重建；原 G01、Curve、DataTable、Sound、StaticMesh 导出与并行公共改动保留。未改 Runtime、公共 host、registry、catalog、主题或原 ZUI。

`TextureSourceAdapter::read(&TextureAsset)` 借用真实已加载资产，保留可选作者描述符、URI、完整 payload/metadata/count，单列 Runtime 派生的 `effective_descriptor`、规范化是否改变以及原始描述符的原生 metadata 诊断。描述符缺失为 `None`，与 `Some([])` 无诊断区分；非法作者值继续可检查，不在读取中修复。活动 CPU payload 字节、保留的已初始化字节数、源文件大小及 GPU 显存不混用；容器旁的非活动 rgba 不丢失。

`upload_preflight(support)` 仅将调用方明确提供的 `TextureUploadSupport` 交给现有 Runtime `upload_readiness`，保留其结果；`Ready` 是 CPU 上传方案，不是源资产有效性、真实设备、上传完成或帧可见。宿主仍须从同次捕获的真实项目/catalog/UUID/provider 取得资产，核原始诊断、设备与当前 publication，缺失不得构造空纹理或启用预览。

8 个纯内存回归定义覆盖原始引用/完整描述符、缺省与无诊断、非法值及规范化、normal map 色彩诊断、容器与非活动 rgba、显式支持的 Ready/Unsupported、array/cube/volume 维度、零 extent/NaN 保留。rustfmt、scoped diff、反向 patch check 均 0；标准 change-guard 单次 217.297s 退出 0，errors/warnings/suppressed=0/0/0，子进程已回收。测试定义不计执行通过。

本轮只读真实 Penpot 原资源 file `b4a1e3ce-9e6a-81fe-8008-d69d60ef2172` / page `…2173`，rev107/2002 objects。原 `TextureSourceContent` component `02db8ec6-f26e-482e-9df2-1c0a02753b02` / main `7f5ecf4a-fb69-4710-b1ad-0384d19e2cbc` 及79后代被 fresh 读回，1280×800 /900×620 /640×520 原板身份保留。此为已存在可编辑源图研究，不是新保存/关闭重开或 Engine TextureAsset/GPU producer 证据。

可接的纹理正文应分别显示原始 metadata、派生 render descriptor、原生诊断和显式 CPU preflight，不用无效果 Play/静态图片充作实时纹理视口。实际 mip/channel/zoom/alpha/压缩重导入、文档保存事务、焦点返回及原生 EXE 输入仍未接；本轮不造共享 Field、Texture Toolkit、第二缓存或 backend。UE 参考继续只读 TextureEditorToolkit 的 imported size/resource mip bias/resource memory 区分。


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

细节：原始8root与16新子形状UUID、四个flow/URI身份及精确source坐标对比见 native-readback-r02.json/native-request.json。最大原生packed坐标误差0.00018609375逻辑像素仅说明SVG几何转换；不代表字体行框或Engine渲染误差。四个真实source board与一块受保护CJK长文本board逐图复核，closeup为直接native clip，无重采样。


### RT-S22-ROWOPS01：DataTable 完整源行复制与批量删除（2026-10-25）

原 `DataTableCellCodec` 新增 `prepare_row_duplicate` 和 `prepare_rows_remove` 产品源码候选。前者复制真实原行的全部 JSON 字节，只改精确 key literal 后追加；后者按精确源键一次预检整批删除，保留存留行顺序/原始 tokens 与数组外层空白。两项均先校验全源/schema/cache，再重读核对完整 columns/keys/field IDs/typed cells（含 u64 边界和 f32 符号零）；缺失/重复选择拒绝，空删除选择仅在全预检后返回 None。不 I/O、不启用命令、不写第二 store/history。UE 依据为 DataTableEditorUtils RemoveRow:457 / DuplicateRow:562 与 DataTableEditor DuplicateSelectedRow:1012 / DeleteSelectedRow:1049。

本批8产品路径＝3原文件插入+5新文件；可精确重建3份旧字节前像，9参考pin未变。增量 patch SHA `f64d78eef0b34efdc4b890186b6d75bce69f5c89d1d476dc7adb03813b7d50c4`，rustfmt/diff/reverse均0，guard exit0/76.719秒/0 errors、0 warnings、0 suppressed。新增9项必要回归定义，当前 data_table_ 共42项；这不是已执行数。

独立新源业务批次 request `ue-assets-data-table-row-ops-20261025-01` / operation `validate-data-table-row-ops-01` / identity SHA `421c6eb896fb0c378161272f2fcba444491f33fb7156babda26f4d4288cd8e11`，当前53172128 capsule 12pins及2canonical匹配，public capability与纯 client/server recipe核对通过。Editor单包/defaults-off/features[]/lib/profile test/filter data_table_/jobs1/threads1/物理E原入口实际POST一次，client exit0/222.016秒，queue18404，input `e05f6fe41fb6fdab03d546463d7803ba77eb824b13bc029d014cfb94e4e814a7`，actualTargetDir null。同身份一次QueryOnly exit2/6.390秒：外层 phase=running/status=pending，内层 status=waiting/reason=management_git_busy，input一致、resubmitted=false。不能把阶段字段当Cargo已启动/未启动或Rust通过，verified Rust0、实际执行未知；不重投，不改原#384/#392/rename queue17885输入或未知锁。

键继续采用明确的非空、区分大小写 Unicode JSON 字符串；不自动套 UE FName 后缀、NAME_None 或归一规则。复制后选择真实提交的新key；删除按提交后的同代过滤视图重算 UE clamped survivor，显示行号不作源身份。AddRow/上下面插入缺真实 RowStruct 默认构造 producer，不能以全零默认代替；完整表格/正式导出/正常EXE/144DPI/双端误差继续open。本轮没有设计或截图通过声明。


### RT-S25-KEYS01：Curve 整批关键帧插入/删除源候选（2026-10-25）

原 `StandaloneCurveCodec` 新增 `prepare_keys_insert` / `prepare_keys_remove`，均接收完整 `(typed channel, authored key)` 数组，一次返回可选完整 DataAsset 候选。插入按真实 input 排序；删除比完整 expected key（含各 f32 位身份），缺失/重复/stale或末项无效均拒绝整批。全源和最终 surface 都须准入，空批次也先预检；空 channel 保留原身份，不补假键。沿原 codec 编码重开，有变化会重排 JSON 排版；typed 增删互逆不等于字节撤销，原宿主须保留完整 before/after，经一次原 Document/permit/history/CAS 提交。`CurveModel`、`StandaloneCurveDelta`、adapter、源 schema/版本及 equality 完全未改，未新增 DTO/store/clock/backend/UUID producer。

7路径＝3精确旧前像+4新增，12参考pin保持；patch SHA `9cf6742716d51e3a09852eae499df8de95379765a830363e25b6b8978b6c7fbb`。新增11项回归定义，当前 standalone_curve_ 共42项；fmt/diff/reverse均0，guard exit0/78.469秒/0 errors、0 warnings、0 suppressed。仅静态通过，verified Rust0、实际执行未知。

本批新源业务 request `ue-assets-curve-key-structure-20261025-01` / operation `validate-curve-key-structure-01` / identity SHA `78cb08a7c37b3be8cb69d9b167891e71522b81df06ee1be96cc7b6cee2c84e35`，当前53172128 capsule12pins/2canonical/public capability与纯recipe核对通过。Editor/defaults-off/features[]/lib/profile test/standalone_curve_/jobs1/threads1/物理E 原入口实际POST一次，client exit0/171.328秒，queue18663，input `17be5c55e3b42e228ee348dbb256721fbb0f2a1d6ba48e16c3c3e1d3843db61f`，actualTargetDir null。同身份一次 QueryOnly exit2/9.937秒，phase waiting / status pending / result waiting / management_git_busy / resubmitted=false，原 input一致。不据此推断 Cargo已/未启动，不重投或修改旧#388/#384/数据表请求，不释放未知锁。

当前模式边界保持：各channel可不同模式，但同一channel须统一Constant/Linear或显式linked/unweighted User Cubic；原/最终的显式default及mixed表面仍UnsupportedSurface，Auto/Break/weights/f64/非Constant外推继续拒绝。UE RichCurve.cpp AddKey:303/DeleteKey:365/AutoSetTangents:680 与 CurveEditor.cpp DeleteSelection:2039 已实际阅读；统一User/Cubic手动斜率保留，删除后typed selection应由原宿主在当前代成功后按UE清除，不把动画time-bit ID当资源持久键。本轮没有GUI/曲线求值或实际输出通过。


### RT-S24-SOURCE-TABLE02：原静态网格业务主件的真实源表格（2026-10-25）

原08 file/page保留，真实112→113(API)→114(nativeUI)→115(API)→116(nativeUI)，现2069objects/6business mains。沿原 `StaticMeshSourceContent` component `c2b1f915-4203-4a85-b433-152c4b6f13f5` / main `37a2a346-caa1-4820-b1f2-5de79d8f8095`，三张原1280×800/900×620/640×520板把一段primitive文本拆为原生列/行flow、标题、2行真实源值和说明。GLB SHA `7a5f880c…16d31131` /318100B，行0=610POSITION/984indices/material0，行1=816/3744/1；仅源JSON声明，不作Runtime几何/静态类型/资产UUID或GPU ready。保留3原heading IDs，新增51本域后代/34 near refs，0新main；复用真实Shared5/4masters/34colors/3typographies，不造公共表格、主题或Property编辑器。

三表原生可访问Row gap控件4→5→4触发原flow，各28高行沿已有density.row_height、gap4/Shared body，数值列右对齐。第一轮6原PNG已逐张打开，发现Indices与左对齐Material容易连读；只把9个自有cell的Material改为语义右对齐，再保存关闭重开、逐张打开最终6原PNG。两轮保存重开data均相同，最终SHA `c82f84f3…c2eff774`；截图为原生Penpot1600×1100/DPR1/71%、98%、134%画布缩放，不是Engine/实际150%DPI。[原件及逐图复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-design-02/individual-review.json)。

[源核对](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-design-02/source-checks.json)：2006原对象完全相同，12改动都在声明viewport内；原5其它business定义精确不变，本主件仅modifiedAt变，34新near refs内容匹配，36cell作者/文字缓存匹配，7输入hash不变，Shared完整raw响应不变。原精确touched列表相等检查为18/19，失败原样保留：两处原heading副本在原geometry-group后追加content-group/text-content-text；原组均保留，非本轮heading的旧override未变。另6新cell内容override和2表gap override明确登记，[补充分析](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-design-02/override-preservation-analysis.json)，没有清override、弱化official guard或把显式copy作者操作说成main自动传播/reset通过。

保留prepared rich-text编码问题、API500 Point标签拒绝(112数据未改)、缺cljs全局和旧selector两次0source-write失败；经真实Point类型/原生accessible控件完成，不改Penpot源/服务。右侧Shared Search在900/640的源位置缺口仍open。原productZUI/IDs/events/bindings、Rust/公共主题未写；official插件/源closure/export/load、真实provider/preview/菜单/焦点/撤销/滚动/EXE144DPI及双端≤1px仍open。4自有Node均wait，4browser.close均await；3独立捕获Chrome根及4Node当前PID无row，首轮Chrome根未另录不造native终态，temp0文件/0B，释放0B。[精确交付](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/static-mesh-source-design-02/handoff.json)。


### RT-S22-CSV01：CSV 精确支持与 UE 导入差异（2026-10-25）

在原 DataTable 模块增加 `csv/{decode,encode,error,records}.rs` 和结构根，沿原 `DataTableSourceAdapter`/`cell_value` 预检。同一显式 Struct schema、JSON源、精确非空Unicode/区分大小写行键；header用真实字段name，不用display label，可重排列及显式key位置，但必须完整包含所有serializable列。bool、i/u8/16/32/64、有限无损f32与String沿现有typed解析；不经f32显示处理u64/i64。UTF-8 BOM、LF/CRLF、逗号/双引号和引号内换行支持，string空值/空格/公式文字按源文本保留。

导出保留真实源行序和非string原JSON token。重导入一次构造完整候选，末行错误也拒绝整批；header-only表示空表。未改单元格保留原token，含超出现编辑器范围的旧值；新值/改值仍检查类型、位精度与range，step/precision不作舍入。readonly/hidden列可原值往返，修改或新行初始化拒绝。不变返回None，保留原URI/format/bytes/cache；有变化的整表包装重编码，不能用typed互逆代完整原源字节撤销。

UE参考实际 `Engine/Source/Runtime/Engine/Private/DataTableCSV.cpp` WriteTable:19 / WriteStructEntry:98 / ReadTable:125。其FName默认key/消毒、RowStruct默认初始化、缺列恢复/保留旧行/跳过错误和部分行提交未模拟；首版全部明确拒绝或不提供。输出CRLF而UE为LF；支持类型之外的嵌套/引用/容器/枚举/f64继续拒绝。该CSV只是JSON DataAsset交换格式，不按扩展名、JSON形状或板名把Data注册成DataTable。

9路径＝3份旧前像byte-exact可还原＋6新增，11只读参考pin不变，patch SHA `c4a7ee48e3694a85a27815d2199de9e12f271cf26a0859e92b4cfa20294f7b83`。新增13条必要回归定义，fmt/diff/guard exit0/0errors/0warnings/0suppressed；不是Rust执行。新业务identity SHA `253edb50621658eeca608379bd36c896a6b6955dd75e490d28f0b13f2cc4a507`，Editor/defaults-off/features[]/lib/profile test/data_table_csv_/jobs1/threads1/物理E。原Editor manifest的Runtime target-editor-host和test-support依赖保留，无Cargo变更。实际一次POST queue19307/#409/input `0f1687c0f23f56e6ebee06f5a3144a98bcd11eb6a79f33af95e2b3608d4f68a6`；同身份一次Query running/pending/resultnull，认证API building=true/resultnull，尚无Cargo/native/harness结果。

[精确合同与证据](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-csv-01/scope-contract.json)。实际schema资格、文件选择/路径、Document/history/CAS/SaveCtx、命令/焦点/撤销和provider未由本codec补造；原ZUI/IDs/events/绑定未改。passed/accepted=false。



### RT-S22-COLUMNS02：列宽调整／中文长文本原生状态板（2026-10-25）

08原file/page实际保存至rev128/vern0/2300 objects，仍为6业务main；在原900 linked-copy board原生Ctrl+D生成3个新板（231对象），再用原生Inspector调整到1280×800、900×620、640×520。原2068非根对象与6主件定义精确不变；根仅增加新板，旧子序列顺序保留。原DataTable component dc042d85-e62f-4025-9680-10ee1619c9d7 / main a7972f31-e63f-4359-8e6d-3e871ebb2d8d不变，未新建/复制main或detach。

这是原DT_Items四行ZUI布局fixture的显式状态示例，未创建真实DataTable资产/schema/UUID/producer。只在新增实例把表头及四行Name列85→180，列宽统一为17/180/76/77/55；新增中文行名和完整长Value，原生Text Auto Width使Value保持单行，由原77宽clip cell裁切。保留原Fill、12px Roboto、Shared body typography ref；中文可见是浏览器fallback，不能据此证明Engine/CJK字体ready。原默认三板不改。

最终r06六张full/crop原PNG已逐张打开；native save-close-reopen128 data SHA 06f62581abb137833ea093a07bc11ec01414482c38eef1ad79dfd8a87d9b8e59一致，fresh GET也一致。新增201本域+21 Shared Search sourceRef全部222/222有效；453个非空style RefId及其453个RefFile逐对有效（非453个唯一style）。Shared真实rev5/4主件/34colors/3typographies、原直接库连接保留；get-profile原响应SHA前后一致，未Install/Open插件或写registry/Shared。源ZUI/catalog/tokens/Starship等8pins与CSV9源码pins仍匹配。

保留F2定位超时、X轴名称不匹配、r03把3个图片pair误比6个文件导致success=false、900单行改动的过早GET失败及随后rev126真实持久化、r05 fresh-rev拒绝、向量字符串触发的本地text walker异常和列存储反序诊断。r06等待实际发布后重开；原始失败未覆盖。12原PNG逐图记录、原控件/事件/import映射及接入缺口见 [handoff](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-columns-design-02/handoff.json)、[individual-review](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-columns-design-02/individual-review.json)、[source-checks-r02](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-columns-design-02/source-checks-r02.json)。原Not implemented (preview)反馈、真实表schema/registry/provider/Toolkit、原Document/history/CAS/SaveCtx、菜单/编辑/保存撤销/滚动焦点、正式family插件导出加载、正常Engine EXE/144DPI/双端<=1逻辑像素仍open。此次无产品ZUI/Rust/公共源写入，无Cargo/桌面/服务/DPI动作；产品passed/accepted=false，goal继续active。

CSV原queue19307/#409最新有界认证API已为building=false/FAILURE；原console terminal为management_operation_interrupted、HEAD, full index or complete source membership changed during capture、statusunknown（保留此前3次resource_capacity等待）。这不是Rust测试失败，也没有native EOF或资源释放证明；原9源码/13定义不变、验证Rust0、实际执行未知。[终态复核](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-csv-01/build409-terminal-review-01.json) SHA 67ad89b31ac7256f01c64b2f66dfe1cd7083973eaeb83c20c38264fee3fbaae4；原control receipt540B SHA eb28e1de273fb54d81c6120fc6d42a5b4b9adf67be106e549a7e51543da75ad1。已沿原identity253edb50…/input0f1687c0…向Jenkins原owner排入消息01a17393-17f2-77c3-bf40-5beed1bddaa8，queued不等于已读；无新QueryOnly/POST/identity/attempt/freeze/锁或State改写。


新board身份：900 ead27079-b190-80ee-8008-d826fca58783，业务copy ead27079-b190-80ee-8008-d826fca58784；1280 12611fd2-8f67-8076-8008-d828b7e9d5d0，copy12611fd2-8f67-8076-8008-d828b7e9d5d1；640 12611fd2-8f67-8076-8008-d828d88efb8a，copy12611fd2-8f67-8076-8008-d828d88efb8b。UE DataTableEditor.cpp 743–783/1163–1199沿FontMeasure max+10或持久manual width，1227/1250/1263共享列宽，1275数据表头本就center；SDataTableListViewRow.cpp 237–289为4/2 padding与单行行名/值。旧默认宽度原本固定，不能仅因表头文字起点与左对齐单元格不同就宣称错列。本次API设计内五行列边界误差为0，未测Engine边界。首版r03四行wrap露出下一行的笔画已由r06原生单行处理；没有通过缩字、隐藏Name或改源资产关闭差异。三种画布fit约71/98/134%不是系统DPI。


### RT-S22-MOVE01：行拖放与首尾移动的源语义（2026-10-25）

实际参考 UE DataTableEditorUtils.cpp 610–677 的 MoveRow 和 SDataTableListViewRow.cpp 89–132 的 OnRowDrop：按真实 RowId 查源位置，再移除/插入，成功后保留该 RowId 并回到 RowNumber 升序。资源候选 prepare_row_move 接精确源 key 与目标 key，不接过滤/排序后的显示序号；上/下/首/尾命令由原宿主在同次当前源上取相邻或边界 key，不保存第二顺序。自移只有全源和两 key 校验后返回 None，空表/缺 key 明确失败。

每个完整 JSON 行保留原字节，外部空白和每个位置的分隔片段保留原槽；缓存重算并严格重开验证整表 typed 内容。反向 key 移动可还原文本，却不保证还原原 null canonical cache，撤销须保存实际完整 before/after DataAsset，不能用回放命令充当 inverse。既有 rev128 三尺寸状态板未改，本轮没有真实拖放截图。9 条定义仅静态通过，#413 已实际收件、运行结果未知；[接入合同](E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-move-01/integration-contract.json)。表语义、宿主命令/撤销/排序/当前代次滚动焦点及产品画面仍 pending，passed/accepted=false。


### RT-S22-MENU01：可编辑行菜单／原生三尺寸重开（2026-10-25）

08原file b4a1e3ce-9e6a-81fe-8008-d69d60ef2172/page ...2173实际rev252/vern0/2759对象、6业务main；新增459对象，旧2299非根对象、6主件定义及旧根子序列精确不变。3个新增DataTable copy+3 Shared SearchInput native linked copy，222新引用=201本域+21Shared。Shared实读rev5/4main/34colors/3typographies且本阶段数据不变，无新main/detach/并行主题。

按实际UE SDataTableListViewRow菜单实现11项/3分组的纵向flow设计，行内caption Fill、5个56宽右对齐快捷键；plus/首尾双上/双下均为原生vector，共9 path，无文本箭头。288×271.4的原生absolute popup按实际Name cell中心夹在板内，native small nudge的Y舍入-0.2，zIndex=1；普通行280×20.4/columnGap10保持flow，48段文本消费原Shared body。此为原4行ZUI layout fixture，不是就绪资产/schema/Toolkit或真实菜单执行。

save-close-reopen及6次捕获后data SHA b415e85d193aa9397d6f19d2c92ccaf696f84ccdef48e83f8d70e77c7d663dbf一致，rev252；两独立context均await关闭。六张原生100% full/crop逐张打开，crop精确1280×800/900×620/640×520，菜单末项完整，未缩字。Chrome dpr1/100%不作Windows150%DPI/正常EXE证据。25/25源pin相同，原ZUI/IDs/imports/bindings/events/Rust及公共主题均未改。

三板及菜单对应：1280板613f52cf-9193-8011-8008-d8427a15e644/menu ...e649；900板765dd739-1bd1-808d-8008-d83273b629bc/menu45280201-8663-8051-8008-d83f627f376e；640板613f52cf-9193-8011-8008-d8428d1ea482/menu ...a487。真实完整UUID及原DataTable component/main、Shared roots保存在交付JSON，不作为媒体/Engine资产UUID。

本次保留文本工具/保存延迟/原生内部错误/错误F2重命名/SVG中心落入子行/剪贴板样式引用丢失/提示列宽和z0遮挡等诊断，仅恢复自有新增菜单：原生Alt+N、16文本真实Shared body引用、56宽快捷键、一处自有多余SVG移除及zIndex1。未重启服务/新host或重复Install。

第一one-hop检查对14旧嵌套引用报false，原结果保留。r03逐个证明当前nested copy→精确外层本域main内同一组件file/id与父引用匹配的nested copy→原canonical Shared main/后代/type，两段闭环合法。全1678引用=1664直接+14两段，新222全部直接，不用任意页内UUID存在放宽guard；不能把此检查误报写成旧组件损坏。

菜单所有productEnabled=false：Duplicate/Rename/Delete/Move只消费原真实纯源候选；3种insert默认构造/新key、行clipboard、语义行引用仍缺，不能用G01过滤后[]报零引用。原ContextMenu/PopupMenu/当前document/view/one permit/history/CAS/SaveCtx由既有owner接线，不造route/store/clock。泛型图标、UE brush几何、small keybinding字体/padding差异、正式source闭包/导出/加载及真实焦点/快捷键/撤销/保存继续open。ReactBits参考沿DataTable+AppDialog，UE实际源优先。

资源未就绪，正式plugin测试/import-export/load、真实popup输入/disabled/IME/Escape/当前generation焦点返回、Document/history/CAS/SaveCtx、正常EXE/原生LayoutReviewCase/144DPI及双端≤1逻辑像素仍open，passed/accepted=false。无桌面/服务/源码副本或新Rust构建，释放0B，必要失败与交付原件保留。

[交付](<E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-menu-design-01/handoff.json>)，[逐图记录](<E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-menu-design-01/individual-review.json>)，[嵌套source核对r03](<E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-row-menu-design-01/source-checks-r03.json>)。


## RT-DT-CELL01-R02：当前源与26项精确验证后继（2026-10-25）

- 源责任 owner 仍为资源08。正式 request=ue-assets-data-table-source-20261025-01、原 session 不变；本次显式新 operation=validate-data-table-cell-source-r02，不是恢复或重投原 #392。原 identity/marker SHA401aaa2b…逐字节未变，原 unknown/native/resource 边界继续保留。
- 当前源 revision 944a2dddbe872852e51fd763df7f91fdc138f28abe218327d203e347dd59d593：10份源声明、28份只读依赖。原 CELL01 为6/10匹配、旧依赖10/10；四个已记录后继不回滚。当前全部38 pin 再读匹配。revision 是报告身份，不向协议加入不存在的 sourceRevision。
- 有限测试采用 --exact 和完整 ui::host::editor_asset_manager::data_table::tests::... 名称：原 source adapter 12 + editing 14 =26项静态定义；当前已接线 DataTable 子树64项，其余38项未列为执行。目录 assets 不等于实际 Rust 模块 editor_asset_manager；不使用宽前缀来虚称26项。
- current selected 5317212844a5cd6ff9117849c8bb01a93b8e97f5ae614778c445b34f005b678e 12/12 pins、public transport、client parse→server validation_recipe 均已核。单 zircon_editor、defaults-off/features=[]、Windows host lib、显式 profile=test、locked/jobs1/threads1、E:/cargo-targets；正常 prepare 自行取得真实 source/resource/writer/native 准入，不额外索要 grant/freeze。
- 本次正常 wrapper 仅调用一次，客户端exit0/reaped，resubmitted=false；真实 queue20950 → zircon-management #419、input 8a8059bffe1bf3fa7662448ff941592c88453d42eddd57f126e13950ffb96e82。17:31:36Z认证API已building=false/result=FAILURE；控制receipt SHA3babac39…、精确State key a0a18027d651a5e62c5f1c70c7d255b7ed229076f3af3e8aed9939e7c6d652c6 v2/unknown，reason=management_operation_interrupted，原文HEAD/full index/complete source membership changed during capture。该通用错误不能识别实际变化分量/写者，不能写成Rust失败、确认未启动或native/资源已释放。
- 只读格式首试误用2024 edition，失败保留；按实际Cargo.toml 2021 edition复查，10路径rustfmt与diff均exit0，无生产源码改动。0 formal QueryOnly/重复POST/服务/桌面/清锁/清缓存；verified Rust0，actual Rust/native/resource结果未证，passed/accepted=false。产品schema/Toolkit/Document一次history-CAS/正式导出/EXE/150%DPI继续open，目标active。
- 完整范围、26项名称、38 pin、argv、原始stdout/stderr、真实API/console/控制与只读State见 E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/data-table-cell-validation-r02/。旧#392与新#419分别保留，后续仅交原执行owner对账本新链，不盲重投。


## RT-SW-RANGE02：SoundWave可见源帧区间（2026-10-25）

UE WaveformEditorSequenceDataProvider.cpp::RequestSequenceView（25–48行）根据可见比例读取完整多声道帧切片，并把原始起点随视图广播。现已在原 SoundWaveSource 中补 waveform_frames(channel, Range<usize>, NonZeroUsize)，复用原借用式 SoundWaveEnvelope；返回 bin 的 start/end 始终为原始源帧坐标，end exclusive。原 waveform(channel,budget) 通过0..frame_count复用同一实现。区间外峰值不会进入当前包络，原声道顺序/相位及全源有限值预检保留，空区间返回零bin，倒序/越界明确InvalidFrameRange，不夹紧为另一个有效视图。

写集仅 sound_wave/source.rs、envelope.rs、error.rs、tests.rs 和新增 tests/visible_range.rs，原 assets/mod.rs 与 sound_wave/mod.rs 均只读保留，未重apply原0276242a补丁。新增7项回归定义、原10项正文保留，当前17；Rust/harness未执行。当前revision 88cf9a92e279e7dce74e6dd83e33fe49170ad0f609878ae721422467d4f27b41，增量patch6ded3ac093f8fcabf85f9b413b749138552b05e33784591b4c44ee524ef87cf9。格式/diff/反向check/guard通过，11只读依赖匹配；45秒guard超时与76.422秒同范围成功均保留。

差异仍open：UE ratio→frame取整/min-two-frame夹紧、原view的zoom/pan/选择/焦点/paint-hit尚未接入；本层显式允许空range用于空数据，不宣称UE接受空ratio。UE transformed PCM/layer链不由本适配器替代，trim/fade/normalize及原Document/history/save另行接入。range成功也不证明asset UUID/当前catalog或view代次，宿主必须在同一次真实source/publication下计算和呈现。不得从包络启用Play/Record/Seek或把sample frame变为动画FPS/CurveKey。

原Penpot业务组件/main/copy、原ZUI/ID/event及公共主题未写，本轮无新的Penpot/plugin/EXE/DPI画面证据。SoundWave新request/operation/attempt保持null，原#388/#392收件与unknown/native/resource边界只复用回传，不轮询/重投/释放。详细源、UE pins、消费合同与17项有限验证提案见 E:/cargo-targets/zircon-local/reports/ue-assets-tools-20261024/sound-wave-visible-range-02/。目标active，passed/accepted=false。
