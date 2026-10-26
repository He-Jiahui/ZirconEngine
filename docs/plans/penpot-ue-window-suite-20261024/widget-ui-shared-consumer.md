# Widget/UI 共享库受控消费

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

## 七张 Toolkit 板的真实 Shared 颜色引用 r01

22:02Z沿原file/page将**147个既有非组件flow容器**的背景经原生`LibraryColor.asFill()`引用Shared rev5四种已发布颜色：surface.0（42）、surface.1（70）、border（14）、surface.input（21）。具体UUID、原目标ID及实际resolved值见[完整前像](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-shared-surfaces-r01/before.json)和[消费回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-shared-surfaces-r01/receipt.json)。只写原147 fills及7条明确localAssignments的域journal，无创建/删除对象、改main或另建主题；原historical theme-source与canonical source metadata保留。

实际update-file保存**rev21/154changes**，原始request/response均保留，生产`verifySaveResponse`按file/session/full changes/revision验证；随后关闭、fresh context重开，同file/page仍rev21/2978objects/8 local components，saved/reopened完整data相等。只移除这147 fill ref及7 journal后，整个data与rev20深比较相等；2831个非目标对象、12 Shared heads/99 shapeRefs的完整子树及所有原文字/font/position-data/source/几何不变。147个SDK几何亦相同；Shared完整数据及插件注册表未变。它证明本域显式颜色引用持久化，不是主件默认继承或全局A/B通过。

[七张实际图及逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-shared-surfaces-r01/visual-review.json)已全部打开：1280空文档/搜索空结果/焦点/anchor夹具、900空态/中文和640 opened Dock均保持。640搜索文字及中文长查询仍右裁切，窄页签省略、More/独立滚动/焦点返回未验证；UE chrome/图标/密度仍有差异。蓝框、No matching widgets、16预设和空Designer分别是设计状态/夹具，不替代实际输入、筛选、Popup单undo、实时Preview/Graph/动画。真实色引用不解决这些缺口；沿公共Search/Tree/typed slot与源码codec owner，保留稳定实例，不加私有覆盖来关闭依赖。

[源与身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-shared-surfaces-r01/source-inputs.json)、[本轮摘要](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-toolkit-shared-surfaces-r01/summary.md)记录当前22只读/保护输入、3自有JS语法检查及原服务出生/EXE/deadline。原产品v2/97唯一controls/6events、两原测试identity保持，未运行新QueryOnly/Cargo/受管插件或正式导出；旧四字节null失败和未应用PNG分支提案不变。本轮PNG为1600×1100/dSF1、fit zoom71/98/134%的真实Penpot浏览器画面，非官方Shape PNG、目标客户端、正常EXE或实际150%DPI。正式导出/产品加载、双端逐图差异与≤1DIP均open，整族passed/accepted=false。


## 文字度量与字体消费复核 r04

2026-10-24T21:09Z以原认证在独立headless context重开同file/page，Widget保持**rev20/2978objects/8local mains**，完整decoded数据SHA与保存前像相等；0条update-file、0项设计写入。Shared独立读回仍rev5/41objects/4main/34colors+3typographies；此次没有sync、改库、改原实例字体或尺寸。4份原engine-face WOFF均实际GET200、缓冲SHA匹配并在该context加载。测量用临时FontFace别名和无textLength的SVG随context销毁，不是Penpot样式、主件或第二套产品字体系统。

此前本域r03把SDK `textBounds`写成glyph bounds不准确。实际`plugins/text.cljs`→`geom/shapes/text.cljc::shape->bounds`计算的是已保存position-data矩形，不是字形墨迹。原报告/JSON字段及失败保留，当前解释按下表。单位为本次Penpot/SVG坐标，**不作为正常EXE的DIP误差**。

