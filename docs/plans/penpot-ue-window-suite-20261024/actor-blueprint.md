# Actor / Blueprint：窗口与真实命令核查

## 实现35：筛选路径与现有展开状态组合

原reader新增[visible_filtered_hierarchy_rows](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:287)，先按原borrowed记录筛选，再只向留下且可见的分支查询完整u64展开状态。无关子项被裁掉后has_children=false；未筛选入口与新入口共用私有遍历，不再由caller拼第二套可见行逻辑。[语义审阅35](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-visibility-35/semantic-review.json)给出边界与两项新回归。

当前42定义=41普通+1原ignored，原40项字节保持；fmt/四源guard通过，Rust verified0/actual null。原02/queue15836/#387/accepted19与所有失败保留；本轮只交源修订，不查询/重投或补签执行。真实per-view生产、搜索副作用、正式导出/Normal Actor/150%DPI/双端≤1DIP仍open，goal active。


## 实现34：原生来源的匹配路径投影

原reader新增[filtered_hierarchy_rows](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:258)：caller直接匹配原borrowed实体/组件记录，结果只保留命中及有效祖先，重算筛选后子项标记，保持原森林顺序与完整u64。重复ID仍Ambiguous，断边与切环不猜父；旧read不刷新，Ready-empty与NotResident分开。[语义审阅34](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-34/semantic-review.json)记录三项新回归与边界。

当前40静态定义=39普通+1原ignored，最终fmt/四源guard通过，Rust verified0/actual null；原37项函数体保持。UE搜索另会展开路径、选择节点及根fallback，不能沿用设计33的仅收起规则，详[AB-PF-26](actor-blueprint-shared-contracts.md)。原02/queue15836/#387/accepted19保持；生产caller、搜索/焦点与正式导出、Normal Actor/150%DPI/双端≤1DIP仍open，goal active。


## 设计33：收起后的选择与来源Details

原Actor file/page已新增三宽各三种可编辑状态：展开且选中6、收起且无选择、重新展开且无选择，另含六个窄窗Details目的地。内容精确来自[source32可见性回归](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:1383)：七条同名来源记录与四项层级诊断；这是标明来源的单元设计输入，未接运行中的Editor。

实际rev207→249／4634对象，15新root／1773新对象，原2860非根对象及两个main不变，3720来源引用均有效。96文字、12原生导航和三处Shared颜色覆盖保存；关闭后新context读回完整data相等，三宽fresh PNG与稳定展开图逐字节相同。身份、边界与28图复核在[结果33](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/result.json)。

新核实的UE默认是[隐藏项不保持选择](../../../dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/STreeView.h:635)，重新展开不自动选回；本域空Details是readonly来源视图的设计，不能据此声称所有Blueprint Inspector均为空。原caller消费边界见[AB-PF-25](actor-blueprint-shared-contracts.md)。37静态定义／0 verified Rust、原02/#387与5保护源保持；正式导出、真实producer/事务、Normal Actor/150%DPI/双端≤1DIP仍open，goal active。


## 实现32：来源树按宿主展开状态生成可见行

原reader新增[visible_hierarchy_rows](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:234)：复用原生森林顺序，向caller查询当前来源内完整u64实体ID，只对可见分支查展开状态；不写树状态或选择。重复/丢失/歧义父节点与切环根保留，收起只隐藏其有效后代；实体parent不冒充UE组件attachment。[源审阅32](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-visibility-32/semantic-review.json)记录实际边界。

三项新回归使当前37定义=36普通+1原ignored；最终fmt/guard通过，Rust0/实际总数null。原02/queue15836/#387/accepted19保持，不重投；生产caller、正式导出导入、Normal Actor/150%DPI/双端仍pending，goal active。[结果32](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-visibility-32/result.json)。

## 实现31：实例来源与原resident key的只读投影

原reader新增[instance_authoring_snapshot](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:156)：以实例自身Prefab引用和原native key检查，再返回既有borrowed override DTO；错误不伪装为Available/空来源，不解释entity/property path或授予写能力。三项回归覆盖引用迁移与opaque/冲突保留、foreign source/manager及registry拒绝、同ID同数字revision重建拒绝旧key，[结果31](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-instance-source-31/result.json)与[审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-instance-source-31/semantic-review.json)可消费。

仅原authoring.rs/tests两源推进，lib/snapshot与5保护不改；当前34静态定义=33普通+1原ignored，fmt/guard0，Rust verified0/actual null。原02/queue15836/#387及accepted19不改、不query/重投/第三ID；生产per-view publisher/typed rows、正式导出导入、Normal Actor/150%DPI/双端仍pending，goal active。

## 设计30：两组歧义组件的全部来源记录

原Actor file/page继续原生可编辑设计，source29单元输入的Z/A两组描述及全部五条opaque记录已落入1280/900/640和两窄窗Details。保存关闭后由全新context重开：rev207/2861对象，原2269非根对象与两个本域main精确不变，2205来源引用有效，Shared5 data不变；[结果30](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/result.json)和[10图逐张记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/visual-review.json)给出实际身份与范围。

这是明确标注的source29回归输入设计，未接运行中Editor或新增fixture producer。640完整能力说明经原生Details可达；原生toggle打开/关闭成立，DOM焦点BODY，键盘返回仍open。现4Rust与5保护源不改，31静态定义（30普通+1原ignored）无新Rust执行；正式导出/原ZUI接入、真实per-view producer/typed rows/事务、normal Actor/150%DPI/双端≤1DIP和Blueprint图后端仍pending，passed/accepted=false，goal active。

## 实现29：来源组件描述冲突的位置可消费

原四份Prefab源新增 `PrefabSourceAuthoringSnapshot.component_conflicts`。每组 `PrefabSourceComponentConflict` 保存本次borrow的entity位置和全部component位置，按原owner/首次出现顺序输出；四字段匹配复用现有descriptor，保留原始payload/引用/数组。重复entity ID不合并组件组，未知provider不解码，描述相同不等于同一个组件实例；这些位置仅用于当前来源诊断，不能作为持久target或写权限。

