# Actor / Blueprint：开放差异与验收场景

## 实现35：筛选和展开已可同源计算，宿主行为仍开放

[原reader组合入口](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-visibility-35/semantic-review.json)解决caller需另拼filtered+collapsed遍历的问题：只查留下且可见的分支，子项被裁尽时has_children=false，全匹配与原未筛选结果等价。它仍只提供本次source的只读行，不产生实际TreeRow/事件或注册producer。

Parent原per-view producer/materializer须以原view incarnation/project/document/selection/request/source及原expected key读前/发布前同源核验，stale仅退休原view；不能以fresh key接回旧迭代器或因筛选只剩一行便把重复ID当唯一target。UE搜索自动展开/主动选择、keyboard/focus/scroll、attachment与写事务仍open，不能将该数据查询当交互完成。42定义/0 verified Rust，正式export/import、正常EXE/150%DPI/双端仍pending，passed/accepted=false。


## 实现34：来源筛选已落源，搜索行为仍开放

[筛选投影34](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-34/semantic-review.json)已保留命中路径、裁剪无关子项、重算有效children，重复/断边与原native key语义不变。它不读控件或更改展开/选择；原visible_hierarchy_rows也未合并搜索状态。下列差异均open。

| UE依据 | 当前边界 | 接续归属 |
| --- | --- | --- |
| TreeNode262/300：term匹配、真实UClass过滤与Actor root特例 | caller predicate可读原记录；无第二parser、继承推断或Actor class后端 | 本域语义/真实schema与Parent caller，不能按provider字符串启用能力 |
| SSubobjectEditor2224/2271：搜索展开、选择端点/根fallback | 本方法仅行投影；不能套设计33的收起清选规则 | Parent原选择/展开/焦点及唯一稳定target；歧义不得选第一行 |
| 原view/source/key发布边界 | stale保留旧数据不刷新；未有真实per-view producer/materializer | Parent读前/发布前核原context/key，只退休旧view |
| 产品与验收 | 40定义/0 verified Rust；无本轮EXE/正式导出/150%DPI/双端图 | 原02受理链对账与公共export/host/capture，passed/accepted=false |


## 设计33：已核树规则及仍未实现的交互

[真实设计33](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/result.json)已把“收起隐藏项时清选择、重新展开不恢复”的UE默认选择规则做成三宽原生可编辑状态及pointer跳转。源码reader32仍只计算可见行；生产选择/Details不是已接。以下均继续open，passed/accepted=false。

| 范围/参考 | 实际与差异 | 原因/依赖/后续 |
| --- | --- | --- |
| STreeView635/741隐藏选择 | Penpot原生跳转隐藏6并清Details，再展开不选回；无运行Editor证据 | Parent原per-view context/key同源检查、原选择集与typed materializer；AB-PF-25，不造第二store |
| 三宽caret与键盘 | native SVG路径可点击，group中心不命中，BODY焦点 | 当前painted-only disclosure hit；公共owner按真实源交hit/focus/键盘合同，本域未扩大row点击动作 |
| 900/640 Details | 空内容实际可达，几何吻合既有8/32内边距；relative-to仍旧trigger ID | 原生复制保留manual anchor字段；需原public/codec owner明示关联规则，未凭屏幕位置签export，焦点返回未过 |
| 字体/文本/1DIP | 28图已复核，中文及长诊断可读；未测本轮glyph ink或Engine字体 | 原provider保持；19ink/16linebox、CJK度量与双端逐行仍待真实实现/证据 |
| 正式导出与产品行为 | 本轮无官方ZUI/Normal Actor/实际150%DPI、搜索滚动/菜单快捷键/写事务undo | 原public owner正式source/export/import，Parent真实producer/host/命令/捕获；fixture不计产品完成 |

初始1280两图顶7px过渡与900热点图已保留并由稳定图替代，不改源掩盖；fresh展开图逐字节一致。保存前日期解码损失另明列，不签原始完整日期元数据。37静态定义无Rust回执、原02/#387终态及旧清理保持，goal active。


## 实现32：展开语义已落源，生产交互仍开放

[可见来源树32](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-visibility-32/result.json)复用原forest并接收caller展开查询，保留异常根和完整u64；没有添加第二树状态、序号target或自动选择fallback。收起后的选中/焦点保留与滚动由原host处理；递归展开命令、组件attachment、per-view typed rows/producer与失效仅清旧view仍由原owner接续，详[AB-PF-24](actor-blueprint-shared-contracts.md)。

37项静态定义未执行，原02 accepted19/unknown不代验；本轮只读fmt/guard通过不表示产品通过。正式源导出导入、正常EXE/native case/150%DPI/并排差异和≤1DIP仍open，passed/accepted=false，goal active。

## 实现31：实例投影的原来源证明与剩余边界

[新源31](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-instance-source-31/result.json)已把“实例Prefab引用必须匹配原resident key”落为原reader真实方法；错误不能生成Available或空数据。它借用原override DTO且保留opaque/重复记录，三项回归已定义，格式/guard通过；34静态定义仍未执行。