| 原实例文字 | 文字层高 | 保存位置框高 | SVG getBBox高 | Canvas字形墨迹高 | 父实例高 |
| --- | --- | --- | --- | --- | --- |
| 主视图 / TreeRow | 16 | 19 | 19.518736 | 13 | 28 |
| 搜索控件 / SearchInput | 16 | 19 | 19.518736 | 13 | 32 |

两条文字各记录47层SVG祖先直到`#render`。其中4个真实clip矩形均包含完整SVG BBox，文字层没有自身clip；原图逐张复核未见这两条中文被裁。`#render`有viewport overflow hidden，结论只覆盖本次可见验证板。因而19/16本身不证明裁切，也不据此加高共享字段；七板640/长查询裁切仍按原G09开放。Noto SC的三条样本文字在当前CSS family与已注册引擎face的自然advance相等，只说明这些样本，不证明字体字节、fallback/atlas、UE或引擎呈现一致。CDP名称中的Thin不能单独证明实际字重错误。

`svg_text.cljs`实际将保存宽度写成`textLength`，并设`lengthAdjust=spacingAndGlyphs`。原SVG宽度吻合保存值会掩盖自然字体差异。移除该约束的临时测量中，Roboto400长文本`Widget Hierarchy / Canvas Anchors / Property Binding`当前family宽320.125，原engine-face宽323.859375，差**3.734375**；Button短词差0.96875。Noto SC样本差0。该对照是浏览器自然文字测量，未给原节点去掉textLength，也不是UE/EXE误差。

公共owner/Parent的精确依赖是沿既有`penpot-font-selection.ts::selectRegisteredFont`和原font-resource-manifest核对family/weight/style→已注册fontId/variant，并核实实际绘制face及保存position-data的重算。原body/caption/strong三个typography UUID继续保留；当前body/caption为gfont-roboto/regular，strong为gfont-roboto/700。`library.cljs`已有`fontId/fontFamily/fontVariantId`受library:write约束的setter，但Shared写入和canonical发布仍由原public owner执行。Google/custom同family CSS同时存在与本次advance差异吻合，具体实际face选择原因仍待该owner核实，不把这一推断写成已修复的字体后端故障。

原测量helper首次把浏览器Response.status属性当方法而失败，0条probe/0写入；后续修正及原失败hash完整保留。r04b/c祖先上限14不足以涵盖frame clip，不用它们证明无裁切；最终r04d完整至render并发现4个clip，均做BBox包含检查。14只读输入、4保护资产、2原验证identity已记录；无Rust/托管插件/正式导出、服务生命周期、桌面或DPI动作。产品passed/accepted仍false。

证据：[最终实际测量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-styles-rev2-consumer-r02/cjk-bounds-r04d.json)、[原始浏览器图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-styles-rev2-consumer-r02/cjk-bounds-r04d-browser.png)、[源码与资源指纹](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-styles-rev2-consumer-r02/cjk-bounds-r04-source-audit.json)、[逐图和度量复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-styles-rev2-consumer-r02/cjk-bounds-r04-review.md)。

## Shared rev5 消费接续 r03

2026-10-24T20:40Z原生GetProcessTimes/exe SHA独立核对Parent当前5进程均匹配，未重启或延长原deadline。20:42Z原认证API/readyz成功，Widget为rev17/2978objects/8local mains，Shared已是**rev5/41objects/4原主件/34colors+3typographies**；旧rev2和0个master绑定均保留为各自时刻历史。当前Button main `6ddc248b-bd35-80c5-8008-d6c8bd9b057f` 的fill真实引用Shared `editor.surface.1` / `599586e3-022c-500b-ac80-5d9c490139af`，即1组ID/file、2个非空字段；不能外推其它主件或全部默认样式已绑定。

操作前保存Widget/Shared/libraries完整原始Transit及完整decoded文件，覆盖全部目标和非目标对象、原12个head及99个实例后代sourceRefs、中文content/font/override和position-data。全部文件有2403条shapeRef，**本受控12实例是其中99条**，计数范围分别保留。真实Libraries→Updates列出1个WorkbenchButton；只点击原Shared的更新，实际update-file ACK保存rev18、synced-at为20:45:28Z。后续新context的Updates为空，未修改modifiedAt或强行再次同步。