[结果29](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-component-conflicts-29/result.json)与[AB-PF-22](actor-blueprint-shared-contracts.md)给精确API/源与消费边界。新增三项回归后31静态定义＝30普通＋1原ignored；格式/四源guard通过，Rust执行仍0已验证、实际总数null。原02/queue15836/#387受理19项/unknown终态不变，不查询、重投或替换输入。Penpot28 rev176仍为source27设计，新增冲突内容的运行显示、真实生产publisher/typed rows、正式导出/normalActor/DPI与全族验收均open，goal active。

## 设计28：source27组件只读内容的三宽可编辑设计

在原Actor file/page通过原生复制与文字/flow覆盖，新增三宽及两窄窗Details目的地；Actor156→176/2270对象，保存、关闭、全新context重开和末次只读观察data相同。原1678个非根对象精确不变，591新增对象均在五个新root内；1700来源引用有效，131 Shared copies、77本域copies及原2main保持原生关系，Shared5 data不变。[结果28](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/result.json)与[最终640预览](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/fresh-640-final.png)可直接复核。

内容精确取source27的reverse=true单元输入：同名根实体0与完整u64MAX、选中实体的原四字段描述/opaque payload/引用表，另四种lookup结果注明来自其它输入。没有固定fixture producer、provider解码或真实运行数据冒充；应用/校验不可用，不计功能完成。900/640原生Details点击开关与边界已测，焦点仍BODY。字体检查入口曾插入空段落，失败原图与修复保留；最终文字/flow及三宽7张新viewer图已重开复核，仍不称EXE/正式导出或≤1DIP通过。

本轮4Rust/5保护源不改，复用source27同字节fmt/guard，28静态定义仍无Rust执行回执；原02/queue15836/#387不查询或重投。20个自有观察birth无剩余，恢复profile消失，首失败profile153文件/7215255B仍待原管理owner受管清理，手工释放0B。真实per-view生产源/typed materializer、事务/undo、正式源导出及normal Actor/150%DPI仍open，goal active。

## 实现27：来源组件的精确只读查询

原 `PrefabSourceReadSnapshot` 新增 `component_by_descriptor`：先以完整u64解析唯一实体，再逐项比较原 `SceneComponentAssetRecord` 的type/provider/schema/version。缺失实体、重复实体、缺失组件、重复完整描述各有独立结果；成功只借用原opaque记录，payload和引用槽保持，不加载provider或解码字段。描述不是组件实例UUID、持久row key或写入许可，相同描述也不能绕过原native key和view上下文重验。

新增三项回归定义，覆盖实体/组件反序及全字段区分、缺失/歧义分支、同描述与相同数字revision的remove/recreate后旧key失效；当前28静态定义（27普通、1原ignored），尚无Rust执行回执。最终fmt/guard通过，原snapshot树源及5保护源未改；见[结果27](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-components-27/result.json)和[AB-PF-21](actor-blueprint-shared-contracts.md)。

本轮不改Penpot或渲染源，复用设计26的三宽层级/只读Details图；原02受理19项、input23f4088d/queue15836/#387终态保持，不重投、换ID或追加到已受理payload。真实per-view producer/typed字段、事务、正式ZUI导出、normal Actor、150%DPI与双端验收仍open，goal active。

## 设计26：无效父级与循环来源的三宽可编辑内容

在原分配文件新增1280×800、900×620、640×520内容板及两份compact Details抽屉，直接原生复制已有connected实例和流式结构。内容逐项对应[真实回归夹具](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:663)：重复ID、歧义父级、缺失父级、循环父边四项诊断，七个实体全部保留，显示六个根；循环成员11的有效子项13保持缩进、选中和只读Details。原始行索引明确仅属于本次快照，实体parent不能当组件attachment。

Actor138→156/1679，保存关闭、新context重开及末次读取data精确一致；591新对象均在五个新root内，原1087个非根对象与两业务main未变。1195个shapeRef全部解析有效（260本域、935 Shared），99 Shared实例根，Shared仍rev5/data精确不变。640初次抽屉留下26像素诊断残边，已用native宽度320→346和marker左移26覆盖完整诊断列；旧图保留。两宽实际点击开/关，900边界对齐、640底边差0.015625px；focus返回BODY，未计键盘返回通过。

见[设计与原图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/result.json)、[保存重开及引用](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/save-reopen-preservation.json)。这是source25单元夹具的可编辑设计，没有注册fixture producer或声称运行中的编辑器。4Rust/5保护源不变，复用实现25的fmt/guard；25静态定义没有Rust执行回执。原02已受理的19项、#387 unknown/interrupted与旧终态保持，不重投。

首个自有headless helper因自编遍历未终止而强制退出，失败证据保留；恢复helper正常exit0，25个观察PID/birth均无剩余，新profile已自动消失。旧11,413,746B profile仅交原管理owner按哈希处理，尚无server清理回执；手工释放0B。正式源导出、生产publisher/typed rows、写事务、normal EXE、150%DPI与双端≤1DIP仍open，goal active、passed/accepted=false。

## 实现25：真实来源的只读树顺序与深度

原 `PrefabSourceAuthoringSnapshot` 新增 `hierarchy_rows`，按已经校验并断开循环的实体父子关系给出前序、深度和 `has_children`。原 `entity_rows` 仍按序列化顺序借用完整实体；根与同级节点保持源顺序，重复 ID、缺失/歧义父级及循环成员不被丢弃。循环成员作为独立根，其有效子项继续可达；Ready-empty 返回零行，不把 NotResident/Stale 变为空树。这里仍是实体 parent，不能代替组件 attachment。

新增两项回归，原深层循环用例增加8193行遍历及空树检查；当前静态定义25项（24普通、1原有ignored）。四源格式检查和guard通过，Rust未执行。数据合同见[AB-PF-20](actor-blueprint-shared-contracts.md)，[结果25](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-hierarchy-25/result.json)保留精确来源和待验边界。Parent仍负责真实per-view生产者、typed行materializer、展开/选择状态与发布前上下文核验，本域没有注册生产者或修改公共树。

