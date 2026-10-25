# Actor / Blueprint → Parent 交接

## 实现35：原caller可消费的筛选+展开组合

原[visible_filtered_hierarchy_rows](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:287)接受native row matcher和原source/view的u64展开查询，返回既有row iterator；先保留有效匹配路径并重算children，再只访问可见保留分支。未筛选入口共用[私有遍历](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:300)，无caller可注入的rows/新DTO/store/producer。搜索改变时的自动展开/主动选择/根fallback不在方法里，具体消费见[AB-PF-27](actor-blueprint-shared-contracts.md)。

请原生产caller继续原view incarnation/project/document/selection/request/source+expected native key读前/发布前核验，失效只退休旧view，不fresh-key收养旧回调或清新上下文。父匹配但子被裁尽时不查展开，重复ID虽仅显示一条仍Ambiguous，原source_index不是持久target。实体parent不是component attachment。

[结果35](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-visibility-35/result.json)/[审阅35](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-visibility-35/semantic-review.json)：当前42=41普通+1原ignored，原40测试保持，fmt/四源guard0且0errwarn；Rust verified0/actual null。仅authoring/tests推进，lib/snapshot/5保护保持；原02/queue15836/input23f4088d/#387终态与accepted19不改、不query/重投/第三ID。旧SDK发布空档已闭合，无新空档/冻结；真实producer/正式导出/EXE/DPI/双端仍open，无服务/桌面/Penpot/Cargo/helper/清理，释放0B，goal active。


## 实现34：可消费的来源匹配路径方法

原[filtered_hierarchy_rows](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:258)接受原PrefabSourceEntityRow的matcher，返回既有PrefabSourceHierarchyRow；每条原记录匹配一次，只沿既有有效父边保留祖先，原森林序/depth不变并重算has_children。父命中不夹带无关后代，重复ID不因只显示一行而变成唯一target。原authoring/tests之外不改，lib/snapshot和5保护字节保持；原37项可逐字节重建。

请原producer/materializer消费[AB-PF-26](actor-blueprint-shared-contracts.md)，读前/发布前仍核原view incarnation/project/document/selection/request/source与原native key，失效仅退休旧view。UE搜索额外展开/主动选端点/根fallback，与仅收起行为不同；此方法不做选择/焦点、文本parser、UClass继承或特殊Actor root。禁止按name/source_index补持久target或fresh-key收养旧回调。

[审阅34](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-34/semantic-review.json)/[结果34](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-34/result.json)给三项新定义、40=39普通+1ignored、最终fmt/四源guard0；Rust verified0/actual null。原request/session/validate-prefab-source-adapter-02/queue15836/input23f4088d/#387终态与accepted19保持，本次只是当前source revision交接，不Query/重投/改payload/第三ID。原per-view生产、正式导出导入、Normal Actor/150%DPI/双端继续open；本轮无服务/桌面/Penpot/EXE/Cargo/临时helper或清理，释放0B，goal active。


## 设计33：三宽展开/收起/无选择可编辑切片可消费

原Actor file/page已真实保存rev249／4634：15新root、三宽各expanded-selected/collapsed-none/expanded-none及六窄窗Details；原2860非根及两main保持，3720 refs全部有效。Shared5／4mains／34+3未写，三个无选择row仅native覆盖真实recessed style。96文字、12导航、40次保存200、关闭新context完整data相等，三宽fresh图与稳定图字节相同。[结果33](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/result.json)、[15身份/源/override](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/save-reopen-preservation.json)、[28图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/visual-review.json)可审阅；初始日期解码损失和过渡图保留，没有签完整原始日期或A/B。

新的精确UE规则见[AB-PF-25](actor-blueprint-shared-contracts.md)：STreeView隐藏选择默认false，重建后移除隐藏项并Direct通知；再次展开不自动选回。原readonly来源树可采用“清隐藏选择与旧Details”，不等于所有Blueprint Inspector皆空。请原caller在原view incarnation/project/document/selection/request/source及原native key同源校验内消费visible_hierarchy_rows，再重核发布并只退休旧view；不fresh-key收养旧回调、不清新上下文、不注册fixture producer。

两项具体公共缺口：①Shared TreeRow6ddc248b-bd35-80c5-8008-d6c8c7a79944的native disclosure当前仅14×14路径paint可命中，group中心无效，请按真实源hit/focus合同接续，不让整row充展开按钮。②native复制后的窄窗overlay目的地已正确，但position-relative-to仍旧trigger UUID；实际900/640几何已读，正式source/codec anchor仍待owner，不凭位置通过。BODY焦点和正式keyboard未过。

4Rust/5保护不改，复用32静态fmt/guard，37定义=36普通+1原ignored，0 verified Rust/实际null；原02/queue15836/input23f4088d/#387和accepted19原样，不query/重投或建第三ID。原生产publisher/materializer/事务与正式导出、Normal Actor/150%DPI/双端≤1DIP继续open。自有helperexit0／14观察birth退出／profile自动消失；原旧清理不动，手动释放0B，goal active。仅此新设计与合同交接，不是新准入/服务/桌面请求。


## 实现32：来源树展开查询可由原caller消费

仅原authoring.rs/tests推进，[visible_hierarchy_rows](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:234)接受已经绑定原source/view的展开predicate，返回已有PrefabSourceHierarchyRow，不创建状态/DTO/producer。它内部从私有source构造forest，按原序输出可见行；只向可见分支传完整u64，重复/异常根与有效子节点保留。positions只属本次read，不能作持久itemKey；entity parent不等于component attachment。

请在原per-view producer/materializer中消费，仍以原incarnation/project/document/selection/request/source与原native key读前/发布前同源校验；本方法不刷新失效read，不让旧回调采用新key，失效只清旧view。展开/选择/焦点/滚动仍由原host保持，AB-PF-24给出边界。没有改共享host/trait/retention/history或31节点ZUI。

[结果32](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-visibility-32/result.json)/[审阅32](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-visibility-32/semantic-review.json)给出当前四SHA与三新回归；37定义=36普通+1原ignored，最终fmt0/guard0/0errwarn、Rust0实际null，lib/snapshot/5保护保持。原02/queue15836/input23f4088d/#387与accepted19不变，本条仅source revision交接，不Query/submit/retry/第三ID。Penpot30归档复用，真实caller/正式export/import/Normal Actor/150%DPI/双端仍pending；无新服务桌面helper或清理，释放0B。

## 实现31：原reader的实例来源核验可消费

原4源范围内只改authoring.rs/tests，新增[PrefabSourceReadSnapshot::instance_authoring_snapshot](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:156)。请在原真实typed instance/source read流程消费：方法使用实例自己的prefab引用经同project registry解析，原self.version_key expected核验后borrow既有override DTO；异源/manager/kind/registry或旧key错误原样返回，不用fresh key收养旧回调，也不把失败变空来源。

Available只表示本次resident核验；不验证entity/property路径语法、字段/provider/schema、写入或preview ready。仍须原view incarnation/project/document/selection/request/source及原native key读前/发布前同源验证，失效仅清对应旧view。PrefabPropertyOverrideAsset只有两path和值，本域没有按Transform文案发明component target，不扩producer/store/clock/公共trait/history/CAS；AB-PF-23说明消费边界。

三项新回归+原31保留＝当前34定义（33普通+1原ignored），覆盖引用迁移/opaque借用与冲突、foreign/registry拒绝、同ID同revision重建拒绝旧key；fmt0/guard0/0errwarn，Rust0实际null。[结果31](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-instance-source-31/result.json)、[审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-instance-source-31/semantic-review.json)给出四SHA、实际新方法/测试位置；原lib/snapshot/5保护未改。旧29/30按其source SHA保留，当前原冲突回归已移1145，不能用旧line967或accepted19代签。

原02/request/session/queue15836/input23f4088d/#387与所有旧失败不动，不新QueryOnly/submit/capture/换ID或借其它scope。请原authority仅登记当前source revision/测试期待；本条不是编译、续接合同或新intake。设计30归档207/10图复用，生产publisher/typed rows/事务、正式导出/normalActor EXE/150%DPI/双端仍open，goal active。无新服务/桌面/helper/tmp/清理，旧profiles与126B受管清理继续待实际receipt，释放0B。

## 设计30：歧义记录三宽与Details可编辑交付

source29/review29的AB-PF-22已在原Actor file/page做出独立设计切片：Z[0,2,4]/A[1,3]的五条原value/引用全部展示，树按来源记录序列且明确不表示attachment。原生新板1280 `5b8f1971-076a-80d3-8008-d8195644b955`、900 `5b8f1971-076a-80d3-8008-d81cf44b0ab6`、640 `5b8f1971-076a-80d3-8008-d81d0233ffa9`；两原生Details目的地及component/main UUID见[结果30](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/result.json)与[身份保存重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/save-reopen-preservation.json)。

Actor176→207/2861，五root新增591，原2269非根与2main精确不变，2205来源引用全部有效（356本域/1849 Shared），Shared5 data exact；保存关闭/新context/交互末次data exact。10原图均已打开、7最终图复核，三宽100%/DPR1；900边差0、640上下−0.015625/右0。640完整位置/native-key边界经实际toggle可达，继承hidden区的最初误判已补正，[640最终开图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/prototype-640-open.png)与[逐图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/visual-review.json)可审阅。BODY焦点不算返回。

本域仍仅交readonly语义和设计，不注册fixture producer、不把序号/描述/文案当持久target、不改公共源。Parent原caller继续真实view incarnation/project/document/selection/request/source及expected native key读前/发布前同源核验；失效只清对应旧view，不能重绑旧回调或清新上下文。正式per-view publisher/typed materializer/事务与导出/正常EXE另接，不重放旧全量host patch。

实际新字体观察：完整ID custom Roboto21glyph；SVG中文标题system Noto Sans SC13glyph、Details mixed177glyph，声明family仍Roboto。DOM rect16px/computed lineHeight19.0666px不等于glyph ink/Engine字体ready；不盲改Shared/provider。当前4Rust和5保护保持29同字节，31静态定义/0 verified Rust，原02 accepted19/input23f4088d/#387/queue15836与全部失败保留，无新query/提交/第三身份。

helper正常exit0、20观察birth均退出，正常profile消失；首失败profile156文件/11413749B仅受管清理提案，旧26/28/126B保留，手动释放0B/自动物理释放未量测。公共/原ZUI/服务/桌面0写入，normal Actor/150%DPI/双端≤1DIP仍open，goal active。

## 实现29：全部歧义组件位置的readonly投影

原四源新增 `PrefabSourceComponentConflict` / `PrefabSourceAuthoringSnapshot.component_conflicts`，按本次borrow的owner位置及首次component位置输出全部同描述位置，完整复用type/schema/version/provider相等；不同payload/references也保留，重复entity IDs不跨owner合组，不判Prefab非法。[AB-PF-22](actor-blueprint-shared-contracts.md)和[结果29](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-component-conflicts-29/result.json)可消费；仅原borrow诊断位置，不能作持久row/scene target或令Apply/Delete生效。

Parent原typed materializer可据此标注全部歧义记录并显示明确提示；仍须原view incarnation/project/document/selection/request/source及expected native key读前/发布前同源验证，不用fresh key重绑或清替换后的新view。未扩公共trait、schema、store/clock、producer或重放旧host patch。当前四源+5保护、3新增回归/31静态定义的精确源与[审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-component-conflicts-29/semantic-review.json)独立交付；格式/guard0，Rust0/实际总数null。

原02/request/session/queue15836/input23f4088d/#387和accepted19保持，不QueryOnly/submit、覆盖输入、ResumeOriginal或第三ID；只向原authority交新revision，不把原unknown/中断说成当前已准备。Penpot28rev176与14原图保持source27设计身份，新冲突显示未冒充已接；真实publisher/typed rows/事务、正式export/import、normal EXE/150%DPI/双端仍open。无新临时/服务/桌面/清理，旧profile/126B不动，释放0B，goal active。

## 设计28：source27组件来源与opaque载荷可编辑设计交付

原分配Actor file/page新增1280 `d79ec46c-7a4d-80ce-8008-d8083ee3a17b`、900 `d79ec46c-7a4d-80ce-8008-d80a34d4846a`、640 `d79ec46c-7a4d-80ce-8008-d80b0d218d01`及两窄窗Details。内容严格是source27 reverse=true单元输入：同名实体0/u64MAX、parent None、原native描述4字段/opaque payload/引用；其它lookup结果单列，不假接provider/场景/写事务。

Actor156→176/2270，新591对象均在五root内，原1678非根对象和2main精确不变；1700来源引用全部有效，77本域copy+131 Shared copy，Shared5 data exact。保存/关闭/全新context/末次API data exact，原7张最终100%三宽及开关图均已打开复核。900弹层三边差0，640上下−0.015625/右0；焦点仍BODY，不计焦点返回或Engine通过。请读[结果28](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/result.json)、[最终身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/save-reopen-preservation-final.json)、[640图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/fresh-640-final.png)与[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/visual-review.json)。

辅助font probe的Enter编辑入口插入两处新文字空段落，失败数据/图完整保留；最终精确文字与逻辑行检查、原生宽641→640和fresh reopen恢复640 y112/h376，未改原板或Shared。source字体及实际glyph证据保留，19ink/16linebox、Engine字体、正式export/import、normal Actor/150%DPI/双端仍open。

4Rust与5保护源不改，复用27同源fmt/guard，28定义无Rust执行回执；原02/queue15836/#387不query/submit或换身份。Parent继续原生产publisher/typed materializer及per-view context/native key前后验证，不重放旧全量patch；本域未碰公共源、服务或桌面。恢复helper正常exit0，20观察birth无剩余，正常profile已消失，首失败profile153文件/7215255B仅交原管理owner精确hash提案，旧126B/phase26保留、释放0B。goal active、passed/accepted=false。

## 实现27：只读Details的精确组件解析可消费

原四源范围内只推进authoring.rs/lib.rs/tests三文件，新增 `PrefabSourceComponentDescriptor` 与 `PrefabSourceReadSnapshot::component_by_descriptor`；snapshot树源和5保护源不改。查询先解析完整u64实体，再比原native记录type/provider/schema/version；重复实体/重复完整描述分别拒绝，成功借用原opaque payload/references，无provider加载/字段解码/新实例UUID或写权限。详见[AB-PF-21](actor-blueprint-shared-contracts.md)与[结果27](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-components-27/result.json)。

请原typed Details consumer在已核原view上下文及expected native key后使用；此描述只是定义字段组合，模型未提供的组件instance identity仍缺，不能当持久row/scene Delete或Prefab写target。同描述/同数字revision的资源重建必须拒绝旧key，不用fresh read重绑。生产caller/materializer、provider schema/typed字段、history/事务继续由既定owner接；本域未碰公共入口或重放旧host patch。

三项新回归覆盖精确字段/完整ID/顺序、五种结果、重建后原key失效，当前28静态定义（27普通+1原ignored），fmt0/guard0，仅静态无Rust回执。原02/queue15836/#387及19项受理版本不改，不QueryOnly/submit/第三ID或伪装batch。设计26rev156/三宽图复用，不称此API已接运行中的窗口；正式导出/normal Actor/150%DPI/双端与旧managed cleanup仍open，goal active、释放0B。

## 设计26：source25层级诊断三宽内容已保存重开

在原Actor file/page原生复制既有connected内容，新增1280 `67885b70-d0cd-80bd-8008-d7f5a8a6924f`、900 `d98468eb-9a4e-8069-8008-d7fb991b008a`、640 `d98468eb-9a4e-8069-8008-d7fcb0594c00`及两窄窗Details目的地。内容精确来自[source25测试](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:663)：七实体、六根、四诊断，11的有效后代13保留缩进/选中/源Details。应用/校验不可用；明确单元夹具，未注册固定producer或按文案派发。