rev17→18仅3个原Button根与1个body文字对象改变。最初严格content比较因此停止，原failed回执及当时16字体缓冲上限失败保留。精确差异是body Button的`content/children/0/children/0/plugin-data`中`dev.zircon.zui`→`shared/dev.zircon.zui`；token-source字符串完全相同，正文、全部字体字段、typography引用及行框无变化。真实`common/logic/libraries.cljc`的typography同步会merge原样式字段，符合此次命名空间替换；这是源码解释，不把raw content说成逐字节未变。后续比较只对这一对象、这一条等值移动单独分类，其它任意content变化仍拒绝。未写回metadata或改公共guard。

沿同一真实页面的原实例继续窄颜色消费：原Widget内容覆盖Button `485c980b-5100-806a-8008-d6e3fcc16075` 使用已发布surface.0 / `5edd281a-cc68-597c-a6fa-1234e164adc8`，保留Label及所有来源；原reset Button `...d6e4018e3a31` 使用accent / `b3f68044-6a40-566a-9d34-fdb415e552e8`，实际保存rev19。只对reset原实例调用原生resetOverrides，保存rev20，恢复当前main的surface.1 ref及#242424；中列surface.0 / #151515覆盖保持。关闭authoring context，再以新认证context重开同file/page，**rev20/2978objects/8local mains/12原heads/99sourceRefs**完整，Shared数据及注册表未变，没有detach、复制main或新建主题。

相对rev17最终仍只有4个对象变化；2843个板外对象、所有原component/main/ref/parent/children及节点metadata保全，正文/字体与4份body引用保全，原12条SDK textBounds和position-data差值为0。这里只是同一Penpot文件的前后保全，**不是UE或产品≤1DIP验收**。原r03曾将此SDK值误称glyph；两条中文的保存位置框19高、文字层16高，已由上节r04区分墨迹/保存框并检查实际clip，不继续据19/16断言裁切。字体消费和产品中文fit仍open。

8份新原始浏览器图和2份仅Penpot衍生比较图已逐张打开。实际像素采样：中列fill从36/36/36→21/21/21且fresh重开保持；右列36/36/36→0/112/224→36/36/36。截图为1600×1100/deviceScaleFactor1、画布108%，不冒充目标客户端尺寸、正常EXE或150%DPI。板上旧“Shared rev2/master refs empty”文字保留为旧r02标签，本轮scope没有改其正文，当前真实库/引用以本节及机器证据为准；PropertyRow.value仍为空，不能据这张板关闭Details/槽位缺口。

新浏览器实际保留121份font response，Roboto400/700的自定义WOFF与原engine-face注册SHA匹配；另1份response在context关闭时取消，旧失败不改写。独立当前认证GET再实际取得NotoSC400及Roboto400/500/700四份WOFF缓冲，4/4与原TTC提取/注册对应SHA匹配；这是当前资源可取证据。它不把CJK当前gfont、Shared body的gfont-roboto与engine custom face视为同一实际绘制选择，也不证明fallback/atlas/引擎字体或1DIP。

精确回执：[首次Update及停止](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-styles-rev2-consumer-r02/native-updates-r03-receipt.json)、[受控颜色/Reset/重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-styles-rev2-consumer-r02/native-updates-r03b-receipt.json)、[完整前像](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-styles-rev2-consumer-r02/current-preimage-r03.json)、[逐图及失败复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-styles-rev2-consumer-r02/native-updates-r03-review.md)、[Penpot前后图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-styles-rev2-consumer-r02/native-updates-r03-penpot-only-comparison.png)、[当前资源字节](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-styles-rev2-consumer-r02/native-updates-r03-font-resource-readiness.json)。5份自有JS语法检查及文档/源身份核对不替代托管插件/Rust测试。原97controls/6events、4保护资产及两原Jenkins identity保持，本轮无Cargo、正式ZUI导出、服务/桌面/DPI操作；两族全局A/B、正式codec/slots、七板消费、正常EXE/双端比较和整个窗口族passed/accepted仍false。