管理owner新对账已确认原02/queue15836对应management #387 FAILURE，State v14为unknown/management_operation_interrupted，六次resource_capacity后观察中断；不是仍在运行，也不是Rust失败。原19项已受理源码与当前25项分开，原input23f4088d、#348及所有终态不改，不重投或另建身份。Penpot138设计证据保留，实际窗口接入、正式导出、normal EXE、150%DPI和双端验收仍open，goal active。

## 设计23：640来源说明与原Details弹层随流式布局接续

在已有三宽内容板把模式文字改为“来源定义：尚未选择场景实例”，原生选择源custom Noto400；实际CDP各13个glyph均为NotoSansSC-Regular。字号13.333333、行高1.2和原Shared颜色pair保留。Inspector曾显示默认字体/14，但保存字段没有丢失原字号或填充，不能把显示默认值当持久格式损失。

640原mode从动作条移到root独立流式行，624宽/16高、单行；main由y88/h400变y112/h376。原Details toggle/destination不换身份，native marker下移23.9998845逻辑像素、原overlay高度400→376；实际viewer点击打开/再点击关闭，overlay/main底边相差0.015625像素。activeElement返回BODY，键盘焦点返回仍未验证。1280/900只改模式文字和字体。

Actor130→138/1088，保存关闭、新context重开data精确相等；80对象变化、1008对象不变，67 Shared根/690引用与两业务main保留，Shared5 data不变。旧640失败板和900弹层保持；13原图逐张看过，详见[结果23](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/result.json)、[字段复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/save-reopen-preservation.json)及[640真实工作区裁剪](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/preview-640.png)。

4 Rust及5保护源未改，23静态定义仍无执行回执；本轮仅一次原02 QueryOnly仍unknown/interrupted，不重投或覆盖19项accepted源。没有正式ZUI导出、真实per-view生产者/写事务、normal Actor或150%DPI证明；这仍是unit内容设计，goal active，passed/accepted=false。

## 设计22：来源内容板的原生实例伸缩与叶行

在原分配file/page内直接修改已有1280来源内容板及900/640 compact板，不新增区域板或main。Search surface/caption改为254/218宽与native左右约束；六行来源树的caption按无场景动作的内容宽度改为190/178。来源文档尚未选取场景实例，因此隐藏已有eye/lock及对应槽；原unit来源子实体没有子节点，隐藏其展开符，保留父行展开符、缩进和选中态。没有假定来源entity就是场景组件attachment，也没有接入按显示名/几何派发的命令。

保存关闭、新context重开及末次只读核对均rev130/1088；39个内部部件变化，其余1049对象精确不变。全部source/slot/event/字体/文字/shapeRef和两个业务main保持，690引用（本域212、Shared478）全部解析，67 Shared根与Shared rev5 data保持。旧640失败板及两份Details弹层保留。实际三宽与100%原生矢量裁剪共12张原图均已打开，见[结果22](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/result.json)与[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/visual-review.json)。

这是native consumer override，不是master修复或正式codec/export通过；没有重写旧allocation/paint guard或Shared。原ZUI与4 Rust不改，当前23定义仍无执行回执；不重复查询/重投原02。生产per-view/真实命令、搜索/焦点/滚动、中文行框、完整UE Toolkit及normal EXE/150%DPI/双端仍open。goal active，passed/accepted=false。

## 只读产品切片21：按来源内真实entity ID唯一查询

原`PrefabSourceReadSnapshot`新增`entity_by_id(u64)`，直接从私有原生载荷查找，返回`PrefabSourceEntityLookup::Unique(&SceneEntityAsset)`、Missing或Ambiguous。相同名称、序列位置与source_index都不参与查询；重复ID不返回第一个实体，不过滤或丢弃原诊断行。0与u64::MAX保留原整数精度，不创建UUID、持久target、缓存或新source store。

两项新回归覆盖同名/重排/完整u64 ID以及重复/缺失；Ready-empty原测试补零ID未找到断言。当前23定义=22普通+1原ignored；最终格式及4源guard通过，无Rust执行回执。[语义与源身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-selection-21/semantic-review.json)记录范围。实际来源lookup仍是历史快照读取，延迟使用前须先revalidate原native key，并核同project/document/view incarnation/selection/request。

本API不把Prefab来源entity ID当场景实例entity、组件attachment或写命令目标。Parent的真实caller可从revalidate返回的读取再做唯一查询；Missing/Ambiguous或Stale只允许在原context仍匹配时失效旧目标，不能清替换后的新view。原typed读取错误保留Reference/WrongAssetKind/NotResident/Stale/KindConflict，未新造重复availability枚举。

本轮不重复查询未变化的原02，不重投/改input/第三ID；原19项accepted source、实现20的21项及本23项分别保留时点，互不代验。未改公共host/producer/materializer/原ZUI或Penpot，复用已保存rev125的状态图，不当本API已接入证明。goal active，真实生产/正式导出/normal EXE/DPI/双端与Jenkins最终验证仍pending，passed/accepted=false。

## 只读产品切片20：来源证明与载荷成对保留

接续实现17，在原authoring/lib/tests三路径新增`PrefabSourceReadSnapshot`，私有字段成对持有原生`ResourceActiveVersionKey`与`ResourceSnapshot<PrefabAsset>`。`read`沿原UUID/类型/ResourceManager入口获取；`authoring_snapshot`从该载荷借用原有lossless实体行，成功的空scene保留为零行。NotResident/Reference/WrongAssetKind/SnapshotRejected仍沿原typed Result传递，不把读取失败改成空树或新enum。

`revalidate`只接受本对象原key，返回核过同一原生版本的读取，不fresh换key；原资源移除重建即使UUID与revision数字相同也拒绝。新增两项回归，并加强原opaque组件/引用行测试以核key、载荷Arc及借用指针。当前21定义=20普通+1原ignored；[语义与资格](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-resident-read-20/semantic-review.json)记录最终格式/静态guard，尚无覆盖本修订的Rust执行回执。