Actor138→156/1679，save/close/new context/final data exact；591新对象在五root内，原1087非根对象与2main完全不变，1195引用全部有效（260Actor/935Shared），99 Shared根，Shared5 data不变。640初次抽屉留26像素残边，已原生宽320→346/marker左移26覆盖整诊断列，旧PNG保留；两窄窗实际点击开/关，900右/底差0、640底差0.015625px。焦点BODY未算通过，混合字体系统Noto/Times回退仍open。请消费[结果26](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/result.json)、[引用/重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/save-reopen-preservation.json)、[640最终Details图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/prototype-640-open-final.png)及[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/visual-review.json)。

4Rust/5保护源不改，复用source25 fmt/guard、25静态定义无Rust回执；原02已受理19项与#387 unknown/interrupted保持，不新query/intake或重投。本域只交readonly稳定key/同次树次序；真实per-view context/key前后核验、typed rows、展开/选择及发布/失效清理仍由原Parent caller接续，未重放旧全量host patch。正式源导出/正常Actor/150%DPI/双端≤1DIP未完成，goal active。

首自有helper因遍历未终止强制exit1，恢复helper正常exit0、25个观察PID/birth无剩余，新profile不存在；旧profile11,413,746B保留，仅按156文件hash计划queue原管理owner，msg01a172da-0f62-7eb3-a1b6-226413eecbbc，不是server维护收件。释放0B，不称清理关闭；旧126B及原服务/桌面/State/终态不动。

## 实现25：来源森林顺序供原typed materializer消费

已在原四个Prefab源文件补 `PrefabSourceHierarchyRow` / `PrefabSourceAuthoringSnapshot.hierarchy_rows`。原实体行保持序列化顺序与原引用，额外序列只携带同次源索引、完整深度和 `has_children`；根/同级按源顺序，缺失/歧义/循环父级不隐藏实体，有效后代仍可达。没有组件attachment或假根节点。消费次序与稳定身份见[AB-PF-20](actor-blueprint-shared-contracts.md)，请在原per-view调用者核验上下文及原native key后消费，不用fresh key重绑旧回调，也不按Mesh/LootSocket文案派发。

四源最终fmt/guard通过，当前25静态定义（24普通+1原ignored）/Rust无执行回执，5保护源及原31节点ZUI不改。[结果25](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-hierarchy-25/result.json)、[源身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-hierarchy-25/source-after.json)与[审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-hierarchy-25/semantic-review.json)为本轮产物，不覆盖source08/review09/17或原02受理源。Parent保有真实producer/typed row/展开选择/发布与失效清理，本域未写共享源或接生产publisher。

已消费原管理owner14830B/db66bb23…终态对账：原02/queue15836=#387 FAILURE，State v14 unknown/interrupted，原input23f4088d和554B archive保持。六次资源容量等待及后续观察中断不归责当前文件；不把缺native行当终结证明，不重投02、不改claim/State或另建第三身份。本轮没有新query/intake/Cargo/冻结；原60秒接收空档已到期，无持续限制。Penpot138与正常EXE/正式导出/DPI验收分别保留，goal active，整族不计通过。

## 设计23：三宽来源字体与640原弹层接续完成

Actor130→138/1088，原三mode改为纯中文并选源custom Noto400；各13个glyph实测NotoSansSC-Regular，原13.333333/line1.2/Shared text.primary pair保留。640 mode用native Layers从actions移到root单独flow行，宽128→624、高32→16；main y88→112/h400→376，footer y496不变。Inspector默认14与API实际值不同，没有据此补签格式丢失或换Google字体。

原640 Details toggle/destination只改manual位置与overlay高度400→376，native viewer实际点击开/关，overlay/main bottom差0.015625px；focus回BODY，键盘返回仍open。保存关闭/新context重开data exact，80对象变、1008不变；67 Shared heads/690 refs（212本域/478Shared）/2main/source/slots/touched全保持，Shared5 data exact。旧640失败及900 overlay保持，640 overlay12子节点不变；13原PNG已逐张复核。

请消费[结果23](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/result.json)、[字段保存重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/save-reopen-preservation.json)、[字体](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/font-probe.json)、[原弹层](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/prototype-anchor.json)与[640预览](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/preview-640.png)。没有公共源/Shared/main/codec写入，旧source allocation/paint guard不重签，不把本地flow/font覆盖当正式ZUI导出。

4Rust/5保护源保持；23静态定义仍无Rust执行回执，复用同源format/guard21。本轮仅一次原02 QueryOnly仍unknown/pending management_operation_interrupted，exit2、input23f4088d/resubmitted=false；不重投、不覆盖19项accepted源或造第三身份。实际per-view incarnation/project/document/selection/request与原key发布/清选仍由Parent接续，本域readonly合同不变。Node34120正常exit0，11观察身份gone、profile absent，手工释放0B/自动回收字节未知，旧失败保留；产品/正式导出/EXE/DPI/双端均open，goal active、passed/accepted=false。

## 设计22：三宽真实内容实例已保存重开

已有1280源内容板及900/640 compact板原生修改，Actor125→130/1088；关闭/新context重开、末次读取均130/data exact。39个part变化，其余1049对象精确不变：Search surface/caption254/218及左右约束；来源树无场景目标，隐藏24个eye/lock/槽，caption190/178；原unit叶行隐藏3展开符。未改root/body flow或以文案/width作runtime target，Rust仍完整u64/重复拒绝。

67 Shared根、690引用（212本域/478Shared）、2业务main与所有source/slot/events/文本/字体/拓扑保持；Shared5 data exact。原640失败95节点和两份13节点Details overlay全同。[结果22](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/result.json)、[保存重开字段](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/save-reopen-preservation.json)、[12图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/visual-review.json)及[640真实预览](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/preview-640.png)可直接查看；三张preview均native Reset100%/DPR1 live-vector crop，保留选框，不是正式exporter或Engine证据。

请公共owner只接收具体消费合同：Search leading固定左、surface与caption伸缩；来源TreeRow不显示无目标场景动作，leaf无展开箭头，label消费剩余内容宽；现有r02 worker/qualified SDK/正式source guard迁移继续由原owner接续。本轮没有改Shared/main/codec或旧allocation/paint guard，不能把local override当export通过，也不要求完美Field/Toolkit才继续。

4 Rust+原ZUI与保护源本轮不改，当前23定义仍无Rust执行回执。没有新query/intake/capture/Cargo/桌面/服务；原02已受理queue15836、input23f4088d及unknown/interrupted末读保持，交原authority对账，不换ID/盲重投。真实per-view发布/事务/正常EXE/150%DPI/双端≤1DIP仍open。Node15076正常exit0，10观察身份无剩余、profile已不存在，释放0B；旧失败与受管cleanup保留。goal active，产品passed/accepted=false。

## 只读产品切片21：唯一来源entity查询合同

新增`PrefabSourceReadSnapshot::entity_by_id(entity_id: u64) -> PrefabSourceEntityLookup<'_>`。Unique直接借用该读取私有原生载荷中的`SceneEntityAsset`；Missing与Ambiguous不返回实体，重复ID仍在原authoring rows/diagnostics中完整保留。名称/顺序/source_index从不作为输入，不增UUID/store/clock/producer或通用写target。原typed来源错误与expected native key合同保持。

Parent接线顺序为保留读取→核原project/document/view incarnation/selection/request→revalidate原native key→从返回值做entity_by_id；发布/清选前仍用原context与permit/CAS/clock重核。失败只在同一原context尚成立时清旧目标，不清替换后的新上下文。返回值是来源内读取，不是场景实例entity/组件attachment身份或任意Delete/Apply授权。无需为本API修改公共trait、pane cache或重放旧全量patch。

2新回归后当前23定义=22普通+1原ignored，验证同名/重排/u64MAX/零与重复/缺失/诊断行保留；原空来源测试增加零ID Missing。最终格式与4源guard通过，Rust执行回执0。[结果21](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-selection-21/result.json)、[语义审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-selection-21/semantic-review.json)及[精确源身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-selection-21/source-after.json)给当前合同；snapshot.rs与5保护源保持。

未重复查询或重投原02，未改accepted input/原1801B候选，原19项已受理、实现20的21项与当前23项不互相代验。请原authority仅沿既有受理链核native/资源/合法source验证接续，本消息不是新intake/capture指令。公共producer/materializer/正式import仍由Parent/public owner接续，本域不停工等待；未改原ZUI/Shared/Penpot、服务或桌面，产品/EXE/DPI/双端仍pending，goal active、passed/accepted=false、释放0B。

## 只读产品切片20：可消费的来源读取合同

原三路径authoring.rs/lib.rs/tests新增并公开`PrefabSourceReadSnapshot`，不改公共host或原snapshot.rs。`read(source, registry, resources)`成对封装原native key与载荷；`version_key()`/`source_snapshot()`保留实际身份与record；`authoring_snapshot()`从其载荷借用原实体行和opaque组件；`revalidate(source, registry, resources)`只核原key，成功返回同版本读取，失败保持原typed诊断。Ready-empty为成功零行，NotResident与Stale为不同错误，不增新store/clock/UUID/producer或可伪造的字段构造器。

Parent后续caller可先保留此读取，在原document/view/selection/request尚一致时调用revalidate，再从返回值借用行；实际发布前仍须用原permit/CAS/clock核全部context/native key。该DTO只解决本域key/payload混配边界，不解决无context snapshot、template-keyed缓存、每view incarnation、虚拟行materializer或原子发布。不可直接用保留对象的旧行冒充当前值，也不重放旧整体patch。

两项新回归防止“失败变空来源”和“移除重建时默换版本”；原lossless行测试增强为真实payload Arc/key及借用指针核对。当前21定义=20普通+1原ignored；最终rustfmt与4源guard通过，无Rust执行终态。[语义审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-resident-read-20/semantic-review.json)、[当前源码](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-resident-read-20/source-after.json)及[切片结果](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-resident-read-20/result.json)给精确SHA和调用边界。

既有02本轮仅一次QueryOnly仍unknown/pending management_operation_interrupted，input23f4088d/resubmitted=false；[原查询](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-resident-read-20/operation-query.json)保留。本修订21项不在原19项已受理源中，不替换accepted payload或重投原操作。请原authority沿原State/delivery/queue/build/native链审阅新source revision后决定合法验证接续，不造第三ID、借别域身份或要求用户手工grant。本域继续独立推进，Shared/producer/SDK/EXE等缺件单列pending，不令整族停工；passed/accepted=false。

## 设计19与原02实际受理后的交付

本域继续执行，goal active。真实Actor file/page保存关闭/新context重开rev125/1088，新增三张stale expected-source板：1280=`5af79cea-4b1f-8012-8008-d7d3386792c8`、900=`5af79cea-4b1f-8012-8008-d7d45b64c5ac`、640=`5af79cea-4b1f-8012-8008-d7d4b65b3a08`。[结果19](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/result.json)保留原992非根对象/2main/旧板顺序，67 Shared roots、690 source refs均有效，Shared认证读5/data保持。20新编辑文字实际source Noto400，未套Google同名字体或改Shared。

已实际复核[20张原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/visual-review.json)。fresh122暴露900标题交叠与1280来源副本高度不足；通过原生Layers mode重排、仅自有copy vertical fixed→auto修复至125，普通内容无坐标偏移。最终文案仅称旧key失效，不凭Stale判定来源更新原因。三板仍为unit状态，不注册fixture producer，不把按钮文案当handler或生产清选完成。

原02本轮已实际受理queue15836，accepted inputDigest`23f4088d…`，1801B/`86b638a0…`候选及#348不变。两次resource_capacity后，最后实际QueryOnly为unknown/pending、`management_operation_interrupted`，消息HEAD/full index/source membership在capture变化；[原authority对账交接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/resume-execution-17/interruption-reconciliation-handoff.json)已实际queue投递Jenkins owner及Parent，仅记投递。请沿已有State/delivery/queue/build/native链认定最终结果；本域不重投、不改payload/第三ID，无Rust执行终态可认定。四Rust保持实现17的19定义，原5保护源/ZUI不变。

本段与另四本域文档为上述中断实读后的新修订，旧受理时元组保留；新[有限源身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/source-after-docs.json)不等于九root membership/writer或capture-ready，不归责历史漂移。继续独立设计与校验；真实producer仍需Parent生命周期注入同项目registry/resources及view/document/incarnation/selection/request，消费本域expected reader并发布前重校验，不重放整体旧patch或扩公共trait。

成功浏览器helper均exit0，当前9个已观察birth gone；首轮失败profile新增153文件/7,214,825逻辑B待原受管清理，手工释放0B。没有新桌面/服务/Shared/host写入，原ZUI官方导出、正常Actor EXE/真实数据/焦点撤回、150%DPI与双端<=1DIP继续open，passed/accepted=false。

## 监督恢复后实现17与设计18交付

新增本域`prefab_source_snapshot_expected(source, registry, resources, expected)`，只改原四路径中的authoring/lib/tests，snapshot.rs不变。函数沿真实UUID/目录类型重解析，由ResourceManager同锁核原native key与载荷；其它asset/manager、移除重建明确Stale，不自动切新版本。3新回归后19定义=18普通+1原ignored/0执行；[当前语义审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/resume-execution-17/semantic-review.json)和guard已落盘。历史source08/final09、02候选与#348保留，不扩公共host/store/schema/事务。

Parent生产接线可消费此expected入口；调用方仍要验证真实每view/project/document/incarnation/selection/request，成功后发布前重核context，失效时清旧树/Details/选择。不要重放原整体host-injection patch，也不要把reader当producer完成。原5份Parent/ZUI保护源未改，类/事件图与Definition实例定位门保持。

原file/page保存关闭/新context重开rev112/993：六状态板mode leaf改source Noto400和“来源定义：尚未选中实体”；三宽各两状态18图已看。仅6对象变化，987对象/2main/58Shared根/653引用与Shared5保持；六fresh SVG各11glyph为custom NotoSansSC-Regular。[设计18](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-local-cjk-status-18/result.json)保留字形19/行框16，不声称全334Han、Shared继承或Engine1DIP。

原02同身份QueryOnly实际missing后normal准备已开始，调用必须保留字面量`'--'`；首轮九根capture到Engine52235文件后identity变化exit2，无POST/Cargo，详[原输出](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/resume-execution-17/submit.log)。当前19项源审阅独立完成，未来同02继续实际准入，不改grants/sealed或要求用户手工grant。无新公共写入、桌面或服务操作；正式导出/normal EXE/DPI/双端仍open。goal保持active，继续本域实现与设计。

## 窄窗来源说明16：原生编辑、保存和重开已核对

原分配Actor file/page的两张640状态板保持UUID不变，原生Layers拖放将既有document-mode文字从actions移到根column，位于actions与workspace-main之间。原文“来源定义 / 无选中实体”从128×32变为624×16、完整单行；两Button仍240宽，不改Shared内部宽度、main或样式。主区通过fill由416变392，高度24来自正常flow，不写普通内容x/y。1280/900和其它板的935对象精确不变。

Actor真实API107→109/993/2main/58Shared roots；Shared仍5，原653 refs（445外链+208本域）全部解析。源码4+Parent/ZUI5项逐字节匹配前次，16静态定义15普通+1ignored、Rust0。既有02的1801B候选不变；此次4份本域文档更新使15的有限11-path元组成为历史，新的本域元组在16 metadata登记，不当九root source capture或已受理payload。

[设计与差异证据16](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-status-16/result.json)、[六图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-status-16/visual-review.json)及[保存重开身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-status-16/save-reopen-preservation.json)保留全部UUID、原文、共享覆盖与native path y+24变更，未重新签source metadata/引入组件或主题。这是Ready-empty/NotResident的unit状态设计，未接真实生产publisher/操作/undo。Penpot来源身份包络仍缺，正式ZUI导出、正常Actor EXE、150%DPI、焦点/滚动首尾和双端≤1DIP仍open，passed/accepted=false。


