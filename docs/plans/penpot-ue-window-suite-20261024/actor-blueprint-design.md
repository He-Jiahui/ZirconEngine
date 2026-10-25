# Actor / Blueprint：具体区域与交互设计

## 设计35：筛选后的展开标记以真实留下的子项为准

[原组合查询](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:287)返回筛选与展开共同决定的行；matcher在创建查询时遍历原native记录，展开predicate在消费迭代器时仅访问可见保留分支。父命中而子项全被裁掉时has_children=false，不再查询它的展开状态；收起仍隐藏有效后代，原root/sibling序与深度保留。

这应用的是宿主已持有的展开状态；[UE搜索回调](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:2224)主动展开/选择/根fallback不在本方法中。原选择/焦点/滚动、匹配语法、特殊Actor root和UClass继承仍依实际模型接续，详[AB-PF-27](actor-blueprint-shared-contracts.md)。本轮无Penpot/运行Editor输入；原设计33的三宽rev249仅作已保存历史设计，未签本轮fresh或真实产品搜索。


## 设计34：筛选来源与搜索选择分开接续

[UE树节点筛选](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectDataInterface/Private/SubobjectEditorTreeNode.cpp:262)把直接命中与ChildMatches合并保留路径，父自身命中不自动显示不匹配后代。[OnFilterTextChanged](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:2224)还会展开可见路径并选择靠前分支端点，无候选时fallback原root；这是搜索操作的额外行为，与设计33只收起/再展开不同。

本次[原readonly adapter](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-34/semantic-review.json)只返回过滤森林：按原source序遍历matcher一次，保留有效祖先，隐藏无关分支；parent-only命中时箭头按实际留下的子项重算。空匹配是零行而不是NotResident。未知provider/opaque组件可由caller读取原字段，不能按类型文案假造UClass继承；UE特殊Actor root规则也未映射成通用实体规则。

尚无真实搜索输入/展开/选择/焦点/滚动或per-view生产接线，本轮不新增Penpot状态或图。原设计33保存rev249/三宽原生状态仅复用历史证据，不当本轮fresh状态；原key/context、歧义拒绝和只退休旧view见[AB-PF-26](actor-blueprint-shared-contracts.md)。


## 设计33：可见性与选择的三个独立状态

UE[STreeView默认AllowInvisibleItemSelection=false](../../../dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/STreeView.h:161)，[可见行重建后移除隐藏选择并发送Direct回执](../../../dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/STreeView.h:635)；[SSubobjectEditor](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:1903)未覆盖该选项，并把实际选择集传给[UpdateSelectionFromNodes](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:2149)。这只证明树的选择规则，不把所有Toolkit的Inspector降级行为归为已核实。

原file b4a1e3ce-9e6a-81fe-8008-d69d60f73bf1/page b4a1e3ce-9e6a-81fe-8008-d69d60f73bf2新增以下九个主状态及六个Details目的地；全部15身份见[保存重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/save-reopen-preservation.json)。

| client | expanded-selected | collapsed-none | expanded-none |
| --- | --- | --- | --- |
| 1280×800 | fae24861-1f5b-8087-8008-d82d094a5e8f | fae24861-1f5b-8087-8008-d830bf26e46c | fae24861-1f5b-8087-8008-d82f6e7c096c |
| 900×620 | fae24861-1f5b-8087-8008-d82e391ae80d | fae24861-1f5b-8087-8008-d830c6683928 | fae24861-1f5b-8087-8008-d82f9064f3b9 |
| 640×520 | fae24861-1f5b-8087-8008-d82e58a27707 | fae24861-1f5b-8087-8008-d830ce670ddc | fae24861-1f5b-8087-8008-d82f98c39703 |

[source32输入](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/fixture-provenance.json)的遍历为1,1,2,3,4,6,5；收起4只隐藏有效子6，六条可见且原source仍七条。再次展开显示七条，但无选择；只有明确点击6才恢复其Details。重复1、歧义/缺失父和切环根不删、不按文案选目标。应用/校验仍不可用。

原生Shared TreeRow/Search/Button和本域SourceReferenceData继续以实例复用；无main复制/detach/Shared写入。三个无选择row从Shared选用editor.surface.recessed真实pair，style4185031b-9d94-5788-ac52-d541bca139e5/file...ac12；原字体/provider未换。96全文覆盖、3个隐藏wrapper、3次原caret旋转及12导航保存重开保持；flow自然重排，未API改几何。15次稳定原生状态跳转覆盖四条不同边，另外两窄窗Details实际开/关；[逐次回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/prototype-transitions.json)。

[14张最终图与14张初始/viewport图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/visual-review.json)按真实PNG身份复核：28图含17种不同内容，25张直接打开、另3张为已打开图的字节相同副本。初始1280两图顶7px过渡遮挡、900热点提示均保留，稳定图已排除；三宽fresh展开图字节相同。900抽屉右内缩8/顶88/底32；640右8/顶111.984375/底32.015625，仅Penpot内部几何，不是双端1DIP通过。箭头仅线条命中、BODY焦点、继承的旧relative-trigger ID、字体ink/CJK及实际宿主仍列开放。


## 设计30：组件描述歧义与完整原始记录

继续以UE[选择到Inspector对象集合](../../../dev/UnrealEngine/Engine/Source/Editor/Kismet/Private/BlueprintEditor.cpp:1239)为区域依据。内容严格来自[source29回归](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:967)的一个实体：完整18446744073709551615、same name、parent None；五条原记录按序0..4保留value1..5及各自mesh引用。Z.Component位置[0,2,4]、A.Component位置[1,3]共享schema test.opaque.Schema/version3/provider unknown-provider，不能合并为可写实例身份。仅本次borrow的序号不是稳定target；完整载荷/引用依据见[来源](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/fixture-provenance.json)。

| 原生root UUID | 尺寸与内容 |
| --- | --- |
| `5b8f1971-076a-80d3-8008-d8195644b955` | 1280×800，实体/记录列表、两组描述与原值、来源Details三列 |
| `5b8f1971-076a-80d3-8008-d81cf44b0ab6` | 900×620，双列及原生Details toggle |
| `5b8f1971-076a-80d3-8008-d81cf44b0b6f` | 900目的地320×500 |
| `5b8f1971-076a-80d3-8008-d81d0233ffa9` | 640×520，全部五条记录与完整实体ID可见 |
| `5b8f1971-076a-80d3-8008-d81d023431c3` | 640目的地346×376，能力说明完整可达 |

五板均在既有file b4a1e3ce-9e6a-81fe-8008-d69d60f73bf1/page b4a1e3ce-9e6a-81fe-8008-d69d60f73bf2，由原生复制、流式布局和文字覆盖形成；原Shared4主件/两个本域main复用，不detach或复制main，不改Shared/theme/catalog。实际组件/main/copy/shapeRef身份见[保存重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/save-reopen-preservation.json)。列表明确“序列化顺序，不表示组件附着关系”；未添加不存在的attachment模型。

640源板的能力卡原已hidden，最初把未显示误判为覆盖，已依据源/拷贝hidden=true补正；未执行隐藏修改。仅把两行来源位置/旧native key边界补入Details，[原生过程与补正](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/narrow-reachability.json)完整保留。900弹层上下/右边差0，640上下−0.015625/右0，是Penpot内部几何；BODY焦点不证明返回。[7张最终图及3张保存前图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-conflicts-30/visual-review.json)均已打开审阅。

实际SVG中文由system Noto Sans SC绘制、声明family仍Roboto，完整ID由custom Roboto绘制；SVG DOM文字rect16px，computed lineHeight19.0666px，不冒称glyph ink测量或源字体等价。字体/逐行≤1DIP与真实Engine加载保持open。搜索、行选择、快捷键、滚动和写命令尚无实际执行证明。

## 设计28：实体身份、组件描述与原始载荷

沿用UE实际[选择更新→Inspector对象集](../../../dev/UnrealEngine/Engine/Source/Editor/Kismet/Private/BlueprintEditor.cpp:1239)的区域关系；只设计已存在的readonly来源语义，不把组件定义字段当场景组件实例或Blueprint可执行图。既有TreeRow/Search/Button与本域SourceReferenceData实例原生复用，未复制main、detach、改Shared或造Field/主题；布局继续使用已有column/row flow与fill。

| 新原生root UUID | 内容与尺寸 |
| --- | --- |
| `d79ec46c-7a4d-80ce-8008-d8083ee3a17b` | 1280×800，实体树/组件原始数据/来源Details三列 |
| `d79ec46c-7a4d-80ce-8008-d80a34d4846a` | 900×620，实体树与组件内容，Details原生toggle |
| `d79ec46c-7a4d-80ce-8008-d80b0d218d01` | 640×520，完整u64单行可见；来源说明单独占行 |
| `d79ec46c-7a4d-80ce-8008-d80a34d4d661` | 900目的地，320×500 |
| `d79ec46c-7a4d-80ce-8008-d80b0d21d503` | 640目的地，346×376 |

[来源身份与单元输入](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/fixture-provenance.json)固定source27测试的reverse=true分支：实体0和18446744073709551615同名、均parent None，选中后者，5条组件记录；描述是test.opaque.Component/test.opaque.Schema/3/test-provider。仅展示原opaque payload和references[0]的船体资源locator，不造UUID或字段编辑器。MissingEntity/AmbiguousEntity/MissingComponent/AmbiguousComponent明确为其它输入的结果，非当前诊断。完整ID保持十进制字符串，不经过JS Number。

最终rev176的新context原生100% viewer三宽及弹层图见[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/visual-review.json)。900弹层三边差0，640上下差−0.015625、右差0；它们只是Penpot内部几何，不替代EXE比较。900关闭图保留点击轮廓，DOM焦点BODY，键盘焦点返回未通过；未测真实滚动/快捷键/事务。

字体probe实际取得custom Noto中文、custom Roboto完整u64及混合载荷的glyph来源，但进入编辑的辅助流程插入空段落；原失败图、rev172数据和[修复记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-details-28/text-correction.json)保留。修复以原生完整文档选择/精确文字与逻辑行检查，点击Layer退出，并正常宽641→640重排；640主区恢复y112/h376。源字体未替换；19px ink/16px linebox差异及实际Engine字体仍open。

## AB-D18：无效来源森林的三宽内容与Details

原Actor file `b4a1e3ce-9e6a-81fe-8008-d69d60f73bf1` / page `b4a1e3ce-9e6a-81fe-8008-d69d60f73bf2` 内原生复制既有内容板，不另造共享按钮、树、字段或主题。UE ordered children依据为`dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:2045`；本域内容取自[source25回归](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:663)，不是生产载荷或引擎截图。

| 尺寸 | 新board UUID | Details |
| --- | --- | --- |
| 1280×800 | 67885b70-d0cd-80bd-8008-d7f5a8a6924f | 三列，右栏显示实体13/原parent11/源定位/无组件记录。 |
| 900×620 | d98468eb-9a4e-8069-8008-d7fb991b008a | 原生toggle目的地d98468eb-9a4e-8069-8008-d7fb991b00df，320×500。 |
| 640×520 | d98468eb-9a4e-8069-8008-d7fcb0594c00 | 原生toggle目的地d98468eb-9a4e-8069-8008-d7fcb0594c55，修正为346×376，覆盖整列诊断。 |

树序为`5,5,9,10,11,13,12`，源索引`0,1,2,3,4,6,5`；只有13深度1、11有展开符，叶项隐藏caret，保留完整实体ID与名称，不按名称恢复目标。四诊断分别保留indices `[0,1]`、`[2]`、`[3]`、`[4,5]`。模式与页脚明确只读单元夹具，应用/校验不可用；640仅隐藏重复状态卡，四诊断和来源说明保留。原有所有板/overlay/组件main不变。

Actor最终rev156/1679保存关闭后fresh data exact，三宽Reset100%/DPR1的原生工作区裁剪像素分别1280×800、900×620、640×520。首次640抽屉320宽留26像素文字残边，改自身board宽346和manual marker x−26，目的地/事件/y不换；实际viewer开/关后与诊断列左边/宽度对齐，底边差0.015625px。900右/底边差0；两个focus均BODY。青色交互提示不等于键盘焦点返回。

[原生动作](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/changes.json)、[测量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/measurements.json)及[逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-hierarchy-diagnostics-26/visual-review.json)保留原失败/首次/最终图。mode实际编辑DOM为custom Noto，混合诊断文字仍见custom Roboto、系统Noto及箭头Times回退；不据此关闭CJK/19ink16line/Engine字体差异，也不称官方导出、生产调用或完整窗口验收。

## AB-D17：来源模式单行与640弹层边界

