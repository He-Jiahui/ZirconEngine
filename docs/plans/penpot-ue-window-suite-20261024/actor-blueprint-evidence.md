# Actor / Blueprint：首轮源码与环境身份

## 实现35：组合可见行及两项未执行回归

[authoring.rs:287](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:287)新增组合入口，[私有遍历:300](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:300)供原未筛选入口与新入口共同使用；原filtered方法体保持、当前声明在[250](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:250)，历史34的258仍按原hash解释。[新1670](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:1670)、[1745](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:1745)覆盖筛选后子标记/隐藏分支不查展开/完整u64/全匹配等价、歧义ID/切环及替换来源前创建的旧迭代器仍读原数据，原key明确拒绝Stale。

[格式](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-visibility-35/format-check.json) exit0；[四源guard](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-visibility-35/change-guard.json) exit0，patchDigest4aba383e7fea9834d76f49b05d67decc88d4d2152e6c598c6fe8b8ffd9e830db，0err/warn/suppression。cases5+snapshot37=42定义，41普通+1原ignored；移除两新测试后原40字节相同，逆向本次提取/新增后生产源匹配34 SHA。[审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-visibility-35/semantic-review.json)只签静态语义与字节，不签类型/借用或运行；Rust verified0/actual null。

首次apply_patch未写入，四源字节保持、文件非readonly；随后原SHA前检的canonical直接编辑写入并读回相等，未改权限/ACL/服务。首个旧测试重建脚本误用LF查CRLF而退出1，newline-aware重核匹配，诊断保留于原manifest。lib/snapshot、inventory与5保护不变；[结果35](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-visibility-35/result.json)保留原02受理终态，不Query/submit/Cargo/新ID。无helper/tmp/清理，释放0B。


## 实现34：筛选投影与三项未执行回归

[authoring.rs:258](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:258)增加一个原reader方法；[1480](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:1480)、[1544](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:1544)、[1596](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:1596)覆盖非序列化层次/中文/完整u64/父命中裁剪、重复ID与缺失/歧义/环边，以及borrowed opaque组件/Ready-empty/NotResident/同revision重建旧key拒绝。移除新增块后，原authoring与原37测试字节均重建匹配before SHA；lib/snapshot与5保护源保持。

[初次fmt](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-34/format-check.json) exit1的换行诊断保留，[最终fmt](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-34/format-final.json) exit0；[最终四源guard](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-34/change-guard-final.json) exit0，patchDigest cc7ffdbec98d53f0252f64fe2a130b3759cb5e5630b6d4fb0ce07819d4bc5637，0err/warn/suppression。cases5+snapshot35=40定义，39普通+1原ignored；仅语法/静态审阅，Rust verified0/actual null，不签类型检查或执行。

[只读引用](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-34/read-reference-pins.json)含真实UE TreeNode/SSubobjectEditor搜索及ResourceSnapshot Deref→原payload合同，不是完整依赖读集。[结果34](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-34/result.json)保留原02/queue15836/input23f4088d/#387 interrupted/resource历史；不query/重投、不把accepted19覆盖为40。本轮无Penpot/EXE/服务/桌面/Cargo/helper文件或清理，释放0B。


## 设计33：真实保存、重开、原生交互与逐图证据

[结果33](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/result.json)关联原file/page、source32输入、15root、组件/样式/override、40次实际update-file HTTP200、96文字和12导航读回及完整关闭重开。rev207→249／2861→4634；原2860非根精确、两个main形状不变，新增1773全在15root内，623flex；3720 refs=3220 Shared+500本域全部解析，新增96 Shared heads+36本域heads。不是Shared版本同步/reset或官方codec验收。

[保存保护](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/save-reopen-preservation.json)保留初始Date解码缺陷：before中的四个文件/组件日期及Shared43日期被解码成空对象，不能签最初完整日期元数据。旧shape无该损失；后续date-aware237参照和Shared5完整参照另存，save/reopen/final完整data相等，Shared5 data相等。原始baseline未覆盖或伪修。

[28图逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/visual-review.json)与[实际测量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/measurements.json)覆盖三宽三个状态、窄窗空Details、fresh及完整viewer。17种不同内容均已打开；其余按同PNG哈希复用。初始7px顶部过渡和绿色热点不作最终参照，稳定图等真实热点fillOpacity归零后捕获。三宽fresh/stable展开图字节相同；actual150%DPI及Editor比较仍无。

[原生跳转](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/prototype-transitions.json)有15次稳定pointer跳转、此前9次及两Details开关；只点击原native caret/row/toggle，未写store/派发假事件。caret中心因无paint不命中，随后点实际路径顶点命中，14×14 painted-only hit area仍是缺口。所有读取/输入/过渡失败保留于failure.json，无服务恢复或验收放宽。

自有helper正常exit0，14观察PID+birth退出、profile自动不存在、临时helper文件0；[closeout](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/process-closeout.json)只证明本浏览器，不替Jenkins/Engine native Job/EOF。手动释放0B，自动释放未量测；旧126B及26/28/30清理提案保留。原四Rust和5保护字节保持32，复用32 fmt/guard；37定义无Rust执行，本轮无Query/POST/Cargo/桌面/服务。


## 实现32：可见森林查询与三项未执行回归

方法在[authoring.rs:234](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:234)，使用私有resident source构造的既有hierarchy，输出原row类型；caller不能注入篡改的source_index。新回归在[1324](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:1324)、[1383](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:1383)、[1416](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:1416)，覆盖非序列化层次/完整u64/只查可见分支、全部异常根与切环成员有效子节点、Ready-empty→NotResident→同ID/revision重建旧key拒绝。

初次fmt exit1两处换行已修，[初始回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-visibility-32/format-check.json)保留；[最终fmt](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-visibility-32/format-final.json) exit0/4a8bcd，[4源guard](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-visibility-32/change-guard.json) exit0/45341f，0err/warn/suppression、patchDigest5038124454d867370ed0881f25e1f50f777860bcaa51e91fa46250b28f4bea9a。当前cases5+snapshot32=37，36普通+1原ignored，Rust verified0/actual null；未把静态定义当运行。

UE只读依据为[SSubobjectEditor::OnGetChildrenForTree](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:2045)及[SetNodeExpansionState](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:3071)，本域只对应数据/展开分离。原lib/snapshot与5保护不改；Penpot30归档仅复用，无新图/服务/桌面/helper/tmp/清理，释放0B。完整[结果32](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-visibility-32/result.json)与[有限后态](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-visibility-32/source-after.json)不构成完整member/writer/capture-ready。

## 实现31：原key检查实例投影与最小静态验证

[有限前态](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-instance-source-31/source-before.json)实读原16项均等于30最终；6参考源与29相同，另实读cases模块，非完整dependency/writer/source-ready证明。新方法156以实例自己的Prefab引用调用expected reader，再borrow原DTO；[语义审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-instance-source-31/semantic-review.json)记录Available和路径/写能力的严格边界，不把源码审阅当类型或运行。

三项新回归实际定义在[116](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:116)、[162](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:162)、[240](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:240)：源迁移/空源/全部opaque冲突值借用，异源/manager/registry/kind拒绝，以及同ID同revision重建后旧key失效。原31定义保留，lib两test module可达，当前34=33普通+1原ignored，0 verified执行/实际总数null。没有新fixture producer、文件夹具或SourceLocator/VM实现。

只读rustfmt首轮exit0/native714865，无后续Rust改动直接复用；最终4源guard exit0/dcd852、0errors/warnings/suppressions，patchDigest a1215b9021b93da32fd8eb1f3485cce993f10ffbe49097010dae84bc9f5c6af2。仅authoring.rs/tests推进，lib/snapshot及5保护保持，正式Rust/Jenkins收件仍pending。[结果31](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-instance-source-31/result.json)保留精确四SHA/范围与全部实际静态回执。

Penpot30的207/2861及10图仅复用归档，不声称本轮live读取/新增截图或生产连接。原#348/#387/02 accepted19/input23f4088d和历史失败保持，无QueryOnly/intake/Cargo/capture/冻结/服务/桌面/清理；无新helper或tmp，释放0B，旧profiles/126B仍沿原受管提案open。

## 设计30：原生保存关闭重开与10图复核

[来源与有限16文件前态](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/source-before.json)只声明本域11+5保护；非完整membership/writer/source-ready。4Rust/5保护保持29同源；静态31定义=30普通+1原ignored，Rust verified0/actual null，复用29格式与guard，不启动新Cargo/QueryOnly或02接续。

Actor176/2270→206初存→207最终存/全新context重开/交互后只读末次。新增591对象全部落在五个新root；2269旧非根对象与2main精确不变。2205 shapeRef按最近componentFile实际解析：356本域、1849 Shared，invalid0；copy heads为91本域/163 Shared，Shared5 data exact。89次原生完整文字编辑涉及87个新text，最终逐项exact；[身份与重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/save-reopen-preservation.json)、[原生动作](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/changes.json)保留完整映射，非官方export/source identity map。

[10图逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/visual-review.json)含3初存+7最终图，全部实际打开；三宽native viewer100%/DPR1，初存默认图与fresh默认图逐像素相等，只证明同Penpot默认视图重开，不是主题A/B或产品证据。900/640实际开关后可见overlay矩形由非空变空；两份相同paint rect不是两个弹层。[几何/字体边界](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/measurements.json)记录900边差0、640上下−0.015625/右0、BODY焦点和未完成的焦点返回。

[实际字体](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/font-probe.json)：fullID custom Roboto21glyph，中文标题system Noto Sans SC13glyph，Details混合177glyph。CSS声明Roboto但实际CJKfallback，custom/gfont/系统回退不等价；SVG rect16与computed lineHeight19.0666不能补签字体保真或Engine ready。

保留CR/LF helper失败、非改写role guard失败及原生排序尝试；最终所有动作都落在新root，旧源字节未受影响。640隐藏区误判已更正为继承hidden，不把native按钮未找到当修改成功。恢复helper正常exit0，20个已观察出生身份均无剩余；新profile已消失，首失败Sxyc2l156文件/11413749B仅精确受管清理提案，旧26/28及126B保留。手动释放0B，自动profile实际物理释放未量测，[退出](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/process-closeout.json)不冒充Engine/native host终态。

正式ZUI导出导入、运行中Actor数据与完整Toolkit、normal EXE/原生捕获/150%DPI/双端差异和≤1DIP均open，passed/accepted=false。

## 实现29：组件歧义原始位置的静态资格

四个原Prefab源均为本轮后继，snapshot新增冲突位置模型/原borrow投影，authoring descriptor派生临时索引所需Ord，authoring/lib只接既有导出；原read/expected/revalidate、层级与payload不重写。31定义（30普通＋1原ignored）均可从原cfg(test)入口/`prefab_`过滤到，三项新增覆盖原记录不合并、owner隔离/完整descriptor和源重建失效。[语义与精确源](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-component-conflicts-29/semantic-review.json)区分source27/原19项accepted，不以旧回执覆盖当前。

本轮read-only rustfmt初次exit1仅格式wrap差异，原log保留，修正后exit0/native d7c2cf；四源guard exit0/native e73bea、0errors/0warnings/0suppressed、patchDigest `15234c5f0d1f8d14709be502f21f0bc4cf08c1052be026bc345d23a486a5646f`。四源diff-check exit0/native6bf3a8；Git CRLF提示未执行index或checkout重写。静态不证明Rust类型/运行通过，已验证执行0/实际总数null。

5保护源与原31节点ZUI保持，未开浏览器/服务/桌面、新Cargo/Jenkins query/submit/source capture或新身份。Penpot28的rev176/三宽原图复用为source27设计证据，未把新增冲突API说成窗口已显示。原02/queue15836/#387 input23f4088d与原01/#348终态全部保留；当前31项需原authority合法接续，不覆盖19项受理输入或创建第三ID。无新临时/helper/源码副本，释放0B，旧受管清理项独立保留。

## 设计28：可重开组件Details证据