仍缺：实例路径→实际entity/component/property的已确认解析语法、provider/schema授权、真实per-view producer/typed materializer，以及通用原Document事务/undo/retention。原PrefabPropertyOverrideAsset没有component实例UUID，不能拆显示路径或序号补身份。Parent原context/native key前后核验及仅清旧view边界保持，详[AB-PF-23](actor-blueprint-shared-contracts.md)。

没有用新reader方法冒称生产窗口连通，也未改31节点ZUI/Shared/主题；Penpot30归档与正式导出、正常EXE/150%DPI/双端≤1DIP分开。原02已受理19、#387 unknown/interrupted不覆盖当前34；无新验证身份/重投或Cargo，passed/accepted=false，goal active。

## 设计30：新交付与继续开放的差异

[来源设计30](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/result.json)已保存并fresh重开三宽与两Details，全部五条原记录和值/引用未合并，原组件库/旧对象保持。它只消费source29单元输入语义，不计运行中Prefab生产源或整个Actor/Blueprint窗口完成。

| 差异 | 实际证据与下一归属 |
| --- | --- |
| 真实Actor数据与写命令 | 仍无per-view生产publisher/typed materializer/事务接线；Parent在原host以incarnation/project/document/selection/request/source与expected native key前后核验。本域不造第二store、fixture producer、attachment或稳定UUID。 |
| 焦点与业务交互 | 原生Details开关实际成立，DOM BODY；搜索、行选中、菜单、快捷键、滚动/undo未执行。不能用按钮轮廓当focus return，[测量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/measurements.json)。 |
| 窄窗信息可达 | 640原能力卡继承hidden；完整边界已补入真实toggle Details，无额外offset或公共组件变更。初次遮挡误判和未执行hide尝试保留，[补正](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/narrow-reachability.json)。 |
| 字体与1DIP | SVG声明Roboto，中文实际system Noto Sans SC/custom=false；fullID为custom Roboto。16px DOM rect/19.0666px computed lineHeight不是glyph ink精度证明；源custom/Shared gfont/隐式CJK仍分别核对。 |
| 官方导出与Engine验收 | 原31节点ZUI及稳定ID/引用/事件未改；正式codec/export/import与正常Actor LayoutReviewCase/EXE指纹/150%DPI/双端逐图仍pending，不借旧root/nonce。 |
| Rust与原02链 | 当前31静态定义无新执行，accepted19不覆盖新源；原02/queue15836/#387 FAILURE与unknown/interrupted按原时点保留，不重投/换ID/State改写或假batch。 |

公共Shared当前5/4mains/34colors/3typographies，未再以rev0/无style作当前阻塞；共享pipeline与Parent各自scope保持，goal active、passed/accepted=false。

## 实现29之后的组件诊断边界

来源模型现在能给出每个序列化owner内相同四字段描述的所有原位置，opaque记录不合并，原实体/组件顺序及native key合同保持。具体[AB-PF-22](actor-blueprint-shared-contracts.md)可供原typed materializer显示歧义行；它不提供持久组件instance identity、attachment、provider schema或写命令许可。重复描述只表示descriptor查询不能唯一定位，不将原记录判为非法。

新增冲突内容尚未进入实际per-view producer或Penpot/EXE控件；Penpot28 rev176仍展示source27唯一匹配/其它lookup结果的unit设计，不算本轮运行验证。31静态定义当前0已验证执行，原02已受理19项不可代验；原unknown/interrupted单recipe接续需原Jenkins authority实际资格。正式导出、normalActor/native case、焦点/滚动/快捷键、实际150%DPI与双端≤1DIP仍open。当前四源/guard/格式证据见[结果29](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-component-conflicts-29/result.json)，goal active、passed/accepted=false。

## 设计28之后仍开放的能力与验收

- 新三宽readonly组件内容与Details原型已保存重开，具体来源是source27单元输入，不是生产project/document/selection的查询。真实producer、per-view incarnation/project/document/selection/request及原native key读前/发布前同源检查仍由Parent caller接续；失效不能用fresh key重绑旧回调。
- 四字段描述只定位来源记录；模型没有持久组件实例身份，未知provider/schema仍opaque。字段编辑、删除、覆盖、Apply、history/undo不能据此启用，Blueprint类/接口/构造/事件后端仍缺。
- 原生640弹层top/bottom差0.015625px、900差0仅为Penpot内部几何；19px中文ink/16px linebox、混合provider及真正Engine字体仍未通过。DOM焦点BODY、900关闭图点击轮廓可见，键盘焦点返回未通过。
- font probe编辑入口引入空段落的失败保留，最终rev176经精确内容/逻辑行、正常resize及新context复核修复；不是公共worker修复或原生插件验收。
- 原4Rust/5保护源未改，28静态定义无当前Rust回执；原02既有queue15836/#387保持unknown/interrupted，不重投或换ID。正式源导出/原31节点ZUI、normal EXE/native case/实际150%DPI/双端逐图≤1DIP仍pending。
- 自有20观察birth已退出，首失败profile7215255B待受管清理，旧126B及phase26清理历史不变；本轮手工释放0B，cleanup/passed/accepted均未关闭。

当前设计与失败身份统一见[结果28](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/result.json)和[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/visual-review.json)。goal active，继续独立业务切片，缺共享项不作为整族停工原因。

## 设计26：来源异常内容已设计，产品链路仍待接