这是本域只读DTO，不是第二store/clock、residency lease、持久row target或view发布许可；caller仍须配对同项目registry/resources并在发布前核document/view incarnation/selection/request及原key。未接公共producer/materializer，不改schema/history/ZUI或制造UUID/fixture producer。Penpot三种状态设计沿rev125原证据，不因本API增加而算产品画面通过。

本轮对既有02只查一次，仍unknown/interrupted、input`23f4088d…`/resubmitted=false，详[查询](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-resident-read-20/operation-query.json)。当前21项是新的本域源码修订，不能让原受理19项的queue15836代验，不覆盖旧input/回执或重投已终结操作。后续由原authority沿现有身份核实际source/资源/native合同；goal active，本域继续独立产品切片，整体passed/accepted=false。

## 设计19与原02实际收件后的接续

本族goal保持active。新增三张“原来源版本失效”可编辑状态板，1280×800、900×620、640×520均通过原生操作保存、关闭与新认证context重开至rev125/1088对象。相对rev112新增95对象，原992个非根对象、两个业务main及旧板顺序保持；67个Shared原生根、690条shapeRef全部解析，Shared仍rev5且data未变。[设计19结果](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/result.json)与[20图逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/visual-review.json)保留全部初版失败及最终三宽图。

900初次fresh的标题/动作条交叠，已将原mode叶通过原生Layers移入根column；1280来源副本高度33不能容纳第三行，已将该副本的vertical sizing从fixed改auto，原生重算为48。没有普通内容坐标补偿或Shared/main改写。20处编辑文字实际为source custom NotoSansSC-Regular；其余中文、16高行框与约19高字形、公共控件内部布局、真实焦点和滚动仍待核。Stale只证明旧native key不可用，不判定具体发生了更新、移除或跨manager；最终文字已按此收敛。

原request/session的`validate-prefab-source-adapter-02`本轮实际受理queue15836，accepted inputDigest为`23f4088db757a892412560ed913bd4541d3451db75c37e28476fc715394a0e98`；原1801B候选`86b638a0…`、原01/#348及其终态均保留。两次完整capture各Engine52242文件/985990739B后，查询曾为resource_capacity；本轮最后实读为unknown/pending、`management_operation_interrupted`，消息为HEAD、完整index或source membership在capture中变化。详[实际阶段与交接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/resume-execution-17/interruption-reconciliation-handoff.json)。没有Rust执行或最终终态回执，不能由客户端exit0推断server进程/EOF或测试通过；不重投、换第三ID或改已受理输入。

本段及同期五文档属于上述中断实读之后的文档修订，原受理时有限源元组仍保留，不能拿旧元组冒充当前完整source identity。四份Rust保持实现17的19项定义（18普通+1原ignored）；原ZUI与五份保护源不改。设计仍是unit状态，真实per-view publisher/事务、原始ZUI正式导出、normal EXE/实际150%DPI与双端≤1DIP继续open，passed/accepted=false。

## 监督恢复后的实现17与设计18

goal按Parent监督恢复为active。本域在原四源码中的三份新增`prefab_source_snapshot_expected`：沿真实UUID重解析来源并要求原native key仍有效，拒绝其它asset/ResourceManager及remove/recreate后的旧key，不自动采用新版本。新增3项回归后当前19定义（18普通+1原ignored）/0执行；[当前源码语义审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/resume-execution-17/semantic-review.json)与change guard通过。source08/final09/02候选保留历史，公共host/生产publisher/事务/原ZUI未改。

既有`validate-prefab-source-adapter-02`实际QueryOnly为original_operation_missing后，normal准备捕获九根输入，在Engine首轮52235文件/985805777B后因HEAD、完整index或成员集合变化exit2，无POST/Cargo；[原输出](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/resume-execution-17/submit.log)保留。首查询参数错误另已定位为PowerShell吞掉未引用的`--`，正式调用须字面量`'--'`；不归责具体写者、不复用#348。

原Actor file/page现rev112/993。六张状态板的独立mode说明改为“来源定义：尚未选中实体”，通过原生字体菜单选择source custom Noto400；[保存/关闭/重开结果](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-local-cjk-status-18/result.json)确认只6对象变化，987对象/2main/58Shared根/653引用及Shared5保持。18张三宽前/后/fresh已逐图看；六fresh SVG各11glyph为custom NotoSansSC-Regular，不推及其它中文/官方导出/Engine字体。生产caller/写命令/Graph、normal EXE/150%DPI/双端验收仍open，passed/accepted=false。

## 最新来源状态设计14

在原file/page新增Ready-empty和NotResident各1280×800、900×620、640×520，已保存关闭/新context重开rev107/993对象。复用18个新Shared原生实例，累计58外链根、653个有效shapeRef；原802非根对象/2业务main/旧画板顺序精确保留，Shared rev5/4main/34+3 data未变。[结果](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/result.json)、[18图逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/visual-review.json)及[保存/重开身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/save-reopen-preservation.json)记录全部UUID与source/覆盖。

已就绪空来源显示0实体/0组件；NotResident显示数量未知，均清除所复制板的旧实体、组件和选择内容。这是当前08只读来源API及静态单测载荷的状态设计，不是生产publisher返回或Blueprint根约束验证。首次flow位置/窄标题重叠和MinH失败图保留，最终窄标题直接进入根column、空态原生居中。640说明仍拆词换行；字体/长文本/真实焦点与命令继续open。

本轮4Rust/5保护源未变，原31节点ZUI/15events/7imports未导出改写。16定义仍0执行；无新Jenkins parse/query/submit/capture/freeze、服务或桌面操作。后继02、原#348和入口13保持历史谱系；本轮没有重核selected资格。浏览器exit0、18个已观察birth gone，失败profile新增153文件/7,206,915B保留，手删0B、物理释放未测。生产调用者/写事务/Graph、正式导出、normal EXE/实际150%DPI/双端≤1DIP仍open，passed/accepted=false。