## 原资产导出接入15：31节点逐项对应与明确的首个导出前置条件

本轮独立认证GET确认Actor仍rev107/993，data与14重开全同；Shared仍rev5/4main。原31节点、29control、15events、7直接widget导入已逐项审阅，原ZUI仍a1e7448b…且没有binding/repeat声明。[完整逐节点语义对应](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-export-map-15.json)记录全部源ID/事件及六板的实际shape对应；这些是语义关系，**不是合法source identity map或ZuiSourceExportManifest**。

发现具体导出前置条件：新六板role/bridge-version/document均无官方源资产包络，全文件0个shape的sourcePath属于原extension_prefab_editor_workspace.zui；Shared native leaf映射只属于各自primitive，不能代表模块Apply/Validate/control/source所有权。现行plugin selectedAssetBoard只接受role=asset，随后要求bridge版本/document；源码restoreOriginalZuiSource还拒绝未映射的component变更。不要手工复制metadata、重签baseline或把新布局板直接应用原资产。请Parent/原pipeline owner先交实际qualified SDK/release、canonical导入/owner上下文及依赖/slot合同；本域再在自己的Actor文件执行原资产导入并记录真实board/map/依赖SHA，保留现有设计和Shared实例，不补重复公共main。本条不授公共owner跨写Actor或其它族。

源七导入中Shared精确canonical匹配只有WorkbenchButton和WorkbenchPropertyEditorRow；Dropdown、Field、ListRow、SectionTitle、TableRow尚无对应已发布UUID。设计使用的TreeRow/Search与原ListRow/Dropdown不同。public owner接真实缺件/slot/闭包或由Parent审阅明确source迁移，不能按同名替换UUID或把Search塞入Property.value。原根是嵌入式Overlay模块body，含rail/right-reserve/drawer；当前设计还有外层document标题/操作条，必须由Parent DocumentHost边界分开，不能整板几何覆盖模块源。

新增AB-PF-18/19明确真实数据到行的第二个边界：UiComponentProjectionPatch只有control attributes/state，原模块没有repeat。现有私有TemplateBridgeVirtualRowSequence要求已有repeat/authored_count=1，并非plugin公共API；TreeView summary painter只取8个字符串、深度夹0～3且没有持久itemKey，不能借它宣称完整树/滚动/选择完成。Parent复用真实row materialization/每view lifecycle，域内继续借用原Prefab记录和原key/context重校验；不扩pub/schema、复制materializer或用source_index/文案发命令。

[原插件host实证](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-export-map-15-host-observation.json)：r03已于22:06:58.508619Z因recorded-deadline正常停止，原root9968/birth134373505299703071 exit0、Job afterTerminal/afterClose0、双EOF/reader/waiter/request-thread joined；当前原birth不在/4213无监听。本轮不恢复、延长或借unknown19002，正常截止不是现源构建/SDK导出或产品验收。无新浏览器、临时目录、服务/桌面/Cargo/Node构建/Jenkins query/submit/capture/freeze，释放0B；旧18,359,439B限定残留照原受管维护保留。

4Rust/5保护源原字节保持、16定义0执行，原02/#348/14交付保留。这里只改本域3文档和E证据，新的有限11路径身份另记metadata，不当完整source-ready。正式导出/生产caller与typed rows/写命令/Graph/normal EXE/实际150%DPI/双端仍open，passed/accepted=false。

## 来源状态设计14交付：实际六板与per-view生产缺口

本域原file/page保存关闭/新context及独立API均rev107/993对象，Shared rev5/4main/34+3 data全同。新增Ready-empty和NotResident各1280/900/640，共190对象、18Shared实例；58外链根、653有效引用（445Shared/208本域）。原802非根对象、两本域main及旧板顺序精确保留。真实6board/source/组件UUID见[完整结果](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/result.json)，保存与覆盖见[重开回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/save-reopen-preservation.json)。

[18图逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/visual-review.json)中最后6张fresh均已打开：数量0与未知区分，旧实体/组件/选择已从新空态板移除；wide保留来源URI，compact没有通往旧有选择Details的入口。初版flow及header0.01重叠/MinH无效和中途记录异常均保留，最终native flow居中/compact标题直接root。640 mode说明仍拆词，两种状态未计完整窗口或1DIP。Apply/Validate不可用是设计意图，搜索/事件/禁用handler无产品证明，不以无效果按钮结项。

这是source08/review09静态定义的unit状态，16定义/0执行；没有生产asset注入。沿Parent已指出的no-context snapshot/template-keyed batch缺口接续真实per-view producer，使用原同项目registry/resources及document/view/incarnation/selection/source/request；发布前核原native key和全部上下文。NotResident/缺失/rejected应清除旧行/选择而非复用缓存；Reference/wrong kind/SnapshotRejected须保留诊断差别。空Prefab不冒充Blueprint根或组件attachment。此轮不请求新共享组件、公共trait/store/时钟或fixture producer。

4Rust及5保护源均匹配，原31节点ZUI未改。无新parse/query/submit/capture/freeze/服务/桌面；原-01/#348和已选02/input/09/13原字节保持。6文档更新后，13有限11路径tuple成为历史，当前有限元组另记source-metadata14，不是全9root/source-ready。无需重复旧资格或让用户grant；实际client/runner资格仍由原管理部署owner交付。

浏览器正常exit0，合并18个出生身份gone；首失败profile新153文件/7,206,915B与旧phase10/126B均待原受管维护。逻辑样本自动消失11,393,483B、物理未测/手删0B，无新维护身份。source-owner导出到原ZUI、实际binding/write/history/Graph、normal Actor case/EXE/150%DPI/双端仍open，passed/accepted=false。以下13/12及此前记录保留原时点，不当当前全族验收。


## 后继入口对账13：已选02，实际client与runner合同仍未闭合

[本域实际对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-successor-route-reconcile-13.json)已核原owner两报告的SHA与来信一致：22:27 a928的pure parse确实cargo_option_unsupported/No Jenkins recipe for Cargo argument: --manifest-path。它是入口能力阻塞，不是Cargo失败或缺用户grant。原-01/#348/queue14345 complete/preparation-conflict、prepare/compile=false与input/marker/terminal均保留，不重投。

本域随后实读installed为5fc41dbe458cd4d030467a779c50e71153643a539f4f317b33faaa5811533b0c，同fc553228…/start-1792836412672596900；capsule.json及11成员逐字节匹配。实际client仍57153be4e44a844556248f47be1c266dd2741ddd9e4b1233365ec915e113239f，正是原owner失败parser字节：parse_cargo 354起的options表无manifest flag，437为unsupported。实际runner b38a18a9e49e7e0c37aed733c391b04fccf06699c7a93561fc390b1ba06278b0 已在4786提供Prefab prefab_、4805插入manifest。此为静态合同不一致定位；没有对5fc新跑parse/query或server调用，不宣称runner执行、闭包或生命周期通过。后续selected变化须重绑事实，旧e063/6d12资格不移植。

请原管理/部署owner在实际selected恢复兼容的bounded nested-manifest client、runner、source-preparation/manifest-lock-config-path依赖合同并给资格回执。本域不改public entry/canonical/sealed、filter或payload，不丢manifest，不借repair/material身份。当前source-owner review09为原31189B/de321763…完成态，逐项审08四源及16定义；原四SHA31e09855/0fd7e4f5/94efaab3/adc17702和5保护源9/9仍匹配。16=15默认+1原ignored、0执行，不要求重复旧13项审阅。

既有validate-prefab-source-adapter-02已选，原1801B/86b638a0…输入与-01仅operationId不同；无第三ID、ResumeOriginal或attempt转换。原owner22:27精确两repo keys为0行/确定marker不存在，只限本次有界观察；没有自动intake/queue/attempt。完整9-root/member/writer/共同稳定窗/pool-cache writer/ResourceManager reservation仍无真实准入，captureReady=false。

本轮仅3本域文档和原E根报告；原09/12报告与Penpot14图不改。12的有限775c9c84…元组随文档变化成为历史，当前有限身份另记，不当完整source。无Cargo/query/capture/freeze/源码副本/服务/State/grant/桌面/清理，释放0B；原126B及phase10旧profile保留待原受管维护。Rust/产品/EXE/DPI/双端仍未通过，passed/accepted=false。

## 紧凑Details设计12：实际交付与宿主接续

原file/page保存关闭重开rev94/803对象、2本域main、40 Shared roots，579引用全有效；原606非根对象/main/来源文字引用精确保留，Shared rev5 data未改。新增两组640/900主板+Details，共196对象/20原生实例，只新增两个真实Shared Button副本。[完整结果](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/result.json)、[14图逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/visual-review.json)及[点击状态](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/viewer-interactions.json)供实际读取；累计87图，旧640失败板不变。

确认已有canonical responsive_compact_drawer="module_details"/minwide，真实host responsive visibility及WorkbenchModuleDetailsDrawerToggle / Workbench/ToggleModuleDetailsDrawer / workbench.module.details_drawer.toggle。设计沿此双栏+Details，不再请求新公共Drawer；中心346/606、Details320×400/500。新板/抽屉UUID及Button ID详AB-D12。native click开/外部不关/同开关闭合，返回SHA相同；自动内frame锚点及manual原点失败图保留，最终仅弹层按实际flow目标/真实源frame计算。SVG g无role/tabindex、BODY前后不是focus证明，不能把prototype当handler。

请沿原分工核真实Actor host中toggle控件存在、能力/生命周期和焦点，再接per-view来源publisher/context重校验及实际数据；本域readonly adapter/原4Rust不变，不补fixture producer。source-owner导出、SVG/画布字体外观、TreeRow能力/合法value、normal EXE/DPI/双端仍open，无新公共源码/服务/桌面写权请求。原#348与未提交-02不变；6文档变化使phase11有限11路径元组也为历史，当前metadata不替代完整writer/source/pool/cache/reservation/native EOF。

浏览器exit0、9 births gone/temp0；自动消失逻辑样本11,393,979B、物理未测/手删0B；旧11,152,398B失败profile+126B保留。仍是unit fixture设计，Rust/产品/EXE/实际150%DPI/双端≤1DIP/accepted均false。不回写旧失败或要求例行ACK/新grant。

## 来源工作区内容11：实际交付及所需合同

原file/page已rev78/607对象、2本域主件/28 Shared roots、441有效shapeRef（273 Shared/168本域）；Shared仍rev5/34+3，data与基线全同。新增1280/900/640三板各95对象/9native copies（5 Shared控件+4本域数据），17处文字覆盖保留touched/source/ref。原321非根对象/2主件/旧板及原body、color leaf精确保留。board UUID、source/override/flow、保存关闭/新context及7张逐图记录在[内容11](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-prefab-workspace-11/result.json)、[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-prefab-workspace-11/visual-review.json)及[重开API](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-prefab-workspace-11/after-reopen.json)。

1280/900显示08测试的两同名实体、完整u64 ID、未激活子实体、两条重复opaque记录及中文mesh/unknown数据，board/title明确unit fixture，不是生产数据或场景预览。640为失败反例：中心18宽、render高1806/viewport520。请公共host/pipeline沿既有Dock/Tab或支持的scroll合同给出窄窗进入/返回Details、记录首尾及focus的接入面；本域不复制公共控件或画假切页关闭。TreeRow动作灰底、leaf disclosure、内部宽度与source-owner export仍归public owner。Apply/Validate不可用，无handler/写事务；完整Toolkit、生产caller、normal Actor case/EXE/DPI/双端均未通过。

本轮Jenkins owner新packet与已存在09审阅的区别已投原owner，队列01a16fc1-929c-7353-bed8-6e15ba47e53b仅queued。原packet讨论-01/#348终态及08，09当前四Rust语义审阅已存在；Parent选择原active request下**现有候选-02普通独立新操作**，不是-01同operation续跑。08/09/input/候选/#348均保留，不新造第三身份或改已受理payload。此次又改6份本域文档，09旧11-path及phase10的48f0…有限元组均属历史；当前有限元组登记metadata，仍非九root完整capture-ready。16定义/0执行、完整writer/source/pool/cache/resource/native EOF门继续pending。

浏览器exit0，9 birth gone，本轮temp0，自动消失逻辑样本11,394,203B，物理未测/手删0B。旧phase10失败profile+126B保留待原受管清理，锁文件盘点失败与r2保留。没有Rust/原ZUI/Shared/公共入口或服务桌面写入。

## Shared rev5原生消费10：实际传播与剩余缺陷

已在原分配file/page消费实际Button发布，Actor rev48→49/322对象，Shared实读rev5/4主件/34+3。原生Update→保存→关闭context→fresh context及独立API后，4个Button根取得`editor.surface.1`/`599586e3-022c-500b-ac80-5d9c490139af`真实Shared引用；原颜色仍#242424。240个linked对象全部解析（120本域+120Shared），全部身份/文字/source/slot/event/touched和覆盖保留；原body及另一local-color leaf全对象精确不变。Shared data前后全同，没有改main/日期/syncedAt造传播或detach。

103对象有native精度/文本positionData变化，完整delta保留，最大数值差0.3800048828125，28个path的命令序列/参数数不变。不能称全对象exact preservation、视觉A/B或双端1DIP通过。5张新增原图均已打开；640/900/1280重开图仍见内部240宽label居中及树标题换行，仍是稀疏consumer板，不是完整Prefab工作区。证据及逐图记录见[同步10](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-sync-10/result.json)、[fresh API](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-sync-10/after-reopen.json)，Shared真实Button/main/color/source身份均在内。公共内部flow、合法Property.value子件、字体provider和正式source-owner导出仍由原public owner接续，本域继续完整内容设计。

本轮4份Rust及检查的Parent/ZUI源均未变，08/09/旧input/未提交successor原字节保留；16项源定义仍15普通+1默认ignored，执行0。**本次五份本域文档更新使09的11路径元组成为历史时点**；四Rust语义审阅仍匹配，正式后继准备仍须捕获当前完整source identity，不借旧元组提交。未创建第三operation、query/Cargo/capture/freeze或新Jenkins身份，原#348仍保留。

首helper stdin关闭及路径分隔符guard失败保留，精确birth/exe受控退出后8个birth gone；第二个exit0/9 births gone。自动profile回收逻辑样本11,131,337B，物理未测/手删0B；首个失败profile156文件/11,152,398B加旧126B仍待原受管清理，未造新maintenance ID。Parent实际queue回执`01a16fb9-037e-7503-987b-7ce953508b0a`；公共owner首次因我填错完整聊天ID被拒绝，按统筹原ID核对后回执`01a16fb9-c430-7592-ba16-a4ad52e1cacf`。这两项均仅queued，不记已读；原拒绝保留。无Shared/公共源码/服务/桌面写入，所有产品门仍open。

## 既有后继operation：当前16项源码审阅合同09

已实读原Jenkins owner的[Parent followup04 disposition](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/plugin-manifest-validation-20261024/workspace-entry/actor-prefab-source-admission-reconciliation-20261024/parent-followup-04-disposition.json)及摘要，SHA分别为`5b4de87c…`/`507a2117…`。其20:35:57Z读取的08是13243B/SHA`8478f93d…`中间状态，guard/documentation尚pending；08于20:41:27Z最终为23941B/SHA`92d445ce931d3e85de65b10d7f7ac04bf623e6a16b6263630658a98fb662439b`，guard已0error/warning/suppression，格式通过，文档结构/318链接有效，完整checker仍因9项历史指纹变化exit1。两者四源完全相同，保留原owner时点，不倒填其已读最终报告。