- 三宽新增可编辑source25内容板：七实体/六根/四诊断，循环有效后代13仍选中；数据来自单元回归，页面明确标注，不注册fixture producer，不把source_index当持久目标。
- 保存关闭重开156/1679、591新对象、原1087非根对象不变；1195引用有效、99 Shared根、Shared5不变。640抽屉原26像素残边已用346宽修复；900边界差0，640底差0.015625px，只是Penpot内部比较。首次图和工具失败保留，详见[结果26](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/result.json)。
- 仍open：两个BODY返回未验证键盘focus；搜索/展开/选择是设计内容，尚无真实typed materializer及per-view发布。混合文字仍有系统CJK/Times回退、19ink16line及Engine度量差异；不改SharedTypography或补签字体通过。
- 原4Rust/5保护源未改，25静态定义无Rust执行回执；原02/queue15836/#387及原accepted19项不变，不重投/换ID。正式import/31节点ZUI、真实写事务/undo、类/接口/构造/事件后端、normal EXE、150%DPI和双端≤1DIP仍pending。
- 恢复helper正常退出，25观察birth无剩余；首失败遗留11,413,746B profile待原Jenkins按精确计划处理，无server维护receipt。新profile自动消失，手工释放0B；goal active，整族passed/accepted=false。

## 设计23：已修来源断行，产品与字体边界仍开放

- 已落实：三宽原mode统一纯中文并选源custom Noto400，13 glyph实测；640原shape移到独立root flow行，128宽/32高变624/16，单行可见。原字号/行高/Shared颜色pair保留，未把Inspector默认14当实际保存值。
- 已接续：640 main下移24/h376，原Details manual marker随移23.9998845、高400→376；native viewer实际开/关，边界差0.015625px。目的地/事件不换；关闭后focus仍BODY，键盘返回没有通过。
- 已核：Actor fresh138/1088、80变化/1008不变、2main/67 Shared heads/690有效refs、Shared5 data exact。旧640失败95节点及900 overlay13节点保持，640 overlay仅原root高度/几何变，12个子节点保持。13原图逐张查看，见[结果23](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/result.json)与[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/visual-review.json)。
- 仍open：Button caption220/公共source guard与qualified SDK、完整CJK provider和19ink/16linebox、真实搜索/树/快捷键/菜单/滚动与focus；本板仍unit内容，不是生产caller。per-view publisher/typed rows/稳定写target/事务/undo、类/接口/构造/事件后端、正式ZUI导出/normal Actor/150%DPI/双端≤1DIP均pending。
- 4Rust未改、23静态定义无执行回执；原02一次QueryOnly仍unknown/interrupted，不重投或覆盖原19项accepted input。goal active，passed/accepted=false；释放0B，自动profile回收字节未知，旧managed cleanup失败保留。

## 设计22：已落实的本域实例覆盖与剩余差异

- 三张已有内容板的Search内部240/204→254/218，native左右约束；六个来源树caption128→190/178。来源未关联场景实例，隐藏24个既有eye/lock/槽；原unit子实体无子项，隐藏3个叶展开符。父展开符/缩进/选中和完整unit源记录保留，不算真实搜索/树操作/命令已接通。
- Actor fresh130/1088，仅39part变化；1049其它对象及全部source/slot/events/字体/字符串/引用保持，690/690 refs有效、Shared5 data精确不变。原失败板/Details overlay保持。12原图实际逐张查看，三个100%原生裁剪均是工作区证据，不是官方export/EXE/150%DPI。
- 该本地覆盖没有更新公共main或重写source allocation/paint guard；source-owner r02 worker/qualified importer及Button caption220仍open。全内容中文provider、19ink/16linebox、640 mode断行和键盘/焦点/首尾滚动没有关闭。
- 生产per-view publisher/稳定写目标/事务/undo、Blueprint类/接口/构造/事件后端、原ZUI导出、normal Actor case及双端≤1DIP仍open。4 Rust当前23定义无执行，原02 queue15836的最后已知unknown/interrupted不改、不重投，19项accepted源不代验23。goal active，passed/accepted=false。

本次具体差异/原图/原生失败见[结果22](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/result.json)和[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/visual-review.json)。本轮helper已正常退出且10观察身份无剩余，临时profile不存在；释放0B，既有受管清理待项保留。

## 设计19：已修复的设计缺陷与开放边界

- 900宽fresh122标题/动作条重叠：原生Layers将既有mode移入根column，fresh125正常分行。1280来源copy高度33不足：只改自有copy的native vertical sizing为auto，重算48，Details随flow留8间隔。旧失败PNG及初次无效drag保留，不把data exact当视觉通过。
- 新三宽失效来源状态可编辑，保存关闭/重开125、20编辑叶custom Noto400、690引用均已核；Stale文案不再冒称具体更新原因。这仍是unit fixture设计，清除旧选择/属性只是设计意图，不计真实publisher、事务、失效回执或UE选择恢复完成。
- 字形约19高与源行框16、公共Button/Search内部allocation及其它中文/700、键盘/焦点/滚动首尾仍open。未发布Field/Number不是等待全部工作的理由；继续在本域已有合法组件和source合同内推进，不临造共享main或value内容。
- 原02实际受理queue15836，后续最后实读为unknown/pending、management_operation_interrupted。原19项定义尚无Rust执行和最终终态回执；保持已受理身份与#348历史，由原authority核State/delivery/queue/build/source/native，不盲重投或改input。
- 原ZUI正式导出/source包络、每view生产publisher/稳定目标/写事务/undo、Blueprint类/接口/构造/事件后端、正常EXE与实际150%DPI、双端≤1DIP继续open。AB-C02及AB-V1～4不因设计图或静态guard关闭，全族passed/accepted=false，goal active。