[最终保存/重开证明](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/save-reopen-preservation-final.json)记录Actor156→176、1679→2270对象，原1678非根对象精确相等、旧根顺序保持、五root内591新对象；保存/关闭/新context重开/最终只读观察data exact。1700/1700最近component owner来源引用有效（308本域/1392 Shared），原生copy heads为77本域+131 Shared，排除2个main；Shared5/data exact。本轮没有官方codec、source map或export baseline伪造。

[14张逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/visual-review.json)包含7张最终100%/DPR1 viewer图及全部中间失败；三宽1280×800/900×620/640×520均直接从原生viewer打开，直接frame-id缺index空白的失败保留，不据此恢复服务。640初次main顶部差19与font probe空段落另行保留，最终revision176修正后再保存重开。900与640原生按钮各实际开关一次Details，源交互目的地为本次原生复制生成的新overlay UUID；focus BODY不通过。

font-probe.json保留实际glyph来源，**整个probe不是只读**：进入编辑的Enter辅助路径插入空段落，修复前rev172/原图和后续精确行数检查保留。未盲套Shared typography、未改字体provider；CJK行框、Engine字体和双端误差尚未闭合。

[进程结项](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/process-closeout.json)：恢复helper33846正常exit0，20个实际观察PID/birth无剩余，正常profile不存在；首helper31530 exit1及其153文件/7215255B profile保留，已形成[精确受管清理提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/managed-cleanup-plan.json)，非server维护receipt。手工释放0B、物理释放未测，不称cleanup closed。4Rust/5保护源保持，source27 fmt/guard复用，28定义/0已验证执行、实际总数null；原02/queue15836/#387与19项accepted源不改，无新Cargo/query/submit/桌面/服务。

## 实现27：精确组件描述与原native key的边界

[描述DTO](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:94)直接借用native记录四字段；[查询](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:170)复用原entity_by_id，重复owner即拒绝，不因只有一个候选组件而选中重复实体；组件描述全部相等但有两条记录仍返回AmbiguousComponent。成功返回原记录引用，不改payload/资产引用/序列化顺序，不造组件UUID或公共schema。

[三项回归](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:771)用原ResourceManager/register_ready/native snapshot覆盖完整u64、同名与反序、四个描述字段差异、五种结果，以及remove/recreate后相同描述/相同数字revision仍拒绝旧key。tests实际由原lib的两条path加载，共28定义（27普通+1ignored），不是Rust执行证据。[语义审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-components-27/semantic-review.json)、[guard](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-components-27/change-guard-final.json)和[格式记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-components-27/rustfmt-final.log)保留静态范围；首次格式换行发现也保留。

三份Rust推进，authoring/snapshot.rs和5保护源保持；原02的19项accepted源码和历史25项另列，不覆盖旧input或终态。本轮无Cargo/QueryOnly/intake、Penpot/服务/桌面、源副本或清理；当前无产品截图变化可宣称。实际consumer仍须在读前/发布前核同一view incarnation/project/document/selection/request及native key，描述字段校验不能代替生命周期、provider schema准入或命令许可。

## 设计26：真实原生编辑、三宽重开与无效森林夹具

原Actor API138/1088→156/1679，新五个root共591对象；原1087非根对象逐字段相等，零root仅增加新子项，原两组件main及data元数据保持。最终save/fresh/末次读取均156/data exact；Shared前后rev5/data exact。1195引用按最近祖先componentFile/componentId解析至真实main子树且type相同，260归Actor、935归Shared，零无效引用；Shared实例根99。原source/slot/pluginData/绑定/事件均随原生copy保留，未detach或改公共main。[完整字段证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/save-reopen-preservation.json)与[原生动作](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/changes.json)记录UUID和覆盖。

七实体、六显示根和四项诊断对应[source25测试](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:663)，源索引只标原始位置；选中13保留有效父边11，循环11/12父边隔离。三宽新context裁剪均native100%/DPR1；两窄窗真实pointer toggle开/关，900右/底差0，640修正后346宽覆盖整诊断列、底差0.015625px，BODY focus不算键盘通过。首次残边图未覆盖，见[测量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/measurements.json)与[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/visual-review.json)。

CDP在实际editable text叶span观察：纯中文mode custom Noto19glyph，选中英文custom Roboto27glyph；混合诊断custom Roboto33、系统Noto27、Times箭头1。SVG外层probe空结果保留为测量限制，不能冒充Canvas/Engine字体来源。4Rust/5保护源哈希未变，同源fmt/guard25复用，25静态定义仍无Rust回执；本轮无Cargo、正式plugin导出或ZUI修改。

首helper遍历零root未终止，已仅对自身PID/birth核实后强制退出1；恢复helper正常exit0/native chunk8ad4fe。25观察身份无剩余，新profile不存在；旧profile11,413,746B仍在，156文件哈希计划已queue原管理owner（msg01a172da-0f62-7eb3-a1b6-226413eecbbc），不是维护intake/清理回执。[进程收尾](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/process-closeout.json)不声称native Job/双EOF或所有清理关闭；释放0B。实际producer、正式导出、normal Actor、150%DPI及双端验收仍open。

## 实现25：只读来源树遍历的静态证据

[原实现](../../../zircon_plugins/prefab_tools/editor/src/authoring/snapshot.rs:92)新增 `PrefabSourceHierarchyRow { source_index, depth, has_children }` 和同次借用的 `hierarchy_rows`；构造阶段在父级解析/循环切边之后以显式栈遍历森林，不递归、不按名称排序、不截断深度。原源载荷、实体行、组件记录和诊断保留，`source_index` 只索引同次快照，不能成为持久目标。

[两项新回归](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:625)覆盖源顺序不同于前序、同名与完整u64身份、重复ID/无效父级/循环及其有效后代；原8193行测试增加无递归遍历和Ready-empty断言。模块实际加载cases5+snapshot20，共25静态定义（24普通、1原有ignored）；没有Rust执行回执。[语义审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-hierarchy-25/semantic-review.json)、[最终guard](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-hierarchy-25/change-guard-final.json)与[格式检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-hierarchy-25/rustfmt-final.log)仅证明静态范围；初次导出列表换行发现仍保留。

原Jenkins owner的[终态对账](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/plugin-manifest-validation-20261024/workspace-entry/actor-prefab-existing02-intake-reconciliation-20261025-r01/result.json)14830B/db66bb23…已按字节复核：#387/queue15836 FAILURE、State v14 unknown/interrupted，原554B archive与Home/control一致。原key范围未见资源/执行/native行不等于Job/children/双EOF证明；当前25定义不能覆盖原19定义受理源。本轮无QueryOnly/POST/Cargo/捕获/服务/桌面或清理，释放0B，产品passed/accepted=false。

## 设计23：rev138单行来源说明与原型开合的实际证据

原Actor file/page通过原生文字编辑、source custom字体选择、Layers reparent、overlay marker及高度Inspector完成130→138，仍1088对象/2本域main。关闭原context，新context认证读取及13原图复核后data均等于保存；Shared仍5/data精确不变。80对象变化均在640原内容树/原overlay及另两mode内，其余1008对象精确保持；全部copy/source/slot/touched/pluginData不变，690 refs（212本域/478Shared）有效，67 Shared heads保持。唯一reparent是640 mode，唯一交互变更是原toggle overlayPosition；旧640失败板与900 overlay精确保留。

[字段与引用检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/save-reopen-preservation.json)和[13图逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/visual-review.json)保留完整身份/原PNG；[CDP字体](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/font-probe.json)各13 glyph为源NotoSansSC-Regular，不据同名Roboto/Google推断度量。保存字号13.333333、line1.2、Shared填充不变；Inspector默认14与实际内容字段有差别。positionData仍19 ink/16 line，SVG缩放测量与Engine尚未比较。

[原型回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/prototype-anchor.json)记录旧toggle实际点击开/关及overlay/main bottom差0.015625px；focus返回BODY，不能称键盘返回已通过。[640裁剪](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/preview-640.png)是native Reset100%/DPR1实际640×520工作区矢量裁剪，保留选框，不是官方exporter或Engine捕获。

自有Node34120正常exit0（native fda288），11观察PID/birth身份全无剩余，精确profile已不存在；无Job/EOF证明，手工释放0B、自动回收字节未测，旧失败/126B仍保留。原02一次只读query仍unknown/pending management_operation_interrupted，原input/resubmitted=false；原19项accepted与当前23项不得互代。4Rust/5保护源保持，复用同源format/guard21；本轮无Cargo、source capture、服务、桌面、DPI或正式ZUI修改，passed/accepted=false。

## 设计22的真实保存、引用与原图证据

本域原生Inspector/Layers编辑39个既有copy内部part，Actor125→130/1088；保存关闭、新context重开和截图后末次读取均130且data精确一致。1049对象不变、2个业务main不变、67 Shared根和全部690引用有效（212本域/478Shared），完整Shared data仍rev5。所有source metadata、字体、字符串、拓扑、slot、event、prototype interaction不变；原640失败板95节点及两份Details overlay各13节点保持，未复制main或detach。

[身份差异](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/save-reopen-preservation.json)、[原生修改记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/changes.json)、[前后几何](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/measurements.json)与[12张逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/visual-review.json)给出真实字段和原PNG。100%工作区原生矢量裁剪保留选框，实际1280×800/900×620/640×520；不是正式PNG exporter、normal EXE或实际150%DPI。两个native selector失败与取消选择后转thumbnail行为保留在[failure](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/failure.json)，没有DOM/CSS替换或补签通过。

[进程退出](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/process-closeout.json)记录Node15076正常exit0（session15721/chunk18a7b8）、两个context正常关闭/browserDisconnected；本轮10个实际观察PID+birth均无剩余，RI6JrL临时profile不存在。它不是server Job/双EOF/nativeLifecycle资格。没有清理动作，释放0B；既有失败profile与126B仍沿原受管清理身份保留。

本轮未改Rust/原ZUI/共享源码、未执行Jenkins query/intake/capture/Cargo或服务/桌面操作。当前23定义=22普通+1ignored仍无Rust执行回执，复用切片21同字节format/guard；完整来源/资源/收件/normal EXE门继续单独取证。官方plugin检查缺qualified source-owner worker/SDK入口，未以native几何或静态检查代替。产品passed/accepted=false。

## 只读切片21的唯一来源定位证据

原authoring/lib/tests三路径新增只读`entity_by_id`及`PrefabSourceEntityLookup`。查询私有原生payload而非可修改的公开projection Vec；Unique为借用实体，Missing/Ambiguous不携带可误作目标的行。两项新定义分别覆盖完整u64/零/同名/重排与重复ID/缺失/全部诊断行保留；原空来源定义增加Missing断言。当前cases5+snapshot18=23定义（22普通+1原ignored），没有Rust执行回执。

[最终源身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-selection-21/source-after.json)、[语义审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-selection-21/semantic-review.json)与[4源guard](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-selection-21/change-guard.json)记录实际字节/模块可达性/0error/0warning/0suppression。首轮一处assert换行diff保留，最终rustfmt check exit0；静态检查不证明Rust类型解析或真实UI/命令接通。snapshot.rs与5份保护源未改。

本轮没有QueryOnly、intake、capture、freeze、Cargo或新操作，原02最新已知查询沿实现20的unknown/interrupted记录，不把它重新包装成新终态；原候选/input/#348均保留。旧19/21项资格不覆盖当前23。未启动浏览器/长驻helper/服务/桌面，未生成fixture/build/source镜像或执行清理，释放0B。既有rev125设计图仅复用原档，normal EXE/真实producer/导出/150%DPI/双端仍open。

## 只读切片20的当前源码证据

新增`PrefabSourceReadSnapshot`仅在原authoring.rs定义，在lib.rs公开原类型并更新原tests；snapshot.rs与五份保护源保持。私有key/payload字段没有任意配对构造器或序列化，所有读取复用既有UUID和ResourceManager同锁检查；保留native record/revision及原Prefab/SceneEntityAsset/opaque组件，不复制model或新增clock/store。