## rev2 接续：真实样式消费与绘制核对

2026-10-24T19:07Z 独立认证 API 读回 Shared 原 file/page `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` / `...ac13` 为 rev2/isShared=true、4原主件、34 colors、3 typographies。Parent 已纠正旧“main内21处实际引用”的计数：21个引用ID及对应file共42字段均为null，实际非空主件绑定为0；本轮独立读回亦为0。旧回执保留，不继续称其默认样式已绑定。body=`99eb86f9-12ff-5935-96ad-3d1a985e61f0`、caption=`c0224e78-f918-526f-8026-a76cd8f347af`、strong=`9ac86733-8214-5a04-820a-9074b5c68310` 均沿原发布身份，无Shared写入。

原18:12:05Z拒连失败和rev1/零样式记录保留为各自时刻的历史。本轮复用原账户/storage、已分配Widget文件/page和原验证板 `485c980b-5100-806a-8008-d6e3f7ef5ff8`，独立API/headless就绪后接续，未启停服务、使用桌面或修改DPI。19:37:48Z最终保存关闭fresh-context重开为 **rev17/2978objects/8local mains/12原linked copies**；与原rev13比较仅21个验证板对象变化，板外对象和全部本地主件定义未变。

先查看原生Libraries→Updates，实际无可用更新。随后在原默认列的4份copy显式应用Shared body，原4份reset copy试用caption并执行原生reset，内容/CJK列保留。保存rev14后，原生Updates才出现“1 typography/editor.typography.body”；点击原Shared Update实际保存rev15，库synced-at更新为2026-10-24T19:14:08Z并跨fresh-context保持。此结果证明本域已用样式的同步持久化，不证明主件默认样式或两族全局A/B传播。reset恢复原主件，其排版ref仍为空；不可把空引用说成已继承Shared样式。

发现并保留真实绘制差异：SDK文字/排版写入后，rev16的content已是“Shared body override”，position-data与SVG却仍显示“Shared default”；body Button内容权重400而SVG仍700。独立字体等待和重开未消除此差异。实际 `plugins/text.cljs:479` 的非WASM characters分支仅更新content，WASM分支另做resize。最后通过正式SDK selection→Enter→实际DraftJS `role=textbox` 原生文字编辑，对同一标题和原Button提交相同内容，生成rev17。保存ACK及重开后SVG为新标题/400字重，4份body引用和CJK覆盖保持。没有直接写position-data；**公共SDK/插件自动重算仍open**，本板的原生重算不能替代公共修复或其他板验收。v2测试标记、SVG命中与嵌套editable选择等失败均保留，前5次原生编辑探查没有文本输入或保存。

当前原默认列标题已实际显示“Shared body override”，4份copy的真实source file/body ID及全部component/main/shapeRef/child/plugin metadata保留。PropertyEditorRow.value仍为空：7类合法字段尚未作为本板可消费主件交付，不塞Text占位。复位/样式是此验证板的窄消费结果，七张窗口布局的Tree/Search/Details语义区域尚未正式替换或导出。

保存证据有明确缺口：rev14原始请求/响应被后续独立观察器的同名序号覆盖，原校验回执和SHA保留；现同名原报文属rev15，已逐字节核实。rev16、17改用独立空目录，实际请求/响应及严格identity/revision校验完整。库状态HTTP观察器只匹配rpc路径，实际UI走main/methods，故未捕获原状态ACK；持久化synced-at已实核，不重放更新或伪造报文。旧failed不改passed。最终readback历史basename为`instance-style-readback-native-edit-07.json`，实际首次成功写入者是attempt12/rev17，按真实身份追踪，不重签旧case。