具体图差异、原因与依赖见[结果19](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/result.json)、[20图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/visual-review.json)及[原02实际交接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/resume-execution-17/interruption-reconciliation-handoff.json)。

## 实现17与设计18的变化

- 本域expected reader已落地，3新回归使当前19定义/0执行。旧key不能采用其它asset/manager或remove/recreate载荷；真实每view/project/document/selection发布前复核、生产publisher及写事务/Undo仍open，不按source_index/显示名派发。
- 六板mode中文实际选用source Noto400，fresh SVG各11glyph确认，987对象/58Shared根/653引用保持。其余中文provider/Shared typography、19高字形与16高shape、Engine度量仍open；不是master继承/A-B。
- 原02已正式尝试准备，字面量`'--'`修正参数边界；完整capture后identity变化exit2/0Cargo。保留#348及原失败，沿同02核实际收件/准备，不把#500管理用例当产品验证。

证据：[源码17](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/resume-execution-17/semantic-review.json)、[设计18](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-local-cjk-status-18/result.json)。goal保持active并行推进，全窗口族未通过。

## 窄窗说明修正16：仅Penpot设计差异收敛

原分配Actor file/page的两张640状态板保持UUID不变，原生Layers拖放将既有document-mode文字从actions移到根column，位于actions与workspace-main之间。原文“来源定义 / 无选中实体”从128×32变为624×16、完整单行；两Button仍240宽，不改Shared内部宽度、main或样式。主区通过fill由416变392，高度24来自正常flow，不写普通内容x/y。1280/900和其它板的935对象精确不变。

[本轮实际记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-status-16/result.json)保留原107换行、109修正及fresh重开6张原图；单行问题在这两种640状态设计中已修正，不扩大为AB-C02/AB-V3运行通过。这是Ready-empty/NotResident的unit状态设计，未接真实生产publisher/操作/undo。Penpot来源身份包络仍缺，正式ZUI导出、正常Actor EXE、150%DPI、焦点/滚动首尾和双端≤1DIP仍open，passed/accepted=false。


## AB-D13来源状态设计14：可见内容与产品边界

当前Actor rev107/993对象、58Shared根、653个有效引用；新增Ready-empty/NotResident各三尺寸六板。原802非根对象、2本域main及旧板顺序精确保留，Shared rev5 data未变。[18图逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/visual-review.json)保留初版6张flow失败与中间4张compact标题失败；最终6张fresh已逐张打开，未见旧实体/组件/选择残留，零数量与未知数量可辨。累计105张browser设计图，不计正式exporter/native EXE捕获。

AB-C02仍open：此处是现有只读API和静态单测的空来源/NotResident设计，不是实际新建资产、root约束或过滤无匹配运行。NotResident不伪装0个实体，Reference/wrong-kind/rejected未合并；真实per-view上下文失效、旧选择清除和重加载必须由生产publisher证明。AB-V1～3只有设计内容，AB-V4无实际150%DPI。没有伪造LayoutRenderEvidence或case nonce。

最终原生flow测量显示标题/action strip无交叠、空态两处消息位于父容器中心、普通文字均在板及父框内；这仅是Penpot源坐标。640 mode说明拆“实体”为两行，CJK/provider/字形/长文本和引擎文本行框仍open。Unavailable按钮文案不是handler或真实禁用证明；搜索、焦点、事务、Undo、类/构造/接口/Graph和preview继续未接通。

[结果](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/result.json)及[布局测量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/layout-measurements.json)记录边界。4Rust/5保护源匹配，16定义0执行；原31节点ZUI未写。本轮无新服务/桌面/Cargo/Jenkins query/capture/freeze，passed/accepted=false。以下AB-D12/rev94及更早段落保留为历史时点。


## AB-D12新增观察及开放边界

[最终640展开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/viewer-640-open-final.png)和[900展开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/viewer-900-open-final.png)证明设计代理开关可达，无引擎命令。外部点击保持打开、同开关闭合、返回SHA相同；SVG g无键盘focus资格。自动nested-frame/错误manual原点/未settle高度、route及重复SVG前置失败均保留，只修真正弹层定位，不隐藏来源字段。

真实per-view publisher/query/context失效、Delete/history/retention、类默认值/构造/接口/事件图、实际preview与动作仍未完成。本轮Rust/ZUI/Shared/公共宿主变更0，16定义仍0执行。FontFace loaded与SVG/画布字形差异并存；无UE/EXE并排、diff、实际150%DPI或双端≤1DIP结果。