[语义审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-resident-read-20/semantic-review.json)与[最终源身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-resident-read-20/source-after.json)记录两项新回归及原行测试增强。当前21定义（20默认+1原ignored），Rust执行回执0；rustfmt check最终exit0，首轮两处格式diff保留；[4 Rust静态guard](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-resident-read-20/change-guard.json)0error/0warning/0suppression。静态检查不证明类型解析、运行或真实caller已接通。

原02本轮[正式QueryOnly](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-resident-read-20/operation-query.json)实际client exit2、unknown/pending management_operation_interrupted，未重投且input23f4088d不变。此为既有操作的当前读回，不是新修订收件、Cargo失败或server已结束的证明；原01/#348、19项受理源及全部失败保留。

本切片未改UI资产/字体/布局，复用设计19的20张图作为原状态设计证据，未补签新API的EXE/双端通过。本轮无浏览器/长驻helper/服务/桌面/生成夹具/构建目录或清理，已返回命令的进程事实不替代server native生命周期，释放0B。每view producer/稳定target/写事务、正式导出、normal EXE/150%DPI与1DIP门保持open。

## 设计19与原02受理、中断的实际证据

[结果19](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/result.json)记录原file/page、rev112→125/1088、三张新board及源ZUI身份。初始保存/重开122和最终125均data精确相同；[保留检查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/save-reopen-preservation.json)确认原992非根对象和main/旧顺序精确保持，67 Shared roots、690 shapeRef均可解析，Shared末次认证读仍5/data相同。没有导出或改写31节点原ZUI。

[20张图逐张复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/visual-review.json)包括baseline、初版/中间失败、原生flow/auto-content修正和最终三张fresh125。900标题交叠与1280副本高度不足不能被save/reopen exact掩盖；均以实际图发现并在新板修复，旧PNG完整保留。[当前测量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/current-measurements.json)确认20编辑叶实际custom NotoSansSC-Regular，不推及其它中文或Engine。1600×1100浏览器/DPR1、画布71/98/134%不是150%DPI、正式exporter或正常EXE证据。

原02实际queue15836，inputDigest`23f4088d…`，客户端准备工具exit0；原候选1801B/`86b638a0…`保持。后续resource_capacity两次查询及最后unknown/pending、`management_operation_interrupted`均保存在[查询与原authority交接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/resume-execution-17/interruption-reconciliation-handoff.json)。原Rust19定义没有可供本族认定的执行/最终终态回执；不把客户端关闭或未知状态写成Cargo未启动/已终止/已通过，不重复提交。

[自有进程关闭记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/final-process-closeout.json)：成功helper32828与17696均正常exit0，当前17696及9个已观察PID+birth均不在；两成功profile已不存在。首轮失败helper39388已按原身份核对停止，其profile尚有153文件/7,214,825逻辑B，保留待受管清理。旧18,359,439B为历史限定记录，未冒称本轮重测；手工释放0B，未获维护receipt，不能称清理关闭。上述浏览器事实不证明Jenkins native Job/双EOF/server生命周期。

同期五份本域文档为原02中断观察之后的新文档修订；原受理元组与旧失败证据保留，新有限源指纹登记[本轮文档后身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/source-after-docs.json)。没有全源码freeze或具体写者归责；产品/EXE/DPI/双端验收继续false，goal active。

## 实现17与设计18的当前证据

[源码审阅17](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/resume-execution-17/semantic-review.json)记录expected reader、当前4源SHA及19定义。snapshot.rs仍source08字节，其余三份为后继；3新回归覆盖UUID移动、其它资源/manager及移除重建。final09只证明旧16项候选，原字节保留，不代验新19项。[change guard](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/resume-execution-17/change-guard.json)exit0/0err/0warn，Rust运行0。

原02候选未变。首wrapper QueryOnly因PowerShell吞掉`--`而exit2，引用后返回original_operation_missing；normal随后实际完整capture，Engine首轮52235文件耗时192.001秒并释放pins，随后因完整identity变化exit2，未POST/Cargo。保留[原输出](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/resume-execution-17/submit.log)，没有复用#348或造第三operation。

[设计18](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-local-cjk-status-18/result.json)Actor109→112/993、Shared5：只6原leaf换source Noto400及完整中文说明；987对象/全部组件引用保持，save/fresh data相等。18张图已逐张看，六次fresh SVG CDP各11glyph为custom NotoSansSC-Regular。CSS agent未启用/离屏nodeId探针失败保留，逐板入视口并启CSS后取得实际字体。[进程退出](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-local-cjk-status-18/process-closeout.json)helper exit0、23个已观察birth无剩余；无桌面/服务/手删，旧profile/126B仍保留。正式导出/normal EXE/150%DPI/双端≤1DIP继续open。

## 窄窗来源说明16的真实设计证据

原分配Actor file/page的两张640状态板保持UUID不变，原生Layers拖放将既有document-mode文字从actions移到根column，位于actions与workspace-main之间。原文“来源定义 / 无选中实体”从128×32变为624×16、完整单行；两Button仍240宽，不改Shared内部宽度、main或样式。主区通过fill由416变392，高度24来自正常flow，不写普通内容x/y。1280/900和其它板的935对象精确不变。

[原始证据16](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-status-16/result.json)及[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-status-16/visual-review.json)记录两次原生拖放、自动保存、新context重开及独立API。6张新增原图均实际打开，累计111张；fresh与after两组PNG各自字节相同。653引用全部解析，所有文字与组件/source/slot/event/touched/paint/layout值保留；2条SVG path因flow整体下移24，命令序列/坐标数不变。

首只读探针因未暴露namespace退出1，正常close后重用同任务；首次viewer hash导航未重跑frame解析而超时，fresh reload修正。首1秒GET仍107是异步保存早读，后续108/109实际确认，无重复拖放。初版证据检查把path.content当文字导致assert失败，实读两条M/L坐标证明仅y+24后按type校验，原失败保留。没有构建、Jenkins query/intake/capture/freeze或源码/Shared/服务/桌面写入。这是Ready-empty/NotResident的unit状态设计，未接真实生产publisher/操作/undo。Penpot来源身份包络仍缺，正式ZUI导出、正常Actor EXE、150%DPI、焦点/滚动首尾和双端≤1DIP仍open，passed/accepted=false。


## 最新源接入证据15：原31节点与当前设计不能直接视为同源导出

本轮23:15Z独立认证GET Actor仍rev107/993、Shared rev5，Actor data与14保存重开全同；原auth仅内存复用，不复制/打印/覆盖。原31nodes/29controls/15events/7imports逐项列于[源与语义对应](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-export-map-15.json)，原资产SHA a1e7448b…保持。全部15原event/route与原child/control ID保留，source中binding和repeat声明均0；这是记录现有源，不声称实际handler或生产数据存在。

六个14板没有官方role=asset/bridge-version/document包络，全文件0个shape sourcePath指向原模块。Shared leaf metadata虽有效，不能充当原模块control/source owner；源7导入仅2个精确Shared canonical匹配，另外5个未发布对应身份。实际代码plugin.ts选择/导出前置、zui-source-export.ts source guard及snapshot/patch/virtual-row/TreeView的当前源码指纹均已实读。该表不是合法导出manifest，没有伪造metadata或执行/代签导出。

[旧r03 host实际结项核查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-export-map-15-host-observation.json)读回native receipt1883B/SHA bd2795a5e61241f12f947ec91f45ec1a7f3bd0da1a44d4dfa3e20289ace3f32e，service1436B/SHA1dfbcf25b17e84ae85aa91d63caf7efcd363e2a175f2f1032b9f3c167fd501f0：22:06:58因原deadline正常停止，root0/Job双0/EOF/readers/waiter/request-threads joined。当前9968原birth不存在、4213无监听；仅核旧host终态，不是当前source build/SDK/产品通过。没有HTTP反复点击PNG或尝试过期host，也无新服务/浏览器/临时文件。

首次检查把component_property_rows的模块逻辑路径误作物理路径，FileNotFound退出1、未写报告或产品；按rg实际dispatch/workbench路径纠正，旧失败tool chunk cc845a保留。此轮3文档更新使14有限11路径元组成为历史，新metadata另记；4Rust/5保护源仍匹配，16定义0执行。无Node/Cargo/Jenkins parse/query/submit/capture/freeze或桌面写入，手删0B；旧限定18,359,439B残留仍待原受管清理。canonical导入/可逆源映射、生产row/写命令/Graph、正式EXE/150%DPI/双端均open，passed/accepted=false。

## 最新证据14：六张来源状态板、保存重开与失败保留

原file/page已94→107/993对象，新190对象：wide各37、900/640各29；新增18Shared根+2本域来源数据copy，累计58外链根/2本域main。653shapeRef=445Shared+208本域，逐个定位对应main子树均有效。原802非根对象/2业务component定义/原顶层相对顺序精确保留，仅zero root的shapes增加新板。[保存/重开回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/save-reopen-preservation.json)实录saved/fresh/独立API均107且data全同，Shared5/4main/34+3 data未改。

[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/visual-review.json)覆盖18张实际browser原图，12张历史初版/中间图未覆盖删除，最后6张fresh逐张打开，累计105图。初版层级/中心流位置陈旧、compact header0.01高导致标题重叠，MinH尝试失败；最终native Inspector居中、compact空wrapper删除后标题直接参与root flow。640/900/1280最后图均无标题/action交叠，640 mode仍两行拆词。源码坐标测量不替代字体/UE/EXE/双端1DIP。

[完整结果](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/result.json)列真实6board、component/main/source/override、只读API/单测与UE symbol依据。Ready-empty是0实体/0组件；NotResident是未知数量，不发布旧实体/选择；它们是静态unit定义设计，Rust/生产publisher未执行。4Rust+5保护源9/9保持，原31nodes/29controls/15events/7imports没有导出改写；source08/review09/input01/candidate02/#348及12/13结果字节保留。

[进程结项](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/process-closeout.json)：正常helper7596/root30616 exit0，首helper92419/root37576因stdin关闭检查8birth后退出exit1，合并18个已观察birth均gone。首失败profile留下153文件/7,206,915B，哈希均已盘点，无新maintenance身份/receipt，不本地删除。正常退出前后逻辑样本差11,393,483B，物理释放未测、手删0B；旧phase10的156文件/11,152,398B和126B仍保留待原受管清理。旧关闭回执不改写为本轮。

仅自有设计与6文档/E根元数据；没有Shared/public Rust/原ZUI/服务/桌面/DPI/Cargo/query/submit/capture/freeze操作。未取得source/writer/member/物理E reservation/pool-cache/native EOF准入，原02与入口13仍历史谱系；产品、正式导出、normal EXE/实际150%DPI/双端均open，passed/accepted=false。以下12及更早指纹/观察为原时点，不用旧receipt覆盖本轮文档。


## 最新证据12：已有drawer合同、native prototype与保留边界

[实际结果](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/result.json)实录Actor78→94/803对象、Shared rev5 data不变。新增196对象/138引用；579/579有效（379 Shared/200本域）、40外链Shared roots。原606非根对象/两业务component记录/顶层相对顺序全同。每组root+Details98对象/10实例（6 Shared+4本域）；2新Button保留component/file/main/shapeRef及文字覆盖，无detach/新main/公共写入。

[重开API](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/after-reopen.json)证明关闭原context、原认证新context读回rev94、Actor/Shared data全同；native SVG交互后data仍不变。[点击状态/像素哈希](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/viewer-interactions.json)实录两宽开/外部不关/同开关闭合，默认/返回SHA相同；前后BODY、无role/tabindex，focusReturnVerified=false。[14图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/visual-review.json)包含早期432/532高、两次锚点失败及最终400/500高，全部打开，累计87图。画布100%/DPR1与SVG不是normal EXE/官方导出/系统150%DPI或双端测量。

原ZUI responsive_compact_drawer/min_tier及实际host toggle/route、narrow≤640/wide≥1260只读pin在结果；4Rust/5保护源9/9仍匹配08/09，16定义/0执行，无query/submit/Cargo/freeze。[进程退出](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/process-closeout.json)为native exit0、9 births gone/temp0；自动消失逻辑样本11,393,979B、物理未测/手删0B；旧失败profile11,152,398B与126B保留。09/10/11有限元组保留历史，当前有限11路径身份另登记，不当九root capture-ready。