本轮9份实际扩展名字体响应含Roboto/Noto Sans SC及Penpot界面字体，另138份font/by-id资源缓冲均保留实际SHA；by-id资源未全部分类为字体。它们不证明引擎editor-ui.ttc相同、fallback/atlas就绪或中文1DIP。当前catalog为9556e66d…/71项，和首轮e4bba…的并行变化原样保留；NumberField原10props、PropertyRow原4slots/7类value接受项的14项源契约重新核对通过。catalog、tokens、Starship、原ZUI及Rust本轮没有写入；原v2/97唯一control IDs/6events保持，gap6仍仅上一轮局部导出实验。

[完整机器核对](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-styles-rev2-consumer-r02/resume-result-audit.json)、[逐图与失败记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-styles-rev2-consumer-r02/resume-review.md)、[最终实际Penpot截图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-styles-rev2-consumer-r02/final-native-text-edit-12-browser.png)和[仅Penpot前后差异](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-styles-rev2-consumer-r02/penpot-only-image-comparison.json)是本轮证据。官方ZUI全量导出/产品加载、公共自动重算、typed value槽、CJK/引擎字体、Jenkins产品Rust、正常EXE/144DPI及双端逐图/1DIP全部仍open；passed/accepted=false。

### 预构建 receiver 接续边界