最新[紧凑设计12](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/result.json)为Actor rev94/803对象、40 Shared roots、579有效引用。640中心346/900中心606，沿原资产已有compact Details合同形成可编辑弹层和真实Penpot点击原型。旧AB-D11的640失败板/图不变。14新图逐张打开、累计87图；默认/返回图SHA相同。仍为只读fixture，键盘focus、字体外观差异、导出/EXE/DPI/双端及真实命令链继续open。

当前产品差异条目均开放。以下为同步10历史时点：当时66张Penpot浏览器图，Actor rev49/322对象、Shared rev5。原生Update后4个Button根取得真实Shared颜色引用，关闭context及fresh reopen保留；文字/引用/覆盖保留，但103对象发生native几何精度/文本缓存变化，最大源数值差0.3800048828125，不称exact preservation。实际颜色没变，非视觉A/B。此前颜色覆盖/撤销/重做、18:11 body保存未确认及字体失败、来源状态设计和公共内部宽度缺陷证据保留。仍无本轮UE原生截图、Penpot官方导出图、Editor原生图、并排图、diff或双端review。**双端**几何与文本误差均未测量，不能记为0；本轮源数据delta不替代双端测量。设计资源未就绪，不创建虚假的`LayoutRenderEvidence(status=pending)`：协议中的pending表示已经捕获但不可发布，本轮尚无产品捕获。

最新[同步10逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-sync-10/result.json)明确：三尺寸仍是consumer probe；内部240宽label不随root拉伸、树标题换行、Property必需value及完整窗口内容仍open。受控消费仅证明这4个Button根继承真实颜色引用，没有关闭本窗口族、公共pipeline或两族样式A/B门。

会话、能力核查和环境依赖见 [工作记录](actor-blueprint.md)，UE 源码 symbol 见 [窗口盘点](actor-blueprint-window-inventory.md)。下表“所有尺寸”指 1280×800 @1.0、900×620 @1.0、640×520 @1.0、1280×800 @实际 1.5；每个具体 case 仍须有独立 identity。

## 开放差异