本域现明确将**原未提交候选**`validate-prefab-source-adapter-02`的源码审阅关联到当前`prefab-source-tree-08`，仍在原session/request `actor-blueprint-prefab-authoring-adapter-20261024`内；候选JSON保持1801B/SHA`86b638a0f57e02615489ffee22c324ca5341571db32ecd94ccf97c422a6a317f`，没有新增第三operation ID、修改已受理payload或重开#348。当前四源tuple `e9d705782cf1022ef4649e59c34a606111fe06fe74c1190694063c68dc129a21`、16项精确测试定义、四文件语义审阅及不变候选字段见[当前后继审阅09](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-successor-review-09.json)。原review04/13项、source-adapter-02、原input/delivery/#348终态及08最终回执均保留原字节。新报告编号09是本域审阅证据，不是server operation或attempt。

最小验证意图保持独立`zircon_plugins/Cargo.toml`/`zircon_plugin_prefab_tools_editor`/defaults-off/features=[]/locked/lib/`prefab_`/profile test/jobs1/threads1，当前15默认可运行+1原ignored，执行0；不混入Parent独立4项Host或Delete设计。候选声明的11项ownedPaths含4Rust+7本域文档，本域四源tuple不是全部11项或完整九root源码身份。正常收件时仍须核对当前Records/marker与原请求、完整membership/所有writer、物理E reservation、pool/cache writer和native EOF；本轮只完成source-owner审阅，不submit/query/capture/freeze，不把20:37限定无记录或OS空闲当全历史结论/资源准入。后继实际server收件、生产caller、Rust及产品门均open。

## Prefab来源树08：域内只读实现与新源码修订

本轮继续既有四文件写权，在`authoring/snapshot.rs`实现`PrefabSourceAuthoringSnapshot::new(&PrefabAsset)`，沿原`authoring.rs`/`lib.rs`导出，回归仍在原`tests/authoring_snapshot.rs`。它借用真实`PrefabAsset.scene.entities`，完整保留原序、u64实体ID、中文/同名项、native字段及opaque组件记录/引用表，不克隆World、不解码未知provider、不创建store/producer或写命令。运行时已有`SceneEntityOverview`只做概览，此处借用完整记录供本域树/Details消费，没有复制该概览字段。

父级仅表示**来源场景实体层级**：`Root/Resolved/Missing/Ambiguous/Cyclic`。重复实体ID全部保留并定位，不能挑首项充当持久目标；无效父级不伪装为根。循环检测使用迭代路径状态，循环成员不发布可遍历父边，后代和原parent字段仍保留。`source_index`及诊断位置只在当前借用内有效；拥有Prefab UUID的document/resource key/version及每view完整重校验仍由真实调用者提供，不能从名称、组件type或行号推导永久itemKey，更不能把entity.parent当组件attachment。

新增3项测试定义覆盖resident实际载荷借用及u64::MAX/重复组件引用、重复ID/缺失与歧义父级、8192层链中的循环及self-parent/空来源。当前共16项定义，15默认可运行+1原ignored release性能门，Rust实际0；不扩大到Parent独立4项Host回归。具体四SHA、前后源修订、静态检查及原谱系见[source-tree-08](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-08.json)。这是新的本域源码修订，旧source-adapter-02/13项、source-revision-04和候选`plugin-source-adapter-02.request.json`均保留原字节，不作为本轮类型或执行证明。

验证意图仍在原active request `actor-blueprint-prefab-authoring-adapter-20261024`内，沿Parent已选的显式后继方向核定原候选`validate-prefab-source-adapter-02`具体payload和当前修订；不是重开`validate-prefab-authoring-01`/#348，也不新造server attempt。最小范围仍独立manifest `zircon_plugins/Cargo.toml`、package `zircon_plugin_prefab_tools_editor`、defaults off/features=[]/locked/lib/filter `prefab_`/profile test/jobs1/threads1，仅记录待准入范围，**本轮不submit、query、capture或freeze**。旧input/delivery/#348终态保留，未获所有writer、pool/cache writer及物理E reservation。

下一步真实接线仍需Parent提供同项目registry/resources和来源资源key、当前document/view/selection/generation，在原每view producer调用`prefab_source_snapshot`后借用该payload构造投影，延迟发布前重校验**原key**及原上下文并清除失效旧值。本轮没有修改5份Parent/ZUI保护源、Shared/catalog/tokens/主题、Penpot或服务/桌面；没有创建假生产资产。写事务、undo、组件attachment/类图后端、正式导出、正常EXE/150%DPI/双端≤1DIP仍open，passed/accepted=false。

## 样式中断接续：当前两目标与历史保存边界

已实读[Parent r04](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/actor-shared-style-interruption-r04-reconciliation.json)及其中20:13计数补正；不恢复服务、不延长deadline、不找回已于18:17关闭的context。本域新隔离context只选中并查看原body与另一Button目标，API前后Actor仍rev48/322对象/components全同，Shared仍rev2且对象/components全同，无Penpot authored写入。

原body field0 `9fffda54-1c42-5c4f-8e69-bb03997f533d` 没有typographyRefId/refFile，仍为SourceReferenceData main的原文字与custom-495ff285…/Roboto/normal-400/13.333333；Shared body则为gfont-roboto/regular。另一Button leaf `89eb3427-27ba-801b-8008-d6d09c3492ba` 只有primary颜色的2处authoring引用+1处positionData缓存，未套Typography，不是原body目标或main默认传播。当前原body无ref不能补签18:11 saved，也不能证明从未暂存后恢复；此前“未保存”表述只限于后继读回没看到引用，本轮明确收窄，不改历史receipt或失败。

全240个linked记录是实际对象UUID：120引用本域业务main、120引用Shared；13是Shared roots。按祖先componentFile与对应main子树逐一核验，240全部解析，旧baseline无缺失。Parent最初ID/file字段口径及本域较早范围不清的表述保留为历史，本次不把120子集冒充全文件覆盖。

两张当前原图均已打开复核，累计61张。body视图Roboto400 loaded/700 unloaded，随后Button视图700 loaded；不把unloaded当error，也不把同family/loaded当provider字节、variant/CJK或Engine ready。具体来源、字段、计数、两图和补正见[当前源对照07](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-compare-07/result.json)。原18:11 error、18:12拒绝连接及18:17关闭证据保留；Shared真实main发布、字体provider由原public owner接续。

自有新浏览器exit0，9个进程出生身份均退出，临时文件0；浏览器自动回收6,932,938B逻辑样本、物理未测，手删0B，旧126B待受管清理。无新Jenkins/query/Cargo、源码/ZUI或Shared写入、服务/桌面/DPI操作；所有产品验收仍open。本轮常规源/文档/进程信息写同一manifest供Parent读取，不重复投递旧摘要。

## 独立Host准备合同收件：未建身份，未开始验证

已实读原Jenkins owner的[准备合同](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/host-targeted-template-pane-20261024/contract.prepared.json)，12139B/SHA `3c3da678f66524863fbf49e5b75737b29e91975d1c63f48d037629883bc3e9a5`，及[摘要](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/host-targeted-template-pane-20261024/summary.md)，2324B/SHA `e6047bfcf48700be4f7aafdd07426203db160671af5817e0dc4270e4b7c0e8da`。三份Parent Host源当前仍逐字节匹配91257173/7d4cb542/b68235c3；4项`targeted_template_pane_`测试模块可达证据复用，执行0。本域不修改共享Host源，也不重放原整体patch。

本次独立读取的installed与capsule manifest均为`0b59401556af970b68922f3f33015c573cf6079723068a71760494a4a3795ac7`，实际5份capsule源码+6份pinned clients共11项全部匹配；entry `933fbeea…`、runner `8caf1cc5…`，driver/runtime仍`fc553228…` / `start-1792836412672596900`。197份driver及认证HTTP binding复用原owner合同，本域没有重做197项或调用API/parser。纯parse/recipe接受只说明入口能表达该范围，不是source-ready、native或Rust通过。

| 验证范围 | 实际身份及状态 |
| --- | --- |
| Parent独立Host | 根Cargo.toml、zircon_editor、defaults off/features=[]、locked/lib、test profile、targeted_template_pane_、jobs1/threads1；request/operation/attempt均null，未加入Prefab payload。完整跨依赖Cargo feature图未执行解析，Runtime target-editor-host/test-support声明保留。 |
| 本域当前Prefab候选 | 原active request内提案validate-prefab-source-adapter-02，candidate SHA仍86b638a0…；四源仍a03354dc/1c21c29a/f1295a3a/898ff6af，全部匹配source-adapter-02；12项默认可运行+1原ignored，执行0。尚无具体operation/payload的server确认，不新增或重投身份。 |
| 原Prefab #348 / queue14345 | 原input、delivery和terminal SHA均独立匹配，终态仍FAILURE/preparation-conflict、prepare/compile=false。合同的三份差异基于旧#348候选；不是当前source-adapter-02的新漂移，也不归责旧失败。单validate不符合resume_delivery的batch或特定未启动lock-prepare例外，未重放。 |

Host的显式`--profile test`模板须经现行local-cargo selector交Jenkins；dev-module-build无Profile参数且当前recipe元数据dev，不能冒称同一显式test模板已执行。未取得完整membership/所有writer、source-prepare/native EOF、物理E reservation及pool/cache-lock回执；OS空闲12596797440B、v11 reserve37580963840B及静态单jobs额外4GiB仅是原观察，不当server拒绝或准入。不安排capture/freeze/新停写窗口，不为造prepare回执调用normal validate，无需用户手工grant。

本域[host-preparation-reconcile-06](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/host-preparation-reconcile-06.json)记录实际11pins、三Host/四Prefab/五保护源、原失败谱系和缺门。原169份报告字节保留；旧含Parent文档的3777 guard不当当前文档通过，仅对本轮两份本域文档另做静态检查。无Rust/Penpot/ZUI写入、Cargo、服务或桌面操作，释放0B；旧126B仍待原受管清理，产品passed/accepted=false。

## Parent第六次接续：单leaf真实颜色引用已保存重开

已读[Parent第六次诊断](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/actor-blueprint-followup-06.json)。本轮先独立确认rev45/322对象与shared-style-04基线全部相同，此次读回没有旧body引用；18:11保存仍未确认，不能证明从未暂存。历史服务/字体失败保留为当时观察。只在原受控Button文字leaf `89eb3427-27ba-801b-8008-d6d09c3492ba`原生选用Shared Assets `editor.text.primary` / `e022fcac-39df-51eb-8515-4723f9932060`，未套Typography、未点库更新、未改Shared或syncedAt。

保存rev46仅该leaf变化；原生Undo rev47恢复全部322基线对象，Redo rev48与保存后322对象完全一致，关闭/全新context重开保留rev48及真实fillColorRefFile/Id。原13copy根、240linked身份、其余321对象、2业务components、中文、custom字体、几何/source/slot/event均保持。该leaf只有三处同一颜色引用和新增text-content-attribute；不称三个控件，也不称main继承。原字面量和样式同为#c0c0c0，未产生可计全局A/B的视觉变化。

Shared仍rev2/41对象/4mains/34+3，所有对象/components/styles/library metadata与本轮前基线相同，syncedAt仍13:50:51.913694Z。**新重开图自然出现共享库更新提示**，新使用color modifiedAt17:37:45.684Z与used-style条件相符；没有点击更新或篡改日期，不能据横幅推断main颜色发布或13copy传播已完成。请public owner沿既有真实main变更交付affected component/revision/source，再由本域逐copy核对消费，不靠Update按钮出现结项。

当前Roboto400/700字体面loaded，保留原custom字体声明；未验证CJK fallback/Engine字体，不盲套Shared Typography。4张新原图均已逐图打开，累计59张，完整[leaf-style-05回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-leaf-style-05/result.json)和[最终重开图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-leaf-style-05/fresh-reopened-leaf-color.png)可复核。149份历史报告及4份Prefab+5份Parent/ZUI源字节均保持。Rust/插件/生产producer/写事务、正常EXE/DPI/双端差异图和≤1DIP均未通过，passed/accepted=false。

新增公共导出合同提案已交[共享合同](actor-blueprint-shared-contracts.md)：原plugin的`singleSolidFill`源码拒绝fillColorRefFile/Id，文本capture经过它；本轮只核查源码，没有跑SDK capture/export，不称实际导出失败。需原pipeline owner确认引用投影和color UUID→token-source→canonical字段的合法回写，保留source/guard/override，不flatten/detach或绕过。此项独立于main真实发布、原PNG exporter和normal EXE gate。

自有浏览器exit0，9个有出生身份的进程全部退出，新临时目录0文件；自动释放11,131,825B逻辑样本，物理释放未测，手工删除0B。旧126B仍待原受管清理。本轮不追加Jenkins提交/查询、不改后继候选或旧#348；只更新本域4份文档和本域报告，无公共源码、服务或桌面操作。

## Parent第五次确认：沿既有后继候选一次核定

已实读 [Parent第五次回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/actor-blueprint-followup-05.json)，43329B/SHA `f9e8a086f168b801b6a3554d196c48097d09ba49bfbf947f52ab19db5698b3d5`。Parent选择显式后继，与本域先前审阅一致；沿用现有候选 `validate-prefab-source-adapter-02` 及1801B/SHA `86b638a0f57e02615489ffee22c324ca5341571db32ecd94ccf97c422a6a317f`，不另建operation ID或覆盖原input。该ID目前是本域提案，尚无原Jenkins owner对具体ID/payload的确认或server attempt，前次队列 `01a16f29-8909-7a10-bf8e-d45c040a2acf` 只证明已入队。本次请原owner一次核定此既有候选，**不立即submit、capture或freeze**。

当前source revision继续为source-adapter-02；[source-revision-04](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-revision-04.json) SHA `e9210103e0e4c9f65244fb317e3666dea9550bd25339be00441765cce659af27` 保持原字节。此轮四源及5份受保护Parent/ZUI源独立hash均匹配；四源全SHA如下，不把本域元组当complete engine source identity。

| 原路径 | 当前SHA-256 |
| --- | --- |
| `zircon_plugins/prefab_tools/editor/src/authoring.rs` | `a03354dc3140d50f5e876a86ffb67f79343d0376686609d5bcd050a680e8d551` |
| `zircon_plugins/prefab_tools/editor/src/lib.rs` | `1c21c29acfdfafc0b5f95f855d7135230084c86456eb2dc37b9de640292d21a0` |
| `zircon_plugins/prefab_tools/editor/src/authoring/snapshot.rs` | `f1295a3a013195e3482d13d387a892664430fbe7c6875cab76b9974c9af9a187` |
| `zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs` | `898ff6af2dc28d572a025ead2544faa6800855026d7cc8b77009f2d485ed9cfb` |

旧request/operation/#348关联继续完整：原input SHA `321d21934164df00cf42b22108a69caeab25b80149fd6962de22751071e4b29b`、delivery marker SHA `9bd998b05bd41313dbc88366db129e0ba40b4665cc0b988c14a316fb6ef2c5f1`、terminal SHA `617446f6156211e9d091589df1a224e927c04ea9630677876ba860a9d9998054` 本轮均匹配，完整路径与#348 queue14345的FAILURE/preparation-conflict见原source-revision-04谱系。后继是同active request内独立验证，不称旧operation恢复或同attempt转换；原report引用旧JenkinsIdentityFile的命令不得执行新源码。

当前installed为 `e5faa0b88d41fe103390edeb18c1134edd0c71d5ddc60a932feed9061bdf45bc`；capsule.json自身SHA与11/11 pins独立核对，entry仍SHA `520681eccf26ca70a230388d3b28bfd6d0a5659333f6c7fcb222ba605fba2cf7`，complete先返回的边界未变。completeSourceDigestEnforcement、checkoutInputIdentityEnforcement、declaredInputHashEnforcement当前false仅记录，不当源码准入、运行host生命周期或Rust证明；按原owner核定的实际当前合同接续，不据旧胶囊报告直接启动。

范围仍12项默认可运行+1原ignored性能门、Rust0；不包含Parent独立4项Editor host。19:27 OS E盘free13086208000B是Parent观察，非ResourceManager admission；writer/window/pool/cache/reservation未准入，尚无新capture或freeze。原126B旧cleanup继续等待原维护收件，不新建维护身份或删除；本轮无服务/桌面/Cargo/源码变更，释放0B。实际核定通知与结果追加原 [artifacts.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/artifacts.json) 的`prefab_successor_source_revision_phase.parentFifthReview`，旧149份报告及候选保持原字节。