## 最新证据11：来源内容、原生引用与窄窗失败

[内容11](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-prefab-workspace-11/result.json)实录Actor rev49→78/607对象，Shared rev5/4主件/34+3且data不变。三张新增板各95对象、9个native component copies；新增15 Shared roots+12本域业务copies，441条shapeRef全部解析（273 Shared、168本域）。原321非根对象和2业务component记录精确不变，保存后关闭原context、用原认证新建context读回，Actor data及rev78全同。没有detach、复制main、改Shared或调用插件导出。

[7图逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-prefab-workspace-11/visual-review.json)保留1280首次/移出指针、900、640及三张fresh图，全部已打开；累计73张browser原图。1280/900只读来源与两条opaque记录可读，640中心18宽、渲染范围高1806，520 viewport内不可达。全部是unit fixture，不是生产数据、normal EXE、150%DPI或双端差异图；native缩略缓存及原context青色轮廓也保留为展示限制。dual-end误差仍未测量。

本轮9/9四Rust+保护源匹配08，16定义=15普通+1原默认ignored，执行0。新增仅Penpot内容和本域说明，不用#348旧8项或02旧13项代验；文档变动再次更新当前11-path元组时点，09四Rust语义审阅仍适用，但原11-path元组不能当完整capture-ready。后继仍同request原候选-02，未submit/query/capture/freeze，资源/pool/cache/writer/reservation门未消失。

自有helper正常exit0，9个原生birth均消失，本轮临时根0文件。[closeout](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-prefab-workspace-11/process-closeout.json)记录关闭前158文件/11,394,203逻辑字节的自动消失；物理allocated未测，手工删除0B。首个live文件哈希盘点遇到锁而不完整，原件保留，r2完整列路径/长度及93个不可读hash，不绕锁。旧phase10失败profile11,152,398B及126B仍待原受管cleanup，不能写全任务已清理。

本文件保留首轮源码/环境观察，并追加真实 Penpot 设计证据；不是 Jenkins 或 Engine 画面验收。所有追加时间使用回执中的 UTC，首轮本地时区标签仅为历史标注。原始命令输出保存在本 chat，未另复制源码或归档产品；后续设计原图与 API 回执见末段。

## 最新证据：Shared rev5原生消费、保存与全新context

[同步10回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-sync-10/result.json)为29135B/SHA`fddd2c8844c701305fee9d32712f24018ea5e5c9cc405b40c51c0f023204e928`；[新context独立API](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-sync-10/after-reopen.json)为2061284B/SHA`bbf998aeefc2d6744dd05240900c6014825cecef7f6af2ffeeeed3b54925c569`。Actor rev48→49/322对象，Shared实读rev5且本轮前后data不变。原生Update后4个Button根取得editor.surface.1真实颜色引用；关闭context、全新context后所有322对象与同步后全同。240个linked对象全部有效（120本域/120Shared），原body和独立local-color leaf均全对象精确不变。

同步前后并非全对象精确不变：103对象的完整delta保留，最大数值差0.3800048828125，28个path命令序列和参数数相同。actual RGBA仍#242424，不能算视觉A/B；这不是与EXE的1DIP误差。5张新增browser原图均逐一打开，三尺寸复核见设计AB-D10，累计66张；仍是稀疏consumer板，Button内部240宽和树换行缺陷未修，完整窗口/原生PNG导出/EXE/实际150%DPI/双端图均未完成。

首个helper未保持stdin，原始写入拒绝与第一次路径分隔符guard失败保留；核对精确自身root的birth/exe后受控退出，native exit1，8个原birth均消失。第二个自有helper native exit0，9个birth均消失。关闭前summary求和null保留，closeout按原条目重新计算；自动profile回收逻辑样本11,131,337B、物理未测、手删0B。首个失败profile156文件/11,152,398B及旧126B仍待原受管清理，逐路径hash见[closeout](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-sync-10/process-closeout.json)，不称整体closed。

四Rust仍是[source-tree-08](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-08.json)及[审阅09](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-successor-review-09.json)的16项定义=15普通+1默认ignored，实际执行0。原#348、旧input、未提交successor及两份报告字节保留；本次5份文档更新不冒充09时点的11路径元组仍当前。没有Jenkins query/submit/Cargo、公共源码/Shared、服务或桌面操作，passed/accepted=false。

## 前轮证据：真实驻留查询与三尺寸来源状态设计

[source-adapter-02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-adapter-02.json)记录原四路径中3份Rust增量、未变snapshot、Parent三份host/测试与原ZUI保护指纹，实际UUID registry→native active key→typed snapshot调用、5项新增源回归、四文件guard/rustfmt。13项源测试中1项原有性能门默认ignored，普通配方12项可运行，实际Rust/插件执行0。原#348准备冲突未重投、未被静态检查覆盖为通过；真实producer/逐view绑定/持久写target/undo仍缺。

[source-state-03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-state-03/result.json)记录Actor rev45/322对象、3张真实板/18个native业务copies/54个子shapeRef和保存关闭/fresh-context读回；全部322对象哈希一致。六种状态是实际API合同设计，明确未接生产者，不是引擎查询截图。原240个非根对象、2个主件、13外链copies及Shared rev1/4mains不变。6张新增browser原图已逐一打开，逐图记录见[AB-D09](actor-blueprint-design.md)，累计52张；没有官方PNG导出/正常EXE/实际150%DPI/并排diff或1DIP测量。

本轮原生mouse drag无创建、SVG严格locator/viewport locator、JSON重排无效与动态input选择失败保留；经真实素材HTML5事件创建实例、合法内容覆盖、Transit UUID重排与原生flow重算后才计保存重开一致。自有helper exit0，按9个PID/birth核对0存活，新temp根0文件；关闭前158文件/11,131,831逻辑字节自动回收，物理量未知、手工删除0B，旧126B仍待原Jenkins清理。共享host、主题/库、原ZUI、服务和桌面本轮未写，passed/accepted=false。

## 最新证据：Shared rev1正式连接及四copy消费

[consumer-01](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01.json)保留独立认证get-file/get-file-libraries、四主件canonical-record/main/source、原生管理库HTTP200和真实dragstart/dragenter/drop、逐节点source/ref对账。Actor从rev12/117对象到rev21/151对象，仍2本域主件/12原copies，新增1个原生column flow验证板和4个Shared copies/33shapeRef；Shared仍rev1/isShared=true/41对象/4主件，colors/typographies0。116个原非根内容对象、两个业务component记录和原根顺序未变。旧rev0空库观察按时间保留。

四个asset board的document解开typedValues包装后，penpot_original_source与当前源字节一致；源SHA见[共享合同](actor-blueprint-shared-contracts.md)，component/main/copy UUID见[设计AB-D08](actor-blueprint-design.md)。4次独立checkpoint的引用/元数据/原对象比较errors均空，含rev19中文覆盖重开、rev20原生reset保存、rev21重设中文保存及第二次新context读回。Button文字子shape的native touched由content-group/text-content-text恢复为无，再重新记录；没有detach、更新main或改公共主题。这是文字覆盖试验，不是属性写入/撤销或全局样式传播。

新增11张原图已逐一打开并记录，累计34张；全部是浏览器Penpot chrome/DPR1，约93%画布（早期12%），非官方PNG导出、Windows实际150%DPI或EXE证据。第一次画板/拖放未创建内容、SVG locator与collapsed layer选择失败、错误innerText读取以及UTF-8读回修正均保留，不能靠文件名充当成功。Property必需value槽无合法已发布child，保留空槽诊断；Search/Tree/Property仍默认文字，未接业务后端。

原认证storage只读复用，无复制/打印/覆盖secret；headless没有使用桌面。真实driver exit0，关闭前9个进程的PID/birth/exe/命令SHA与关闭后对账为0存活。临时profile关闭前158文件/11,131,339逻辑字节，Chromium关闭后新临时根0文件；自动回收不作为物理释放测量，手工删除0B，旧126B继续待原Jenkins清理。过程见[退出回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01/processes-after-close.json)。本轮仅Actor文件、6份本域文档与reports/manifest变化，4份既有Rust和原ZUI未变。插件/受管Rust未运行，原PNG404不重试、服务与桌面未动、passed=0/accepted=0。

## 实际执行范围

读取 `AGENTS.md`、套件既有记录、相关技能与配置；本地只读 Git 状态/HEAD、`rg` 搜索、TOML 解析、SHA-256、进程/监听端口和正式 Jenkins status。读取与结构检查使用 PowerShell 及通用 Codex Python，`-B` 禁止 bytecode 产物。另经本机 CLI `queue --help` 确认受支持入口，向用户指定 Parent 排队交接，回执见下。没有调用 Cargo、插件测试、源码/资源生成 recipe、截图、Git 写入、服务启停或旧协调器。

`git -C dev/UnrealEngine rev-parse --show-toplevel` 返回 `E:/Git/ZirconEngine`；参考树没有在本轮解析出独立 UE Git revision。`git -C ... rev-parse HEAD` 返回的是父仓库 HEAD，**不能**作为 UE 上游版本。本轮采用下面实际文件指纹，后续不推测 Epic revision。

`Engine/Build/Build.version` 实际值为 6.0.0、Changelist=0、CompatibleChangelist=0、BranchName=UE5。它不足以确定正式 UE 发布标签。

## UE 目标文件 SHA-256

以下路径相对 `dev/UnrealEngine/Engine/`。这些是检查时的文件身份；不是参考图片，不能用于计算视觉相似度。

| 文件 | 字节数 | SHA-256 |
| --- | --- | --- |
| `Build/Build.version` | 191 | `3dcd8b6654872d013f2b00aa5f0ef1e0db89d4e8627ce820554675a27f228ea6` |
| `Source/Editor/Kismet/Private/BlueprintEditor.cpp` | 394742 | `9becb2c85c3edd524a2f3a2705994994fdd17232eef9b568bd67ae58186de096` |
| `Source/Editor/Kismet/Private/BlueprintEditorTabs.cpp` | 1351 | `a635045188b5413e349407631c6e97a1ba8a4e3b4434cfb99715bd117b38ac80` |
| `Source/Editor/Kismet/Private/BlueprintEditorTabFactories.cpp` | 18227 | `e7a2ce69e4fc62ae9e6ce1e43fa750689aa151a8f5e6018e21a701602ccb6038` |
| `Source/Editor/Kismet/Private/BlueprintEditorModes.cpp` | 25402 | `3e8fb84afdb1810f5b912a0e02feb9d4f3963d0426a5b4107fe96a2e541e6547` |
| `Source/Editor/Kismet/Private/BlueprintEditorCommands.cpp` | 20900 | `0ae27ab0b7c5758ef645de14e1a328e7adcb36a3d346aa1021561ae6428ac1a5` |
| `Source/Editor/Kismet/Private/SBlueprintEditorToolbar.cpp` | 28079 | `b709784c4410dd93b70eec88ed301175abb52b98adf64cd440aa4284bdc3f334` |
| `Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp` | 131777 | `073a88aadd97bc8d85b7c59392ea3d9d691a19bb387d9db554d4208f3c9e0b81` |
| `Source/Editor/SubobjectEditor/Private/SSubobjectBlueprintEditor.cpp` | 40290 | `3ba9a23529ea48e976172b6976468ba723aaf20db524393a615bcf91dedb9f99` |
| `Source/Editor/SubobjectEditor/Private/SSubobjectInstanceEditor.cpp` | 29478 | `07c5533e307dade6e37fe3595837ab8feea58cf0cd23a63fb7a2d1d8e4372412` |
| `Source/Editor/EditorStyle/Private/StarshipStyle.cpp` | 579911 | `1c91e6767a76ba82505c16d815fb3530b622b59a8d13fc97c18662cba2abd2ca` |
| `Source/Editor/LevelEditor/Private/LevelEditorActions.cpp` | 203248 | `36824031f368b275f161d9da59a4bd9bdb81685396c478ae1277681707713c29` |
| `Source/Editor/KismetWidgets/Private/CreateBlueprintFromActorDialog.cpp` | 32886 | `27f49dca7af8330a4e4336c3b6cbea38155d0f25b16fb8ac8c5dc9b675529ffc` |
| `Source/Editor/GraphEditor/Private/GraphEditorActions.cpp` | 16412 | `8c2a073ff69807530725203465a9f1e386e190d7eafbdeb7abfd00f212d40d02` |
| `Source/Editor/GraphEditor/Private/SGraphPanel.cpp` | 106089 | `85093de079f1ef275f09120051d77fd7423e21a3521be9111df006dd016d74b1` |
| `Source/Editor/SubobjectDataInterface/Private/SubobjectData.cpp` | 39949 | `4f38bea9c84ea8eb945d1402ee7fe4c864605a27ae7cd967dc7e8bd79ac26188` |
| `Source/Editor/SubobjectDataInterface/Private/SubobjectDataSubsystem.cpp` | 110488 | `6ad2570a5ff1971eaf6c3a64e52f0ace703b9a21e7aca03d17b475d28301c26a` |
| `Source/Editor/BlueprintGraph/Private/EdGraphSchema_K2.cpp` | 314439 | `e99a0e2009953f532510efb03f091e32d4460f761403f790c1e501815d009594` |
| `Source/Editor/Kismet/Private/STimelineEditor.cpp` | 65120 | `417cb85acd66cf186db448e526d7fae2001a8976474f008f286e9733e760bc79` |