| ID / 状态 | 窗口、场景、DPI | UE 依据 | 实际差异与已证实原因 | 依赖 owner / 处理建议与待决策 | 实际证据 |
| --- | --- | --- | --- | --- | --- |
| AB-001 / open | Components/Prefab，默认/空/选择，所有尺寸 | `SSubobjectEditor.cpp` tree/selection/filter；`FConstructionScriptEditorSummoner` | 原资产固定行/统计未成项目投影。Parent 已接实际 view/template/capability及callback后guard；source/batch仍无逐view生产发布。本域已有UUID目录+驻留Prefab snapshot/key查询，本轮加入真实来源实体只读投影，保留全部记录并诊断重复ID/缺失/歧义/循环父级；没有host调用者/producer。实体parent不等于组件attachment，来源到实例映射/SCS与override执行尚缺，不能按文案或source_index派发。 | Parent交同项目registry/resources及per-view完整上下文，消费AB-PF-15/17后按原key重校验；持久target、retention/cancel和写factory/history仍按AB-PF-07～16接续，不扩trait/store/时钟。 | [source-tree-08](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-08.json)；4项独立host/当前16项Prefab定义均0执行，1项原性能门默认ignored；旧#348/8项及source-adapter-02/13项保留；双端图片缺失 |
| AB-002 / open | Class Defaults 与场景 instance details，修改/reset/reopen | `FDefaultsEditorSummoner`、`SSubobjectInstanceEditor.cpp:700` | 本域已增加借用真实 PrefabInstanceAsset 的只读投影，保留原序覆盖值及所有冲突行位置，区分未检查/可用/缺失来源；尚无生产文档注入、写事务或 UI 绑定。类默认值、父类继承及 reset 仍未接通。 | 本族与 runtime/asset owner；Parent 对接脚本类后端并交真实文档/revision 读取 handle。保留类与实例两个上下文。 | authoring/snapshot.rs 和三项回归测试源码；静态检查通过，Jenkins 聚焦执行待回执，真实类/实例图缺失 |
| AB-003 / open | Add/Delete/Duplicate/Rename/Reparent/MakeRoot、undo/redo | `SSubobjectBlueprintEditor.cpp:333,576,646,716`；instance 对应实现；`SSubobjectEditor.cpp:1930` generic commands；SubobjectData.cpp 的 CanDelete/CanReparent/CanRename | 已有scene Add/history及Add undo：remove_created_dynamic_components对全批value/creation receipt预检后移除本命令创建的未变组件。runtime通用remove存在；任意既存组件的Editor Delete/journal/inverse/retention/多目标原子性仍缺，不能混为全无撤销或一键Delete完成。Arc创建证明不作持久行ID。 | Parent核心层接通用Delete和history/恢复；本域消费真实document/entity/type、模型组件身份与独立代际。entity.parent不作component attachment，Make Root/Reparent缺模型时不可用。Add的已应用后网关失败按真实回执处理，不把Err一律当零变更。 | [target-contract-03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-target-contract-03.json)、AB-D03/AB-PF-16；源复核与格式检查不代替命令/EXE执行，条目保持open |
| AB-004 / open | Construction Script / Event Graph / Level Blueprint，编辑/执行/保存 | `FGraphEditorSummoner<UEdGraph>`、`GraphEditorActions.cpp`、`BlueprintEditor.cpp`、`LevelEditorActions.cpp:2951` | 尚无已确认的本族可编辑图资产与执行链；关卡脚本绑定与 Actor 类图需分流。通用 Graph descriptor、材质/动画图或画线不构成该能力。 | 材质牵头图表面、Parent 跨域合同/集成；本族/runtime 脚本 owner 明确 typed pin、call/event、构造时机、关卡生命周期、持久化与事务合同。 | graph foundation、descriptor 与 type registry；具体消费者合同已提交，实际图和执行截图缺失 |
| AB-005 / open | Blueprint Interface、Class Settings、Macro/Function Library | `RegisterApplicationModes`、Interface/Macro layout、toolbar class settings | 接口签名、父类可变性、纯库和 Actor 模式分流未实现核查；不能显示不可执行的 Components/Viewport。 | Parent/VM/asset owner 确认可用 schema；保留 UE 目标设计和待实现边界。 | UE modes 源码；Penpot/产品资源与行为证据缺失 |
| AB-006 / open | 编译/dirty/success/warning/error、结果点击定位 | `FCompilerResultsSummoner`；`CompileBlueprint` F7；`SBlueprintEditorToolbar.cpp:496` | 缺本族编译请求、状态、error source map；现有 Validate 按钮返回 preview。不能用固定“编译成功”填补。 | 本族与脚本编译 owner；复用 WorkbenchDiagnosticRow/任务服务，绑定真实文档/节点/引脚 ID。 | preview feedback、只读 plugin test；真实编译和跳转证据缺失 |
| AB-007 / open | 从选择创建类资产、主场景 Actor → 资产文档 → 保存 → 回场景，双文档 undo | `CreateBlueprintFromActorDialog.cpp:181,877,914`；`InitAssetEditor`、`FDocumentTabFactory`、`JumpToHyperlink` | Prefab create 命令未注册；UE Subclass/ChildActor/Harvest 三种创建语义未映射。已有通用 view OpenView，资产上下文 open 仍未注册；canonical 插件 UI 文件缺失、原 page/index 与 native pane 旧 source suffix 不一致，实际打包/remap 未验证。UI asset session 的 undo API 不能冒充 prefab API。 | Parent 文档/Dock/原 pipeline owner + 资源会话；先核实际 package/source/instance 闭包，再由本族接入真实创建/替换/保存与失败原子性，保持 selection/document identity。 | source-integration-audit-01 的 AB-LOAD-01…03、只读 plugin test/navigation/spec；不是已观察 EXE 加载错误，实际创建及跳转证据缺失 |
| AB-008 / awaiting-parent | 菜单/上下文弹层、快捷键焦点、关闭后焦点返回 | `SSubobjectEditor.cpp` generic commands/selection；BlueprintEditorCommands | 当前没有桌面队列和正常窗口；无法验证 Delete、F7、Ctrl+F、Home、PageUp/PageDown 等作用域及焦点恢复。 | Parent 共享菜单/Dock/focus owner；本族测试输入 scope，图/树快捷键不得落到错误文档或主场景。 | 用户回复等待 Parent；无原生输入或焦点 receipt |
| AB-009 / open | 真实 preview viewport、增删/变换/构造后反馈 | `FSCSViewportSummoner`、`SSCSEditorViewport.cpp` / viewport client | 现有 Prefab 工作区以 composition table 为中心；共享 WorkbenchPreviewViewport 的真实根是 Image，默认 animation_mannequin_viewport.svg 且输入关闭。静态默认组件不能算 UE Components 实时视口。 | 真实渲染 host/resource owner + 本族；AB-SHARED-05 请求 renderer/input/resource 合同，需同一 EXE 实际场景/组件 resource 和 present 证据。 | 原始工作区及 preview_viewport.zui；GPU/readback/真实窗口截图缺失 |
| AB-010 / open | 900×620、640×520，details/树滚动首尾、中文/长文本 | UE unified splitter/tab 关系 + Slate 测量；项目 flow/content 合同 | 原资产有固定 left/right 宽度、responsive tier 与 details drawer；无双端测量，不知隐藏/恢复、裁剪、缩进和文本行框是否忠实。 | 本族按共享语义尺寸/flow 修改；Parent 提供通用 Dock/窄窗策略，不以坐标偏移修图。 | 现有 `.zui`；所有几何误差/文本误差未测量 |
| AB-011 / open | 可编辑Penpot实例、共享样式、保存/关闭/重开 | UE toolkit/tab/style及UE/Penpot fidelity协议 | 最新Actor rev107/993对象、58Shared根；653引用=445Shared+208本域main，全有效；本轮802旧非根对象与2main精确保留、重开data全同。新增六来源状态板是unit设计，640断词/生产publisher仍open；rev94/803及579引用为AB-D12历史。后续已完成rev5真实Button Update消费；rev48/240引用为旧时点。单个Button颜色覆盖/Undo/Redo/fresh reopen证据有效；原body是另一目标，目前无Typography引用，18:11 saved/暂存仍未确认。原custom/normal-400与Shared gfont/regular不同；浏览器Roboto面loaded不等于provider/variant/CJK或Engine ready。早期07时点未点Update保留历史；10已消费真实main发布的Update并保留文字/引用/覆盖，native几何有delta，仍非全局样式视觉A/B。 | Parent/原public owner接后续真实style/export及字体provider；本域按实际affected component/revision/source消费并保留override。内部宽度、合法value件、codec/Dock继续open，不改公共main/guard或重复PNG404。 | [源对照07](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-compare-07/result.json)两新图及[leaf-style-05](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-leaf-style-05/result.json)；相同#c0c0c0无全局A/B视觉delta，旧失败保留 |
| AB-012 / open | Penpot 样式修改 → 导出 → 原 .zui → 产品重载变化 | `EditorStyle/Private/StarshipStyle.cpp`；共享 editor tokens/Starship | 没有完整传播实测。catalog/tokens 文件存在不能作为传播/自定义设置验收。 | 共享主题归 Parent；本族先验证自己支持的局部覆盖。全局修改必须由 Parent 统筹至少两个窗口族。 | 基线文件 SHA 已取；before/after Penpot 与 EXE 图均缺失 |
| AB-013 / runtime-dependency | 所有必需尺寸，实际 150% DPI，capture→passed→accepted | LayoutReviewCase、normal-frame evidence 协议 | 当前公开 capability ready/nativeLifecycleVerified=false。同一 Runtime 元数据请求三次提交前退出：字段预检、legacy source policy、full-file-index-v2 前后读取间两份动画文档变化；Cargo 未启动，query-only 返回 session authorization pending。无本域 EXE/资源/nonce/generation/hold、144DPI 和桌面时段。 | Parent/Jenkins owner 接续原 request/operation 的自动登记和输入核对，保留并发修改；原 pipeline owner 交 Actor normal recipe，之后排桌面逐 case 复核。 | runtime-prefab-metadata-01/02/03 原输入/日志及 02/03 query；不是 Rust 测试失败或通过；旧失败保留，0 capture/0 passed/0 accepted |
| AB-014 / inventory-only | 地形/植被/导航/Levels/日志/分析等 45 个候选条目 | 见 OVW 稳定条目与实际 mode/tab 注册；Editor 全目录字面入口扫描命中 60 个第一层目录、125 个文件 | Parent 已提供 45 项窗口级收件表；世界创作/导航/诊断等由 Parent 保留待接续，不能写成另有实现聊天。归属不代表后端或窗口实现。 | 以 [Parent 覆盖对账](parent-window-coverage.md) 接续；Actor 收件 OVW-45 Editor Utility，Widget 协作内容，Parent 对接 VM/宿主。本族不抢占其他 owner。 | 已有源码盘点和统筹收件表；实现验收均缺失 |