## Jenkins源码修订接续：显式后继operation候选

已核对原Jenkins owner的 [三方对账](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/plugin-manifest-validation-20261024/workspace-entry/actor-prefab-source-admission-reconciliation-20261024/result.json) SHA `1a21a6549630d126035600eed640b0ce63c6e6b3f48f5c69cb897f3ffdc7ba2b` 和 [准入需求](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/plugin-manifest-validation-20261024/workspace-entry/actor-prefab-source-admission-reconciliation-20261024/source-admission-requirements.json) SHA `f50c25363719bc180c9593530db8db144d8e5d5f3bf8d5a4cedccb087558a615`。原request `actor-blueprint-prefab-authoring-adapter-20261024` / operation `validate-prefab-authoring-01` / queue14345/#348的FAILURE/preparation-conflict保持，prepare/compile=false；原input、delivery marker、terminal receipt均不改。当前authorization active不重开旧终态，ResumeOriginal也不会转换本案。

本域source owner明确选择同request下的**独立后继operation** `validate-prefab-source-adapter-02`，不是同operation恢复或attempt转换。实际候选 [plugin-source-adapter-02.request.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/plugin-source-adapter-02.request.json) 只将原input的operationId替换为上述后继名称；repository/session/request、ownedPaths、driver/runtime、外部根、原full-file-index-v2和最小配方均保留。谱系与当前源码审阅见 [source-revision-04](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-revision-04.json)，该文件是本域审阅记录，候选尚未server intake，不是正式attempt、capture或执行回执。

四Rust当前字节仍匹配source-adapter-02：authoring `a03354dc…`、lib `1c21c29a…`、snapshot `f1295a3a…`、tests `898ff6af…`。前后候选中只有authoring/lib/tests三份后来变化，snapshot不变；这些变化不能归责#348缺逐文件表的旧engine filesDigest漂移。已审阅真实UUID驻留查询、原native key失效、只读借用与五项新增回归源码；当前13项源码=12普通可运行+1原ignored性能门，实际Rust0。原四源guard/format仅按相同四hash复用，不是类型检查或新依赖执行证明。

后继最小配方仍为`zircon_plugins/Cargo.toml` / `zircon_plugin_prefab_tools_editor`、defaults off/features=[]、Windows host lib、`prefab_`、test profile、locked、jobs1/test-threads1、物理E:/cargo-targets。**旧report里指向plugin-authoring-01.request.json的命令仅是历史配方描述，不能验证新源码**；新候选的条件命令在source-revision-04中，只有经正常源码准入后才能使用。

本域只读核实selected `e0635d5d…` 的11pins与原driver/start一致；不据此领取host生命周期通过。当前complete membership/所有writer、稳定窗口、pool/cache writer及ResourceManager reservation均未获，不启动full capture或Cargo；E盘空闲/政策reserve只是原owner观察，不冒充资源准入。请原Jenkins owner沿同request正常收件该明确后继，先复核当前源码与真实准入，再执行；不换request、借repair身份、改grant或要求用户手工授权。原126B仍无已提供的server维护身份/receipt，本轮删除/释放0B，清理继续open。

## Shared rev2接续：发布已核实，原生同步与字体仍未通过

本域18:05Z独立认证读取Actor rev45/322对象/13外链native copies，以及Shared rev2/isShared=true/41对象/原4main/34 colors/3 typographies；真实body/caption/strong UUID与Parent交付一致。不能再把旧rev1/style0或空库当当前阻塞。新catalog SHA `4c2f849c533f1f227903018eca860faadd7fa3138fdc265300913e3833ad36e5` 的10个NumberField props、Property四槽及原required value已独立结构核对，未运行新plugin/Rust验证，未发布或推导NumberField master。

实际原生管理库→更新显示“没有需要更新的共享库”。四component modifiedAt仍13:21:31.125Z，Actor library syncedAt13:50:51.913694Z，Shared文件modifiedAt17:37:45.683436Z。已核对Penpot原生源码按已使用component modifiedAt过滤的链；旧copy尚无新style ref，空typographyRef字段不算绑定。请原public owner接续合法component发布，再由本域原生同步并保持覆盖；本域未改Shared、detach、unlink/relink、清覆盖或重签export guard。

仅在本域SourceReferenceData main field0 `9fffda54-1c42-5c4f-8e69-bb03997f533d` 原生选择Shared body。Inspector已显示body；之后18:11～18:12Z独立get-file及一次首页readiness均WinError10061拒绝连接，故新保存/revision/关闭重开不作已完成。Roboto400/700字体面error，正文和中文font check=false；截图可见serif fallback，不计CJK或Engine字体ready。两项新事实分别入Parent原队列：`01a16f19-6fa9-7341-bd4c-b6bcf0f9fb21`、`01a16f1d-7099-74b3-9931-27553a34866d`，仅ACK入队，不当已读或修复。

原始API基线、13copy及shapeRef/覆盖、catalog源、三张逐图复核、原生selector失败及服务错误见 [shared-style-04](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-style-04/result.json)。恢复后先读取实际field style ref，确认是否保存再决定是否重做；不假定新revision或清覆盖。Prefab override为serde_json::Value且无min/max/step/precision schema，模板42/0..100不绑定业务，i64/u64不经f32往返；description/source/reset只是槽合同，真实数据/事务缺口见 [共享合同](actor-blueprint-shared-contracts.md)。

本轮不改Rust、原31节点ZUI、公共catalog/tokens/Starship、服务或桌面。原13项Prefab/4项host测试、per-view producer、写事务/undo、官方导出、normal EXE/150%DPI与双端≤1DIP门保持open；三张新图只作Penpot诊断，passed/accepted=false。

隔离helper已exit0，9个PID/birth均退出，新temp根0文件；关闭前158文件/11,131,337逻辑B样本由浏览器自动回收，物理释放未测、手工删除0B，旧126B仍待原受管清理。见 [本轮closeout](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-style-04/process-closeout.json)。七文档结构、25+45条目和原31节点资产检查仍有效；历史75指纹当前8份已变化，包含本次Parent catalog及材质canvas，不覆盖历史表或伪称完整checker通过。

## 第四次接续：最终回执已在原路径，Add undo与通用Delete边界已补

已读取Parent followup-04与统筹“Actor 第四次接续”。Parent在16:44～16:48记录的`prefab-source-adapter-02.json`尚不存在是该时点事实；本轮不改Parent回执或倒填已读。原路径现在实际存在15277B，SHA `657586343d35a6fbc34302eb11e5bbc3cbcf25172e3c6cb17fe4ac3e1fbba64c`，完整四源guard `aba28200…`/0error-warning-suppression、格式结果、13项源码（12普通可运行+1原ignored性能门）/0执行、UUID与native active key失效边界均已落盘。该最终文件保持不变；原#348/旧8项/compileStarted=false不能代验新13项。

真实进程closeout在原 `penpot-source-state-03/process-closeout.json`，3925B、SHA `533842ac8ac6bde2fb7e6224d435c7a431bd93877e8558d7c5acddeb6f764715`：helper exit0、9个PID/birth均退出、新temp根0文件，自动回收样本11,131,831逻辑B、物理未知，旧126B待受管清理。原两份通知ACK均在closeout之后。本轮完整引用、当前四源/Parent三源/ZUI字节、格式复核及消费边界统一见 [target-contract-03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-target-contract-03.json)，不另建任务、request或operation。

AB-D03、AB-003与AB-PF-16已明确已有Add undo：全批token/value预检，仅撤销该命令创建且未变的组件。任意既存组件的通用Editor Delete/journal/inverse/retention/多目标原子性仍归Parent核心层；本域不复制事务、广开pub或序列化Arc。另按真实 `AppliedThenGatewayFailed` 分支修正笼统“失败零变更”：预检拒绝零变更，执行后错误消费核心实际结果，不自行补偿/重投。

已交真实Inspector映射：component.id是完整type、property.id是field_id，显示名非目标且该投影自身不含document/entity；`entity://selected`/`node://`也不足以保存跨文档目标。持久行按原document authority/entity/type及模型组件身份恢复，creation/revision独立核验。延迟来源结果必须验证原native key，不能取最新key给旧payload盖章。Make Root/Reparent缺attachment模型时不可用；类/Prefab写事务与scene Add保持各自文档authority。

本轮未改Rust、原ZUI、共享库/主题、服务或桌面；原Penpot file/page及rev45设计回执有效，无新画面验收。当前13项Prefab与独立4项host测试、真实per-view producer、写命令与normal EXE/150%DPI/双端≤1DIP仍pending，passed/accepted=false。本次通知的实际queue结果追加原manifest，不将入队当已读或执行。

## 最新域内实现：真实驻留来源查询，生产调用者仍缺

本轮只改原四文件范围中的 authoring.rs、lib.rs 和 tests/authoring_snapshot.rs，authoring/snapshot.rs 未变。新增公开 `prefab_source_snapshot(&AssetReference, &AssetRegistryIndex, &ResourceManager)`，返回实际 `ResourceSnapshot<PrefabAsset>` 与既有 `ResourceActiveVersionKey`；通过 UUID 目录解析和 `snapshot_expected` 核对真实驻留版本/载荷类型。未注册 producer、增加 manifest 依赖、写共享 host 或重放旧三文件 patch。Reference、WrongAssetKind、NotResident、SnapshotRejected 分别保留；NotResident 不能写成来源缺失，key 不能作持久实体或 override target。

当前四 SHA 依次为 authoring.rs `a03354dc3140d50f5e876a86ffb67f79343d0376686609d5bcd050a680e8d551`、lib.rs `1c21c29acfdfafc0b5f95f855d7135230084c86456eb2dc37b9de640292d21a0`、authoring/snapshot.rs `f1295a3a013195e3482d13d387a892664430fbe7c6875cab76b9974c9af9a187`、tests/authoring_snapshot.rs `898ff6af2dc28d572a025ead2544faa6800855026d7cc8b77009f2d485ed9cfb`。新增五项真实内存 registry/resource 回归源码，合计13项源码，其中原有1项 release performance gate 默认 ignored；普通配方可运行12项，实际运行0项。四文件 change guard `aba28200fa8ed5763e1e6255e3b1b11a41594b10343090f0e4abff785c993533` 为0 error/warning/suppression，rustfmt通过，均非类型检查或Rust执行。详见 [source-adapter-02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-adapter-02.json)。下文“四源未变/8项源码”是各历史阶段事实，不覆盖本轮三文件增量。

Parent 原三份 guard/测试、未应用的 scene reader、原 ZUI SHA 均再次匹配。下一处真实接线仍是 Parent 的逐 view/document producer：注入同项目 registry/resources 和 typed instance，将结果连同完整文档/选择/World/schema/request authority 发布，失效时退休旧结果。此函数不替代原 loader、per-view batch、view incarnation/retention，也不解决源实体→实例实体、组件 attachment 或持久 override 写 target。写命令/factory/history/undo/save、完整 Blueprint/Widget Definition 后端继续 open。

Actor 设计当前独立读回 rev45/322对象，新增三张 AB-SOURCE-03 查询状态设计板：640 `1514bd85-ad89-4263-b755-db1e85ee5c54`、900 `0589f584-db29-8000-8008-d6f89659e87f`、1280 `0589f584-db29-8000-8008-d6f8c8b3be19`。18个原生本域业务 copies/54个子 shapeRef、文字覆盖和 column flow 保存/关闭/fresh-context重开一致；原240个非根对象、2业务mains、13个外链copies未变。六种状态取自本轮实际 API 合同，板上明示“未接生产者，未验证EXE”，不是当前项目查询结果或完整窗口。Shared 仍 rev1/4mains，宽度继承缺陷及样式/导出由原 owner 接续。三张最终图已逐图查看，见 [AB-D09](actor-blueprint-design.md) 与 [source-state-03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-state-03/result.json)。

原 #348 preparation-conflict/compileStarted=false、原 request/operation 和旧126B待受管清理保留。本轮不重投 Cargo、换身份、改 grants、启服务或占桌面；实际EXE/150%DPI/双端≤1DIP仍无证据，passed/accepted=false。自有headless helper exit0，9个进程出生身份均不再存活，新临时根0文件；关闭前样本11,131,831逻辑字节由浏览器自动回收，物理释放未测、手工删除0B。

本轮七文档/271本地链接/66个UE路径、25+45条目、18场景/4环境/14开放差异及9设计段结构均有效；完整checker exit1的原因是75份历史指纹中6份已有合法变化：本轮3份域内Rust、材质2份Graph源及Parent的extension_access.rs。历史表不覆盖成当前SHA，当前四源另存source-adapter-02/manifest；不将该exit1改写为全检查通过。原source/测试/截图失败和未获ACK记录保持，新的实际投递结果继续写本域manifest。

本轮实际投递：原Jenkins owner queue exit0 / ACK `01a16edc-c40a-71f3-9389-e7c99d83fafe`，Parent queue exit0 / ACK `01a16edd-0f02-75a2-a3e8-121be61c5bd2`。两者仅确认入队，尚未确认已读、准入或编译接续；完整正文/SHA/原输出在原artifacts.json的prefab_source_adapter_phase.coordination_outbox。历史满队列拒绝与所有旧ACK保留，不重复重投或据此改passed/accepted。

## 最新宿主接续：定向 guard 已见源码，稳定身份合同已补

已读取 Parent“Actor 第三次接续”及 followup-03，当前源码确认两处定向入口改动和四项 controller 源测试；本域未写上述共享路径。实际 SHA：extension_access.rs `91257173706dba627ed7f396cc7ce9b9d305cd15e5f7e0a13b36415066ae1373`、pane_payloads.rs `b68235c36c225ec906b4f9e24fa8f384e146028ca7e67913f4a9cacb9cca3582`、template_pane_data.rs `7d4cb54241be0ef418138738e80db7a8ba9a9934394de799215fe300bd07807f`。snapshot.rs 仍原 `2ab680838fc85d63db96bede7e5b00ca1b2524f76da923041972575032571505`，场景 reader/stamp 未应用。旧三文件 patch 只保留审阅历史，不整体重放。Parent 的静态检查不是本族 Rust/产品通过，四项 host 测试不计入原 Prefab 请求。

已补 [AB-PF-07～14](actor-blueprint-shared-contracts.md)：区分当前 scene/document/World、来源 Prefab UUID/载荷版本、源实体与实例实体、组件完整类型/provider/schema、override 借用行的身份。实际可用来源链为 registry `resolve_asset_id_for_reference`（UUID authority，locator 仅提示）→ 原加载任务 → ResourceManager `projection_snapshot().active().key(id)` / `snapshot_expected::<PrefabMarker, PrefabAsset>`；无需新 registry、trait 或版本时钟。ResourceActiveVersionKey 只属进程内，不能序列化为持久 ID；snapshot 不主动加载/提供 residency lease，Parent 仍需原 producer/生命周期/retention 接线。

两个具体业务缺口：没有已确认的源 Prefab entity → 场景 instance entity 映射或组件 attachment/SCS 身份；现有 `ComponentPropertyPath::parse` 对含点组件类型有明确历史 TODO，后续须从已核验的完整 component_id 与字段段调用结构化 new，不能猜 override 字符串。原 adapter 仍只借用 typed instance、保留序列化顺序/冲突位置；source_index、标题和相同 u64 都不能跨输入/文档当 target。精确源码、失效/错误语义和后续消费次序见 [readonly-identity-02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-readonly-identity-02.json)。