样式定位：`StarshipStyle.cpp:4026` 的 `SetupGraphEditorStyles`、`SetupMyBlueprintStyles`、`Kismet.Tabs.*`、`Blueprint.CompileStatus.*`，以及 toolbar 实际读取的 compile status brushes。需要由 Parent 把适用样式映射到既有 Starship/tokens，不把整段 UE 样式复制成新主题。

## 原始资产及共享依赖基线

以下路径相对 `E:/Git/ZirconEngine`。共享文件可能被其他会话继续修改；本表是本轮基线观察，**不是**最终依赖闭包或可复用 capture 的依据。

| 文件 | 字节数 | SHA-256 |
| --- | --- | --- |
| `zircon_editor/assets/ui/editor/world/extension_prefab_editor_workspace.zui` | 16284 | `a1e7448b621b78c0c511349434a4a55c9239af750a6fc3cc376502fd4c81bbbf` |
| `zircon_editor/assets/ui/editor/components/catalog.toml` | 26263 | `357e63b22d6203029c3a10980f2cf072e28ca584074d5f14863fd181f04d05cb` |
| `zircon_editor/assets/ui/editor/theme/editor_tokens.zui` | 14617 | `dc23135b1e349773266e09cfb354c01bf047774ac0beb70a6a129e5ab4f66743` |
| `zircon_editor/assets/ui/editor/unreal/starship.zui` | 11889 | `1e9140bd0b234271d13a575bf5c983c180f86bd395135afa84c261a2bc05f470` |
| `zircon_plugins/prefab_tools/editor/src/authoring.rs` | 2861 | `407497ea821c4430b0517ea06c1c8c9ab8292b2d305d3a51d5e828aab2b86dfc` |
| `zircon_plugins/prefab_tools/editor/src/lib.rs` | 805 | `1b9afe8c09244359413c9cb7e77bc1a93940606508646ed0fbbc2821143d99c2` |
| `zircon_plugins/prefab_tools/editor/src/authoring/snapshot.rs` | 4492 | `f1295a3a013195e3482d13d387a892664430fbe7c6875cab76b9974c9af9a187` |
| `zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs` | 4728 | `dd897d134d76feb8aa59353a68b4bca15d82db7d078038133c5f6ac6ab0ab8ff` |
| `zircon_plugins/prefab_tools/editor/src/plugin.rs` | 3062 | `c1e7a0111976bcf9d9635b880e5fddf783ab6545cd3a43ea4e9ef87a387c4917` |
| `zircon_plugins/prefab_tools/editor/src/tests/cases.rs` | 9679 | `d3b3966e459ea051581609e57b621b9f865b2c2c3d244b539d4133e4769a80ee` |
| `zircon_editor/src/core/editing/command/add_component.rs` | 16697 | `f128dac42c66865b24dbaed8d8ba0c9cc2fe7d2ac47682740ab830510334837e` |
| `zircon_editor/src/ui/retained/dispatch/templates/workbench/feedback.rs` | 43652 | `c505e6083162dfa1354f5726888374ba4fde835c4d2f6bd527a1dba57a876f01` |
| `zircon_editor/src/core/asset/type_registry/builtin.rs` | 8505 | `f5e26e835b7d3d6df8bd02fa786ec33a8c6b7d440ed07b16cde232aaebb5e8ce` |
| `zircon_editor/assets/ui/editor/components/workbench/primitives/data/tree_row.zui` | 2201 | `c6687ef563179bfbe395917cb4805914cb9e2b3f7f0bc49091f00135be144cf2` |
| `zircon_editor/assets/ui/editor/workbench/primitives/data/preview_viewport.zui` | 736 | `6172ff574c9e884e0f5d4666a07306f00ba5f0f5f48a7f05c4f900b1ac5fb27f` |
| `zircon_runtime/src/scene/world/dynamic_components/creation.rs` | 13633 | `185aa76b199d514063bb3c0f6f22fd5e9dcb68e5c645292f4c1df0c68dfc28d8` |
| `zircon_runtime/src/scene/world/dynamic_components.rs` | 41001 | `fdfd314b9af6b40c2bb9de62baa19373b9e5688fb1c388467c077740d92e2965` |
| `zircon_runtime/src/scene/world/typed_api/dynamic_component_presence.rs` | 1852 | `1330b68102c505e549cfb2e718197761a9e12cf4b476985fdfee9eadd40adcbc` |
| `zircon_runtime/src/asset/assets/scene/entity.rs` | 3291 | `1bd72aacff95bef0fcbc16bc4a75c565f3027c793211e9dfa490df934a30648b` |
| `zircon_editor/src/ui/graph/model.rs` | 12374 | `ba24acaa709b08cd9ed5ce7ec65070df603d85ed6c6ab9ee22a21c5fbbdd5e4d` |
| `zircon_editor/src/ui/graph/canvas.rs` | 5271 | `23ea48fa11978477851bfee22e288e89594fb29c248b4bbdd78d5e0736e810bb` |
| `zircon_editor/src/ui/graph/node_widget.rs` | 1101 | `84d22f5ae28b8fe9d0091439d6aee64e03efc6d780f1fe48ce720925332719d2` |
| `zircon_editor/src/ui/graph/commands.rs` | 13152 | `0a94bdc81e7e817cbf871d85ac986e8c0bc86dd8bd98bd42d77f24af883f92c4` |
| `zircon_editor/src/ui/graph/routing.rs` | 2407 | `e791d5f28a91c052c90b484f1e72e5d50e028d47548c91fc13b04ce5abf7838b` |
| `zircon_editor/src/ui/timeline/model.rs` | 3736 | `54c0d5e369c5b4165e519169c23301d03ea8a8b8a5480ac86d5bb54fa0d1411b` |
| `zircon_editor/src/ui/curve/model.rs` | 4939 | `970bbe0125c87e3f95dc9f3d90a125d0f4ba0686871b3fffff0f9021c0025b84` |
| `zircon_editor/src/core/editor_authoring_extension.rs` | 11517 | `f6f3fd845bf375bbaefa0ffbffdf1078c7edb39319750e19a6a588c946375f3e` |

原始 asset：`kind=component`、`version=2`、`id=res://ui/editor/world/extension_prefab_editor_workspace.zui`。组件 `WorkbenchExtensionPrefabEditorWorkspace` 的 root 为 `prefab_editor_workspace`，当前 31 个 nodes。尚未修改。

保留的主要控件身份包括：Workspace、PrefabRootRow、MeshComponentRow、LootSocketRow、MeshTableRow、LootSocketTableRow、LightTableRow、OverrideTableRow、ApplyButton、ValidateButton、PrefabDropdown、VariantDropdown、InstanceField（完整 `WorkbenchExtensionPrefabEditor…` 前缀见原资产）。

原资产中有 15 个 event entries，覆盖 Click、Change、Submit。Apply/Validate event routes 分别为 `workbench.extension.prefab_editor.apply` 与 `.validate`；dispatch spec 使用 `.apply.invoke` 与 `.validate.invoke`。这是两个层次的现有身份，后续需跟踪实际 route translation；本轮不凭字符串不同就判定 routing bug，也不擅自统一后缀。导出应用必须保留既有 event ID 和绑定语义。

## 当前真实能力的回归入口

这些测试在本轮**没有运行**，仅列出现有源码 symbol 以供真实实现后选择最小检查。文档盘点本身不触发 Rust 构建。

| 所有者 | 现有测试/入口 | 能证明的边界 |
| --- | --- | --- |
| `zircon_plugin_prefab_tools_editor` | `prefab_authoring_registration_exposes_only_read_only_surfaces_without_factories` | 当前只读注册合同；新增真实写命令后需有意更新该合同，不能将它现有通过推断成写能力 |
| 同上 | `prefab_override_query_preserves_ordered_conflicts_for_same_property`、`prefab_instance_validation_rejects_duplicate_override_paths`、`prefab_instance_validation_reports_missing_source_and_bad_override_paths` | 覆盖查询与诊断，无产品 Apply/Undo/Save 证明 |
| `zircon_editor` | `core/editing/command/add_component/tests.rs` 中 `registered_add_operation_commits_one_document_record_and_round_trips_multi_target_undo_redo` | 真实 Add operation、多目标 document journal/undo/redo；不包含类/Prefab attachment 模型 |
| 同上 | `add_rejects_stale_selection_schema_and_world_preparations`、`add_journal_replays_through_registered_codec_with_fresh_instance_receipts_and_document_undo`、`add_history_rejects_a_replacement_world_without_removing_its_equal_json_components` | 拒绝 stale selection/schema/world、重放与 world/instance 身份隔离 |
| `zircon_runtime` | `scene_assets_preserve_prefab_instance_metadata_through_world_roundtrip`、`scene_assets_keep_prefab_instance_metadata_after_runtime_extension_installation` | Prefab 实例元数据的 World/Scene 往返；不证明实例展开、Apply、保存事务或类默认 |
| 同上 | `formal_scene_writer_reader_preserves_prefab_instance_metadata` | 正式 scene writer/reader 保留 Prefab GUID/路径提示、local transform 和 typed override value；不是磁盘作者事务、资源 readiness 或实例展开 |
| 同上 | `importer_registry_prefers_available_extension_importer_over_higher_priority_diagnostic`、`importer_registry_prefers_available_full_suffix_importer_over_higher_priority_diagnostic` | 可用 importer 优先于 DiagnosticOnly；不能用插件 stub 推断内建 typed Prefab importer 不存在 |

`dev-module-build.ps1` 能显式接收 `-Package`、`-Action test`、`-LibTests`、`-TestFilter`；它构造 `--no-default-features --locked --jobs 1` 的最小命令。Prefab editor 包没有声明可选 feature，验证时应显式空集；传递依赖仍须检查实际 feature resolution。Editor 包的 features 和实际目标由变化范围决定，不能套用 full/default fallback。

## 实时环境观察

实际 status 命令：

```powershell
& 'C:/Users/HeJiahui/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe' -B -X utf8 -m tools.jenkins.tray --formal --config .jenkins/deployment-spec.json status
```

返回 exit=0 只表示状态查询完成：`state=degraded`、`serviceReady=false`、`ownerKnown=false`、`ownerLost=true`；controller/agent/plugins/acceptingRequests/ready 均为 false。保留原 runtime generation `start-1792418123320632400`，不按历史 operation 的 `state=running` 宣称当前健康。