沿AB-D11/12原三宽内容板编辑三个既有mode，统一纯中文“来源定义：尚未选择场景实例”。只选已注册源字体`custom-2ddd793a-5f89-40b0-aec1-fbe879e37a57`/normal-400；同名字体的Google SVG预览与源custom文字预览分开，recent重复项通过原生虚拟列表grid限定，原locator失败保留。保存后的字号13.333333/行高1.2/Shared text.primary pair不变，各13个glyph实测源Noto。

640 mode `7634458e-b3cb-8050-8008-d73fbb45a9c3`用原生Layers拖到root `7634458e-b3cb-8050-8008-d73fbb4570ba`，位于header之后、actions之前，不新建文字或普通内容坐标偏移。宽128→624、高32→16；新增root行使main y88→112/h400→376，footer y496不变，两条opaque记录与状态卡保留。

依赖此flow变化，仅调整原toggle `e01d87e6-0c76-8063-8008-d7410589d576`的manual overlay位置和原destination `7634458e-b3cb-8050-8008-d73fbb4570bf`高度；相对header/事件/目的地不换。原生pointer marker派生zoom后移动24逻辑像素，实际delta23.9998845；destination仍320宽，height376。fresh原生viewer打开/关闭，主区与弹层右/底边在0.015625像素内；没有把此Penpot局部边界比较当引擎双端≤1DIP。

[原生动作](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/changes.json)、[字体实际测量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/font-probe.json)、[弹层回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/prototype-anchor.json)及[13图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-source-mode-23/visual-review.json)记录保存/关闭/新context重开rev138。指针关闭后activeElement=BODY，原生键盘focus/history未验证。Noto positionData ink19与line16、其它混合文字provider仍open；本段不替代正式源导出或真实生产publisher。

## AB-D15：原来源版本失效的三宽设计

## AB-D16：来源树与搜索实例的实际内部宽度

沿原AB-D11/12三张已有内容板原生编辑，不另造按钮/树/主题。Actor保存关闭/新context重开rev125→130，1088对象不增减；使用的Shared文件仍rev5且data前后精确一致。

| 尺寸 | 原board UUID | 本次设计消费 |
| --- | --- | --- |
| 1280×800 | 8d782253-7df4-44be-903e-c04d1bc25211 | 保留三列来源/构成/Details；Search与来源树内部跟随各自copy宽度。 |
| 900×620 | 7634458e-b3cb-8050-8008-d73fc3335cd9 | 保留compact详情入口和原overlay；正文/来源记录未改。 |
| 640×520 | 7634458e-b3cb-8050-8008-d73fbb4570ba | 同上；mode里的场景实例短语仍跨行，原问题保留。 |

真实UE参考`dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:420`的`SSubobject_RowWidget::GenerateWidgetForColumn`：图标/展开符为AutoWidth，名称为FillWidth(1.0f)，名称有只读及Ellipsis合同；`:1896`附近的树绑定选择、子项、上下文菜单。这里仍是Prefab来源实体unit设计，不把它称作已实现UE类/继承/组件attachment模型。

Search原copy254而surface240/caption204，本次native Inspector改surface254、caption218，并选左右固定；leading图标不动。六个TreeRow原标签128，本次无场景动作时按`copy.width - text.x52 - tree_right_inset12`得父行190、深度wrapper内子行178。12来自现有retained row metrics/labels的内容variant规则；只处理控件内部painter，不按截图修普通内容坐标。原原始painter/metadata保持，不能因此声明qualified importer已接受这些覆盖。

来源未关联场景实例，eye/lock没有业务目标；原生Layers隐藏六行已有24个图形/槽，不替换或删除source parts。原AB-D11 unit映射为根42与子18446744073709551615，实际子行仍在原child-depth wrapper；只隐藏这三个已审阅子行的展开符。自动化的版面辨识仅用在该unit设计操作，未写入runtime resolver/target/handler；Rust仍仅按完整u64 ID查询并拒绝重复/缺失。

39个对象仅变化hidden/touched或width/points/selrect/constraintsH/touched，其余1049对象精确不变。690条shapeRef均解析，本域212/Shared478；所有1088对象的source/slot/events/字体/文本/父子与引用保持。旧640失败95节点和两个13节点Details overlay精确保持，未detach或复制main。[保存/重开身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/save-reopen-preservation.json)与[几何公式/实值](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/measurements.json)记录细节。

[12图逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-content-consumers-22/visual-review.json)含前/后/fresh各三宽及三个100%工作区原生矢量裁剪；最后三图真实像素分别1280×800、900×620、640×520，DPR1，边缘保留native选框。取消选择会切为原生thumbnail，两次selector失败完整保留；未注入DOM/CSS或缩放PNG，不把此图当官方exporter/Engine/150%DPI。

Shared/source-owner的宽度修复worker仍待资格化，Button内caption220、全内容中文provider/19ink与16linebox、搜索/多选/折叠/键盘/焦点/首尾滚动及生产发布/事务继续open。沿AB-D01的app-shell/file-manager/empty-state映射，不引入新参考主题。passed/accepted=false。

设计19从本域三张NotResident板原生Ctrl+D创建，不复制main或detach。保存关闭、新context重开及末次只读核对均rev125/1088。真实新板如下，均为unit状态设计，不是生产来源或完整窗口验收。

| 尺寸 | board UUID |
| --- | --- |
| 1280×800 | 5af79cea-4b1f-8012-8008-d7d3386792c8 |
| 900×620 | 5af79cea-4b1f-8012-8008-d7d45b64c5ac |
| 640×520 | 5af79cea-4b1f-8012-8008-d7d4b65b3a08 |

状态标题为“来源版本已失效（只读）”，树显示数量未知，构成区提示重新选择，旧选择与Details清空。来源URI保留既有fixture文本，不注册producer；“应用/校验（不可用）”仍只是设计意图，不能算handler或真实禁用验证。真实`ResourceManager::snapshot_expected`在锁内拒绝不存在或不匹配的旧native key，Stale并不说明特定根因；因此六处初版“已变更”文字已原生改为“已失效/旧版本不可用”。实际调用方仍须核project/document/view/incarnation/selection/request，并在发布前重校验，不能只保留旧行。

UE依据继续为`SSubobjectEditor::UpdateTree`及`RestoreSelectionState`：先移除旧UI选择，再恢复仍有效的handle，并非一律丢弃所有选择；`BlueprintEditor::OnBlueprintChangedImpl`/`OnBlueprintCompiled`提供刷新和缓存失效参考。Zircon本状态面向旧来源证明已拒绝的情况，尚未实现完整UE选择恢复或Blueprint执行。沿AB-D01的app-shell/file-manager/empty-state结构参考，不引入新主题。

fresh122揭示900标题Y908与动作Y917交叠，原生Layers重排既有mode至根column后变为title908/action932/mode972/main996，正常gap8；首次dragTo无效证据保留。1280来源副本第三行越出33高根，原生Inspector仅将自有copy垂直fixed改auto，重算48，Details随flow后移至Y160。640初版瞬时交叠在fresh中消失，原因未定，未用坐标修正。最终三宽及所有中间失败图见[20图逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/visual-review.json)。

[保存/重开身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-expected-source-stale-19/save-reopen-preservation.json)同时保留初始122和最终125：原992非根对象不变，95新对象、67 Shared根、690引用有效；20处文字原生选择source custom Noto400并由实际SVG字体探针确认。字形约19高与源行框16的差异仍存在，不做baseline补偿或1DIP声明。主件内部allocation、其它中文/700、真实搜索/焦点/首尾滚动、生产失效处理、官方导出和normal EXE/DPI仍open。

## AB-D14：六板来源说明的明确字体消费

沿AB-D13原六板，不新增板或公共组件。三种尺寸的Ready-empty/NotResident既有mode leaf统一为“来源定义：尚未选中实体”；文本全部在原editor-ui.font.toml的Noto Han/全宽标点覆盖范围。原生菜单选择custom `custom-2ddd793a-5f89-40b0-aec1-fbe879e37a57` / `normal-400`，13.333333/400及原shared颜色保持；没有套用Shared Google typography或改master。

[设计18](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-local-cjk-status-18/result.json)保存关闭/新context重开rev112，993对象中仅6leaf变化，其余几何/flow/parent/source/slot/event/touched与linked引用保持，Shared仍5。18张前/后/fresh均已打开，六fresh实际SVG各11glyph为custom NotoSansSC-Regular；其它中文及500/700不在本次结论范围。

mode shape仍16高，positionData字形盒19高；SVG getBBox随画布缩放有差异，保留[测量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-local-cjk-status-18/measurements.json)，不靠baseline/坐标偏移补偿。仍是unit设计，功能按钮/搜索/焦点/滚动、生产source与官方ZUI/EXE待验。继续复用AB-D01的UE Toolkit/SSCSEditor及既有app-shell/file-manager/empty-state映射。

## AB-D13：来源为空与内容不可用的实际控件设计

本轮接续AB-D11/12的原生可编辑工作区，在分配的Actor file/page新增两种状态各三尺寸。状态依据为原authoring.rs::prefab_source_snapshot及tests::source_asset：resident空PrefabAsset的scene.entities为空；metadata存在但active native key缺失返回NotResident。Reference、wrong kind和SnapshotRejected仍为不同诊断，不能全部改为“未加载”。这是静态定义推导的unit状态设计，16项Rust定义仍0执行，不当作真实生产数据。空Prefab来源也不等于UE Actor/Blueprint的必需根或继承规则。

| 来源状态 | 1280×800 board UUID | 900×620 board UUID | 640×520 board UUID |
| --- | --- | --- | --- |
| Ready-empty | 483945c7-7976-80c8-8008-d749816e0a5c | 483945c7-7976-80c8-8008-d7498c295190 | 483945c7-7976-80c8-8008-d74994b7f95d |
| NotResident | 483945c7-7976-80c8-8008-d74986b31964 | 483945c7-7976-80c8-8008-d7499085d4f8 | 483945c7-7976-80c8-8008-d749999e41bc |

六板从自己的已存在1280内容板及900/640 compact板原生Ctrl+D创建。原生Delete移除新板中的实体TreeRow、组件记录、已选Details和紧凑“详情”入口，避免空态再打开旧有数据；原板/弹层不改。Ready-empty显示“0个实体/实体0·组件0”，NotResident使用“数量未知”，都没有可编辑选中实体。wide保留来源URI的SourceReferenceData实例与状态覆盖；compact不引入假Load/Retry/Graph按钮。

每板保留原Shared两Button+Search实例，新增18个外链根；wide另有1个本域SourceReferenceData副本。所有component/file/main/shapeRef与native touched/文字覆盖见[保存/重开身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/save-reopen-preservation.json)，不detach、不复制main、不改Shared/时间戳/字体provider。“应用/校验（不可用）”仅表达设计意图，没有已接通或真实禁用的运行时handler，Search亦不计完成交互。

根column、主体row、tree270及center fill沿原flow；wide Details320。空树提示置入hierarchy-rows、构成提示置入component-records，用原生Inspector justify-center。初版异步flow失效导致位置陈旧，compact空header收缩到0.01高；MinH32尝试无效。最终只在新compact板移除多余空wrapper，将标题直接放入根column，无普通内容坐标偏移。最终center宽640=346、900=606，标题与action strip无交叠；640 mode文字拆“实体”为两行，可见但换行润色仍open。

保存关闭/新认证context及独立API均rev107，data精确一致；993对象/2本域main/58Shared根，653引用全部解析，原802非根对象/顶层相对顺序精确保留。Shared rev5 data不变。[18张原图逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/visual-review.json)包含12张初版/中间图，名字canvas-final的中间图不计最终通过；最后6张fresh显示修正结果，累计105图。测量仅为Penpot坐标，不是实际DPI或双端1DIP；浏览器图不是正式exporter/EXE证据。

真实per-view producer仍缺：当前source snapshot无context、batch按template聚合，不能拿任一global active asset替代view/document。由Parent在真实生命周期注入同项目registry/resources及document/view/incarnation/selection/source/request，发布前重校验原native key；内容不可用应同时失效旧树、Details和选择。本域不扩公共trait/时钟/store、不注册fixture producer。AB-C02及AB-V1～3仅增加设计证据，正式原ZUI导出、真实动作/Undo/focus、normal EXE/AB-V4/双端仍pending。


## AB-D12：消费既有compact Details合同

原资产 zircon_editor/assets/ui/editor/world/extension_prefab_editor_workspace.zui 已声明right/right_reserve的 responsive_min_tier="wide"，right另有 responsive_compact_drawer="module_details"。现有responsive_layout、toolbar_layout、componentized_window及reference_menu_actions已提供可见性/开关/route路径；实际narrow≤640、wide≥1260，900属于regular。本轮沿此作Zircon窄窗适配，UE Unified三列目标保留，不称UE默认布局。原31节点ZUI未改，正常Actor host的真实toggle控件/producer仍未证明。