四份域内 Rust、原 ZUI、原 patch 和 #348 preparation-conflict/compileStarted=false 回执均未改；没有提交新编译、运行插件/EXE、占桌面或恢复服务。真实逐 view 查询/绑定/事务/undo、完整 Blueprint backend、原正常 Actor case 及双端验收仍 pending，passed/accepted=false。原 ZUI 的七个导入位于 `imports.widgets`，`imports.styles` 为空；imports 两个分组不是两个导入。以下旧章节保留各自阶段事实，“三文件未应用”应以上述最新选择性实施为准。

本轮文档检查覆盖七份记录、257个本地链接、25+45盘点和原ZUI结构，无结构/链接错误；完整 checker 仍 exit1，因为75份历史指纹中的 graph/node_widget.rs、graph/routing.rs（材质 owner）及本次 extension_access.rs（Parent）已变化。历史证据不覆盖成新 hash，也不回滚 owner 改动。新增24项 source/身份静态核查为24/24；均不是插件/Rust/EXE执行。

## 最新设计接续：真实Shared消费者的中文/三尺寸约束问题

Actor原file/page现rev30/241对象；原150个非根对象、业务mains/12copies、旧四件消费者及原顶层相对顺序不变。新增AB-CONSUMER-02三板：640 `d9109291-fc9f-8088-8008-d6e13ae96660`、900 `d9109291-fc9f-8088-8008-d6e1ba22bcaf`、1280 `d9109291-fc9f-8088-8008-d6e1e02f8f15`。原生复制形成9个linked roots/87个有效shapeRef。中文“添加组件”“搜索组件”“PrefabInstance（场景实例）”是设计内容检查，不算实际实例查询。两轮保存/关闭/fresh-context重开，对象哈希变化0；Shared仍rev1/41对象/4mains、colors/typographies0，未写公共库或detach。

根fill后实际宽592/852/1232；Button text仍220宽，离根中心分别偏左176/306/496；Search内部背景仍240、text204；Tree caption仍128×16，观察为2行且动作留在源宽位置。这是Penpot对象几何与截图发现，**不等于UE/EXE双端≤1DIP测量**。12张新增原图均已打开逐图复核，总46张；均为DPR1的browser图。完整身份、数据和图片SHA见[component-content-02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/result.json)，逐图表见[设计AB-D08](actor-blueprint-design.md)。

给公共owner的最小提案：Button文字宽/居中、Search surface/text宽、Tree单行ellipsis/末尾动作分配应随consumer根宽变化；不由本域改内部坐标补偿。9个copy的native-layout-allocation.parent均指向canonical父节点。只读定位`tools/penpot/apps/zircon-zui-plugin/src/penpot-control-content.ts:288,554`与`penpot-layout-allocation.ts:37`；原TreeRow foreign-parent guard保持。本域未调用refresh/export，仅记录潜在映射约束，**不声称插件已抛运行错误**。需原pipeline owner接native copy/source/shapeRef/generated-state/allocation合同；原合法Property value件、原资产投影/slot/override/style导出请求保持，probe不得覆盖原workspace ZUI。具体见[共享合同](actor-blueprint-shared-contracts.md)。

业务边界再核对：`zircon_editor/src/core/editing/command.rs`的EditorCommand及rename/set_parent/set_reflected_scene_field构造仍为pub(crate)，插件不能直接调用；公共OperationCommand/OperationCommandFactory存在，但Prefab未注册写factory/operation。原per-view host三文件diff未应用，四份Rust和原ZUI SHA不变；没有扩公共trait/history/schema、fixture producer或把source_index当持久target。原#348 preparation-conflict、8项目标测试/0执行保持，未重查或重投未变化结果。真实query/绑定/事务/undo、Widget Definition/class/graph backend、正常EXE case/host、桌面/DPI与双端验收继续pending，passed/accepted=false。

自有浏览器均退出，新临时根0文件；两个close前样本共22,262,675B逻辑文件由浏览器自动释放，物理量未测，手工删除0B。stdin关闭、自有终止helper退出-1及复制输入未就绪失败保留，旧126B仍待原Jenkins维护身份。本轮没有服务/公共Rust/主题/目录写入或桌面操作；既有手工窗口已过期，不承诺新窗口。

## 当前接续：恢复 r2、现场 selector 与原 Prefab 终态分别核对

2026-10-24T14:52:18Z 本域实读 installed selector 已为 `2790fb5199bd06ca7291ac197f03545eaeae05c56b125117fad268ce19d4b0c6`，entry SHA `c0ec4b748c21c0ef537dcd8d27e3c3e442474ec033840c0958edf8323359f24a`；manifest自哈希及5实现/6客户端均匹配，仍绑定原 `fc553228…` / `start-1792836412672596900`。Parent 20b3、后续9c684的观察各自保留历史，不当作当前selector。实际绝对插件manifest参数的纯解析与原identity配方一致：`zircon_plugin_prefab_tools_editor`、defaults off/features=[]、locked、lib、`prefab_`、test profile、jobs1/test-threads1、物理E根；没有调用Cargo解析或执行。

正式wrapper原身份QueryOnly本轮exit0，返回 `phase=complete/status=observed/resubmitted=false`，result与#348归档逐字段一致，仍为 `preparation-conflict/preparationStarted=false/compileStarted=false`，目标8项测试运行0项。Parent其它session的历史authorization pending不能套成本域当前结果；本域也未改grant/写数据库。当前入口 `resume_delivery` 仍只接受durably checkpointed batch，原单配方不满足；需原Jenkins owner沿原request/operation给支持的source-admission接续。前次具体消息ACK `01a16e63-a17a-7690-ba44-52c5017ba713` 仅代表给原owner入队，不证明已读；未重复发同一请求、换ID、改batch/source guard或重投Cargo。

恢复r2的原始native-terminal实际SHA `dcbcd30d3c62e00b72bca867921b8669948312116ae8c389089493faa9534675`、root0/active0/childrenGone/双EOF/readersJoined/jobClosedAfterZero本域已直接核对。Parent的197文件/固定工具/11-job历史与后续Home变化保留为Parent证据，本轮未重新做全量197或11-job API/Home验收。上述仅证明原Jenkins恢复终态与本次入口/查询，不关闭P21/Runtime11、产品Rust或正常EXE画面。具体命令、实际输入指纹、恢复边界及只读业务复核见 [本次对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/plugin-authoring-05-recovery-reconcile.json)。业务源码仍为Prefab只读注册、命名Apply/Validate控件与多实例view；原per-view host diff未应用，写命令/undo、真实producer与Widget class/graph Definition边界继续open。本次无编译提交、服务/桌面操作或新稳定窗口，passed/accepted均false。126B未删且没有可提供的既有server maintenance身份，原queue交接和清理未完成状态不变。

## 最新验证接续：原 Prefab 请求已受理，准备冲突前未运行 Cargo

2026-10-24 本轮只读观察的安装从 `28698b9e8dcd4972e372ccc17969d3b3f28c2389327037636f805c5b58efbe4f` 进到 `20b3bf855a26a9516845cf7611b216a2808aef944022cd99554525a71a319c75`；后者 pinned client 明确接受 `zircon_plugins/Cargo.toml` / `zircon_plugin_prefab_tools_editor` / `prefab_` 单包 lib 配方。原 request `actor-blueprint-prefab-authoring-adapter-20261024` / operation `validate-prefab-authoring-01` 已通过正式 `local-cargo.ps1` 受管入口实际提交，沿原 identity，未替换请求、包、过滤器、runtime 或 source policy。范围 defaults off/features=[]、locked、test profile、jobs1/test-threads1、物理 `E:/cargo-targets`；module 入口不能表达独立插件 manifest。客户端两轮完整输入读取均完成，本地客户端退出0，收到 queue14345；这不作为测试通过。