最新入口变化见[后继对账13](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-successor-route-reconcile-13.json)：本轮只更新3份本域文档和原E根元数据，4Rust/5保护源9/9仍匹配，16定义/0执行。22:27实证与随后5fc只读源码观察分时登记；不把11pins或旧成功配方当当前可执行资格，无新parse/query/intake/Cargo/服务/桌面。12的14图及Penpot观察保持原证据。

## 最新紧凑设计12

[实际身份与14图结果](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/result.json)登记两组主板+Details，各98对象/10原生实例。新增共享控件仅两个Button副本；其余由本域内容板原生Ctrl+D保留，不复制main、detach、改Shared或主题。旧09/10/11有限11路径元组保留历史，本轮只更新当前文档元数据；4Rust及5保护源逐字节相同，16定义/0执行。原#348及未提交-02身份不变，无Cargo/query/submit/freeze。

[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/visual-review.json)保留自动锚点遮挡、manual原点错误、路由/重复SVG前置失败。最终点击返回图与默认图SHA相同；SVG与画布字形仍不同，Roboto400/700 loaded不关闭provider/CJK/Engine门。9个自有浏览器进程均退出/temp0，自动消失逻辑样本11,393,979B、物理未测/手删0B；旧11,152,398B失败profile及126B仍待原受管清理。

本窗口族尚未完成。25项本族和45项遗漏盘点不扩范围。最新Actor rev94/803对象、2本域主件、40个Shared原生copy；579条shapeRef全有效（379 Shared、200本域）。在原文件新增640/900紧凑Details可编辑设计，沿原资产已有drawer合同，保留旧640失败板；tree270、center346/606、Details320×400/500。保存、关闭context、新认证context重开及原生SVG点击展开/外部不关/同一开关关闭均实录，原606非根对象/2主件/全部原文字引用精确保留。14图逐张打开、累计87图；仍是只读unit fixture设计。生产caller、写命令、Blueprint后端、正式导出、normal EXE/150%DPI及双端验收仍开放，passed/accepted=false。

本轮[Shared消费10](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-sync-10/result.json)保留同步前完整基线、103个对象的实际delta及逐图复核；ID、文字、source/slot/event和覆盖保留，但几何/文本缓存并非逐字节相同。源数据最大数值差0.3800048828125不能替代双端1DIP测量，实际颜色仍#242424，尚非视觉A/B。四Rust仍匹配08/09的16项定义，实际执行0；本次五份文档更新使09的11路径元组只保留为当时身份，后继正式准备仍须取得当前完整source identity，不重放旧input或#348。

## 会话与写入边界

- 执行会话：`01a16d53-8965-7472-931c-73cba748fe8e`。
- Parent：`codex://threads/01a16d4e-0491-7353-84fa-038bd7706546`。
- 用户分工：Actor / Blueprint 的组件树、类默认值、构造逻辑、事件图、接口和适用关联窗口；另盘点其他会话可能遗漏的适用 UE 窗口。
- 用户指定实际 checkout：`E:/Git/ZirconEngine`；UE 参考树 `dev/UnrealEngine` 只读。保留进入任务时已有的全部外来改动。未创建分支、工作树、源码副本或主题系统，未提交或推送。
- 本会话写入 `actor-blueprint*.md`、自有回执、已分配的 Actor Penpot 文件及 Parent 明确接续给本域的 Prefab 作者 adapter。当前产品代码仅改 `zircon_plugins/prefab_tools/editor/src` 下四个明确文件；公共 catalog、tokens、Starship、正常捕获协议及其他窗口族源码不在本会话的独占写入范围内。
- [本族生命周期清单](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/artifacts.json)。保留Penpot API/浏览器回执及设计截图，没有编译或正式导出产物。自有浏览器均已退出，新临时根0文件；各轮自动回收样本见原回执，物理未测，手工删除0字节。旧两项126B仍待原Jenkins维护身份，不能写成全任务清理关闭。

## 当前执行依赖

| 编号 | 已核查事实 | 对后续工作的影响 |
| --- | --- | --- |
| AB-ENV-01 | 最新Actor rev94/803对象、40 Shared roots；579引用全有效，原606非根对象保留，保存关闭重开data相同。Shared实读rev5/4主件/34颜色/3字体样式，data未改；rev48/rev2为历史。原body与local-color目标及18:11未知保存边界不变。 | [源对照07](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-compare-07/result.json)保留精确来源和两目标边界。公共copy内部宽度、Property必需value、字体provider/CJK、正式导出、normal EXE仍open；不重放旧操作或占桌面。 |
| AB-ENV-02 | 原#348/queue14345仍FAILURE/preparation-conflict，prepare/compile=false。原owner22:27的a928实证client拒绝--manifest-path；本域随后只读installed为5fc41dbe…，11成员匹配，client仍同57153be4…，runner已有Prefab分支，未新跑parse/query。 | 同request的既有-02已明确选定，原86b638a0…输入只改operationId；09已审当前08四源/16定义，旧04/13项和有限11路径不是完整source。管理owner须在实际selected闭合并资格化client/runner/source链；不重投-01、不绕manifest或换身份，captureReady=false。 |
| AB-ENV-03 | `.codex/state/jenkins-coordinator/active.json` 不存在；正式 `.jenkins/state/management/installed.json` 存在，并绑定 driver `fc55322804fb53ab3e39f996c848ef5e5f3907497d3f38167b7095244b2a7399`。 | 不把历史 pilot 启用规则冒充当前正式服务可用。未启动旧本地协调器，未改 Jenkins 状态或固定 EXE 路径。 |
| AB-ENV-04 | 本轮仅原四份Prefab源码加入PrefabSourceAuthoringSnapshot：保留真实实体和opaque组件记录，按u64实体ID解析父级，诊断重复ID/缺失/歧义/循环且切断循环遍历边。新增3项回归后共16项定义，15默认可运行+1原ignored性能门，实际0执行。 | [source-tree-08](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-08.json)登记本轮修订与静态结果；旧13项回执不代验。Parent宿主/原ZUI未改，生产publisher、稳定写target/事务/undo仍待接；实体parent不提供component attachment，Make Root/Reparent继续不可用。 |