| 实际闭合board | Details board | 尺寸与实际布局 |
| --- | --- | --- |
| 7634458e-b3cb-8050-8008-d73fbb4570ba | 7634458e-b3cb-8050-8008-d73fbb4570bf | 640×520；tree270/center346；Details320×400，主板相对312,88。 |
| 7634458e-b3cb-8050-8008-d73fc3335cd9 | 7634458e-b3cb-8050-8008-d73fc3335cde | 900×620；tree270/center606；Details320×500，主板相对572,88。 |

原生Ctrl+D复制本域内容板，所有来源文字/数据引用保留，再将原Details子树移为可编辑弹层。普通header/主体/卡片/来源状态仍native flex。新Shared Button实例为 e01d87e6-0c76-8063-8008-d7410589d576 / e01d87e6-0c76-8063-8008-d741296f86ba，引用原组件6ddc248b-bd35-80c5-8008-d6c8c01096f3、main6ddc248b-bd35-80c5-8008-d6c8bd9b057f、Shared rev5；仅自有“详情”文字/prototype interaction覆盖。每组98对象/10原生实例（6 Shared+4本域数据），无新main或业务handler；原606非根对象/2main/顶层相对顺序和来源文字引用全同。

原生SVG实录关闭→同一开关展开→外部点击保持展开→同一开关闭合，返回图与默认图SHA相同。自动top-right相对内层frame实测落在画板顶边遮住开关；manual初版错用父header原点，横向多384，两失败图保留。实际cfh/get-frame对frame返回自身，最终只把真实弹层定位按已flow的main目标和按钮frame原点计算，普通内容无坐标偏移。每板固定prototype位置是设计验证；未来尺寸变化须宿主重算，不是生产自适应已接通。

[14图逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/visual-review.json)及[完整身份/实例/锚点回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/result.json)记录保存关闭/新context重开rev94、Actor与Shared data全同。测试样例u64、两条同类型opaque记录、中文mesh及unknown保留；未注册fixture producer或用source_index作target。

“详情”为设计代理，未绑引擎操作。SVG g无role/tabindex，前后BODY不是focus返回；Escape/Tab/大数据首尾scroll仍pending。SVG字形呈serif样外观，与画布不同；两context报Roboto400/700 loaded仍不能证明实际provider/glyph/CJK/行框相同，不做1DIP或A/B结论。TreeRow灰底/叶disclosure/action、真正Dock/Tab、合法value、生产caller/写事务/Graph、正式导出/normal EXE/DPI仍pending，passed/accepted=false。

本文件保留UE目标与各时点试验。最新AB-D12为紧凑Details原生设计/点击原型，Actor rev94/803对象、40 Shared roots、579有效shapeRef；640/900只读fixture可读，旧AB-D11三栏640失败完整保留。尚无完整Toolkit、生产预览、正式导出或Engine验收。

## AB-D11：来源工作区内容与640失败边界

在原file/page新增来源文档工作区，沿原31节点ZUI的树、构成/来源状态、Details和Apply/Validate语义组合。数据来自08的 prefab_source_rows_borrow_the_resident_payload_without_losing_identity_or_components 测试：两个同名实体，来源根42、未激活子实体18446744073709551615，两条同类型opaque组件记录及中文mesh引用。标题与board名明确是unit fixture；不注册固定fixture producer，不按文案或source_index派发，不从来源ID推导场景实例ID。每板使用5个Shared native copies（2 Button、Search、2 TreeRow）和4个原本域SourceReferenceData copies；没有复制main、detach或填入非法Property.value子件。

| 实际board UUID | 逻辑尺寸 | 实际复核 |
| --- | --- | --- |
| 8d782253-7df4-44be-903e-c04d1bc25211 | 1280×800 | [fresh图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-prefab-workspace-11/fresh-reopened-1280.png)：树父子缩进、选中状态、完整u64字符串、两条独立组件记录和来源Details可读。只读数据设计，不是Viewport/Graph或完整Toolkit。 |
| 3a7bfe6d-c630-80c5-8008-d73a0703ddb6 | 900×620 | [fresh图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-prefab-workspace-11/fresh-reopened-900.png)：保留270/320两侧列，中心278；类型长文本换行后卡片增长，来源状态仍在可视区。 |
| 3a7bfe6d-c630-80c5-8008-d73a803f37a2 | 640×520 | [fresh失败图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-prefab-workspace-11/fresh-reopened-640.png)：中心仅18宽，文字竖排并溢出；native整组render extent高1806，源viewport仍520。失败保留，不缩图、隐藏内容或补装饰切页冒充可达。 |

27个初始流框/文字加1个child-depth wrapper均为本域普通内容；区域使用native flex，深度1的12单位padding来自retained tree_depth投影，不是截图坐标修补。原生Assets拖放创建实例，落入上级pane的copy仅以官方typed Transit UUID move改到本域语义slot；source file/component/shapeRef保留，无skipValidate或allow-altering-copies绕过。17处文字覆盖自动登记native touched。900/640由本域完整板原生Ctrl+D复制并以原生尺寸Inspector修改；首次900读取早于异步保存，失败保留，随后精确900×620读回成立。

新增7张原图全部已打开，逐图见[视觉记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-prefab-workspace-11/visual-review.json)。原context的两张1280图仍有青色Details展示轮廓，不能据clean文件名当无覆盖图；fresh context图已无该轮廓。native frame缩略缓存可能软化字形，browser截图不是官方PNG导出或1DIP证据。浏览器2050×1250/DPR1，clip取实际source geometry与native SVG矩阵，三个viewport分别1280×800、900×620、640×520；没有Windows144DPI或Engine截图。

保存前后Actor data全同、rev78不变。相对rev49，原321非根对象和2业务component记录精确不变，新增285对象/201个有效shapeRef；441条全部解析。Shared data前后全同，原body、local-color leaf和其它板未改。完整身份、每板95对象/9实例、source检查、失败与退出见[内容11](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-prefab-workspace-11/result.json)及[重开API](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-prefab-workspace-11/after-reopen.json)。

Apply/Validate文字明确不可用，未接操作handler；灰色TreeRow动作槽、叶行disclosure和组件内部宽度仍交公共owner，不能算业务命令完成。640建议消费既有Dock/Tab或受支持的垂直滚动组合，并证明进入/返回Details、来源记录首尾及focus；本域不造平行公共组件。生产per-view caller、原key及上下文重校验、稳定写target/事务/undo、类/事件图后端、source-owner正式ZUI导出和normal Actor case均pending，passed/accepted=false。

## AB-D10：真实Button发布的原生消费及边界

本轮没有改Shared、日期或syncedAt来制造提示。新API实读Button组件`6ddc248b-bd35-80c5-8008-d6c8c01096f3` / main `6ddc248b-bd35-80c5-8008-d6c8bd9b057f`根已有`editor.surface.1`颜色`599586e3-022c-500b-ac80-5d9c490139af`，颜色source metadata指向原editor_tokens。原生Update直接同步而未打开modal；Actor rev48→49，随后关闭context、新建context并独立API读回，322对象与同步后全同，Shared data前后全同。4个Button根取得正确Shared style pair，实际#242424未变；不称全局视觉A/B或完整style/export链通过。

240个linked对象、13个外链根、2个业务main、文字、source/slot/event/touched及覆盖均保留。原body与独立local-color leaf全对象精确不变；其它103对象有native精度/文本positionData变化，最大数值差0.3800048828125，28个path的命令序列/参数数量保持。完整差异在[同步10](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-sync-10/result.json)及[新context API](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-sync-10/after-reopen.json)，不能把数值差当Penpot与EXE的1DIP比较。

| 实际新context原图 | 逐图复核 | 未完成范围 |
| --- | --- | --- |
| [640](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-sync-10/final-reopened-640.png) | 640×520板、Button/Search/Tree中文内容保留；树标题两行。 | Button根拉伸但label仍在原240宽内居中，Search仍240宽；稀疏consumer板，非完整窄窗策略。 |
| [900](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-sync-10/final-reopened-900.png) | 900×620板与同一内容保留。 | 同一内部宽度及树换行问题；未覆盖完整工作区/焦点/滚动。 |
| [1280](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-sync-10/final-reopened-1280.png) | 1280×800板，71%画布缩放下可见相同label偏置。 | 大片consumer空区仍在；不是UE窗口完成或实际DPI图。 |

5张新增原图均已打开，累计66张。Roboto400/700当前loaded，500 unloaded；没有盲套Typography，不把family/loaded当provider/CJK/Engine就绪。完整Prefab内容仍按AB-D01/02/03及原31节点ZUI身份接续；Property合法value子件、公共内部flow/正式导出/正常EXE各自pending，不为绕过它们复制公共控件。

## 当前源核对与18:11历史边界

已实读[Parent r04及20:13计数补正](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/actor-shared-style-interruption-r04-reconciliation.json)，并用新隔离context只选择、查看两个不同目标，没有编辑、选择Typography或点击Update。当前原body field0 `9fffda54-1c42-5c4f-8e69-bb03997f533d` 仍属于SourceReferenceData component `8ff62384-7110-5d59-bd61-64732823f235` / main shape `5b9092d3-1ecd-5d74-999f-69b8e082d26f`，文字“Scene instance / source fixture”、custom字体/normal-400/13.333333不变，没有Typography引用。当前状态不能补签18:11 saved，也不能证明从未出现暂存后恢复；旧Inspector、服务中断、字体error和关闭回执保留原字节。

Shared body `99eb86f9-12ff-5935-96ad-3d1a985e61f0` 使用`gfont-roboto/regular`，原body使用`custom-495ff285-7c71-49bc-bc4e-df20017e2d66/normal-400`。同Roboto家族名、400和13.333333不证明provider URL、字体字节、variant或度量等价。本次body视图中Roboto400为loaded，700为unloaded；随后Button视图中700为loaded。unloaded不作error，新状态不覆盖18:11的error，也不构成Shared gfont、CJK fallback或Engine字体ready。

| 本轮原图 | 当前观察 |
| --- | --- |
| [原body精确目标](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-compare-07/fresh-original-body-source.png) | 原main field0选中，Inspector为Roboto Regular、字面量颜色，当前Latin文字为sans外观；画布241%，非实际DPI或完整窗口。 |
| [另一Button颜色目标](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-compare-07/fresh-shared-button-color-source.png) | “添加组件”及editor.text.primary保留，2处authoring颜色引用+1处positionData缓存仍属同一leaf；没有Typography、main默认传播或业务动作证明。 |

两图均已打开逐图复核，累计61张browser原图。API前后仍rev48，322对象和components全同；Shared rev2对象/components也全同。按最近祖先componentFile核验240个真实对象UUID，120引用本域main、120引用Shared，全部位于对应main子树；13是Shared copy根数，不能与位置数或ID/file字段数混用。完整[当前源对照](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-compare-07/result.json)保留两目标、字体声明、逐对象解析及历史表述补正。新浏览器正常exit0，9个出生身份退出、临时文件0，自动回收6,932,938B逻辑样本/物理未测；旧126B仍待受管清理，无服务或桌面操作。

## Shared颜色：受控leaf的原生覆盖与重开

按[Parent第六次交接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/parent-01a16d4e/actor-blueprint-followup-06.json)，先重新认证读取rev45，确认322对象与shared-style-04基线全部一致，且此次读回未见body引用；18:11保存仍未确认，不能推定从未暂存。保留原失败记录，不重新套Typography。此轮仅在原板`89eb3427-27ba-801b-8008-d6cfcabb7a3c`的Button copy `89eb3427-27ba-801b-8008-d6d09c3492b9`中，给原文字leaf `89eb3427-27ba-801b-8008-d6d09c3492ba`选择Shared Assets的`editor.text.primary`，真实color UUID为`e022fcac-39df-51eb-8515-4723f9932060`。

独立读回：保存rev46仅此leaf变化，原生Ctrl+Z得到rev47并恢复全部322个基线对象，Ctrl+Shift+Z得到rev48并与保存后322对象完全一致；关闭/全新context重开仍为rev48，引用保留。新增的是该leaf的paragraph/run/positionData三处同一颜色引用和`text-content-attribute`覆盖组。其余321对象、components、13copy根、240条linked身份、中文“添加组件”、原custom Roboto/700/13.333333、几何、源元数据和shapeRef `6ddc248b-bd35-80c5-8008-d6c8bdb3e2f8`全部保持；剔除新增引用及该覆盖组后leaf也与基线相同。

颜色原值与样式值均为`#c0c0c0`，所以外观不变。这只证明原生局部颜色覆盖的持久化/撤销，不是全局样式A/B、Prefab业务undo或ZUI/产品生效。Shared对象/components/34+3样式及library metadata均与基线完全一致，syncedAt仍13:50:51.913694Z。重开图自然出现“共享库有更新”；新引用颜色的modifiedAt为17:37:45.684Z，与used-style过滤条件相符，未点击更新或改时间戳。仍待public owner交付真正main变更后，才消费影响到的组件并逐一核对覆盖。