## 必需视觉环境

`LayoutReviewCase.viewport` 用逻辑 client size，`dpi` 为 scale factor。最后一行须检查目标正常窗口的实际 DPI 和 logical/physical client size；更改图片大小或浏览器 deviceScaleFactor 不能代替 Windows DPI 验证。若正常宿主对 viewport 采用不同定义，先按现有协议对齐并记录，不悄悄改验收尺寸。

| 环境 ID | 逻辑 client size | dpi | 需要的真实证据 | 当前状态 |
| --- | --- | --- | --- | --- |
| AB-V1 | 1280×800 | 1.0 | 实际窗口 client、source identity、完整图、case/hash/hold | 未执行 |
| AB-V2 | 900×620 | 1.0 | 面板溢出/恢复、菜单锚点、滚动 owner、同尺寸 Penpot 官方图 | 未执行 |
| AB-V3 | 640×520 | 1.0 | 组件树与 details 可达、焦点/遮挡、同尺寸双端图 | 未执行 |
| AB-V4 | 1280×800 | 1.5 | 实际 HWND DPI 144、scale=1.5；逻辑尺寸与实际物理 client（预期 1920×1200）对应；新 capture identity | 未执行 |

## 业务与状态场景

以下是场景要求，不是假造的 LayoutReviewCase 实例或测试通过回执。取得实际资源和生产数据后，由既有 producer 生成 `LayoutReviewCase`，每行在四个环境中执行适用状态；不对尚不存在的 graph node/控件编造 source ID。