初次状态来自本族只读检查，最新变化明确归因 Parent 交接，不用旧 service-health 或新公开 ready 推断本域验收。没有使用本地 Cargo、原生捕获脚本、备用 keeper 或源码包绕过 Jenkins；独立 Penpot API/无头浏览器设计按 Parent 的明确分工进行。

随后收到 Parent 明确统筹指令：服务启动/恢复、共享库/主题、文件页面和桌面截图由 Parent 统一协调，当时尚无本域 Penpot file ID；本族继续源码/能力盘点及具体设计，不重复询问用户。已按要求汇报明确路径、计划、共享提案、缺口和下一项独立工作，队列回执 `01a16d6a-9ea9-7e61-844c-8c5753c0f1ae`、exit=0。此前 Jenkins 观察仅保留为既有检查记录，不因 Parent 来信而猜测服务当前已恢复。

再后续读取 [Parent 最新记录](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md) 和实际分配回执，确认已创建本域文件，不覆盖上述历史。实际入口：[09 Actor and Blueprint](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60f73bf1&page-id=b4a1e3ce-9e6a-81fe-8008-d69d60f73bf2)。独立 API 读回与具体消费者需求见 [共享合同](actor-blueprint-shared-contracts.md)。Parent 允许隔离 API/无头浏览器设计，尚未授予桌面输入/捕获。

## 参考身份与布局依据

实际源码的 `Engine/Build/Build.version` 声明 `MajorVersion=6, MinorVersion=0, PatchVersion=0, BranchName=UE5`。这不是已确认的 Epic 发布版本。参考树的 `git rev-parse --show-toplevel` 返回父 checkout，未解析出独立 UE Git 版本；不能把父 checkout 的 HEAD 当作 UE revision，已按逐文件 SHA-256 锁定本轮依据，见 [身份记录](actor-blueprint-evidence.md)。

本轮 Zircon 初始 HEAD：`75a76ee963792d5a316f738519e966937f770036`，分支 `main`。HEAD 不能表示共享未提交改动的完整内容；实际资源验证须追加依赖闭包哈希。

UE 视觉与操作优先，以 [UE/Penpot fidelity](../../../.codex/skills/zircon-project-skills/zr-ui-layout-reference/references/ue-penpot-fidelity.md) 为执行协议。布局的补充信息架构映射为 App Shell 4（三列及上下文区）、List（树与选择）、Settings Form（默认值及修改状态）、App Dialog（弹层/焦点返回）、Agent Activity 5（编译输出）。这些是项目 reference map 中的候选，尚未据此制作新布局，也未将其视为覆盖 UE 目标的依据。

UE Actor Blueprint 的 unified 模式由 `BlueprintEditorModes.cpp:517` 起定义：

- `Blueprints_Unified_Components_v7`：横向 0.15 / 0.60 / 0.25；左列 Components / My Blueprint 按 0.30 / 0.70 分栈。
- 中列 Viewport 与 Document 同属主文档栈，下方为 Compiler Results / Find Results / Bookmarks，垂直系数 0.80 / 0.20。
- 右列 Details 默认开启，Palette 默认关闭。关闭的 tab 不能简单删掉；需保留菜单重开及焦点恢复。
- Class Defaults、Components、Interface、Macro 有各自 application mode。类默认值不是主场景中已选实例属性的别名。

比例是 UE 初始停靠关系，不代表窄窗允许挤压到不可读；900×620、640×520 的溢出和面板恢复需在真实双端测量后记录，不能把任意改版当作忠实还原。

具体窗口及关联工具见 [窗口盘点](actor-blueprint-window-inventory.md)。模式、flow/滚动、类/实例事务、命令/焦点、原资产身份迁移和四环境测量锚点已细化在 [具体设计规格](actor-blueprint-design.md)；它不是已完成 Penpot 或产品接入。

## 已证实的 Zircon 能力与缺口