| 本轮原图 | 实际复核与限制 |
| --- | --- |
| [选择前受控板](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-leaf-style-05/controlled-board-before.png) | 原四件和中文保留；不是完整Actor窗口，Property value仍空。 |
| [原生颜色覆盖](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-leaf-style-05/leaf-color-local-override.png) | Shared Assets与Inspector显示真实style名称；相同色值无视觉delta。 |
| [原生撤销](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-leaf-style-05/leaf-color-undo.png) | Inspector恢复字面量c0c0c0；API证明322对象全部回到基线。 |
| [重做并关闭重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-leaf-style-05/fresh-reopened-leaf-color.png) | 相同leaf引用保留，出现真实共享库更新提示；未执行更新。 |

四图均已打开逐图复核，累计59张browser原图；1600×1100/DPR1、画布约93%，不等于实际150%DPI。新context的Roboto400/700字体面为loaded，原custom字体声明未变；不据此宣称中文glyph fallback、Engine字体或≤1DIP已通过。完整引用、基线、源token SHA与进程退出见[leaf-style-05](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-leaf-style-05/result.json)。浏览器exit0、9个出生身份退出、新临时文件0；浏览器自动释放11,131,825B逻辑样本，物理量未测、手工删除0B，旧126B受管清理仍open。

## Shared rev2：上一轮原生同步、字体与catalog消费核查

独立API读回Shared `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` rev2/isShared=true/41对象、原四main UUID不变、34 colors和3 typographies。正文 `99eb86f9-12ff-5935-96ad-3d1a985e61f0`、说明 `c0224e78-f918-526f-8026-a76cd8f347af`、强调 `9ac86733-8214-5a04-820a-9074b5c68310` 均为真实发布身份。完整本轮回执见 [shared-style-04](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-style-04/result.json)，旧rev1/style0段落按其观察时刻保留。

原生“管理库→更新”实际显示“没有需要更新的共享库”。四组件modifiedAt仍为13:21:31.125Z，早于本域linked library syncedAt 13:50:51.913694Z；Shared文件modifiedAt已为17:37:45.683436Z。Penpot的`assets-need-sync`经`used-assets-changed-since`按已使用组件的modifiedAt过滤，旧copy尚无新style引用；需原public owner接续正式组件变更发布。未unlink、detach、重置原覆盖或直接复制main；尚未证明13个外部copies同步成功。

在本域SourceReferenceData主件field0 `9fffda54-1c42-5c4f-8e69-bb03997f533d` 通过原生素材点击body后，Inspector显示真实body样式。随后独立get-file和一次首页readiness均WinError10061拒绝连接，故只记为页面内选择，未记保存或新revision。Roboto 400/700字体面为error，正文及中文font check均false；可见fallback文字不构成字体ready或1DIP测量。服务恢复后须先独立读取该shape的真实style ref再决定是否重做，不能假定已保存或先清覆盖。

| 本轮原图 | 逐图复核 | 未通过范围 |
| --- | --- | --- |
| [原生更新页](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-style-04/native-no-updates-rev2.png) | 明确显示无需更新，无更新操作入口。 | 跨文件实例同步 |
| [业务main选择前](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-style-04/business-style-assets-before.png) | 共享三种排版可见，原Roboto声明实际出现serif fallback。 | 字体资源与中文测量 |
| [body选择后、保存未确认](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-style-04/business-body-unconfirmed-save.png) | 原生Inspector出现body引用名称，fallback未消失。 | 持久化、关闭重开、ZUI与EXE传播 |

catalog新SHA `4c2f849c533f1f227903018eca860faadd7fa3138fdc265300913e3833ad36e5` 的NumberField共10props、PropertyEditorRow共4slots已独立结构核对；原source早已具备，不代表新增Penpot master或测试通过。Prefab override实际存储serde_json::Value，当前适配只读借用；42、0..100、precision0不能当业务值/schema。真实数值范围、step、精度及写authority未到达前保持只读，i64/u64不经f32往返。具体槽/参数消费见共享合同本轮段落。无公共源码、Rust、原31节点ZUI或服务改动。

范围与能力见 [工作记录](actor-blueprint.md)，适用性见 [盘点](actor-blueprint-window-inventory.md)，状态矩阵沿用 [差异与场景](actor-blueprint-differences.md)。UE 源码前缀 `UE/` 为 `dev/UnrealEngine/Engine/Source/`，只读。

## AB-D09：实际来源查询合同的可编辑状态设计

本轮新增三张 `AB-SOURCE-03` 板，只设计已实现 `prefab_source_snapshot` 的结果显示语义，不是当前项目数据、可操作完整窗口或后端已接通的证明。来源可读、引用未解析、尚未驻留、目录类型不符、版本失效、载荷类型不符六组内容分别复用原业务 `SourceReferenceData` component `8ff62384-7110-5d59-bd61-64732823f235` / main `5b9092d3-1ecd-5d74-999f-69b8e082d26f`。每板6个native copies/18个子shapeRef，文字采用合法content-group覆盖；保留原320×48业务件和column flow、24内边距/12行距，没有复制main、detach、另造按钮/树/主题或回避公共宽度缺陷。

| 板尺寸 | 真实board UUID | 最终重开原图与逐图复核 |
| --- | --- | --- |
| 640×520 | `1514bd85-ad89-4263-b755-db1e85ee5c54` | [640-final-reopen.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-state-03/640-final-reopen.png)：标题/范围提示在上，六状态及中文完整，无板内裁切；不是原完整工作区或空数据EXE状态 |
| 900×620 | `0589f584-db29-8000-8008-d6f89659e87f` | [900-final-reopen.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-state-03/900-final-reopen.png)：原生宽高已读回，流式文字与覆盖保留，右侧留空；这是状态规格排版，不计完整窗口设计 |
| 1280×800 | `0589f584-db29-8000-8008-d6f8c8b3be19` | [1280-final-reopen.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-state-03/1280-final-reopen.png)：同六状态完整，画布约71%适配，文字随画布缩放；非150%DPI、产品预览或≤1DIP对比 |

保存前后独立API均为rev45/322对象，对象哈希全部相同；原240个非根对象、components记录和顶层相对顺序保留，Shared仍rev1/4mains。组件/子shape UUID、覆盖、几何、API回执与退出见 [source-state-03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-state-03/result.json)。原始失败亦保留并逐图查看：素材mouse drag未创建、严格SVG locator/错误viewport locator、普通JSON重排无效；Transit UUID重排后经原生Row gap 12→13→12触发真实flow，未手动搬子节点。

| 过程原图 | 实际观察与处理 |
| --- | --- |
| [640-first-reopen.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-state-03/640-first-reopen.png) | 内容已保存但flow顺序反向，标题在底；未作为最终设计 |
| [640-native-reflow.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-state-03/640-native-reflow.png) | 首次按动态input ID触发无有效重排且选择不符；独立哈希确认未改变任何原非根对象，改用语义Row gap控件 |
| [640-order-reflow.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-state-03/640-order-reflow.png) | Transit带UUID重排及原生flow重算后，标题/范围提示与六状态顺序正确；随后保存并新context重开验证 |

新增6张browser原图均实际打开，累计52张。全部DPR1，浏览器viewport1600×1100；画板尺寸不等于EXE client size。资源就绪/native捕获/双端逐图门均未满足，passed/accepted仍false。

## AB-D01：模式与停靠结构

外层菜单、工具栏、资产文档页签、Dock 与状态栏归 Parent。资产标题显示真实文档名称和 dirty 状态；相同资产 URI 的重开遵循共享 DocumentHost 合同，不能每次新建互不关联的静态工作区。本族提供实际文档内容和命令上下文。

| 模式 | 实际 UE 布局身份与源码 | 设计区域及默认 tab 状态 |
| --- | --- | --- |
| Actor class unified，具备组件能力 | `UE/Editor/Kismet/Private/BlueprintEditorModes.cpp:517`；`Blueprints_Unified_Components_v7` | 左/中/右 splitter 权重 0.15/0.60/0.25。左列 Components/My Blueprint 为 0.30/0.70；中列上部 Viewport 开启、Document 初始关闭，下部 Compiler Results/Find Results/Bookmarks 初始关闭，权重 0.80/0.20；右列 Details 开启、Palette 关闭。 |
| 无组件能力的 unified / 纯库候选 | 同文件 `Blueprints_Unified_v5` 分支；`BlueprintEditor.cpp:4848` `CanAccessComponentsMode` | 左 My Blueprint、中 Document 与下部结果、右 Details/Palette；不创建 Components 或 Actor Viewport。是否进入此模式由真实资产能力决定，不能根据文件名含 Blueprint 推断。 |
| 独立 Components mode | 同文件 `Standalone_BlueprintComponents_Layout_v6`，约 `:203` 起 | 横向 0.15/0.85；左列 Components/Details 为 0.35/0.65；右侧大 Viewport。与 unified 的右 Details 三列模式分别制作状态，不能混成一个截图。 |
| Class Defaults / data-only | 同文件 `:164`，`Standalone_BlueprintDefaults_Layout_v7` | 主内容为 DefaultEditor，tab well 隐藏；不是“主场景实例 Details 改标题”。真实类默认值为数据 owner，需区分多个编辑对象的混合值。 |
| Interface | 同文件 `:338`，`Standalone_BlueprintInterface_Layout_v4` | 左侧主图文档/结果，右侧 My Blueprint 与 Details；源码显式 splitter 权重为 0.70 与 0.15，不写成占整窗 70%/15%。没有 Actor Components/Viewport；函数签名与图可编辑性由实际 schema 决定。 |
| Macro | 同文件 `:434`，`Standalone_BlueprintMacro_Layout_v4` | 主文档及下方 Find/Bookmarks，右侧 My Blueprint 与 Details/Palette；源码没有为该 layout 增加 Compiler Results tab，不从 unified 复制无依据的底部面板。 |
| Level script | `UE/Editor/LevelEditor/Private/LevelEditorActions.cpp:2951`、`BlueprintEditor.cpp:1907,4089,4364` | 从真实关卡入口打开与该关卡绑定的文档；按 capability 排除 Actor 组件操作。运行、保存、卸载/重载和 Actor 引用失效归关卡脚本合同，当前仍待定。 |

`ConstructionScriptEditor` 是组件树 tab；`SCSViewport` 是组件预览；`GraphEditor`/`Document` 承载图文档。实际字符串常量在 `UE/Editor/Kismet/Private/BlueprintEditorTabs.cpp`：Details 对应 `Inspector`，Palette 对应 `PaletteList`，Bookmarks 对应 `BookmarkList`。这些是 UE 参考身份，不能直接假造为 Zircon 已注册 tab ID。

拟在已分配 page `b4a1e3ce-9e6a-81fe-8008-d69d60f73bf2` 分别展示上述模式及其主要状态，板名可读，board UUID 由真实 Penpot 返回。关闭的 panel 保留 Window 菜单重开和布局恢复；模式切换保留文档/选择/视图状态，依据 mode 的 `PreDeactivateMode` / `PostActivateMode`。布局权重只作用于扣除宿主 chrome 与 splitter 后的内容区，不能拿 client size 直接算绝对控件坐标。

## AB-D02：区域流、滚动与文字

| 区域 | 布局与数据 | 滚动、截断与焦点 |
| --- | --- | --- |
| Components | VerticalBox：共享 panel header 与 Add/action 区、过滤输入、树内容。行按真实组件层级投影；标识类继承、实例新增、override、只读/不可删及实际 root | 树独占一个有界虚拟滚动 owner；只复用可见行，itemKey 不能用显示名称或当前索引。展开状态按文档与稳定 item identity 保存；标签 start 对齐，ellipsis 配完整值提示。 |
| My Blueprint | 共享 header/创建菜单、搜索、按真实成员分类的树/列表：变量、函数、宏、图、分发器等 | 一个有界集合滚动 owner；搜索空结果与资产无成员分开。成员操作按资产能力过滤，不给 Interface/纯库补虚假的 Actor 成员。 |
| Details / Defaults | 分类与属性行按流排列；label 与 value 使用共享 row/slot，值编辑类型来自反射 schema，reset/override 按真实默认值比较 | 每个独立 Details panel 一个滚动 owner；外层文档不再套同向滚动。编辑值保持完整，窄窗按共享属性行合同收缩/换行，不截断实际提交值。 |
| Graph document | 文档内 toolbar/breadcrumb 使用 flow；节点/引脚/连线是实际 diagram，可使用图坐标。坐标由真实图模型和 view transform 所有 | 图的平移/缩放不是页面滚动。连接、框选、右键菜单、拖线预览由共用 graph owner 处理；overlay 以 pointer/pin 为锚，受窗口 clipping、焦点恢复和取消约束。 |
| Viewport | Dock body 的 stretch 内容，camera、selection、gizmo 与实际预览 world/renderer 关联 | 仅真实渲染宿主消费视口输入；不在外层 overlay 用坐标命中模拟操作。无可渲染组件、加载中和失败由真实 readiness/capability 表示。 |
| Compiler / Find / Bookmarks | 独立可停靠结果 tab，header/filter/action 与行集合使用 flow；日志级别和位置/节点跳转分别为交互项 | 各活动 tab 只有一个滚动 owner。错误行使用可定位的 document/node/pin identity；用户离开日志尾部后遵循共享 follow/pause 合同，不能抢回滚动。 |