| 场景 ID | 操作与真实数据来源 | 必须核对的结果 |
| --- | --- | --- |
| AB-C01 default | 打开实际项目的 prefab/class 资产 | 真实资源 URI、类型、组件树、默认值、文档标题/dirty 状态一致；无固定演示统计 |
| AB-C02 empty | 新建合法空资产、空组件集合；过滤无结果 | Root 的必需约束和“无匹配”区别明确；零数据来源真实；未选择时属性禁用 |
| AB-C03 selected | 鼠标/键盘选择组件、Ctrl/Shift 多选 | 相同稳定 item ID 关联树、属性、视口；selection 与 focus 不混同 |
| AB-C04 focus-popup | Tab/Shift+Tab、树/字段 focus；Add/context menu；Esc/选项提交关闭 | 焦点环、键盘 scope、popup clipping 与 z-order；关闭回到真实 trigger/fallback，不落到主场景 |
| AB-C05 scroll | 增加生产可序列化组件和长属性集合；分别滚动树与 details 到首尾 | 独立 scroll owner、稳定行身份、选择不漂移、没有多重滚动/重复项；`scroll-before/after` 与 scrollPosition 一致 |
| AB-C06 locale-long | 给真实资产/组件/变量命名中文及足够长文本，再保存重开 | `locale` 与内容来源明确；字体 face/glyph/resource readiness；文本行框、截断提示与编辑值无丢失 |
| AB-C07 add-remove | 从真实可创建 catalog 添加；拒绝重复/不可创建；删除/恢复 | 组件 provider/schema/instance ID、事务 journal、资产 dirty、视口与树更新；失败原子性和 undo/redo |
| AB-C08 parent | 拖动/命令修改合法父子；尝试自身/后代循环、不可变继承根 | 拒绝原因、world transform 规则、根替换限制、序列化和 undo/redo 一致 |
| AB-C09 class-instance | 改类默认值与单个实例 override；reset/保存/重开 | class 和 instance 的数据/dirty/history 分离；覆盖与继承关系真实；不能影响错误实例 |
| AB-C10 graph | 真实图节点添加、typed pin 连接/拒绝、断开、删除、移动、框选、缩放、上下文搜索 | 模型与连线/节点一致，执行结果可观测；持久化、撤销和错误节点跳转有效；不是装饰性画线 |
| AB-C11 construction | 修改组件/构造逻辑，执行真实重新构造 | 被编辑资产与对应实例变化可证明；world/project identity 正确，无静态预览替代 |
| AB-C12 interfaces | 添加/移除真实 interface 与函数签名，错误签名编译 | 正确 interface mode，不出现无效 Actor 控件；依赖传播、错误反馈和 undo 有效 |
| AB-C13 compile | dirty→compile success/warning/error；F7；点击 diagnostic | 状态来自实际编译器；真实 node/pin/source map、相应 focus、可复现失败；不伪造成功 |
| AB-C14 navigation | 从已选 scene Actor 打开相应 asset editor；打开第二资产；保存/关闭/重开/回场景 | Asset URI、document tab、selection、undo context、焦点恢复、实际持久化与预览同步 |
| AB-C15 docking-menu | 关闭/重开面板、切换 graph/viewport tab、restore layout、menu/context commands | Dock lifecycle 和脏文档行为正确；关闭的可重开 tab 未被永久删去 |
| AB-C16 style-loop | 在真实 Penpot 编辑支持的样式，保存/重开/导出并应用，再正常产品重载 | 稳定组件引用/控件/事件/绑定未损坏；before/after 资源哈希与截图对应变化；公共 token 仍由 Parent 维护 |
| AB-C17 create-from-selection | 真实选择单个/多个 Actor，打开创建对话框；测试可用模式、非法/重复名称、路径取消、提交失败和成功 | 模式由真实能力过滤；取消不变更场景，失败不留半成品；创建/替换与 undo、资产持久化和选择结果相符；关闭回到正确 trigger |
| AB-C18 level-script | 若 Parent 确认可用关卡脚本，从当前关卡入口打开、绑定实际 Actor、保存后卸载/重载关卡，再处理引用失效 | 关卡/脚本/Actor identity 与运行生命周期一致；不显示无效组件类操作；无真实后端时记录待定，不能用普通事件图代替通过 |

## 证据格式与状态约束

直接复用 `tools/penpot/apps/zircon-zui-plugin/tools/zui-layout-review-contract.ts` 的 `LayoutReviewCase`、`LayoutRenderEvidence`、`LayoutCaseReview`、`CatalogReview`，以及 `tools/analysis/visual/record-editor-normal-window-evidence.py` 的正常窗口 adapter。不要添加本族平行的“视觉通过”格式。

- case：sourcePath、host=editor、viewport、dpi、locale、state、实际 data、themeSourcePath、必要 reviewHost/hash；动态 owner 使用实际 sourcePath/sourceNodeId/instancePath，重复行还需真实 itemKey/propertyFieldId。
- 双端：同 caseSha256、源与依赖 SHA-256；真实 Penpot browser/runtime、save/reopen/file/page/component version；实际正常 EXE SHA-256、PID/creationTime、rendererKind=`zircon-editor-retained-host`。
- 正常窗口：完整原生捕获入口、进程/窗口/nonce/generation/hold 对应、正常 GPU present、真实字体/图标/媒体 readiness，不能用 headless fixture 或邻近窗口图顶替。
- `passed`：资源已就绪、捕获有效且 hash/source association 合法；`accepted`：双端比较及逐图 review 通过。caseReviews 必须给具体 observations、maxGeometryDeltaPx、textMatches、visibilityMatches。
- 关键边界与文本行框目标 ≤1 逻辑像素；保留 full images、side-by-side、diff、逐 case review、失败原因及未测量字段。一个 case 通过不升级整个窗口族。

当前统计：原五个区域flow板、业务mains/12copies及旧四件验证板保留，新增三个尺寸probe/9个linked copies，最新rev30/241对象；46张浏览器原图有逐图记录。中文、根fill及关闭重开已验证，内部固定宽与树文字换行已复现；探针不计完整窗口。必需value槽、公共copy/source/allocation映射、共享style、官方ZUI导出/真实产品加载仍缺；原#348准备冲突前未Cargo、8项测试/0执行保持。normal capture0、双端图0、passed0、accepted0；旧失败和126B清理项保留。