| 能力 | 真实源码依据 | 当前结论 |
| --- | --- | --- |
| 场景节点与事务 | `zircon_editor/src/core/editing/command`、`zircon_editor/src/tests/editing/transaction_engine/journal_scene_commands.rs` | 已有节点创建/删除/重命名/反射字段等命令与 journal 测试源码。尚未在本轮 EXE 中验证；不等于具备 Blueprint 类资产编辑事务。 |
| Prefab 资产/实例覆盖 | `zircon_runtime/src/asset/assets/scene/component.rs`；`zircon_plugins/prefab_tools/editor/src/authoring.rs` | 有组件记录及覆盖查询、重复路径/缺失 source 诊断。不能推断存在父类、CDO、事件图或接口编译。 |
| Prefab 导入与加载 | `zircon_runtime/src/asset/importer/ingest/asset_importer.rs:137`、`import_authoring_asset.rs:9`；`asset/pipeline/manager/assets/loading/load_asset.rs:128` | 内建 `zircon.builtin.toml.prefab` 使用真实 typed TOML parser，AssetManager 已有 `load_prefab_asset`。registry 优先可用 importer；插件的 DiagnosticOnly 不能据此推广为所有 Prefab 导入缺失。本轮未执行导入/加载。 |
| 实例元数据往返 | `zircon_runtime/src/scene/world/project_io/prefab.rs`、`scene/tests/asset_scene/hierarchy_sources.rs`、`asset/assets/project_document/tests/scene.rs:120` | 已有两项 World/Scene 与一项正式 scene writer/reader 元数据往返测试源码；本轮托管入口尚未启动 Cargo，不代表 Prefab 展开/Apply/类默认或实时预览。与插件 `prefab_tools.Component.PrefabInstance` 的 adapter 尚未核实；override 记录没有持久 UUID，重复路径不能作唯一可编辑行 ID。 |
| Prefab 写入操作 | `zircon_plugins/prefab_tools/editor/src/plugin.rs:80`；`editor/src/tests/cases.rs:12` | contribution batch 仅注册 inspector customization。测试明确要求 create/open/apply/revert/break 命令未注册、对应 menu 不暴露、asset type contribution 为空。 |
| 插件文档、Inspector 与宿主闭包 | `ui/runtime/runtime/plugin_documents.rs`、`core/extension/inspector.rs`、`ui/host/editor_event_runtime_access/component_dispatch.rs`、`retained/contract/componentized_workbench_regions/panes.rs` | 通用 `view.prefab_tools.authoring.open` 已由 editor_support 注册，不等于带资产上下文的 `prefab_tools.authoring.open`。插件 UI 解析使用实际包 root，canonical 对应文件缺失；Prefab Inspector 未声明 template/data-root/bindings。通用 host 会真实 invoke operation，不能称整体 no-op。canonical 资源路径与 native pane 预期旧路径不一致；实际包/EXE remap 未验证，详见下方审计，不宣称已发生运行错误。 |
| 现有 Prefab 工作区 | `zircon_editor/assets/ui/editor/world/extension_prefab_editor_workspace.zui` | v2 资产、稳定 component/control/event ID 已存在，但左树、表格、属性选项和统计为固定示例内容。不得计为真实编辑器。 |
| 工作区命令反馈 | `zircon_editor/src/ui/retained/dispatch/templates/workbench/feedback.rs:498` | open / apply / validate 返回 `Not implemented (preview)`。真实 prefab 写入链路尚未接上。 |
| 工作区导航 | `zircon_editor/src/ui/retained/dispatch/templates/workbench/navigation.rs`、`asset_editor_menu.rs`；`ui/dispatch/specs/world_building/prefab_and_scatter.rs` | 已有打开工作区和静态控件 action 映射，尚未证明主场景选择→精确资产 URI→独立文档→保存→实例同步。 |
| Blueprint 菜单 | `zircon_editor/src/ui/runtime/builtin/unreal_editor_template_bindings.rs`；`ui/retained/preview_actions.rs` | `workbench.unreal.blueprints.open` 是既有菜单入口；名字不证明有 Blueprint document/编译器。 |
| 图扩展契约 | `zircon_editor/src/core/editor_authoring_extension.rs` | 有 GraphPin/GraphNode/GraphEditor descriptor。注册描述不能作为真实 Actor 事件图或构造图执行能力证据。 |
| 场景脚本运行 | `zircon_runtime/src/scene/world/project_io/script.rs`、`zircon_runtime/src/script/vm/scene_system.rs` | 已有 `script.bindings` 的 package/module/enabled/properties，以及 onStart/onUpdate/onFixedUpdate 回调与插件代际刷新。不能据此认定有可编辑类资产、构造图、事件图持久化、编译器 source map 或接口作者工具。 |

禁止把 `PF_Chest_A`、`Mesh_Chest`、`LootSocket`、`18 children / 6 overrides / 2 warnings` 当真实项目数据。当前不改写原始 .zui；必须先取得真实 Penpot 可编辑设计与导出，再按原身份应用，不能用手写新画面冒充导出结果。

## 接入命令链与不能跨越的数据边界

| 用户操作 | 已确认的真实入口与所有者 | 本族接入约束/缺口 |
| --- | --- | --- |
| Add Component | `ui/host/inspector_component_add.rs` 生成实际 catalog/scope；`ui/retained/dispatch/templates/workbench/inspector_component_add.rs` 提交 `inspector.component.add`；`core/commands/registry.rs` 注册 `AddDynamicComponentFactory`；`core/editing/command/add_component.rs` | 复用 `AddComponentScope` 的 document history、gateway/runtime/session、selection/world/schema/instance admission。现有命令是场景实例的 absence-only Add，不能直接拿 class-default 列表或 Prefab 文档 ID 调用。 |
| Delete Component | Runtime `scene/ecs/commands/commands/facade.rs` 有 `queue_remove<T>` / `RemoveComponentCommand<T>`；继续追踪确认 `scene/world/dynamic_components.rs:326` 另有 `World::remove_dynamic_component` | 运行时已有实际移除 API，缺席返回 false并更新 presence/cache/generation；这仍不提供本族所需的编辑器文档事务、payload/reference 恢复、继承限制或多目标失败原子性。本次搜索的 editor editing/Prefab surface 未找到对应可撤销命令；不得让 UI 直接调用移除 API 绕过 history。 |
| Delete / Reparent Actor | `core/editing/command.rs` 的 `delete_node`、`set_parent` 和 `UpdateNodeCommand` | 这些处理 scene node，不是组件 attachment。组件树删除快捷键不可误路由为删除整个 Actor；entity parent 与 component parent 的语义需分别确认。 |
| 属性编辑 | `EditorCommand::set_reflected_scene_field` 捕获 node/type/field/value；`SceneComponentAssetRecord` 保留 provider/schema 及独立 asset-reference slots | 场景实例字段可复用既有反射合同；类默认值、继承 override、reset 与跨实例刷新仍需资产 owner。保留 reference slots，不能用无类型 JSON 覆盖整条记录。 |
| Prefab Apply / Create / Revert | Prefab plugin 的 `prefab_authoring_batch` 只有 inspector customization；authoring.rs 仅 effective overrides/validation | 需要真实资产命令、文档保存、实例同步和撤销合同。当前 preview route 不能作为写成功；新注册必须有意更新既有只读注册测试。 |
| 事件/构造图到运行时 | `script.bindings` → `ScriptSceneRuntimeSystem` → VM callback；`core/framework/script.rs` 有 host function/type descriptors | 这是可复用运行能力的候选边界，尚未确认图→module 编译/资源持久化、构造回调或 interface dispatch 的作者合同。先由 Parent/脚本 owner 确认，不在 UI 层新造 VM 或绑定假的 Compile。 |