受管认证实际读回 [zircon-management #348](http://127.0.0.1:18080/job/zircon-management/348/)：queueId14345、building=false、FAILURE、原身份参数相符。唯一归档 [原始回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/plugin-authoring-03-jenkins-receipt.json) 为6337B/SHA `617446f6156211e9d091589df1a224e927c04ea9630677876ba860a9d9998054`，`status=conflict/reasonCode=preparation-conflict/preparationStarted=false/compileStarted=false`。随后原 identity QueryOnly 返回 `phase=complete/status=observed`，result 与归档逐字段一致；早先 `management_git_busy` 是中间态，未当作实际仍有进程等待的证明。源码中8项目标测试实际运行0项，passed/accepted均false。

提交 marker 与归档 `sourceComponents` 的差异只有 `source.filesDigest` 及其两个派生摘要；HEAD、index digest/metadata、fileCount、zr_vm及7个Gitlink、dependency-cache配置均相同。预期完整摘要 `84604294796c185f783a7cb41ffba5d7bb689b41603ddd1093bc859d9388cd62`，实际 `dac3115b7fc8bd82a5d637b3c47511927e8aa72a87b89bbec4c6ed8c20299fc4`。保留摘要没有逐文件映射，fileCount相同也不能证明未跟踪文件membership相同，因此没有推定具体变更路径或owner。失败后、更新本handoff前，原7文档+4Rust共11份canonical指纹全匹配；原ZUI仍为原SHA。没有据此删除、回滚或排除其它会话输入。

完整命令、5次只读查询、PowerShell未引用分隔符/历史查询buildRoot字符串比较两项调用边界、原queue/build/终态、source摘要比较见 [本次接续](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/plugin-authoring-03-continuation.json) 与 [查询记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/plugin-authoring-03-query.json)。缺会话登记的QueryOnly旧失败保留；正式自动收件后成功登记与受理，未手工注册旧协调器。当前pinned `resume_delivery` 源码只允许durably checkpointed batch，本原始单配方没有batch；本域未发resume/新请求、改原payload、跳过source guard或运行本地Cargo。Parent/Jenkins原owner需沿该原身份给出支持的source-admission接续；不能继续把本次当前障碍写成“仅材质配方”。

Parent单文件手工停写窗口已实际履行 `2026-10-24T14:15:01Z`～`14:19:01Z`，结束核对仍为 `af2d720c87fb43dd4d5f1bad9b2fd3a514162b3c47dd1a6b7efbd41d1bfbde51`。时间、owner、原SHA及到期观察保存在 [原manifest](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/artifacts.json) 的 `parent_capture_window`；queue通知因100条上限拒绝、未送达且未重投。窗口到期后才在此记录新验证结果，未续设窗口或认领全仓稳定；原窗口不是当前文件SHA的无限期承诺。除本handoff之外，本轮未写仓库文件、服务或桌面；Penpot仍rev21/Shared rev1，生产query/事务、合法value件及导出、正常EXE与原生逐图验收继续pending。新产物仅保留有界报告/失败回执；手工删除0B，旧126B仍待受管清理。

## 最新接续：Shared rev1四件的原生消费

Actor原file/page不变，最终**rev21/vern0、151对象**。新增受控board `89eb3427-27ba-801b-8008-d6cfcabb7a3c`，原生管理库正式连接Shared file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12`（仍rev1/isShared=true/4主件），Assets真实拖动生成Button/SearchInput/TreeRow/PropertyEditorRow四个native copies。完整component/main/copy/33shapeRef、覆盖值和API见[consumer-01](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01.json)，11张新图见[设计逐图表](actor-blueprint-design.md)。原五板、两个业务main/12copies保留，116个原非根内容对象、两个component记录及原顶层相对顺序逐项比较未变；Shared/公共源码只读。

原生中文字“添加组件”保存rev19，关闭/fresh context保留；原生“重置覆盖项”保存rev20，恢复Button且touched清空；重设中文保存rev21，第二次fresh context保留。四个Shared asset board解包后的penpot_original_source均与当前原ZUI字节一致；canonical-source/record在component.pluginData.devZirconZui，不在root.metadata。源SHA与metadata/版本边界详见[共享合同](actor-blueprint-shared-contracts.md)。这是消费者独立观察，不代原owner正式codec回执。

最小依赖：PropertyEditorRow必需value槽的一个合法发布Field/NumberField等主件及slot/override导出合同。当前4件都不在允许集合，copy value_slot保持空；没有塞入SearchInput或造UUID。colors/typographies仍0，公共两族style A/B、Menu/Dock、官方ZUI/PNG、原资产回写与EXE另待原owner。验证板仍是四控件消费切片，没有实际Add/Tree/Property命令，不作为完整窗口。

本轮11新图逐一打开，总34图；首轮工具/拖放未生效、locator及UTF-8读回错误保留。headless/DPR1，无桌面操作；driver exit0，9个自有进程出生身份均退出，新临时目录0文件。旧126B继续待Jenkins受管清理，手工删除0B；自动临时profile回收的物理释放未量测。原四份Prefab代码、未应用host diff、原验证request/失败/Runtime lineage和AB-DEF合同不变；本轮未运行插件/Jenkins/Cargo、服务或EXE，passed=0/accepted=0，全族未完成。

以本节与同一handoff供Parent读取，沿既有队列满失败记录不清队列、不重投，也不将文件更新记为聊天已读。下一项独立接续是消费真实发布的合法value件和slot合同，再接本族属性组合；生产query/逐view失效及事务仍按AB-PF候选交原host owner接续。下面rev0/rev4/rev12各段保留历史。

## 最新分配与消费者合同

已读取 [主统筹记录](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)，当前分配已落地：file `b4a1e3ce-9e6a-81fe-8008-d69d60f73bf1`、page `b4a1e3ce-9e6a-81fe-8008-d69d60f73bf2`；本域 2026-10-24T10:33:10Z 独立认证 get-file 返回 HTTP200/revn0/vern0/仅根对象，组件/颜色/typographies=0。公共文件同轮读回也为空且 `isShared=false`，不是已发布库。两个响应指纹与身份见 [共享消费者合同](actor-blueprint-shared-contracts.md)。不再把等待文件创建作为当前阻塞。

当前明确 owner：材质 `01a16d50-6c7f-7370-853d-4581184b66a1` 牵头图表面；动画 `01a16d51-741b-73b3-a61a-2189cd38f81e` 负责 timeline/curve/transport；Parent 负责跨域协议、公共树/属性/主题/Dock、插件与证据集成。实际需求已整理为 AB-SHARED-01…06，保留域业务 authority，不改共享文件。需交接实际公共 component/style IDs/version、library/link/publish 与原插件导入保存导出路径；桌面时段未授予。

本域计划为 `docs/plans/penpot-ue-window-suite-20261024/actor-blueprint*.md` 七份；产品目标与写入边界见下。已通过官方 CLI 把实际 file/page 读回、计划路径、公共消费者合同和下一项能力核查提交 Parent，回执 `01a16d7a-5871-7d52-b2c0-44f3fa6e5a9b`、exit=0；仅证明排队，不证明已读。Jenkins 新公开 ready 与 nativeLifecycleVerified=false 按 Parent 原记录区分，本域未执行验证或重启。

后续已在本域文件真实保存五个 body composition 草稿板，58 对象/41 flex frame、0 原生组件/实例；API 保存 revn=1，浏览器后续读回 revn=2。首板关闭/全新 context 重开及其余四板完整截图均已逐图查看，实际身份与具体裁切问题见 [具体设计](actor-blueprint-design.md) 的 AB-D08。原样保留 Node 模块混用、两次直接 CDP 启动失败；改用现成 Playwright 后成功，失败脚本的直接启动根因仍未确认，不归因 Penpot 服务。自有浏览器均已退出、未占桌面；两份失败 profile 小文件共 126 字节待 Parent/Jenkins 托管清理，实际释放 0 字节。

上述 revn=2 是第一阶段记录。当前 revn=4/vern=0，117 对象、55 flex frame，五板 UUID 不变；已新增两个本域业务数据主件、12 个原生 copy roots，详见设计 AB-D08。两个主件不是公共 Tree/Property/Button 或主题副本，尚无 Engine component ID、事件、查询或命令。内容取真实源码测试夹具而未执行查询，资源 readiness 未验证。公共实例、样式和正式导出仍未完成。

SourceReferenceData 的 component/main UUID 分别为 `8ff62384-7110-5d59-bd61-64732823f235` / `5b9092d3-1ecd-5d74-999f-69b8e082d26f`；OverrideRecordData 为 `8513f987-512c-5fc9-9dd1-059c6777a2a2` / `2f849db9-4bae-5293-985c-0f5df153c64b`。返回 modifiedAt 分别为 `2026-10-24T11:07:45.880Z` / `.881Z`，不虚构语义版本。三尺寸共 12 copies，含主件的 component-reference roots 才是 14。首轮截图的实例文字同步错误已修复，typed set-touched 的 HTTP200 与后续四 context 重开见 domain-data-03 / receipt-06；失败请求和错误图原样保留。640 URI/标题仍裁切，不作为正式窄窗通过。

上述实际进展、全部七份计划路径、产品目标/共享边界、六项公共合同、真实能力缺口、下一项独立工作与 126 字节临时项清理选择，已通过官方 `codex queue` 投递 Parent，回执 `01a16d8f-dfdf-71f0-8ab6-abce24d4bd10`、exit=0。只确认排队；未据此声称 Parent 已读、交付组件或接收清理。未请求或占用桌面时段。

新增真实组件身份、原图、实例覆盖修复、内建 Prefab 导入/加载与元数据往返能力补正已通过官方入口排队，回执 `01a16da0-5ec5-7011-a9c8-782514d5dc2c`、exit=0。也提示 Parent 原资产 `nodes[*].events` 合计 15 entries，根无 events 不应算全体为 0；原 hash 未变。已读取 Parent 的 45 项 [覆盖对账](parent-window-coverage.md) 和公共 pipeline 委派；仍未收到发布/导出/Actor normal recipe 的实际回执，没有申请桌面时段。

## 保留的既有交接与失败

本文件已保存；已核实本机受支持的 `codex queue --thread … --message …`，向指定 Parent 发送交接，返回 `Queued message 01a16d63-831d-7a93-845c-c96e36ffd0fd`、exit 0。该回执只证明排队成功，未证明 Parent 已读取。当前工具列表仍没有 Penpot 或桌面连接器。

执行会话 `01a16d53-8965-7472-931c-73cba748fe8e`，Parent `01a16d4e-0491-7353-84fa-038bd7706546`。用户先前回复“等待parent告知”；随后收到 Parent 的明确接续指令：服务、共享组件/主题、文件页面和桌面统一协调，当时尚无本域 Penpot file ID，现已分配，见本文件开头。本族遵循该分工继续源码/能力盘点及具体设计，不重复询问用户。已按要求排队汇报明确路径、计划、提案、缺口和下一项独立工作，回执 `01a16d6a-9ea9-7e61-844c-8c5753c0f1ae`，exit=0；未宣称这条消息已读。

明确本域目标为 `zircon_editor/assets/ui/editor/world/extension_prefab_editor_workspace.zui`，业务候选为 `zircon_plugins/prefab_tools/editor/src/**`。产品资产/业务源码均未编辑；当前写入为本族 `actor-blueprint*.md`、自有证据与已分配 Penpot 文件，未认领共享 core/UI 框架。

## 已完成的独立工作

- [工作记录](actor-blueprint.md)：核查 shared checkout、既有组件/样式、真实 Prefab/场景命令与环境依赖。
- [窗口盘点](actor-blueprint-window-inventory.md)：25 个本族条目，45 个世界创作/日志/分析等覆盖候选；Editor 入口扫描命中 60 个第一层目录、125 个文件。列出适用/待定/不适用、实际注册依据、当前能力与建议 owner，补齐 Level Blueprint 和创建类对话框。不是 70 个已实现窗口。首条排队消息中数量为补齐前的 23+45，以当前盘点为准。
- [差异及场景](actor-blueprint-differences.md)：14 个开放差异、四种必需环境、18 个业务/状态场景，沿用 LayoutReviewCase 与原生证据格式。
- [具体设计](actor-blueprint-design.md)：七种模式的区域/Dock、flow/scroll、类与实例事务、快捷键/焦点、typed graph/compile 接口、旧实例迁移及测量锚点；五个真实板、两个业务主件及设计原图在 AB-D08。公共引用和产品图仍缺。

本会话没有修改公共组件、catalog、tokens、Starship、正常捕获协议或其他窗口族源码。此前未分配、首次区域草稿的历史均保留；当前已有本域可编辑业务数据实例及 Penpot 图像复核，身份见顶部。正式导出、EXE、原生捕获、双端逐图和 passed/accepted 继续缺失。

## Parent 需要交接的具体资源

1. file/page 已在本次交接解决，实际身份见顶部。仍需实际 board/component/shared-style/version 及库链接、发布和受支持导入导出路径，避免重复主库；空文件的存在不关闭这些缺口。
2. 桌面输入/正常窗口捕获队列及当前占用者、调度入口、可复用的正常 Editor 启动 recipe、project/EXE/case producer 身份。收到有效排期后再操作桌面。
3. TreeRow 状态/拖动、图表面、文档/Dock/focus、属性/诊断、真实 preview、Timeline/Curve/Transport 的具体消费者合同在 AB-SHARED-01…06；牵头归属已明确，实际组件/实现合同仍待接续，不另建平行实现。
4. 原 Jenkins `start-1792418123320632400` 的 degraded/owner-lost、固定 Codex EXE 缺失作为历史保留。当前新 runtime `start-1792836412672596900` 公开 ready，但本族元数据聚焦检查仍在提交前退出，见下方原请求记录；需要 Jenkins owner 对账自动登记/输入核对回执，不需要再次启动服务。本族不重启、另开 keeper/构建池或创建源副本。
5. 确认 Actor/Blueprint 目标的真实资产语义：已有 Prefab/反射组件适配，还是另有可编辑脚本类/事件图/接口资产。当前 prefab plugin 只读查询/诊断，create/open/apply/revert/break 写命令没有注册；通用 graph descriptor 不是 Actor 事件图后端。
6. 将 OVW-01…45 覆盖候选对账到独立 chat 的实际收件范围。资源域已承接 Audio 整体，但 Cue/Class/Submix 在其 RT-20 中仍待 Parent 确认图形能力和 owner；Widget 与 AI 是独立会话。盘点附实际 chat ID 和记录依据。Font/Font Face、Undo History、资源/像素/环境光查看器、类型、string table、世界创作和诊断等工具仍需窗口级收件确认，不认定全部候选都无人覆盖。

新增具体依赖：AB-SHARED-01 需确认原 WorkbenchListRow 示例实例及固定事件路由如何保留身份迁移到真实层级树；不能按 Mesh/LootSocket 文案做业务定位。AB-SHARED-05 的 WorkbenchPreviewViewport 已确认是静态 Image/SVG 默认实现、输入关闭，需要公共 renderer/input/resource 合同。两项均未改共享源码。

设计初稿及依赖已追加排队，回执 `01a16d70-e8d5-7d60-96ad-af43f41f8806`、exit=0。继续追踪后补正：runtime 除结构队列外已有 `World::remove_dynamic_component`，本族缺的是 Editor operation/journal/逆操作/retention 与多目标原子性，不能认定 runtime 没有移除 API。动态创建令牌为 World 内临时证明，序列化/投影重建不保留，不能作资产树持久 ID；具体切片与 crate 可见性边界已写入设计 AB-D03。

能力补正与当轮检查结果已追加同步，回执 `01a16d73-51ff-7ba0-9171-24e69c09f4f6`、exit=0。当时设计/文档共六份，26 个链接、64 个显式 UE 路径、33 项指纹和原资产身份检查通过；新增共享合同的该阶段结果为七份、39 个链接、66 个显式 UE 路径、43 项指纹。后续独立检查另记于 evidence/清单，不把该旧结果覆盖后续改动或升级为插件/Rust/画面验收。

## 可以接续的最小产品切片

先完成真实 Actor/Prefab 组件树与类/实例属性、从主场景定位打开资产、事务/持久化和正常预览；再接真实构造/事件图及接口。必须先有真实 Penpot 可编辑设计、保存/关闭/重开与 export，再修改对应原始 .zui v2。Component tree TabID 为 `ConstructionScriptEditor`，它不是构造脚本图，两者不要混接。

首次只读盘点阶段没有编译或生成请求。后续为核实已有 Prefab 实例元数据能力，已声明并沿正式入口接续同一受管请求；三次均在提交前退出，尚无 Cargo 执行或通过回执，详见下方。不是为纯文档重复构建 Rust，也不据公开 ready 声称测试已执行。

生命周期清单：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/artifacts.json`。保留设计 API/浏览器回执和完整原图；没有源码副本、构建或正式导出产物，实际释放 0 字节，自有失败 profile 小文件仍待统筹托管清理。任务仍 active，不能关闭为完成。

## 当前资源闭包与 Runtime 请求接续

[source-integration-audit-01](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/source-integration-audit-01.json) 已记录 AB-LOAD-01…03、20 项读取时指纹和原请求日志。canonical 插件缺 authoring/prefab_instance ZUI；插件 root 来自实际 discovered package manifest，严格 owner-relative 解析并在全部源编译后原子发布。Prefab Inspector 没有 template/data-root/bindings；通用 host 实际 invoke operation，缺口为本域绑定和写工厂。另 canonical page/index 与 native pane/test 期待不同 source suffix，尚未核实生产 remap。请 Parent/原 pipeline owner 交当前包、bundle/source/instance/review-host 闭包；这三项不是已观察的 EXE 失败，不补复制资产或修改共享检查来关闭。

受管请求固定为 `actor-blueprint-runtime-prefab-metadata-20261024` / `validate-prefab-metadata-01`，本会话原 session，driver `fc553228…`、runtime `start-1792836412672596900`。范围为 `zircon_runtime --no-default-features --features native-host --lib`，filter `prefab_instance_metadata` 的 3 项已有测试，test profile/jobs1/test-threads1、物理 E:/cargo-targets。两项 World roundtrip 之外包含正式 scene writer/reader，均只验证元数据，不是实例展开/作者命令或图编译。

01 输入中我误加的 `taskEvidence` 不在工具 schema，预检 exit2，原输入/日志保留；02 删除该字段后使用 legacy source policy，兄弟依赖两 tracked plans 被拒绝。已核实 installed capsule `baac70c97872d3395c5c422fa44a7cdaa481a8ac4f62820d1cc2ac099ad0ac08` 支持明确的 `sourceEvidencePolicy=full-file-index-v2`：保留观察字节/index/hash，区分 stage0/mode100644 的 root plan Markdown 与编译投影，未改 guard 或外部文件。03 沿原身份采用该协议后越过拒绝点，两次 52122 文件核对间检出动画域 `animation-parent-handoff.md`、`animation-shared-controls.md` 变化，exit2 `Source changed during pre-submit capture`。没有 POST/queue/execution 身份，也没有启动 Cargo；02/03 query-only 均为 `Current session authorization is pending`，不能把 pending 当作终态测试回执。

上述实际发现、原请求和下一项工作已通过官方 CLI 投递 Parent，ACK `01a16db5-38e3-7220-81fa-cf81a60aab83`、exit0。请求 Jenkins owner 接续同一 lineage 的自动登记/有界输入核对，保留并发修改，不盲投新 identity。该 ACK 不证明 Parent/Jenkins owner 已读或已排期。自有提交客户端已退出，无 Rust/EXE/桌面输入；公共组件/导出/Actor normal recipe、0 passed/accepted 与 126 B 待托管清理状态保持开放。

后续补充最新 11:42:40Z readiness、清单完整性和真实 pane-data 读取 handle 合同时，官方 queue 返回 exit1：`thread/queue/add failed: queue cannot contain more than 100 submissions (code -32600)`，未产生 ACK；没有把它计作送达或覆盖上述成功记录。完整补充保留在本域 shared-contracts、evidence 和 artifacts.json。没有删改他人消息或重复投递满队列；Parent 可沿已成功入队的本域路径读取最新记录。

## Parent 本轮收件后的 Prefab 作者适配

已读取 Parent 的 25+45 项分工与资源闭包交接。Parent 在 11:01Z 看到的 revn2 区域板保留为早期观察；后续实际 revn4 的两业务主件/12 实例没有改变完整窗口及产品验收未完成的结论。规范插件 URI 映射仍按 AB-LOAD-01 等待实际 package/root/bundle，不创建假资产；脚本类/接口/构造/事件后端仍由 Parent 对接。

按本域 adapter 分工，已修改 `zircon_plugins/prefab_tools/editor/src/{authoring.rs,lib.rs,authoring/snapshot.rs,tests/authoring_snapshot.rs}`。新增借用真实资产的只读来源/覆盖投影和可定位诊断，保留重复项、原序及旧校验消息；未注册固定 producer，未加写命令、平行 store 或公共 trait。读取 handle/document revision 仍需宿主合同。四文件格式/差异检查及 change guard 通过，guard 0 errors/0 warnings，三项新增测试尚未执行。

独立插件验证使用新语义请求 `actor-blueprint-prefab-authoring-adapter-20261024` / `validate-prefab-authoring-01`；原 Runtime 元数据请求与三份失败完整保留，不借新 ID 重试旧请求。当前 installed capsule 已核实允许 `manifestPath=zircon_plugins/Cargo.toml` 的单包结构化 recipe；模块脚本和 cargo argv adapter 不能表达该清单，因此仅用正式 `submit --input-file`。范围仅 `zircon_plugin_prefab_tools_editor`、features 空、no defaults、lib/filter `prefab_`、test profile/jobs1/test-threads1、E:/cargo-targets，预期 7 项非 ignored、1 项既有 release performance gate 保持 ignored。输入及源指纹见 [adapter 证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/authoring-adapter-01.json)；提交/查询结果在相同证据和下方接续，不把准备或客户端退出计作执行。

此次插件客户端实际 exit2：两次 capture 均完整释放 pins，前后 52131 个输入只报告四份外来文档变化：`08-resource-tools-design.md`、`08-resource-tools-shared-contracts.md`、`08-resource-tools.md`、`settings-plugins-parent-handoff.md`。本域 Rust 文件未被列入漂移项。无 POST/queue/execution 回执，未启动 Cargo；原身份 query-only 仍为 `Current session authorization is pending`。客户端已退出，不重投或删除他人文档。请 Jenkins owner 沿上述插件身份接续有界稳定输入核对，旧 Runtime lineage 保持独立。

另只读核对了实际历史产品：[packaging-inspection-01](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/packaging-inspection-01.json)。N57 staging 的 675 项中只有原 canonical Prefab 工作区与三个图标相关，实际部署的原工作区哈希与 checkout 一致；两个旧 native pane suffix 文件均不存在。R94 项目只声明 rendering/obj_importer/shader_wgsl_importer 选择，没有声明 Prefab。这不是所有运行发现来源的证明，也不是已观察的 EXE 错误，仍需实际 Prefab package/root/enablement 与 Actor 打开/data/host/case 闭包；未启动该 EXE 或重用旧 nonce。

12:00:12Z 独立 API 仍为本域 revn4/117 objects/2 mains、公共 revn0/1 object/0 components/colors/typographies/isShared=false，响应哈希与此前设计引用/文字复核一致。重新打开既有 640 原图确认 URI 与左栏标题裁切；没有修改文字值掩盖它。公共 Tree/Property/Dock/样式和生产数据注入合同仍待接续，桌面仍无授时段；0 passed/0 accepted。

本轮含四代码路径、插件原请求与实际包核查的新交接仅投递一次，官方 queue 再返回 exit1 / `queue cannot contain more than 100 submissions`，没有 ACK。完整消息及 SHA 保存在 artifacts.json 的 `coordination.failed_supplements`，此前失败也保留；最后成功 ACK 仍为 `01a16db5-38e3-7220-81fa-cf81a60aab83`。未删改其他会话消息，也未把本轮代码交接写为已送达或已读。

## Widget Graph 后端合同收件

按 Parent「Widget/UI 第二次接续」核查，具体七项合同 AB-WG-01～07 已写 [本域共享合同](actor-blueprint-shared-contracts.md)，47 项初始来源指纹及限定搜索保留在 [widget-graph-contract-01](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/widget-graph-contract-01.json)。可复用的实际基础包括 UI 源三元组和 binding 编译来源、真实 command registry/ListOperations、retained 通用 Route/Action 消费者、UiInvocationResult/错误、真实 VM 项目编译与 generation callback；均不等于 Widget 事件图已接通。

未接通项：UI binding → class/variable/function/interface/event graph 的持久身份和 callable SourceLocator；preview 外部动作到宿主结果的关联；VM editor operation 的 Editor 消费者（既有 CR-VM-HOST-EDITOR-CONSUMER-0001，限定 Editor 搜索无该贡献消费）；graph 代码生成/编译诊断/运行 source map。还核实 VM operation 恰好三段/宽字符校验与 EditorCommandId 至少三段/小写字符校验不一致，不由 Widget 擅自改写 ID。

Widget 保有 UI 源/property/binding 与 UiAssetPreviewHost；Actor 接本域图语义/后端适配盘点，Parent 对接实际 script/VM 实现 owner并持公共命令目录、host、GotoDefinition/focus；材质仅共享中立图表面。此次只交本域文档/回执，无跨域产品修改、Penpot 操作、插件/Rust/桌面输入或新增图。消息的真实收件状态在下方和 artifacts.json 记录，不据此关闭窗口族验收。

实际投递：Widget 官方 queue exit0，ACK `01a16dd6-aee2-7ab3-9751-5b5fa1e7f6ae`，只证明入队，未确认已读或接收实现。Parent 同一合同新交接投递一次，exit1 / `queue cannot contain more than 100 submissions`（-32600），无 ACK；没有清空队列或重投。完整消息、SHA 与两份输出保留在 widget-graph-contract-01.json，Parent 失败追加至 artifacts.json 的既有 failed_supplements，原失败不覆盖。来源核查共 47 项初始及 7 项追加指纹；初始指纹复核无漂移。

后续结束复核发现 Widget design 并发更新（初始 28737 B → 29775 B），已保留两个指纹并重读当前 Events/Graph 和共享 owner 段；仍明确 Source/Graph、preview 外部 consumer 和真实 Graph 文档缺口，与七项合同一致。其余 53 项来源未变化；不覆盖 Widget 文档，也不将本次文档更新当作后端接通或已读 ACK。

## 共享插件入口 route-02 接续

Parent 的 [route-02](../../../.jenkins/records/ue-plugin-workspace-entry-20261024.route-02.json) 是已安装 baac 胶囊的 selector/parser 记录，明确没有材质编译提交；11 项文件 SHA 本轮均匹配。实际 `installed.json` 随后已选 `cdd14500fa39088ae2c9870f62825acc2e526ba647d24d77cd6287722b5d0f31`，driver/runtime 仍是 `fc553228…` / `start-1792836412672596900`；当前 manifest、runner 和选定客户端 SHA 也已核实，未用旧 canonical parse 代表当前能力。

沿正式 `tools/dev/local-cargo.ps1 -QueryOnly` 使用原本域 [Prefab identity](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/plugin-authoring-01.request.json)：单包 `zircon_plugin_prefab_tools_editor`、manifest `zircon_plugins/Cargo.toml`、defaults off/features=[]、`--lib`、`prefab_` filter、test profile、jobs=1/test-threads=1、物理 E:/cargo-targets。首个直接 PowerShell 调用未保留 harness `--`，已用参数检查定位并改为数组 splat；该解析失败原样保留。正确数组调用仍 exit1 / `build_scope_policy`：当前 pinned entry 的 `parse_cargo` 第 472～482 行只准 `zircon_plugin_material_editor_editor` / `material_graph_`。该限制在 QueryOnly 也检查，尚未进入 identity/原请求状态查询、source capture、POST 或 Cargo；不能把这次结果写成新 authorization 或 Rust 失败。

实际回执与精确参数见 [plugin-route-02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/plugin-route-02.json) 和 [当前 wrapper 查询输出](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/plugin-authoring-02-query.json)。原 request `actor-blueprint-prefab-authoring-adapter-20261024` / `validate-prefab-authoring-01`、四份 Rust、旧 source-drift/authorization-pending 历史均保留；没有提交新测试或借用材质 filter/修复 request。下一步请公共 Jenkins 入口 owner 在现行 installed/client/runner 合同中交付明确受限的 Prefab 单包支持，再核对本域原请求并提前声明实际编译命令。仅验证解析不能关闭 Rust 或 normal EXE 验收。

Parent 仅委派材质 owner 接续 `zircon_editor/src/ui/graph/mod.rs` 的两个既有 proof/prepared 类型重导出；本会话不并行修改。保留 retention/prepare/一次 permit 语义，实际交付后才消费，不把重导出或 Script metadata 当作 Widget Graph 后端。

本次当前 cdd/Prefab 范围新交接通过官方 queue 投递一次，exit1 / -32600 / `queue cannot contain more than 100 submissions`，无 ACK；完整消息和失败输出已保存在 plugin-route-02.json 并追加 artifacts.json，未重投、未删除已有队列项，不记作 Parent 已收到。

## 本轮真实Penpot接续：revn12及原生导出404

明确写入范围仍为本域`actor-blueprint*.md`、本域reports和已分配Actor Penpot file/page；本轮未再修改四份Prefab Rust或任何共享/产品源码。25项本族+45项额外盘点及Parent窗口收件表保持，盘点不计实现。file `b4a1e3ce-9e6a-81fe-8008-d69d60f73bf1` / page `b4a1e3ce-9e6a-81fe-8008-d69d60f73bf2`、五board及两个业务component/main UUID不变；最终独立读回revn12/vern0、117objects/55flex、2主件/12copies，modifiedAt仍11:07:45.880Z/.881Z。12copy及36子shape-ref、字段原值逐项核对，错误为空；内容仍标识为测试fixture，未接实际资源查询或命令。

640板`d6a3256a-7bc8-56c7-8b37-c1241993bb33`的初始窄列现在由真实原生尺寸事务重排为Left94.8/Viewport355.2/Details182，来源URI完整，普通属性/警示自然换行，两处左标注高32。182来自当前Roboto字体URI量测181.8943取整，不是新全局Dock/样式常量。900/1280保持原分配，原生标注重测后的图对应revn9；640最新图对应revn12。设计和全部失败逐图表在AB-D08，总23张浏览器原图（本轮9张），其中API只写width但未重算selrect/points的失败保留并已由原生事务替代。不能用文件名中的balanced作为通过结论。

实际[flow-05回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-domain-flow-05.json)汇总原生几何、关闭/全新context重开、真实进程退出、8项Penpot源码SHA、身份与逐图观察；[最终640图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-flow-640-final-native.png)可直接复核。仍为空区域+本域fixture切片，未获得公共Tree/Property/菜单/Dock组件、完整Toolkit、共享样式或实时viewport，未达到≤1DIP/任何LayoutReviewCase验收。

本次首次实际走原生中文“新增导出”/“导出1个元素”：PNG1×预设保存为revn10，但`http://127.0.0.1:19001/api/export`返回404，随后内部错误页，20秒内0download；900/1280同边界不重复请求。见[receipt-11](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-11.json)。随后fresh context可正常重开，未启停服务、清库或绕过为静态导出。请Parent/原pipeline owner核对当前导出路由；官方ZUI插件、原资产映射及样式→产品变化链仍另行pending。

12:57Z公共file只读仍revn0/1根对象、0components/colors/typographies、isShared=false。具体共享需求已追加本域shared-contracts：正式库component/style/version和Tree/Property/Dock窄宽/复制/tooltip/焦点/滚动合同；现行cdd入口需精确Prefab单包支持，保留原request/operation，未借材质filter。AB-WG七项实际能力/缺口合同不变，仍需脚本/VM执行owner与SourceLocator实现。

正常捕获请求继续按原交接排队，当前仍缺Actor专用打开/data/host/case和EXE/资源依赖闭包，未申请或占用桌面/DPI时段；待这些ready后覆盖1280×800、900×620、640×520及实际144DPI和18项状态。没有启动EXE、借R94旧nonce/root-case或使用其它域画面。Rust/插件本轮未运行、产品ZUI未改，passed=0/accepted=0。新交接的真实queue结果写入下方及flow-05/artifacts.json；入队不代表已读，失败不标送达。

本次revn12/原生导出404的新进展交接已通过指定官方queue投递一次，实际exit1/-32600：`queue cannot contain more than 100 submissions`，无ACK、未送达、未确认已读；没有重投、删改或清空已有队列。完整消息、SHA256、失败原输出保留于flow-05.coordination，并追加artifacts.json既有failed_supplements。此前成功ACK及所有失败历史保持，不把本轮交接标为Parent已经收到。

## Widget 第三次接续：实际 Definition locator 交付

本轮精确范围为本文件、`actor-blueprint-shared-contracts.md`、`actor-blueprint-evidence.md` 及本域报告/manifest。已读 Parent WG-C07、followup-03 与 Widget 设计，新增 [AB-DEF-01～08](actor-blueprint-shared-contracts.md) 和 [源码回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/widget-definition-locator-01.json)：48 项输入指纹、11 项 VM pin 比对、三项限定零命中搜索、现有 API、必要字段/诊断、五组精确候选路径和原 owner 均可审阅。本轮不申请或使用跨域 Rust、服务或桌面写权。

核心发现：`E:/Git/zr_vm` 的 C LSP `ZrLanguageServer_Lsp_GetDefinition` 与语义查询/快照在 plugin pin `3c1352ecb6ddfab02752d46682cc376558d5f481` 已存在，可从真实源 URI/位置提供物理文件或只读虚拟声明的范围；safe/sys Rust 未暴露这些查询，pin 的 sys build.rs 关闭 LSP。当前项目插件仅 open/compile/start_session，不能当已接的 Definition 服务。请 Parent 将这组具体出口交既有 script/VM language/binding owner 核实最小桥接，不要求 Actor 新建 VM、Graph backend 或服务。

Widget 的独立可做接续：在现有 `source/source_sync.rs`、`session/navigation_state.rs` 与 current/last-valid 缓存补 binding 声明 span/真实 owning component 归属，保留原 ID/event/target/payload/source，精确关联文本修订。当前节点块可作为明确标记的源位置降级，不称精确 binding 定义。Parent 在原 `host/asset_editor_sessions/{open,editing/navigation,editing/source}.rs` 和 command/operation registry 接实际 handler/provider 准入、目标文档、history/focus 和完成/错误/取消；脚本/虚拟声明/图的目标 editor 未证实，需真实注册后才能导航。

VM registry 的 callback 可还原登记 module/export，数字 slot/module/function 和 symbol ID 都有代际范围；仍缺 handler→成员→定义位置桥接及持久 class/graph/member/node/pin authority。`GetDefinition=false` / stdio空数组不能直接判定符号不存在；multiple/virtual/no-provider/stale/invalid-current-source 等结果必须诚实区分。已有 TrackDependency 失败被忽略的 TODO 保留，不声称全部快照边界已验证。类/接口/构造/事件图继续 open；材质仍只中立 Graph 表面，无新共享源码写入。

本次交接通过官方 queue 的实际结果另记本回执及 artifacts.json，入队不等于已读/实施；历史队列满失败不删不重试。三文档结构检查与原资产身份核查不计插件/Rust/正常窗口通过；本轮没有 Penpot/产品变更或新截图，passed=0/accepted=0，全族未完成。

实际投递：Parent 新 Definition 消息一次尝试 exit1/-32600，仍因 `queue cannot contain more than 100 submissions` 被拒，未送达/无ACK；完整正文SHA256 `fd5fc948b171e101576783ad37942865a6c378f1c67a45454e82fc1e91a7b3ce` 与失败原输出保留在 Definition 回执，未重投或操作旧队列。Widget 新消息 exit0，ACK `01a16e11-5fa6-7762-8403-12a66e2feb01`，仅确认入队，未确认已读或实现接受。现有计划/差异和25+45盘点不变；实际释放0字节，旧2个126字节临时标记继续待原Jenkins清理，不称关闭。

期间Parent记录与Widget设计并发更新，已重新读取当前WG-C07和“Actor 第二次接续”，保留前后hash与首次一致性拒绝。Parent已独立确认13:16Z的revn12/117对象/55flex/2业务主件/12copies及linked libraries=[]；此为Parent追加的独立libraries证据，不倒填本域旧get-file缺字段时的判断。Parent另报告PNG404已定位为当前`/api/`统一转backend、无export专用路由且exporter尚未启动/就绪，交原pipeline/Parent接续；本域本轮未独立审计该运行脚本或恢复服务，保留receipt-11并停止重复导出探测。原cdd/Prefab范围与normal host/data/case缺口仍一致。本次新增Definition技术证据未因这两份交接文档更新而改变。

## Prefab 接续：原 host / 调用者的最小精确 diff

按本次Parent指令，新增 [prefab-host-injection-01.patch](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-host-injection-01.patch) 与 [50项输入/生命周期回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-host-injection-01.json)，并将 AB-PF-01～06 写回原 shared-contracts。patch SHA256 `eadd42e718a3cd1db94e9779dead5e00f0bff2fba93cadd53127f5b65b626066`，是可审阅而未应用的三文件候选；不请求新增公共写权，也未改原四份 Rust。以本节供Parent读取，不再向满队列重投。

精确注入点：`zircon_editor/src/ui/retained/app/host_lifecycle/pane_payloads.rs` 的原 target 分支保留 instance_id，传给原 `host/editor_event_runtime_access/extension_access.rs::ui_template_pane_data_snapshot`；后者用现有ViewRegistry定向核实活跃实例/能力/template。`host/editor_event_runtime_access/snapshot.rs` 提案增加私有 `read_template_pane_scene_component`，在同一已提交scene绑定和Edit单选中，经现有World借用读一个真实组件原值及data/schema generation，回传完整scene/project/source/gateway/selection/view身份；没有fixture、盘上加载、compile、World克隆或写permit。

实际Prefab接法：场景保留记录的完整键是 `zircon.prefab.instance`，不同于插件注册 `prefab_tools.Component.PrefabInstance`；Parent在宿主侧使用真实Runtime类型解码后，将typed instance和有依据的source_state交原 `PrefabInstanceAuthoringSnapshot::new`。该域crate当前serde_json仅dev依赖，未擅加生产依赖。来源查询用原Project/AssetManager，在shell/World借用外加载并关联Prefab自身revision/request；catalog membership/source_hash或scene.source_revision都不等于引用资源已加载。Unchecked不能改成Missing/Available掩盖未知或失败。

这份diff明确尚不解决三项依赖：①原无参DataSource与template-keyed batch/消费仍没有逐view文档结果，应由Parent在原 `committed_shell_state.rs`、`pane_payloads.rs`、`pane_projection.rs`、`product_state.rs` 交真实消费/发布合同；不另建全局map或扩公共trait。②stamp仅是观察，延迟发布前完整重读比较并消费取消；当前ViewInstance未见incarnation，关闭后同ID重开须原host retention/cancel排除，失效绑定与旧值清除仍未接，不能只比字符串。③当前读取限scene-instance；Prefab asset文档/selection/revision authority、字段写入、inverse/history/undo/save仍缺，不把source_index当写target。完整Blueprint backend和AB-DEF合同保持open。

候选初版适用性check与stdin格式解析通过，原四Rust静态证据复用；格式化版适用性及本域结构/指纹检查结果见回执，均不算Rust类型/插件/正常EXE通过。本轮未运行Jenkins/query/Cargo、PNG导出、服务或桌面，未新增Penpot画面，原31node/29control IDs/15events/7imports不变；passed=0/accepted=0、全族未完成。旧队列失败/Runtime lineage/126B待受管清理保留，释放0字节。