普通内容不新增 CanvasBox、定位偏移或截图补丁。树缩进、行高、图标、字体、focus、separator 使用共享 token 和既有控件合同；不另填猜测的像素值。`WorkbenchTreeRow` 当前已提供 `tree_depth/expanded/selected/focused/hovered/pressed` 和选中/非活动选中配色，不能为本族再建一套默认状态主题。

固定界面文字按项目 `UiLocalizedTextRef`/locale catalog 合同接入。资产、组件、成员、错误内容和用户输入由 binding/state 供给；中文与长名称是实际可序列化内容，不用默认 PF_Chest 等例子作为生产 fallback。普通标签 start/top 对齐；仅数值统计等确需比较的值使用 end 对齐。

## AB-D03：类、实例、选择与事务

| 上下文 | 目标身份与显示内容 | 命令及撤销边界 |
| --- | --- | --- |
| 类/Prefab 文档 | 真实 asset URI、document identity、组件模板或实际 Prefab 内容、类默认值；继承来源有可定位身份 | 由本资产文档的 journal/history 所有；不直接拿场景 AddComponentScope 操作类资产。保存/重开要证明资产内容变化，而不只是树重新排版。 |
| 场景实例 | project/world/domain、node/entity identity、组件 schema/provider/instance identity、默认来源与 override | 复用已有 scene document 事务与 admission。保持 edit/play 世界区别；scene node parent 与 component attachment 不是同一种关系。 |
| 图选择 | document/graph/node/pin identity 与 selection generation，视图状态独立 | 节点删除/移动/连线属于当前图文档。编辑字段的文本撤销先归字段，提交后的业务撤销归资产；不能落入主场景。 |

Components 树、Details 与真实 Viewport 使用同一 selection 投影；focus 作为独立状态保留。鼠标/键盘多选、过滤与滚动不更换目标 owner。删除后选择合法相邻项或组件根；撤销恢复相同业务身份，不能按“第几行”重选。

Add 菜单基于真实 catalog：已有项、不可创建类型和运行状态给出真实可用性。提交时复核 document/gateway/selection/world/schema；预检拒绝不更新成功提示、dirty 或 Viewport。核心执行若返回“已应用后网关失败”，UI须按实际命令回执重读变化并显示错误，不能把所有Err都解释为零变更或自行补偿。现有 `inspector.component.add` 已有此类验证，本域仅在语义匹配的场景实例上下文复用。

Delete、Duplicate、Rename、Reparent、Detach、Make Root 都需真实命令和事务。UE `SSubobjectEditor.cpp:2769` 对整组选择检查 `CanDelete`，`:2807` 要求单项且 `CanRename`；类和实例写入分别在 `SSubobjectBlueprintEditor.cpp` 与 `SSubobjectInstanceEditor.cpp`。拖动先显示合法/非法目标及原因，落下时再验证当前身份。Detach 的参考实现是挂回当前 scene root，不能无依据变成“没有父级”。

UE 类组件删除对正在使用的 bound events 提供实际引用相关确认，见 `SSubobjectBlueprintEditor.cpp:333` 起。Zircon 只有在真实引用查询证明会破坏关联时才使用相应确认，不对每次普通删除增加猜测的审批。确认取消必须零变更；接受后也必须保留错误引用诊断或执行已定义的引用更新。

进一步能力对账如下。UE `UE/Editor/SubobjectDataInterface/Private/SubobjectData.cpp:154,218,235` 的 adapter/继承/root/child-actor 限制是命令可用性依据；Zircon 的相应 capability 必须来自自己的数据 owner，不把 UE 的 native C++ 标志直接套到 Rust provider。

| 接点 | 已确认源码事实 | 最小业务切片的要求 |
| --- | --- | --- |
| 类型与多目标 Add | `zircon_runtime/src/scene/world/dynamic_components/creation.rs` 的 spec 为 type_path/schema_generation/schema_json/value；catalog 只列 editor_visible 类型，模板只使用声明的默认值，缺必需字段由调用方提供；已存在任一目标时拒绝 Add | 在已有场景实例 Add 合同内验证零/单/多目标、真实必需字段、stale schema 和已存在组件；不把 Add 变成覆盖既有值的 setter。 |
| 实例证明与持久行 ID | `zircon_runtime/src/scene/world/typed_api/dynamic_component_presence.rs` 的 `DynamicComponentCreationId` 是 World 所有的 Arc 指针证明，序列化/重投影不保留 | 文档中的持久标识与运行时创建代际分别携带。undo 检查仍用真实 creation proof；树重开后的 itemKey 不能用该指针、显示名称或行序号。当前动态组件按 entity/type 存储，不能据 UE 同类多组件外推现有存储支持。 |
| 实际删除入口 | `zircon_runtime/src/scene/world/dynamic_components.rs:326` 的 `remove_dynamic_component` 已存在，处理实体缺失、组件缺席、presence/cache/generation 更新 | 原先“只有结构队列”的观察需补正；缺口是 Editor operation/journal、逆操作恢复、retention admission、schema/world/instance 复核和多目标原子性。不得把现有 Result<bool> 直接接按钮后宣称 Delete/Undo 完成。 |
| 已有 Add undo | `dynamic_components/creation.rs:268,305` 全批核对目标、完整值及creation receipt，再移除本命令创建且未变的组件；`core/editing/command/add_component.rs:366` 的 revert 已调用它 | 保留这项真实能力；任意既存组件的通用Editor Delete/journal/inverse/retention/多目标原子性仍归Parent核心层，不能说引擎全无撤销，也不能拿Add撤销冒充通用Delete。 |
| 命令结果与实际变化 | 同一Add命令的apply/revert区分 `SceneWriteCompletion::Completed` 与 `AppliedThenGatewayFailed`，后者返回 `applied(error)` | UI不把错误统一改写为零变更或成功；沿原命令/历史owner的完成与错误分类刷新实际树、Details和dirty。此为消费合同，不新建补偿事务或推断失败操作的历史终态。 |
| entity 与 component 层级 | `SceneEntityAsset` 在 `zircon_runtime/src/asset/assets/scene/entity.rs` 有 parent/transform；`SceneComponentAssetRecord` 和 creation spec 没有通用 attachment parent/root 字段 | provider payload 可能有自定义关系，但已读公共记录不构成 UE SceneComponent attachment 合同。先明确资产目标，不能直接用 entity reparent 冒充 component attach；Make Root 还涉及默认根/原根及变换处理，见 `UE/Editor/SubobjectDataInterface/Private/SubobjectDataSubsystem.cpp:1768`。 |
| 代码可见性与 owner | Editor 的 AddComponentScope/Factory/Command 当前为 crate 内部实现，Prefab editor 不能凭名字跨 crate 调用 | 复用或新增核心操作/public 合同先交 Parent/core owner；本域不在插件内复制一套 gateway/history/retention 验证。类/Prefab 写事务尚需独立文档数据与保存合同。 |

据此可以先验证已有 scene-instance Add；下一项新增删除命令应先由核心 owner 明确 operation/payload/journal/恢复与批处理合同，再接本域 UI。组件 attachment、类 defaults 和 Prefab Apply 属后续有明确模型依赖的切片。此顺序不减少用户要求，只把尚缺的合同保持可见；当前未新增或执行任何命令。

本轮AB-PF-16补齐真实行/Details/operation消费映射：实际document authority中的entity/完整type才参与行定位；creation proof、World与资源revision分别只作当前实例/失效验证。`entity://selected` 是执行时当前选择，`node://`也未携带文档/激活，均不能独立保存为跨文档目标。Make Root/Reparent在通用attachment模型和真实capability缺失时保持不可用；不序列化Arc、不新造UUID字段、不改公共trait或插件事务。当前源及原回执/closeout复核见 [target-contract-03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-target-contract-03.json)。

## AB-D04：快捷键、菜单与焦点返回

| 命令/触发 | UE 源码依据 | 设计作用域和返回规则 |
| --- | --- | --- |
| 组件 Cut/Copy/Paste/Duplicate/Delete/Rename | `UE/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:1926` 的 GenericCommands 映射、`:2518` `OnKeyDown` | 活动组件树且 capability 允许时执行；文本编辑框先处理自身输入。不把组件 Delete 转为 Actor Delete。具体 GenericCommands chord 由共享 command registry 对账，不另行猜测。 |
| Compile，F7 | `UE/Editor/Kismet/Private/BlueprintEditorCommands.cpp:58` | 当前可编译资产；状态来自真实 compiler request/revision。完成后错误结果可见但不随意抢走字段焦点；点击 error 再进入实际节点并聚焦。 |
| Find，Ctrl+F；Find All，Ctrl+Shift+F | 同文件 `:54` | 当前图/资产搜索和跨资产搜索分别有上下文。结果点击打开精确文档，返回时恢复此前 graph view/selection。 |
| Home；PageUp/Backspace；PageDown | 同文件 `:69` 起 | Graph 的选择适配、父图/子图导航；字段中的 Backspace 不导航。没有父/子图时不可执行，不伪造跳转。 |
| F9、Shift+Delete / Shift+Backspace | `UE/Editor/GraphEditor/Private/GraphEditorActions.cpp` 的 ToggleBreakpoint、DeleteAndReconnectNodes | 断点需要真实 VM/source map；删除并重连需要图 schema 的合法连接规则。后端缺失时保留目标设计与差异，不提供会显示成功的空操作。 |
| Add、树/图右键、引脚拖线菜单 | `SSubobjectEditor.cpp` 命令与拖动；`UE/Editor/GraphEditor/Private/SGraphPanel.cpp:1281,1882` | 打开时捕获 trigger 和业务上下文；Esc、取消或提交后返回该 trigger，若项已删除则回该文档的合法相邻项/树/图。禁止默认跳回主场景。 |
| 关闭/重开 panel、切换 mode/tab | `BlueprintEditorModes.cpp` 的 Save/RestoreEditedObjectState、component selection cache | Dock/focus 归共享宿主；本族保存真实文档/图/组件选择和视图状态。关闭的 tab 可经 Window 菜单重开；脏文档关闭按共享保存/取消合同处理。 |

每个弹层需要分别验证 keyboard open、mouse open、Esc、提交、点击外部、trigger 被删除/文档关闭六条返回路径。模态创建对话框关闭后返回原场景 trigger；若创建成功并打开资产，则按明确操作结果聚焦新资产 tab，不抢焦到另一个已开文档。

## AB-D05：构造、事件、接口与编译反馈

构造逻辑图、Event Graph、函数图、Interface、Macro/Function Library 分别由真实资产能力声明。左侧导航、Palette 与 context menu 的动作来自相同节点 schema/provider，保持名称、类型过滤与可执行性一致。typed pin 的拒绝、默认值与转换由图模型决定；不能只画线后显示“连接成功”。

已发现的 runtime 候选是 `script.bindings` 与 onStart/onUpdate/onFixedUpdate；它没有单独证明 construction callback、可编辑类/接口 schema、图到 module 的编译或 source map。预期接入边界是：本域图编辑命令 → 资产文档事务/保存 → 脚本 owner 的真实 compile request → diagnostics/source map → runtime module/实例刷新 → 正常 Viewport 可观测结果。此链的缺失段仍属 AB-004/005/006/009，不由设计文档补成已实现。

编译状态至少区分未编译/内容已变更、进行中、成功、警告、失败及结果已过期；具体状态枚举沿用实际 compiler service。请求绑定文档和 source revision，旧请求完成不得覆盖新修订的状态。Compile 不自动等于 Save，UE toolbar 的 Save on Compile 有独立策略；Zircon 是否提供该策略由现有保存合同与 Parent 统筹，不复制虚假选项。

## AB-D06：组件复用、原资产与共享提案

以下 document 相对 `zircon_editor/assets/`，身份来自实际catalog。Shared现已发布四个真实主件，本域已在受控板连接消费；UUID、source核对和限制见AB-D08末尾。其余公共控件与style/export合同仍未交付。本域两个业务数据主件不替代公共控件。