`.jenkins/state/independent-service/status.json` 返回 `status=recovery-pending`、`reasonCode=FileNotFoundError`，描述固定 Codex EXE 路径不存在；已有 resident PID 为 17668，creationTime `134372271172265517`。这里只记录观察，不把 PID 单独当作接管/终止授权；本会话没有启动、停止、修复或替换服务。旧 `service-health.json` 与 `tray-health.json` 的历史 running 记录不代表当前在线。

初次检查没有发现运行中的 Zircon 命名产品进程；没有本族 EXE 指纹。当时真实 Penpot URL/file/page/authenticated session 均待交接；后续已分配并独立读取，见下。board/component、设计保存重开和官方导出仍缺，不打印或复制认证内容。

## Parent 交接回执

本机 CLI `C:/Users/HeJiahui/AppData/Local/OpenAI/Codex/bin/2993767bd1dfb009/codex.exe` 的 `queue --help` 明确支持向既有 session 排队消息。使用 `queue --thread 01a16d4e-0491-7353-84fa-038bd7706546 --message …`，实际返回 `Queued message 01a16d63-831d-7a93-845c-c96e36ffd0fd`，exit=0。消息包含本族路径、当前能力缺口、共享依赖和 Penpot/桌面/Jenkins 等待事项；没有读取或改写 chat 数据库。这个回执不证明 Parent 已读或资源已就绪。

收到 Parent 明确接续指令后，按其要求再次汇报本域明确路径、计划、共享提案、真实缺口及下一项独立可做工作，实际返回 `Queued message 01a16d6a-9ea9-7e61-844c-8c5753c0f1ae`、exit=0。当时真实 Penpot 文件仍未分配；没有启动/恢复 Jenkins/Penpot 或占用桌面输入。新增 [具体设计规格](actor-blueprint-design.md) 是该项独立工作的结果，不是已保存 Penpot。

设计初稿及具体树/预览依赖已追加汇报，回执 `Queued message 01a16d70-e8d5-7d60-96ad-af43f41f8806`、exit=0。随后继续追踪组件 API，确认 runtime 除结构队列之外已有 `World::remove_dynamic_component`，缺口是 Editor 文档事务/逆操作/多目标原子性，已补正工作记录及设计；创建令牌不参与序列化的边界也已记录。早期“只有结构队列”的描述不可作为当前能力结论。

该能力补正及本轮结果已排队同步 Parent，回执 `01a16d73-51ff-7ba0-9171-24e69c09f4f6`、exit=0；不把 queue success 记为已读或 Penpot 文件分配成功。

Parent 的生命周期目录已出现 `launch.json`，自述其服务辅助程序为 `started-readiness-pending`、`formalAcceptance=false`。这只是一份 Parent 所有的启动 metadata，本族没有接管或复用其进程，也没有据此声称真实 Penpot 已可用。

最新 [Parent 统筹记录](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md) 已提供实际 ready、九文件分配及共享 owner；本域于 10:33:10Z 使用既有 P21 认证的 cookie 仅在内存中独立调用只读 get-file，两文件均 HTTP200，身份/页根/组件/样式及响应 SHA-256 已记录于 [共享合同](actor-blueprint-shared-contracts.md)。未另存 auth、写 Penpot 或运行桌面。首次旧字段 `pages-index` 解析断言失败，按实际 `pagesIndex/isShared` 修正检查器后读回；失败保留，不改服务。已排队汇报真实分配/公共合同，回执 `01a16d7a-5871-7d52-b2c0-44f3fa6e5a9b`、exit=0。

Parent 后续 Jenkins 记录为新 runtime `start-1792836412672596900` 的公开 capability/controller/agent/plugins ready，且 nativeLifecycleVerified=false。本域后续独立 capability 也 exit0/ready；正式受管输入已接续，但尚无 Cargo 或测试终态，详见末节。不修改历史失败、重复重启服务或据 ready 声称插件/Rust 验证通过；普通验证沿工具自动登记，不重复向用户索要会话许可。

## 本轮文档结构与原资产检查

首轮对当时 5 份文档执行只读结构检查：19 个本地链接、59 个显式 UE 源路径均存在，23 项表列源码/资源 SHA-256 与字节数匹配；25 个 ABW、45 个 OVW、14 个差异、18 个场景和 4 个环境的稳定 ID 序列有效。原始 ZUI v2 可解析、局部 child 引用及 control ID 有效；31 nodes、15 event entries、7 个 widget imports 和导入组件身份保持原样，exit=0。这不是插件检查、Rust 测试或视觉验收。本轮新增设计后补充结构检查并更新生命周期清单，不把先前结果冒充新文件已验证。

本轮新增设计及细化能力后，实际检查为 6 份文档、26 个本地链接、64 个显式 UE 路径、33 项源码/资源 SHA-256 与字节数，全部匹配，exit=0；原资产和上述条目序列保持。另核对 8 个设计段、7 个 catalog/document 身份及 slots，其中两个 region_panel 是 view 内的原生宿主节点，五个为 component 文档定义；确认静态预览默认内容的能力限制。新增校验初次将 view 当成 component 文档而报 `KeyError: components`，已按真实结构修正检查器；原失败输出仍在工具记录，没有改产品使检查通过。结构检查不能代替实际命令/插件、Rust、Penpot 或窗口验收。

## 验收与保留

新增共享合同后，实际只读检查为 7 份文档、39 个本地链接、66 个显式 UE 路径、43 项源码/资源字节数及 SHA-256，全部匹配，exit=0。稳定 ID 序列、原始 ZUI 的 31 nodes / 15 events / 7 imports、8 个设计段及既有 catalog/slots 检查也通过。检查范围仍仅限文档、源码与资产身份；没有运行插件、Rust 或视觉验收。

后续实际保存区域草稿：真实 update-file HTTP200，58 条 change 后 get-file 为 revn=1/58 对象/41 flow frame/0 components/0 instances；文件范围仅为本域已分配 file/page。准备属性与后续 revn=2 只读比较未发现改变，16 个文本出现 positionData；不补造未捕获的浏览器自动保存请求。实际板 UUID、六张完整浏览器原图及逐图复核见具体设计 AB-D08，自有布局/browser 回执保留原输入/响应/图像 hash。

首次 Node 检查把 CommonJS require 与顶层 await 混用而报 ERR_AMBIGUOUS_MODULE_SYNTAX，尚未创建进程；包装异步入口后，直接 CDP 两次未取得连接，Chrome exit=3，原 receipt.json/receipt-02.json 保留。切换到项目已存在的 Playwright 依赖和同一 Chromium 后，首板首开/关闭/fresh context 重开、其他四板独立打开/关闭成功，零 page error、最终自有 Chromium 进程为 0。直接启动失败根因未确认；成功路径证明本轮服务、认证和真实布局可访问，不把辅助工具失败替换为产品通过。

区域草稿阶段保留规范文档、Penpot API/浏览器回执及六张设计原图，无正式导出或产品编译输出；后续业务阶段另记 8 张原图。没有自有后台进程、落盘临时脚本或源码副本；失败直接 CDP profile 的两份小文件共 126 字节待 Parent/Jenkins 按实际路径/hash 托管清理，清单保持 active。没有执行清理，实际释放 0 字节；共享服务和其他会话产物未动。

加入实际板/原图/回执链接后，文档检查为 7 份、49 个本地链接、66 个显式 UE 路径、43 项匹配指纹，稳定 ID、原资产与 catalog/slots 结果保持，exit=0。限定本域文档的 git diff --check 也 exit=0；七份文件为 untracked，空 diff 检查不能覆盖其内容，UTF-8/尾部空白/链接检查已由实际文件读取完成。未运行插件或 Rust 测试。

实际草稿/公共合同及清理交接已再次投递指定 Parent，官方 queue 回执为 `01a16d8f-dfdf-71f0-8ab6-abce24d4bd10`、exit=0。目标已读、公共组件交付与托管清理均未确认；不将消息 ACK 升级为任何验收或释放空间。

产品验收状态仍是 0 capture、0 passed、0 accepted；源码/结构检查不能替代插件测试、Jenkins Rust 测试或原生画面验收。
## 后续业务数据设计与能力边界（UTC）

原五板已接续两个本域业务数据主件、12 个原生 copies，当前 file revn=4/vern=0，117 对象、55 flex frames。真实 component/main UUID、modifiedAt、夹具来源和源资产候选节点见 [设计 AB-D08](actor-blueprint-design.md)，原产品 .zui 和 Rust 未改。

证据链为 [domain-data-01](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-domain-data-01.json) 的实际初次保存 → [receipt-05](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-05.json) 暴露 1/2/3 全变成 1 的渲染错误 → [domain-data-02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-domain-data-02.json) 保留无类型 set/val 请求遭拒 → [domain-data-03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-domain-data-03.json) 的 typed set-touched HTTP200/revn4 → [receipt-06](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-06.json) 实际四 context 首开/关闭/重开及三尺寸原图。

两轮业务设计各四张原图均已查看，具体缺陷/修复结论逐图写在设计 AB-D08；此前六张区域图也保留。共 14 张为 Penpot 浏览器全图，没有 Engine native capture。receipt-05 的 component_instance_count=14 包含两主件，receipt-06 明确 12 copies/14 reference roots；其空 business_instance_dom 未作为内容断言。源码测试夹具不算生产查询或资源 readiness，首末滚动/焦点/弹层/中文/真实 DPI 和双端误差继续未验证。

新增源码事实：内建 `zircon.builtin.toml.prefab` 使用 `import_prefab` 的 typed TOML parser，AssetManager 已有 `load_prefab_asset`；registry 的 availability 排序可避开 DiagnosticOnly 候选。场景 `zircon.prefab.instance` 元数据 roundtrip 的现有测试源码不证明实例展开/Apply；与插件 `prefab_tools.Component.PrefabInstance` 的 adapter、无持久 UUID 的重复 override 编辑身份均待核实。没有运行表中 Rust 测试、导入器或资源查询，也不新建平行 parser/store。

以下增加 13 个只读源码指纹；仍不是当前产品 EXE 的完整依赖闭包。Penpot 源只读，不修改其服务/构建。

| 文件 | 字节数 | SHA-256 |
| --- | --- | --- |
| `zircon_plugins/prefab_tools/runtime/src/plugin.rs` | 4886 | `b414b7bbcf30456d2af38fabf0fb08a455716b076437a27a2c0c08cd3748d247` |
| `zircon_runtime/src/asset/assets/authoring.rs` | 10894 | `a75dd01502aaa2669eb8f259a14e694b627ca42208b7a8c3f45889fb87bc6a36` |
| `zircon_runtime/src/asset/importer/ingest/asset_importer.rs` | 22525 | `af566d80a62d6dca6f96aa34883d11aec27ae68e65e309da529a92aee341b076` |
| `zircon_runtime/src/asset/importer/ingest/import_authoring_asset.rs` | 3542 | `3905c2e372c275ea1ff7b0394d7b02d91dc812f539a882694a3a07fe33f05c7e` |
| `zircon_runtime/src/asset/importer/registry.rs` | 16594 | `e354bf10a9d5ea0057790e052a90f81ecc35a5e1dfbee383d8b01b423a277a89` |
| `zircon_runtime/src/asset/importer/contract.rs` | 17872 | `56013987a57b5621dd8e1764b0b98b055b631a9bcdd77cfa9030f6eca14ece51` |
| `zircon_runtime/src/asset/tests/assets/importer/registry_priority.rs` | 9990 | `b785ec6a4a353bbe3fa0eaadc5715c6786a1fb053be3e05f3771f7c3838104a4` |
| `zircon_runtime/src/asset/pipeline/manager/assets/loading/load_asset.rs` | 9383 | `c42fb21d7410494ef94b11c4a634a06097e489a0225d14f1ca113fb1d162fc8e` |
| `zircon_runtime/src/scene/world/project_io/prefab.rs` | 855 | `10180746ce63c8493cdf821de0aa9906e7eefe50d8f5dd2fd9cdda67510fa5b3` |
| `zircon_runtime/src/scene/tests/asset_scene/hierarchy_sources.rs` | 15063 | `767769602f775158079215970ca9b1561afd0074849642d4fd678a63bb7a2a0b` |
| `dev/penpot/common/src/app/common/logic/libraries.cljc` | 155451 | `eec4d0ef0047c50adc6a554e91f74ab54dc70b49f1bf0ccfa05933da61d99367` |
| `dev/penpot/common/src/app/common/files/changes.cljc` | 47130 | `096aefdc170d9c8b8414badd8da3a783bc1470c7b283b714d89c68fb5e942475` |
| `dev/penpot/common/test/common_tests/logic/text_sync_test.cljc` | 50396 | `29e076d7eef07042fdac2553d267d0ffb458a3466fb600e48a9160ea48d9d56f` |