上述结论来自限定目录的实际阅读和搜索，不宣称证明全仓不存在扩展命令。新的 owner/注册证据到达后更新本表，不用命名相似的 API 替代语义核查。

本轮 [资源与受管输入审计](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/source-integration-audit-01.json) 记录 AB-LOAD-01…03、20 项读取时指纹、三项元数据测试范围和原请求失败 lineage。它是源码核查，未生成平行的 LayoutReviewCase/通过格式；Parent 与原 pipeline owner 需要提供当前 package root、bundle/source/instance 闭包及 normal recipe，才能接续产品加载和原生证据。

## 公共依赖提案，待 Parent 统筹

| 提案 | 复用身份/owner | 本族需要的具体内容 | 验收要求 |
| --- | --- | --- | --- |
| AB-SHARED-01 组件树 | `WorkbenchTreeRow`、`WorkbenchPanelHeader`；Parent 公共组件 | 继承/实例/默认根状态、层级缩进、过滤、只读/不可删提示、可拖动行与焦点状态。业务项由本族模型供给。 | 行选择与稳定实体/组件 ID 关联；添加/删除/重父级可撤销；重绘和滚动后不换 owner。 |
| AB-SHARED-02 图表面 | 材质 chat `01a16d50-6c7f-7370-853d-4581184b66a1` 牵头；Parent 跨域合同/集成 | typed pins、连线、平移缩放、框选、上下文菜单、节点/引脚错误定位；复用 GraphModel/Canvas/DeltaCommand；Actor schema、pin identity/remap、exec/data verdict 和 compiler 归域 owner。 | 使用真实持久化图模型和执行/编译链路。Palette 实例可复用；无效果节点不计完成。 |
| AB-SHARED-03 文档与焦点 | `UiHostWindow`、`DocumentHost`、菜单/Dock/命令 owner 为 Parent | 资产 URI 对应 tab、脏状态、关闭确认、最近文档恢复、菜单/弹层返回原焦点、撤销 context 路由。 | 组件删除快捷键不误删场景 Actor；两个资产文档的 undo history 相互隔离。 |
| AB-SHARED-04 属性与诊断 | `WorkbenchPropertyEditorRow`、`WorkbenchDiagnosticRow` | 类默认值/实例覆盖标识、reset 路径、可定位编译错误、长中文与滚动。 | 属性行绑定真实类型/字段 ID；error 点击定位同一文档中的真实节点。 |
| AB-SHARED-05 预览 | `WorkbenchPreviewViewport` 与实际渲染宿主 owner | 该公共组件当前根是 Image，默认 `animation_mannequin_viewport.svg` 且输入关闭；需要真实组件增删/变换、重新构造、编译后实例刷新及 renderer/input/resource 合同，不由本族改成另一张图。 | 同一正常 EXE 的 GPU resource/present 及 source owner 证据；静态图片不算预览。 |
| AB-SHARED-06 内嵌 Timeline/Curve/Transport | 动画 chat `01a16d51-741b-73b3-a61a-2189cd38f81e`；Parent 跨域合同/集成 | Blueprint Timeline 子文档复用共享表面，同 owning asset/history、seconds 时钟、track/key/curve/component identity、可逆 delta 与实际 playback feedback；Actor timeline→graph/VM 由本域负责。 | 不复制独立 Sequence/时间轴或运行时；无真实模板/事件/求值合同的能力继续记待定。 |

公用身份来自 `catalog.toml`；具体 document、slots、binding namespace、组件版本必须在 Penpot 实例表中记录。沿用 `editor_tokens.zui` 与 `unreal/starship.zui`。本族未新增全局 token、颜色字面值或重复组件。

## 后续实施顺序与验收

1. Parent 交付真实 Penpot 文件/页面及共享库身份；明确本族独占 board 和桌面队列。检查现有设计后续作，先在 Penpot 完成可编辑组件实例、共享样式、flow、保存→关闭→重开，保存实际 update-file/get-file 响应及官方导出身份。
2. 先实现真实 Prefab/Actor 组件树、属性和场景→资产文档跳转这一可独立使用的切片；明确默认值/实例覆盖的语义。写入命令使用既有事务与资产保存所有者；不能把 prefab 查询接口变成假的 Apply 操作。
3. 事件图、构造逻辑、接口取决于实际类/脚本资产模型。若没有后端，保持目标设计、列明依赖；Parent 明确映射后实现，不将材质/动画图执行器冒充 Actor 脚本编译器。
4. 应用真实导出到原 `.zui` v2，逐项核对 component/control/event/binding/slot ID 及依赖哈希。对全局组件的变化先由 Parent 集成。
5. 使用受管插件检查、最小 Cargo 包和命名测试过滤器；提交每次编译前声明模块、包、features、target、profile、命令及覆盖理由。先检查 recipe 真实命令，拒绝全仓 fallback。物理输出使用 E 盘根 `E:/cargo-targets` 下的兼容共享池。
6. 正常启动 Zircon Engine EXE；每张图绑定 `LayoutReviewCase`、当前源/依赖闭包、EXE SHA-256、进程出生身份、nonce/generation/hold、实际 DPI、资源 readiness 与 GPU present。复用既有 normal-window evidence adapter，不能放宽协议使本族通过。
7. 同内容/状态/DPI 的 Penpot 官方图和真实窗口图生成并排与 diff；逐图评审边界及文本行框，目标 ≤1 逻辑像素。图像有效只允许该 case 为 `passed`，双端比较与逐图复核通过后才为 `accepted`。
8. 至少一次真实 Penpot 局部样式编辑→保存→导出→原资产应用→产品重新加载→截图发生对应变化。全局样式跨窗口传播由 Parent 组织，不能用手工改 ZUI 模拟这条链路。

完整行为场景及未完成项见 [差异台账](actor-blueprint-differences.md)。本文件不包含任何 passing 或 accepted 结论。