原Jenkins owner的18:13:58Z对账已核实，详 [原入口记录](widget-ui-validation.md#预构建-penpot-入口原-owner-对账仍-unsupported)。当前e0635d5d…胶囊没有Penpot/headless recipe；原mjs固定Level owner、非空页原成功回执、临时pluginServer及认证state持久化等行为，不能由简单receiver包装消除。新action/schema未实现、不可提交，不用Cargo/generic shell/Editor preview替代，不重建worker或改旧sealed。

此前本域Shared原生SDK同步候选仅语法检查，不能作为该新受管receiver或正式source/export链；本条handoff不启动它。服务readiness只是连接前提，不能解决上述二进制/谱系约束。12个原copy身份及两条状态标签的候选delta保留，实际current membership/写集尚未取得。局部source specimen的既有SDK保存事实另已按原字节核对，未变更旧failed回执；它缺旧mjs的workspaceUrl，不提升为完整runner/native成功。

## r01 历史保存记录

2026-10-24 接 Parent 的 Shared rev1 更新，在原 Widget 页新增一张独立验证板。正式库连接、12 个跨文件原生 copy、文字覆盖/原生 reset、保存关闭重开已实际完成。此板验证来源和消费合同，不替代原七张 UE 窗口目标，也不证明搜索、选择、插入控件或 Details 编辑执行。整族 open，LayoutReviewCase passed=0、accepted=0。

## 实际身份和写入范围

| 项目 | 实际记录 |
| --- | --- |
| 文件 / 页面 | `b4a1e3ce-9e6a-81fe-8008-d69d60d24223` / `b4a1e3ce-9e6a-81fe-8008-d69d60d24224`；[原 Widget 页面](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60d24223&page-id=b4a1e3ce-9e6a-81fe-8008-d69d60d24224) |
| 验证板 | `485c980b-5100-806a-8008-d6e3f7ef5ff8`，816×550，三列 flow：Shared default / Widget content override / Override → native reset |
| 保存 / 重开 | revn9→10 / vern0；2843→2978 objects，仅新增135；原8个 local components 保持 |
| r01 当时 Shared | file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` / page `b4a1e3ce-9e6a-81fe-8008-d69d607aac13`，rev1 / isShared=true / 41objects / 4mains；get-file-libraries 实际返回此库 |
| r01 当时样式 | Shared colors=0、typographies=0；复用既有 tokens 输入与已注册 Roboto/Noto Sans SC，无新增主题或样式 UUID |
| 变更边界 | 新板、子对象及本页自有 journal；真实保存930条 changes，仅新增135个 shape IDs，0既有内容对象/组件写入。原2843对象哈希、8组件定义和 Shared 全41对象/4组件均不变 |

r01 当时的发布状态已覆盖更早 rev0 空库结论；这些时点作为历史保留。当前发布以本文 rev2 接续为准。本共享消费轮保存时，七张完整目标为原rev9内容；之后Resize Slot的rev11与28个被动页签的rev12修订已保留，当前文件2978objects/8local components。两次修订未改变本验证板的12份Shared copy或来源映射，该次观测 Shared 为rev1/41objects。七板语义Tree/Search/Details区域尚未正式替换或导出，原Button prototype页签不计公共Dock实例复用；最新七图及精确变更集合见[rev12记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-tab-overflow-r01/review.md)。

## 来源、slot 和实例

真实来源为 Shared 的 `shared/dev.zircon.zui` / `canonical-source`、`canonical-record`。copy 保留 sourcePath/sourceNodeId/instancePath；下表 main UUID 即 copy 的 shapeRef。没有 detach、复制 main 或新建本地主件。

| 组件 | 发布 component UUID | main UUID / shapeRef | 原源资产 |
| --- | --- | --- | --- |
| WorkbenchButton | `6ddc248b-bd35-80c5-8008-d6c8c01096f3` | `6ddc248b-bd35-80c5-8008-d6c8bd9b057f` | [button.zui](../../../zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/button.zui) |
| WorkbenchSearchInput | `6ddc248b-bd35-80c5-8008-d6c8c3ce892e` | `6ddc248b-bd35-80c5-8008-d6c8c0f9c196` | [search_input.zui](../../../zircon_editor/assets/ui/editor/workbench/primitives/inputs/search_input.zui) |
| WorkbenchTreeRow | `6ddc248b-bd35-80c5-8008-d6c8c7a79944` | `6ddc248b-bd35-80c5-8008-d6c8c452bf0c` | [tree_row.zui](../../../zircon_editor/assets/ui/editor/components/workbench/primitives/data/tree_row.zui) |
| WorkbenchPropertyEditorRow | `6ddc248b-bd35-80c5-8008-d6c8cab1e7d0` | `6ddc248b-bd35-80c5-8008-d6c8c84d848f` | [property_editor_row.zui](../../../zircon_editor/assets/ui/editor/workbench/inputs/property_editor_row.zui) |

每项3个 copy，分别为默认、覆盖、覆盖后调用真正 `resetOverrides()`。文字覆盖：Button→Label、Search→搜索控件、Tree item→主视图、Property→Visible；reset 恢复四项原文字和 Roboto，新 context 重开后仍一致。全部12个 UUID、main/ref、18条节点映射、覆盖值见 [实例审计](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-library-consumer-r01/instance-reference-audit.json)，31条文本 bounds、23个 flow 容器见 [SDK audit](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-library-consumer-r01/native-audit.json)。

Button 的产品目标是原 `palette_label` / `PaletteLabel`，不映射为实际使用 IconButton 的 HeaderSave。Search/Tree 目标分别为原 `palette_search` / `PaletteSearchInput`、`hierarchy_root_row` / `HierarchyRootRow`；中文是明确的设计状态输入，不宣称生产搜索、选择或本地化接口执行。

PropertyEditorRow.value 必填、单值、linear，仅接受 WorkbenchCheckbox / Dropdown / Field / NumberField / RangeSlider / Slider / Toggle。这7种字段未在本次4主件发布中交付。三份 copy 的原 `value_slot` 来源保持、children为空，不能算完整 Details 或 slot 写回通过；未塞文字、放宽 accepts 或另造字段。发布字段也不是充分条件：当前 SDK 的 appendChild/insertChild 检查 `changes-component-copy-structure?`，禁止修改 copy 子树；本轮没有尝试非法改结构。公共插件仍需交付保留真实实例链接的合法 slot 作者/导出机制。

四源文件物理 SHA256 见 [来源核查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-library-consumer-r01/source-contract-audit.json)。发布 metadata 未提供 sourceHash，这些指纹只表示当前源码，不能证明 canonical 导出闭包一致性。正式 sourceHash/typed slot 写回/实例覆盖编码、全局样式 A/B 仍待公共 owner。

## 保存、画面和失败

复用本域既有 SDK 注册及原认证存储，通过隔离无头浏览器路由交付本域客户端；未新开服务/监听、改公共插件、账号注册或认证文件。实际执行官方 `connectLibrary`、`component.instance` 和 `resetOverrides`，不以伪造 JSON 后端直写。真实 update-file HTTP200 ACK 对应 rev10；关闭 authoring context，再由新 context 重开并以认证 API 复核。完整 [回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-library-consumer-r01/receipt.json) 保留请求/响应哈希，插件注册表前后哈希相同。

两次启动超时已保留：当前 ui.open 是 `(name,url,options)`，原客户端把 options 当 URL，实际请求 `/[object%20Object]`；恢复真实签名后打开成功。最初 handshake 假设未独立证实，不归责服务。第三次正式库连接返回时数据未可用，检查提前停止而未创建板；下一新 context 已加载真实库并以 get-file-libraries 证明链接。加载时序原因仅为按源码作的推断，没有声称修复了公共 SDK。

一次有界 Shape PNG export 返回签名 `6e756c6c`（null），无有效 PNG，失败保留。没有恢复 exporter 或以浏览器图替代正式导出门。[实际浏览器图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-library-consumer-r01/validation-board-browser.png) 为1600×1100隔离浏览器、deviceScaleFactor1、画布108%缩放；不是正常EXE、目标客户区或150%DPI证据。已打开逐图检查：三列、四行、中文覆盖及两条未完成说明均可见，默认和reset对应，Details value 明确为空。31条原生文字 bounds 有限；没有UE/产品对应图，不宣称≤1逻辑像素验收。

逐图检查与失败见 [review.md](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-library-consumer-r01/review.md)。UE区域/操作目标继续按 [具体设计](widget-ui-design.md) 与 [盘点](widget-ui-inventory.md)，验证板文字不驱动产品事件。

## 接续门槛

最小产品消费面为 Palette Search/Button、Hierarchy TreeRow 及 Details 的合法字段 slot。正式落回原资产前，由公共 owner 补 sourceHash/覆盖编码/typed slot 和非破坏式导出合同；保持旧有效 source 主件及97 IDs/6events。本轮未正式导出ZUI，原产品gap6、tokens、Starship、catalog及上一轮14个binding定位源码指纹均未变。

[正式导出前置核查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/shared-library-consumer-r01/export-preflight.json) 固定7个实际插件/SDK源码指纹与入口行。首个源码前置是 selectedAssetBoard 要求 role=asset 祖先；本验证板没有该role、bridge-version或可逆document。不能伪填metadata把probe变成原资产。后续仍须原shape guard、reconcileReviewSource及实际UI节点/组件调用映射；r01当时native shelf resolver/cache的观察保留历史。最新checkout已在native-components调用local+connected的canonical resolver，并在canonical-library支持source/sourcePath别名及歧义拒绝（当前cc3ac544…/8a366fd4…）；R3回执对应源码已漂移，尚无相同runner/UI/worker实际消费回执，不能从当前源码或发布事实推断产品复用通过。CanonicalComponentRecord现接口本身没有sourceHash字段，这是待补验收合同，不是本轮实测导出异常。以上均是源码核查，没有执行正式export-zui或宣称部署bundle通过。

本轮为设计/结构核查，无 Rust/Jenkins 新请求；原 layout/Canvas identity 与历史失败保持。原准入 captureReady=false、官方导出/PNG、正常EXE、桌面/150%DPI、四尺寸状态矩阵及双端逐图比较均 pending。发布、链接或局部reset不提升为产品或整族通过。