上述真实身份、修复和能力补正已通过官方 CLI 向 Parent 排队，消息 `01a16da0-5ec5-7011-a9c8-782514d5dc2c`、exit=0；只证明已入队。已读 Parent 对公共 pipeline 的委派和 45 项覆盖对账，发布/导出/Actor normal recipe 仍无实际交付回执。产品接入、插件/托管 Rust、normal capture、passed/accepted 均未完成，自有临时项 126 字节待统筹托管清理、实际释放 0 字节。

## 资源入口与受管测试的最新实际记录

[业务实例只读复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-domain-verification-01.json) 的 errors=[]，核对 2 mains/12 copies/36 子文本及 9 项不同文字 sub-touch；没有填补空 DOM 断言，也不作为正常窗口 capture。[资源接入审计](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/source-integration-audit-01.json) 保存另 20 项读取时指纹、AB-LOAD-01…03 和原请求各轮结果；其中产品/适配源码的 16 项补记如下，其余为正式入口与 immutable capsule 身份。

| 文件 | 字节数 | SHA-256 |
| --- | --- | --- |
| `zircon_runtime/src/asset/assets/project_document/tests/scene.rs` | 6910 | `8346b7ab0cf4e4d53d060d14299c30dcb59b1c2fafb355c4ea0521f69c9799bf` |
| `zircon_plugins/editor_support/src/lib.rs` | 4817 | `e6b3759def96161c21400ed23c3a4dde6432f52de0bda51503658f95aef523ed` |
| `zircon_editor/src/core/editor_extension/view_descriptor.rs` | 3209 | `ef04d8dcf416fb1e1c848cde450cecd64d0de4b794be1ed18bd5c1b2244f55d1` |
| `zircon_editor/src/core/editor_extension/template_contributions.rs` | 7346 | `7377bd6a2741455f3dd43fd6fd068afa160b595abe5eac50d23a227098713db8` |
| `zircon_editor/src/core/extension/inspector.rs` | 15494 | `08854d90cdc19d2f9603da95f61705193f5046e7cb6c4084d22add4bc5c682f3` |
| `zircon_editor/src/ui/host/exports/native_registration/manager.rs` | 5672 | `6a8994cda43716fa5379e3cb5b6e990e3f811b5c574cf50b1e1b169e7e3bfd04` |
| `zircon_editor/src/ui/runtime/runtime/plugin_documents.rs` | 20854 | `614b8c0c2518b011ed6a2a5cd22653d86ddbefc8f899863457b9d672fb38b39a` |
| `zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs` | 19864 | `52ff6cede018e9217ac5838230007176b59925c852ea7a5b98ef2597fa13be24` |
| `zircon_editor/src/ui/runtime/component_adapter/component_drawer.rs` | 3524 | `6857233a8fea510170b7b449e2159c46d3813bd2f7d3d03d406a796d5fd948a7` |
| `zircon_editor/src/ui/host/editor_event_runtime_access/component_dispatch.rs` | 5004 | `2ed8f2efdf37b01cfcad1f62a3d3b5c9702ac6f6dc16bed1d85381d0f56aa290` |
| `zircon_editor/src/ui/host/editor_event_runtime_access/extension_access.rs` | 8101 | `f3d0108ae7ae25811b6023f6b93a81f2e8c9f64b5d9469b5f9a17dbc552eaf3b` |
| `zircon_editor/src/ui/retained/contract/componentized_workbench_regions/panes.rs` | 46977 | `c315ec7f2b10a3f30900458c958b38f5238962d4a330eb2d5b829108d5999b23` |
| `zircon_editor/src/ui/retained/app/tests/module_tools_menus/cases.rs` | 24694 | `29553d973abd8a216a4ae81eebebe6f06eb795741a1da659bceacf03bd555ba0` |
| `zircon_editor/src/ui/workbench/page_layout_template.rs` | 9282 | `59fb3e1f92b4957c69a1a62f9a1a8532e98ee9e96a90de7ff14751ea7c8afaf7` |
| `zircon_editor/assets/ui/editor/layout/page_templates.toml` | 7306 | `68503db214a7886aa6c2d28e3e9586213854449f648b731c5a3f1aaa94e90fbc` |
| `zircon_editor/assets/ui/editor/modules/index/extension_module_workspaces.zui` | 26099 | `38f67174b336f75ef23e3d420909bc6a867e91fe9850e01999ca3e220fc1b784` |

实际调用链为 discovered eligible package manifest parent → bind_ui_template_root → strict plugins://owner/relative → 全部源 load/compile → 原子发布。canonical Prefab 插件缺 editor/authoring.zui 和 editor/prefab_instance.zui，实际 distribution root 尚未观察，仅记源码路径缺口。Inspector descriptor 的 template/data-root/bindings 为空；snapshot customization_available 仅 schema/customization 存在，不能证明文档加载。通用 host 在校验 allowed binding 后会 invoke_operation，Prefab 缺自身绑定/写工厂，不能将通用 host 认定为 no-op。

canonical page/index 引用 res://ui/editor/world/extension_prefab_editor_workspace.zui，而 native extension_workspace_source_identity 和 module_tools_menus 测试期待旧 /workbench/modules/extensions/world/workbench_extension_prefab_editor_workspace.zui。尚无运行时 source/instance/remap 回执，未执行这些测试、改共享断言或创建复制路径。通用 view.prefab_tools.authoring.open 已注册，资产上下文 prefab_tools.authoring.open 仍缺，二者不混算。

本轮 Runtime 聚焦验证已在每次编译能力入口前声明：仅 zircon_runtime、default features off、native-host、lib、filter prefab_instance_metadata、预期 3 项测试、test profile/jobs1/test-threads1，物理 E:/cargo-targets。覆盖正式 scene 文档与 World 元数据往返，不是为盘点文档构建，不覆盖 Prefab 展开/作者命令/Blueprint 编译。没有 Rust 或产品资产修改。

固定 identity：session 01a16d53-8965-7472-931c-73cba748fe8e，request actor-blueprint-runtime-prefab-metadata-20261024，operation validate-prefab-metadata-01，driver fc553228… / runtime start-1792836412672596900。installed capsule 为 baac70c97872d3395c5c422fa44a7cdaa481a8ac4f62820d1cc2ac099ad0ac08。原输入/日志均留在本族报告根目录：

| 输入/日志前缀 | 实际退出与原因 | 边界 |
| --- | --- | --- |
| runtime-prefab-metadata-01 | exit2，Unsupported Cargo fields: taskEvidence | 我误加自有 evidence 字段，预检拒绝；原输入保留。模块 wrapper 无显式 profile selector，解析默认 dev；没有 Cargo。后续正式入口显式 test，不把声明当实际 profile 证明。 |
| runtime-prefab-metadata-02 | exit2，external_control_forbidden | 除去非 schema 字段、显式 test 后，legacy source policy 将 zr_vm 两 tracked plans 视为控制输入拒绝。不是第三方源码错误；未删文件、改 ignore/index。 |
| runtime-prefab-metadata-03 | exit2，Source changed during pre-submit capture | 已安装代码支持明确的 full-file-index-v2；采用后保留完整 observed bytes/index/hash，合法区分 stage0/mode100644 的 root plan Markdown。两次 52122 文件读取间两份动画文档变化，外部输入/index/membership 未变，未 POST/启动 Cargo。 |

03 的两个实际变化路径为 docs/plans/penpot-ue-window-suite-20261024/animation-parent-handoff.md 和 animation-shared-controls.md，before/after hashes 与 capture digests 全量保存在原日志和审计中。本域未改这些文件，也不回退并发写入。02/03 的原身份 query-only 都 exit2，Current session authorization is pending；不是正式执行或测试失败/通过回执。没有排队/执行 ID、actual target dir 或 Rust 结果，不盲投新 identity、绕过 guard 或复制 checkout。

三个提交客户端和两个 query 客户端均已退出；native Cargo=0。具体接入缺口、原 lineage 和协调请求已投递 Parent，ACK 01a16db5-38e3-7220-81fa-cf81a60aab83、exit0，仅证明排队。Parent/Jenkins owner 接续自动登记与有界输入核对，原 pipeline owner 接续资源闭包、公共库/导出和 Actor normal recipe；本域继续 0 capture/0 passed/0 accepted、cleanup open/实际释放 0 B。

2026-10-24T11:42:40Z 再次独立只读 [Penpot readiness](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-readiness-02.json)：本域仍 HTTP200/revn4/117 对象/55 flex/2 主件/12 copies，完整响应 SHA-256 与先前业务实例复核完全相同，复用原引用/文字证据，没有重跑视觉测试。共享文件仍 revn0/1 根对象、0 components/colors/typographies、isShared=false；libraries 字段缺失，不假定已核实全部库链接。它证明当前访问和上述资源状态，不能代替已发布共享样式、插件导出或产品加载。


## Prefab 作者数据投影：本域代码接续

Parent 本轮明确由本域接续作者 adapter；四个代码路径及前后指纹见 [authoring-adapter-01](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/authoring-adapter-01.json)。上表 authoring.rs 已更新为本轮实现，原始 3356 B / `ea05eaedca825ddcfedd78c434f38575e2b886238922569739beada593364a97` 保存在 adapter 的 before 记录；其他来源基线仍按各自读取时点。新投影借用真实 PrefabInstanceAsset，保留原序覆盖与冲突行，来源状态分 Unchecked/Available/Missing；未创建固定生产数据或公共 trait。旧消息合同保留，effective query 和既有 release performance gate 未改。

最初 change guard 报告 snapshot 子模块未达；删除 lib.rs 中冗余的 authoring path attribute，采用标准 mod 声明后，四文件 guard 通过（0 errors/0 warnings），两个原始回执均保留。四文件 rustfmt --check 通过，git diff --check 通过但有仓库 LF/CRLF 提示；未执行格式生成。新增三个有意义的回归用例，尚未有 Rust 执行回执。

正式结构化 Jenkins 输入 [plugin-authoring-01.request](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/plugin-authoring-01.request.json) 指定单包独立 manifest、空 features/no defaults、lib/filter prefab_、test profile、单任务/单线程、E:/cargo-targets。预期 8 项命中，其中既有 release performance gate 1 项 ignored。此请求验证新增插件行为，不替代旧 Runtime 元数据请求；产品 source producer、资产写入/undo、Penpot 公共实例/导出和正常 EXE 验收继续 pending。

实际 [插件提交日志](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/plugin-authoring-01-submit.log) 为 exit2 `Source changed during pre-submit capture`。两轮 52131 文件 capture 期间资源工具三份文档及设置 handoff 一份变化，完整 drift 路径/前后指纹在日志与 adapter JSON 中；本域 Rust 未被列为漂移项。无 POST、Cargo 或测试终态；同身份 [query](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/plugin-authoring-01-query.json) 为 `Current session authorization is pending`。客户端退出、pins 均释放。随后公开 capability 再读 ready，nativeLifecycleVerified=false；不能替代上述缺失回执。

[实际历史包核查](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/packaging-inspection-01.json) 锁定 staging manifest 与 R94 项目清单指纹，N57 原 canonical 工作区与 checkout 同为 `a1e7448b…`；旧 native pane 两 suffix 均未在该部署路径发现。675 个 staging 项仅有原工作区及三个图标命中 Prefab，无 authoring URI 映射；R94 声明的三个插件中无 Prefab。没有把清单字面未命中推广为所有发现路径不可用，未启动产品或新增假资产。