| 用途 | 既有组件 / document | 已有合同与本族缺口 |
| --- | --- | --- |
| 外层文档 | `UiHostWindow`、`DocumentHost` / `ui/editor/unreal/editor_shell.zui` | 这是 view 文档里的原生宿主节点、catalog tier 为 region_panel，不是可直接导入的 `[components]` 定义。catalog 声明 DocumentHost 的 tabs 接 `DocumentTabs`、content 必需 `PaneSurface`；实际 view 当前是 host → document_host → pane_surface。不把本族内容塞入未声明 slot，宿主集成先交 Parent。 |
| Panel header | `WorkbenchPanelHeader` / `ui/editor/workbench/composites/chrome/panel_header.zui` | title 必需，actions 支持既有 button/icon/dropdown 等实例；本族供给标题与业务动作。 |
| 属性 | `WorkbenchPropertyEditorRow` / `ui/editor/workbench/inputs/property_editor_row.zui` | 必需 value slot 接 Field/Dropdown/NumberField 等；继承、override、reset 的状态与业务 owner 依赖 AB-SHARED-04。 |
| 诊断 | `WorkbenchDiagnosticRow` / `ui/editor/workbench/composites/feedback/diagnostic_row.zui` | severity/message 为必需 slot，均接受 WorkbenchStatusItem；节点/位置跳转动作支持需 Parent 明确，不能假造未声明 action slot。 |
| 树 | `WorkbenchTreeRow` / `ui/editor/components/workbench/primitives/data/tree_row.zui` | 已有 tree_depth/expanded/selected/focused 状态和共享样式；继承标识、drag target、root/immutable 提示列入 AB-SHARED-01，不另建同名替代组件。 |
| 预览 | `WorkbenchPreviewViewport` / `ui/editor/workbench/primitives/data/preview_viewport.zui` | 当前根是 Image，默认引用 `animation_mannequin_viewport.svg`，输入关闭。这是已确认静态默认内容，不能证明任何本域实时预览；真实 renderer/input/resource 宿主由 AB-SHARED-05 解决。 |
| 图 | 材质会话牵头复用 `ui/graph` foundation；catalog 中尚未确认可供本族直接使用的完整 typed graph/pin/canvas 身份 | AB-SHARED-02 请求材质 owner/Parent 给出 document/component/slots/state/command 合同；本域提供真实图数据与命令，exec/data schema 不套用材质 Dag。具体 pin identity/connection verdict 等见共享合同，不新建平行图组件或调色板。 |

原始资产继续为 `res://ui/editor/world/extension_prefab_editor_workspace.zui`、v2、`WorkbenchExtensionPrefabEditorWorkspace`、root `prefab_editor_workspace`。现有 31 nodes、15 events、7 widget imports 的基线见 [证据](actor-blueprint-evidence.md)。不重建根组件、重置 control ID、flatten 导入实例或改写事件名称来迁就设计。

必须保留的事件分组：三条 hierarchy 行、四条 component/override table 行、Apply/Validate 两按钮 Click；Prefab/Variant/Instance 各自的 Change/Submit。现有 `hierarchy.mesh`、`component.loot_socket` 等 fixture 专用路由不能硬编码为任意实际组件的标识。动态行需实际 sourcePath/sourceNodeId/instancePath/itemKey 与类型/字段身份；导出后按旧事件/绑定逐项对账，由 adapter 解析真实 target，不能按显示文案分发。

现有 hierarchy 使用 WorkbenchListRow，UE 目标需要 TreeRow 的展开/缩进/drag。旧实例的组件引用及稳定 ID 如何迁移到真实树，由 Parent 结合本域 Penpot 导出确认明确映射；未确认前不擅自替换引用，也不为保留例子而把假行塞进生产树。这是 AB-SHARED-01/AB-001 的具体依赖，不是已经解决的迁移。

Apply/Validate route 与 `.apply.invoke` / `.validate.invoke` 属不同既有映射层。先追踪实际 translation，再接真实文档命令；不因为后缀不同而批量重命名。新控制和绑定仅在真实 Penpot 导出及共享合同允许后登记身份。

## AB-D07：四种环境与测量锚点

AB-V1/2/3/4 和 AB-C01…18 继续沿用既有稳定编号。1280×800、900×620、640×520 均需真实逻辑 client；150% 组须实际 HWND DPI=144、逻辑 1280×800 与物理 client 对应。字体、图标、资源 readiness 在 capture 前确认。

窄窗目标先保留 UE Dock/可调整 splitter/可恢复 tab 的关系。现有 Prefab 资产的 `responsive_min_tier` 和 details drawer 尚未证明符合 UE；是否采用共享窄窗 drawer/折叠策略由 Parent 决定并作为实际差异记录。不得仅隐藏树或 Details 使截图整齐、却让操作失去入口。最小宽高由实际控件/文字测量与共享 token 决定；当前没有测量值，不预填数字。

| 比较锚点 | 测量内容 | 当前值 |
| --- | --- | --- |
| 文档 body 与 splitter | client/chrome、pane 边界、tab well、可拖动边界 | 未测量 |
| Components | header/过滤行、展开箭头、图标、层级文字起点、选中/focus 行框、首尾可见项 | 未测量 |
| Details / Defaults | label/value 的起点/行框、字段高度、分类缩进、reset/override、首尾滚动 | 未测量 |
| 图 | 相同模型/view transform 下的节点边框、pin 中心、连线端点、标题/引脚文字行框 | 未测量 |
| 弹层与结果 | 锚点/clipping、首末项、焦点框、关闭返回、diagnostic 跳转后的实际文档位置 | 未测量 |

目标是关键边界和文本行框误差不超过 1 逻辑像素；所有值需要同内容/状态/DPI 的 Penpot 官方图和正常 EXE 全图，不能把“未测量”记为 0。图比较使用相同图坐标与 view transform，避免把相机/缩放差异误当 UI 误差。

## AB-D08：参考映射及接续输入

UE 实际 Toolkit/Slate/layout 为视觉和操作目标。项目 reference map 的辅助信息架构映射：主框架 [App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4)，树与选择 [List](https://pro.reactbits.dev/docs/app-ui/list)，默认值 [Settings Form](https://pro.reactbits.dev/docs/app-ui/settings-form)，弹层 [App Dialog](https://pro.reactbits.dev/docs/app-ui/app-dialog)，结果流 [Agent Activity 5](https://pro.reactbits.dev/docs/app-ui/agent-activity/agent-activity-5)。这里只登记批准类别；本轮没有外站 DOM/iframe 核查，不把它们作为覆盖 UE 布局的依据或复制其资产。

下一项独立工作可继续细化关联窗口的实际菜单/注册与组件 attachment、类型/脚本 schema 的能力映射。真实 Penpot 文件已分配，公共文件仍无组件；使用 Parent 交接的实际共享 component/style/version 与导入导出路径，按 AB-D01 模式及 AB-D02…07 的状态制作可编辑实例/flow，再保存/关闭/重开与导出。Timeline/Curve/Transport 复用动画 owner，图表面由材质 owner 牵头；公共入口/catalog/tokens/Starship、图/预览组件、插件导出和正常捕获协议的具体变更都先交 Parent。本文件不授权本域修改这些共享文件。

当前：区域布局草稿已保存并读回；完整组件设计 persistence 未完成，产品集成=0，normal-window capture=0，passed=0，accepted=0。区域草稿与设计规格不改变窗口验收状态。

### 真实 Penpot 区域草稿与身份

本次仅在已分配 file/page 内完成三种模式的五个内容区布局板。2026-10-24 的真实 `update-file` 为 HTTP 200，58 条 change（57 个对象及一条规划 metadata），独立 `get-file` 为 revn=1/vern=0、58 个对象（含页根）、41 个 flex frame、16 个标注文本、0 组件主件、0 实例。后续浏览器读回 revn=2/vern=0；再次逐项对照准备值，全部主动设置属性保持一致，16 个文本已有 `positionData`。这与浏览器完成文字测量的行为一致，但未抓取这次自动保存的请求，不将该解释记为独立请求证据。

| 实际 board UUID | 模式 / 内容区尺寸 | 当前范围 |
| --- | --- | --- |
| `303061a7-2206-5e2d-a663-2f682af7b389` | Unified Components，1280×800 | 默认 Components/My Blueprint、Viewport、Details；关闭页签只在规划 metadata 登记，没有画成可操作 tab。 |
| `4d20898e-4021-5260-9a9f-d6cabe1429ff` | Unified Components，900×620 | 同一 UE splitter 权重的区域草稿。 |
| `d6a3256a-7bc8-56c7-8b37-c1241993bb33` | Unified Components，640×520 | 初始长标注裁切已在后续原生重排改善；最新设计切片见本节末，尚非正式窄窗 Dock 策略。 |
| `0d9efc02-ac9b-5504-b690-329f221a9283` | Components mode，1280×800 | 左 Components/Details，右 Viewport；与 Unified 分开。 |
| `6a17c3bc-7a3c-5223-bc0f-9e50d0f3b7ce` | Class Defaults，1280×800 | 独立 DefaultEditor 内容区；没有把场景实例属性伪装成类默认值。 |

所有板均是 body composition；`client_size_verified=false`。没有公共窗口 chrome、正式 header/tab/树/属性行实例、实际文档数据或 renderer。文字仅为设计标注，规划 metadata 明确排除产品导出。背景、前景、字体和 splitter 取现有 Starship 的八个 token 解析值，保留来源 SHA-256；尚无 Penpot 公共 style ID 引用，因此共享样式验收仍 open。未在本文件建立公共 component main 或第二套 theme。

各尺寸板最初按相同 UE splitter 权重扣除共享 separator 形成区域；初始列宽/面板高度为各板固定值，内容体使用 fill。后续仅640板改为量测来源 URI 的 Details 初始宽度与剩余空间 Viewport，900/1280保留原分配。原生 flex 结构存在，但任意拖动缩放、共享最小尺寸及比例保持没有验证，不能将三个板的存在作为响应式 Dock 实现。

规划信息使用独立 `dev.zircon.actor-blueprint` namespace：原资产 URI/组件/root、31 个 source node/control 身份和 hash、区域到原节点的候选映射、实际板身份与关闭 tab。它不是 ZUI 插件导出 metadata，不能据此执行自动导出或声称保留链已验收。API 更新/响应 hash 和完整身份见 [布局回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-layout-01.json)。

### 逐图复核：仅 Penpot 区域草稿

实际隔离 headless Chromium 使用既有 EXE 与认证；首个 context 关闭后建立全新 context，重开同一文件/页面/首板。其余四板也分别打开、捕获并关闭 context。两轮均无 page error，浏览器关闭后按自有路径查得存活进程为 0。此项只验证设计持久化/可访问性；不是正常 Engine capture，也不是 `LayoutCaseReview` 的 passed/accepted。

| 完整原图 | 实际复核结果 | 未完成项 |
| --- | --- | --- |
| [首开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/first-open-r3.png) | Unified 三列与左侧上下顺序正确，1280×800 board 在右侧属性面板可见，flex 设置实际存在。 | 全部内容体为空；0 实例，非产品界面。 |
| [全新 context 重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/fresh-context-reopen-r3.png) | 同一首板及五板图层仍在，布局与首开一致；独立 API 也读回五板。 | 正式组件/样式和导出未验证。 |
| [900×620](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/board-2-audit-r4.png) | board 尺寸与三列区域存在，顺序正确。 | 左列长标注被裁切；尚无最小尺寸、tooltip/换行或实际控件测量。 |
| [640×520](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/board-3-audit-r4.png) | 三列及左上下区域均保留，没有通过隐藏区域整理画面。 | Components/My Blueprint/Details 的长标注裁切；当前不是可接受的正式窄窗设计。 |
| [Components mode](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/board-4-audit-r4.png) | 左 Components/Details 的上下关系与右大 Viewport 正确；未混入 Unified 的右 Details。 | 树、属性和实时预览尚未接入。 |
| [Class Defaults](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/board-5-audit-r4.png) | 为单独内容区，没有 Actor 组件树/Viewport。 | 类文档 authority、默认值表单与真实数据仍缺。 |

原回执 [receipt-03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-03.json) 与 [receipt-04](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-04.json) 保存实际截图 hash、URL、时间、API revision、进程身份和关闭记录。API 读回不是官方图导出；浏览器画布 zoom、DPR=1 也不是实际 HWND DPI。Interface/Macro/Level script、图编辑/选择/焦点/弹层/滚动/中文长内容以及所有正常 EXE 比较继续未完成。

### 本域业务数据组件：实际保存与边界

接续第一阶段区域草稿，在同一 file/page 新增两个业务数据主件，并在三种 Unified 尺寸各放置来源引用 1 个、覆盖记录 3 个原生实例。该阶段 `revn=4/vern=0`、117 对象、55 flex frame、2 个组件定义、12 个 copy roots；14 个 component-reference roots 包含两个主件。最新 revn12 保留这些身份和数量。没有复制 TreeRow、PropertyEditorRow、DiagnosticRow、Button、Dock 或公共主题，尚无公共 style 引用。

| 本域业务主件 | 实际 component UUID / main UUID | 版本、原资产候选与合同 |
| --- | --- | --- |
| SourceReferenceData | `8ff62384-7110-5d59-bd61-64732823f235` / `5b9092d3-1ecd-5d74-999f-69b8e082d26f` | 返回 `modifiedAt=2026-10-24T11:07:45.880Z`，包含文件 revn=4；主件页为本域 page。来源 URI/scene-instance 上下文/readiness 设计字段，候选原节点 `prefab_editor_right_content`；无 Engine component ID 或事件。 |
| OverrideRecordData | `8513f987-512c-5fc9-9dd1-059c6777a2a2` / `2f849db9-4bae-5293-985c-0f5df153c64b` | 返回 `modifiedAt=2026-10-24T11:07:45.881Z`，同一文件修订；字段为 entity_path/property_path/value。每个 copy 保留原生 component-file/id、shape-ref；无 mutation/跳转/undo 事件。 |

版本只记录服务真实返回的 modifiedAt 和 file revision，没有虚构组件语义版本。12 个 copy UUID、父对象、原始 payload 与 shape identity 保存在 [domain-data-01](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-domain-data-01.json)。主件用原生 column flex，文本使用 fill/auto-height；窄宽实际 URI 裁切仍见下表，不能据此声称响应式已完成。

内容是 `zircon_plugins/prefab_tools/editor/src/tests/cases.rs` 的 `prefab_override_query_preserves_ordered_conflicts_for_same_property` **设计夹具**：来源 `res://prefabs/ship.prefab.toml`，root / Transform.translation.x 两条覆盖值 1、2，另 root / Transform.translation.y 值 3。本轮没有执行该测试或真实查询，没有确认 URI 能加载，不是生产数据。图中明确标注来源为 fixture、readiness 未验证、无写命令或实时资源；这些标注不导出为产品控件。

`PrefabPropertyOverrideAsset` 只有 entity_path/property_path/value，没有持久 override UUID；现有查询刻意保留重复路径。两字段 tuple 不是唯一 mutation identity，source_index 只可标识本夹具修订内的显示位置。接真实编辑必须由资产 owner 提供稳定身份/修订校验，不能将行序号、同路径或显示文本直接写成持久 itemKey。该业务组件也没有类默认值/父类模型语义。

### 实例文字错误及修复证据

首轮 [receipt-05](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-05.json) 的四图都显示覆盖值 1/1/1，而独立 get-file 的 copy content 已是 1/2/3。已据实际 `dev/penpot/common/src/app/common/logic/libraries.cljc:1699` 的 text-change-value 与现有 text_sync_test 追踪：只有 content-group 不足以保留不同文本，还须 text-content-text sub-touch。第一次使用无类型 set/val 修改 touched 被服务拒绝，文件仍 revn=3；[domain-data-02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-domain-data-02.json) 保留 HTTP500 assertion/hash，未关闭验证或改服务。

改用 `common/files/changes.cljc:62` 声明的原生 set-touched 操作，补齐 9 个实际不同文字的 copy 标记后，HTTP200 保存 revn=4；独立读取仍为同一 117 对象/2 主件/12 copies，content 和 shape-ref 均保留，见 [domain-data-03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-domain-data-03.json)。未修改 Penpot 源码、共享组件或 Engine 文件。

随后 [receipt-06](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-06.json) 实际建立四个隔离 context，依次首开/关闭/全新 context 重开 1280 板，再分别重开 900 与 640；每次独立 get-file 都为 revn=4。page errors=0，所有 context 和自有浏览器均已退出。receipt-05 的旧字段 component_instance_count=14 实际计入两个主件，保留原回执并以 receipt-06 的 copy count=12 澄清。receipt-06 的 business_instance_dom 为空数组，未作为 DOM 内容断言；文字正确性来自下列完整原图逐图查看。

| 完整原图 | 逐图复核结果 | 仍缺 |
| --- | --- | --- |
| [修复后首开 1280](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-1280-first-r6.png) | 来源和三条记录实际可见，覆盖依次 1、2、3，最后一条为 translation.y；五板和两个主件图层身份保留。 | 仅业务叶数据设计；正式公共控件、窗口 chrome、树/图/实时 viewport 未接。 |
| [关闭后全新 context 重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-1280-reopen-r6.png) | 同一首板、实例文字和布局保留，截图 hash 与首开一致。 | 不证明产品查询、资源加载、导出、EXE 或硬件 DPI。 |
| [修复后 900](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-900-r6.png) | 三条覆盖和 URI 可见；说明文字自然换行，保持三列可达区域。 | 左列标题仍裁切；非正式 Dock/窄窗策略，控件焦点和滚动未接。 |
| [修复后 640](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-640-r6.png) | 属性标签随宽度换行，1/2/3 和 x/x/y 均可辨；说明未溢出下边。 | 来源 URI 与区域标题仍横向裁切；长无空格 URI 的显示/完整复制、tooltip 与滚动策略需共享属性行合同，不以改坏原始 URI 的硬换行掩盖。 |

修复前四张 `*-r5.png` 已逐图查看并原样保留：两个 1280、900、640 都为 1/1/1，最后一条错误地显示 translation.x；640 同时有上述裁切。它们是失败证据，不列为已修复画面或 accepted。上述阶段共保留并查看14张原图（6区域、4错误实例、4修复实例）；本轮另追加下方9图，总计23张，均非 Engine 原生捕获。关键边界/文本误差仍未测量；插件/Rust、正式导出和150%/双端 review继续pending。

独立只读 [业务实例复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-domain-verification-01.json) 核对两个 main 的组件/page/root、12 个 copy 的 component-file/id/shape-ref/parent、36 个子文本引用及内容和 9 个不同文本的 sub-touch，错误列表为空。这是文件数据身份复核，未补造 receipt-06 中缺失的 DOM 断言，不扩展为窗口行为或像素验收。

### 原资产回写与运行加载的具体依赖

[实际源码审计](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/source-integration-audit-01.json) 区分三条资源链。业务数据主件仅提供 slot 内容提案，不能靠自有 metadata 把它们注册成共享 Engine component，或代替以下依赖：

- `plugins://prefab_tools/editor/authoring.zui` 由 discovered package manifest 的父目录绑定 root，再严格拼接 owner-relative 路径；`compile_plugin_v2_document_sources` 在全部文件加载编译后才发布。canonical 包缺该文件及 `prefab_instance.zui`，实际 distribution root 尚未观察，不声称已触发加载错误。
- Prefab Inspector descriptor 当前仅 target/URI/controller，`template_id=None`、`data_root=None`、`bindings=[]`。snapshot 的 customization_available 不是文档加载证明；真实反射字段仍可投影，缺 schema 的 JSON fallback 只读。通用 host 会校验 enabled binding 并调用 operation，本域需提供实际写命令/绑定和事务，不通过伪造 changed 回执接入。
- 原 page/index 使用 `res://ui/editor/world/extension_prefab_editor_workspace.zui`；native pane 身份检查和菜单测试仍期待 `/workbench/modules/extensions/world/workbench_extension_prefab_editor_workspace.zui`。Parent/原 owner 需交当前 bundle 的真实 source/instance/review-host 闭包，确认是否已有 remap；本域不补复制路径、改共享断言或套用历史主窗口 capture。

通用 `view.prefab_tools.authoring.open` 与资产操作 `prefab_tools.authoring.open` 是不同命令：前者由 editor_support 注册并发送 OpenView，后者仍在缺失写命令的既有测试集合中。正式场景→资产跳转必须携带真实资产/文档/selection/history，不能因通用 view 能打开而关闭 AB-007。

### 本轮 flow 量测、原生重排及逐图接续

真实文件/页面、五个 board、两个 component/main UUID和12个copy roots均沿用上文；没有生成公共component或修改原ZUI。最终独立API于2026-10-24T12:57Z读回revn12/vern0、117对象/55 flex/2主件/12 copies；逐一核对12个原始copy的component-file/id、shape-ref、parent、36个子shape-ref及字段文字，错误为空。源数据仍是明确标注的测试夹具，不是已运行的查询。主件modifiedAt仍为11:07:45.880Z/.881Z，未虚构组件版本。

现有字体实际加载为Roboto 13.333333、line-height16；无空格URI的浏览器测量advance为181.8943328857422。640 Details的182初始宽度来自该量测取整，Viewport取剩余355.2、左栏94.8、两条separator合计8；这是内容区设计参数提案，未确认为UE/公共字体、共享Dock政策或可交互最小宽度。900/1280列分配未改。12个业务copy宽度改为fill，设计标注使用fill/auto-height；640和900各两处左栏标注通过原生文本控件重测为32高，未给文本插入硬换行或修改业务值。

保留两类失败：初次两列fill导致640 Details=359.595、Viewport=177.605，URI可见但空间分配偏离目标；随后API直接set width=182只改属性，未同步selrect/points、子宽度及父布局，产生裁切。最终通过真实原生尺寸控件提交182，原生update-dimensions事务同步重排，Details的width与selrect.width都为182，Viewport二者均为355.2。原生主件/实例文字与样式保持。没有手工修补截图或逐项偏移坐标。

| 完整原图 | 保存修订与逐图观察 | 局限/后续 |
| --- | --- | --- |
| [640初次flow](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-flow-640-r7.png) | revn5：URI已可见，左栏标注第二行尚裁切。 | 后被原生文本重测替代；过宽Details仍待修。 |
| [900初次flow](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-flow-900-r7.png) | revn5：URI、1/2/3及x/x/y可见，左栏标题仍裁切。 | 保留历史，不充当最终图。 |
| [1280初次flow](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-flow-1280-r7.png) | revn5：区域与数据顺序保留。 | 仍有大面积空区域，无真实控件/预览。 |
| [640原生文本重测](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-flow-640-r8.png) | revn9：左栏两行完整，URI/覆盖值可见。 | Details/Viewport过宽/过窄，后续已替代。 |
| [900原生文本重测](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-flow-900-r8.png) | revn9：两处标注第二行only可见；URI、覆盖值与警示可见。 | 最新900设计图；没有真实树、图、焦点或滚动。 |
| [1280重开](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-flow-1280-r8.png) | revn9：单行标注、数据和区域顺序保留。 | 最新1280设计图；未增加完整Toolkit内容。 |
| [导出失败后全新重开640](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-flow-640-r10-unselected.png) | revn10：错误页未持久化，真实Escape取消选择后layout only首字可见。 | 旧fill分配仍存在，不能当最终空间策略。 |
| [API宽度失败640](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-flow-640-r11-balanced.png) | revn11：**失败**，属性宽度与selrect不一致，URI/属性/警示裁切并留下未使用区域。 | 原图保留，文件名不表示已平衡或通过；后被原生事务替代。 |
| [最终原生重排640](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/business-flow-640-final-native.png) | revn12：全新context再次重开；94.8/355.2/182分配保留，完整URI，属性自然换行，1/2/3、x/x/y和完整警示可见。 | 最新640设计图；仍是区域/夹具研究，没有产品viewport或事件执行，不测为≤1DIP。 |

[flow-04](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-domain-flow-04.json)保留量测与初次30项约束更新；[flow-05](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-domain-flow-05.json)汇总原生几何恢复、最终独立身份与9图观察、8项Penpot只读源码SHA。原始[receipt-08](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-08.json)、[receipt-14](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-14.json)保留未选中文本和旧title定位失败；真实控件成功证据为[receipt-16](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-16.json)，最终关闭/重开为[receipt-15](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-15.json)。未改Penpot源或注入内部store。

### 原生导出实际边界

已在真实中文界面使用“新增导出”及“导出1个元素”，保存640板PNG 1×预设为revn10。实际网络`/api/export`返回404，随后显示Penpot内部错误，20秒内没有download；900/1280同一边界不重复尝试。最初错误查找英文按钮的失败另保留，不能将它描述为服务错误。实际[receipt-11](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-11.json)记录请求状态和原生进程树退出，未生成PNG、未启停服务。Parent/原pipeline owner需核对当前路由/导出服务；本域不绕过为手工SVG/截图导出。

以上23图均包含Penpot chrome，浏览器DPR=1、画布缩放约148%/109%/78%；不是Windows实际150%DPI、官方ZUI export或正常EXE图。该阶段12:57Z公共读回为revn0/空库/isShared=false，是历史状态；下方新消费试验以实际rev1为准。真实控制/数据/后端、正式源回写、托管Rust、正常EXE及双端逐图验收继续open，passed=0、accepted=0。

### Shared rev1：本域受控消费板

按Parent最新分配，在原file/page新增board `89eb3427-27ba-801b-8008-d6cfcabb7a3c`，名称`AB-SHARED-01 / rev1 linked consumer - not product`。原生画板与Inspector事务保存960×600、column flex、gap16、四边padding24；这些是消费试验板的排布，不是新公共尺寸/token。原五板、两个业务main、12copies和源资产映射均保留。该阶段Actor最终revn21/vern0、151对象；Shared仍rev1/isShared=true/41对象/4主件。真实API、引用和过程见[consumer-01](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01.json)。

通过原生Assets→管理库→连接按钮建立外部library关联，HTTP200后另用get-file-libraries读回Shared rev1。四件均由Assets的真实dragstart/dragenter/drop触发`instantiate-component`，未手造copy/shapeRef或复制main。受控板只消费现有组件，不改Shared/catalog/tokens/Starship。

| 组件 | Shared component UUID | 原main / 本域copy UUID |
| --- | --- | --- |
| WorkbenchButton | `6ddc248b-bd35-80c5-8008-d6c8c01096f3` | `6ddc248b-bd35-80c5-8008-d6c8bd9b057f` / `89eb3427-27ba-801b-8008-d6d09c3492b9` |
| WorkbenchSearchInput | `6ddc248b-bd35-80c5-8008-d6c8c3ce892e` | `6ddc248b-bd35-80c5-8008-d6c8c0f9c196` / `89eb3427-27ba-801b-8008-d6d0b477bb2b` |
| WorkbenchTreeRow | `6ddc248b-bd35-80c5-8008-d6c8c7a79944` | `6ddc248b-bd35-80c5-8008-d6c8c452bf0c` / `89eb3427-27ba-801b-8008-d6d0b5438093` |
| WorkbenchPropertyEditorRow | `6ddc248b-bd35-80c5-8008-d6c8cab1e7d0` | `6ddc248b-bd35-80c5-8008-d6c8c84d848f` / `89eb3427-27ba-801b-8008-d6d0b6203905` |

四件的canonical-record与canonical-source位于**component.pluginData.devZirconZui**；根shape另外保留sourcePath/sourceNodeId/instancePath/controlId。各主件asset board的document是带typedValues的包装，解包后的penpot_original_source与当前四个原ZUI逐字节一致；不能因为root.metadata无字段而判定无源。组件记录version1是metadata格式版本，library rev1与真实modifiedAt另记，不造组件版本号。完整33节点的shapeRef、源元数据均与Shared主件对应，未detach；原116个非根内容对象、两个业务component记录及原顶层相对顺序逐项比较不变。

原生文字编辑把Button改为“添加组件”，text shape `89eb3427-27ba-801b-8008-d6d09c3492ba`仍引用`6ddc248b-bd35-80c5-8008-d6c8bdb3e2f8`，native touched为`content-group`/`text-content-text`。rev19关闭/新context重开后保留；再用原生“重置覆盖项”得到rev20/Button且touched清空；最后重设中文并保存rev21，第二次新context重开保留。此为Penpot本地文字覆盖/重置试验，不是类/实例属性reset、公共样式A/B传播或业务Add执行。

PropertyEditorRow的`value`必需槽仍空：当前四主件没有catalog允许的Field/NumberField等value组件。copy slot `89eb3427-27ba-801b-8008-d6d0b6203907`仍引用原`6ddc248b-bd35-80c5-8008-d6c8c88eaacb`、无children，未塞入SearchInput或伪造Field。Search/Tree/Property仍显示主件默认文字，未接真实查询/选择/属性事务；本板不能计为完整Prefab窗口。colors/typographies仍0，paint沿主件继承不等于native共享style引用。

以下11张新增原图均已打开逐图核对，总设计原图34张；本轮浏览器1600×1100、DPR1、画布约93%（早期全景12%），没有Windows144DPI或产品证据。

| 原图 | 实际观察与限制 |
| --- | --- |
| [连接前](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01/before-link.png) | 原五板/业务main的12%全景；尚无Shared实例。 |
| [原生库弹窗](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01/native-library-linked.png) | 截图时弹窗仍绘制Shared加号/4组件；不能靠文件名证明连接，实际HTTP200及独立libraries读回另证。 |
| [首次画板动作](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01/controlled-board-initial.png) | 工具切换尚未就绪时的动作未创建board；保留失败全景。 |
| [素材与空板](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01/shared-assets-ready.png) | 原生960×600板和四主件素材可见；板中尚无copy。 |
| [首次拖放](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01/first-linked-copy.png) | 板仍空；API也为0Shared copies。修正拖动路径、先进入空白viewport-controls后才成功，不倒填为首copy成功。 |
| [四个copy](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01/four-linked-copies.png) | 4组件沿column flow排列，属性value为空；Inspector确认复制引用。 |
| [中文字覆盖](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01/local-caption-override.png) | 添加组件可见、未见缺字；这是text子shape的选择框，不作为root宽度改变。 |
| [第一次新context](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01/reopened-caption-override.png) | 中文、四copy和flow Inspector保留，尚未执行reset。 |
| [原生reset菜单](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01/native-reset-menu.png) | 真实copy上下文菜单含重置覆盖项；没有点击解绑或更新main。 |
| [reset结果](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01/local-caption-reset.png) | 恢复Button；API确认相同copy/ref、touched清空。 |
| [最终第二次新context](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01/final-reopened-linked-consumer.png) | rev21恢复中文后保存重开，Shared库标题和4copy可见；本轮最终消费图。 |

失败locator/工具就绪/拖放/文字选择、API UTF-8读回错误与修正均记入consumer-01；不把截图命名或原生素材缩略图当验收。隔离浏览器正常exit0，9个有出生身份的自有进程均退出，新临时目录0文件；旧126B仍待原Jenkins清理。公共style/export、真正属性槽、正式源回写、托管Rust、完整Toolkit/实时preview及正常EXE双端验收保持open，passed=0/accepted=0。

### AB-CONSUMER-02：中文内容与原生宽度约束复现

在同一file/page通过原生Ctrl+D复制已连接consumer板；新板保留Button、Search、Tree三个native linked copies，未复制main、detach、改Shared或填入非法Property value。添加组件、搜索组件、PrefabInstance（场景实例）用于设计内容检查；PrefabInstance是实际组件类型的名称，**不是选中实例查询或生产fallback**。初期AB-SHARED-02板名前缀已原生改为AB-CONSUMER-02，避免与材质Graph合同同名；旧图保留历史。

| 真实board UUID | 逻辑尺寸 | copy根宽 | 实测内部约束问题 |
| --- | --- | --- | --- |
| `d9109291-fc9f-8088-8008-d6e13ae96660` | 640×520 | 592 | Button文字中心偏左176；Search背景仍240宽；Tree文字仍128宽、观察为2行。 |
| `d9109291-fc9f-8088-8008-d6e1ba22bcaf` | 900×620 | 852 | Button文字中心偏左306；Search背景仍240宽；Tree文字仍128宽、观察为2行。 |
| `d9109291-fc9f-8088-8008-d6e1e02f8f15` | 1280×800 | 1232 | Button文字中心偏左496；Search背景仍240宽；Tree文字仍128宽、观察为2行。 |

这些数字来自Penpot对象几何，不替代UE/EXE双端≤1DIP量测。三copy按column flow及既有padding24/gap16排列；fill只改变根宽，未自行改共享组件内部坐标、约束或token。Button文字保持220宽、Search文字204宽、Tree文字128×16；截图确认中文可绘制，但树行换行、动作停留在源宽位置。

最终rev30/241对象，原150个非根对象、两个业务main/12copies和原顶层相对顺序不变，仅根对象增加新板。新增9个linked roots、87个shapeRef均存在于Shared rev1；库仍41对象/4组件、colors/typographies0，对象与component记录哈希保持。rev29及改名后的rev30分别关闭/全新context重开，保存前后对象哈希变化0。copy/source/control身份见
[component-content-02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/result.json)。

静态检查发现，9个copy的native-layout-allocation仍记录canonical主件父节点；TreeRow的treeRowPainterSize校验实际parent。没有调用插件refresh/export，因此仅记录接入约束，不称为已触发的插件运行错误；需owner保留guard并处理native copy/source/shapeRef/generated-state/allocation映射。具体公共提案见[共享合同](actor-blueprint-shared-contracts.md)及[Parent交接](actor-blueprint-parent-handoff.md)。

以下12张新增原图已全部打开逐图复核，连同旧34张共46张。它们仍是1600×1100、DPR1、画布约134%/98%/71%的Penpot browser图，不是正常EXE、官方PNG/ZUI导出或Windows实际150%DPI。

| 原图 | 实际观察与限制 |
| --- | --- |
| [before-native-consumer.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/before-native-consumer.png) | 原rev21四件验证板，Button中文、Search/Tree/Property默认；属性value仍空。 |
| [640-before-fill-long-text.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/640-before-fill-long-text.png) | 新增640板中中文搜索和PrefabInstance场景实例文字可见；240宽Tree文本已换两行，未连接实际场景。 |
| [640-after-fill-native-content.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/640-after-fill-native-content.png) | 三个copy根均fill至592；Button文字位置、Search内部背景和Tree文字/动作仍按240源宽绘制。 |
| [width-duplicate-not-ready.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/width-duplicate-not-ready.png) | 原生复制/重命名尚未就绪，出现混合尺寸和多选；守卫拒绝尺寸写入。保留失败，不当作900状态。 |
| [900-native-content-width.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/900-native-content-width.png) | 原生900x620板，根宽852；内部固定宽与Tree两行复现；侧栏名称处于旧文字状态。 |
| [1280-native-content-width.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/1280-native-content-width.png) | 原生1280x800板，根宽1232；内部固定宽与Tree两行复现；不是完整窗口。 |
| [640-fresh-reopen.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/640-fresh-reopen.png) | 第一次fresh context的640板，中文与fill保留；同一内部宽度问题复现。 |
| [900-fresh-reopen.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/900-fresh-reopen.png) | 第一次fresh context的900板，名称/尺寸保存；同一内部宽度问题复现。 |
| [1280-fresh-reopen.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/1280-fresh-reopen.png) | 第一次fresh context的1280板，名称/尺寸保存；同一内部宽度问题复现。 |
| [640-final-reopen.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/640-final-reopen.png) | 最终rev30第二次fresh context，名称改为AB-CONSUMER-02避免与Graph合同同名；640板/中文/实例引用保留。 |
| [900-final-reopen.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/900-final-reopen.png) | 最终rev30第二次fresh context，900板/中文/fill保留；Search仍240、Tree仍两行。 |
| [1280-final-reopen.png](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/1280-final-reopen.png) | 最终rev30第二次fresh context，1280板/中文/fill保留；Button文字仍偏左，未达到完整窗口目标。 |

自有浏览器正常close exit0；最终10个有出生身份的自有进程退出，新临时根0文件。两次close前样本共22,262,675B逻辑临时文件由浏览器自动释放，物理量未测，手工删除0B。stdin关闭、自有终止helper退出-1及复制输入未就绪的失败保留；旧126B仍待原Jenkins管理身份清理。原ZUI和四份Prefab Rust保持原SHA，无新Cargo；原#348 preparation-conflict、8项测试/0执行不变。真实query/选择/写命令/undo、Blueprint后端、正式导出、正常EXE/case/桌面/DPI及双端逐图验收仍open，passed=0/accepted=0。

### AB-D13窄窗来源说明的流式修正16

原分配Actor file/page的两张640状态板保持UUID不变，原生Layers拖放将既有document-mode文字从actions移到根column，位于actions与workspace-main之间。原文“来源定义 / 无选中实体”从128×32变为624×16、完整单行；两Button仍240宽，不改Shared内部宽度、main或样式。主区通过fill由416变392，高度24来自正常flow，不写普通内容x/y。1280/900和其它板的935对象精确不变。

使用既有UE Toolkit统一区域与原资产compact适配边界，延续[App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4)的窄窗信息优先级；状态为独立阅读信息，不把它拆成两个交互控件。原AB-D13的rev107换行是历史观察，现rev109仅此窄窗文字布局已修正。

[六图逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-status-16/visual-review.json)和[保存/重开身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-status-16/save-reopen-preservation.json)证明关闭原context、新context及独立API均rev109/data相同，Shared5不变；993对象/2main/58Shared roots、653引用（445Shared+208本域）全部实际解析。所有文字/content、source/slot/event/touched和paint/layout属性保留。另2个Search图标path的content是坐标路径，原生flow将其所有y同移24，M/L命令及点数不变，不能谎称这两个路径的content字节未变。

这是Ready-empty/NotResident的unit状态设计，未接真实生产publisher/操作/undo。Penpot来源身份包络仍缺，正式ZUI导出、正常Actor EXE、150%DPI、焦点/滚动首尾和双端≤1DIP仍open，passed/accepted=false。