12:00:12Z 再次独立 HTTP200 get-file 的 own/shared 响应分别保持 `73df20a5ab37537af6a8f1900524a73ed6638d77428983820a2b544bb19ece08` / `fe0d8e5c3ff7850da27eade99425f4192fa7f8665047b43a506167ff4afcc734`，复用 revn4 引用/文字复核证据。原 [640 设计图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-640-r6.png) 再读仍有 URI/左栏标题裁切；仅本轮复核，无新浏览器或原生截图，0 passed/accepted。

## Widget Graph 边界核查

读取 Parent 最新「Widget/UI 第二次接续」和 Widget design 的 Events/Graph、共享 owner 合同，只在本域 shared-contracts/parent-handoff/evidence 及生命周期报告追加。真实 source provenance、编译绑定 owner/generation、命令目录/参数与 route/action 消费、结果/error、VM 注册/真实编译和 Source/Graph 差别见 AB-WG-01～07。原始来源指纹与限定搜索命令/退出码在 [widget-graph-contract-01](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/widget-graph-contract-01.json)。

初始 47 项加 7 项追加来源均有 SHA；初始复核无漂移。Widget 合同消息 ACK 为 `01a16dd6-aee2-7ab3-9751-5b5fa1e7f6ae`，Parent 新合同消息被 queue 的 100 条上限拒绝，无 ACK；报告保存完整内容及原输出，不以入队替代已读或实现。Runtime 反射动作只是目录描述，`CallAction` 检查及 `CR-UI-COMP-0002` 的订阅 receiver 缺口也已补入合同；本轮没有修复共享 runtime/host。

12:17:01Z 结束复核中，Widget design 是唯一改变的来源：28737 B / `b1d6209b2f7f4d7afc1a5d400bc150bf4eb2ae0f7932829bcbe3c03778f6cad2` → 29775 B / `30f95da4013fe2e499073bbff2b0e2e99da5b8983d98d53c5fd3f6354d708cfd`。已重读当前 Events/Graph、preview 和共享 owner 合同，确认仍保留实际事件/图后端缺口；原始指纹不改写，其余 53 项来源一致。此并发文档更新未触发编译或跨域修改。

三项限定搜索分别检查 SourceLocator/Goto Definition 入口名、Editor 对 VM operation 注册描述的消费者、Widget/script/VM 范围的 Actor graph 后端名称，均 rg exit1/0 命中；不是命令错误，也不是整个引擎/VM 不存在此类能力的证明。VM descriptor 本身已有 CR-VM-HOST-EDITOR-CONSUMER-0001，和未找到 Editor 消费者的观察相符。实际文本项目编译存在，错误目前在该 plugin bridge 映射为 VmError::Operation 文本；没有据此宣称外部 VM 本身缺 source spans。

本轮仅文档/只读合同核查，按验证政策检查链接、稳定合同编号、源指纹与差异；未触发 Rust、插件、生成、Penpot、EXE 或捕获。前轮 Prefab 四代码路径及 Jenkins 原失败继续保留，未修改任何跨域共享源码或用户 UI 资产，passed/accepted 保持 0。

## 当前插件 wrapper 与域内 identity 核对

Parent route-02 中 baac 的 11 项文件哈希均匹配，但当前 installed 已选 cdd14500；当前 manifest 与六个 runner/客户端文件已按安装清单核验。正式 wrapper 使用当前 selector，无固定旧胶囊或 canonical 入口替代。结果记录于 [plugin-route-02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/plugin-route-02.json)。

本轮只有两次 QueryOnly：第一次直接调用丢失 PowerShell 的 `--` 后返回 `cargo_option_unsupported`；独立参数检查确认后，以数组保留精确参数，当前 pinned parser 返回 `build_scope_policy`（插件 manifest 只允许材质单包/filter）。后者原始 JSON 见 [plugin-authoring-02-query](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/plugin-authoring-02-query.json)。两次均在解析阶段退出，无 source capture/POST/Cargo；不是测试终态或本域 identity 的新授权查询结果。提交日志仅预登记、未创建，原请求及全部旧失败原样保留。

四份既有 Prefab Rust 未改；共享 graph/mod.rs 由材质 owner 独占接续，本会话只读。插件/Rust/原生验收仍 pending，0 passed/accepted。本次不启动或恢复服务、不操作桌面、不改产品资产，不从材质 recipe 借通过。所有新增证据留在既有 E:/cargo-targets 任务报告目录，无源码副本。

## 真实Penpot flow接续与导出404

本阶段实际编辑仅发生在已分配Actor file/page，最终独立读回时间2026-10-24T12:57Z、revn12/vern0、117对象/55 flex、2本域业务主件/12 copies。主件UUID及11:07:45.880Z/.881Z modifiedAt沿用原身份；12份copy的父/组件/文件/主件引用、36个子shape-ref和原夹具文字再核对无错误。检查器最初把原生shapes的逆向z-order当字段顺序，产生12项文字误报；原始误报与纠正原因保留在[flow-05](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-domain-flow-05.json)的verification_attempts。最终按已核实shape-ref关联的主件field身份排序核对，未更改产品数据或原生顺序。

Roboto13.333333实际URI advance=181.8943328857422；640板最终Details初始宽182、Viewport355.2、Left94.8，文本无硬换行。直接API写width后selrect/points与子布局未重算的失败图原样保留；原生宽度控件提交及[真实父布局回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-16.json)确认width/selrect一致、组件/文字/样式不变。随后[两次全新context](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-15.json)均读回revn12、关闭退出，最新[640原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-flow-640-final-native.png)可见完整来源URI、自然换行及1/2/3值。900/1280最新图对应revn9，其列分配未随640修改；不冒称两图拍于revn12。

[flow-04](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-domain-flow-04.json)及flow-05保留实际API请求/响应SHA、原生文本/尺寸操作、8项只读Penpot源码指纹与本轮9张原图逐图观察；总计23图包括所有历史失败，详细表在AB-D08。所有图均是Penpot浏览器UI，无官方图、Engine或双端误差测量；不存在新LayoutReviewCase通过回执。

原生中文“新增导出”及“导出1个元素”已实际点击，640的PNG1×预设保存为revn10；[receipt-11](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-11.json)记录`/api/export` HTTP404、内部错误页和20秒无download，0个PNG产物。最初英文控件定位错误见receipt-10，未发出网络请求。错误页在后续fresh context未复现，设计仍可重开；没有重启服务、借截图充当官方导出或手工生成ZUI。

12:57Z公共文件只读get-file仍revn0/1根对象/0 components、colors、typographies/isShared=false，响应SHA仍`fe0d8e5c3ff7850da27eade99425f4192fa7f8665047b43a506167ff4afcc734`。本域无外部component-file引用；get-file缺libraries字段，未推广为整个服务的已链接库查询。原ZUI仍`a1e7448b621b78c0c511349434a4a55c9239af750a6fc3cc376502fd4c81bbbf`。本阶段未改Rust/产品ZUI或共享源码，未启动Cargo，桌面/DPI/正常EXE均未操作。公共库、导出、现行Prefab包受管支持、真实业务命令和完整窗口仍open；0 passed/accepted。

## Widget Definition 源码核查

Parent WG-C07 的本轮交付为 [AB-DEF-01～08 合同](actor-blueprint-shared-contracts.md)及 [独立源码回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/widget-definition-locator-01.json)，覆盖 48 项精确输入指纹。三项限定 `rg` 搜索分别检查 safe/sys Rust LSP 出口、UI 精确 binding span、现有 Editor/script Definition bridge 名称，均 exit1/0匹配；范围和原输出保留，不推广为整个 VM 无导航能力。两次查找用了不存在的文件路径，已用 `rg --files` / 类型搜索纠正为 `asset_editor_sessions/open.rs` 与 `ui/v2/asset.rs`，未补假文件；查找失败也记录在回执。

真实外部 VM 的 C LSP 有 `GetDefinition`、语义查询和语义快照；同 plugin pin `3c1352ecb6ddfab02752d46682cc376558d5f481` 的 11 项只读 blob 比对证明入口在固定版本存在。当前外部 checkout HEAD `447abb1e8efb44203b353671e9a0417173e16d58` 不是插件运行版本证据；其中 semantic_query.c/build.rs 不同，已分别核实 pin 的语义版本门槛与 `BUILD_LANGUAGE_SERVER=OFF`。safe/sys Rust 没有定义查询出口，ProjectWorkspace manifest 只有 module/hash/artifacts/imports，没有 callable span；没有运行真实项目导航或证明当前 EXE 可用。

UI 当前只有源 node 块索引，缺精确 binding/event 声明范围；VM registry 只能从已登记 callback 还原 module/export，缺 provider-to-source 及持久 class/member/graph 身份映射。源/注册/语义三层 generation、UTF-16/字节位置转换、借用寿命、空结果不明原因和现有 TrackDependency TODO 均逐项记录。Actor 类/接口/构造/事件图后端继续 open，未将 route 建成 Graph 或将 source 模式计为 Graph。

本轮实际修改限三份本域文档与报告/manifest；未改四份既有 Prefab Rust、共享 host/Runtime/Graph、产品 ZUI 或外部仓库，未编译、运行插件、启停服务、调用 Penpot、使用桌面/DPI/捕获。只执行文档结构/来源一致性检查；已有失败、23张 Penpot 原图及2个共126字节待 Jenkins 清理标记继续保留。资源/引擎/双端验收无升级，passed=0/accepted=0。

最终一致性检查首次因两份活动交接文档变化而拒绝：Parent session 与 Widget design 的 hash 已更新，46项技术源码/原交接JSON未变。失败原输出及前后hash保留，不覆盖先前48项观察。已复读当前WG-C07、Parent“Actor 第二次接续”及Widget预览API/共享边界，归属和本次Definition缺口相容；最终按技术输入与并发交接分别核对，不把活动文档冻结或将旧指纹改成新历史。

## Prefab 原宿主读取口与未应用 diff

本轮 [host-injection 回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-host-injection-01.json) 登记50项精确输入与Parent活动记录的单独观察，[三文件 patch](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-host-injection-01.patch) SHA256为 `eadd42e718a3cd1db94e9779dead5e00f0bff2fba93cadd53127f5b65b626066`。只写报告和三份域内文档；未写 extension_access.rs/snapshot.rs/pane_payloads.rs，原四份Prefab Rust与原ZUI字节保持。

已核对真实无参 pane DataSource、target ID 丢失处、template-keyed map消费、committed scene identity、authoring gateway借用、selection与component/schema generation，以及引用来源catalog身份。具体AB-PF-01～06见共享合同；候选只提供原调用者的view准入及私有场景单选组件读取，不能当完整逐窗口生产者/绑定/事务。保留元数据键 `zircon.prefab.instance` 与插件声明路径不同，未建立假别名。当前域serde_json仅dev依赖，未扩manifest或公共trait。

初版patch只读 `git apply --check` exit0；随后使用当前已安装stable rustfmt、edition2021、skip_children与stdout方式，在内存验证三个原文件和候选，六次格式/解析exit0且原文件格式无差异。初版hash/格式化后hash均留回执，不保存源码整份副本；这不是Rust类型检查。引用asset revision、真实producer组合点、旧值清除/关闭取消及同ID重开retention、资产文档与事务authority均未交付，不能把stamp当permit。

原Jenkins Prefab recipe仍由原owner接续，request/operation及Runtime独立lineage不换；未查询/重投/借材质filter或运行local Cargo。本轮无Penpot/PNG导出/服务/EXE/桌面操作，无新增视觉证据或产品通过；原23张图、失败历史及126B待受管清理保持，实际释放0字节，passed=0/accepted=0。

补充资产检查器一度误按 `imports.components` 取引用，实际原格式为 `imports.widgets`；已按原数据纠正检查器，失败与KeyError原输出保存在回执，没有迁移或改写资产。原七文档结构检查已按widgets正确通过。原Prefab聚焦recipe只覆盖域内适配；未应用的共享host候选若由Parent接入，须由其按实际editor模块/调用者另行声明和验证，不借原Prefab回执宣称host通过。
