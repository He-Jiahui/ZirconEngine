# 材质窗口族：源码盘点、实现与证据

本会话：`01a16d50-6c7f-7370-853d-4581184b66a1`。Parent：`01a16d4e-0491-7353-84fa-038bd7706546`。套件日期：2026-10-24；实际运行回执使用 UTC。

## Expression Details73：节点默认值与真实纹理引用（2026-10-27）

继续在原ready工作树w1167fba90b的四条路径内实现。实际 [MaterialGraphNodeKindAsset](../../../zircon_runtime/src/asset/assets/authoring.rs) 支持ScalarParameter／VectorParameter的default和TextureSample的必需AssetReference；图级parameters是另一张显式override map。UE的MaterialExpressionScalarParameter.h／MaterialExpressionVectorParameter.h的DefaultValue和MaterialExpressionTextureBase.h的Texture均为可编辑源属性；本域不推导UE Group／SortPriority／slider／SamplerType等当前schema没有的字段。已有真实Penpot Parameter proposal主件904a343b／main a4544d75含8对象和20个原生copy，本轮只读289／3906并保留原件，未把它称为通用Graph库或已经接通的默认值属性面板。

[details_selection.rs](../../../zircon_plugins/material_editor/editor/src/details_selection.rs)新增plan_material_graph_expression_details_edit及私有构造的候选。它复用Details整组source身份检查，以精确node ID定位原借用节点；陈旧／空白／重复源键拒绝，Output仍走原Graph Details，Add／Multiply没有伪造可编辑默认值。候选只保存有限的标量／四通道前后值或原纹理引用，不克隆全图、不改节点ID／位置／kind／参数名／连线／override；相等默认、零值和wrong-type override均保留presence。私有类型化前后值对避免错误kind组合，公开路径没有unchecked索引或不可达分支panic。名字迁移和override reset不混入默认值编辑。

新的数值必须finite，允许finite负值与HDR，不依据某个PBR槽把源节点统一夹到0..1。比较保留实际Real位模式，+0→-0是作者值变化，codec重开仍需保留符号；当前原zr_math::Real=f32，只复用原公开别名而不改变数学精度。Texture没有可伪造的None／fallback reset：实际变化的UUID＋locator完整对通过调用方当前catalog解析一次且必须ResourceKind::Texture；同对no-op不重新查询，不以文件后缀当类型证据。真实host仍须在同view／asset／document／subdocument／source revision／selection request和resource上下文重新校验，再以原预算准备一次field delta／inverse／permit／CAS／primary history，并承担dirty、Details发布、编译失效、renderer及保存；候选本身不执行这些动作。

新增12个material_graph_expression_details_回归定义，覆盖源借用、同名参数的不同节点、override presence、finite／HDR／四通道与signed-zero、旧非法值修复、精确长中文ID／stale／歧义、字段kind、完整纹理对／resolver次数／错误，以及真实MaterialGraphAsset codec→既有CPU material compiler的roughness／base_color／base_color_texture。测试里的小图复制和字段安装只用于codec／compiler夹具，不是生产作者事务、renderer或live producer。总静态定义169→181（180普通＋1原ignored）；实际Rust发现／执行仍未知，不能借旧13／19或68输入背书。

本轮运行三Rust格式、四路径diff／reverse／reachability guard及单材质plugin静态检查后正常deliver-task。当前实际pinned31652db8 runner仍要求nested plugin使用canonical checkout，原main只含source61，不能把当前worktree73塞进旧68／r10，也不回写共享main或用根workspace／本地Cargo代替。受管Rust需合法当前source路由或集成后新准入；原68／#496 FAILURE、r19 reader／native／resource未闭合继续独立保留。具体输入／patch／guard与下一项host消费边界见稳定报告根的material-expression-details-73-*；原ZUI导出、专用Field与表达式Details设计消费、正常EXE／实际150%DPI／双端1DIP和整族passed／accepted继续open，goal active，释放0B。

## Details tabs72：六板原生流式宽度修正（2026-10-27）

在本域原Material文件 `b4a1e3ce-9e6a-81fe-8008-d69d6092d704`／page `b4a1e3ce-9e6a-81fe-8008-d69d6092d705` 的既有默认／root选中六板，修正 Details、Parameters、Nodes Overview 页签。原84宽等分allocation令96逻辑像素的Overview文字落入70宽clip。通过真实workspace inspector的Horizontal Fit content依次操作18个caption clip和18个tab allocation，不按截图坐标写几何；原生保存253→289，3906对象保持，48原对象变化、0新增／删除、页签以外0变化，文字／样式／组件引用／原prototype均保持。新宽为56／83／110，clip为42／69／96，原256宽row和2间距保持。六个原board UUID见本轮review，不重排fork的Instance／Hierarchy／四SearchInput。

保存后显式关闭原context，再以新认证context重开，全部3906 shape哈希与保存结果逐一相等；viewer后再次核对仍相等。1280×800、900×620、640×520的默认／选中六张真实Penpot PNG已逐图打开复核，18个实际DOM text Range均在对应clip内。中文Name和Shader locator保持；选中仍是原蓝色描边，不能据此签UE阴影painter。预览与Compiler Results仍显示未编译设计状态，不是引擎渲染或实际编译结果。原交互metadata逐项相等，本轮没有重新执行页签跳转、焦点或快捷键。

首轮多选inspector找不到原生控件、两次viewer测量scale0,0均保留为失败。实际只读诊断发现字体ready后同一frame ID出现隐藏0×0测量SVG；最终捕获按确切source frame选取含原caption的可见SVG，并等待fonts loaded、限定同一root测量。未改DOM样式或设计几何，也未将此局部捕获方法视为官方codec／capture source修复。最终Node49804正常exit0、browserClosed／authUnchanged为真，复用browser-overview-52临时目录空，释放0B。

稳定报告根为 `E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/`：`material-details-tabs-72-save-r2.json`、`material-details-tabs-72-reopen-r3.json`、`material-details-tabs-72-review.json`及六张`material-details-tabs-72-*-r3.png`。本轮只在ready工作树增量记录本md；[原material_workspace.zui](../../../zircon_editor/assets/ui/editor/modules/core/rendering/material_workspace.zui)没有写回。source69三Rust不变，复用同源插件／静态证据并执行单md结构、diff与guard；不新建Cargo身份、不重投已终态68／#496。正式原ZUI导出／加载、真实host／renderer／Apply／Save、正常EXE／实际150%DPI／双端≤1DIP仍open，whole-family passed／accepted=false，goal active。

## Shared 合同71：选择提示与键盘焦点分开（2026-10-27）

已消费 Parent r125 的共享合同提交 `23dc04952ad52b501522119d84d3596efd60ee43`（queue26936／management#502 SUCCESS，仅 [component-reuse.md](component-reuse.md)，main integration pending）。其原回执 SHAab1a9ec686a8c07e0aca03af42562ff909a128c3d7d6cc59e5b246d5a5ec6233 与提交正文 SHAa5292635801b76ad64af5259300a6cc96711d7019e0b04f2d6fd20b99d4f5fdb 已实际核对。原58三图、source61、67三图和旧失败继续按各自时间保留，不重套设计或公共源。

**调用勘误：** UE `MaterialEditor.cpp:8748` 在 SetObjects 后调用 `FocusDetailsPanel()`；其 `9175–9181` 实现只对有效且非前台的 Details tab 调用 `DrawAttention()`。下文58／67的这个函数名不表示键盘焦点转移，也不证明关闭弹层后返焦。选择回调应发布已校验的 Details 目标并按既有页签规则提示关注；键盘焦点仍由真实 retained input owner 处理，不能为匹配函数名抢焦点。

本域当前 source69 的选择投影段与 source61 按 LF 规范化后逐字相同（SHA97442d19377ab06416abce1af9a1590d1c18af9c44229e91626bff2becea6876）。继续从同一 Document 借用原 Graph／Expression，保留 source 顺序、Output 原 ID 和作者 shader presence；非空 stale／重复／缺失／歧义选择整组拒绝。读取与发布前均须核同一 view incarnation、asset／document／subdocument、revision、request 及 selection lease，不能在 paint 中重建、重捕获 expected 或从 effective MaterialAsset 还原作者源。69新增 Name／Shader 候选没有安装这个生产 caller。

选择阴影由原 retained painter 在原 shadow layer 使用已资格化 brush 和膨胀 paint geometry；原 body allocation、pin／hit bounds、viewport clip 和同一次 Canvas／DPI 变换保持。UE 的 12×12 shadow size 与 18/64 九宫格边距不是节点命中框；原2px设计描边不因此变为正式阴影。公共 host／painter、真实 PropertyEditorRow source slot、官方 codec／export、原ZUI加载及引擎焦点仍待接入；本轮不改公共实现、Penpot 或 fork 的 Instance／Hierarchy／四 SearchInput。

验证时序更新：r125 的23:29Z building是历史观察；同一68 queue26827／#496已于2026-10-27T00:04:16Z认证读回 FAILURE，原550B归档 SHA0583993fcd4fe830fa586459e9e16bfbaae5fd0b25f668a802ec6e1096093010 仍是 checkout_source_busy／原r19 reader。编译、实际发现／执行和 native／资源终态仍unknown；不重投68、#420、#448或r10。source69的169静态定义不倒签68的161项输入；当前实际提交为69的c9b87309和文档70的cf053d1b，均不等于Rust通过。原终态hand-off以material-details-current-68-terminal-handoff.json为准。

本轮仅增量更新本md，复用同源Rust／插件静态证据，运行文档结构／diff／guard检查和正常deliver-task；无新Cargo身份。goal active，真实renderer／Apply／Save、正常EXE／实际150%DPI／双端≤1DIP与整族passed／accepted保持open，释放0B。

## Details edit69：Name／Shader 的真实作者候选（2026-10-26）

接续66／67已保存、关闭、新context重开的两组Details设计，在原ready工作树w1167fba90b-mat64的四条授权路径内补齐图级字段候选。原 [MaterialGraphAsset](../../../zircon_runtime/src/asset/assets/authoring.rs) 的name是String，shader是Option<AssetReference>；未把Name伪装成可缺省属性，也不将作者shader=None替换成编译器fallback。UE MaterialEditor.cpp:7689起的NotifyPostChange要求既有ScopedTransaction并安排预览／图刷新；本域 [details_selection.rs](../../../zircon_plugins/material_editor/editor/src/details_selection.rs) 的新plan_material_graph_details_edit仅产生借用原source／request的候选，实际Document、primary history、retention／prepare／一次permit／CAS、dirty／焦点／编译失效和保存仍由真实宿主承担。Name／Shader是Zircon当前作者schema的适配，不宣称完整UMaterial属性复刻。

候选保留前值与后值的精确文本、UUID和locator，不克隆全图，不创建store／clock／revision／ID，不安装变化，也不从effective MaterialAsset反推作者文档。Name的空串、空白、中文和长文本保持原样；Shader清除显式产生None。实际发生变化的Some引用调用当前project catalog resolver一次，必须返回ResourceKind::Shader；扩展名不作为类型证据，同locator不同UUID及同UUID迁移locator均需重新解析。内容相同或None→None为可见no-op，不为旧不可用引用额外查询；source本身的诊断仍由既有validator处理。无效／重复节点等图错误不阻止修复图级字段，节点、连线、参数override与孤立作者值不被重置。

在既有tests/details_selection.rs增加8项有意义回归定义：精确中文长名借用、空名与等值no-op、完整Shader身份单次解析、等值／缺席Shader no-op、UUID／locator各自变化、错误资源种类／解析失败无部分写、清除后的真实codec／compiler、Name＋Shader候选经真实TOML重开且参数／身份／PBR保留。它们仍未由Rust harness发现／执行；旧161定义与新169定义（168普通＋1旧ignored）分时记录，不用格式／guard或测试源码冒充执行。

67文档已由原queue26736／Jenkins#487 SUCCESS提交为018b93c5c1120e29450d844eee2af392f7e807eb，仅本md；2180B终回执SHA8dbc0966a8d9ad793cbb28dfa442f2c602380a044168a83e66cf9d1e210e3e50的HTTP／Home相同，集成另行pending。唯一后继validate68：原request ue-material-graph-20261024-01／operation matgraph-details-current-01a16d50-20261026-68，实际queue26827→#496；身份b6bf53c892eda8a8088bd0013499938643e2ba0ca8c13996bd932136ee221317固定提交时canonical29源／27依赖。23:22Z原550B回执为checkout_source_busy／原r19 reader；23:38Z Home后继499B回执为management_git_busy，原件各保留。同期新capability只读查询因owner PID27460的OpenProcess错误5返回process_identity_inconclusive；不据此推断服务／root退出或释放资源。compile／发现执行数／native／resource仍unknown，未重投；新69只在工作树，未混入68的旧输入或把169塞入161历史结果。

本轮未修改Penpot／Shared／fork30 Instance、Hierarchy、四SearchInput及其Overlay、公共graph／Field／主题或原ZUI。沿67真实rev253／3906的三尺寸设计及66／67原PNG，不将新候选描述成实际按钮／Field接线。正式source slot／codec导出／原v2加载、宿主事务和GPU材质修改、正常EXE／真实150%DPI／双端≤1DIP均open；原r10/#370等失败不变。guard／格式／单插件静态检查和实际deliver以material-details-edit-69原回执为准；本轮不新建Cargo身份。goal active，passed=false，accepted=false，释放0B。

## Default Details67：空选择仍检查原材质（2026-10-26）

本轮接续66当时保留的默认板差异：UE `MaterialEditor.cpp:8708-8750` 的 `OnSelectedNodesChanged` 在空选择时把原 Material／MaterialFunction 的 EditObject 交给 SetObjects，随后调用 FocusDetailsPanel。本域 [details_selection.rs](../../../zircon_plugins/material_editor/editor/src/details_selection.rs) 的空选择分支已返回 `Graph(&原作者图)`；这不等于空 Details，也不从展平 MaterialAsset 重建作者文档。现将原默认三板的错误“选择表达式”提示替换为与66一致的 Name／Shader；数据仍为61既有 graph() 声明，selectedIds=[]，明确是设计／回归数据，未描述为当前 EXE 打开的资产或完整 UE 属性 schema。

| 逻辑尺寸 | 原 default 板身份 | 新 context 重开与逐图结果 |
| --- | --- | --- |
| 1280×800 | 9fce0704-72a3-4f09-84e2-00e0d9efab78 | 4行原生文字完整，993宽图视口，Output 无裁切。 |
| 900×620 | 46039af0-ecf6-4d12-a29c-60c09d698950 | 中文名称和完整 Shader URI 可见，613宽图视口。 |
| 640×520 | 722ca546-223f-4c7f-9e52-a37348a0250e | 同一数据完整显示，353宽图视口及230宽 Output 保持可见。 |

真实 file/page 沿原 ...6092d704／...6092d705，rev242→253，3906对象不变；349变化全部位于自有三板457原对象内，0增删、0范围外变化。15原 native copies（12 Shared）及 componentFile/componentId/shapeRef 保留有效；fork30 的 Instance／Hierarchy／四 SearchInput／Overlay、66根选板及其它会话对象未变。Shared 当次仍 rev5／4原主件／34色／3字体，不写公共文件。文字使用真实 body／strong／editor.text.primary；原六层容器、Text 与自有 Search copy 用 Fill，预览提示用左右约束，preferred sidebar=256 沿66本域设计规则，未冒充 UE 数字常量、canonical intrinsic minimum 或 source/reset 资格。

这次真实原生 inspector 重排已产生4条 position-data，未手写行框，也无需重复66的临时文本编辑。既有 Output 设计 copy 用原生水平对齐保持与66的同尺寸选中板几何一致；未修改原作者节点坐标或实现引擎 camera。默认 Output 的 fills／strokes／shadow／交互及源身份与基线一致，未把空选择画成选中。原型在独立新 context 逐尺寸实点“已空时点空白→选择 Output→点空白清除”，9条路径均保持预期 Graph 字段和实际可见目标；这是 Penpot viewer 导航证据，不是引擎选择存储、FocusDetailsPanel、撤销或 Esc 结果。

三PNG均已打开逐图，文字与原生240分配一致，无字段或节点裁切。原始序列化字段顺序使初次 JSON 字符串顺序比较为 false；语义深比较、原生行框和渲染文字均相同，原判定与后续说明一并保留，未据此虚报保存失败或修改 guard。资料见原 reports/ue-material-family-20261024-01a16d50 下 material-default-details-67 的 source／save／flow-native／native-framing／reopen／pointer／review.json 及三尺寸PNG。

66的默认空提示在当时确实 pending，本节仅关闭其 Penpot 设计差异；表达式 schema、多选字段、真正 PropertyEditorRow/value、修改／重置／资产跳转、长无断点路径和原 Nodes Overview caption 裁切仍 open。源 [material_workspace.zui](../../../zircon_editor/assets/ui/editor/modules/core/rendering/material_workspace.zui) 的29节点／27控件／13事件及三行静态表格未改；正式 asset-document／codec／导出、真实宿主输入与 GPU preview／Apply／Save、正常 EXE／实际150%DPI／双端≤1DIP均未通过。本轮3 Rust 无修改，复用64同源单插件静态检查；161静态定义不当执行数，#448／r10等原失败及 native/resource 未知保留，不重投。65已交原 owner 的 committed-worktree 精确 nested recipe 缺项不阻塞本域设计。

66文档已实际由 queue26665／Jenkins #481 SUCCESS 提交为 dd191bd9e9ded3faf5cb28de4901f39779bd7e21，仅本文件；archive2164B／SHAc0b378537fa2397c08fb02bbe26de764cd96ac4dacb3fb245285cb924f349e2c 的认证 HTTP／Home 字节一致，main integration 仍 pending，不等于 Rust 或产品通过。67继续在 ready 工作树 w1167fba90b-mat64 增量记录并正常 deliver-task，实际 queue/commit 以本轮后继回执为准。goal active，passed=false、accepted=false，释放0B。

## Details66：根选择的作者字段与窄窗布局（2026-10-26）

本轮仅接续本会话的三张既有 root-selected 板，未进入 fork30 的 Instance 四板、Hierarchy、四 SearchInput 或其 Overlay 续作；未写 Shared/catalog/tokens/Starship/runtime/原 ZUI。专属 file/page 保持 b4a1e3ce-9e6a-81fe-8008-d69d6092d704 / b4a1e3ce-9e6a-81fe-8008-d69d6092d705，真实保存并在新认证 context 重开至 rev242 / 3906 objects。349 个变化全部位于这三张自有板的 457 个原对象内，无增删、无范围外对象变化。15 个原 native copy roots（含12个 Shared）及其 componentFile/componentId/shapeRef 均保留并读回有效。

Details 数据采用61既有回归 graph() 和 Output 选择用例的作者字段声明：Name 为“Material Details / 材质属性”，Shader 为 res://shaders/pbr.wgsl。它们是明确标注的设计/回归数据，未被描述为正常 EXE 当前打开的资产。根选择依据 [details_selection.rs](../../../zircon_plugins/material_editor/editor/src/details_selection.rs) 的 Graph(&原作者图)，不从编译/展平的 MaterialAsset 重建作者文档；这次替换根选择板上错误的“选择表达式”提示，仅呈现 name 和 shader 引用，并没有实现属性编辑、资产跳转或 UE 完整 UMaterial schema。

| 逻辑尺寸 | 原板身份 | 真实重开复核 |
| --- | --- | --- |
| 1280×800 | f147d768-b2cf-8022-8008-d9ae323dae26 | 原字体/共享 style refs，4行文字完整；Output 在993宽图视口内。 |
| 900×620 | c6260bd3-1397-80a5-8008-d9ae58f54a67 | 名称中文与完整 Shader URI 无裁切；Output 在613宽图视口内。 |
| 640×520 | cb46e97a-7efe-8085-8008-d9ae7bafd8c3 | 名称中文与完整 Shader URI 无裁切；Output 在353宽图视口内。 |

三张板复用现有240宽 native SearchInput 与既有 Details 左右8内边距，采用本域固定 preferred sidebar=256 的设计方案；它不是 Shared 源组件的 intrinsic minimum，也不是已核准的 UE 数字常量。原左侧6个容器、hint Text 与 Search copy 改为真正 Fill；预览提示保持原左右约束。最终尺寸和文字 position-data 均由真实 Penpot 工作区原生布局/文本编辑保存，未按截图手写 glyph 行框或覆盖源签名。Shared 实读仍 rev5 / shared=true / 4原main / 34colors / 3typographies；文字引用原 body、strong 和 editor.text.primary，未修改或发布新主题。

首次 rich-text 编码把字符串 root 写成 keyword 导致HTTP500，原失败回执保留；按 Penpot types/text.cljc 的原生字符串合同修正后真实HTTP200。首轮640 URI裁切、重排后初始取景的节点边缘裁切及两组原PNG也保留；后继只用原生流式布局和既有 Output copy 的原生水平对齐修正设计取景。这个 Penpot 对齐不等于引擎 frame-all：宿主仍须复用既有 GraphCanvasState::fit_graph_bounds 和同代际 viewport/transform，在原 source/history 边界内处理真实输入，不移动作者节点来适配截图。

最终三PNG均已逐图打开，源文字与真实渲染一致、行框在分配范围内、原根节点完整可见。新 context 的9条 pointer 路径（每尺寸：已选根、空白返回、再次选根）按实际可见目标对象验证并全部通过；首轮仅按 frame-id URL 注记断言的超时保留，Penpot viewer.cljs:613-663 的原生导航实际更新 index。该原型结果不证明引擎 selection/focus/undo，也不证明外点/Esc；空白返回的原 default 板仍有早期空提示，作为独立差异保留。

当前源资产的真实路径为 [material_workspace.zui](../../../zircon_editor/assets/ui/editor/modules/core/rendering/material_workspace.zui)，逻辑URI为 res://ui/editor/modules/core/rendering/material_workspace.zui；canonical 与受管工作树均为13809B、SHA5856c88655aafcae3b8f8af2d4ed7786ed89e61551610324faa51ce767af0c12，沿用29nodes/27controls/13events/8imports，同源证据复用。原三行表格和静态编译反馈仍在该源中；本轮没有通过官方 codec 导出修改，不能把本轮 Penpot 设计算作原 ZUI 或产品接入完成。

| 未完成项 | 具体边界与接续 |
| --- | --- |
| Default / expression Details | 根选板已显示作者字段；空选原板与表达式字段、真正 PropertyEditorRow/value slot、编辑/重置及源引用跳转仍需当前 document/revision/request 的真实宿主适配。 |
| Dock chrome / 长文本 | Nodes Overview caption 仍有早期裁切；更长无断点路径的省略/tooltip/滚动未实现，不能用当前两项字段通过关闭整套长文本和 Dock 验收。 |
| 图与选择 | 固定侧栏和初始取景只是设计状态。真实 paint/hit、同一 transform/DPI、焦点返回、共享 selected shadow 的 UE 差异及 GraphDelta prepare/一次permit/undo 保持 open。 |
| 官方往返 / 渲染 | 正式 asset-document、slot/source-delta、导出回原 v2、引擎加载以及真实 GPU material/Apply/Save 尚未完成；静态“Compile material to preview”仍明确是缺能力提示。 |
| 原生验收 | EXE/资源闭包与捕获 nonce/generation 尚无本切片有效证据；正常 EXE、实际150%DPI、双端并排/差异图和≤1DIP均未通过。 |

来源与逐图记录位于既有 reports/ue-material-family-20261024-01a16d50 的 material-details-design-66-source/reopen-r3/review/pointer-r2.json 及3个最终PNG，所有初轮失败和中间PNG保留。Source64 已由真实 Jenkins #457 / queue26228 提交为 1bd1894979eea09400e50e2ccc7387a7496d5987；主干集成以原交付记录为准。3个Rust未改，复用64同源单插件静态验证；161静态定义不是执行数。原61 #448 已 FAILURE、归档 waiting/management_git_busy，compile/native/资源仍未知，不重投。65已把“plugin测试仅允许canonical，ready工作树在提交后baseHead旧值”的具体入口缺项交原owner；本轮文档正常deliver-task，不新增Cargo身份或绕回root workspace。完整goal active，passed=false、accepted=false，释放0B。

## Managed delivery64：Details 源切片转入受管工作树

2026-10-26 本轮按用户“managed worktree + deliver-task”指令接续。正式 create-worktree 使用原 request `ue-material-graph-20261024-01`、operation `material-worktree-01a16d50-20261026-64`，真实 queue26181 / Jenkins #454 已 SUCCESS。server 实际分配 writer `w1167fba90b`、worktree `w1167fba90b-mat64`、branch `codex/jenkins/w1167fba90b/mat64`；基线为 main 不可变提交 `0569fb6a00049526a1aace835780acaa530dc2e3`。归属精确限定 lib.rs、details_selection.rs、tests/details_selection.rs 及本文四路径。原 canonical 当前为 codex/review/remaining-local-dirty-20261025，未切分支、清理外来内容或手工执行 Git 生命周期；后续本切片源码写入使用实际 ready 工作树。

3 个 Rust 文件沿61已审语义：Details 只借用原作者图/原表达式，保留精确稳定 ID、来源顺序和覆盖 presence；空选/根返回原图目标，stale/歧义非空选拒绝整批，修复中的无效图仍可查看。原161静态定义（160普通+1旧 ignored）不是 harness 发现或运行结果。原 #448 / queue25902 / matgraph-details-01a16d50-20261026-61 在20:55:51Z认证 API 为 FAILURE；491B 原归档 SHA `7d907c538ea013f9dac30df13afb0d117aa9bdddf3ca5335d5c49ad31c318d83` 仍 waiting / management_git_busy，先前保留同一 r19 reader 等待。compile、发现/执行数、native、writer/resource 终结均未知，不写成 false/0，不重投该 operation 或 r10。

本轮仅重基本域3源及58/61/63新增文档段；原基线中全部其它会话、fork Instance/Hierarchy/四 SearchInput、原ZUI、Shared/catalog/theme/runtime 内容保留。组合补丁 `dc73ec24c941f3dd5cc7257af3bd7933aa39f6e9ee8efbf9a41471223e5fe6ab` 的四路径 guard 为0error/0warning/0suppressed，反向检查及 diff-check exit0；Git换行提示原样保留。原61相同Rust的格式/单插件静态检查复用，不扩成全仓构建。工作树创建归档16053188B/SHA `7448852ec7935d41f3724cd9caece8aaccf2be98e4a19b84adeb7dec922aa11b` 在原Home保留；正式HTTP客户端8MiB边界拒绝整份读取的诊断保留，未放宽边界或复制全源归档。新工作树的成功不关闭 retained r19 reader，也不证明旧请求 EOF。

正式交付使用 `deliver-material-details-01a16d50-20261026-64`，仅四原路径、advisory validation=[]；真实 queue/commit/integration 以原 reports/material-worktree-delivery-64 后继回执为准，本文不预签提交或合并成功。Penpot沿63实际 rev222/3906及真实11 source refs；query metadata、source allocation/reset、交互状态 paint、真实宿主/导出/renderer、正常EXE/实际150%DPI/双端1DIP仍open。完整goal active，passed=false、accepted=false。原失败及未知生命周期保留，释放0B。

## Node Search63：消费原Source32公共审阅

收到原public owner的review.json（SHAabec8fb27543055aa282eba10431d1c194b139186287a9898892b653a0c01d0e）及consumer-instance-contract.json（SHA768b241c7f2d91e39c274587eb3df75f8c56fe75f5fb20851c8a988efb0f8515）。这是2026-10-25T17:02Z对归档rev156的审阅，不覆盖后继源或把NodeSearch重新派给fork。73项correction均为原196新ID、3Search/8Tree原Shared身份与三PNG逐图审阅继续保留；原#386 FAILURE不QueryOnly或重投。

2026-10-26T20:38Z隔离无头context实际只读原file/page，当前rev222/3906对象，11目标sourceRef全部有效，Shared rev5/4原main/34colors/3typographies。三个Search仍是Fill384、文字x28/w348、leftright；45的6对象约束修正是已保存设计后继，但原native-layout-allocation依然写240/master-parent，初始化guard不因此取得新资格。math与“不存在的材质节点”两项实际可见内容对应的text-property仍为placeholder，正式源应从真实MaterialGraphNodeSearch::query()产生query投影，原placeholder保持Search。空查询仍用原placeholder，不手改metadata、baseline、callsite或instancePath。

| 差异 | 当前实际与最小接续 |
| --- | --- |
| Query | host从实际保留的search.query()/matches()/selected()读源，经既有SearchField query属性与原projectEditorField投影；不把显示文字反写placeholder、生成第二查询store或由Penpot UUID造调用身份。 |
| 分配/guard | 原240与204文字初始化证据保持；384→窄宽→原默认/reset、实际source/font/main/allocation准入、重开/正式导出仍未执行。45只修原生消费约束，不等于source/reset验收。 |
| Flat rows | 八行当前x8/w368、各隐藏6辅助部分；现有content变体仍保box、x52，在384宽只得320，不能把它当已支持的同配置。公共TS/native同一真实平面动作行profile或现有原语组合交Parent，不造public prop、空icon hack、全局折叠或新master。 |
| Selected paint | 原Multiply的editor.text.selected/body配对保存为真实设计证据；当前sourceColorProperties仍明确拒绝互动状态的normal-token绑定。须保原node/control/instancePath、真实状态、token/library/revision/alpha闭包，不用同白色literal/normal token、剥ref或重签通过。 |
| Host | 原primitive根各0 authored events；真实输入/clear/IME/焦点/创建/一次prepare-permit-CAS/inverse-history及codec仍须原host接线。create_graph不是create_node，不新增event UUID。 |

另记录真实源与画面选择态的差异：MaterialGraphNodeSearch::new在有结果时选首项Output，归档全量六行却都标selected=false；new(math)首选Add，Multiply需要真实select_id/navigation或保留原selection的过滤过程。投影必须消费真实当前selection与独立focus，而非为复刻图片改动领域行为或推断操作已经发生。原set_query、稳定action key和46的plan_material_graph_node_search_creation继续复用，不添加重复getter/API或第二model/history/clock，不引入Editor→leaf依赖倒置。

当前17公共参考有4项字节完全一致、17项规范LF后全部匹配原review，不能用旧字节hash签新CRLF，也没有发现这17项的语义后继。本轮只读Penpot/API，浏览器真实exit0/已关闭、auth不变，未写Rust/原ZUI/Shared/catalog/theme/runtime或fork的Instance/Hierarchy/四Search。只更新本域记录与消费合同，不因这些公共资格open停止独立工作，不新派Cargo/producer。原61 queue25902/#448另链继续，161静态定义不计Rust结果。交付material-node-search-public-review-63-consumer-handoff.json SHA808010b50d0eb426124bc69a50e0b22ff63d8f60f31fb67304f111d952b43231，live-readback保留当前11实例数据。goal active，official export/reset/native host/正常EXE/150%DPI/双端1DIP未通过，整族passed=false、accepted=false，释放0B。

## Material Details61：真实选择源到属性目标的领域投影

2026-10-26T20:29:35Z 实际受管后继已替代下文初始“将登记”的时点：原request ue-material-graph-20261024-01 / 新operation matgraph-details-01a16d50-20261026-61，identity SHA126b0888715ebc450295f8c387d8faaeb920ec1b8c3da1c5df636b502bd1abc5，29owned+27deps。正式客户端exit0，queue25902→management#448；input d2638ae2fe29a33d7d8f1f7c6e31c90e054a779b172b2a47bde5cee5a171298d。认证API为building=true/result=null，542B归档SHA48ea4e122160eaaf01b9cb9784385b70e42c9e236f9110489383729da1801353仍waiting/checkout_source_busy，blockingReadOwner仍原r19 management-0e3e11c3d922cbeccad3eb823d0e2c15172b188027aa2ff3113d7dff6cdc1dea。编译、实际发现/执行、原native/资源继续unknown，不把161静态定义或client exit0算通过，也不把该门前source-busy称budget/fair_queue拒绝。严格nested单包、features[]/defaultsoff、locked/lib/profiletest/jobs1/material_graph_/threads1/E物理根配方保持。source57+61完整后继已透明提交；本段为其后已知本域文档增量，原identity/payload不改。原56/#420及r10/#370不重放，#448沿本次唯一producer原键跟进。具体回执material-details-managed-61-client-handoff.json及build-observation-02.json。

UE阴影补充62已精确核对：SNodePanel.h:647–650选择原Brush，SGraphNode.cpp:1821–1836绘制同一节点的阴影；Geometry.h:407–414为size+2×delta及offset=-delta，child scale不变。Starship默认delta=(12,12)，即每侧12 Slate单位/总宽高+24；18/64是纹理切片比例，不是18DIP膨胀。当前蓝色2px描边近似保持并继续open，实际纹理alpha/颜色未取得，不用它声称九切片一致。精确五源hash、同代际paint/clip/一次DPI与独立hit合同在material-selection-shadow-62-source-contract.json（SHA33729b6a7b72fa442ae6c14ee15c4108c6593324baddab3ac4684ed5ed332350）；只向Parent/public owner交付提案，未写公共主题/painter或本族设计。

沿设计58确认的UE差异，仅在本域lib.rs的模块/重导出结构接线，新增details_selection.rs与tests/details_selection.rs。project_material_graph_details_selection返回私有构造的MaterialGraphDetailsSelection，targets只借用原MaterialGraphAsset或原表达式node，selected_nodes另保留每个原根/表达式的源身份。未复制作者图、从编译后MaterialAsset反建作者文档、创建Function模型或第二selection store/history/clock。

按MaterialEditor.cpp:8708–8754，空选择或Output根投影到Graph属性目标，普通表达式投影到实际node；混合选择保留两种目标，has_expression对应UE普通节点标志。Graph在此明确是原作者图，不是声称完整UE Material属性对象；实际Toolkit必须用同一真实document上下文解析它。多个坏图Output保留每个原root身份但同一Graph目标只列一次，普通合法图仍只有一个根。目标以源顺序稳定呈现，不以输入selection顺序、显示名或重复参数名替换原node key。

非空选择的空/重复/stale key、任何歧义/空source ID整体拒绝，不能先发布部分Details再回落到别的材质。空选择仍允许在ID有错时检查原文档。required-input、错误连线、循环、非有限或错型参数不成为查看修复属性的编译门；值与原诊断均保留，不生成成功默认。中文、长ID、大小写、带空白的精确合法key完整保留。API不包含focus权限、运行状态、revision计数或可编辑快照；原host发布前仍须比对view incarnation/document/subdocument/source revision/selection request，编辑继续一次prepare/permit/CAS与原primary inverse-history。

新增8项回归候选覆盖坏ID文档的空选择、不可编译根仍检查Graph、同名参数按真实node与源顺序、混合/多根不丢身份、stale/重复/空选择的原子拒绝、未选中source的歧义、中文长key及NaN/错型值保留，以及真实TOML重开后shader/显式默认/orphan -0.0存在性与现CPU PBR结果不变。当前material_graph_过滤器有161静态定义＝160普通+1旧ignored，仍未Rust类型检查、发现或执行。初次把rustfmt --emit stdout的文件名标题误写入两份新文件，语法检查拦截；只删除标题/空行后重跑，原错误源码补丁与失败记录保留，不算Cargo失败/通过，不改断言或绕required-input。

当前三路径rustfmt、scoped diff-check、补丁反向检查均exit0；最终增量patch SHA461035cb7afe65ae760008ec0755af776a92caad49501b91650fb4d8bb72bc94，guard 0errors/0warnings/0suppressed。官方单材质plugin validate exit0、diagnostics=[]；46项只读参考与1项UE source在本轮检查时全部保持，本文是其后已知文档增量。最终以material-details-selection-61-source-r2.json/validation-r2.json为当前，原source.json/validation.json保存首轮formatter transport失败。公共Graph/host/Runtime/manifest/锁、原ZUI、Shared/catalog/theme与fork的Instance/Hierarchy均未写。

本轮复用设计58已关闭/新context重开的三板九次pointer与三PNG，不以helper修复源码冒充已经更新Details画面。原56/#420真实FAILURE与r19 source-busy/native资源未知独立保留。后继验证将由原Node/Graph唯一producer在原request ue-material-graph-20261024-01下，透明登记当前完整source57+61及有限recipe；不复用终结56/r10操作，不要求全局停写，不把历史未知变成独立准备的普遍门。正常Material/Instance/Function Toolkit、真实Details/focus、作者事务/renderer、官方导出→原资产、正常EXE/150%DPI/双端≤1DIP仍open。goal active，整族passed=false、accepted=false，释放0B。

## Material selection design58：根节点选中原型与托管56真实终态

2026-10-26在原 Material file b4a1e3ce-9e6a-81fe-8008-d69d6092d704 / page ...d705，从实际rev212/3449对象追加三张独立业务板，最终rev222/3906对象/原5业务main。1280×800板f147d768-b2cf-8022-8008-d9ae323dae26，900×620板c6260bd3-1397-80a5-8008-d9ae58f54a67，640×520板cb46e97a-7efe-8085-8008-d9ae7bafd8c3。原生Duplicate和X Inspector创建/排布，457个新增对象；原对象仅页面顺序与3个本域默认Output的navigate发生预期变化，删除0、非目标变化0。fork30/32/59的Instance、Hierarchy和四SearchInput子树保持；未整页重排或写Shared。

每板保留本域Output实例component d7173cd6-1da7-4897-be89-08583d32f598 / main shape926a240d-18dd-4741-9cdc-8ce08292e721，以及原3 Button+1 SearchInput的Shared身份。每板33处既有流式布局、所有sourceRef有效，没有detach或复制main。通过原生描边Inspector产生stroke-group覆盖，再以真实Transit UUID/枚举协议引用已发布editor.surface.selected：5d5e9ce2-753a-5a57-a1fc-91829199dbcd / Shared ...aac12 / #0070e0。Shared当前实际rev5/4main/34colors/3typographies；本轮仅消费颜色，不据此声称完整样式继承或跨文件A/B通过。

保存context实际关闭后，新context读回12个目标hash与28个保护子对象一致。另一次新context按尺寸重新加载实际Viewer，真实指针逐一完成默认Output→选中板、点选中节点保留选择、点画布空白→默认板，共9次已验证导航；三张PNG尺寸准确且已逐图打开。首轮普通JSON请求HTTP500、首轮重复/隐藏SVG定位失败均保留；修正只涉及真实类型编码和可见Viewer定位，没有放宽源保护。reopen-r2的renderedStrokes只查到交互overlay的显式stroke属性，不能据它认定视觉描边宽度；2为服务器实际原生stroke值，蓝色轮廓另有PNG复核。

这只完成可编辑的选择原型。UE SGraphNode.cpp:1821–1836实际使用Graph.Node.ShadowSelected与膨胀paint geometry，StarshipStyle.cpp:4062为RegularNode_shadow_selected纹理/18/64边距；当前平面2px全局选中色不能认定该阴影或≤1DIP复刻完成。UE MaterialEditor.cpp:8708–8754明确空选择与根选择显示Material（Function上下文则Function）并FocusDetailsPanel；当前三板仍留空Details提示，这是已证差异，不冒充正确业务Details。三个尺寸页签均有硬裁切，900/640预览提示换行；根节点与五个输入完整可见。预览/Compiler Results仍未连接真实引擎，未添加根Delete、编译成功、焦点或Undo的假交互。

原唯一producer的matgraph-parameters-01a16d50-20261025-56 / queue20964 / management#420已是实际FAILURE，2026-10-25T18:37:21.573Z结束；2026-10-26T19:12认证GET及终态日志取代先前building观察。545B原归档SHA245dc21f24692767822d27a7ce6568f4bab43dffd49033946ee1eff04cf7878b仍waiting/checkout_source_busy，blocking owner为原r19 management-0e3e11c3d922cbeccad3eb823d0e2c15172b188027aa2ff3113d7dff6cdc1dea。实际保留的208条回复为206 source_busy+2 mutex_busy，首块日志仅保部分文本，不能称已统计全240条。input e80093df…、identity697d646f…及全部旧失败保持，停止例行终态轮询；compileStarted、Rust发现/执行、原native/资源收口仍unknown，不借r19自身Cargo EOF释放reader/pool/reservation，也不重投已终结56或r10。

source57三个Rust当前字节为CRLF，但规范化LF后逐一等于原57声明；来源UNKNOWN，未回写/回滚源码，也不以旧LF guard签当前字节。153静态定义仍是152普通+1旧ignored，尚无实际Rust结果。具体读回见material-selection-design-58-source-line-ending-readback.json；fork的独立Hierarchy后继#432另链保留，不合并其输入。官方导出→原material_workspace.zui仍未执行，原v2/29nodes/27controls/13events/8widget imports与SHA5856c886…保持。真实Toolkit/Document/一次prepare-permit-CAS/history、选择→Details/focus、材质修改进入renderer、正常EXE/实际150% DPI/双端≤1DIP继续open。

当前证据：[设计闭包与逐图记录](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-selection-design-58-final.json)、[新context九次指针验证](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-selection-design-58-reopen-r2.json)、[56真实终态](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-parameters-managed-56-terminal-summary-04.json)。本轮无Cargo/重投/桌面/服务/State/锁写；自有浏览器已关闭、auth哈希不变、登记tmp为空，释放0B。goal active，passed=false、accepted=false；公共缺件不阻止后续域逻辑。

## 材质作者语义57：根节点保护与精确删除候选

按 UE MaterialEditor.cpp:6464–6658 / MaterialGraphNode_Root.h:27–28 接续删除行为。UE 的 CanDeleteNodes 要求选择中至少一个可删节点；DeleteNodesInternal 跳过不可删根节点，先断开待删节点全部连线；一次 FScopedTransaction 后清理选择、恢复被删除表达式的预览并刷新材质。本轮只改本域 lib.rs 的结构声明/重导出，新增 node_deletion.rs 与 tests/node_deletion.rs，不写公共 Graph/commands/model/retention、Runtime、原 ZUI、Shared/catalog/theme 或 fork30 的 Instance/Hierarchy/四 SearchInput。

新增 plan_material_graph_node_deletion 与私有构造的 MaterialGraphNodeDeletionPlan。实际源图、待删节点/关联连线和保护根均保持借用，返回 source 顺序与原 occurrence index；重复边和自环按各自原发生位置完整保留，选择两个相连节点不把同一边删两次。不克隆作者图、不生成 ID、不改变参数 map。空选择与仅 Output 选择为 can_delete=false；混合选择保留 Output、只返回真实可删表达式。当前六种枚举用穷尽 match，后继 schema 必须重新审阅删除语义，不把 Function/复合图当现有支持。

空、重复、失效 selection key 及歧义/空 source ID 整体拒绝，不发布部分候选。节点 key 的大小写、空白、中文与长文本保持精确，显示名不充当身份。编译错误、循环、无效 pin 或 dangling edge 不阻止删除一个表达式以修复图；与待删节点无关的连线/诊断保持原样。默认相同、-0.0、全零向量与孤立 override 都保留，避免删除一个同名声明时误清另一个声明仍使用的作者覆盖。

所有 index 只属于同一源借用/文档 revision，不是稳定 edge UUID 或事务许可。真实宿主仍须保有 asset/document/graph/subdocument、revision/retention/source，完成一次 prepare/permit/CAS、一次原 primary-history inverse、取消、选择/focus、预览表达式失效与 renderer 更新。本轮测试中的 fixture 克隆、删除及逆向插回仅验证候选数据足以恢复真实 TOML/现 PBR compiler 输出，不是正常 EXE 的 Delete/Undo 或第二生产 history。

新增9项有意义回归候选：空/仅根 no-op，混合选择/源顺序/原引用，过期/重复/空选择的原子拒绝，未选节点也有歧义 ID 的拒绝，精确 key，重复边/自环/双端选择/dangling 边，允许修复且保留非有限值诊断，同名与孤立覆盖 presence，以及真实 TOML 重开→保持 base_color 的删除候选→逆数据恢复原序列和 roughness 编译结果。人工审阅修正了首稿缺 base_color 的测试输入，未绕过 required-input 校验或计作 Rust 失败/通过。当前153静态定义＝152普通＋1旧 ignored，尚无本轮 Rust 类型检查、发现或执行。三路径 rustfmt、精确 diff-check、补丁反向检查均exit0；guard 0errors/0warnings/0suppressed，官方单插件 validate diagnostics=[]。51项只读参考和2项 UE 哈希在源检查时全匹配；本节是其后已知文档增量。

源码修订57补丁 SHA8ec3641061e81f28147f62292585427d4a20689a8d2f127c296498ac8aa912c3，证据统一 material-node-deletion-57-*。正在执行的原合法后继56为 matgraph-parameters-01a16d50-20261025-56 / queue20964 / management#420；17:55:47Z真实545B归档 SHA245dc21f…为 waiting/checkout_source_busy，blockingReadOwner management-0e3e11c3d922cbeccad3eb823d0e2c15172b188027aa2ff3113d7dff6cdc1dea。其原 identity697d646f…/inpute80093df…和25owned+27dependency不改；本轮 lib、两新文件与本节是透明的后继源修订，不能把153定义塞入旧payload或据旧144声明称当前整组已验证。任何实际56执行必须按其真实捕获闭包与测试发现结果审阅。原51/#414 FAILURE以及全部旧失败保留，原 native/resource 未明；本轮未追加第二并行 producer、重投终态或清锁。

本轮未改变布局，复用设计54的真实三尺寸/九图及其裁剪差异，不制造删除成功的假预览。正常宿主 Delete/Undo/焦点、官方导出→原资产、材质修改进入真实 renderer、正常 EXE/实际150% DPI/双端≤1DIP与整族 passed/accepted 继续 open。goal active，自有静态检查进程均退出，无 tmp/fixture 生成，释放0B。

## 材质作者语义55：真实参数声明、覆盖 presence 与诊断保留

在设计54空状态之后，当前只改本域 leaf 的 lib.rs / graph_presentation.rs，新增 parameters.rs / tests/parameters.rs。MaterialGraphPresentation::parameter_declarations() 迭代实际 ScalarParameter 与 VectorParameter，保留 source 顺序、原 node/name/override 引用，默认值仅按已有 MaterialGraphParameterAsset 拷贝固定数值。没有复制作者图、第二参数 store、时钟、history、ID 分配或修改权限；原 Document/revision/retention/一次 permit/CAS 仍由真实宿主负责。

name 是原精确 schema key，大小写、空白、中文、长名均保留，不拿显示名做身份或替 UI 猜重命名映射。重复名称仍保留不同 node/default，未擅自折叠成单个组或选择任意默认值；孤立 override map key 不生成假参数声明。override_value() 保留本地 presence，包括等于默认、-0.0 与全零向量；effective_value() 只在类型一致时消费原 override，错型返回 None 并保留原错值供修复，不悄悄显示成功默认。NaN/Inf 也完整保留，必须继续消费现有完整 validator；有效覆盖不能抹去非有限默认值的错误，投影结果不代表 compile/readiness 通过。

该 API 包含未连线表达式，是作者声明投影与节点 Details 的数据入口，不是 UE 已编译可见 Parameters 列表。UE 原 CreateGroupsWidget 依赖 ParameterGroups、association、ExpressionId、真实 PropertyRowGenerator 与 preview 编译结果；当前 graph schema 无 group/sort/association 元数据，宿主须有真实 visibility/author metadata 后再填对应页面，不能以声明计数替代 UE 的“连接参数”判定。真实 Property 行输入、重置事务、组展开、Save Child、焦点与 source locator 跳转及 MAT-INSTANCE-01 参数 ABI 仍 open。

新增8项回归候选覆盖：孤立 override 不造行、原节点/名称借用与未连线默认值、默认相同/-0.0/全零 override 的真实 TOML presence、错型值与诊断保留、重复名称/混合类型/不同默认、大小写/空白/中文与重命名后不偷 remap、真实 TOML 重开→现 PBR compiler 的 roughness/base_color→删除 override 后恢复默认，以及非有限值仍可见且错误不被抹除。最后两项使用真实现行 codec/compiler，不是 renderer 或实例作者持久化的通过证据。当前144静态定义＝143普通＋1原ignored；未 Rust 类型检查、发现或执行，旧128/136和全部原 payload 按各自时点保留。

四 Rust 路径 rustfmt edition2021、精确增量 diff-check、补丁反向检查与 change-guard 均 exit0，guard 0errors/0warnings/0suppressed；单插件官方 validate material_editor --form dist --json exit0、diagnostics=[]。Git 的 LF→CRLF 提示保留，实际两个既有文件均继续 LF，补丁只有新增段落。最初校验 orchestration 遇 Windows 命令行过长、独立 stdin 提前关闭，检查进程未开始；随后仅在内存生成精确最小补丁再跑正式静态检查，没有落整份源备份或临时 helper，失败诊断保留。14项只读参考及设计54文档在本轮源检查时哈希全匹配；本节是其后已知文档增量，不用旧 guard 签当前新文档。

源补丁 SHA724b25a4f4c4af748633ed96cc6fde3aea58edbf6887a7602e5a8c3e790b1a9a，证据统一 material-parameter-declarations-55-*。本轮未写 Runtime、公共 Graph/model/commands、manifest/锁、原 ZUI、Shared/catalog/tokens/主题或 fork30 Instance/Hierarchy/四 SearchInput。本轮复用设计54已保存三尺寸/九图，未生成非空假参数预览。原 #414 已 FAILURE，未借旧 operation 换源；当前源修订将由本唯一 producer 在原 request 下公开合法新后继，经正式 source/writer/resource 准入执行，沿 nested 单材质包/defaults off/features[]/lib/test/material_graph_/jobs1/threads1/locked/物理E范围。goal active，EXE/renderer/正式导出/实际150% DPI/双端≤1DIP及整族 passed/accepted 继续 open，释放0B。

## 材质设计54：Parameters 空状态与三页签原生往返

本轮在原 Material file b4a1e3ce-9e6a-81fe-8008-d69d6092d704 / page ...d705 接续 rev183，真实保存为 rev191 / 3401对象 / 5业务main。新增三张 Parameters 业务板：ef8f8ae6-3f20-8069-8008-d845d72fe586（1280×800）、6b38ed4b-c968-80ec-8008-d845e58ca321（900×620）、7500229c-a015-80d0-8008-d845f481f5ce（640×520）。先用原生 Duplicate 复制本域默认板，再用原生 X Inspector 排入独立列，最终436个新增对象；原对象仅页面顺序和六个本域 Parameters allocation 的导航发生预期变化，原对象删除0、非目标变化0，fork30四根及子树完整保留。

UE 依据为 SMaterialParametersOverviewWidget.cpp:467–556。没有可显示参数时实际使用 Padding(10)、HAlign(Center) 的 SBox 与“Connect a parameter to see it here.”，不是空白树或 Details 搜索框；因此只删新副本的21个 SearchInput 对象，保留并重用原文字对象、Shared body/strong/color 及流式布局，不生成假参数组或 Save Child 按钮。能力勘误：save.json 的 emptySource.scope 错称 schema 没有参数表达式；实际 authoring.rs:328 已有带 name/default 的 ScalarParameter 和 VectorParameter。本次设计仅含未接线 Output，所以参数数为0；这不证明生产文档的参数列表已经接入。原错误说明保留，final-closure-r2.json 明确更正；组/顺序/展开、真实值编辑、保存子实例及真实节点跳转仍 open。

新增提示采用原生字体测量的 auto-width，实际文字行宽208.984375逻辑像素、高16，容器保持10逻辑像素 padding/水平居中，没有按截图逐尺寸补坐标。1280下完整；900容器173.8宽、左右各约17.6溢出，640容器121.8宽、左右各约43.6溢出，实际图已出现裁切。窄窗 Parameters 与 Nodes Overview 页签也硬裁切，不能把 native navigate 可点当作 UE Ellipsis/tooltip、键盘或焦点通过。原生 X Inspector 的首轮旧 CSS 选择器失败及初次错误源路径 ENOENT 均单独保留；修正后读回实际 Inspector 值和唯一原始 material_workspace.zui 路径，没有放宽验证或改公共源。

保存 workspace context 已实际关闭；三个独立新 context 均直接重开 Parameters，再实际点击 Details→Parameters→Nodes Overview→Parameters→Details。九张 PNG 实际为1280×800、900×620、640×520；三张 Parameters 已实际打开逐图复核，另外六张与已复核的设计52截图逐字节 SHA 相同并复用其审阅。仅为 Penpot 原生原型，未证明编辑器宿主页签/选择/焦点/撤销、正式 source→ZUI 导出或 renderer。

三张新板各保留3个 Shared Button 实例及1个本域 Output 实例；75处 shapeRef 和61处实际颜色/字体引用均解析到真实源。Shared 此刻 rev5 / 4main / 34colors / 3typographies，本轮未写 Shared/catalog/tokens/主题。4个 source53 Rust 与4个只读参考及原始 ZUI 的9项当前指纹全匹配；原 ZUI 仍13809B / SHA5856c88655aafcae3b8f8af2d4ed7786ed89e61551610324faa51ce767af0c12。官方单插件 validate material_editor --form dist --json 实际exit0、diagnostics=[]，不是 Rust 或产品通过。136＝135普通＋1原ignored仍仅静态测试定义；原 #414/queue19891 的 FAILURE、207 mutex＋33 checkout_source_busy、native/资源未知独立保留，本轮未复投任何终态 operation。

已消费 publication-gap-current-reconciliation-r03/result.json（SHAfe876bebb12b1bda87886ffb92dc4bd34a971fbffb7beee7f6a576e9a650a4a7），旧 b1 接收配置发布空档已解除，不继续等待 ACK。正式 runner 当前编辑分支仍限定 AI 文件，Material 未有已编译验证的 spacing-edit/官方资产根；没有借 AI flag 盲导入、生成本地主件或重签业务板 metadata，19002/服务均未操作。正式 codec/export→原资产、真实 Material/Instance/Function 作者宿主、引擎渲染及正常 EXE/实际150% DPI/双端≤1DIP仍 open，goal active、整族 passed=false/accepted=false。

同时修正本域文档章节插入位置：此前52/53的 LF 插入误落在51首段之后；仅按两份已登记补丁的唯一完整插入字节抽取并移回顶层。移除二者后完整字节恰好恢复到51原 SHA1c85057f54ea19345d1c0bf2d976b3fde5c3b8da4d76b807af25295b940a8055，再按 CRLF/LF 兼容的真实段落边界插入54/53/52，保留所有旧内容与 fork 小节。证据统一 material-parameters-54-*。五个自有 Node 浏览器进程均实际退出，browser closed、认证未改，登记 tmp 为空；释放0B，失败证据与必要图片保留。


## 材质作者语义53：Overview 直接投影真实纹理表达式

在设计52的已保存三尺寸空表基础上，本轮只改材质 leaf 的 lib.rs、graph_presentation.rs，并新增 overview.rs / tests/overview.rs。MaterialGraphPresentation::texture_overview(display_name, compare_names) 从同一借用的 MaterialGraphAsset 节点建立只读用量行；行保留原 AssetReference 与每个真实节点引用，reference_count 是实际 TextureSample 表达式数，包含未连线表达式，不将 shader 引用或输出连线数混入。空图返回空行，不生成占位纹理、不查假注册表。

分组使用完整 UUID＋locator 相等，不按显示名合并，也不把同 UUID 不同地址/子资源的未验证别名擅自折叠。名称由实际宿主提供，每个不同引用仅调用一次；空名称、中文和长名称完整保留。当前 schema 没有 Material Function 节点，因此函数递归用量仍不支持，不能按一张只读图关闭 Function 缺口；引用存在和 ResourceKind/加载状态仍由真实 registry resolver 验证，名称闭包不等于已加载。

排序先按实际引用计数升序。进一步核对 UnrealString.h.inl:873 发现 UE FString 的名称比较实际调用 Stricmp；因此显式消费宿主的一致名称比较器，不把 Rust 默认大小写敏感字典序当 UE 等价实现。相同 count/name 时稳定保留首个原表达式顺序，HashMap 只做索引不决定输出顺序。Details 目标返回原 source 顺序中最后一个匹配表达式，符合 UE Overview OnMouseButtonClick 的选择规则；全部节点引用仍提供给宿主作真实定位。返回值不带文档授权，宿主须核同 document/subdocument/revision 与当前 SourceLocator 后，才可触发 Details、选择或焦点；本轮没有第二 store/model/history、复制作者图、分配 ID、命令 permit 或 renderer 接线。

新增8项有意义回归定义覆盖零纹理/无 resolver 调用、未连线表达式计数、升序/名称排序、同名不同 UUID 与未验证 alias/子资源、Details 原节点及最后 occurrence、错误类型图仍可见且诊断不抹除、中文真实 TOML 保存重开→现 PBR 编译器保留 base_color_texture、显示改名只重排以及宿主忽略大小写比较与稳定 tie。它们消费现有实际 DTO/codec/compiler，不是 UI/事务/渲染证明。当前 literal cfg(test) 接线的 material_graph_ 为136静态定义＝135普通＋1原 ignored，尚无 Rust 类型检查、发现或执行结果；原 source49 的128仍留其时点，不覆盖旧 #414 payload。

四路径 rustfmt（实际 workspace edition2021）、diff-check、精确增量补丁反向检查和 change-guard 均 exit0；guard 为0errors/0warnings/0suppressed。首次错误使用 edition2024 的格式检查与随后新测试折行修正保留在 validation 记录，没有全文件调整既有导入顺序。官方单插件 python -B -m tools.export plugin validate material_editor --form dist --json 实际 exit0、diagnostics=[]；这是静态插件契约检查，不是 Cargo、插件动态加载或产品通过。原51的46项声明只有 lib.rs/graph_presentation.rs 两项为本轮透明后继，另外44项当前哈希相同；新增2源不塞入旧身份。

原 request ue-material-graph-20261024-01 的最新实际 managed operation51 / queue19891 / #414 已 FAILURE，240回复207management_git_busy＋33checkout_source_busy，原 archive 仍 waiting；native/reader/资源与实际 Rust 数未知，不重投旧操作、不手改锁。当前53是可审阅源修订，尚未创建或派发新测试 operation；待正式 source/writer/resource 准入后，由本唯一 producer 固定当时完整源并公开合法后继。后续范围保持 nested zircon_plugins/Cargo.toml / 唯一 zircon_plugin_material_editor_editor / defaults off/features[] / --locked --lib --profile test --jobs 1 material_graph_ -- --test-threads 1 / 物理 E:/cargo-targets，传递 Runtime/native-host 与 Editor/target-editor-host；不回退 root 或本地 Cargo。

证据统一 material-texture-overview-53-*，源补丁 SHA a08e2dcc7b76d9d8ab53606e0e158823f8604b245e768458cae99c4cb95a6c87；原始 ZUI、公共 Graph、Runtime、锁/manifest、Shared/catalog/tokens/主题及 fork30 资产均未写。本轮复用同一 source 空状态的设计52保存/关闭/重开和9图复核，不伪造新的非空生产截图。正式 source→UI 行绑定/单击/焦点、codec/export→原资产、renderer、正常 EXE/150% DPI与双端≤1DIP仍 open；goal active，整族 passed=false、accepted=false，释放0B。

## 材质设计52：真实 Overview 空状态与三尺寸原生往返

2026-10-25T16:37Z 实际只读复核原 Material file b4a1e3ce-9e6a-81fe-8008-d69d6092d704 / page ...d705 为 rev183、2965对象、5业务main。接续 rev175，通过 Penpot 原生重复三张本域默认板后，只在新副本落下 Textures / Name / Reference Count 空表及流式标题/列容器；451个新增对象，未新增 main。新增板分别是 7f57dee9-2255-80f5-8008-d839d64eba2f（1280×800）、fa96d4d6-a569-80a8-8008-d83c743e201b（900×620）、d99f3fa8-8edf-8005-8008-d83c84c5079e（640×520）。未修改 fork30 的 Instance/Hierarchy/四 SearchInput copies；四受保护根及子树全部与 rev175 指纹相同。

源码依据是 UE SMaterialExpressionsOverviewWidget.cpp:67–182：该页实际展示纹理引用计数，并非通用节点清单；零 TextureSample 的作者状态应保留空行，不填测试纹理。实际比较器 CountA < CountB，同计数再按名称，不把源码注释的 descending 当成真实排序。当前板只对应未接线 Output 的设计状态；没有消费生产作者文档、纹理 resolver 或 Material Function 递归，因此非空纹理计数、行选中→真实 Details/图定位继续 open，不能用名称归并取代资源身份。

三尺寸均已实际保存、关闭保存 context、在新 context 直接重开；原 Details 板点击 Nodes Overview 的真实 allocation 可进入对应新板，再点 Details 返回原板。它是 Penpot 原生 navigate 原型，不是编辑器页签、焦点、撤销或 Dock 完成。9张截图逐图记录：5张已实际打开；另外4张通过与已打开图片的逐字节 SHA 相同复用复核。最终 PNG 是实际1280×800、900×620、640×520。1280 表头完整；900/640 的 Reference Count 与三尺寸 Nodes Overview 页签仍硬裁切，UE Ellipsis/tooltip、键盘、焦点返回和滚动未通过，未以删字或假数据关闭差异。预览/编译仍未接 renderer，不把 Compile material to preview 或 Not compiled 当真实引擎能力。

当前三新板各保留3个 Shared Button 实例和1个本域 Output 实例，共75处 sourceRef、58处真实样式引用全部解析到实际源。Shared 仍 rev5 / 4主件 / 34色 / 3字体；本轮未写 Shared、catalog、tokens 或主题。原生重复回执 nativeSharedCopies 字段名覆盖了本域 Output，最初每板5个实际是4 Shared＋1本域；删去新副本的 Details SearchInput 后各4个实际为3 Shared＋1本域，不能将这个字段报成全部 Shared。rev175 原对象只有页面根顺序及三个自有 Overview allocation 的 navigate 交互发生预期变化；无非目标变化或原对象删除。

本轮消费 parent-lineage-version-merge-r04 原件 SHA d755597d422800bd16c4ef16d50ec07802528528bc4401a71343f4b18bba7f89，不重查其旧任务。#386 / #391 的 building 观察均保留为历史；#391 已在10:18:28.090Z FAILURE，240回复为232 management_git_busy＋8明确 r19 checkout_source_busy；#386 为200＋40。撤回以错误 state/management/management-git.lock 推断无 holder，实际锁是 state/locks/management-git.lock；旧件保留。r19 Cargo 自身 EOF 不能替其 unknown reader、负向 source 连续性及 active pool/reservation 作 CAS 收口。

原合法后继 matgraph-frame-bounds-01a16d50-20261025-51 / queue19891 / #414 已实际终态 FAILURE；16:36:59.087134Z 认证 GET 读回 building=false，duration2399945ms，归档仍 waiting/management_git_busy。完整 console 的240个该 operation 响应为207 management_git_busy＋33 checkout_source_busy，不能把全过程归为互斥锁，也不把界面失败推成预算拒绝或释放证明。compileStarted、Rust发现/执行及 native/root/children/EOF、资源终结继续 unknown；128项仍只是静态定义。本轮没有重投 #414、#391、#386 或 r10，没有本地 Cargo、QueryOnly、State/锁/gate/服务写入。

证据统一 material-overview-52-*，最新受管观察为 material-frame-bounds-51-build-observation-04.json（SHA c89205e5168c99d3b2df64bc3e0ef52165e6d06158570e38111208c5d6d4efb5），全部在原 reports 根。四个自有 Node 浏览器进程 PID21928/12388/39172/32484 实际 exit0、browser closed、认证文件未变；登记 tmp 为空，释放0B。正式 codec/export→原 ZUI、真实 Document/history/renderer、正常 EXE/150% DPI 与双端≤1DIP均 open；goal active，整族 passed=false、accepted=false。
## 材质托管验证51：source49 的合法后继已受理

原 request ue-material-graph-20261024-01 下，唯一 producer 01a16d50 通过正式 tools.dev.local_cargo 新建 operation matgraph-frame-bounds-01a16d50-20261025-51，queue19891→zircon-management#414。identity SHA9f7dffb2d4ab9b999ba376f4d46059eceeef7381c8c4984d862903eb66c70668，inputDigest8d59418d2fc915f4e77c45c1c5faeeef783bcef06aee208dda25d47eb517af78。这是 source49 实测 bounds 适配的透明新后继，不复用已 FAILURE 的 #410 或 r10/#370，不抹旧 native/resource 未知。

当前21 owned＋25 dependency固定哈希，较 source47 仅 lib.rs/graph_presentation.rs 两项有已审后继，新增 tests/graph_bounds.rs 一项，25依赖全相同。范围严格为独立 zircon_plugins/Cargo.toml / zircon_plugin_material_editor_editor / defaults off/features[] / lib / 显式profile test / --locked / --jobs 1 / material_graph_ / --test-threads 1，物理 E:/cargo-targets；传递 Runtime/native-host 与 Editor/target-editor-host。128项仅为127普通＋1原ignored静态定义，未冒充Rust发现或执行。

正式 entry 客户端PID32468与orchestrator31356实际exit0，stdout/stderr已关闭。15:58:54.459256Z认证 GET将queue绑定#414，building=true/result=null；原归档receipt为waiting/management_git_busy。compileStarted、实际测试数/结果、native/root/children/EOF及resource闭合仍unknown，不能以收件或管理JUnit报通过。首次 observation01 错用不附带认证头的 opener，API403；已保留并在 observation02 用原 public Transport._request_with_headers 正确认证200，不能把前者当server授权失败。

delivery.json SHA43c428ec600fdddbaff15de46178c14dcf78b1a09031418b1c1270219fd9b730，build-observation-02.json SHA42146a6b0021f7b400569afeb38a1c3fd445560958ebcd9242aba62c3b0834d1，均在原 reports 根 material-frame-bounds-51-*。原Jenkins owner已获实际operation/queue/build的具体handoff，不新建第二producer、不手改锁/State/gate、不用本地Cargo替代；当前等待其真实阶段或终态变化，独立设计继续。goal active；Rust、产品EXE/renderer、正式导出和整族验收未通过。

## 材质设计50：原生文字缓存修复与 UE 页签名称勘误

本轮沿原 Material 文件 b4a1e3ce-9e6a-81fe-8008-d69d6092d704 / page ...d705，仅操作三张默认板既有文字对象（1d1be4b2-d611-4922-bcda-d7db72e2a6cf、311517cc-aa44-4f05-b2a8-cd8c8ef6f0d7、2f3d3ff6-40c3-4888-936d-f860b8a7b29b）。当前实际 rev175 / 2514对象 / 5业务main；fork30 Instance/Hierarchy/四 SearchInput copies、Shared/public 源和原 ZUI均未写入，全部非目标对象指纹保持。

重要勘误：我先将 RegisterTabSpawner 的菜单显示名 Overview（MaterialEditor.cpp:617）误当可见页签标题，把三标题改短。随后核对 SpawnTab_MaterialExpressionsOverview 的 SDockTab.Label（:6014），实际标题是 Nodes Overview；现已恢复完整正确标题，不以删减名称掩盖窄窗裁切。错误的中间版本、首轮文档补丁/guard 和截图均保留为历史，不能据旧 guard 或中间 Overview 图声称 UE 复刻通过。SDockingTabWell.cpp:306–342 证实页签均分宽度，SDockTab.h:113 默认 Ellipsis；不应改成不同宽度或另造菜单来掩盖差异。

这次实际修复验证了文字缓存链：仅更新 content 到 rev171 后，三个新 context 仍绘制旧 position-data。按 Penpot container/set-shape-attr 的 nil→dissoc 契约仅清除三目标的派生缓存，再由 workspace text-changes-renderer / calc-position-data 原生测量并保存，rev173 的 Overview 与后续 rev174→175 恢复的 Nodes Overview 都在实际新 context 重开出现。没有手造缓存、字体宽度或文字坐标。最终保存 context 明确关闭后，三个独立 context 读回 Nodes Overview，同一实际字宽96.40625逻辑像素；PNG实际尺寸分别1280×800、900×620、640×520。

12张过程图逐图复核：首轮3张实际是1600×1050的未稳定查看器捕获，不能作目标尺寸验收；中间3张错误短标题图与完全关闭浏览器后的3张重开图逐字节相同，复用同源视觉复核；最终3张恢复正确标题图已实际打开。三个尺寸仍存在裁切，缺 UE 的原生省略号/完整标题 tooltip 及真实 Dock 行为，均 open。原预览和编译反馈未接 renderer，不能把图面当运行时预览或编译成功。精确公共提案仅交 Parent 已有 tab/text primitive 的 overflow/tooltip 合同，不改公共源或伪造组件 UUID。

证据统一为 material-tab-caption-50-*，最终以 reference-correction-* 为当前，reflow/closed-* 为中间历史。自有 Node PID31364、33636、24964、7956均已实际 exit0，浏览器明确关闭、认证文件哈希不变、登记 tmp 为空，释放0B。本轮设计修订未运行Cargo；source49的128项静态定义仍不等于Rust发现/执行。原 shader-contract47 / queue19353 / #410已实际 FAILURE，归档仍 waiting/management_git_busy，compile/native/resource unknown，不重投旧operation。#391的终态与实际锁路径勘误沿 consumption49 保留，不依据 Cargo EOF 释放r19 reader/pool/reservation。

正式 codec/export→原 ZUI、真实作者document/history/焦点→renderer、正常EXE/150%DPI与双端≤1DIP继续分别 open。goal active；整族 passed=false、accepted=false。

## 材质作者语义34：创建与自动连线共同预检

当前实际收件：原 request ue-material-graph-20261024-01 / 新 operation matgraph-autowire-01a16d50-20261025-34 / queue16229 / zircon-management#391；identity9f8f7982…，9自有源＋21依赖全部固定当前修订。正式 ordinary entry 客户端exit0、09:33:50Z实际build已启动，认证API building=true/result=null；归档receipt status=waiting/reason=management_git_busy，operationKey9adab3d26a57bd0f49442ed4926e5aa57d5f677235c63511a176b2679f1de8bb。compileStarted/实际发现执行数/native及资源闭合仍unknown，不能报75已跑。新收件不是重放#386或r10。交付回执 material-node-autowire-34-delivery.json（SHA96442c8f29898d3901a159b349a7b52bda39ad7c46655ca07be341b3c41bfe0d），原客户端逐字日志和并发advisory保留；4处其他域文档/插件测试变化未阻断，不归责、不冻结。

原唯一 Rust producer 接续33，在材质 leaf 的 lib.rs、connections.rs 和新增 node_creation_autowire.rs / tests/node_creation_autowire.rs 落实 plan_material_graph_node_creation。它直接借用真实 MaterialGraphAsset、候选 MaterialGraphNodeAsset 与可选 MaterialGraphLinkAsset；先校验注册动作、ID/位置、参数和完整资源引用，再以同一类型/环路分析器检查包含新节点的连线候选。已有 can_connect_material_graph 的公开签名、默认结果及原9项连接测试保持。四路径最小补丁 SHA56ed4c1c245db7d4787b14b19b25043acf84161c972cb6a907dabe0bef6f318a，当前 source34 与旧 search31 身份26项中24项相同，lib/connections 两处为透明后继；旧身份、输入和原回执不覆写。

可选连线必须恰有一端为新节点；无关边、自连、缺端点、显示名冒充 pin key、类型不符、重复ID或未解析纹理均拒绝。占用输入先在候选中去除原边，再作全图类型/环检查，返回 exact 原边引用以便宿主一次替换。允许未接齐输入的作者状态。计划只借用值，不分配稳定ID、不 clone 作者图、不修改文档、不发 permit。宿主仍须在同一真实 document/subdocument/source revision、lease/retention 与实际 transform generation 下，将插入节点、可选删除旧边/插入新边作一个 prepared delta 和一个 primary-history inverse；取消/失配不能发布半个节点。UE依据为 MaterialGraphSchema.cpp:66–111 的 FScopedTransaction / CreateNewMaterialExpression / AutowireNewNode。

新增9项有意义回归定义覆盖双向创建接线、原边替换、类型/端点拒绝、修复旧环、完整纹理引用解析和中文稳定键的真实 TOML 保存重开/MaterialAsset 编译兼容；仅测试辅助函数投影DTO，不把它当产品事务/Undo或渲染接入。当前 material_graph_ 过滤器为75项静态定义＝74普通＋1原 ignored，全部 lib 的78定义中另3项不匹配过滤器。四源码 rustfmt、diff-check 和精确补丁 guard 均0错误/0警告/0抑制；7个保护输入未变。尚无 Rust 类型检查、实际发现/执行结果，不能用静态数报通过。before/validation/patch/guard/handoff 统一保留于原 reports 目录 material-node-autowire-34-*。

原 search31 唯一操作 matgraph-search-01a16d50-20261025-31 / queue15814 / zircon-management#386 在09:19:36Z认证API实际为 FAILURE；控制台重试237–239仍 management_git_busy，最终 Resource admission remains pending / Finished: FAILURE。490B归档仍 waiting/management_git_busy（SHA bad1a456…），不据API失败或此文案推断 State 终态、native EOF、资源已释放、预算拒绝或 compileStarted=false。原 Jenkins owner 已获正式queue01a1725d-2440-7d53-abf2-527ccd560277的新终态交接；不重投该已终结operation或r10/#370。source34 后继验证须固定当前完整源、透明声明新operation并沿原request正常准入，不能借旧源#386作本轮通过。

本轮未改公共Graph/model/commands、主题/catalog、原ZUI、Penpot文件或其他域。复用 source31 三板的已保存/关闭/新context重开设计证据，不冒充本轮视觉复测。真实 popup/IME/focus → 作者document/history → 保存/应用 → renderer、官方导出、正常EXE/144DPI及双端≤1DIP仍逐项 open；goal保持active，与分叉30的Instance/Hierarchy/SearchInput设计切片并行，整族 passed=false、accepted=false。

## 材质作者语义33：真实 typed 节点创建预检

本轮按Parent“唯一producer、独立推进作者语义”的明确续作授权，新增node_creation.rs及tests/node_creation.rs，并在原lib.rs接线/公开validate_material_graph_node_creation；不改公共host、graph model/commands/retention、runtime、原ZUI或Shared。不创建第二model/store/history/clock，不分配或伪造UUID。候选直接使用现有MaterialGraphNodeAsset/MaterialGraphNodeKindAsset；动作必须与当前真实六项palette descriptor完全一致，不能只认显示名或过期pin schema。

预检拒绝空/重复/已被现有边引用的候选ID、歧义的现有ID、非有限graph-local位置、第二个Output、空参数名/非有限默认值、同名参数跨scalar/vector类型，以及不兼容或非有限的实际override。允许未接线Output/Math和同类型参数共享原名及各自默认值，不为编译尚未就绪的正常作者过程加假门；不重命名、不补参数map、不修改节点或连线。新候选错误只定位Graph，避免为尚未存在的node造SourceLocator；已存在且唯一的节点/parameter冲突保留实际source key。

AssetReference构造不查注册表，因此纹理预检显式调用宿主resolver，传入完整原UUID/locator引用并要求ResourceKind::Texture；拒绝未解析和错误类型，不从.png等扩展名猜类型，不改或重新派生身份。无效/过期候选不调用resolver；非纹理也不调用。实际catalog/revision/resolver安装继续由原host负责，单元测试闭包不当作生产加载能力。

只读UE依据为MaterialGraph/MaterialGraphSchema.cpp:66–111：NewNode在同一ScopedTransaction中创建并Autowire。当前leaf只完成对应域预检；真正创建与自动连接仍须原Document/subdocument/revision、graph-local anchor、一次prepare/permit、inverse-delta/history及焦点恢复，不拿material_editor.graph.create整资产operation冒充新增节点。没有更改Material Function模型；其作者schema/compiler缺口仍open。

新增11个有意义回归定义：真实六kind、过期动作/类型错配、稳定/保留ID、有限坐标、Output唯一与未完成图、中文参数及非法值、参数类型共享、原override保留、完整纹理resolver、失败不变，以及搜索activation→typed候选→MaterialGraphAsset TOML保存重开→实际PBR编译器roughness descriptor。最后一项只建立DTO/codec/compiler兼容，不声称host事务、GPU帧或EXE通过。当前七个literal cfg(test)模块共有66个material_graph_静态定义=65普通+1原ignored，尚无本轮Rust发现/执行结果。

三源码rustfmt、精确diff与guard71均exit0/0errors/0warnings/0suppressed。最小patch通过反向移除本轮三条lib声明精确重建f0f76fbb源前像，排除旧refactor；首次错误假定CRLF的重建被拒绝，实际LF字节/hash验证后生成patch，失败单列，不改源换行来迎合hash。原31 guard68/55项保持原时点，不签33新源码。

原request ue-material-graph-20261024-01 / matgraph-search-01a16d50-20261025-31 / queue15814/#386仍保留唯一既有投递；08:54:28Z实读building=true/result=null，原490B归档仍waiting/management_git_busy。本轮lib接线是透明source33后继：原31 identity仅25/26声明仍匹配，唯一变化为本轮owned lib；22依赖与另3原owned文件不变，新增2真实源码未塞写旧identity/payload。#386若后续执行，适用性只按其实际捕获闭包/源码/发现数核对，不拿旧声明或静态定义补成功；本轮未新增POST/重投旧终态。后续有限验证仍独立zircon_plugins/Cargo.toml/唯一material editor/defaults-off/features[]/locked/lib/profile test/jobs1/material_graph_/threads1/物理E，执行前另按当时源码透明声明。

证据为material-node-creation-33-before/validation/handoff.json、material-node-creation-33.patch及guard71/71-delta/72。复用原31三张真实Penpot设计/重开逐图记录，本轮无新Penpot、浏览器、服务或桌面操作；原公共Field/codec/Toolkit各自pending，不阻断本域实作。正常EXE、实际材质修改进入renderer、官方导出写回、144DPI及双端1DIP仍未完成；goal active，passed/accepted=false，释放0B。

## 强制续跑分叉30：Instance 层级原型与搜索框流式布局

用户于2026-10-25明确授权分叉 `01a1721d-c601-7b60-aea2-93eb528fa2c4` 接续原目标，goal实际保持active。原材质会话随后恢复并继续下节节点搜索/托管测试；本分叉仅处理既有Instance板上的Hierarchy三对象与四个SearchInput实例，不提交第二个Rust producer，不覆盖其新增板、源码或历史。分工通知已通过官方queue投递；投递不代表对方已读。

真实Material file/page仍为 `b4a1e3ce-9e6a-81fe-8008-d69d6092d704` / `b4a1e3ce-9e6a-81fe-8008-d69d6092d705`。依据只读UE `MaterialInstanceEditor.cpp:978–1007`，将原层级标题文本 `2c008adf-0071-4da4-8126-ef39b59726eb` 从Material Parents改为Parent Chain；保留No Parent Found空状态及所有原ID。在既有Hierarchy按钮实例 `1b77b5dc-466d-4a61-bed2-12de205c6284` / `7861e2da-a4a4-4780-b22d-810dca123d11` 上通过原生面板添加Navigate/Previous screen。实际点击可进入原opened板 `c7ac5e73-c553-4e70-b343-f357780e8a27` 并返回closed板 `eb526e4e-6399-4857-9f4c-37a7ac29f26d`；rev145保存、关闭context、新context重开后本域对象一致。它是设计状态导航，外部点击/Esc未关闭、焦点仍BODY，故弹层生命周期及焦点返回未通过，不能作为真实引擎菜单实现。

随后依据原 `workbench/primitives/inputs/search_input.zui` 的root width=Stretch，在四个既有Shared SearchInput consumer roots上设置原生Fill，并将其背景/文字子项从靠左固定改为左右约束；保留图标16、文字内容/字体/样式、componentId/componentFile/shapeRef及全部原UUID。仅设置root Fill的首次试验仍溢出，失败图单列保留；撤销该次自有改动后才修正子约束，未按截图硬填宽度。

| 场景 / 原board | SearchInput copy root | Details内宽 / 搜索框宽 |
| --- | --- | --- |
| 1280×800 closed / `eb526e4e-6399-4857-9f4c-37a7ac29f26d` | `2b056488-4bda-4940-85f4-96f1421a9902` | 382.8 / 366.8 |
| 900×620 closed / `9c6906df-acb7-4646-8d9d-280fe7025700` | `8ac3c5fd-fe08-4a21-9581-04678f87aba6` | 268.8 / 252.8 |
| 640×520 closed / `0206676f-5e96-42f2-ad25-31508a2f0a84` | `dc07cd1c-7976-40fc-89a7-c96ba3b36476` | 190.8 / 174.8 |
| 1280×800 opened / `c7ac5e73-c553-4e70-b343-f357780e8a27` | `0cd099fe-ed3e-4095-9ee5-3299666ae280` | 382.8 / 366.8 |

四项均保留左右8逻辑像素padding，文字宽随框宽变化。Shared组件身份仍为 `6ddc248b-bd35-80c5-8008-d6c8c3ce892e`、main shapeRef `6ddc248b-bd35-80c5-8008-d6c8c0f9c196`、source file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12`。08:34:55Z新context重开实读rev156/2167objects，四板共12个对象的本轮差异保存一致；全部原对象保留、Material五业务主件不变。本分叉没有创建Node Search三板，不将并发的196个对象算作自有交付。1280/900/640逐图及修正前后均保留；浏览器DPR1与画布缩放不作为实际150%DPI证明。

公共owner交接的具体候选为：保持现SearchInput的源Stretch、16图标及原文字slot/identity，将consumer native Fill与背景/文字左右约束映射回原v2布局语义，严格校验source closure与override/reset；不得通过detach、重签导入baseline或移除source/style refs规避官方导出检查。本分叉未改Shared、catalog、tokens、Starship或原ZUI，未执行正式插件导出、Rust或EXE。原会话接续31的55静态测试定义不能计为本分叉运行结果；旧47/34及所有Jenkins失败仍按原时点保留。

证据位于原报告根 `E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/` 的material-fork-30.json、material-hierarchy-30.json、material-instance-search-flow-31.json、browser-session-30-receipt.json及guard-67.json。自有browser PID14168已实际exit0，context/browser关闭，认证文件哈希未变，专属临时目录为空；本轮未删除文件，释放0B，必要失败证据及他人活动产物保留。原ZUI官方回写、真实作者文档/图宿主/renderer、Jenkins产品聚焦测试、正常EXE与实际144DPI、双端逐图≤1DIP及全族passed/accepted仍open；goal保持active，继续独立工作并沿Parent统筹交接。

### fork32 后继：Hierarchy native Overlay 的历史交接消费70

原fork32于2026-10-26T01:33–01:40Z保存并新context重开至rev205／3404objects；本会话于后续时点消费原件，不把205写成当前设计版本，也不回写覆盖66／67后来rev253或69源码。原3401IDs／5业务main保持，仅原生增加overlay `0e6a6a6d-2d5a-8037-8008-d8b5262edf4b`及两文本`...edf4c`／`...edf4d`，并修改以下3个原按钮的interactions。它们连同overlay的6个自有对象在fresh context逐项exact；四原Shared SearchInput roots／source refs保持，未detach或修改Shared。

| 历史设计尺寸／原board | 原Hierarchy trigger | rev205原生输入结果 |
| --- | --- | --- |
| 1280×800／eb526e4e-6399-4857-9f4c-37a7ac29f26d | 1b77b5dc-466d-4a61-bed2-12de205c6284 | 打开／外点关闭成功；Esc不关闭、focus BODY。 |
| 900×620／9c6906df-acb7-4646-8d9d-280fe7025700 | 6289ad06-c3ad-4f05-8e5d-9504839918a6 | 同一overlay；打开／外点关闭成功；Esc／焦点失败。 |
| 640×520／0206676f-5e96-42f2-ad25-31508a2f0a84 | 3d9ad777-bffc-4b78-8b06-f744ed46f8c8 | 原生280×80弹层可见并覆盖部分Details；外点关闭成功，Esc／焦点失败。 |

原Navigate／Previous screen及bottom-left错误取景保留为早期失败；后继为真实click→toggle-overlay、relative-to原按钮、manual x=0／y=真实源高32、closeClickOutside=true。这里的源锚点差异＜0.001只是该设计几何，不是Engine行框／边界≤1DIP通过。关闭态青色提示不能证明焦点返回，未注入JS伪造Esc、keyboard action或focus。Parent Chain／No Parent Found仍是明确无父设计数据，不当作已查询真实instance／registry结果。

本会话核对原handoff SHA058d9ec250bd8f08407ac0cf1539becf0abfecc35db491cf5d8ebced9a6b511c、report SHAa698760bf8e67dfd446e3c571f6ec09ce8ad3838935aacd80ca50a9a112ab80a及checks SHAe9d7e4405c4f7750a16f5d8201b32e0da795dd3f5ed9bdd9411533cd7a945488；16PNG原哈希／1600×1050原始像素尺寸全部匹配，复用原owner16图逐图结论。本会话额外直接打开1280fresh／640fresh-open／640closed三图，确认可见空父文案、窄窗遮盖及关闭态；未把采样复核说成自己重做16条输入。另4份UE／Penpot interaction／原ZUI／Search源指纹仍匹配。

真实宿主继续消费[既有共享popup边界](shared-node-graph-contract.md)：使用实际authoring instance／父链AssetReference与registry／Toolkit路由、document/subdocument/revision、popup incarnation/source generation、当前retained trigger instancePath和同代际viewport变换；DPI只转换一次。Outside／Escape／selection关闭同一popup，source切换拒绝旧callback，焦点仅在原source／pane仍有效时归还原input owner；关闭弹层不新增Document mutation／undo。不得新造第二popup模型或用颜色当焦点回执。

原v2仍SHA5856c886…／29nodes／27controls／13events，无Instance Hierarchy源控件或对应事件；需要真实作者父链及source-bound追加／事件映射和官方codec，不能把Penpot原型metadata签成已导出产品。原helper70890／PID25276缺原receipt／native EOF继续open；r02／PID35272明确close／exit0／auth unchanged仅证明自身，不关闭旧profile。旧profile11414415逻辑B及必要失败证据保留，释放0B。本轮只在本owned工作树增量合并本文，不写fork对象／源码／公共源／原ZUI；不新建Cargo身份，不因docs改变重跑Rust或单插件检查。真实Instance宿主／renderer／正常EXE／144DPI／双端≤1DIP仍open，goal active，passed=false，accepted=false。证据沿原任务reports根material-hierarchy-overlay-32-*及本次material-hierarchy-intake-70-*。

## 监督恢复31：节点搜索实际设计与域内实现
Parent于2026-10-25 01:02 PDT明确解除共享等待，goal实际active、runnable恢复授权有效。本轮继续独立实现和验证，不再把缺少完美共享交付作为整族停止的理由；旧28/29按原时点保留。并发fork30（01a1721d-c601-7b60-aea2-93eb528fa2c4）仅处理其Hierarchy/Instance SearchInput，本轮保留其全部对象、文档、helper和产物；原会话只增加Node Search31三板及下述四个材质Rust路径，不改公共graph、runtime、catalog、tokens或主题。

真实Material file/page仍为b4a1e3ce-9e6a-81fe-8008-d69d6092d704 / b4a1e3ce-9e6a-81fe-8008-d69d6092d705。rev145→146新增196个对象，原1971对象ID全保留；08:40:09Z独立认证API读回rev156/2167objects。三块400×400可编辑动作板依据只读UE SGraphEditorActionMenu.cpp:37–48，使用flex与11个原Shared SearchInput/TreeRow native实例，无新公共主件、detach或Shared写入。Shared同次读回rev5/4main/34colors/3typographies；背景、选中、主文字和body typography使用实际公开style refs，不能由此推导全局继承或正式导出通过。

| 状态 | 实际board UUID | 本轮逐图复核 |
| --- | --- | --- |
| 全部真实节点动作 | cbe04791-e3e8-4068-8570-ce848ad5032f | Output、Texture Sample、Scalar/Vector Parameter、Add、Multiply六项与实际palette顺序一致，无资产眼睛/锁按钮 |
| math筛选、Multiply选中 | af647700-59ca-4b2a-b508-825290869480 | 只有Add/Multiply；选中覆盖384逻辑像素内宽，白字及选中背景引用公开样式 |
| 中文无结果 | f977dac8-a7c2-4fa7-9916-3eed79ebd040 | 中文查询可见，无方框字；真实无结果，无伪造Function或别名节点 |

首次不存在的selection token在发写前被拒绝，失败原样保留。初图暴露TreeRow尾部资产控件与固定宽度问题，已仅在新增copy上隐藏不适用子项并设置Fill/文字约束、选中行和公开选中文字样式。三个独立browser context逐一保存、关闭、重开后捕获并人工复核；最终图为penpot-node-search-31-final-all/math/cjk.png。它们是Penpot DPR1/画布201%设计证据，不作为正常EXE或实际144DPI；局部几何/可见性override尚未通过原owner codec/export/reset，未补签官方asset-document。

域内新增palette_search.rs复用实际material_node_palette六种动作，不增加Function或未知节点。多词AND匹配稳定key/显示名/类别，保留注册顺序与仍可见的选中ID；方向/Home/End/Page15有界导航，空结果不触发，取消/激活后不重复产出动作。activation只返回真实descriptor；真实作者document/revision、创建参数、graph-local anchor、一次prepare/permit、history和焦点返回仍走现有host/领域适配，不造第二文档或历史。UE加权排名、鼠标/键盘宿主和节点真实创建仍待接线。四Rust格式、diff与guard68通过；新增8项，当前55静态定义=54普通+1原ignored，实际Rust发现/执行未知，不沿用47或旧34计数。

已按精确声明经canonical ordinary entry实际提交唯一当前源码后继：request ue-material-graph-20261024-01 / operation matgraph-search-01a16d50-20261025-31 / queue15814 / zircon-management#386。范围为独立zircon_plugins/Cargo.toml、唯一zircon_plugin_material_editor_editor、defaults off/features[]、--locked --lib --profile test --jobs 1 material_graph_ -- --test-threads 1、物理E根，传递runtime native-host/editor target-editor-host。原r10/#370及全部旧失败保留，未借外部r18/r19输入或改闸门/锁/服务。客户端第一次只读查询因buildRoot字面斜线差异失败，修正为原存储Windows字面值后返回management_git_busy；08:39:42Z认证API确认#386仍building=true、result=null，console重复Another process owns this operation。原Jenkins owner已获queue01a17238-b6c6-70c1-8c30-251154d7a7f6交接，继续原已投递操作，不新投或清锁。compileStarted、发现/执行数及native/resource终态均未知；queued/管理等待不计测试通过。

本轮交付material-node-search-31-handoff.json、before/validation、identity/managed/query/build-observation与三图逐图记录。正常EXE、真实材质修改进入renderer、官方导出写回、实际144DPI及双端≤1逻辑像素仍逐项open；窗口族passed/accepted=false。goal持续active，复用原报告根，未创建源码副本或运行本地Cargo；自有API/browser contexts与提交客户端均已退出，实际释放0B，临时产物清理仍待受管处理。

## 历史阻塞28（已由监督恢复31解除）
本轮是同一产品接入条件连续第三轮未满足的复核，未确认有仍在运行且能解除依赖的自有任务。九份材质源码与source24、十份宿主源码与host02的哈希均保持一致。47个material_graph_静态定义尚未编译、发现或执行；管理#499三例通过不计入材质验证。

公共owner的新runner-release-qualification-r01明确未编译、未发布：指定runner、metafile、compile-receipt三个输出均不存在，runner-only补丁仍是未应用提案。本轮06:47:42Z只读GET得到19002 worker HTTP200、348732B、SHA6ded04494d4126e9540b6b225ff0c5ecd61b9772408d8ac2d2e80f1d6c3701f3；可访问不等于源绑定导出能力就绪。Penpot沿用readiness26实读rev140，不借本轮GET声称重新保存或导出。

接续所需实际交付有三项：Parent提供canonical材质作者文档/subdocument/request/revision及Operation安装、图输入/paint/hit/focus、未保存MaterialAsset进入renderer的接口，或明确逐文件宿主交接；公共pipeline owner交付当前worker与reuse runner的正式构建资格及原ZUI源绑定导入/导出；Jenkins原owner给#370原native/writer/resource事实与受支持typed terminal-reconcile/CAS及当前源码验证接续。现continuation-contract的implemented=false，不能以合同代替receiver、重投已受理r10或改State/锁。

本轮完成上述限定复核和文档检查后申请将目标标为blocked，目标本身未完成。已交Parent的具体接入请求01a171c8-e615-7a40-bb0b-a38c4ec1c6b0保留；不新建编译身份、不重放r10、不更改闸门/服务或抢用桌面。正常EXE、真实144DPI、双端图与≤1逻辑像素逐图验收仍open，passed/accepted均false。报告为material-integration-blocked-audit-28.json及其validation/guard-64；本轮释放0B，历史失败和他人产物均保留。下列旧状态按原时序保留。

## 原请求源码差额接续29（只读回执消费）
Jenkins原owner于2026-10-25T07:25:50Z的result-current.json已核10606B/SHA80bb76fcb0271f6c3e9eab2bd42cd5a195504a5d925647f82985c75fd1550753，接原ue-material-graph-20261024-01链。旧compile-evaluation01/source8ba05523、validationee6b6640、patch4cebe850与guard53保留原历史：该packet的当时源码只匹配1/5、依赖9/12，不能背书后继完整源码。owner已复用现成source24的review e9d1eef7、handoff f77bb997、validation1c334de1，并记录九Rust在其观察时9/9匹配；本次仅核这些回执字节，不重复01/24源码盘点或重新扫描Rust。

34项属于01历史。当前五个已接线测试模块的47项定义（46普通+1原ignored）保留为source24及owner当前观察，不降回34、不写入旧payload，也不计为Rust发现、编译或执行。原四evaluation测试文件5df6310a和tests/cases的4784d9e3时点由来件保存，不拿旧guard53给当前整组签名。

r10缺省字段比较的644ccda8修复已交付，继续从当前阻塞中排除该旧问题；#370 ABORTED、State running/null与原native未知仍分别保留。实际执行前还缺合法sole producer及明确current successor、00f真实生成/完整raw diff/版本越界处置/CAS/单writer/资源/native恢复发布、当前完整member/control/.cargo/path/compiler和all-writer有界窗口、物理E reservation及兼容pool/cache/native writer，以及原操作权威收敛。现成lockgen-3c23/fdc33cc1准备与00f四边齐全不等于后者生成发布验收。本次不选择后继、不借外部r18/r19、不Query/重投r10、不resolver/手筛/回滚锁、不请求freeze，也不广播重复ACK；runnable=false，原目标blocked保持。

消费回执为material-compile-evaluation-reconciliation-consumption-29.json；本阶段只更新本域文档并运行guard65/66与diff结构检查，未运行Rust/插件、网络/OS资源探测、服务或桌面操作；释放0B。EXE/真实144DPI/双端逐图验收及全族passed/accepted仍open。

既有状态记录（保留原时序）：材质域编译器、连线 preflight 与获授权的公共图表面源码候选已形成，实际产品接线仍未完成。r9 queue15133/#369 与外部 producer 的 r10 queue15149/#370 均为 ABORTED；原生、资源和 Rust 终态仍未证实。本域 r10 首次客户端于2026-10-24T22:03:57Z因 manifest parser 拒绝退出2，不代表整个 operation 未投递。原 owner 的43187c30/#434—#436已修复后续 nested manifest 入口；本域按当时建议于2026-10-25T01:41Z用原 identity 正式 retry 一次，客户端因 Original State and delivery payloads differ 在新POST前退出2，无新queue/build。唯一原payload差异是 State显式observationOnly=false、原delivery缺该字段；未改写原字节。原owner随后撤回“r10未提交、可retry”建议：不再重放r10、不复用已终结operation新执行，也不自行创建后继身份。旧22:11Z/23:05Z parser拒绝和所有旧失败保留为历史，入口修复不关闭#370原生、资源、测试门。没有 Rust、原生或产品通过结论。

当前 `material_graph_` 有47个静态 `#[test]` 定义：默认可运行46个（原 cases 中7项、output_contract 中13项、connections 中9项、evaluation 中4项、diagnostics 中新增13项），另有 cases 中1个既有 `#[ignore = "release performance gate"]` 性能门控。evaluation和diagnostics都是透明的后继源码修订，不是 r10 原输入或已发现测试；没有把新源塞入任何旧 operation。本轮配方不增加 `--ignored`。历史“13项”是新增输出合同子集/当时计划数，不是 Cargo 发现总数；r3 未到测试发现阶段，执行0；r4 的实际发现数和执行结果未知，r9/r10 仍未到 Cargo 或测试发现阶段。公共 core 原19项及后续 bounds-fit 七项合计26项静态定义也均未运行，不混入材质配方。

真实 Penpot 当前为本域原文件 rev140 / 1971对象、5业务主件、102个native copies（其中69个Shared根）。接续17的失败恢复后，本域修正12张既有Material/Instance/Function画板共159个自有容器的流式子项顺序，并修正两个原Function主件及8个原实例的根子项顺序。21对既有Apply文字完成原生编辑往返，22在同一copy通过原生颜色面板绑定已发布样式；23沿8个既有页签条增加48个普通流式分配/裁切容器，原1923个对象ID均保留，没有增加或detach组件。通过Penpot原生宽高输入、保存、关闭与新context重开验证，component/file/shapeRef保持，详见下节。当前有效颜色引用1、字体引用0；这是单copy显式样式覆盖，不是主件继承或全局传播。Shared仍rev5、41对象、原4main UUID、34色/3文字样式，本轮未写Shared。17中rev55→56的40个颜色引用伴随1568对象几何变化和69处业务文字重置，已精确恢复至rev57，仍是未通过的同步，不能用本次布局修正或单copy绑定关闭。原时点和失败回执保留。既有设计保存重开仍不算官方原源往返。正常EXE、渲染、官方导出、实际150%/144DPI和双端≤1逻辑像素验收继续open，没有passed或accepted的材质窗口。

## 产品接线readiness26：真实文件与当前worker

2026-10-25T06:33—06:35Z使用原认证、只读API重新核对：Material原file/page仍rev140 / 1971对象 / 5业务main / 102 native copies，其中69个来自Shared；有效颜色引用1、typography引用0。Shared原文件仍rev5 / 41对象 / 4原main / 34颜色 / 3 typography，有效颜色引用1、typography引用0。Material own颜色/字体资产0不能当Shared库0。API实际返回camelCase字段；本轮最初误用内部hyphen字段而得到copies/ref零值的投影已标为不可用并完整保留。核对真实copy身份字段后按`componentId/componentRoot/shapeRef/componentFile`重算，三次Material response SHA676782fb…一致，未修改Penpot。

沿公共metadata.ts的真实`dev.zircon.zui`、role=`asset`和非空document合同，API解码namespace为`devZirconZui`。Material当前非空document标记0、官方asset-document根0；只记录已保存设计，不能给草图补签源metadata。现19002 manifest和worker均实际HTTP200：manifest SHA dbcb22b4…，worker仍348732B/SHA6ded04494d4126e9540b6b225ff0c5ecd61b9772408d8ac2d2e80f1d6c3701f3，缺canonical-source/canonical-record。这个负向事实与公共owner已有准入拒绝一致，不把健康或字符串搜索当严格构建资格。没有启动原publisher/runner、创建平行导出、重签旧source seal或写原ZUI。

host02的10个确切源哈希全部未变：原Material命令descriptor无factory、dist无invoke_command，retained Material workspace仍是三profile表格和Not implemented反馈，没有当前MaterialGraphAsset文档输入。限定检查中retained图消费只见AnimationGraphCanvasSlotAnchor，不能借此证明材质native paint/hit或作者事务。24的9个本域源候选仍可供接线，但需要Parent提供canonical活动材质文档/revision与命令安装、SourceLocator/焦点、同一retained图表面及生产MaterialAsset发布入口。不能新建第二份文档store/history/World、把payload演示图编译成真编辑，或强制Save模拟Apply。下一步是交付这些真实宿主接口或具体逐文件交接，不继续堆积无消费者的材质源码候选。

本轮是当前接入阻点核验，没有产品功能进展，也不是某个已证实live进程的等待。r10比较修复不提供single-recipe续跑、原native/资源准入或当前47定义的Rust结果；不重放/轮询旧终态、不申请无ready EXE/资源闭包的桌面捕获。正常EXE/实际144DPI/所有尺寸状态和双端逐图≤1DIP仍缺。本轮完整读回、纠正前投影、路径/指纹及11项交付要求差距留在原E根`material-integration-readiness-26{,-review,-validation}.json`与`guard-62.json`；只写本计划和任务报告，外部owner原件、认证与源资产保留。

## r10终态合同消费27：本域PID不能补服务端证明

原owner的parent-contract-review-r02/result.json已核SHA3d1772f454e3780b513cacf33a59d80403500868c81be6e6cecfa7ce57d64a17；这是复用27e2a07f…/entry a69e9c7c…的合同审阅，不是新安装。QUERY_ONLY默认false来自原job BooleanParameter、Groovy注入和runner精确true分支；只有missing/exact false在完整原POST/envelope校验后按比较副本等价。true/null/0/字符串false、真实recipe差异及未知execution字段仍拒绝。原#495的61 tests+23 subtests按原定义AST证明复用；新增State/delivery/candidate未知字段三例由#499/queue15699实际SUCCESS/3 tests/0fail/skip，管理native及10项local/Home归档属于管理资格，不属于材质Rust或#370产品原生。

限定核对本域原件：PID24060/birth2026-10-24T22:03:55.940746Z在22:03:57因manifest parser拒绝退出2；PID3536/birth2026-10-25T01:41:15.524619Z在01:41:17因原State/delivery比较拒绝退出2，两者均无新queue。原observer顶层compileStarted=false属于这些本地客户端阶段，不能解读成外部queue15149/#370的服务端未编译证明。外部#370原control SHA26d35cc6…和本域原console SHA62b4de7e…保持；control没有PID/birth/Job字段，本域observer只读记录也没有产品native证明。#499管理子进程PID31908同样不能替代#370。原observer、identity、control、payload、State和marker均未改写。

continuation-contract.json（SHA7911275681b122c61000938c20e882a3b73f48ea1d806a44ad1a09fc1085e5f5）明确implemented=false：所需typed terminal-reconcile/CAS receiver当前不存在。未来必须绑定同原键/State版本/inputDigest、原body/envelope/marker/control/runtime、认证Jenkins终态和原native/children/Job/双EOF/readers/waiter，以及source/pool/cache/writer/reservation释放或保留证明；缺证、输入改变或未知claim均拒绝。合格后的效果也只能CAS登记原历史ABORTED和证据，不能写passed、重放或编译；后继执行另需受支持合同。single-recipe不能借batch ResumeOriginal绕过这些条件。

本轮只消费新的管理合同/三例回归并澄清本域记录的证明范围，没有产品进展或新的native身份。没有HTTP/QueryOnly/POST、Cargo、capture、receiver实现、State/锁/源policy修改，也不重复向原owner发送同pending ACK。47个材质静态定义、真实文档/宿主/官方导出/renderer和EXE/144DPI/双端验收仍open。回执为原E根`material-r10-terminal-contract-consumption-27{,-validation}.json`及`guard-63.json`；管理owner清理的783逻辑B不计为本域释放，本域释放0B。

## r10入口一致性修复消费25：原终态与准入仍独立

已离线实读原owner的`material-r10-delivery-consistency-r01/result.json`（13266B，SHA644ccda89c108ff79c5832b7b60cc4b6555678e9108cfa767a56e12b76f8d611）及`summary.md`（2641B，SHA47e534f6cf48af9611d4022bc31815c39eb1618f213d2a3f2af4829717dfbf61），不再把此前Original State and delivery payloads differ作为未修复的当前入口问题。owner归档的immutable capsule为27e2a07fbcf4b439d8cb8ff374e22ce4ba64e30f62af1726ed33d617a79641c1，entry为a69e9c7cc60c1521f6067b6e079975588540273f27b13acf8b0737cf3a34ead8。修复仅在原POST/body/envelope完整校验后，按缺省与精确`observationOnly=false`统一执行payload比较；其他执行字段和inputDigest仍须精确相同，6个server成员及5项source/pool策略不变。先前本域01:41客户端拒绝、旧immutable版本和原失败仍按时点保留，未改原payload或重签后继源。

管理资格#495为61 testcase+23 subtests/0skip，#496 CheckOnly/#497 apply/#498自有清理SUCCESS；#494外层计数失败保留。5份管理native ledger及其实际文件哈希分别核对，只证明这些管理子进程的root/Job/children/双EOF/readers/waiter，不代替产品#370或运行中host。原owner清理11994逻辑字节与未知物理回收属于其任务，本域释放0B，不重复计入。

原request `ue-material-graph-20261024-01` / operation `matgraph-tests-01a16d50-20261024-r10` / identity52062316… / queue15149/#370完全保留。归档只读对账仍pending/running/result=null/resubmitted=false；State v1、outer489aadc6…、marker7b66566d…、input728f31db…保持。#370的ABORTED与State running分别记录；single-recipe ResumeOriginal没有受支持attempt转换，不能因比较修复重投r10、补造新operation或推导原native/EOF、writer、pool/cache/resource reservation已释放。实际Rust发现/执行仍null，旧19/20观察及00f嵌套锁后继不能借此重签。本域没有再次HTTP/QueryOnly/POST、capture、Cargo、服务、State/锁或gate动作。

本次仅消费新管理修复并刷新文档；下节24的9个材质源与原静态回执哈希均相同，47个静态定义仍未运行。不发同pending的例行ACK或重复查询，不改Parent/公共owner路径。完整引用、限定核验、文档guard与实际未完成项留在原E根`material-r10-consistency-consumption-25{,-validation}.json`及`guard-61.json`。正式官方导出、宿主诊断/材质作者文档与生产renderer握手、正常EXE/144DPI/双端≤1DIP继续open，passed/accepted=false。

## 材质编译诊断位置24：本域接线候选

宿主02所需的诊断跳转原来缺编译源位置：编译器只返回字符串，公共`GraphDiagnostic`仅表示required-port的静态message，不能表达图级或实际求值错误。本轮先记录9个本域路径的before及15项只读依赖，再沿实际校验、连线类型检查和非递归求值路径落实`MaterialGraphDiagnostic`与`MaterialGraphDiagnosticTarget`，新增`validate_material_graph_with_diagnostics` / `compile_material_graph_with_diagnostics`。旧String API、原消息顺序/去重与`MaterialGraphCompileReport`字段保持；同文案但不同实际目标在新API中分别保留，旧API仍按文案去重。不是解析错误文字猜ID，也不替公共图或其它域扩展业务模型。

目标只包含本次输入中的Graph、唯一Node、真实Input/Output schema key或参数map key；空/重复/不存在的node ID不能按index最后一项伪造跳转，退回Graph目标。非法pin本身不存在时定位其唯一所属node。参数名仍是当前schema key，rename后必须由宿主revision/source remap处理，不能冒充永久UUID。宿主负责把返回值与实际document/subdocument/request/revision/incarnation绑定，拒绝过期结果并走现有SourceLocator/Details/焦点合同；位置数据不授予GraphDelta permit，也不创建history或预览World。

实现限自有material_editor/editor/src：原lib、graph_contract、connections、evaluation、outputs，以及新[diagnostics](../../../zircon_plugins/material_editor/editor/src/diagnostics.rs)、[compilation](../../../zircon_plugins/material_editor/editor/src/compilation.rs)、graph_index和[13项回归](../../../zircon_plugins/material_editor/editor/src/tests/diagnostics.rs)。把原校验/编译和私有index从lib移至对应实现文件，入口只保留声明与重导出。UE只读依据为MaterialEditor.cpp:3080的真实token点击、3177—3182的FailingExpression token，以及6388—6433按表达式身份解析后JumpToNode；尚不等于Zircon宿主已接入。实际数值溢出保留运算节点，metallic/roughness文案前缀保留上游目标；不存在的pin定位唯一所属node，重复/空node ID退回Graph。新回归覆盖真实schema key、重复/缺失身份、同文案多目标/旧去重与shader顺序、参数map键、类型错误、上下游溢出、cycle及旧报告struct literal。

原34项与新增13项合计47个静态定义，全部仍未Rust发现/编译/执行。当前是可审阅源码候选，格式/guard/闭包/差异检查见原E根`material-diagnostics-24-{before,review,validation,handoff}.json`、`material-diagnostics-24.patch`及`guard-60.json`，不替代托管测试。旧Jenkins失败及原r10输入保留，不新建编译身份、不重放r10、不改锁/闸或Parent的runtime五路径。Typed target还没有真实宿主SourceLocator/点击/焦点消费者，Apply/Save与生产renderer也未接通；本轮未写Penpot、Shared、原ZUI、公共图或宿主源码。正常EXE/144DPI/双端≤1DIP与全族passed/accepted继续open。

## 自有流式布局接续18—20：原生重排与Function主件修正

实际Penpot flex实现按shape Z顺序的反向排列普通row/column；此前部分自有容器直接存储视觉顺序，原生重排会倒置菜单、工具条和区域。18先在原640×520未连接Material画板修正15个非copy容器：首次`reorder-children`虽HTTP200并增加rev57→58，独立GET证实数组未变，原失败保留；后以支持的`mod-obj assign shapes`修正至rev59。原生宽度640→641→640后rev61，关闭重开数组/对象一致，板外几何无变化。只改容器顺序，没有按截图改节点坐标。

19把同一修正扩展到余下11张既有画板的144个自有容器，rev61→62；Hierarchy弹层仅反转in-flow子序列，保留绝对子项Z位置。11次宽度+1再还原共22个原生输入后rev84，关闭重开对象一致，identity与caption差异均0，235处几何变化全部在本次画板内。12张画板合计覆盖Material、Instance、Function的1280×800、900×620、640×520及既有1280组合/Hierarchy状态；这些是设计尺寸及原生设计编辑行为，尚未证明产品交互或真实DPI。

逐图复核没有把19全数记为通过：四张Function画板中的两个业务节点在原生重排后标题位于底部。20沿原主件Input `6b647966-33a3-4326-ac96-1b15c26ff5d4` / main `ca50a72a-64fe-4550-9c60-6fffeea2b16d`，Output `f0ae6a1b-c6aa-40a4-b925-6021df706cdb` / main `887826cc-2f45-40e5-ab77-079633df253d` 修正源根顺序；8个既有copy按精确shapeRef一一映射相同顺序，未新建、detach或改业务文字。rev84→85只改变10个根的shapes字段；随后两主件和四画板共12个原生宽度输入推进至rev97，正常主件modifiedAt随原生编辑更新。新context重开确认全部10个标题与引脚区域上下正确，identity/caption差异0。50处几何变化中7个包含容器仅有最大3.41e-13的数值舍入，全部位于实际gesture范围；最初review脚本遗漏这些包含容器而停止的断言也在20-review中保留勘误，未改原native回执。

四张Function图的原有设计fixture连线仍对应原Value引脚，API测得端点与引脚中心最大误差0.000197设计逻辑像素，六张主件/画板截图已逐图查看。这只是Penpot几何复核，不是生产paint/hit、真实Function文档边或引擎≤1DIP验收。900和640左侧Details/Parameters/Overview页签仍裁切，需Parent的真实Dock/tab overflow与焦点合同；Function持久文档、compiler、预览与Apply/Save行为也仍缺，现有fixture不计后端实现。

证据继续留在原E根任务报告：`penpot-material-flow-order-18*.json`、`penpot-material-flow-order-19*.json`、`penpot-function-flow-20*.json`及`penpot-browser-10/flow-18-*`、`flow-19-*`、`function-flow-20-*`截图。18/19的失败和后继20分开保存。Shared、catalog、tokens、Starship、公共codec、原Material ZUI和Rust本轮均未改；不重放r10、不增加Cargo身份。官方导出/样式传播/真实渲染、34项材质静态定义与26项公共图静态定义的托管执行、正常EXE和150%DPI仍open。自有浏览器/context已退出，原认证不变，任务与产品验收未关闭。

## 既有页签流式分配23：完整标题与裁切差异

只读UE的`SDockingTabWell.cpp:305`确认页签均分可用宽度并受minor 160×25上限约束；`StarshipCoreStyle.cpp:1893`为minor页签规定左/上/右/下4/3/10/4 padding及-2 overlap，即相邻间隔2；`SDockTab.h:113`默认Ellipsis，并由cpp:723—726交给STextBlock。`MaterialEditor.cpp:6014`的实际标题是Nodes Overview。此前本域三个固定宽文字在窄窗挤掉第三项，与这个合同不同。

沿8个已有Material/Function页签条及原24个文字ID，增加24个普通等宽分配frame和24个内容裁切frame，保留完整可编辑标题；8处Overview内容改为实际Nodes Overview。三个分配区使用原生fill、max160、高25、gap2，内容区位于实际padding内；不新建主件、公共Dock组件或样式，不手写省略号/截短业务标题，也不补签官方metadata。当前1971对象包含全部原1923对象；相对rev100，原对象中仅8个页签条和24个标题改变，其余1891个原对象语义完全不变。Apply的21文字覆盖及22共享颜色引用也完全保留。

本轮共37次真实原生宽高输入，包含原640试验、8画板往返、8个自有页签条往返，以及三处高度纠正。1280/900/640的均分宽分别约81.9333/56.6/39.2667；两张1280组合板是84。600px临时扩宽实测触发三个160上限，随后还原原宽。最终24处分配区高25、内容区高18，等分/位置/padding算术最大误差2.85e-14设计逻辑像素；这不是UE或引擎画面≤1DIP验收。

逐图与严格检查保留了失败：rev120中auto-width文字会画进右padding，后增加独立内容clip；rev137中最早三处Material640试验frame仍实际高32而非25，几何检查拒绝后通过原生高度输入分别保存138/139/140修正。原137错误图不覆盖；另一次review把Python与Node不同JSON序列化的字节摘要混比而停止，后按原Node算法证实1971个对象摘要一致，Shared数据也未变，未放宽视觉或来源规则。最终关闭、新context重开rev140，全部对象与已保存值相等。16张中间/最终/纠正截图均实际查看，逐图状态及全部旧阶段在E根`penpot-tab-flow-23{,-review,-source-review}.json`和`penpot-browser-10/tab-flow-23-*.png`中分列。

当前原生Penpot画面仍直接裁切，缺UE按实际字体测量的省略号，不能把这次分配修正记成视觉通过。现有公共`penpot-text-overflow.ts`已有保留源文字的原生测量合同，需要原owner完成当前worker与真实source绑定资格；不另建省略号系统。关闭按钮、icon/selected/hover/focus/tooltip、dragDock、实际命中与焦点返回仍由Parent公共宿主接线；现有32高页签条与25高分配区的宿主留白也需实际Dock复核。八板仍是按尺寸的设计fixture，不证明生产自适应布局。原ZUI、Rust、公共theme/catalog/codec均未改；官方导出、真实材质渲染、托管测试与EXE/144DPI/双端验收继续open。

## 单实例原生共享颜色消费22：保留文字与几何

沿原640×520 Material画板 `722ca546-223f-4c7f-9e52-a37348a0250e`、同一Apply copy `b4ea9cdd-7450-4e6d-883b-16b359aef1e8`，通过真实层级选择、Fill面板和Shared颜色选择器，点击已发布 `editor.surface.1` / `599586e3-022c-500b-ac80-5d9c490139af`。先核对样式的实际`token-source`、theme文件哈希dc23135b…及原Button正常态`$editor.surface.1`，没有按RGB或名称猜测来源，也未改hover/focus/selected样式。保存rev99→100，关闭context后新context重开仍100。

整个页面仅该copy的fills和touched两个字段改变：原#242424值不变，新增真实Shared file/style引用，touched新增原生`fill-group`。21的Apply文字、content-group与text-content-text标记、全部几何、component/file/shapeRef及其余对象均保持，重开对象与已保存对象相等。四张原生界面图已逐图查看，最初选择器范围不唯一及等待异步色板不足的安全退出保留在原回执；picker的预览HEX不代替实际提交值与持久引用。证据为原E根`penpot-native-color-style-22{,-review,-source-review}.json`及`penpot-browser-10/color-style-22-*.png`。

公共owner已形成严格source-bound颜色捕获源码候选：`captureSourceFill/Stroke`可在真实token、源文件哈希及已发布UUID一致时保留引用，未映射/过期引用仍拒绝。旧“正式捕获一概拒绝非空引用”只适用于此前部署版本，不能概括这个后继候选。当前14路径中8项仍与owner初始回执同哈希，6项已有后继，未借旧静态检查签整套当前源。该候选尚无本域取得的受管编译、当前网络worker兼容或正式Material导出证据；字体绑定、缺少canonical组件、composite slot和同ID主件迁移仍open。

本次证明一处显式copy样式消费与原生保存重开，不证明主件默认继承、Update/reset、跨族A/B或Penpot修改→官方导出→产品加载的完整链。未再运行Shared Update、改公共源/原ZUI、创建Cargo后继或占桌面；正常EXE、实际144DPI及双端≤1DIP继续未验收。rev99的零有效引用和17失败均按原时点保留。

## 单实例原生文字覆盖21：实际标记差异

沿同一640×520 Material画板，只编辑原Apply文字 `c99644f1-19bd-4e76-8230-0cb8f88f7fd0`，其copy为 `b4ea9cdd-7450-4e6d-883b-16b359aef1e8`、Shared Button component为 `6ddc248b-bd35-80c5-8008-d6c8c01096f3`。通过实际层级展开按钮选择原文字、Enter进入运行中的DraftEditor，执行Apply→Apply!→Apply，保存rev97→98→99；关闭、新context重开仍rev99。原来仅有`text-content-text`的touched变为`content-group`加`text-content-text`。整个文件最终只有该文字的content和touched两个字段改变；content仅增加原生编辑器paragraph key，文字、字体、颜色、几何、父copy及source metadata均不变，没有改其它实例或Shared。

四张原生界面截图已逐图查看；首次SVG点击被实际选择覆盖层拦截、第二次检测到两个嵌套contenteditable而拒绝输入的自动化失败均留在原始回执。第三次沿真实层级和已核对的唯一leaf编辑器完成往返，不强制点击、不用DOM注入或手写touched冒充原生编辑。原始与review为E根`penpot-native-text-override-21.json`及`penpot-native-text-override-21-review.json`，图为`penpot-browser-10/override-21-*.png`。

这是部署编辑器实际生成覆盖标记的证据，还不能判定17全部文字重置的原因，也未证明增加content-group可同时保留文字覆盖和共享字体/样式传播。只读dev/penpot的`text-change-value`已经识别partial text标记，需公共owner核对实际部署版本与正式import/export的兼容合同。没有给其余实例批量补标记，也未再次Shared Update、修改公共codec或以导出器metadata补签通过。上节rev97交付和本节rev99后继分开保留；官方往返、产品、Rust和全族验收继续open。

## 共享库原生更新实测17：失败及恢复

本域在原文件、原P21认证和隔离无头浏览器中实际看到“共享库有更新”，通过Penpot原生“更新”执行同步。`update-file-library-sync-status`与`update-file`均返回HTTP200，Material实际rev55→56；原有1923对象ID、5业务component/main身份及69个Shared根的component/file/shapeRef均保留，Shared数据未变。新增40处颜色引用全部指向Shared的`editor.surface.1`（599586e3-022c-500b-ac80-5d9c490139af），其token映射与当前editor_tokens.zui的dc23135b…一致。没有绑定字体；域文本使用custom Roboto ID，已发布排版使用gfont-roboto，不能仅按同名字体贴typography ID。

同步同时改变1568对象的几何、194处hidden字段及69处业务文字；例如Apply→Button、Search nodes→Search、Emissive→Property。1280×800未连接根窗口出现菜单/工具条移到底部等重排。失败图为E根任务目录`penpot-browser-10/closed-context-reopen.png`。这些是一次真实同步的副作用，尚未证明底层原因，不能归责某个writer或以sourceRef有效证明override/reset保留。原文件无官方asset-document根、composite与正式codec仍缺口，未通过自动导出。

发现差异后先核对rev56及每个待恢复字段仍匹配本次after记录，再向同一文件提交1568个精确inverse assign，保留touched/geometry控制和所有ID；HTTP200、rev57。API语义比较确认记录内所有字段恢复，新context重开仍rev57，恢复图为`penpot-browser-10/restored-closed-reopen.png`。原receipt-04使用JSON字符串顺序比较而误报263项positionData差异；独立字典/数组语义比较为0，原误报回执不改写，勘误见`penpot-native-library-update-17-review.json`。同步失败及精确逆操作不是一次可接受的样式传播。

本次证据在同一E根任务目录：`penpot-shared-style-admission-17.json`、`penpot-native-library-update-17.json`、`penpot-native-library-update-17-recovery.json`及逐图review。公共owner需沿既有组件/导出合同修复实际实例覆盖与flow在原生同步时的保留，并交付严格source/token映射；本域不改Shared、catalog、主题、公共codec或宿主。不重放r10，不新增Cargo身份；插件/Rust/正常EXE/DPI及双端验收均未执行或通过。隔离浏览器与context已关闭，原认证未改，自有临时目录实测0文件/0字节，无删除、释放0B。

实例持久化的当前单项状态为 **MAT-INSTANCE-02：`source-fixed/managed-tests-pending`**。Parent 的五路径补丁与八项正式 codec/父链回归已只读对账，sourceAfter 5/5、原 runtime identity 与回执哈希匹配；原请求 `ue-material-instance-presence-20261024-01` / `validate-material-instance-presence-01` 第二次捕获为 `ordinary_entry_unproven`，在 `finish_full_file_index` 拒绝且未保留历史before/after，不能细分究竟HEAD、index还是membership改变或归责。未起Cargo，无runtime验证作业回执。原owner的诊断修复与管理#407已交付并另行核对，八项Rust仍未运行。它与上述材质插件13项、公共core19项各自分开，均未记作Rust通过，不重投或换身份。MAT-INSTANCE-01参数ABI、03作者editor元数据、非canonical alias、函数/图host及真实产品接线继续open。 原owner21:44Z后继核对为Runtime源5/5、依赖5/6；Home #407归档已缺，本地证据仍保留，工具后继指纹及当前证据边界见下方consumption05勘误。

历史恢复交接后的本域原身份查询已实际完成：材质 r2 与 core 两项 QueryOnly 均 exit0，分别读回原 #352、#353 准备冲突，`resubmitted=false`、`compileStarted=false`，不计 Rust 通过。当时实际选中后续 `d2873ff3…` 胶囊，manifest 与5份server/6份pinned client全部匹配，driver/runtime不变；Parent 原20b3观察和本域2790历史均保留。当时 Parent 副本因100条上限拒绝是历史投递结果，不是当前阻塞。r3 已按用户新授权消费 e0635d5d…正式入口并得到上述新终态，原身份没有改写。

## readiness10：原导出链当前边界（2026-10-25）

原pipeline owner回执为 E:/cargo-targets/zircon-local/reports/penpot-unreal-01a0f04d/ue-penpot-resume-20261023/local-execution/public-component-closure-01/existing-plugin-session-r01/material-readiness-10-r01/handoff.md，SHA a9daa22ee9b187832bb2d420f9cd448a7e51479c4bf6835e3d78bf16900799e6。本域只读复核当前10源均与新fingerprints匹配；完整fingerprints SHA 66ae78bdcf61ae01b79c8718d811beba7baf3b444433d895690c9f4ba17210a8。递归10文件/16条import边，根material_workspace.zui仍5856c886…、v2/component、29节点/27control/13events/8个直接widget imports；默认collapsed、稳定ID、typed slots和事件不改。相对旧十源仅PropertyEditorRow依赖由bf29d574…变dc2fde3a…，不回滚公共owner的后继改动，不借旧seal承认新canonical等价。

02:34Z原owner实际19002网络plugin.js为348732B/6ded0449…，对应D盘原发布目录；E盘R3仍485232B/442bed70…且与网络不同。本域本轮核对两处物理字节，未再次网络捕获或启动浏览器。原R3记录输入现103/118匹配，旧runner现143/173匹配，均为原owner本次审查；旧118/118或161/173只作历史。worker物理字节未变不等于当前源码与runner/UI/worker同配置严格兼容。Runtime真实字体定义384d7083…和字体03564700…在场；不能以Editor目录无该文件误报全局缺字体，真实挂载/浏览器字体和中文测量仍待验证。

原generic review host两份TS候选只去掉一个生成的首层review_component假callsite，保留真实内层ancestry/source owner/control/slots/events；严格apply guard c46c386c…未放宽。候选patch为75f33211…，两源为1b68337e…/64139f87…；本域与实体文件匹配，复用同源静态guard及5项原生静态检查，不计作Node/codec运行。新增3项原codec回归仍定义3/执行0，Material没有已执行的spacing-edit分支；AI-only flag不可用于Material。

当前materializer已查询local与connected libraries，但canonical不匹配时仍可创建owned shelf/本地主件，createComponentMaster=false不足以禁止内层fallback。带children的prefabSourceForNode返回null，真实PropertyRow.value中的Dropdown/Field组合尚非native composite复用；Shared四主件也不包含Field/NumberField。公共owner须交付strict canonical admission、typed-slot物化和受管回归，不能用缺主件时创建的本地主件关门，也不能Text替换required/single/七类accepts。

后续继续原zui-layout-real-session.ts --source-closure与mode=reuse-existing、原19002/pluginId cce74285-44c0-8011-8008-d6ea3345bdbe、原只读storage和精确Material file/page。实际执行前重读revn/vern与全部旧对象/main/copy/shapeRef，用正式transit registry/manifest/bundle seal，仅追加独立源验证板；不套用本次JSON诊断哈希、不建平行runner、不补签七张业务草图metadata。正常导入须replaceSelection=false/createComponentMaster=false并先证明内层canonical/typed composite合同；当前不盲跑full import。原owner记录8h期限02:13:48Z仅为旧时点；Parent 的 penpot-runtime-01/current.json 已记录02:14:57.478036Z的新8h owner。本轮对该PID1684/birth134373680935404589只读核验一致且仍active，未操作服务；这不代替各服务身份、当前HTTP或正式生命周期验收，也不授予本会话接管。

实际仍缺：各服务当前生命周期证据、受管同配置runner/UI/worker兼容、三个codec回归、Material真实spacing-edit可执行分支、严格canonical/typed composite、official import/save ACK/close/fresh reopen/edit/export和源delta审查。只在这些原链条件成立后写回唯一原Material owner；本轮不改原ZUI/公共组件或主题，不编译/操作服务、浏览器、桌面或DPI，不重放Jenkins旧operation。Engine144DPI、真实材质渲染与逐图≤1DIP、passed/accepted/captureReady均open，释放0B。

## 托管验证接续：r9 与 r10 的实际来源和终态

同一请求仍为 `ue-material-graph-20261024-01`；下表 operation 前缀均为 `matgraph-tests-01a16d50-20261024-`。物理根保持 `E:/cargo-targets`，唯一 package 为 `zircon_plugin_material_editor_editor`，独立 manifest 为 `zircon_plugins/Cargo.toml`，defaults off、features=[]、locked/lib/test profile、jobs1、`material_graph_`、testThreads1，传递 runtime `native-host` / editor `target-editor-host`。r9/r10 的实际服务端控制文件均已逐字段核对该范围，没有扩为根 workspace 或 core 单元测试。

| 实际调用 / producer | queue / build | 已观察结果 |
| --- | --- | --- |
| 本会话 r9，identity `1fcfd5e7…`；正式 entry 客户端于21:48:59Z wait 完成、exit0 | 15133 / #369 | `ABORTED by admin / Hard kill!`；最后阶段为 `checkout_read_inputs begin`，未看到其完成、guard 或 Cargo；无最终服务端 Rust/native 回执 |
| 本会话 r10，identity `52062316…`；正式 entry 客户端于22:03:57Z wait 完成、exit2 | 本次调用未生成 queue | 原始 stdout 181 B：`cargo_option_unsupported` / `No Jenkins recipe for Cargo argument: --manifest-path`；stderr 0 B |
| 外部 producer 在22:04:55.383Z 用同一 r10 operation 投递；并非上一行客户端投递 | 15149 / #370 | 服务端控制 SHA `26d35cc6…`，范围匹配；295.040秒后 `ABORTED by admin`，最后仍是输入盘点，未到 Cargo；不得再将 r10 当未投递身份重试 |

实际入口变化也按时点保留：r9 使用 c2b07350…；中止后先出现 `Pinned management runner changed`，随后本地3364f4eb…与在线旧 c2b…短暂不一致。3364f4eb…的公开绑定恢复后才进行本会话 r10 调用，实际 parser 已拒绝 manifest。22:11:08Z 后续 a9a82c24…的真实 capability/publicTransport 通过，但其选中 entry SHA `57153be4…`、签名 `(arguments, *, enforce_policy=True)` 仍返回同一配方拒绝。没有把旧 canonical parser、历史已交付的 cdd 配方或单独 ready 当作当前可用性证明。原 owner 的历史 cdd/管理检查继续保留；本次回归需要恢复已存在的 manifest/单包/闭包/pool 合同，不扩大配方或改 gate。

本轮本会话只改本域台账和 E 根报告，没有修改管理源码、安装 capsule、POST Jenkins 配置、停止进程、恢复服务、清锁、创建源码副本或占用桌面。管理版本变化与管理员中止来自外部；不可读的 SYSTEM 命令行或 PID 消失没有被写成 root/children/EOF 终止证明。r9 在盘点期间持续有 I/O；同一进程 birth 与计数已保存，没有把日志暂时沉默当作进程死亡。

`material-managed-validation-r9-owner-observer-01a16d50.json`、`material-managed-validation-r10-owner-observer-01a16d50.json` 保存各自实际 identity、客户端 stdout/stderr/退出记录及当前状态；`material-managed-external-build-370-observation.json` 与原日志保存后续外部投递证据。它们位于本域原 E 根任务报告，原始失败不覆盖。最新配方回归与外部投递勘误已通过官方 queue 发给 Parent 和原 owner `01a12af6-7511-70d1-a452-61b80678eb56`；queue ACK 仅代表排队，不代表已读或修复。当前没有 Rust 通过、产品 passed 或 accepted；本轮实际释放0 B，必要证据和未核实的原生终态保留。

## r4 receiver恢复与已受理原操作的连续性

原管理owner22:55:48Z沿当时真实pinned entry的capability(public_transport=True)及原verify_public_transport均通过。selector为2d2debd72e7331afafe8fae9a9296e0c59904873ddca647cf6385aed27864907，11pins读取期间稳定；认证API与Home的script同为5b448192…，模板恰1次、8参数、sandbox=true、disabled=false。两个XML原字节不同但脚本相同，不能要求XML序列化逐字一致。当前receiver mismatch这一旧阻塞已解除，不需要再apply、POST或重放e5历史绑定修复。

最初e5拒绝、当时无queue仅为早期状态；同一 `ue-material-graph-20261024-01 / matgraph-tests-01a16d50-20261024-r4` 与identity b284fa46…在19:47:45Z已受理queue14924→#364，inputDigest65c737f9…。本轮owner认证GET仍为ABORTED/building=false/artifacts=[]；原operationKey f625b72c…的management_operation为running/version1，未登记result。最初“无queue、compile=false、执行0”不能再充当r4当前结果；compileStarted、真实发现/执行数均unknown，30静态定义（29普通+1ignored）不是发现结果。

原durable-745f95f1四个文件在owner观察时均不可见；文件缺失、Jenkins ABORTED和running记录之间不能互相替代，native root/children/双EOF/readers/Job-close及资源释放仍缺证明。不能据receiver恢复重新投递同r4、创建替代request、关闭running或释放writer。嵌套manifest/唯一材质包/lib/test/material_graph_/defaults off/features[]/locked/jobs1/threads1/物理E根的原范围不变；后继r5—r10及r2/#352、r3/#362、core/#353继续各自保留。

本域只读核验result、summary、认证观察和原状态/native边界等七份引用及原identity，记录在E根 `material-r4-receiver-consumption-22.json`。这是receiver绑定与受理连续性的纠正，不是parser/route全套资格、Rust、native或产品SUCCESS；余项沿原#364/Records/native链补真实阶段、响应/EOF与资源回执。本轮没有再次capability/QueryOnly、POST、源码捕获、Cargo、身份/receiver创建、gate/selector/Home/sealed/DB/锁/服务或清理操作，释放0B。

### r4 client与单配方续接：23:05Z后继核对

原owner后继 `client-capture-stability-reconciliation-01.json`（c1cef2bf…）与其观察351b1e11…已核验。23:05Z的snapshot f91c3124…及11pins、正式public transport均通过；当前client的source_digest实现记录两次capture差异后选后一次，不再抛旧19:27漂移拒绝。本轮只读代码/纯内存判别，没有实际capture、POST或编译；19:27七文档失败与后来同identity queue14924/#364均保留，不能把早期失败作当前入口结果。

新的可执行性缺口已明确：同一当前前端纯parse返回cargo_option_unsupported/--manifest-path，server对原nested材质单包配方的纯内存校验则接受。这是前后端合同不一致，不是可执行资格或成功回执；本域不另跑parse/投递，沿既有plugin-entry管理lineage由原owner恢复并资格化manifest前端。

原r4是single recipe，当前resume_delivery只准有batch的原payload，不能给它补batch或jenkins-admission字段。另一个原submitted marker `9a686d57…json` 已存在，inputDigest65c737f9…与State匹配；这与前一回执中durable-745f95f1四文件缺失分属不同精确路径，不是所有marker缺失。该submitted marker省略observationOnly，State显式false，因而原始payload不相同，当前严格replay等同比较也不满足。原字节不得通过归一化、改State或放宽比较来“修成一致”；此差异没有已证实写者/原因。

本次r4的20项声明pin仍限定读回匹配，不代表完整闭包、writer或resource准入。原#364为ABORTED、operation running/result null，compile/真实测试发现执行/root-children-EOF仍unknown；续接须先沿原native/Records链收口，并在现管理lineage资格化single-recipe continuation及精确marker/State语义。旧七文档窗口已经过期/取消，未续期或重新静默，真实captureReady仍false。独立source-owner记录为E根 `material-r4-client-continuation-consumption-23.json`；本轮无QueryOnly、POST、Cargo、新identity、gate/共享State/服务修改或清理，释放0B，SUCCESS未达成。

## r6 原producer的认证终态与容量观察

Parent于21:30:33Z认证API将原queue15016绑定 `zircon-management#366`：building=false/result=ABORTED/artifacts=[]；timestamp+duration为20:42:41.512—20:53:56.900Z，只是Jenkins时间，不能当native EOF。其consoleText为5011B、SHA8cc3e7be…，完成52199输入捕获后出现Aborted by admin/Hard kill/Finished ABORTED，未见Cargo启动行；实际compileStarted、发现及执行测试数仍unknown。admin显示名不确定实际操作者或中止原因。旧本域progressive日志8411B、SHAf72cf77a…和原r6 client/identity逐一保留，两类输出不覆盖或宣称逐字一致；原r6简写的“进程消失”不能作为实际死亡或EOF证明。

同一Parent回执另含21:27Z的E盘43425722368B（40.443GiB）读回。若当时仍适用v11的35GiB reserve与4GiB静态预算，容量条件超过39GiB约1.44GiB；实际ResourceManager reservation、物理E/target、pool/cache writer均未授予，不能以余量替代准入，也不能继续引用较早不足发起盲删。容量增加来源未知，本域与Parent本次清理均0B。

本次只读核对r6 identity26a389f5…及20个声明pin，原payload、stdout和失败未改；证据登记于E根 `material-r6-parent-observation-19.json`。Parent已沿原owner补充#366响应/EOF、reservation和中止原因请求，ACK只证明queued；本域没有重发、QueryOnly或另开r6 producer。r5的独立State/native链仍open，不用r6关闭；较晚r9/r10按上节独立历史保留。本次历史事实补入不表示得到新SUCCESS或Rust通过，后续继续由原执行owner给真实响应、Job/root/children/双EOF/readersJoined及资源释放/实质修复结果。

## consumption04 源码修订与旧输入边界

本次按原 Jenkins owner 的21:08:49—53Z只读回执复核源码；该回执的6d120059胶囊和E盘14.54GiB均是当时观察，不覆盖上节较晚的r9/r10记录。原 `material-managed-validation-r2.identity.json`（30a5b632…）和 `shared-graph-managed-validation.identity.json`（f33013fc…）字节未改；本次分别匹配12/17、10/13。r10较新声明匹配20/20，三者基线不同，不能用r10的匹配结果否定旧声明已漂移，也不能推导完整checkout或全部writer稳定。

五个变化pin能与既有本域源码交付对应：共享三文件的当前哈希逐一匹配17:57:43Z的 `ai-bounds-fit-contract-07.json`；材质两文件及新增模块/测试逐一匹配19:03:24Z的 `material-connection-preflight-11.json`。完整before/after和本次限定文件读回写入E根 `material-source-revision-review-18.json`，本轮没有再改Rust。

| 当前源 | 原声明SHA前缀 → 当前SHA前缀 | 后继修订语义 |
| --- | --- | --- |
| [graph/canvas.rs](../../../zircon_editor/src/ui/graph/canvas.rs) | 23ea48fa… → a072722f… | 现有CanvasState新增 `fit_graph_bounds`，只以已测bounds、局部逻辑viewport/padding原子更新pan/zoom；保留selection、drag、源节点位置和history，无第二次DPI换算 |
| [graph/mod.rs](../../../zircon_editor/src/ui/graph/mod.rs) | 2a0a4cda… → 0ed32746… | 公开 `GraphBoundsFitResult`；原proof/prepared/attachment和方向路由导出保留，构造与permit权限未扩大 |
| [graph/tests/cases.rs](../../../zircon_editor/src/ui/graph/tests/cases.rs) | 31635777… → 9f7e906a… | 新增七项fit回归，旧十九项不改记为已运行 |
| [material lib.rs](../../../zircon_plugins/material_editor/editor/src/lib.rs) | 7ef40319… → 928f63be… | 接入并公开材质专属连线预检模块及四个合同类型，注册新增九项测试 |
| [material graph_contract.rs](../../../zircon_plugins/material_editor/editor/src/graph_contract.rs) | bd94895a… → 5875ae83… | 原编译验证在已有诊断为空时消费同一 `connections::validate_types`，与作者预检共用类型约束 |

旧17项还未声明新增的 [connections.rs](../../../zircon_plugins/material_editor/editor/src/connections.rs)（666a5639…）和 [tests/connections.rs](../../../zircon_plugins/material_editor/editor/src/tests/connections.rs)（2e456df3…）。生产预检借用旧边返回Connect/Replace/Unchanged，先排除被替换输入的旧边，再检查候选拓扑及所有组件的类型；临时索引不是作者快照或撤销history。Undetermined允许未完成编辑，不代表可编译发布。宿主仍须将原边和新边绑定同一真实作者文档revision，经过prepare、一次permit和inverse delta应用；本API没有该生命周期，也没有renderer发布能力。

| 当前源码定义 | 静态数量 | 本次审阅的回归边界 | Rust发现/执行 |
| --- | --- | --- | --- |
| 七项 `canvas_bounds_fit_*` | 7（计入下行26） | 895/629/447逻辑宽、平移与selection/drag保留、四侧padding、zoom上下限和最小zoom仍无法全显的显式结果、零/负/NaN/Inf/溢出保持状态不变、逆变换及滚轮锚点 | 未运行 |
| `ui::graph::tests::` 的cases | 26 | 原十九项加上述七项；另一个 `canvas::tests` 默认值测试不在此filter内 | 未运行 |
| `material_graph_` 三个测试源 | 30（29普通、1原有ignored） | cases 8、output_contract 13、connections 9；新增九项覆盖标量广播/直接normal、类型拒绝、替换修复环、Unchanged/歧义输入、端点/方向/环、未完成math的下游约束、向量math/fanout、重复/空ID | 未运行，未加 `--ignored` |

这些是源码定义计数，没有执行Cargo discovery。原回执中的材质13项和core19项仍各自not_run；新增定义不能补写旧终态。连线测试中的prospective clone仅为测试fixture，不能当生产apply或撤销合同。

原#352仍complete/preparation-conflict/compileStarted=false，#353仍complete/checkout_input_identity_conflict/compileStarted=false；两笔active授权不需要补人工grant，也不证明writer或native EOF。原完整输入/四份查询日志/两项前端查询失败均保留。当前单配方对complete原operation换源continuation不受支持；本次没有QueryOnly、重投、创建身份或ACK。后续仍须由原owner提供受支持接续，取得当前完整成员、实际writer、物理E reservation、pool/cache/target及native writer回执，再由Parent协调有界窗口和真实取消/EOF。旧9root记录不能借给core作为当前capture证明；未声明全局freeze或捕获时长。真实host几何、renderer、正常EXE和150%DPI继续open。

## 材质连线判定与实际宿主接续

本轮沿既有材质 `can_connect` 领域边界，只改本域 `editor/src/{lib,graph_contract,connections}.rs`、新增 `tests/connections.rs` 和本域文档。公共 graph、model/commands/retention、runtime 实例修复、主题、catalog、原 ZUI 保持只读。具体 before/after、来源闭包和验证在 `E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-connection-preflight-11.json`；宿主逐文件提案为同目录 `material-connection-host-handoff-17.json`。

`can_connect_material_graph` 返回 Connect、携带原始借用边的 Replace 或 Unchanged。端点使用现有 node ID 和 pin schema key，不把显示名、边数组索引或新造 UUID 当稳定身份。候选视图先排除被替换输入的旧边，再检查所有连通分量的端点、单输入、环路和类型；不克隆整图或保存新 history。compiler validation 复用同一类型分析，所以纹理数学、向量/纹理到标量输出、非直接 TextureSample 的 Normal 会在编译发布前报同类限制。Scalar/Vector 混合数学、Scalar→BaseColor/Emissive 及直接纹理 Normal 仍保留真实后端支持。

编辑中的缺输入可为 Undetermined；无 Output 或缺 BaseColor 不单独阻止拖线。该许可不代表可编译、有限值已验证、shader 已就绪或已进入 renderer。检查下游约束，避免先把未完成 math 接到 Roughness，之后用 Vector 输入把它变成不支持的标量表达式。九项新回归覆盖这些分支以及输入替换修复原环路、原借用边身份、重复输入、无效节点/端口、方向、自连、source fanout 和原图不变；当前仅定义和静态检查，执行仍 pending。

只读 UE `MaterialGraphSchema.cpp:577` 的 CanCreateConnection 对占用输入返回 BREAK_OTHERS 并保留 source fanout；`:660` 的 TryCreateConnection 在实际改图后更新材质。UE 当前 ConnectionCausesLoop 分支提示警告，disallow 已注释；Zircon 既有 compiler 拒绝 DAG 环路，本轮保留该差异。通用 palette 的 Add/Multiply 仍标 float，而实际 compiler 支持 Vector 数学；这是待 schema/adapter 统一的差异，没有用共享 descriptor 推导领域类型，也没有宣称 UE 全类型转换已支持。

| 实际断点 / 路径 | 最小接续与责任 |
| --- | --- |
| `zircon_plugins/material_editor/editor/src/plugin.rs` 只注册 command descriptor；`dist/src/lib.rs` 的 invoke_command 为 None | 材质域负责实际 authoring handler；沿已存在 `register_operation_command` / `OperationCommandFactoryRegistration` 接入，不新增无状态假成功 ABI |
| `zircon_editor/src/ui/retained/dispatch/workbench/material_workspace_state.rs` 的 Compile 仍给演示反馈 | Parent 公共宿主把原事件定向到真实启用 command 和当前作者文档，保留失败/焦点返回；该文件本轮只读，未暗改公共路由 |
| `zircon_editor/src/ui/host/editor_operation_dispatch.rs` 与现有 EditOperationTarget | Parent 提供真实 document/asset/graph/subdocument、retention revision、当前 transform/input owner 的适配；本域检查结果只给候选，不授予 permit、不绕过 prepare/history |
| 编译结果到真实 material preview / Apply / Save | 本域提供真实 MaterialAsset 结果；Parent renderer/唯一 PreviewSession 给资源 readiness、generation/frame/fence 与失败返回；保存保留作者 ZMaterialDocument，不从 effective MaterialAsset 展平重建 |

下一次托管范围仍是独立 `zircon_plugins/Cargo.toml`、唯一 `zircon_plugin_material_editor_editor`、defaults off/features=[]、locked/lib/test profile、jobs1、`material_graph_`/testThreads1，传递 Runtime native-host / Editor target-editor-host，物理 E:/cargo-targets。连线源码阶段当时未创建新 validate 身份；随后已按用户新授权执行上表 r9/r10，保留原 r3，不运行本地 Cargo。当前先由原入口 owner 恢复真实 manifest 配方，再沿实际 delivery/operation 核对接续；继续声明真实 owned/dependency hashes，按用户授权的 advisory 策略记录其它文档漂移，不冻结全仓或覆盖旧身份。

## r3 准备层边界诊断接续

原管理owner22:47:49Z的边界诊断已按三份输入SHA及引用证据核对。本轮只合并诊断，不提交产品：原 `ue-material-graph-20261024-01 / matgraph-tests-01a16d50-20261024-r3 / queue14773 / #362` 归档e47c126c…仍为checkout_input_identity_conflict、compileStarted=false、原13项未运行，native root/children/EOF仍缺。r2/#352与core/#353各自保留；该owner本轮未生成r4，不影响本域已有后继r4—r10的独立历史。

与原pin相同的e063 runner2e4e…先把完整source inventory的每个成员加入执行输入，之后又递归重读docs等树；因此即使 `08-resource-tools-design.md` 不在作者17项中，也会进入逐文件身份比较。owner用原函数和两原哈希的纯内存判别复现了该冲突类型。总digest与逐文件身份门是两层检查；原每读provenance/写者证据未留，不能据复现指定写者。本域复用这项诊断证据，没有再运行probe、捕获或请求文档静默。

当前限定读回的r3声明为15/17，变化仅是material `lib.rs`、`graph_contract.rs`。两项后继源码及新增connections模块/测试的语义已在本域 `material-source-revision-review-18.json` 中审阅，原17/17仅为历史。该审阅不构成Jenkins当前完整成员/closure或writer准备证明；实际准备ready后仍应固定届时真实版本。现有Cargo与字面module/path边只能提供局部成员线索，不能替代完整feature/host/target图和动态build/proc-macro/resource读取证据。

owner当时selected2d2debd7…的11pins匹配；其观察到canonical及selected对冲突取后次hash、声明失配仅记录。这是当时现场策略，不等于所要求的闭包保护已交付。本域未更改gate、canonical或sealed，也不把待审提案当已重新开启严格门。

`bounded-source-preparation-treatment.json` 是REVIEWABLE_TREATMENT_NOT_IMPLEMENTED_OR_DEPLOYED：全仓inventory继续审计，Jenkins从真实checkout及当前单包Cargo/控制/资源闭包推导执行输入，纳入显式owned/hash和实际包含的module、build.rs、include、工具/配置、生成输入及测试资源。仅审计变化保留双方哈希；include_str/build.rs/资源或声明内文档继续受保护，不能只认旧17项或跳过所有docs。未知动态读取或不完整Cargo图仍未就绪。所提versioned proof不能在全仓变化时伪填originalEngineUnchanged=true；实际resource/cache/pool/writer和native生命周期门保留。

这些均是待实施合同：准备层实现、当前route/recipe资格、聚焦Jenkins验证、immutable发布、完整closure与物理E资源/writer/capture-ready回执均open，captureReady=false。本次没有元数据Cargo、resolver、源码捕获、重投、身份/receiver创建、freeze或服务/数据库/锁/清理动作，释放0B。可追溯记录为E根 `material-r3-closure-boundary-consumption-21.json`；整族产品/EXE/DPI验收不变。

## 范围与来源

用户指定的实际 checkout 是 `E:/Git/ZirconEngine`，源码基点为 `75a76ee963792d5a316f738519e966937f770036`。保留现有共享主工作台、插件和原生捕获改动；公共源码按逐文件授权涉及 `ui/graph/{mod,routing,node_widget}.rs`、`tests/cases.rs`，交付原两类型重导出与兼容方向/箭头/附件身份投影；Parent第五次接续另追加 `canvas.rs`、必要mod导出和原cases的中立bounds-fit，未扩到宿主。model/commands 和 retention 实现保持只读，不改公共 catalog、tokens、Starship 或其它共享实现。Parent 已修正 overview 的早期合并分组，以下按独立 Toolkit 与真实业务归属接续。

### 资源域边界与跨域引用

依据 [Parent 资源交接](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md#资源抽屉与资源工具共享接口交接) 与 [RT-S01～09](08-resource-tools-shared-contracts.md)，窗口与适配职责为：

| 窗口或业务 | 实际 owner 与本域边界 |
| --- | --- |
| Material / Material Instance / Material Function、关联材质工具 | 本材质会话；保留本计划逐项适用/待定判定，牵头 MAT-SHARED-001 图表面、材质槽与纹理引用的材质业务适配 |
| Static Mesh / Texture 独立 Toolkit、Browser / Picker / Import、Reference Viewer | 资源会话 `01a16d53-60db-7d02-b03e-0042f277783d`；材质引用行或预览网格选择器不替代这些 Toolkit |
| Skeletal Mesh / Skeleton / Physics Asset | 角色会话 `01a16d52-6b1e-7543-82f0-4f05ca92ad58`；同一 Mesh resource kind 不足以判定静态/蒙皮编辑器 |
| Niagara / 粒子 | 粒子会话 `01a16d52-a4a5-7271-90a2-336091337737`；消费通用图合同，保留 VFX 自己的模型、编译及 preview |
| 独立声音工具及潜在音频图 | 资源会话；当前未申请音频节点图，SoundCue/Class/Submix 的模型与执行缺口不因共享图可绘制而关闭 |

下表是跨域接入要求，不是新增 Rust schema 或已部署路由。公共宿主、registry/Toolkit、唯一 PreviewSession 和焦点返回由 Parent 协调；本域提供实际作者文档、引用校验和材质命令，不并发修改资源、角色、粒子原资产或公共源。

| 操作 | 必须保留的真实身份与完成边界 |
| --- | --- |
| 双击纹理、材质父项或真实 Function 调用 | 项目 activation、实际 asset UUID/type/locator、源 document/graph/node/field 与 revision/generation；走启用 registry 的 Toolkit/open operation，已开同资产聚焦原文档。资源 UUID 或显示名本身不证明存在对应 Toolkit |
| Find-in-Browser / 引用定位 | 复用资源命令中心和 Browser 选择/滚动；不把 `ActivateReference` 的定位结果记作编辑器已打开 |
| 材质槽选择与赋值 | 消费来源域提供的实际 mesh/skin/primitive/slot 与 revision，不按第一个 mesh 或文件名猜槽。跳转材质编辑器与改变来源资产的槽是不同命令；槽写入/撤销留在来源文档 history，本域负责材质适用性与引用校验 |
| 材质纹理引用编辑 | 本域在原作者文档的 texture 引用字段提交、验证和撤销；资源域负责真实 Picker/Texture 工具及资源 readiness。打开纹理不自动更改材质引用，成功选择也不等于材质已应用到 renderer |
| 打开失败、关闭及返回 | 缺类型、Toolkit、命令、资源或内容加载失败分别保留原因；不切换成另一类编辑器宣称成功，不改变原来源选择。返回只恢复仍属当前 generation 的 node/field/focus，失效则回合法来源 pane |

已只读核对 [asset_event.rs](../../../zircon_editor/src/ui/host/editor_event_execution/asset_event.rs)：`OpenAsset` 解析 `AssetUri`，查询已索引类型、启用 registry 和 Toolkit command 的 enabled 状态，再由 `open_asset_document_view` 打开 view、写入 route metadata 并请求聚焦；`ActivateReference` 只调用 `navigate_to_asset`。这些源码证明路由入口存在，尚不证明材质/纹理/网格各 Toolkit 已注册、真实内容已加载、重复打开策略或关闭焦点返回通过产品验证。资源 [只读依赖图提案](08-resource-tools-shared-contracts.md#材质-owner只读依赖图消费提案) 接入同一 [共享图合同](shared-node-graph-contract.md#资源消费适配只读-reference-viewer)，不增加音频图或第二套依赖图实现。

### Pixel Inspector 与 Audio 的消费边界

按 [Parent 窗口覆盖表](parent-window-coverage.md)，OVW-32 Pixel Inspector 的窗口主体、renderer/readback provider 与采样生命周期归 Parent；材质仅消费真实帧/像素/颜色空间。OVW-36 Sound Cue/Class/Submix 及 Audio 图作者业务归资源，材质只提供既有共享 graph 表面合同；当前没有音频节点申请，不能用材质节点或 compiler 填补音频模型/执行缺口。

| 材质消费点 | 实际输入与未完成边界 |
| --- | --- |
| 帧与采样身份 | Parent provider 提供实际 session/view、frame/generation、资源/source revision、readback ready/error 与物理像素位置/尺寸。输入坐标和 viewport 裁剪/实际 DPI 由同一宿主变换关联；关闭/切资源/帧失效后拒绝旧结果。本轮尚未接该 provider，不采样 Penpot 或静态 SVG 冒充真实帧 |
| 像素与颜色空间 | 消费真实 buffer/texture format、通道/数值类型、采样所在渲染阶段及显式颜色空间/transfer function；scene-linear、pre/post exposure、tone-map 前后、display 值分开。未知元数据保持未知，不把截图 RGB 或统一 gamma 当原始线性材质输出 |
| 原始值、显示值与来源 | 原始采样和经明确转换的显示值分别保留，并绑定同一 request/frame。材质/节点/表面来源仅在 renderer 给出真实映射时可跳转，不从像素颜色猜 asset/node；尚无数据的 GBuffer/深度等字段不填演示数值 |
| 首次实际验证 | Parent 提供正常 EXE 的实际资源/帧和 readback；材质修改确实进入 renderer 后，同一 source/frame 合同核对像素变化、坐标、颜色转换及 stale/关闭返回。当前未执行，不提升材质预览或 OVW-32 的 passed/accepted |

只读 UE `Engine/Source/Editor/PixelInspector/Private/PixelInspectorResult.cpp` 的 `DecodeFinalColor`、`DecodeSceneColorBeforePostProcessing`、`DecodeSceneColorBeforeToneMap` 已核对：final color 的 gamma/alpha 与 scene color 的 OneOverPreExposure/工作颜色空间亮度走不同解码路径。此为消费字段依据，不是要求移植 UE 渲染格式或证明 Zircon 已有等效 readback。本域不另建 Pixel Inspector 窗口、采样服务、音频图或平行主题；缺失能力留在 Parent/资源和本域材质 renderer 连接依赖中。

只读参考根为 `dev/UnrealEngine`。本地 `Engine/Build/Build.version` 声明 `MajorVersion=6, MinorVersion=0, PatchVersion=0, Changelist=0, BranchName=UE5`；这不是可据以推定上游发行版或提交号的版本证明。未发现该参考根的独立 `.git`，以实际文件 SHA-256 固定输入，不把 Zircon 的提交号当成 UE 修订。

以下相对路径均位于该参考根：

| 来源 | 已核对入口及作用 |
| --- | --- |
| `Engine/Source/Editor/MaterialEditor/Private/MaterialEditorModule.cpp` | `CreateMaterialEditor` 的 Material/Function 重载；`CreateMaterialInstanceEditor` 的 MaterialInstance/FunctionInstance 重载 |
| `Engine/Source/Editor/MaterialEditor/Private/MaterialEditor.cpp` | `RegisterToolbarTab`、`InitEditorForMaterialFunction`、图事件、命令映射、Apply、Save、关闭及撤销 |
| `Engine/Source/Editor/MaterialEditor/Private/MaterialEditorModes.cpp` | `Standalone_MaterialEditor_Layout_v17`；图、预览、详情、底部工具与侧栏关系 |
| `Engine/Source/Editor/MaterialEditor/Private/Tabs/MaterialEditorTabFactories.cpp` | `FMaterialGraphEditorSummoner`；激活/失焦、文档历史、图视点和缩放恢复 |
| `Engine/Source/Editor/MaterialEditor/Private/MaterialEditorTabs.h` | Material 与 MaterialInstance 的稳定 tab ID 集合 |
| `Engine/Source/Editor/MaterialEditor/Private/MaterialInstanceEditor.cpp` | `Standalone_MaterialInstanceEditor_Layout_v8`、Details、预览、Hierarchy 和 FunctionInstance 分支 |
| `Engine/Source/Editor/MaterialEditor/Private/MaterialEditorInstanceDetailCustomization.cpp` | 实例参数、分组、覆盖/重置及属性定制入口 |
| `Engine/Source/Editor/MaterialEditor/Private/MaterialPropertyHelpers.cpp`、`Engine/Source/Editor/UnrealEd/Private/PreviewMaterial.cpp` | 覆盖开关、保留开关的重置、原实例实时更新、分组与参数排序；不是所有实例都走 Material 的 Apply 流程 |
| `Engine/Source/Runtime/Engine/Private/Materials/MaterialInterface.cpp`、`MaterialInstance.cpp` | `GetParameterDefaultValue` 跳过当前实例的覆盖，继续查父实例/基础表达式，不能用引擎常量默认值替代 |
| `Engine/Source/Runtime/Engine/Public/Materials/MaterialExpressionFunctionInput.h`、`MaterialExpressionFunctionOutput.h`、`Engine/Source/Runtime/Engine/Private/Materials/MaterialExpressions.cpp` | 函数端口的稳定身份、类型、预览与可选默认、排序、调用连接迁移及循环依赖拒绝 |
| `Engine/Source/Editor/MaterialEditor/Private/MaterialGraph/MaterialGraphNode_FunctionCall.cpp` | 未连接端口的 advanced 展开状态；已连接端口始终可见 |
| `Engine/Source/Editor/MaterialEditor/Private/SMaterialPalette.cpp`、`FindInMaterial.cpp` | 分类/搜索、拖入节点、查找结果及跳转 |
| `Engine/Source/Editor/MaterialEditor/Private/MaterialGraph/MaterialGraphSchema.cpp` | 上下文创建、连接资格/替换、断线事务 |
| `Engine/Source/Editor/MaterialEditor/Private/MaterialGraphConnectionDrawingPolicy.cpp`、`MaterialNodes/SGraphNodeMaterialBase.cpp` | 连接表现及节点 Slate 内容 |
| `Engine/Source/Editor/MaterialEditor/Private/SMaterialEditorViewport.cpp`、`MaterialEditorViewportToolbarSections.cpp` | 实际预览视口和网格/场景操作 |
| `Engine/Source/Editor/MaterialEditor/Private/MaterialEditorActions.cpp` | Apply、预览网格、实时预览、Stats、上下文动作；快捷键以注册与映射为准 |
| `Engine/Source/Editor/MaterialEditor/Private/MaterialStats.cpp` | Stats、旧消息日志、HLSL、平台/质量/实例相关 shader-code tabs |
| `Engine/Source/Editor/EditorStyle/Private/StarshipStyle.cpp` | `Graph.Panel.*`、`Graph.Node.*`、`Graph.Pin.*`、材质工具图标；网格周期、节点标题/引脚字体等 |
| `Engine/Source/Runtime/SlateCore/Private/Styling/StarshipCoreStyle.cpp` | 基础控件与文本样式来源，继续复用主工作台的导入结果 |

## 适用窗口及关联工具

“适用”表示本任务目标，不表示当前后端或窗口已经可用。Material Function 不能因为缺少后端而从用户范围中排除。

| 窗口/页签/工具 | 分类 | Zircon 能力、差距和理由 |
| --- | --- | --- |
| Material Editor | 适用，待设计/接入 | `MaterialGraphAsset` 和 `MaterialAsset` 已有；当前工作区是列表演示，缺真正的图交互与预览 |
| Material Instance Editor | 适用，部分设计、待接入 | `.zmaterial` 有 `parent`、属性覆盖和 `inherit_parent_values_from`；Penpot 已有三尺寸无父项空态和 Hierarchy 弹层设计，产品实例窗口、真实参数分组/覆盖开关、父项导航及渲染证据仍缺 |
| Material Function Editor | 适用，待实现 | UE 复用 Material Toolkit；Zircon 当前六种图节点没有函数输入/输出/调用、依赖及函数编译模型，不能用普通材质图冒充 |
| Material Function Instance | 待定 | UE 模块存在专用创建重载；是否采用函数实例资产须随 Function 的模型决策，不能仅给 MaterialInstance 改标题 |
| Graph / Document | 适用 | UE document factory 和历史恢复明确；需要共享图表面与真实节点/连接模型，现有三行表格不计完成 |
| Viewport / Preview Scene Settings | 适用 | 需要材质专用 renderer 输入、场景/网格和资源就绪证据；公共 `WorkbenchPreviewViewport` 当前是静态 mannequin SVG，不能直接作为材质预览 |
| Details / Parameter Defaults | 适用 | 图参数当前为 `BTreeMap`，缺组名、排序、描述及实例命令投影；shader 已有 property/option 的 `editor` 元数据、texture slot 的 `group/label/editor`，应复用，不能再笼统称所有组元数据都缺失；实例须保留父材质来源 |
| Palette / 节点搜索与创建 | 适用 | 可先投影真实六类节点 `output/texture_sample/scalar_parameter/vector_parameter/add/multiply`；搜索匹配、拖放/连线创建、撤销待接入 |
| Find Results / Overview | 适用 | 由真实图索引驱动，点击须跳转/选择并恢复焦点，不能用假结果列表 |
| Stats / 编译消息日志 | 适用 | 显示实际图诊断、资源/编译结果；不得沿用 `0 errors, 2 warnings` 的演示字串 |
| Platform Stats / 平台、质量、实例 Shader Code | 待定 | 仅能显示引擎真实支持的目标和编译统计；尚无对应多平台统计消费者证据 |
| HLSL Code 页签 | UE 专用内容不适用；WGSL 等效工具适用 | Zircon 使用自己的 shader 合同；等效页须打开实际 shader/编译结果，不能伪造 UE HLSL |
| Named Reroutes | 待定 | 当前资产枚举没有 reroute 声明/引用；需决定其图语义和序列化后再启用 |
| Custom Primitive Data | 待定 | 本轮未核验材质图到实例 primitive 数据的完整消费链，需 renderer owner 明确合同 |
| Layer Parameters / Material Layer / Layer Blend | 待定 | UE 标签及树存在；Zircon 当前材质图无相应 layer/function 模型 |
| Substrate | 不适用当前渲染模型 | UE 专用 BSDF/拓扑工具，不能以空板复制；若引擎以后采用等效模型再另行设计 |
| Material Diff / Revision Menu | 待定 | `Private/DiffTool/Widgets/SMaterialDiff.cpp` 有实际 diff 表面；需真实历史版本与字段/图差异服务 |
| Material Parameter Collection | 待定 | UE `AssetDefinition_MaterialParameterCollection.h` 和 `DetailCustomizations/Private/MaterialParameterCollectionDetails.cpp`；Zircon 尚未核验共享参数资产及渲染广播链 |
| Pixel Inspector / OVW-32 | 材质数据消费适用；窗口主体归 Parent | UE PixelInspector 的实际帧、像素与颜色空间解码是依据；renderer/readback 尚待 Parent 接续，不以图设计或静态 preview 替代 |
| Audio 图作者工具 / OVW-36 | 窗口与业务不属本域；仅共享图表面适配 | Sound Cue/Class/Submix 归资源，尚未申请音频节点；真实 schema/compiler/音频执行缺口不通过材质 compiler 关闭 |
| Material Validation / Similarity / Group Diff / Changelist | 待定 | UE `Plugins/Editor/MaterialValidation/Source/Private/MaterialValidationToolkit.cpp` 及同目录工具；依赖真实资产/编译统计、验证与版本数据 |
| Material Analyzer | 待定 | `Plugins/Editor/MaterialAnalyzer/Source/MaterialAnalyzerModule.cpp` 注册独立 Nomad tab；`Private/SMaterialAnalyzer.cpp` 提供资产选择、继承树、参数过滤、相同 permutation 建议和 CSV。Zircon 尚缺相应分析服务，不能用假统计复刻 |
| Create Material / Create Material Instance 向导 | 适用的关联流程，待接入 | `Plugins/Editor/MaterialAssetWizard/Source/MaterialAssetWizard/Private/MaterialFactoryWithAssetWizard.cpp`：异步选择基材、创建/复制材质、设置实例 parent、完成/取消；复用资源浏览器和真实持久化，不能只注册缺失的 template URI |
| Dynamic Material Editor | 不纳入当前范围 | `Plugins/VirtualProduction/DynamicMaterial` 是另一套虚拟制作材质 authoring 系统，当前无对应 Zircon 模型；不能另建平行材质系统 |
| Mutable / MetaHuman 材质专用查看与工具 | 不适用当前范围 | 依赖各自角色/生成管线；材质编辑的通用操作继续由本族提供 |
| Flatten / 材质烘焙 | 待定 | UE Actions 中有入口，尚无 Zircon 材质烘焙产物/任务合同；未接入前不能给出无效果按钮 |
| Physical Material / Mask | 不纳入图编辑范围 | 资产定义存在，但由物理材质数据与对应 owner 负责；与本族的跳转依赖待同步 |

## 目标布局与交互

Material/Function 依 UE v17：菜单和工具栏归公共 host；左侧纵向预览与 Details/Parameters/Overview 标签栈，中心图及底部编译/Find 工具栈，Palette/Named Reroutes 位于右侧可收起侧栏。源码的 splitter 系数分别表达左侧 `0.2`、中心 `0.80`、图/底部 `0.8/0.20`，不把它们直接解释成三个独立相加的百分比。MaterialInstance 依 v8：预览与右侧属性 `0.70/0.30`；FunctionPreview 不展示 Layer Parameters。

外围采用共享流式/停靠布局；节点坐标是图文档内容坐标，统一经过平移/缩放/DPI 变换，节点内部用实际文本测量与流式布局。缩窄窗口保留图/预览的可访问主视图，通过已有 dock/drawer 机制访问属性和调色板。不能截断按钮标签或为各截图加坐标补丁。细化断点须经真实 Penpot 与产品双端验证。

搜索创建、拖放、接线/替换/断线、节点位置、参数提交、删除/复制/粘贴应产生域命令及撤销记录。选中只改变选择；预览编译、Apply 到正在使用的材质、Save 持久化是不同状态。编译失败保留最后有效渲染版本且展示失败，不能标成已应用；取消弹层返回原 control/node 焦点。Apply 依 `MaterialEditorActions.cpp:31` 的语义更新原材质及 world 使用；保存与关闭逻辑还须对照 `MaterialEditor.cpp` 的 `SaveAsset_Execute`、`OnRequestClose`、`OnApply` 和撤销处理。

已直接核对的命令基线（仍未接入产品）：`Apply` 没有默认快捷键且只在材质 dirty 时可用；`SaveAsset_Execute` 先成功更新原材质再交给资产保存；关闭 dirty 材质有 Apply/丢弃/取消三条路径。`MaterialEditorActions.cpp` 注册 `Ctrl+F` 当前材质查找、`Ctrl+Shift+F` 全材质查找、`Space` 强制刷新预览、`Shift+Space` 预览悬停节点、`Shift+C` 创建 ComponentMask。节点快速创建从 `MaterialEditorSpawnNodes` 配置解析，不能凭习惯硬编码其它快捷键。后端未支持的 ComponentMask、全项目查找、节点预览等动作继续列为缺口，不显示可点击但无效果的完成态控件。Undo/Redo 由 toolkit 映射到 `FGenericCommands`，图编辑焦点决定消费位置；材质窗口不能吞掉文本框自己的撤销。

ReactBits 结构参考映射：主框架 [App Shell 4](https://pro.reactbits.dev/docs/app-ui/app-shell/app-shell-4)，搜索 [Command Menu](https://pro.reactbits.dev/docs/app-ui/command-menu)，属性 [Settings Form](https://pro.reactbits.dev/docs/app-ui/settings-form)。仅借用区域/搜索/表单语义；本任务视觉、停靠和行为目标优先采用上述本地 UE 源码，尚未据这些网页推定具体几何或宣称完成布局。

## Material Instance 的具体状态与操作合同

以下是接续设计和实现的合同，未实现或验证的项目继续标为 open。真实 UE 依据为 `MaterialPropertyHelpers.cpp:589/1199/1285/1407`、`PreviewMaterial.cpp:525/651/901`、`MaterialInstanceEditor.cpp:789/804/1669/1801`，以及 `MaterialInterface.cpp:1420`、`MaterialInstance.cpp:1240`。覆盖开关控制是否向原实例写本地值；Reset 读取跳过当前实例后的默认值，修改参数值但不清除 `bOverride`。普通 Material Instance 的字段修改即时更新 source/preview 并使资产 dirty；工具栏 Apply 只对 Function Preview 实例可见，不能给普通实例套用 Material Editor 的 Apply 门槛。Function Instance 仍待定，其 Apply/关闭提示不能冒充普通实例流程。

窗口沿 v8 的预览/属性停靠结构，右侧 Details 的父材质字段、过滤器、分组、参数行均用既有组件和 flow。每行由覆盖开关、真实参数名、类型对应编辑器、reset 及来源信息组成；长名称/中文允许换行，组展开与滚动位置属于视图状态。实际可编辑行来自资产和 shader 声明，不显示固定三行样例。空参数与过滤后无匹配须区分；无父项、加载失败、类型错误、资源未就绪也分别呈现真实诊断。

| 操作/状态 | 域 authority、事件及逆操作 | 焦点、渲染与失败约束 |
| --- | --- | --- |
| 打开实例 | 保留原始 `ZMaterialDocument`、材料/Shader 的资产身份和版本，读取实际 parent 链。属性行以声明所属域、key、kind 定位；图节点 ID 与同名 shader 属性不是同一个身份 | loading → ready/empty/error；只有实际声明存在且类型匹配的值可编辑。effective 渲染投影不作为保存文档 |
| 勾选覆盖 | 从当前有效值建立一个显式本地条目，单次事务记录原来是否存在及原值；勾选值恰好等于引擎默认值时也必须保留覆盖身份 | 开关保持焦点，字段随 enabled 更新；Undo 恢复原 presence/value，不只恢复画面数字 |
| 提交数值/颜色 | 依据声明检查类型、有限值与支持范围，再更新原始 override。连续拖动先展示交互值，提交合并为一次可撤销域编辑；Esc 撤回未提交值 | 文本输入 Undo 优先；验证失败留在原字段并显示实际诊断，文档和资源版本不变；成功提交后驱动真实资源更新 |
| Reset 值 | 查询跳过当前实例覆盖后的父链/基础默认值，写回本地值并保留覆盖开关；逆操作恢复 reset 前值 | 仅实际值不同才显示 reset affordance；父链不可解析时显示原因，不写入猜测的 0/1。此动作不能等同取消覆盖 |
| 取消覆盖 | 从作者 map 删除对应本地覆盖，重新解析继承；逆操作恢复完整原条目，包括纹理槽附加元数据 | 展示来源和值且禁用字段编辑。之后父值改变应继续传播；不能把继承值写回本地而假装取消 |
| 纹理选择/清空 | 复用真实 AssetReference/资源选择器，按 shader slot dimension 校验，保留 fallback/transform/uv_channel；清空本地 reference 和取消整项覆盖需要独立语义 | 选择器关闭回原字段；缺失/未就绪/不支持类型不显示成功缩略图。canonical slot 的空 reference 是否作为显式空覆盖须先补持久化合同 |
| 修改 parent | 在临时候选上解析注册资产、同 shader、环和深度约束；通过后单次事务更新 parent 与明确的 override 迁移，逆操作恢复两者 | 当前 renderer 上限为四层 parent。UE 会清理无效参数覆盖；Zircon 对不兼容 key 的保留/迁移策略尚待 Parent 确认，禁止静默丢字段；失败保留原 parent、dirty 状态及最后有效帧 |
| 过滤/组展开 | Show Only Overrides、搜索、组展开和滚动只改 view state；组行和参数行来自同一份真实投影 | 不产生材料 Undo。隐藏当前焦点行前把焦点交给仍可见的过滤器/组头；取消弹层回原控件，不能跳到图快捷键消费者 |
| Hierarchy/父项打开 | 父链项绑定真实 AssetReference，交资源 registry/Toolkit 打开；保存当前文档的选择、滚动和返回焦点 | 没有可打开目标就显示缺失原因；不以 toast 或改标题冒充跳转。父链诊断可展开到具体断点 |
| 普通实例实时修改/Save | 编辑后更新原实例的内存资产 revision、预览/使用者，并单独保留未保存状态；Save 用原作者文档的真实 project serializer 持久化，成功后才更新 saved revision | 普通实例不显示无效果 Apply。渲染/编译失败留最后有效帧并保留失败反馈；不能把作者文档已改、renderer 已采用、磁盘已保存合成一个布尔值。Undo/Redo 也须更新实际 source/preview |

字段映射先复用现有数据：`ZMaterialDocument.overrides/textures/options/queue/parent/editor` 是作者文档；`MaterialAsset.property_values/texture_slots/options/queue/parent` 是运行时读取入口。`ShaderMaterialPropertyAsset` 和 `ShaderOptionAsset` 有 `kind/default/editor`，`ShaderTextureSlotAsset` 有 `kind/required/default/group/label/editor`。属性编辑器应消费这些声明，复用同一类型与 required/资源验证。现有 `zircon_editor/src/ui/material_editor/projection.rs::MaterialEditorProjection` 已读取 property `editor.group/label` 与 texture `group/label`，按本地 map presence 产生 `is_overridden/override_value`，并把诊断投影为 source/path/message；应扩展这份投影，不另建 property schema。该实现没有父链来源/继承有效值、options/queue 行或 UE group/parameter sort priority；直接传入展平后的 effective MaterialAsset 又会把继承条目算作本地覆盖，须分开传递原始本地 presence 与已解析来源/值。当前引用检索只见定义和测试，未发现生产工作区消费；存在结构投影不等于实例窗口已接通。

图 `ScalarParameter/VectorParameter` 的 `name/default` 与 `MaterialGraphAsset.parameters` 只参与现有图编译时的取值；任意名称不会自动成为 shader 属性。`compile_material_graph` 当前将表达式结果落为 PBR 数值/纹理字段，未生成对应具名 shader uniform 或实例参数 ABI。把图里的 Roughness 行直接绑定同名 `property_values` 不能证明它会影响原表达式；例如 `Multiply(Roughness,Roughness)` 需要真正的图重编译或参数化 shader 消费。基于图创建实例的参数身份、defaults/overrides、派生输出与父资源 revision 的关系仍为 MAT-INSTANCE-01 接入提案，不允许借改标签声称已支持。

早期源码核对发现两个具体持久化风险，以下原始推论保留；后续 Parent 修复状态单列，尚无 Rust 行为通过证据：

- **MAT-INSTANCE-02：原显式默认覆盖丢失，Parent 补丁待行为验证。** 修复前的 `MaterialAsset::property_overrides_with_schema_v1_defaults` 会把 metallic=0、roughness=1、白色 base_color、黑色 emissive、Opaque 和 double_sided=false 对应 key 删除，未区分 parent 是否存在/本地 key 是否显式存在。例如父 metallic=0.8、子显式 metallic=0，经旧实现导出后可能重新继承为 0.8。旧 `texture_slots_with_schema_v1_defaults` 还可能移除无附加元数据的空 canonical slot，影响“显式无纹理”。实例写入不能仅靠 PBR 投影值判断 override presence；Parent 已在原资产层保留 presence 和空槽，补丁及真实 parent 往返测试身份见下一段，未运行不能写成已通过。本会话保持运行时这些共享文件只读。
- **MAT-INSTANCE-03：编辑元数据投影丢失。** `ZMaterialDocument.editor` 存在，但 `MaterialAsset` 不持有该表，`to_zmaterial_document` 总是生成空 `editor`。分组/作者信息若仅放该表后经运行时对象重建保存会丢失。保留原作者文档是本域保存接入的必要条件；不能把 `MaterialAsset` 声称为完整 authoring authority。未知作者字段和纹理元数据也须随命令/保存保留。

Parent 第四次接续已落实 MAT-INSTANCE-02 的最小原资产修复：以 parent 与原本地 map presence 保留显式默认 PBR/控制项覆盖及显式空 canonical 纹理槽，缺失 key 继续继承，standalone 紧凑序列化保留。原 runtime owner 的五条源码路径与八项正式 codec/继承回归由 Parent 管理，本域不重复修改或覆盖。其原验证请求 `ue-material-instance-presence-20261024-01` 在 source capture 阶段返回 `ordinary_entry_unproven`，Cargo 未开始，不能将静态修复当行为通过。材质插件原请求另为 `ue-material-graph-20261024-01`，终态 `preparation-conflict`，两者不合并、不重放；原输入和失败均保留。MAT-INSTANCE-01 参数 ABI、03 元数据、非 canonical alias 优先级及产品接线继续 open。

Parent 本轮交付回执 `material-instance-presence-01.json` 的实际SHA-256为 `84e4f7970c8cfa90531c0a2e244a581e5d174ad5fcecae608ca5236d10467a3a`；嵌入的原静态guard仍为 `a388be48022fa67f60bbd9e9958dd06deddb072bec6d25a43b8fae690bc3efeb`、0 errors/warnings。当前五路径sourceAfter逐项匹配，runtime原identity SHA `b4984a1c78141ce1c8acc4f1c98d16b7547576b7af32e05c39431be3d4a498be`匹配；八项定义实际引用正式project codec、AssetRef/PersistedAssetReference和inherit_parent_values_from。保留首次路径前置失败、同ID规范路径更正及第二次capture漂移，不从source-fixed、格式/guard或定义计数推导行为通过。详见本域 `material-instance-source-fixed-consumption-03.json`，未运行Parent测试或修改这五条源码。

Parent已复核的rev40中文/长文本原图SHA `b3dcd811cc2237560f27958a9652bb6c8af1ba41c99e019881feeddbcef8e64f`在本域文件中再次字节匹配；本轮复用原图身份，不新造关闭重开或产品捕获。原rev0空库、材质未提交测试、rev40无外链库等记录仍是各自时刻历史；当前摘要使用已经实际提交的插件/core终态及rev55/Shared rev1的消费事实，四个主件不作为完整主题或source-owner/export/跨族A-B通过。

### Parent 序列化修复后的域内作者文档边界

本次接续再次只读核对 Parent 所有的 `material_asset.rs`、`material_asset/value_sync.rs`、`material_asset/tests/value_sync.rs`、`asset/tests/assets/material.rs`、`asset/tests/assets/material/instance_serialization.rs`，不重叠修改这五条路径，也不扩大到 editor metadata、参数ABI或Function模型。八项正式codec/继承回归当前只是源码定义；本域不使用它们代替自己的13项编译器回归或共享core19。既有公共graph四文件授权、旧默认路由和材质can_connect领域边界保持，未新增Rust、重投原请求或创建替代验证身份。

源代码显示，取消覆盖还需由编辑器完成正确的作者文档事务。例如父metallic=0.8、子本地metallic=0.3：若只删除已解析 `MaterialAsset.property_values["metallic"]`，却保留 `MaterialAsset.metallic=0.3`，下一次 `sync_f32_override` 仍会因其不是根默认0而写回本地key。这是源代码推导的编辑器接入风险，尚未执行该场景；不把Parent的presence修复改成对有效材质对象的整张保存，也不要求Parent扩大本轮序列化补丁。

| 域内操作 | 作者文档、预览与逆操作的精确接续要求 |
| --- | --- |
| 取消覆盖 | 在原 `ZMaterialDocument.overrides` 删除本地key，保留删除前typed值用于undo；从更新后的原作者文档重建raw child，再解析父链得到effective值。保存原作者文档的缺席状态，不仅删除已hydrated map或序列化effective clone |
| Reset到父值 | 查询跳过当前实例的父链/基础默认，把返回的typed值写入原本地key并保持覆盖勾选；undo恢复原presence与值。父值不能解析则不提交，不能把Reset当移除key |
| 清空纹理与取消纹理覆盖 | 清空保留本地canonical slot及reference=None，并保留fallback/transform/uv-channel；取消覆盖移除该原作者slot。二者需不同命令，来源/slot key/原完整值和revision进入同一逆事务。非canonical alias优先级仍open |
| 保存与资源更新 | dirty/undo由原作者文档事务持有；保留parent/options/queue/editor和稳定asset/source身份。编译、preview和Save回执分别绑定同一作者revision；失败保留dirty和最后有效帧，不从effective projection反推作者presence |
| 实例字段显示 | 继续扩展现有 `MaterialEditorProjection`；本地勾选取raw authoring presence，显示值和来源另取resolved父链。现有from_material调用若传effective clone会把继承值算成本地覆盖，必须在消费者接入时验证 |

上述是接入合同，尚无实际工作区handler/父链读取/撤销/Save或renderer证据，不能把源推导当新Rust通过。当前修复不保留 `ZMaterialDocument.editor` 的既有运行时丢失问题：`to_zmaterial_document` 仍创建空editor表，MAT-INSTANCE-03继续open，Save仍应由持有原文档的owner执行。

13:35:33Z的Material rev40/330对象/21copies和Shared rev1是Parent原时点观察。16:47:23Z独立真实API当前读回Material rev55/1923对象/5主件/102copies，69个实例正式引用Shared rev1四主件、shapeRef错误0；colors/typographies均0，既有保存关闭重开和逐图证据按rev55复用。只读取本域文件和已发布库，无设计/主工作台/Shared修改；两组事实按时间分列，旧空库或旧rev40不覆盖当前状态，当前69实例也不证明source-owner/slot/codec、全局A/B或EXE已通过。五条只读源指纹和反例为 `material-instance-owner-boundary-02.json`，位置见本文末E根报告目录。

## Material Function 的具体状态与操作合同

UE 函数并非改名后的 Material Output。`MaterialExpressionFunctionInput.h` 定义 input name、description、稳定 Id、type、PreviewValue、UsePreviewValueAsDefault 和 SortPriority；`MaterialExpressionFunctionOutput.h` 定义 output name、description、稳定 Id、连接与 SortPriority。`MaterialExpressions.cpp:14451` 按 SortPriority 构造调用端口，`:16463` 用 Id 迁移同一函数的连接并传播改名，`:16377` 切换不同函数时才按名称匹配原输入/输出。`:16377` 同时拒绝递归依赖；`:16268/16918/17374` 区分可选输入、缺失必需输入和缺失函数输出。`MaterialEditor.cpp:8783` 的双击打开真实函数资产；`MaterialGraphNode_FunctionCall.cpp` 令所有连接端口保持可见，advanced 开关只控制未连接端口。

Zircon 的实际 `MaterialGraphNodeKindAsset` 仍只有 Output、TextureSample、ScalarParameter、VectorParameter、Add、Multiply；`validate_material_graph` 要求唯一 Material Output 和 base_color。`direct_references` 仅枚举 shader/texture，没有函数依赖。以下提案需要 Parent 明确 runtime authoring/依赖 owner 后实施；不得将现有 Material Output 改标题、加入空 panel，或把两份图资产相互拷贝当作函数调用。

| 设计与命令 | 需要的真实模型及行为 | 撤销、焦点与错误约束 |
| --- | --- | --- |
| 函数文档/创建 | 复用现有图节点/边及共享表面，增加明确的函数文档种类、输入/输出声明和调用表达式；资产格式/扩展名及注册由 authoring owner 定义，不在 UI 临时发明私有文件格式 | 函数允许多个具名输出；没有输出时显示实际编译诊断。不能调用要求唯一 PBR root 的现有编译器后假称成功 |
| Input/Output Details | 每个端口拥有独立稳定 ID、name、type、description、sort priority；Input 另有 preview/default 策略。排序仅改显示顺序，改名保持 ID | 创建/删除/重命名/改类型是带 inverse 的图事务；重复名称或不支持类型拒绝提交且返回原字段。删除端口连同受影响连线一次撤销，不留下悬空引用 |
| 预览与可选默认 | 编辑函数时可用实际预览输入；调用时只有显式 UsePreviewValueAsDefault 才可将未连接输入视为可选 | 必需输入缺失必须定位实际 call/input；预览值不能悄悄变为调用默认。当前 Scalar/Vector4 表达式可作首批实现基础；UE 其它类型逐项列缺口，不显示假支持的完整类型菜单 |
| Call/同函数更新 | Call 保存真实函数 AssetReference、签名 revision 和以端口 ID 连接的映射。重命名/重排后按 ID 保留连线，并刷新标签/顺序 | 类型不兼容、端口删除、目标缺失均留下具体节点/端口诊断和可恢复的原关联；不按新列表索引串错线。Undo 恢复签名与关联 |
| 切换调用目标 | 复用真实资源选择器，先读取实际新签名；对应 UE 不同函数按名称迁移的行为，只在类型兼容且无歧义时保留连接 | 目标及连接迁移一次事务，取消保持原 call。循环依赖在提交前拒绝；回到原 call/control 焦点 |
| 展开/打开/返回 | 未使用端口展开复用共享 graph advanced affordance；已连接端口一直可见。双击真实调用资产交 registry 打开，保留父图视点/选择与导航历史 | 隐藏端口不得删除边；展开状态的持久化位置须明示。关闭子文档返回真实原调用节点，不创建静态“函数详情”替代编辑器 |
| 编译/Apply/Save | 扩展真实依赖枚举与无环检查，将函数引用、签名和实现 revision 纳入编译失效判断；Apply 更新原函数并使实际依赖材料重新编译/加载；Save 持久化原函数文档 | 过期编译结果不能覆盖新修订。依赖材料失败保留最后有效渲染并可跳到函数/调用链，不能以“函数自身无错误”宣称所有使用者完成。关闭 dirty Function 依 Material Toolkit 的 Apply/丢弃/取消与保存合同 |

MAT-FUNCTION-01 的最小共享接续是 authoring 的函数种类/签名/AssetReference、真实依赖加载及编译修订合同；材质插件负责对应域编译/命令适配，共享表面继续复用 `GraphModel/GraphCanvasState/GraphDeltaCommand`，不另造图框架。首批类型、同名参数作用域、不同函数替换时的歧义处理及 Function Instance 资产是否采用，均须记录实际能力/设计决策；这些未决项不撤销用户指定的 Material Function 目标。

实例/函数后续聚焦验证至少包含：显式默认值覆盖 parent → Save/重开仍覆盖；Reset 保留覆盖与取消覆盖的父项传播差别；纹理元数据和显式空值往返；同 shader、不同 shader、环、深度/缺失 parent；图参数通过真实表达式改变帧；普通实例不经 Apply 即更新真实 source；函数端口改名/排序保留连接、删除和撤销恢复、必需/可选输入、缺失输出、循环函数、签名变更后依赖失效、双击与返回焦点。所有这些用例当前均 `not_run`，未用静态推论或 Penpot fixture 代替实际渲染/持久化证据。

## 原始资产、组件与共享提案

Parent 本轮 AB-SHARED-02 已接续到[同一共享图合同的 Actor 节](shared-node-graph-contract.md#actor-消费适配ab-shared-02)。明确 asset/document/graph/subdocument 与 retention revision、稳定 pin schema key/独立显示名、动态 pin/rename/split-recombine 的真实 remap、单次 DPI、drag preview/release 一个 delta/Esc 取消、带 request/source revision 的 node/pin 诊断与源跳转。Actor 拥有真实类/事件/构造/接口 schema、compiler、文档与存储；公共表面不承担 Actor VM、不另建快照 history。当前完整 BP backend 尚缺，AB-004/005/006 保持 open；本轮仅更新本域合同，未改 Actor 原资产/源码、公共实现或 Parent 记录，也未提交编译。

原始 v2 资产：[material_workspace.zui](../../../zircon_editor/assets/ui/editor/modules/core/rendering/material_workspace.zui)。它的 `WorkbenchMaterial*` control ID、组件引用、绑定和事件必须保留并从真实 Penpot 导出回写。本轮尚未改该资产。插件注册的 `plugins://material_editor/editor/graph.zui` 及 `plugins://material_editor/templates/default_material_graph.toml` 在插件目录中未发现实体文件；不能将注册描述当成已加载的产品资源。

已沿原资产事件继续追到 `ui/runtime/builtin/module_template_bindings.rs`、`ui/retained/dispatch/workbench/material_workspace_state.rs` 和 `ui/retained/dispatch/templates/workbench/module_field_edit.rs`。原 ZUI `route` 与绑定消费的 action ID 并不总同名；实际绑定以 `WorkbenchModule/...` ID 注册，导出不能把 route 字符串直接改成另一套命令或丢弃原 ID。完整逐节点/13 事件只读清单登记为本任务产物 `material-owner-bindings-01.json`，不是 Penpot 导出或产品执行回执。

| 原 control / 绑定身份 | 当前注册 action 与实际效果 | 后续正式接入约束 |
| --- | --- | --- |
| `WorkbenchModuleMaterialWorkspace` / `WorkbenchMaterialWorkspace` component | `workbench.module.material.select` 挂载原资产并初始化第一条固定 profile；`module_workspace_mount.rs` 对非 Scene 入口有 `preview_features_enabled` 门槛 | 保留文档/组件/root 身份和 source receipts；正常 EXE 的入口、真实 asset context 与能力启用需 Parent 接入，不能凭强开演示门槛认定实现 |
| `WorkbenchMaterialBaseColorRow/RoughnessRow/NormalRow` / `WorkbenchModule/MaterialBaseColorRow/MaterialRoughnessRow/MaterialNormalRow` | `workbench.module.material.base_color_row.select/roughness_row.select/normal_row.select` 只同步两个固定 row 的 selected 和标题字串 | 改为实际参数/所属节点选择、Details 和图定位，保留选择与焦点可回溯身份；空数据无固定假参数 |
| `WorkbenchMaterialNodeRow01/02/03` / `WorkbenchModule/MaterialNodeAlbedo/MaterialNodeRoughness/MaterialNodeNormal` | `workbench.module.material.node_albedo.select/node_roughness.select/node_normal.select` 同样只读静态 profile，没有真实 GraphNodeId | 与共享图 host 的动态 node/sourceRef 映射须 Parent/导出 owner 逐项定稿；不能把这些演示 ID 强制当实际节点、删除原事件或隐藏假行伪称完成 |
| `WorkbenchMaterialCompileButton` / `WorkbenchModule/MaterialCompile` | ZUI route 为 `workbench.module.material.compile`，真实 action 为 `workbench.module.material.compile.invoke`；当前只写 `Not implemented (preview)` 和预览输入文字 | 保留绑定/route 对照，接实际 compile operation、诊断 revision 和 renderer readiness；编译反馈不是 Apply 或 Save |
| `WorkbenchMaterialDomainDropdown` / `WorkbenchModule/MaterialDomainEdit/MaterialDomainCommit` | Change/Submit action 分别为 `workbench.module.material.domain.edit/commit`，只改控件 value/value_text；Surface/Post Process/Decal 是原演示选项 | 当前 MaterialAsset 无对应 domain 字段；须实际 shader/domain capability 定义支持范围，不能把下拉选项当引擎能力 |
| `WorkbenchMaterialBlendDropdown` / `WorkbenchModule/MaterialBlendEdit/MaterialBlendCommit` | `workbench.module.material.blend.edit/commit` 只改控件；Opaque/Masked/Translucent 未写资产 | 对照真实 `AlphaMode::Opaque/Mask{cutoff}/Blend` 做事务，Masked 必须能编辑/保留实际 cutoff；Change/Submit 和取消/Undo 不合并成无逆写值 |
| `WorkbenchMaterialPreviewField` / `WorkbenchModule/MaterialPreviewEdit/MaterialPreviewCommit` | `workbench.module.material.preview.edit/commit` 只改 Sphere 字串，Compile 把它拼入标题 | 复用真实网格选择/PreviewSession 与资源身份、ready、输入和释放；没有实际 mesh/render binding 时不宣称改变预览 |
| `WorkbenchMaterialCenterTitle/OutputRow` | 原始固定 `M_Rock_Cliff` 和 `0 errors, 2 warnings`；选择/Compile 后改成其它演示文字 | 标题取实际资产名/dirty 状态，输出来自同一次编译的真实诊断与定位；无数据/未编译时不得沿用旧示例结果 |

原资产无 Save/Apply、参数覆盖/重置、函数导航或图创建/接线事件；这些操作需接 toolkit/document 命令和共享 host 的实际 undo/focus 路由，不因已有 generic field `.commit` 入口就声称已具备。共享组件、导出和 dispatcher 文件本轮均只读。

继续复用 [catalog](../../../zircon_editor/assets/ui/editor/components/catalog.toml)、[editor_tokens](../../../zircon_editor/assets/ui/editor/theme/editor_tokens.zui)、[Starship](../../../zircon_editor/assets/ui/editor/unreal/starship.zui)。按钮、字段、数值输入、属性行、面板标题引用现有 component ID 和 slot；真实文件和图提案组件身份见下文，公共基础组件/样式库映射仍待 Parent，不另建全局主题。

共享提案 **MAT-SHARED-001**（Parent 统筹公共组件/目录；依 `.codex/sessions/ue-window-suite-parent-01a16d4e.md`，材质会话牵头节点图表面的设计与实现提案，未获得覆盖其它会话文件的许可）：

已接续 Parent 最新分工并核对动画、粒子、AI、Widget、角色和 Actor 的实际消费者需求，具体跨域接入见 [共享节点图合同](shared-node-graph-contract.md)。其中逐项列出当前 API 能力、外部插件最小重导出、无边模式、有序重接、方向/箭头、附件身份、稳定 pin key、retention/焦点和证据边界；早期只读合同与最新四文件实现分别保留。该合同初稿的只读库存为材质 revn12/105 对象/3 主件、公共 revn0/仅根/0 主件；后续实际设计版本见下文，不能让较早“尚无分配/零主件”覆盖已保存身份。

Parent 最新 AI-C01～12 交接已细化到[同一提案的 AI 接续节](shared-node-graph-contract.md#ai-消费适配ai-c0112)及[聚焦验证](shared-node-graph-contract.md#ai-聚焦验证与执行前提)：C05 保留已有 proof/prepared 导出；C03 保留 AI 一次有序 Reparent；C04 不改 budget/retention；C06 host 只施加一次实际 DPI。最新四文件授权已实施 C07 的可选四向路由、路径归一化切线/目标箭头和 C09 的原 attachment ID/host ID/类别/名称投影，旧 API、字段、默认水平结果及 GraphNodePresentation 原 struct literal 保留。新增7项回归源码，原12项测试主体和只读 model/commands/canvas 指纹不变；core `zircon_editor --lib ui::graph::tests::` 合计19项未编译/执行，不能与材质原13项终态准备冲突混记。精确 diff/前后 hash/UE 来源登记 `graph-neutral-presentation-02.json`，最终静态检查另记新 guard。四向辅助不声称 UE 1.5线宽/4.5分离/brush 或任意锚点差 tangent 已还原；实际 bounds/instancePath/独立命中/选择焦点/文本行框仍待真实 retained consumer，C06/C07/C09保持 open。AI 的 `AiBehaviorTreeAsset::edited` 仍是 clone/校验/编译候选值，尚非预算准入后的文档事务；AI 拥有单父/root/child-count/顺序/整图和文档 adapter。Decorator 保留原 wrapper，Service/完整 trace 缺口继续 open。该四文件实现阶段未创建 core Cargo identity；后续独立请求的真实 #353 终态另记下文，不回改该阶段历史。未重投材质或动入口服务。

| 需求 | 公共组件合同与材质依赖 |
| --- | --- |
| 图视图 | 单一 graph transform、网格、裁剪、缩放/平移、框选、命中、焦点和原生捕获几何；节点身份取 `MaterialGraphNodeAsset.id`，连接身份取稳定端点，不保存截图坐标 |
| 节点/引脚/连线 | 可复用组件实例、typed pin、输入替换、预览连线、合法性反馈和连线命中；消费已有 graph palette descriptor，材质语义/编译归材质插件 |
| 搜索/上下文菜单 | 复用公共 command/search/popover；带来源 node/pin/context，Enter 创建、Esc 取消、返回焦点，拖入同一 command path |
| 属性编辑 | 复用属性行、NumberField/Field/Dropdown 与撤销事务；材质提供组/排序/继承、覆盖状态和验证，不另建主题 |
| 真实预览 | 公共视口须能接 renderer 外部图像、ready/generation/resource identity 及输入；本族提供 MaterialAsset/mesh/scene，不使用现有静态 SVG 作为完成证据 |
| 状态与导出 | Penpot 组件实例和共享样式的身份、slots、bindings/events、局部覆盖/重置需随导出保留；业务状态由产品投影，不能把 fixture 文字写成真实状态 |

现有共享 Rust 基础已核对，必须复用：`zircon_editor/src/ui/graph/model.rs` 的 `GraphModel`、`GraphNodeView`、`GraphPortRef`、`GraphEdgeView`、`ConnectVerdict`；`canvas.rs` 的 `GraphCanvasState`/选择/框选/拖动；`routing.rs` 的 Bezier/Orthogonal route；`commands.rs` 的 `GraphDeltaCommand`、`GraphEditContext`、`GraphClipboardModel`。`node_widget.rs::GraphNodePresentation` 是 renderer-neutral 投影，存在这些类型不能证明 retained host 已有可交互图表面。当前 `GraphModel` 的实际实现只在该目录测试中发现，材质插件尚无域适配器。

| 接入合同 | 具体约束与待实现项 |
| --- | --- |
| 坐标与命中 | `GraphCanvasState` 的 screen 坐标使用 pane-local 逻辑坐标；物理输入先减原生窗口/pane 原点并除真实 DPI，再调用 `screen_to_graph`。绘制走反变换后仅由 host 施加一次 DPI。pan/zoom 不写进节点资产；节点 `position` 继续保存图坐标。pin anchor 来自测量后的节点内容，不从截图猜坐标 |
| 绘制与组件 | node title/字段/pin rows 用共享文本/字段和流式布局；只对图节点/连线采用画布放置。材质连接按 UE drawing policy 不画结束箭头；先复用已有 Bezier route，差异在实际曲线与命中测试中核验。图内节点不再以表格行代替，普通工具栏/面板不能借画布例外绝对定位 |
| 连线资格 | 共享 `default_connection_verdict` 只检查同型值及 Tree 的单入边；材质使用 Dag 并额外校验同一输入只保留一条连线，替换必须是一个可撤销事务。既有 compiler 允许标量展开为 Base Color/Emissive，默认严格同型规则不能直接作为材质最终 verdict；需要显式域转换反馈。Normal 只接受直接 TextureSample 的当前能力也由域 schema 判断，不能凭 `vec4` 类型放行所有表达式 |
| 撤销与取消 | 消费既有 `GraphDeltaCommand` 的 revision-bound、retention proof、prepare/apply 流程；不另建无预算的整图快照历史。拖动中只改变预览，释放时一次域 delta；Esc 丢弃未提交预览。创建、移动、接线替换、断线、删除、参数提交分别拥有明确 inverse；错误保证模型不变 |
| 搜索、剪贴板与焦点 | 节点 palette 来自实际 `GraphNodePaletteDescriptor`。搜索提交与拖入走同一个创建命令，取消返回原 node/pin/control 焦点。剪贴板用 `GraphClipboardModel` 的域序列化和稳定 ID remap；参数名称冲突须明确提示/选择，不静默覆盖原图参数。文本框 Undo/复制快捷键优先于图命令 |
| 文档及渲染 | 资产文档是唯一图 authority；共享表面持有选择/视点等 view state，读取域节点/连线投影。编译成功产物、已应用资源 revision 与已保存文档版本分开记录，shader/texture 未就绪不能标为完成。native sourceRef/instancePath 必须回溯实际图节点/引脚/路径及文本行框，用于 LayoutReviewCase，而非仅提供装饰图像 |

材质 Penpot 文件已由 Parent 在真实服务中创建并以第二个认证上下文复读：file `b4a1e3ce-9e6a-81fe-8008-d69d6092d704`，初始 page `b4a1e3ce-9e6a-81fe-8008-d69d6092d705`，project `b4a1e3ce-9e6a-81fe-8008-d69d606f6160`，team `822661f1-7ba3-81f7-8008-c6dcb4ae6a8f`。直接入口为 [01 Material Editor](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d6092d704&page-id=b4a1e3ce-9e6a-81fe-8008-d69d6092d705)。分配时 `revn=0, vern=0`、page object count=1；以下为接续后的真实图提案，不等同整个窗口完成。

公共组件文件 `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` 在初次分配时同样为空。当时已请求 Parent 提供认证 storage 的本地路径及基础库迁移/链接方案（消息 `01a16d68-85d9-7842-9fc7-0c750966cefd`），在真实 component IDs 和样式 owner 明确前不复制一套按钮/字段/主题。允许独立隔离浏览器设计，但尚未获得桌面输入/原生捕获时段；后续实际库存见下文，初始空文件不是当前状态。

随后材质会话成功复用了 P21 的既有认证 storage，只在内存中使用 cookie，经真实 API `get-file` 独立读取三个文件均返回 HTTP 200（`Accept: application/json`），未打印 secret、未另建账户、未改原主文件。读回主工作台 `revn=2/isShared=false`，其 `data.components` 仅有两个 `WorkbenchAxisValueField` 和一个 `WorkbenchComponentPropertyRow`，无 `colors`/`typographies` 字段；公共文件和材质文件无 `components` 字段。认证依赖已解决，基础组件与共享样式库仍待 Parent 建立可复用映射。该实际库存和现有 Rust 图基础已追加交接，消息 `01a16d6d-01cb-75d1-b437-3ab8b123c0e8`；API 读取不是设计保存关闭重开验证。

认证与基础库第三次接续已核对：继续只读使用 Parent 指定的 P21 `authenticated-storage-state.json`，只在内存中使用 cookie，不复制、覆盖或输出其内容。2026-10-24T15:06:51–52Z 的3次独立只读 API 库存读回均成功，前后端 readiness 均HTTP200；这是3个文件的实际 API 读取，不声称创建了3个独立浏览器 context。主工作台已为 rev7、477对象、3主件/13实例，仍未发布；Shared 已为 rev1、41对象/4主件、已发布，但 colors/typographies 均0；材质当时为 rev48、1063对象/5主件/69实例，已链接 Shared rev1。原12:13Z 的 rev5/rev0/rev40 观察保留为历史，不能覆盖新库存；回执 `penpot-auth-library-reconcile-03.json` 保留实际时间和身份。

明确沿用 Parent 决策：不发布/链接原主工作台整文件作为基础库；唯一公共目标仍是 Shared `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` / page `b4a1e3ce-9e6a-81fe-8008-d69d607aac13`。原主文件两个同名 Axis 的 component/main 分别为 `cabdc6c1-a15d-8065-8008-d6964a1dc6c7` / `cabdc6c1-a15d-8065-8008-d6964a17b7f2`、`cabdc6c1-a15d-8065-8008-d6964a8d5b45` / `cabdc6c1-a15d-8065-8008-d6964a88f034`；PropertyRow 为 `cabdc6c1-a15d-8065-8008-d6964b9d4171` / `cabdc6c1-a15d-8065-8008-d6964b982617`，未合并或迁移。Parent 持公共 ownership，原 pipeline owner `01a0f04d-9d54-7fe2-9adf-120186804cfb` 负责9项 catalog 原源的 canonical 主件/样式、旧映射、链接和官方导出。材质只消费已存在身份，缺少的输入、Dock、样式继续交接，不重复造底层控件或改公共源。

### Shared rev2：已发布样式与原生同步检查

2026-10-24T18:34–18:44Z 真实 API 与隔离原生 Penpot 的库面板均确认 Shared rev2/isShared=true、原4个主件、34个颜色和3个文字样式。文字样式 body/caption/strong 分别为 `99eb86f9-12ff-5935-96ad-3d1a985e61f0`、`c0224e78-f918-526f-8026-a76cd8f347af`、`9ac86733-8214-5a04-820a-9074b5c68310`。不再以旧 rev0/无样式阻塞消费；早期 rev1/style0 按原时点保留。勘误：原 `pfx-empty-ports-contract-08.json` 的 `domain=shared` 当时已经是 rev2、34色/3文字样式；错的是本域消息及两份文档的“rev2/style0”表述，不是库在其后才发布。consumer 自身 `styles=0` 不能推导库为0。原 JSON/SHA `2a950ff2d42eaab7173eefe54a375aee2ddb500727ed5d0e1426e833453ce53b` 原样保留，Parent 19:36–19:38Z 的独立读回也确认同一库计数；勘误不回放发布、改 Shared 或授予样式传播通过。

需要区分发布集合和有效引用：Parent `settings-plugins-followup-05.json` 的21条 `styleReferencesInMasterShapes`，`refId/refFile` 实际全部为 null。本次直接遍历 Shared 原生数据同样只找到42个空引用属性，即21对空槽，有效颜色/文字样式引用均0；材质有效引用也为0。四个组件的 `canonical-record` v1 和 `canonical-source` metadata 均存在，实际原生 namespace 是 `shared/dev.zircon.zui`，没有将空引用误报为组件或源 metadata 缺失。

在本域现有69个 linked Shared 实例上检查原生「素材 → 管理库 → 更新」：库页显示“4个组件、34种颜色、3个排版素材”，更新页显示“没有需要更新的共享库”，未提供该库的 Update 按钮。因此本轮没有执行原生同步、修改 `synced-at`、reset、detach、复制 main 或生成设计新 revision，也不能记作样式传播生效。关闭浏览器后新进程重开，再次得到相同结果；独立 API 读回 Material 仍 rev55/1923对象/5业务主件/69份公共实例。1923个原对象逐一 SHA 和完整 data 摘要均未变化，实例 source/component/shapeRef 与局部覆盖保留；Shared 完整 data 摘要也未变化。本轮验证的是既有已保存文件重开，不是一次新样式修改的保存或产品加载证明。

上述18:34Z阶段的本机 catalog SHA 为 `e4bba09c3ba16d55b90eabb5a821ab3e76eb0606f9afc17e9f656e2d16fe0a99`，不同于 Parent 消息较早的 `4c2f849c…`；仍读到68组件、NumberField 10 props、PropertyEditorRow 的 required/single value 与可选 description/source/reset 共4 slots。只消费这些声明，不修改公共 catalog 或给未发布 NumberField 主件预填 UUID。模板默认42、0..100、precision0不作为材质实际 schema/value；输入绑定、reset 的作者文档事务及数值 ABI 继续等待真实宿主接入，不能借 f32 声称保证 u64/i64 精度。

回执 `penpot-shared-styles-consumption-10.json`、`penpot-browser-09/receipt-01.json` 与 `image-review.json` 保留完整 UUID、before/after 哈希、三张真实原生工作区图、原选择器失败及更正。`penpot-shared-style-owner-handoff-16.json` 将空引用与无可用更新入口交 Parent/原 public owner，等待其实际 master 绑定和同步/覆盖/reset 回执；本域不写 Shared。CJK/引擎字体度量、正式 codec 导出、正常 EXE、150% DPI 与跨族 A/B 继续 open，published 不等于 passed/accepted。

材质独立托管 r3 已按用户新授权真实重提为 queue14773/Jenkins #362，终态 `checkout_input_identity_conflict`，冲突为外域 `08-resource-tools-design.md`；在原r3终态的历史读回中，本域6源码及11依赖声明哈希17/17一致，`compileStarted=false`，原13项未运行；该17/17不代表当前源，后继限定读回为15/17，见上文r3准备层边界诊断。该终态与原 #352 完整源码 digest 失败、core19 #353 分列，详见 `material-managed-validation-r3.json`；未盲投后续请求，本轮没有新增 Rust 或插件执行。

### Shared rev5 公开引用边界

原public owner03:45Z独立GET及本域原r16/consumption10的回顾记录在style-source-binding-boundary.md，SHA18edb7971226a6b81b026be4ac2733b304c793199f375299938b7cdba4d35350。Shared rev5的22对引用中21对空，唯一有效颜色引用是Button main 6ddc248b-bd35-80c5-8008-d6c8bd9b057f 的fills[0]，绑定Shared file中的editor.surface.1 / 599586e3-022c-500b-ac80-5d9c490139af；其modifiedAt为2026-10-24T20:30:23.587Z，属于历史rev2观察之后，本域未写。Material仍rev55/1923，69根sourceRef有效但640对样式引用全null；原生Update/reset/覆盖保留、正式owner导出和跨族A/B均未通过。18:34Z的零有效引用及“没有需要更新的共享库”保持原时点，不据此否认后继1个绑定，也不把后继绑定回填原消费成功。

原r16例子leaf 6ddc248b-bd35-80c5-8008-d6c8c4c69ae5 实际属于WorkbenchTreeRow main 6ddc248b-bd35-80c5-8008-d6c8c452bf0c / component 6ddc248b-bd35-80c5-8008-d6c8c7a79944，先前Button标注有误。content/positionData typography引用仍为空；按真实source/子树身份处理，不能因误标给TreeRow套Button strong。原回执、旧标签及哈希不改写。本轮catalog9556e66d…/71项与先前e4bba09c…/68项分别保留，原NumberField/PropertyRow声明不重apply，不为未发布主件造UUID。

公共source→样式引用→原owner导出的实际断点已定位：singleSolidFill/Stroke拒绝非空ref，applyPenpotTextStyle未绑定公开Typography。修复须沿原37项style UUID/token-source映射和真实source path/node/state、source SHA、闭包/版本与类型，把fill/stroke/text引用解析接入原capture和source-delta；未知/半引用继续拒绝。保留原$token/imports/IDs/events/slots/bindings，token变化归原theme source owner；不删ref、flatten、按hex猜token、只贴typo ID或放宽严格guard。原19002 guard5/Shared guard7 SDK与worker/UI/runner/font同配置准入仍由public owner/Parent处理，本域不启host、改dist或请求重建。本轮只读消费与文档更正，未新增绑定、更新/reset、导出、浏览器/桌面、编译或服务操作，释放0B。

### 既有图表面设计与保存重开记录

2026-10-24T10:29Z 已通过真实 `update-file` 保存 MAT-SHARED-001 图表面提案，`get-file` 独立重读 `revn=2/vern=0`，104 个对象、3 个原生组件主件、5 个组件实例、45 个 flow frame、4 条可编辑 path 连线。board `2fa31016-62fd-45d2-82d7-83457be89e6c`，大小 1280×800；组件身份：Parameter `904a343b-18f1-4af9-928f-b7d5d028a69a`，Math `f9ea0da8-6a97-4ccb-b57d-b31b273ea4f0`，Output `d7173cd6-1da7-4897-be89-08583d32f598`。这些是本域的共享图提案主件，未发布为公共 catalog/library，也没有新建按钮/字段/平行主题。颜色和字体只取现有 Starship，公共 Penpot 共享 style 引用仍 open；图中的参数值是明确的设计 fixture，不充当真实引擎材质数据。

实际隔离 Chromium 已打开该文件、关闭第一个 context、用相同既有认证创建全新 context 后重开，并关闭 browser。初版逐图检查发现 native flow 的 stack order 倒置，实例文字显示主件默认值。`revn=3` 的 `reorder-children` JSON `shapes` 为 `any`，没有解码其中的 UUID，revision 增加但顺序没有变化；改用 Penpot 源码提供的 typed `assign` 后 `revn=4` 重读和实际 SVG 均确认顺序生效。覆盖标记已修为源码定义的 `content-group`，但 API 文字与画布仍不同；没有把该修正假称为视觉通过。

随后通过原生 Layers 展开/选择 → Enter 编辑 → Escape 提交的真实 Penpot UI，完成 Metallic、Base Color、Vector Parameter 及数值覆盖。每次实际 `update-file` HTTP 200 后重读文字和真实排版 `position-data`；`receipt-07.json` 记录一次成功保存后自动化误用 SVG `innerText` 的读取错误，修为 `textContent` 后 `receipt-08.json` 完成保存、关闭 context、全新 context 重开和 browser 退出，五项实际 SVG 文字均匹配，零 page error。UI 保存和测量数据解决了此设计文件中的显示差异；未据此认定 Penpot 本身或公共插件的根因已经修复。

Multiply 的两个必需输入均已接线；图仍是明确的设计 fixture。依据实际 SVG 引脚中心修正四条 path 起点的 2 逻辑像素偏移，保留全部组件、实例和 path ID。该阶段文件 `revn=12/vern=0`，105 个对象、3 主件、5 实例、45 flow frame、5 连线。`receipt-09.json` 再次完成打开/关闭/全新 context 重开/退出，五条线的起终点到 native SVG 引脚中心最大误差约 `0.00013` 逻辑像素。这只是 Penpot 提案内部几何检查，不是 UE/产品双端误差或窗口族的 `passed/accepted`。

下表逐图记录位于本任务 `penpot-browser-01` 的原图；修正图另存，失败证据保留。关闭重开只证明实际设计持久化，不代替官方 ZUI 导出、产品加载或 LayoutReviewCase 接受。

| 实际原图 | 复核结果与接续 |
| --- | --- |
| `first-open.png` | 标题在节点底部、引脚顺序倒置，实例均显示 Roughness；提案视觉失败，已修正 |
| `closed-context-reopen.png` | 与初版图相同的布局/文字错误被持久化；不能因重开成功而接受 |
| `corrected-first-open.png` | revn=3 仍倒置，证明仅 revision 增加不代表 typed 子节点顺序生效；失败保留 |
| `corrected-closed-context-reopen.png` | 同一错误重现，转为 typed assign 修复；失败保留 |
| `typed-shape-audit.png` | 标题和引脚顺序正确；实例文字覆盖仍错误，继续原生 UI 核对 |
| `text-contract-audit.png` | SVG 和 API 内容差异的实际证据；不可当作文字覆盖完成 |
| `native-overrides-saved.png` | 原生 UI 保存后文字正确；Multiply 两输入完整，参数连线起点仍偏 2px，后续修正 |
| `native-overrides-reopened.png` | 新 context 保留文字/实例，起点偏移仍存在；不是 1px 双端验收 |
| `final-proposal-open.png` | revn=12，内部 pin/path 对齐和文字检查通过；UE 精细样式和产品比较未完成 |
| `final-proposal-reopened.png` | 新 context 同样保留全部身份/文字/连线；仅图表面结构提案，窗口验收 open |

上述 revn12 的样式差异继续以 UE 源码为依据接续。`MaterialGraphNode::GetNodeTitle` 反序组成参数的多行 caption，Scalar 为名称和 `Param (%.4g)`，Vector 为名称和 `Param (%.3g,%.3g,%.3g,%.3g)`；还必须追踪到 `SGraphNode.cpp::SNodeTitle::GetHeadTitle`：可改名的参数通过 `MaterialExpressionParameter::CanRenameNode` 使用 EditableTitle，所以实际首行是无引号的参数名称，不能直接照抄带引号的 FullTitle caption。root 标题来自真实原资产名，当前 Material 文案仍是结构标签。`GraphEditorSettings.cpp` 定义标量/向量/结果标题色、引脚上下各 4、左右 10 的 padding、线命中 2/6 tolerance、默认 1.5 数据线宽及材质 alpha/thickness 规则；`StarshipStyle.cpp` 定义 connected/disconnected 11×11 brush、节点 ColorSpill/九宫格/阴影。当前本地参考内容未找到这些 brush 贴图，灰色标题和圆形近似不能宣称最终样式对齐。`SGraphNodeMaterialBase.cpp` 的 106×106 真实表达式预览和 5px padding 也尚无引擎实现，未用贴图填充。材质侧不把这些度量复制为第二套全局主题。

本轮直接修改原材质 Penpot 文件，未启动或重启服务、未占桌面。revn13 保留全部 105 个旧对象 ID，新增 3 条可编辑网格 path。网格消费 `EditorStyle/Private/Settings/EditorStyleClasses.cpp` 中实际默认设置：snap=16、Regular/Rule/Center linear RGB 分别为 .024/.010/.005，转换一次 sRGB 为 `#2B2B2B/#191919/#101010`；`SNodePanel::PaintBackgroundAsLines` 每 8 格 rule，zoom 下按最小 8px 条件递增 inflation。这里是待公共主题链接的专责设计值，不创建平行 token 或发布新全局样式。

原生 UI 编辑后，参数组件/实例改为双行名称与 Param 值，隐藏的旧结构标签仍保留原 ID。header/body/pin row 继续使用 native flex；图节点和连线才使用文档坐标。引脚边界改为 11×11，未连接 Normal/Emissive 用空心近似标识。实际画面发现网格盖在节点上、移动引脚后连线仍接旧位置，已于 revn15 修正：网格进入非 flow 绘制顺序底层，五条 forward 连线按 `UGraphEditorSettings::ComputeSplineTangent` 计算 Hermite tangent，再以 tangent/3 转为 cubic Bezier 控制点，不沿用固定 100px。`penpot-graph-refinement-02.json` 和 `penpot-browser-02/receipt-05.json` 保留两次全新 context 的打开/关闭/重开及 SVG 几何，最大端点误差约 0.000045 逻辑像素；这仅是 Penpot 提案内部检查。

新增两个图视口设计板，复用相同 3 个 native 组件，没有新造按钮、字段或 Dock。revn16 为 238 objects、15 instances、97 flow frames；实际浏览器文字测量保存到 revn17。所有新实例保留 componentFile/componentId/shapeRef；节点、文字、padding、pin 与 wire 在同一 zoom 下缩放，DPI 未模拟。`penpot-graph-fit-boards-03.json` 保存每个旧→新对象映射及 camera，`penpot-browser-02/receipt-06.json` 保存四次独立上下文的读回与关闭。900 板的首开/重开跨 revn16→17，640 板两次均为 revn17；所有端点内部误差 <0.000062 逻辑像素，不能据此声称产品误差或整窗通过。

| 图视口 board | 实际 UUID | 设计 camera / grid | 验证范围 |
| --- | --- | --- | --- |
| 原 1280×800 提案 | `2fa31016-62fd-45d2-82d7-83457be89e6c` | zoom=1，16px 格；保留原 header | 图 fixture，非完整 Material Editor |
| 900×620 fit fixture | `ce4a6bdc-5f45-4151-b8ee-e07228bf4328` | zoom=.8，pan=(-19.2,-96)，12.8px 格 | 图视口，未覆盖菜单/Dock/预览/Details |
| 640×520 fit fixture | `b89331c9-90de-4ee7-bd5f-ffc51cd845ad` | zoom=.5，pan=(0,-48)，inflation=2、16px 格 | 图视口，未覆盖整窗尺寸或真实 DPI |

继续追踪到 Slate 后发现参数首行不应保留引号，已在原生文本编辑器修正主件及九个参数实例，共十项，实际十次保存均 HTTP 200，revn17→27；见 `penpot-browser-02/receipt-07.json`。副标题仍有具体开放差异：`StarshipStyle.cpp` 的 `Graph.Node.NodeTitleExtraLines` 是 9pt Italic、特定半透明颜色，`SNodeTitle::RebuildWidget` 还按 snap 为额外行补高；本设计目前两行均为同一 Bold 样式，未实现该层次与 snap padding。真实 `get-font-variants` 仅返回 Roboto normal 400/500/700 与 Noto Sans SC normal 400，缺少 Roboto Italic。该字体及公共图标题样式交 Parent 统一提供身份和映射，不能用合成斜体、literal 拷贝或现有组件数量宣称共享样式已完成。

改名后的重开暴露两项必须保留的失败，不能沿用此前几何结果：revn27 的窄视口参数 pin 偏移 2.2628/5.6569 逻辑像素，因为实例只声明 geometry override，native 同步把缩放的 body padding 6.4/4 重置为主件 8。按 `component.cljc::sync-attrs` 声明实际 `layout-padding/layout-gap/stroke-group/radius-group` 并重新应用同一文档 camera 后，revn30 的 pin 检查通过，但逐图仍发现 Roughness 字号异常放大。真实 leaf fontSize 已变为未缩放的 13.333333，部分缓存的 positionData 仍为缩放值；仅检查 API 文字或 pin 几何会漏报此差异。

进一步读取实际服务 HTML 和运行中的 shared.js，确认 deployed Penpot 为 **2.17.0**，其文字同步区分 `text-content-text` 和 `text-content-attribute`，不能仅依赖本地 `dev/penpot` 的旧 `content-group` 参考。该真实 bundle SHA-256 为 `65738e9825f2904fb329470fd545e00e7c7a9a2c97571caaf2f01d6961258b90`，只在内存读取，未保存源码副本。恢复一致的 rich content/glyph data 并声明实际 attribute override 后，采用键盘仅选择首行名称、改名再恢复，不使用整块替换：900 和 640 板分别仍保存 fontSize=10.6666664、6.6666665；native 保存至 revn33。精确 group、失败图片与修复记录在 `penpot-graph-fit-boards-03.json`，键盘探针与真实字号回读在 `penpot-browser-02/receipt-11.json`。

最后三个全新 context 均读回 **revn33/238 objects/3 components/15 instances/97 flow frames**，保持旧身份和三块 board；`receipt-12.json` 同时断言 pin/path 误差 ≤1、持久化 rich content 与实际 SVG 字号一致，三个最终图逐一打开复核。九个参数 caption 的实际字号按同一 camera 缩放，十五条线端点最大内部误差约 0.000062 逻辑像素。该验证只覆盖 Penpot fixture，不是 UE/产品双端误差。实例的字体 attribute override 会冻结更宽的 rich text 属性，公共主题传播/局部 reset 尚未验证，必须交 Parent 的组件/导出合同接续，不能当作全局样式联动通过。

| `penpot-browser-02` 最终原图 | 逐图复核与剩余差异 |
| --- | --- |
| `final-verified-1280x800-reopened.png` | revn33；名称/Param 两行可见，5 线对齐，Normal/Emissive 空心；灰色标题、统一 Bold、副标题 padding、真实 preview 与公共样式仍缺 |
| `final-verified-900x620-reopened.png` | 同样的 3 主件实例，zoom=.8，字号缩放与 pin 对齐；这是 900×620 图视口，尚非完整窗口 |
| `final-verified-640x520-reopened.png` | zoom=.5，grid inflation=2，字号已修复为同一缩放，pin 对齐；小字号符合该 camera fixture，整窗可访问性/焦点/操作/DPI 未验证 |

所有中间失败原图和回执继续保留，逐图台账为 `penpot-graph-review-02.json`；未看过的中间自动化失败图会明确记为未完成视觉复核，不填“通过”。本轮没有公共库发布、正式导出、EXE 指纹/原生图像、150% DPI、并排/差异图或样式往返生效证据，窗口族仍 `passed=false/accepted=false`。

共享源码的两类型命名缺口已按 Parent 后续逐文件授权完成最小源码修改：`ui/graph/mod.rs` 从原 private model 公开重导出 `GraphDeltaProofV1` 与 `PreparedGraphDeltaV1`，不复制类型、不公开整个内部 module、不改定义/构造或 retention/revision/prepare/一次 permit。修改前该文件无已有 diff；实际 diff/hash 纳入新一轮九路径 guard 和受管验证记录。较早“只记录提案/未导出”仍是对应历史观察，不再当当前状态。材质适配器接续路径仍为本插件 `editor/src`；公共 native graph host、catalog 注册、实际 paint/input/sourceRef 继续待 Parent 明确交接。原 `WorkbenchMaterialGraphCanvas` 及三行 control/event 身份迁移须逐项对照原资产；没有正式导出与实际 host 证据，不删除/隐藏占位行或宣称产品已接通。

Parent 后续转交 Widget owner `01a16d52-d54d-7ac0-b0d5-21864f383a1e` 的 UI-SH-GRAPH-01/W11/W12/W20。已在[既有共享提案](shared-node-graph-contract.md#widget-消费适配ui-sh-graph-01w11w12w20)细化稳定 node/pin/edge identity、document/view/DPI 同一变换、跨 Designer/Graph/Source 选择焦点、同一域 history 的命令/撤销、异步诊断/source locator，以及真实 Bookmarks/Find 的适配与缺口。Widget 自己拥有事件、属性绑定、存储和执行；材质 compiler/MaterialAsset 不进入中立表面。当前 `UiAssetEditorMode` 的四个真实模式保持原义，Source 不改名为 Graph；本域 3 个 Material 节点主件也不改标签冒充 Widget 图消费者。该接续只改本域两份计划，未改 Widget 原资产/源码、公共 catalog/tokens/组件、Penpot、服务或桌面；运行接入与 W11/W12/W20 仍 pending。

Widget 最新接续已在[同一合同的canonical源/版本节](shared-node-graph-contract.md#widget-接续canonical-源版本和组件身份)补齐真实slot、事件、override/reset、既有本地binding ticket到Details/Source/焦点的事务顺序及公共逐文件提案。本轮只读解析原Widget v2资产，97 nodes/97唯一control IDs/6原events保持；30路径指纹和现有Shared canonical-record v1沿真实来源记录，未给中立Graph预填UUID/版本。Parent的Material rev48/Widget rev9观察保留为历史，当前复用本域既有rev55回执，Widget owner文档已到rev12但本轮未API重读。发现实际AI `route_edge` →共享Down/Up路由的跨crate源码消费者；10项域测试仍未执行，没有生产retained caller/frame/paint/hit或运行图像，故只作源码级证据，不升级为运行级consumer验收。`widget-shared-graph-contract-04.json` 与 `widget-shared-graph-public-proposal-09.json` 给Parent具体路径、当前SHA、Owner、依赖和聚焦验证；公共Rust/主题/catalog/插件及Widget/AI源均未改，原proof/retention/一次permit和两份原测试identity保持。

Parent 再转交角色 owner `01a16d52-6b1e-7543-82f0-4f05ca92ad58` 的 Control Rig/Retarget 消费需求。已接续[同一 MAT-SHARED-001 的角色适配节](shared-node-graph-contract.md#角色消费适配cs-gr01cs-d020304)，对照其设计 5.3/10 和 CS-GR01，列明稳定 typed node/pin/edge 与 bone/control/op locator、单一图变换、跨层级/Details/picking 选择焦点、既有 command/history、revision-bound 诊断/跳转，以及真实求值/pose/保存的域合同。Retarget OpStack 保持有序列表/树，Rig 图坐标不写入 pose/body_offset；数学 IK、可画表面与已存在的预览事件不能证明 RigVM/retarget 能力。实际 Rig/typed 文档、OpStack、求解、pose 和持久化仍由角色 owner 实现，MaterialAsset/材质 compiler 不进入跨域模型；本轮只改两份材质计划，未写角色资产/runtime 或公共实现，消费者集成与产品验证继续 pending。

Parent 后续转交粒子 owner `01a16d52-a4a5-7271-90a2-336091337737` 的 PFX-GRAPH01（域表 PFX-GRAPH-01）。已在[同一共享提案的粒子适配节](shared-node-graph-contract.md#粒子消费适配pfx-graph01)更新当前能力：五类 VfxGraphNode 确有真实 program lowering、prepare、executor/readback 源码，旧“全部 no-op”过期；但现有 emitter/curve source key 不等于 VFX node 身份，VfxGraphAsset 仍无 stable node/pin/edge，Editor descriptor 未接 source/handler。合同细化无连线 Overview、选择/DPI/坐标、域持久 schema/typed 事务、同次编译 index→stable key/field 映射、global 诊断与 configure-once 的受控 preview 替换；MaterialAsset/材质 compiler 不充当 VFX 模型，任意 Niagara script/wire 能力继续缺失。本轮只改本域两份计划，未写粒子 runtime/原 ZUI 或公共组件源，未运行 Rust/插件/EXE；消费者实现和整族验收保持 pending。

动画 owner `01a16d51-741b-73b3-a61a-2189cd38f81e` 后续交接 [共享 timeline/curve/transport 合同](animation-shared-controls.md)及 graph 消费需求，已接入[同一 MAT-SHARED-001 的动画适配节](shared-node-graph-contract.md#动画消费适配)。保留原 `animation_graph_body.zui` root/canvas/slot、同一动画作者文档/history，以及 GraphDeltaProofV1/PreparedGraphDeltaV1 的 revision/retention/一次 permit；公共层提供身份/几何/焦点和属性/诊断呈现，Clip/Blend/Additive/Mask/Output、状态 entry/transition/条件、编译与播放仍由动画域负责。核实并列明 Output/transition/condition 身份、node-only 接线不足、palette blend_space_1d/2d 与 Graph schema 不符，以及 Blend `a/b` 与 compiler 多值 `inputs` 不符等缺口；不画假节点或由材质修动画模型。曲线/transport 复用该域中立 adapter 和真实 backend，不强制动画 World/第二 clock。公共 UUID/样式/slot/正式导出由 Parent 集成；本轮只改本域两份文档及回执，未写动画 Rust/ZUI 或公共源，产品/消费者集成与测试保持 pending。

AI owner随后以接续12正式确认同一源码级定向路由消费；本域已核对12源/8依赖/4原ZUI及累计patch `b00543e0…`，与前述AI graph hash一致，不算新增公共实现或执行。共享合同的早期`tests::shared_graph_`候选已改为AI现有`authoring_tests::`过滤范围；28个静态定义（24 authoring含graph10，另gameplay4）均未运行。真实测量/transform generation、paint/hit/clip/单次DPI、焦点、预算与一次permit仍由host/域接入，Decorator不折叠附件；完整交接和未完成边界见[AI接续12](shared-node-graph-contract.md#ai-接续12域内定向路由消费确认)。本轮只改两份文档，不重投core/材质，不写AI/公共Rust或原ZUI。

AI-C06随后量得895/629/447可用宽与650 task union。Parent第五次接续已明确追加`canvas.rs`、必要`mod.rs`导出及原`tests/cases.rs`的最小写权；本域沿原`GraphCanvasState`写入O(1) `fit_graph_bounds`/`GraphBoundsFitResult`和七项回归定义，不改域模型、history或第二相机。无效/溢出几何保持状态，最小zoom仍装不下会明确返回未全显；真实retained bounds/同代际transform、输入、paint/hit/clip/focus和SourceLocator消费仍缺，UE离散fit/padding/动画策略也未等同。详见[AI-C06 bounds-fit合同](shared-node-graph-contract.md#ai-c06-接续同一-canvasstate-的-bounds-fit)。本轮只执行静态/格式检查，现26个core静态定义不能背书原19项`#353`，未新建编译身份、未Cargo/重投、未动原ZUI或桌面。

PFX无连接消费者的接续已并入[同一共享合同](shared-node-graph-contract.md#pfx-接续空端口空边与无连线交互)：真实nodes可见而inputs/outputs/edges为空是合法状态；`null link interaction`不改Vec schema，不生成pin/wire/命中目标。仅 `edges=[]` 不禁用具有真实合法端口的图，实际 required-input 诊断仍保留；`GraphCanvasState` 本身没有 source-bound 连线手势生命周期，该接线/取消责任仍由宿主与域适配实现。连接工具、菜单、键盘及过期事件共用域capability和真实禁用原因，取消不写delta/undo，空source与未就绪/失败分开。原只读观察为Material rev55/1923/5业务main非共享库、PFX rev30/388对象/11 Shared copies；原 JSON 同时已记录 Shared rev2/4基础main/41对象/34色/3文字样式。此段旧句“无颜色/字体样式”是误述，保留勘误来源，不把 consumer own styles0 当库0；公共Graph主件UUID仍未发布，master有效style refs在该rev2时点仍0；较晚rev5的1个颜色引用见公开引用边界，不能倒改旧JSON。Parent第五次接续另报告PFX后继rev38/388对象/18copies/106 sourceRefs，真实System/Emitter只读投影与640 caption已完成并经Parent复核，仍有空value与文字草稿；该后继不是本域新API/产品验收，旧rev30和当时640待交付观察按原时点保留。真实retained命中/焦点/undo及preview/GPU帧关联继续open。本次只改本域两份文档与E根证据，未改粒子源、共享Rust、原ZUI或Penpot，未新增测试/生成/冻结/桌面请求。guard39只覆盖其当时13路径，原JSON和SHA不改，后继文档另取静态检查；现有26个core静态定义及原#353/#352失败不重做或重投。

### 中文、长名称与未接线状态接续

Parent 本轮转来的材质 `components=0` 是过时库存。材质会话再次独立调用真实 `get-file`：接续前 revn33，238 objects、3 native components、15 instances；公共文件仍为 revn0、0 components。这里的 3 个主件是本域既有图节点提案，不表示已复用公共库或共享样式。Parent 已明确确认通过官方聊天历史收到上轮交接；本轮库存更正的 `codex queue` 投递仍因 100 条上限失败，保留失败，不清队列或绕过入口。

在同一 file/page 新增下列两个可编辑状态板，继续实例化既有 Parameter/Math/Output 主件；旧 238 个对象及全部组件 UUID 保留。最终独立读回 revn40、330 objects、3 components、21 instances、133 native flex frames。身份、旧→新对象映射、修正历史及读回摘要见 `penpot-graph-state-04.json`。

| 图状态 | board UUID | 设计及验证范围 |
| --- | --- | --- |
| 中文与长参数名 | `32579e18-8156-4efd-8c46-87b2354f882f` | 1280×800 图视口；英文长名称及中文参数；标题按内容扩宽，无强制自动换行/省略；引脚行用 fill 跟随节点宽度 |
| 未接线材质根节点 | `c4cbc598-5492-4095-845c-fe15609188f8` | 1280×800 图视口；只保留实际 Output 实例和 5 个空心引脚近似；没有编译成功/失败横幅或伪造预览 |

长名称处理依据 `SGraphNode.cpp::SNodeTitle::RebuildWidget/CreateTitleWidget` 及 `SGraphNode.h::GetNameOverflowPolicy`，不把任意单词折行冒充 UE 节点标题规则。中文使用服务真实存在的 Noto Sans SC normal400；英文仍为 Roboto normal700。UE Bold/Italic 和中文 fallback 的最终共享字体身份仍待 Parent，当前 fallback 不能证明文字行框已与 UE 相差 ≤1px。未接线状态也不代表完整窗口的“空数据”用例：UE 材质可使用未连接输出的默认值，Zircon 现有 compiler 必需 BaseColor 的语义差异继续 open。

首轮原图显示长标题撑宽后引脚行仍固定宽度，路径接旧位置；未接线前三个引脚只去掉填充、没有描边。revn37 修为 native fill 引脚行，按真实 pin 中心及 UE tangent 重算 5 条 cubic path，并复用已有 Normal 引脚的空心描边属性，保留所有实例和 path ID。原生改名探测随后暴露读回早于异步保存的问题：DOM 已有新文字，API 仍为旧版本，下一轮才读到前次修改。`receipt-02/03.json` 的失败保留；`receipt-04.json` 改为有期限地等待目标文字真正读回，记录原宽 356、扩展名宽 421、恢复原名宽 356，引脚右侧 inset 始终 13.5 逻辑像素，恢复文字保存到 revn40 后才关闭。

`receipt-05.json` 的中文板首开/全新 context 重开均为 revn40，5 线端点内部最大误差约 0.0000542 逻辑像素，三个标题均保留名称与 Param 两行，水平 glyph 不溢出。该回执整体仍是失败：其空心引脚检查误将 fill ellipse 当作 stroke paint。实际 DOM 见 `receipt-06.json`，描边由独立 defs/use 绘制；修正检查后，`receipt-07.json` 用两个全新 context 读回 revn40，确认 5 个 fill=none 且具有实际 stroke use 的引脚。没有通过改动产品图像掩盖检查错误。

本轮 `penpot-browser-03` 保留 7 个回执、11 张原图，逐图记录为 `penpot-graph-review-03.json`；重复字节只在记录 SHA-256 和实际已看原图后复用视觉复核。最终中文板和未接线板的首开/重开分别字节相同，最终图已实际打开检查。还逐张补看了上一阶段 8 张未复核中间图，追加到 `penpot-graph-review-02.json`，不删除原失败：640 caption 图的第三行越过标题区、缩放字号/引脚回归及旧引号仍明确记为失败或被后续版本替代。以上全是 Penpot 图设计证据，仍无官方导出、EXE、真实 DPI 或双端接受。

## Material Editor 首个完整区域组合板

本次沿既有 UE v17 / ReactBits 结构映射，在原 file/page 新增 board `10fce53f-a2f4-4c68-8bd1-595758475d42`，原有 330 个对象 ID 全部保留。组合板覆盖文档标题、菜单、工具栏、左侧 Preview/Details 栈、中心图/编译反馈和展开的 Palette；场景明确为未接线 PBR root 的设计 fixture，未编译、没有渲染结果。37 个新增 native flow frames 负责框架/内部组合；图坐标仍独立于外围 flow。该板尚未覆盖真实停靠、焦点、命令或产品交互。

公共 file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` 的新实际读回为 revn1 / `isShared=true`，4 个主件、colors=0、typographies=0，替代此前“公共文件为空”的当前判断。材质只链接/消费此库，没有写入公共文件、catalog、tokens、Starship 或组件源。以下 component/main 身份来自真实 canonical-record，并已逐一核对实际源资产 ID、组件定义/root 和 SHA-256；没有另造对应主件。

| 公共组件 | 原生 component UUID | 原生 main UUID | 新板实例 |
| --- | --- | --- | --- |
| WorkbenchButton | `6ddc248b-bd35-80c5-8008-d6c8c01096f3` | `6ddc248b-bd35-80c5-8008-d6c8bd9b057f` | Save、Apply、Find 共 3 个 |
| WorkbenchSearchInput | `6ddc248b-bd35-80c5-8008-d6c8c3ce892e` | `6ddc248b-bd35-80c5-8008-d6c8c0f9c196` | Details/Palette 共 2 个 |
| WorkbenchTreeRow | `6ddc248b-bd35-80c5-8008-d6c8c7a79944` | `6ddc248b-bd35-80c5-8008-d6c8c452bf0c` | 真实六种节点名称，共 6 个 |
| WorkbenchPropertyEditorRow | `6ddc248b-bd35-80c5-8008-d6c8cab1e7d0` | `6ddc248b-bd35-80c5-8008-d6c8c84d848f` | PBR 五输出 defaults，共 5 个 |

另一个实例复用原 Material Output 主件；新增合计 17 个实例，其中 16 个跨文件引用公共库。材质最终读回 revn43 / vern0、634 对象、3 个本域主件、38 个实例；新板全部 304 个对象和旧身份均在，所有新 frame 引用指向实际最近 frame。公共库仍只有发布和本消费身份/重开的证据；共享样式、传播/override reset、官方导出及实际产品复用未验收。

实际保存过程中的失败均保留：首次 add-obj 把 native path 的空 width/height 当尺寸，HTTP400 且原文件保持 rev40；修正为真实 selrect，并按实际 28-byte penpot/path-data 解码/平移路径后 HTTP200，随后返回值数组假设失败，通过真实 get-file 对账 rev41，未重放新增。原生首开生成文字位置保存到 rev42。逐图发现辅助背景被错误取消隐藏及 group 被误作 frame；仅修正新对象，原 native hidden 和最近 frame 关系恢复到 rev43。JSON set 的 UUID 字符串与错误的 keyword feature set 分别被拒绝，回滚保持 rev42；改用真实 Transit UUID 与 string feature set 后读回成功，不放宽服务校验。

隔离 headless browser 的首轮启动 code3、错误 shell 文件名和仅有加载图的过早 readiness 都记录为失败；没有占用桌面或重启服务。最终两个独立 context 分别实际打开、选中原 board、使用 Penpot 的 Shift+2 适配，关闭后重开均读回 rev43，17 个 component/file/shape-ref 一致，browser exit0。全图与 board diagnostic 共 10 张原图已逐张复核；最终画面为约 71% Penpot 缩放，不是实际 150% DPI、官方导出或 UE/EXE 的 1 DIP 误差证明。证据为本任务 E 根 `penpot-toolkit-composition-01.json`、`penpot-browser-04/receipt-06.json` 和 `image-review.json`。

具体公共提案：Palette 叶节点需要域图标和无 child 时的 expander、visibility/lock action 可见性合同。当前 TreeRow 原始 ZUI 未声明这些业务能力，而公共图形仍显示 cube/eye/lock；保留为可见差异，交 Parent 的原组件/导出 owner 统一投影，不在材质内复制 TreeRow。菜单/tab/Dock 原生组件及共享文字/颜色样式也继续依赖 Parent。原 `material_workspace.zui` 的稳定 control/event/import/绑定仍未改动；本板不是正式源导出，Save/Apply/Find、Palette 与属性值只完成可编辑设计，尚未连接命令、authoring history 或 renderer。

后续逐项核对还发现 `WorkbenchPropertyEditorRow.value` 是 required/single slot，仅接受原声明的 Checkbox、Dropdown、Field、NumberField、RangeSlider、Slider、Toggle；新板五个值当前均为无 canonical component/shape-ref 的 native text，只能作为设计文字，不能计作已导出的可编辑字段。14:14Z 的真实 API 已逐行保存 slot/child 身份，公共库仍 rev1/四主件，未发现这些输入组件。公共 owner 需交付 `WorkbenchField`、`WorkbenchNumberField`、`WorkbenchCheckbox`、`WorkbenchDropdown` 的既有原资产对应 main/版本、slot/props/绑定/事件和正式 codec，材质再按原 ID 替换设计值并验证输入、覆盖/重置、焦点和保存；不在本域造平行输入组件。该结构核对不是官方导出检查通过。实际向 Parent 的 rev43/guard20 汇报再次被官方 queue 的 100 条上限拒绝；失败原文与未交付状态在 `penpot-toolkit-composition-01.json.parentDelivery` 保留，不移除其它队列条目或宣称送达。

### Material 未连接输出节点与三尺寸接续

在已保存 rev53 上接续同一 UE v17 布局、现有本域 GraphNode Output 主件和正式 linked Shared 控件，没有新增基础控件主件、复制公共 main、detach 或改公共文件。三板保持右侧27逻辑像素关闭侧栏、20/80主分区、Preview/Details各半、Graph/Compiler Results为80/20；Material 输出主件为 `d7173cd6-1da7-4897-be89-08583d32f598`，引用原 main `926a240d-18dd-4741-9cdc-8ce08292e721`。Output 的五个实际 pin key 为 `base_color/metallic/roughness/normal/emissive`；本状态无连线、未选中、未编译，仅为设计夹具，没有伪造引擎诊断或预览帧。

| 逻辑尺寸 | 实际 board UUID | Output 实例 UUID |
| --- | --- | --- |
| 1280×800 | `9fce0704-72a3-4f09-84e2-00e0d9efab78` | `28ce6a21-2d41-462c-983b-c9b0b8efff1f` |
| 900×620 | `46039af0-ecf6-4d12-a29c-60c09d698950` | `6f8c2246-7259-4a54-9e02-4cae1a37260f` |
| 640×520 | `722ca546-223f-4c7f-9e52-a37348a0250e` | `f32a3756-2ddd-4efc-ac9c-521959673e30` |

真实 API 增加439个原生对象、每板27个 flex frame，保存为 rev54；首次浏览器完成本机字体排版后为 rev55。原1484个对象ID及除根 children 追加外的全部原对象哈希均保持，5个本域主件不变；现在102实例，其中69个引用 Shared rev1四个主件。新增15个实例均保留 component-file/component-id/shape-ref，公共引用错误0，文件仍正式 linked Shared；共享色和字体仍0。原始 `material_workspace.zui` 的 SHA-256 仍 `5856c88655aafcae3b8f8af2d4ed7786ed89e61551610324faa51ce767af0c12`，本阶段未导出或回写。

复用既有 browser04 profile，沿已成功的 Headless Shell151入口，两个全新浏览器/context分别完成三尺寸原生 Layers选板、原生100% zoom、保存关闭重开；DPR1，不作为系统150%DPI。每个尺寸保存打开与全新浏览器重开的完整原图、诊断crop各一组，12张最终图的6份唯一内容已实际打开审阅，另外6张按相同SHA复用；两次各尺寸 full/crop哈希均一致。首次未稳定捕获的12张图及overview保留、10张初始图未作为最终复核；标准Chrome启动code3与更换至原成功Shell入口的记录未删。最终关键15个文字对象具有原生保存的18行 position-data，包括窄预览提示1/2/3行；没有伪造行框。原生最终渲染不再暴露节点子SVG fill，未用空DOM框或文件坐标声称实测边界误差通过。

当前仍有明确差异：SearchInput原生宽240，而Details可用233.8/157.8/105.8，超出内容6.2/82.2/134.2逻辑像素；900的Overview、640的Parameters/Overview被裁剪，公共Dock溢出/焦点菜单和Search Stretch投影未接。右侧抽屉、Save/Apply/Find和节点pin均没有产品交互验证，renderer仍无真实材质输入；4213 manifest于本次再次实际ECONNREFUSED，正式导出待原pipeline owner。未占桌面、未改DPI/Shared/theme/公共host，不把fixture或原生设计截图当正常EXE。

身份、源映射、原对象哈希、flow/几何及差异为 `penpot-material-responsive-01.json`；两次稳定浏览器回执与逐图记录为 `penpot-browser-08/receipt-05.json`、`image-review.json`，均在本文末原E根任务报告目录。5个成功浏览器均退出0、context全部关闭，失败启动记录和自有helper修正记录保留；未执行删除，释放0 B。最新设计及core终态已通过官方入口入队 Parent，message `01a16ecb-e27b-7283-a64b-26a6547cf991`；原core03的容量拒绝保留，新消息受理不表示Parent已读或批准验收，具体为 `material-responsive-parent-handoff-04.json`。

## Material Function 原生业务主件与组合板

接续同一 file/page 新增 board `b3ebf6f5-63c4-45ca-8f6a-a0f2a15e2403`。沿用 Material Toolkit v17 外围组合，复用已有公共 Button/SearchInput/TreeRow 的 9 个跨文件实例；Details 保持未选择表达式的空状态，不将此前静态值文字复制成函数签名输入字段。另创建以下两个独立业务主件及其两个实例；没有改名/复制 Material Output 主件充当函数输出，也没有引入第二份共享图模型。

| 设计身份 | 原生 component UUID | 原生 main UUID | 后端与源映射 |
| --- | --- | --- | --- |
| MaterialFunctionInput proposal | `6b647966-33a3-4326-ac96-1b15c26ff5d4` | `ca50a72a-64fe-4550-9c60-6fffeea2b16d` | UE 默认 `Input In (Vector3)`，左侧 Preview 输入、右侧 Value 输出；Zircon Function schema/源资产未实现，`sourceAsset=null` |
| MaterialFunctionOutput proposal | `f0ae6a1b-c6aa-40a4-b925-6021df706cdb` | `887826cc-2f45-40e5-ab77-079633df253d` | UE 默认 `Output Result`，左侧接收 Value；没有伪造既有 Function 资产或 registry 身份 |

新增 246 个对象、27 个 native flow frames，保留前一阶段全部 634 个 ID。`In → Result` 是明确的设计 fixture；未连接 Preview pin 为空心、连线两端为实心，真实 API 读回的端点与对应 pin 中心差为 0，仅证明 Penpot 本身的几何一致。新增对象的最近 frame 引用全部核对正确。HTTP200 保存至 rev44，原生编辑器生成文字位置后保存至 rev45；两个独立 context 关闭重开均为 rev45/880 对象、5 本域主件/49 实例，11 个本板实例的 component/file/shape-ref 全部保留，浏览器退出0。新阶段四张原图均已实际打开复核，完整图与 board diagnostic 保存在本任务 `penpot-browser-05`，逐图台账 `image-review.json`；`penpot-function-design-01.json` 保留组件、pin/wire、源参考哈希及 readback。复用已有自有 browser04 profile/cache，没有创建源码副本或新主题。

仍有具体视觉和业务差异：UE `UMaterialGraphNode::GetNodeTitleColor` 为 Input 使用 `FunctionCallNodeTitleColor`、Output 使用 `ResultNodeTitleColor`，对应 GraphEditorSettings 线性颜色和实际 Starship brush 合成尚未落实，当前设计仍沿用暗色节点标题。Palette 仍继承公共 TreeRow 的 cube/eye/lock/expander。函数 typed 作者文档、稳定端口 ID 与签名迁移、创建/搜索/调用、依赖/递归诊断、选择 Details、命令/undo、Apply/Save 和 renderer 均未接通；设计中的线不代表编译或求值能力。Source owner/格式/注册需 Parent 与 runtime authoring owner 明确后接入，未写原 ZUI、公共实现或 catalog。约71%画布缩放与 DPR1 不证明150%实际 DPI，未产生官方导出、正常EXE、并排/差异图或≤1 DIP结论；MAT-004/MAT-007继续 open。

### Function 右侧抽屉关闭与三尺寸接续

进一步读取 `MaterialEditorModes.cpp` v17、Slate `TabManager.h`、`SDockingTabWell.cpp` 和 `STabSidebar.cpp` 后确认，原 Function 板把 Palette 固定为240宽第三栏，不能代表默认关闭的右侧 sidebar。v17 为 Palette/Named Reroutes 注册 `SidebarTab/Right/0.14f`；`FLayoutNode::MinSize` 默认0，v17没有设置240最小宽。原板及旧截图完整保留，不把它的常驻 Palette 或最小宽当作 UE 依据。

本轮只在材质专属文件新增以下可编辑状态板，保留原1063个对象 ID；复用原两个 Function 业务主件和已链接公共 Button/SearchInput。各板新增6个实例（3个 Button、1个 SearchInput、2个 Function 节点），合计421对象、69个 native flow frames、18实例，其中12个引用 Shared。没有新增主件、主题或公共源修改。

| 状态/逻辑尺寸 | 原生 board UUID | 预览/Details 左栏宽 | 图中心栏宽 | SearchInput 的源 Stretch 目标/当前值 |
| --- | --- | --- | --- | --- |
| 未选择表达式、未编译、sidebar关闭 / 1280×800 | `b154e2cc-f1d6-4291-99df-6c0fa61fc29d` | 249.8 | 999.2 | 233.8 / 240 |
| 同一设计 fixture / 900×620 | `3bdd85e0-f901-4c4c-864c-b4a99a14f594` | 173.8 | 695.2 | 157.8 / 240 |
| 同一设计 fixture / 640×520 | `7f2846d3-3895-4b07-82d2-13141f628f1a` | 121.8 | 487.2 | 105.8 / 240 |

新板右侧 rail 为27 DIP（Slate PanelTab 的25宽旋转按钮加右 sidebar 的2左 padding），两个 tab 的160高度来自 `MaxMinorTabSize(160,25)`；关闭右抽屉的文字为逆时针90度。原生 flow 组合在 rail 外保留4 DIP主分隔，左/中心为20/80，左侧 preview/details 为相同默认系数，中心 graph/反馈为80/20；这些是来源明确的设计组合，公共 Dock 的实际 splitter、隐藏 tab well、背景/边线 brush 和 overflow 尚未验证。底部仍投影 Zircon 的 Compiler Results，UE默认打开的 Substrate 不因改名而成为已实现能力。公共 Dock/renderer owner 仍须接入实际菜单、tab状态和资源。

三板使用同一紧凑的 `In → Result` 设计 fixture：节点保持220×106且字体不缩放，世界布局宽464、节点间隔24，居中于各自画布。它与旧展开 Palette 板的 fixture 位置不同，`sourceAsset=null`，没有伪造 Function 文档或声称真实编译。按本板 pin 中心重新计算 native cubic path；原生编辑器生成排版后 API 最大端点误差约0.000197 DIP，只是本设计内部几何检查，不是 UE/EXE 双端≤1 DIP验收。

真实 API 保存为 rev49；原生编辑器测量后为 rev50。首轮两次 context 的三尺寸捕获发现 Palette 有文字内容却没有 `position-data` 和可见字形，Named Reroutes 正常，12张原图及失败记录保留。随后沿真实 Layers 的 Alt展开子树、选择文字、Enter编辑、Escape提交，逐板在 API 确认 Palette 的原生排版保存，依次到 rev51/52/53；没有写入伪造的测量框。全新 context 再打开三板均为 rev53，6个侧栏文字有实际测量、18个实例的 component/file/shape-ref 保留。浏览器和4个累计 context 全部关闭，两个 browser 进程分别退出0。现象已在本文件修正，未据此声称 Penpot 全局根因已修复。

最终15:25Z读回 rev53 / vern0、1484对象、5本域主件/87实例，57个引用 Shared（Button31、Search11、Tree10、Property5），Shared仍为 rev1/4主件、共享色/字体均0。`penpot-function-responsive-02.json` 保留 board/组件/源映射、几何、源码参考哈希、原始API参数失败及更正后的实际读回；`penpot-browser-07/receipt-01.json`、`receipt-02.json` 保留保存关闭重开、原图和界面测量，逐图台账 `image-review.json` 覆盖24张图（18份唯一内容已实际打开，6份相同 SHA 复用审阅）。复用自有 browser04 的同一配置 profile/cache，没有新建源码副本。

仍待共享 owner 接续的具体差异如下；未用任意最小宽或重新签署 canonical geometry baseline 隐藏裁切。

| 差异 | 实际情况、原因和消费合同 | 状态/证据 |
| --- | --- | --- |
| Function 900/640 搜索字段 | root仍为240，目标157.8/105.8；相对内容区超出82.2/134.2，相对左pane超出74.2/126.2。父frame实际裁切，当前可见文字不等于输入合同可用；公共原源声明Stretch，需 pipeline owner 给正式几何/测量/codec支持 | Shared SearchInput原 UUID 保留；三板表及 browser07。open |
| 窄栏 Details 标签与 viewport控制 | 900 Overview部分裁切，640 Parameters和Lit被裁切；需要公共Dock overflow菜单/真实选择焦点和viewport toolbar溢出合同，不能删标题或缩字体冒称解决 | UE Slate Dock/tab/toolbar；没有正常EXE或键盘/焦点证据。open |
| sidebar展开/收起/固定及焦点返回 | 新rail/旋转文字仅为本域原生布局，尚无已发布canonical Dock主件或真实drawer。UE storedPct0.14按overlay可用宽计算并clamp到0.5，含shadow/min/resize及focus-lost/closed回调，不是固定240宽第三栏 | `STabSidebar.cpp:794`及本轮参考hash；Parent公共Dock集成依赖。open |
| 节点、编译和预览 | Function标题的UE颜色/brush、typed函数文档、端口迁移、创建搜索/调用、Details、撤销、Apply/Save、依赖重编译和renderer均未接通。未编译文字与连线只作为设计fixture | MAT-004/007；无效果按钮与占位预览不计完成。open |
| 官方导出及验收 | 本轮未运行公共插件导出、托管Rust或正常EXE，未产生与产品等效状态的并排/差异图。Penpot约71%/98%/134%画布缩放和DPR1不证明实际150%DPI | 历史13项准备冲突与core19项未运行分开保留；MAT-007/008。open |

## Material Instance 空态与 Hierarchy 原生设计

同一真实 file/page 新增以下四块原生 board；没有选择或生成实例资产，`sourceAsset=null`，Parent None 为只读设计文字，参数声明数为零。布局依据 UE `Standalone_MaterialInstanceEditor_Layout_v8`，减去 4 DIP 分隔后采用 preview/details 的 0.70/0.30 系数。普通实例不展示 Apply（UE `OnApplyVisible` 仅 FunctionPreview 为真），工具栏使用实际命令标题 Show Inactive、Platform Stats 和 Hierarchy；Layer Parameters 只作为 UE 待定 tab，不添加不存在的 layer 数据。

| 原生 board UUID | 逻辑尺寸与状态 | preview/details 宽度 |
| --- | --- | --- |
| `eb526e4e-6399-4857-9f4c-37a7ac29f26d` | 1280×800，无父项空态 | 893.2 / 382.8 |
| `9c6906df-acb7-4646-8d9d-280fe7025700` | 900×620，无父项空态 | 627.2 / 268.8 |
| `0206676f-5e96-42f2-ad25-31508a2f0a84` | 640×520，无父项空态 | 445.2 / 190.8 |
| `c7ac5e73-c553-4e70-b343-f357780e8a27` | 1280×800，Hierarchy 展开且无父项 | 893.2 / 382.8 |

新增 183 对象、57 个 native flow frames；20 个跨文件实例全部复用现有公共 Button/SearchInput（各板 4 Button、1 SearchInput），没有新建公共主件或改公共源。保留之前全部 880 个对象 ID，新增对象最近 frame 引用全部正确。HTTP200 保存至 rev46，原生编辑器生成文字测量后保存至 rev47；修正本板弹层的实际 sibling paint order 后读回 rev48 / vern0、1063 对象、5 本域主件、69 实例。公共库实际仍为 rev1 / 4 主件 / colors=0 / typographies=0；全页 45 个公共实例分别为 Button22、Search8、Tree10、Property5，链接身份不是正式导出/共享样式验收。

保留首轮失败图并实际修正：Hierarchy 弹层 `f7cafe56-a78b-4e18-ab32-dcf3cb4aabc3` 最初被后绘制的 body 遮挡；仅重排所属 board 子项，保持 UUID/几何不变，最终图已看到 Material Parents / No Parent Found。锚点为 Button 实例 `7861e2da-a4a4-4780-b22d-810dca123d11`，关闭后应返回该按钮的焦点；这只是两种可编辑设计状态，没有安装 prototype interaction，真实打开/Esc/选择/焦点返回仍未实现。

640 首轮诊断截图还暴露了测量错误：SVG group 的 `getBoundingClientRect()` 合并了被裁剪的组件后代，错误把额外区域计入 board 宽度，并截入 Penpot 侧栏。独立只读 probe 核对真实 `getScreenCTM`、frame 背景和 clip rect 后，改由 SVG 变换与声明的 640×520 frame 计算诊断边界；保留错误原图/数值，并在最终 8 状态捕获中使用新方法。两次独立 context 均读回 rev48，20 个实例引用一致，浏览器/contexts 全部关闭并退出0。32 张原始图的 19 份唯一内容已实际打开，另 13 张按完全相同 SHA-256 复用已审阅内容；逐图记录包括失败和现存差异，没有把失败图重新标成通过。1280、900、640 的实际 Penpot 缩放分别约71%、98%、134%，浏览器 DPR1；这些不证明 Windows 实际150% DPI 或产品画面。

仍需 Parent/原组件与导出 owner 接续的精确差异：640 右栏宽190.8，SearchInput 实例 `dc07cd1c-7976-40fc-89a7-c96ba3b36476` 的 canonical native root 仍宽240，右侧超出 board **57.2000095 DIP**；其原始 `zircon_editor/assets/ui/editor/workbench/primitives/inputs/search_input.zui` 声明 width=Stretch，无240最小宽。扣除两侧8 DIP padding 后，真实目标应为 root174.8、内部文字区138.8，保持现有 control/font/caption/稳定引用与输入事件。需正式 source/font/caption/anchor/几何合同支持原组件的 Stretch 投影和官方 codec；本域没有改公共主件、源资产或重签 geometry baseline 来绕过守卫，也没有捏造 Dock 最小宽。640 tab 标签框同样越过右栏，但实际文字仍可读；真实 Dock overflow/menu/键盘焦点尚未验证。

证据为本任务 E 根 `penpot-instance-empty-design-01.json`、`penpot-browser-06/receipt-01.json`、`receipt-02.json`、`geometry-investigation-01.json` 和 `image-review.json`。仍缺实际 asset/parent picker 与跳转、组和 typed 输入、覆盖/关闭覆盖/保留开关的重置、父项变化、dirty/undo、实时材质 renderer、Save、正式源导出、样式修改往返、正常 EXE、真实 DPI 和双端比较。原 Material rev43 / Function rev45 图保留原 revision，不冒称已在 rev48 重拍。MAT-003/MAT-007/MAT-008 均保持 open。

## 域内后端修复

2026-10-25 后继求值修订直接接入既有 `compile_material_graph`：原颜色和标量求值都会递归展开两个输入，只用活动路径集合检查环，不缓存共享前驱。每层两个输入都引用上一层的深度 d 图会发生 `2^(d+1)-1` 次求值调用；长链还有调用栈增长。这是源码控制流分析，未报告为已经运行的 Rust 复现。

`editor/src/evaluation.rs` 现用显式后序栈和本次 compile 的两类值缓存；Base Color/Emissive 共享颜色结果，Metallic/Roughness 共享标量结果。只访问实际输出依赖，保留既有标量颜色展开、参数覆盖、纹理引用、不支持类型和非有限值诊断；断开的有限参数算术溢出仍在连接输出后才求值。临时索引/缓存/栈由图大小界定，没有全图作者快照或跨次全局缓存。`lib.rs` 和 `outputs.rs` 的实际调用已替换，公开编译 API、资产 schema、作者坐标/连线、共享 GraphModel/DeltaCommand/permit 与运行时资产层均未改。

新增 `tests/evaluation.rs` 四项候选回归覆盖64层共享输入同时驱动四输出、4096层颜色表达式、参数修改后新编译失效和作者文档不变、断开溢出连接后才产生颜色/标量错误。它们仍在原 `material_graph_` 过滤范围；全部仅为源码定义，未编译、发现或执行。旧30项及所有旧 Jenkins 原输入/终态保留，新源须经当前来源、writer、资源、pool/cache 和合法后继身份准备后受管验证，不重放 r10。

初次四条 Rust 路径、计划更新、before/after依赖与静态结果保留在本域 E 根 `material-compile-evaluation-01.*` 和 `guard-53.json`。后继复查发现既有 ignored 性能测试的 optimized 和 legacy 分支仍引用两个已迁移的私有函数，初次静态守卫未覆盖 Rust 名称解析；即使默认不执行 ignored 测试，lib tests 仍须编译该函数体。已将 optimized 分支接入同一 MaterialGraphEvaluator，并把 legacy 的乘法保留在原测试对照内，不加入生产兼容 shim；原采样、阈值、ignore 与测试总数均不变。新增 `tests/cases.rs` 为第五条 Rust 路径，本轮 `material-compile-evaluation-02.*` / `guard-54.json` 记录修正及当前六路径检查；旧01报告不回写成完整迁移或类型检查通过。该修订改善既有 CPU 编译路径，不关闭下述命令/原生宿主、作者文档、真实 renderer、官方导出或产品画面缺口。

现有 `material_workspace_state.rs` 的 Compile 只写 `Not implemented (preview)`，未调用插件图编译器；插件 native dist 的 `invoke_command` 也为 `None`。这两个事实证明目前没有完整的“UI → 材质编译 → 产品 renderer”链路。

本次只在 `zircon_plugins/material_editor/editor` 补齐既有 compiler 合同：

- Output 除必需的 Base Color 外，处理可选 Metallic/Roughness 标量表达式、Normal 的直接 texture sample、Emissive 常量/texture；沿用既有 `MaterialAsset` 和持久化合同，默认输出不变。
- Palette 和 validator 共用同一输出引脚定义；不存在的源/目标引脚与错误方向返回诊断，避免忽略连线后仍报告成功。
- 校验参数类型/有限值与整个图的环；算术溢出返回错误。Normal 向量表达式、纹理算术和标量输出的 vector/texture 输入仍明确报不支持，不伪造结果。
- 新增 `output_contract` 回归，覆盖 renderer descriptor、`.zmaterial` 序列化/重读、可选默认、纹理 Emissive、非法引脚/方向、断开图分量的环、错误参数类型、非有限数值及溢出。这些测试尚未由 Jenkins 编译运行，不能记为通过。

本轮没有增加函数资产、共享图组件或 UI 假动作，也没有把编译器可生成 `MaterialAsset` 写成已验证 GPU 渲染。

真实渲染消费入口已追到 `zircon_runtime/src/graphics/scene/resources/resource_streamer/ensure_material.rs`：加载带 revision 的 material snapshot、`resolve_effective_material_asset` 解析 parent、按 shader 合同生成 descriptor、检查 shader/texture 依赖和 readiness。后续 Apply 必须更新这条资产/版本链并观察实际帧；本轮只覆盖了 descriptor 和持久化的待运行回归，没有绕开 renderer readiness，也没有该链路已执行的证据。

继续核对项目实际写入入口 `zircon_runtime/src/asset/pipeline/manager/assets/generated_source.rs::ProjectAssetManager::publish_generated_project_source`：它要求 `res://` URI，准备 source/sidecar/artifacts 与 live resource mutation，在 generation/activation/preparation epoch 校验后提交，发布变化并 `ensure_durable`，返回 source URI、generation、前后源哈希及 status。该入口确实会持久化项目源；不能拿来实现无需 Save 的预览或 Apply。本轮在材质 editor/plugin 搜索未找到该方法的调用，未调用发布方法或改任何用户材质文件。最小接入合同如下，仅交 Parent/资源 owner 评审，不新增平行资产存储。

| 操作 | authority、结果与待接合同 |
| --- | --- |
| Compile | 使用现有 `compile_material_graph_operation` 生成候选与诊断；保留文档 revision 和 node/pin/source 路径，不把 CPU 成功升级为渲染完成 |
| Preview / Apply | 需要 owner 提供可消费候选材质的真实 renderer/资源版本入口，保持未保存的 dirty 状态与既有撤销 authority；现有 durable source 发布不是这项能力的证明 |
| Save | 从原 authoring 文档保留稳定 ID、parent/local override presence 和 editor 元数据后走正式项目事务；核对提交 source hash/generation/status，失败保留 dirty 与原有效渲染版本 |
| 帧验收 | 通过 `load_material_asset_snapshot` 和实际 renderer readiness 观察已消费的 material/shader/texture revision，再取得正常 EXE 原生帧；文件写入或 receipt generation 单独不代表 GPU 已使用新材质 |

## 验证矩阵与开放差异

三个主窗口均需覆盖四个实际配置：`1280×800 @1.0`、`900×620 @1.0`、`640×520 @1.0`、`1280×800 @1.5`。最后一项必须由新窗口实际 DPI 证明；每项状态包括默认、空数据、选中、键盘焦点、弹层/关闭后焦点返回、滚动首尾、中文/长文本。下列材质特有流程另行逐步捕获：节点搜索/创建/移动、接线/替换/断线/取消、类型错误/环、Undo/Redo、参数分组/继承/覆盖重置、预览网格与相机、编译成功/失败、Apply 前后 world 渲染变化、Save/关闭/重开、函数导航与编译依赖。

当前所有产品尺寸/状态图像与流程均为 `not_run`。每个实际 `LayoutReviewCase` 须关联 source/dependency/theme/EXE 哈希、Penpot file/page/board/component/instance、state、逻辑/物理尺寸、实际 DPI、ready 状态、native nonce/generation/PID+birth/source 证据及完整原图。有效捕获与资源就绪后才允许 `passed`；同内容双端并排/差异图、逐图复核和关键边界/文本行框误差 ≤1 逻辑像素后才允许 `accepted`。还须留一次真实 Penpot 样式修改 → 官方导出 → 原资产加载 → 产品截图变化的证据。

| 差异 ID | 参考与实际差异 | 原因/依赖/待决策 | 证据与状态 |
| --- | --- | --- | --- |
| MAT-001 | UE 图节点/连线 vs 三行演示表格 | 缺 MAT-SHARED-001 图表面及域命令消费；需 Parent 统一组件 owner | UE GraphSchema/TabFactory 与原始 ZUI；两端图像缺失；open |
| MAT-002 | UE 实时材质预览 vs 无材质 renderer 绑定 | 工作区仅演示状态，公共 preview primitive 是静态 SVG；需 renderer/host 合同 | `material_workspace_state.rs`、`preview_viewport.zui`；原生图缺失；open |
| MAT-003 | UE 参数分组/覆盖/重置 vs 仅实例空态设计、无产品实例窗口 | parent/override 与 shader 元数据已有；图实例参数 ABI、presence 保留、editor 表保留及 authoring/撤销待接入 | MAT-INSTANCE-01～03、`.zmaterial`/`inherit_parent_values_from`、UE detail customization/helpers；rev48 四空态/弹层板无真实参数，open |
| MAT-004 | UE Function 输入输出/调用 vs 六种基础节点 | 须确定 Function 资产/签名/依赖编译合同，保持用户指定目标可见 | UE Module/GraphNode_FunctionCall；Zircon authoring enum；open |
| MAT-005 | Apply/Save 更新产品与磁盘 vs Compile 演示反馈 | compiler 与操作、文档、资源重载、renderer 尚未接通；已确认的 generated-source 发布会持久化，不能代替未保存候选的 Preview/Apply | UI `Not implemented (preview)`、native `invoke_command=None`、`publish_generated_project_source`；open |
| MAT-006 | PBR 图连线曾被忽略 | 本轮已有源代码修复；待 Jenkins 测试及实际渲染验证 | 新 `outputs.rs`、`graph_contract.rs`、`output_contract.rs`；awaiting-validation |
| MAT-007 | 窗口族状态、共享样式引用和官方导出链仍缺 | 原5图状态板、Material/Function组合板、Instance空态/Hierarchy及Function sidebar关闭三板保留，新增Material未连接root三尺寸板。5本域主件/102实例，其中69实例引用公共库4主件。Shared rev2与rev5观察按原时点保留；22在原Apply copy原生绑定一处有效共享颜色引用，主件继承、Update/reset与全局传播未通过；23修正8页签条等分/内容裁切，缺源测量省略号、关闭/icon及真实Dock交互。Property输入slot、Palette叶行图标、Search Stretch、viewport溢出、样式/brush及完整交互仍缺 | 最新Material保存关闭重开rev140/1971对象，原1923 IDs保留；23的16图与错误高度纠正分列，22四图逐图复核。18—20实际流式重排、21原生文字覆盖与旧图按各自原版保留；无官方导出、产品加载/并排图；open |
| MAT-008 | 所有尺寸/DPI/交互逐图证据缺失 | 需排队桌面输入、有效正常 EXE 捕获和双端逐图审阅 | 无图像；不可测量 1px 误差；open |
| MAT-009 | 插件注册 UI/template 资源缺实体 | 官方 Penpot 导出需落原始资产并明确 plugin resource resolver，不新造占位模板 | 插件目录实际文件清单及 `plugin.rs` URI；open |

## 运行入口与接续

### Jenkins 恢复后原身份收件查询

本轮消费 Parent `jenkins-recovery-handoff-02.json`，实测文件SHA `9a0924d8572d44ff455a76540bcc641fe3c7e5ab8066bd1c9c487857975d3f72`。Parent 已核实原恢复 r2 的 root0/active0/childrenGone、双EOF/readersJoined/jobClosedAfterZero及197项不可变文件；本域不重新运行恢复，也不以服务恢复补产品、P21/Runtime11或窗口族验收。Parent 两项历史查询的 authorization pending 不是本会话结果，本域本次未观察到该结果，也不要求用户手工grant。

当前实测安装记录 SHA `1d9eca0714630a10729f170344b4d7b73f6d6abd13f1642921ee146344551805`，selected capsule `d2873ff353ea93d6b76f7d7caa12878761146e326070fd4c4526b73f3dc23112`；capsule自身及5份server/6份 `capsule/clients` 原生文件逐项匹配，查询前后安装绑定未变，同driver `fc553228…` / runtime `start-1792836412672596900`。首次只读辅助把client与canonical checkout entry比较所见差异保留为路径选错的非权威结果，已按实际 `clientRoot=clients` 校正；不据该差异否定安装。本次没有重审整套Home/API，不把Parent旧11/11作为当前Home结论。

原材质 r2 identity SHA `30a5b632e1ca0c319ef2617b0f6772cb2c774264d320f033a90a28d4181e292c` 及6源/11依赖、core identity SHA `f33013fc0d1eacda260b1581f72159d90388a8ed415e826c920ace97fb752de5` 及4源/9依赖均当前一致；原identity没有attempt字段，本轮不补造。材质沿原 `tools.dev.local_cargo` 的独立 `zircon_plugins/Cargo.toml` / 单包lib / defaults off / features=[] / test profile / `material_graph_` / jobs1 / threads1 / locked 精确argv，core沿原module wrapper与 `ui::graph::tests::` 过滤器，均只追加QueryOnly。材质最新日志 `material-managed-validation-r2-query-05.log`、core `shared-graph-managed-validation-query-04.log` 均 native exit0、observed/complete/resubmitted=false，分别读回原#352的preparation-conflict和#353的checkout_input_identity_conflict；13/19项仍0执行。core冲突仍是当时AI计划文档的原before/after，材质原expected/actual source digest保留，不据今日源码哈希匹配重写历史失败。

两次未查到原操作的本地失败单列：`material-managed-validation-r2-query-03.log` 的PowerShell裸 `--` 被转发吞掉，导致 `--test-threads` 误当Cargo选项；纯args probe已复现，原canonical Python按argv数组转发后解决。`query-04.log` 的 `Input differs from explicit output option: buildRoot` 源于当前pinned QueryOnly直接比较原文本，`E:/cargo-targets` 与原publicResult `E:\cargo-targets` 的同物理路径拼写不相同；`query-05` 使用原归档字面值后查询成功。没有修改recipe、身份、状态库或原输入，也不把本地转发失败计作新Jenkins作业失败。

完整时点、当前11项指纹、同身份原输出及两个前置失败在 `jenkins-recovery-consumption-04.json`。官方交接 `jenkins-recovery-owner-handoff-07.json` 已入队原owner `01a12af6-7511-70d1-a452-61b80678eb56`，message `01a16edd-1135-7de2-b44e-1b8cfd7a8164`；Parent副本容量100拒绝，双方未获已读确认。接续仍须原owner/Parent核对原writer、源码capture稳定条件及物理E根资源准入；本轮未新提交/重投Cargo，没有新request、attempt或本地fallback。Parent实例覆盖8项继续 `source-fixed/managed-tests-pending`，本域未跨查询或修改其原身份/五路径。没有桌面输入、新Penpot变更、服务操作或源码副本，释放0 B，实际产品验收保持open。

官方导出入口另外于17:04:03Z实际只读复查：Penpot `19001/readyz`、backend `16060/readyz` 均HTTP200，但 `4213/manifest.json` 为WinError10061连接拒绝；未启动/重启或恢复服务。完整输出在 `penpot-export-readiness-09.json`，已沿原owner交接至 `01a0f04d-9d54-7fe2-9adf-120186804cfb`，message `01a16edf-4132-7522-89e7-07d27a237052` 入队、未获已读，见 `penpot-export-owner-handoff-08.json`。该HTTP复查没有重新读取设计文件revision，不将原rev55读回改成新设计证据；正式codec/export、公共样式和typed slots、原ZUI产品加载及修改传播均仍未通过。

### 实例持久化原请求的失败诊断交付接续

接 Parent 转交的原 `ue-material-instance-presence-20261024-01` / `validate-material-instance-presence-01` 管理工具修复，本域只读核对 `operator/ue-material-instance-presence-20261024-01/result.json`、summary、完整gzip元数据和outer native四份传入SHA均匹配，17份声明证据也全部匹配。在14:55Z原诊断交付的核对时点，13份本地native与各自Home archive逐字节一致；原QueryOnly为root2，其余root0，双EOF/readers/waiter及Job active0证据保留。该历史Home匹配不代表后继归档仍存在，21:44Z状态见下方consumption05勘误。这是管理#407的86用例/267子检查、1历史fixture排除范围；不等于原runtime八项、材质插件13项或公共core19项已执行，也不代替运行host/产品生命周期验收。本域独立消费回执为E根 `material-instance-diagnostics-consumption-05.json`。

历史失败的准确边界已修正：原 `source_preparation.finish_full_file_index` 只抛通用ValueError，未保存历史完整before/after，无法恢复真实HEAD/index/membership差异、路径或责任人。本轮14:49:03～14:51:31Z的metadata-only观测前后HEAD、full index/debug、52168个成员及其digest都相同，新增/删除/index差异为0；147.910s只属于这一次元数据观测，不是历史失败恢复、全源内容稳定证明或下一轮完整capture窗口估计。新诊断只观察原有四条Git读取，生产新增Git命令为0，原错误时才将完整before/after和raw元数据写入先登记归属的 `capture-diagnostics/<identity>/<hash>/metadata.json.gz`；64MiB超限为incomplete、写盘失败为unavailable，原异常和比较保留。

交付时点14:55Z的immutable capsule为`2790fb…`，entry `c0ec4b74…`、source helper `2a38f559…`均实际核对。不能把该历史安装写成现在selector：本域17:32Z读到后续`66784e…`，最终有界11文件核对又为`e0635d…`，同driver/runtime，manifest+5server/6client全部匹配且该次installed前后字节稳定。后续manifest明确 `completeSourceDigestEnforcement=false`，注释指聚合expectedSourceDigest/actualSourceDigest的准备冲突执行策略；当前source helper与2790交付时仍逐字相同。此项后续策略变化单列交原owner核对，不归因诊断补丁，不从该flag推断 `finish_full_file_index` 的HEAD/index/membership比较已变，更不把旧2790的完整策略证据直接套到新版本。本域不改、回滚或采用此选项绕过当前任务准入。

原runtime identity仍为`b4984a1c…`，本轮实际比对五份源码与六项依赖11/11匹配。Parent/原owner在14:55Z的QueryOnly exit2/pending/`Current session authorization is pending`是其历史上下文，本域没有重查或创建intake，不要求手工grant，也不泛化为所有server/translated intake不存在。原插件r1的queue14241/operation key/`complete/conflict/preparation-conflict`原样，r2/#352与公共core/#353各自继续保留，均compileStarted=false。runtime命令recipe记dev、Cargo无显式profile参数，实际`cargo test`默认为test，不能写成执行了dev profile。

交付时captureReady=false；policyv11 reserve35GiB、E盘当时17.00GiB且无reservation，是14:55Z报告的资源条件，本域未将其改作当前磁盘实测或申请新reservation。后续仍由Parent/原source/intake/resource owner按原身份取得真实条件，不冻结全checkout或自动重投。Jenkins owner报告释放45,420,640B属于该owner，本域本轮释放0B；源/Git/cache/历史失败/锁/服务/Penpot无变更。MAT-INSTANCE-02仍 `source-fixed/managed-tests-pending`，全族passed/accepted仍false。

### 实例诊断 consumption05：归档可用性与工具指纹勘误

原owner21:44:02—41Z只读对账的JSON/摘要已核验为7be402d5…（126944B）/e71b8b1a…；四项原交付、本地压缩metadata及原b4984a1c输入在本次读回仍匹配，原payload和Parent session不改，无attempt补造。其17项固定证据、13项local native匹配及原终态字段沿原owner审阅保留，都是管理辅助进程证据，不能替代运行host或八项Runtime Rust。

该回执明确21:44Z时Home的 `zircon-management-tests/builds/407` 整个目录不存在，13份archive native及压缩metadata均缺；因此上文“Home逐字节一致”仅保留为原交付时的历史观察。本地22094272B的metadata仍是813cf971…，原#407成功和本地回执不删、不重建归档或重跑，不归责缺失的执行者。#407的86/267管理检查也不替后续胶囊资格、源码捕获或产品验证。

五份Runtime源码在本次限定读回仍匹配；原六依赖中五项匹配。唯一失配是canonical `tools/jenkins-management/entry.py`：原声明4e8d20f3…，21:44Z的owner读回8c87487b…/75667B，本域本次读回又为57153be4…/75955B。这是三个独立时点的控制工具修订，不是Runtime源码漂移；canonical与pinned client不混算，也不能把较晚工具塞回原输入。工具语义/正式执行路由的审阅由原管理owner在同一业务链完成，本次仅记录指纹，无工具修改或重新资格验证。

21:44Z的c2b07350、11/11 pins和driver197/receiver核对属于该时刻官方capability观察，不覆盖本域较晚r10入口记录。其declared hashes及两次capture之间的drift为advisory、使用第二次capture并保留诊断；单次 `finish_full_file_index` 内HEAD/index/member比较仍拒绝变化，source_preparation保留原2a38f559…。本轮未改gate或sealed文件，不把旧#407或e063观察当后继整套资格。

原完整identity/key6f39720b…的限定只读intake/route/alias/marker未取得对应项，结论仅限其查询范围，不推导历史从未收件，不补server receipt或要求手工grant。source/writer/resource/pool/cache/native worker仍未准入，captureReady=false、预计耗时null。21:44:41Z的E盘65466257408B（60.97GiB）是OS余量，不是本轮释放、配方预算或reservation；本域本轮0B，原owner历史45420640B不重复计入。147.91s仍只证明旧metadata-only观察，旧HEAD/index/member差异不可恢复或归责。

独立Runtime八项继续not_run，MAT-INSTANCE-02仍source-fixed/managed-tests-pending。plugin r1/r2/#352及core19/#353原终态、较晚plugin尝试分别保留。后续由原owner沿原业务链先审阅实际工具版本，再取得真实ordinary intake/delivery、完整closure/writer和Runtime资源/native回执；本次没有QueryOnly、validate、intake、freeze、Cargo、resolver或新的ACK。可追溯记录在E根 `material-instance-consumption05-20.json`；正常EXE/DPI及整族验收继续open。

### 官方19002导出入口与材质源闭包接续

17:26Z～17:27Z另一次真实只读检查确认 Penpot前后端readyz均200，现存 `http://127.0.0.1:19002/manifest.json` 为200/278B/SHA `dbcb22b4…`，`assets/plugin.js` 为200/485232B/SHA `442bed70…`。两项HTTP字节与现served-dist及worker-r3实际输出逐字一致；当前账号已有plugin `cce74285-44c0-8011-8008-d6ea3345bdbe`，没有再次Install/Allow、改registry或启动host。原4213连接拒绝保留历史，当前可用入口不再写成4213。旧预构建runner `26d2e4fc…` 的自建host/注册副作用及旧runner配新R3兼容未获证明，未运行该入口，不另造平行runner。真实回执为本域E根 `penpot-material-export-readiness-10.json`。

本次独立API读回Material仍为rev55/isShared=false/1923对象/5业务main/102copies/69份Shared实例，Shared仍rev1/isShared=true/41对象/4main/0色与字体。首次get-file请求未声明三个现有feature而被400拒绝，随后按实际错误补报 `design-tokens/v1`、`variants/v1`、`layout/grid` 只读成功；失败原输出保留，不是认证失败或新设计保存。核查七个Material/Instance业务板的完整原生属性，均没有可逆ZUI document metadata；这些可编辑设计不能直接冒充官方codec导入产物，也不能伪写source metadata使其通过导出。Instance三尺寸空态与1280 Hierarchy原板已存在且保存重开，不重复建板。

给原pipeline owner的最小source-owner白名单提案只包含原 `zircon_editor/assets/ui/editor/modules/core/rendering/material_workspace.zui`，SHA `5856c88655aafcae3b8f8af2d4ed7786ed89e61551610324faa51ce767af0c12`：29 nodes、27稳定control IDs、13 events、8直接widget imports，递归10份ZUI、16条import边且无缺失ZUI。闭包各路径/摘要、root及完整原ID/events均在回执。主题/catalog是只读公共上下文，不能据闭包阅读取得写权或复制缺失的公共主件；root原 `visibility=collapsed` 须由实际host review投影激活，不能改原源visibility或使用固定LevelEditor assert伪造产品。`res://fonts/editor-ui.font.toml` 的editor-assets路径候选不存在，但实际runtime同URI候选及13,353,468B TTC都已定位并记录摘要；实际挂载和浏览器字体就绪仍须验证，不误报整个引擎字体缺失。

原public owner `01a0f04d-9d54-7fe2-9adf-120186804cfb` 已收到该具体source导入/保存重开/正式导出与既有host/registry复用合同的官方入队消息 `01a16ef5-ac7a-7453-b762-6838d4ba0e71`，详见 `material-export-owner-handoff-11.json`；入队不等于已读或完成。Parent前一条100容量拒绝保留，本轮没有盲重投。原材质13/core19的准备冲突身份与终态不变；本次无Cargo、原ZUI/public源码修改、桌面输入、DPI或服务操作，释放0 B。样式修改→正式导出→原源加载→普通EXE真实渲染/实际150%DPI与逐图≤1逻辑像素验收仍open。

### 共享 core19 与公共库当前读回

接续 Parent 的 `ai-followup-03.json`，原 `mod.rs` / `29584afa…` 导出阶段保留为历史，当前四文件和 `2a0a4cda…` 的 proof/prepared/GraphAttachmentView、中立方向/箭头/原附件ID投影没有重做。实际消费者边界与本次来源冲突记录在[同一 MAT-SHARED-001 接续](shared-node-graph-contract.md#ai-followup-03-的实际共享身份与-core-终态)；不扩大 model/commands/canvas/retention、不改默认路由或 AI 整树/Replace/undo 语义。

16:04Z～16:16Z 真实 API 再次确认本域 rev53/1484对象/5业务主件，正式 linked Shared rev1/isShared=true/4主件，57个实例 head 的 componentId/sourceFile/shapeRef 全部匹配。四 canonical-record version1、main UUID、源路径/root/controlId 已读回；原保存关闭重开/24图证据按同 rev53 保留，本轮没有新 Penpot 修改或浏览器重开。Shared 色/字体仍0、4213 manifest 仍不可用；Property 五个 value_slot copy 身份正确而内容仍是 Text fixture，实际源 required/single/linear 只接收七种真实输入主件，这些尚未由公共库提供。实际输入/正式导出/产品链路仍 open，不复制公共 Field 或把路径/元数据匹配算效果通过。原生 metadata 命名空间初次读错的空字段探针也已保留、更正，不推断组件缺失。

独立 core 请求 `ue-shared-graph-20261024-01` / `graph-surface-core-01a16d50-20261024` 已经正式模块入口、真实 identity 提交，不是复用材质修复ID或重放原13项。精确唯一 `zircon_editor`、defaults off/features=[]、lib、`ui::graph::tests::`19项、jobs1/testThreads1、locked，Cargo 默认 test profile/模块 recipe 默认 dev；传递 target-editor-host/native-host/test-support，物理 E:/cargo-targets。实际 queue14515→Jenkins #353 在488.546秒后 FAILURE；同身份 query 为 complete/conflict/checkout_input_identity_conflict、compileStarted=false。冲突明确为 AI 计划文档，前后哈希见共享合同与回执；本域四源/九依赖全部未变，19项仍 not_run。归档/原 workspace 字节一致，SHA `26d25ae5…`；不借管理检查、query exit0 或 job API 终态补 Rust/原生验收。

回执分别为 `shared-graph-managed-validation-01.json`、`shared-graph-managed-validation-jenkins-353.receipt.json` 和 `shared-graph-consumer-reconcile-03.json`，位于本文末原 E 根任务报告。Parent 官方交接再次容量100拒绝，queued=false，具体消息和失败保存在 `shared-graph-parent-handoff-03.json`。后续依原 source-preparation owner/Parent 的可核验稳定窗口继续；原材质13项/r1/r2和本次core19各自保留、不替换或重放，未占桌面/启服务/建theme或clock。源码冻结持续到#353实际终止后才更新本计划；本轮无Rust/原ZUI/公共组件修改，释放0 B，产品验收未通过。

### 入口正式交付后的实际消费：r2

原实现 owner `01a12af6-7511-70d1-a452-61b80678eb56` 已正式交付 `ue-plugin-workspace-entry-20261024`。本域逐字核对交付 `result.json`、`final-current-binding-delta.json`、外层 `native-terminal.json` 的 SHA-256，保留 #400 的57项/118子项管理检查与1项明确排除失败；它们不计入材质 Rust。交付时 `cdd14500…`、安装记录 `1b3aa044…` 与提交 `2e811b2774a2c16bf20dcbf2be3e238a460f9e7f` 属原时点证据，主干/PR 集成仍 pending。本次 selector 实际选择后续 capsule `2790fb5199bd06ca7291ac197f03545eaeae05c56b125117fad268ce19d4b0c6`，当前安装记录 SHA-256 `350b91360090fb9ff551e4d0f86666a9080a9dd24c2a9872fb7ec2188b82f8a1`；同一 driver/runtime，11项 capsule/client 文件和公开 capability 均核实。没有回退部署、导入主干旧 parser 或更改入口源码。

交接示例直接运行 `tools/dev/local_cargo.py` 首先出现 `ModuleNotFoundError: tools`，在 selector/intake 前退出1，原日志保留。随后从 canonical repo cwd 使用同一文件的标准模块入口 `-m tools.dev.local_cargo`，未修改 `PYTHONPATH`、入口、公共源或服务。先对旧 operation 仅 query，确认原 `preparation-conflict` 终态后，在同 request `ue-material-graph-20261024-01` 下创建最新源码 operation `matgraph-tests-01a16d50-20261024-r2`。新 identity SHA-256 为 `30a5b632e1ca0c319ef2617b0f6772cb2c774264d320f033a90a28d4181e292c`；旧 identity、失败、归档和请求均保留，不重放。

本次执行前已向用户声明精确范围：唯一 package `zircon_plugin_material_editor_editor`；canonical `zircon_plugins/Cargo.toml`；defaults off、features=[]、locked、library target、test profile、`material_graph_`、jobs1/testThreads1，传递 `zircon_runtime/native-host` 与 editor 的 `target-editor-host`。`sourceView=checkout`、external root 为 canonical `E:/Git/zr_vm`，物理编译/缓存根只使用 `E:/cargo-targets`；无需也未回退根 workspace。六项源哈希、十一项依赖哈希合计17项绑定实际源码；其它三份获授 graph 文件和只读 model/canvas/commands 在依赖中明确绑定，未把 core19 冒充此次执行范围。

第一次客户端双轮捕获发现资源两份计划与粒子四份计划发生变化，退出2且未投递；精确六路径、前后哈希、输入摘要及未投递证明在 `material-managed-validation-r2.json`，不覆盖原失败。仅进行一次同 identity/同命令有界重试，成功捕获后实际入队14473，运行 [zircon-management #352](http://127.0.0.1:18080/job/zircon-management/352/)，67.344秒后 FAILURE。后续同 identity query 返回 `phase=complete/status=conflict/reasonCode=preparation-conflict`，`preparationStarted=false/compileStarted=false`。query exit0 只证明取回终态，13项 Rust 仍 not_run。

服务端准备前 expected digest `91e3cd28ddafa47bee11bdf47a20f76925fbe420bde51c0746dc5933e595936c`，actual `b78a8d99979ebf365c2bf3d159d56fe86e0303226054ce3075a6d6b7d187da70`。变化仅在 engine source 与聚合摘要；文件数52173→52174，而 Git HEAD/完整 index、dependency cache、external roots 全部一致。本域17项声明源码/依赖在终态后再次逐字一致。当前归档只含聚合摘要，没有第二次漂移的完整文件表，不能猜测新增文件或归因 owner。未选择实际 target pool，未编译，也未产生引擎包、符号或增量缓存；不能以本次管理作业失败代替 Rust 失败诊断，更不能计为通过。

实际归档 `material-managed-validation-r2-jenkins-352.receipt.json` SHA-256 为 `4a1b991b6979efe2e78b395fdf6fd47c000c9d821e6fe5dd3f7176c05959d440`；入口与源码/安装对账为 `plugin-entry-delivery-consumption-03.json`，完整 r2 过程为 `material-managed-validation-r2.json`，均位于本文末的同一 E 根任务报告目录。作业 API 的 `building=false` 与归档终态均已读回，但归档未提供该动作 root/children/EOF/readers/waiter 的原生终止证明；不借用 #400 helper 或服务 host 证明补齐。自有 wrapper/query 已退出，无本域客户端残留；host 生命周期及产品验收仍 open。

本域已尝试官方 `codex queue --thread 01a16d4e-0491-7353-84fa-038bd7706546 --message` 交接具体漂移与有界源码稳定窗口需求，仍因100条容量上限被拒绝，exit1/code-32600、`queued=false`；完整消息和拒绝保存在 `plugin-entry-consumption-parent-handoff-03.json`，没有 Parent 收件/已读证明。后续由 Parent/原 source-preparation owner 协调可核验的稳定/准入窗口及逐文件差异；本域不放宽 guard、不改旧 identity/历史记录、不继续替换请求、不创建源码副本或本地 Cargo fallback。入口已经交付的事实不再写成“等待入口修复”；下一步继续本域设计、源映射和具体差异复核，公共导出/样式/正常 EXE 接入仍依原 owner 协作。

### baac 同版历史回执补入

2026-10-24T15:56Z，本域只读复核原 owner 的 `baac-route-receipt-reconciliation.json`，实际 SHA-256 与交接 `9a6f91febb3cdddb724ea45bf417101edc3e2bb1708a1467dc92e12793e95d28` 一致。它接续原 `ue-plugin-workspace-entry-20261024` / Parent route-02：baac `#398 SUCCESS / queue14011 / RUN_ID plugin-manifest-20261024-r4` 通过 r4 的 `reused-qualification.json` 明确复用 r3 相同 SHA 的45项/97子项（含8项 manifest 守卫）；既有固定历史 fixture 在未改基线也失败，单列且未计通过，不能写成 r4 重新执行。

本域核对43个文件引用，42个历史/不可变引用逐字匹配；baac的5份实现、6份 client 与四份原生回执均匹配。四份 native 均为 root0、真实 Job active0、双 EOF、readers/native waiter 完成；regression 实际路径仍在 r3，其余 installer-check/apply/outer 在 r4。唯一不同引用是可变的 `installed.json`：该对账在12:52Z记录 cdd；本域15:56Z读取实际 selector 仍为后续 `2790fb…`、安装记录 SHA `350b9136…`，与上节 r2 一致。此差别按观察时点保留，不回退 cdd/baac，不改写旧安装记录，也不推定历史 sealed 文件损坏。cdd #400 的57项/118子项与四份独立 native 继续单列；baac 的 parser/recipe/environment 不能替代 cdd 后续 nested/cross-workspace 连续闭包与单包/混批保护。

原 `plugin-manifest-20261024-commit1` / session `wed5ba903a0` / queue13985 已提交 `bb07e79f64632f79c29136a193eb6b5ecfa70891`，按原 #543 回执，远端 PR #42 merged、merge commit `47da4b7f9817044937b4cad7190399dc531e5dd5`；#543 整体 UNSTABLE、可选 candidate validation skipped、本地同步 pending。#544 历史 fetch 超时仍保留。本次只读 HEAD 仍为 `75a76ee963792d5a316f738519e966937f770036`，不以远端合并证明本地同步、材质或完整 host 验收。

本域新增只读消费回执 `baac-history-consumption-01.json` 保留全部哈希比较、原生终态和时点解释，静态守卫另存 `guard-27.json`。本轮没有重跑管理检查、提交作业/Cargo、修改 Rust/原资产/sealed/Git 或操作服务；只接续本计划和任务报告。原 #352 `preparation-conflict` 及材质13项/core19项 not_run 保持原字节，不拿修复ID充当材质 validate 身份，不由这次历史回执触发新请求。实际释放0 B；必要失败和复用产物保留。

### 首次材质测试请求及较早入口历史

本次已通过正式 `tools/dev/local-cargo.ps1` 和本域真实 validation identity 提交原请求 `ue-material-graph-20261024-01` / operation `matgraph-tests-01a16d50-20261024`，Jenkins 队列项 `14241`。实际安装为 `cdd14500…`，driver `fc553228…`、runtime `start-1792836412672596900`；不是用入口修复 request 代替 Cargo identity。精确范围为独立 plugin manifest、唯一 `zircon_plugin_material_editor_editor`、defaults off/features=[]、lib/test profile、`material_graph_`、jobs1/testThreads1，传递 runtime `native-host` / editor `target-editor-host`，物理输出根 `E:/cargo-targets`。

首次调用因 PowerShell 吃掉未加引号的 `--` 而在派发前拒绝；内存 argv 探针证明原因后，同一 identity 以 `'--'` 保留分隔符，实际提交成功。首次 QueryOnly 又因入口把原保存的 `E:\cargo-targets` 与 `E:/cargo-targets` 做字符串等同比较而拒绝；改用已存的规范路径查询，同一 inputDigest `d7af72d8a70c613583d0b3fbd54bbc77e6e7cfd85c1508c42568a0a1014933c9` 返回 `phase=complete` / `status=conflict` / `reasonCode=preparation-conflict`，`preparationStarted=false`、`compileStarted=false`。wrapper exit0 只是成功取回终态，不是 Rust 通过。

原始完整输入摘要 `432e2e2b384c09d167e16fcfcb085bba2fd9e9d43da2c6b8b4b235bab59c7812` 与服务端 `2bedbb13ab3cf0ffdcb1333fc4635ba3ba83b75aac1f1c236a465ea12bee5245` 不同。逐项比较组件回执，变化仅定位到 engine source 的 filesDigest/sourceDigest；external `zr_vm`、Git HEAD/index 元数据及缓存配置摘要一致。本域六个 Rust 源、五个 manifest/lock 哈希在终态后全部未变；聚合回执未提供精确变化文件，不能归因某个 owner。请求、两次调用和两次查询的原始日志均保留在本任务 E 根报告的 `material-managed-validation-01.json` 与关联日志；identity SHA-256 `7a924e531b8d8eba181bc6693db16679fcde2c4bb1a24421faa187dc7665c42b` 不变，没有重投或新身份。

公共 `ui/graph/mod.rs` 在两类型最小导出阶段的 SHA-256 为 `29584afa5a5953f33528b1514074f94c6f1bddd72a0ef6e167fcae0b9a24af3e`；这是原材质提交阶段身份，最新四文件辅助接口/指纹见上文和 `graph-neutral-presentation-02.json`。private model、定义/构造、retention/revision/prepare/一次 permit 保留。九路径 guard-18 的 `0d2514ea…` 是提交前静态证据；后续另留新守卫，不覆盖原回执。原请求终态和有界源码稳定窗口仍交原实现/monitor 对账；不放宽 drift guard、不复制源码、不本地 Cargo 代验。材质13项、独立core19项预计批、外 crate消费编译和正常 EXE验收分别未完成。

以下保留此前修复、首次入口和恢复的历史观察，不覆盖上面的实际提交终态。较早 public capability 为 `ready`，但 `nativeLifecycleVerified=false`、正式 tray `ownerKnown=false/canStart=false`；本会话没有据此接管或重启服务，公开 ready 不代表材质测试或正式验收。

Parent 的 `material-plugin-test-entry.json` 在 2026-10-24T10:42Z 确认当时 capsule `37d84f0c5b4e950ead63fba8523c4da1bc9add28a4618198852775e67c764f8b` 不含 manifest selector，实际 parser 拒绝 `--manifest-path`，未启动 Cargo。稳定修复请求为 [ue-plugin-workspace-entry-20261024](../../../.jenkins/records/ue-plugin-workspace-entry-20261024.request.json)，归原管理实现 owner 和既有 monitor；原请求字节与旧失败保留。后续 [route-02](../../../.jenkins/records/ue-plugin-workspace-entry-20261024.route-02.json) 于 11:35Z 记录 `baac70c97872d3395c5c422fa44a7cdaa481a8ac4f62820d1cc2ac099ad0ac08` 已安装受限 manifest 的 parser/runner 路由，但明确不是服务端执行回执。不能用较早“不支持”覆盖该路由，也不能把路由观察当 Rust/原生验收。

先前独立只读对账曾核实实际 selector 为 capsule `cdd14500fa39088ae2c9870f62825acc2e526ba647d24d77cd6287722b5d0f31`，driver/runtime 为上段身份；完整 capsule/pinned inventory 和 public HTTP receiver 绑定校验通过。其 pinned entry SHA-256 为 `5e10bc46daaa2e891d373d2dceb66e6497d4a477d5a9ff9a1f5bda96b45fcc9a`；按实际入口调用 `parse_cargo(arguments, repo=canonical_repo)`，完整绝对 manifest argv 解析为 `zircon_plugins/Cargo.toml`、唯一 `zircon_plugin_material_editor_editor`、defaults off/features=[]、lib/test profile、`material_graph_`、jobs1/testThreads1。首次只读探针遗漏 repo 导致 manifest 拒绝、随后旧 helper 名称 introspection 失败均保留，并与最终 parser-only 成功分开；这两个探针问题不记作部署故障。该次独立对账当时只证明受限解析，不能替代 runner/source/lock/config/pathdeps、pool、服务端或原生验收。此后的实际材质托管提交和 preparation-conflict 终态见本节前段，不能再写成“从未提交”；13 项回归仍 not_run。

最新原 owner 交接确认 `ue-plugin-workspace-entry-20261024` 已由 `01a12af6-7511-70d1-a452-61b80678eb56` 接单；请求 JSON 的实际 SHA-256 仍为 `e5af9470c7ebc7a4f84ec9b3540f949fc5e968eaa9ed1c3f7b02085fb8cad13d`。交接报告 baac70… 的 #398 真实安装及45项相关检查，后续 nested manifest、实际 lock、`.cargo`、跨 workspace path deps 源码守卫、pool/实际回执和 native/EOF 链均由该 owner 在原任务树专项补齐、正式发布。该报告与上述后续实际 cdd145… 观察分别保留来源，不据旧 37d84… 或单独 parser 结果推翻交接，也不把不同时间记录当当前安装回退。本域未修改入口、未并行追修其守卫、未重放服务恢复/旧材质请求、未创建新 Cargo identity、未运行 Cargo；auto-restart/mergefetch/cleanup 原状态保留。完整可用性继续等原 owner 的真实交付回执，本域继续独立设计。记录为 `plugin-entry-owner-handoff-02.json`。

补丁身份同样按回执区分：传入 `7b0d948f…` 与 Parent 实读 guard-02 的 `bba23679eb3825e43629505d31c973cc3bc3514fca9ee5e41ef0ea19ed3e5fb0` 均为历史，不改写；本次开始前已推进到独立 guard-12 的 `08834be8ba02ddbdc6b47e23cd9e0a2e2b8aeeb83d04d1fc6318557b9acef198`。其精确 8 路径源指纹与实际 checkout 一致，五个 Rust 文件及 README 仍与 Parent 旧记录相同，后续变化是本域计划/共享合同。当前本段又更新一份计划，以新 guard-13 和 `plugin-entry-reconciliation-01.json` 记录最终源摘要；各轮 receipt 不互相覆盖，未来任何真实提交必须重新匹配当时实际源码和入口。

`tools/jenkins-management/entry.py capability` 返回 endpoint connection refused（WinError 10061）。正式 tray `status` 显示 `ownerLost=true, canStart=true`；通过正式 `start` 尝试恢复，在 `lost_recovery_runtime._artifact` 校验 `deploymentSpec` 时以 `Original runtime evidence changed` 拒绝。本会话未修改服务身份、部署配置、锁、历史回执，也未启动退役协调器。Parent 消息明确请求启动 Penpot、协调 Jenkins 恢复和桌面排队；队列接收不表示 Parent 已读取。

后续观测：Parent 新 Penpot supervisor `31848` 的 `penpot-runtime-01/current.json` 已记录全新 PID/birth 和前后端 HTTP 200；本会话独立 `HEAD http://127.0.0.1:19001/` 返回 200。原 Runtime11/P21 的缺失终态仍保留，未拿新启动补旧退出证明。Jenkins 18080 已监听但 capability 仍为 pending，原因已变为 `Selected runtime changed`；当前 driver runtime ID 为 `start-1792836412672596900`，Parent 已交既有 Jenkins owner 恢复，不由材质侧重复启停。

编译仅选择 `zircon_plugin_material_editor_editor`，关闭默认 features、显式空 feature 集，library test 过滤 `material_graph_`，test profile、单 test thread。插件位于独立的 `zircon_plugins/Cargo.toml` workspace；Jenkins recipe 必须显式支持该 manifest/cwd，不能在根 workspace 选择不存在的包或扩大全仓。执行前按 AGENTS 声明完整命令及影响依赖；所有编译/缓存物理在批准的 `E:/cargo-targets`，复用兼容受管池。当前没有提交编译命令或将 dry run 当作通过。

最初记录的 parser/runner 不含 manifest selector、checkout 固定 canonical repo root 的缺口和启动问题曾追加发给 Parent，队列 ACK 为 `01a16d5f-5048-7020-a7fe-ff7ae33a2f92`；该旧观察/ACK 保留，不代表当前 pinned parser 状态。需要的有界执行语义仍为 `cargo test --manifest-path E:/Git/ZirconEngine/zircon_plugins/Cargo.toml -p zircon_plugin_material_editor_editor --no-default-features --locked --lib --profile test --jobs 1 material_graph_ -- --test-threads 1`，不是本轮已执行命令。传递依赖含 runtime `native-host` 和 editor `target-editor-host`。真实执行仍须先声明当时精确范围，并由受管入口按最新 source/lock/config/pathdeps 与物理 E:/cargo-targets 池核对。

早期已执行五个 Rust 文件的 `rustfmt --check --edition 2021 --config skip_children=true`、限定材质路径的 `git diff --check` 和精确七文件静态守卫；五个 Rust 源指纹未变，复用其格式证据。新增共享合同后，静态守卫范围为精确八文件，base 仍为 `75a76ee963792d5a316f738519e966937f770036`。本次只改一份材质计划，检查相关链接/锚点、差异及同八文件守卫；结果和最终摘要保存在独立 guard-13/对账回执及清单，不替代 Rust 编译、Jenkins 执行或原生验收。

产物清单：`E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/artifacts.json`。未创建源码副本、构建或生成夹具；隔离浏览器均已正常退出，实际 Penpot 原图、API/浏览器回执、静态检查及同配置的浏览器临时目录继续保留接续。未执行删除，释放空间为 `0 bytes`，任务未关闭。

## 节点搜索接续35：稳定 action ID 的点击确认

本域在 palette_search.rs 增加 MaterialGraphNodeSearch::activate_id(id)。它以当前真实可见的稳定 action key 一次完成行选择和激活；合法点击返回原注册 descriptor 一次并关闭临时状态，已被过滤/未知key返回None且query/results/当前selection/open完整不变，取消后迟到点击也不产出动作。原 select_id、activate、导航和query API保持。对应UE SGraphActionMenu.cpp:1117–1122、1222–1234直接投递实际所选action；没有把显示文字/旧行序号当身份。

新增2项状态回归定义覆盖点击非当前行的exact descriptor/一次激活与重放拒绝，以及query改变后的旧行点击不关闭弹层、不产生别的action并仍可正常键盘确认；原cancel回归补真实迟到pointer调用。当前 material_graph_ 静态定义77＝76普通＋1原ignored（全lib80定义中另3项不匹配过滤器）；rustfmt/diff/精确2路径guard均0错误/警告/抑制，未Rust发现/类型检查/执行。最小patch SHA0ff11568cf14baeba8c242f013ada0f93ec366cb1c0a880a097d747c76e64541；before/validation/guard/handoff保留原reports的 material-node-search-click-35-*。

这是菜单局部检查，不证明host callback/popup incarnation或document/source/revision ABA安全，不改变focus/IME/Document/history/permit；原host必须对实际打开者上下文、输入owner、真实palette/schema与创建revision作guard，然后消费33/34领域预检/一次prepare。没有构造第二store或假节点。Source35只改2个本域Rust；fork30的Instance四SearchInput根2b056488/8ac3c5fd/dc07cd1c/0cd099fe、Hierarchy3对象/原prototype、原三搜索板及所有其它Penpot对象本轮均未写。fork30可用fresh字节校验继续插入独立小节，本域只追加自己的记录，保留既有内容。

原request ue-material-graph-20261024-01 / operation matgraph-autowire-01a16d50-20261025-34 / queue16229/#391不重投、不改identity。09:41:32Z认证API仍building=true/result=null，receipt仍waiting/management_git_busy；compileStarted、实际发现/执行和native/resource仍unknown。其原identity9f8f7982…的30输入现在28匹配，两处差异正是source35搜索模块/测试，本轮透明修订而非来源不明；旧source34的75计数保持历史，后续只按实际捕获源和真实发现数判断结果。没有新operation、local Cargo、gate/锁/服务或公共源变更。goal active，passed=false/accepted=false；源作者宿主、renderer、official export与normal EXE/144DPI/双端1DIP仍open。

## 接续36：fork30 的十三图直接复核与弹层消费边界

原 producer 已逐张打开 fork30 的13张实际PNG，并核对 material-fork-30.json 的 SHA db9e4ee61ecb4575d3989c53bc3676e6997720c167f3e792b4100ab04a2f25fb、全部13图哈希和3份参考/原源指纹。独立逐图记录为原E根 material-fork30-consumption-36.json；不修改fork原回执、旧失败或其四个Instance搜索copy/Hierarchy对象。所有图均为Penpot workspace/viewer；以下不是EXE、实际144DPI或1DIP测量。

| 原图文件名（同任务reports根） | 本会话直接复核 |
| --- | --- |
| material-hierarchy-30-workspace.png | 可编辑1280板显示Parent Chain/No Parent Found，原生返回连线可见；空预览仍为Preview unavailable。 |
| material-hierarchy-30-reopened.png | 新context重开保留层级文案/布局；71%画布显示不用于像素容差验收。 |
| material-hierarchy-30-prototype.png | viewer100%打开态菜单位于Hierarchy下，Parent None；单图不独立证明输入已投递。 |
| material-hierarchy-30-prototype-closed.png | 初始关闭基线无菜单，不当作外点/Esc成功。 |
| material-hierarchy-30-prototype-returned.png | 返回后菜单消失，按钮青色描边可见；描边不能证明真实焦点已回到打开者。 |
| material-hierarchy-30-prototype-escape.png | 失败保留：菜单仍可见；外点/Esc步骤及BODY焦点来自fork操作回执，不能由截图补造关闭成功。 |
| material-instance-30-900.png | 旧240宽搜索未填满Details内宽。 |
| material-instance-30-640.png | 旧搜索背景在窄Details右边界被裁。 |
| material-instance-search-flow-31-640.png | 根单独Fill仍有背景/文本裁边，保留失败试验。 |
| material-instance-search-flow-31-640-corrected.png | 原背景/文本left-right后，两侧内边距可见，无可见溢出；174.8宽/8内边距是fork API几何，非本图测量。 |
| material-instance-search-flow-31-900.png | 搜索填充Details可用宽度，图标/文案保留。 |
| material-instance-search-flow-31-1280.png | 同实例按可用宽度填充，空父/空预览状态保留；71%图不签1DIP。 |
| material-instance-search-flow-31-reopened.png | 新context的640板保留修正布局；保存身份一致性复用fork API证据，不扩为原ZUI导出通过。 |

复用的是Shared SearchInput原component/main 6ddc248b-bd35-80c5-8008-d6c8c3ce892e / 6ddc248b-bd35-80c5-8008-d6c8c0f9c196，源Stretch消费为native Fill，并保留原背景/文本left-right、图标left16。四板的consumer override不是canonical source delta；source token/override/reset及正式导出仍交原public owner，不能detach、重签基线或改Shared来抹掉失败。原 material_workspace.zui 的5856c886…、29节点/27control/13events保持。

已将Hierarchy和节点搜索共同需要的真实popup source/input/focus边界接续到 [共享合同36](shared-node-graph-contract.md#接续36弹层关闭焦点和可逆布局消费)。域只交实际action ID或完整AssetRef，no-parent行不能导航；外点/Esc必须真正关闭原overlay并拒绝迟到callback，焦点返回须验证实际打开者及当前source，不能用按钮颜色替代。此轮无新Penpot/公共host/Rust/原ZUI写入、无新测试identity或POST；#391沿原操作，09:49:48Z读回与09:41:32Z同waiting/management_git_busy，无变化不另造终态。source35的77静态定义与原source34的75继续分列，Rust/renderer/EXE和整族passed/accepted仍未通过，goal active，释放0B。

## 作者语义接续37：拖线创建时选择真实对向端口

本域新增 suggest_material_graph_autowire(graph, action, node, anchor, resolve_kind)，anchor直接复用共享GraphPortRef<String>的实际node/schema pin/direction。它先拒绝已不存在、歧义或方向/端口不符的源，再验证原注册action/真实typed候选/完整纹理引用；按注册的对向端口顺序，用既有材质连接分析器选择第一条合法候选边。没有用caption/截图坐标或palette的float文字判断实际vector math/texture兼容性，也没有克隆图、分配ID或建立另一套模型/clock/history。

依据只读 UE MaterialGraphNode_Base.cpp:351–391 的 AutowireNewNode 首个兼容pin循环。本次精确支持新节点接空输入、现输出接新节点首个合法输入、真实texture/color与scalar/vector域类型；无对向/无兼容端口时Ok(None)，允许未接线的作者节点，不将其算编译ready。UE已连输入的InsertNewNode需要保留原边并插入多边；本单边建议对此明确返回原实际Input定位的multi-edge insertion错误，绝不静默丢旧边或把显式replace冒充完整UE插入。原34显式重连plan/API保持。

实现只改本域lib.rs并新增node_pin_autowire.rs、tests/node_pin_autowire.rs三路径；16保护输入（既有33/34/35域逻辑、公共graph/model/commands、原ZUI、manifest/lock和UE参考）全未变。最小后继patch SHA1d6f4d05812d24ab64bf9d7a6618e59574adebf0077b7f5c15e6ca4f8db90f85；lib原像在内存逆转本次三个插入并精确哈希匹配，未建源码副本。格式/diff-check/精确3路径guard通过，0错误/警告/抑制。七项新增回归定义覆盖首次端口、反向拖线、真实vector math、texture→Output/拒绝math、stale/歧义/方向及resolver先后、占用输入不丢边、无匹配保持可创建；测试仅投影DTO调用真实compiler，不代替host/undo/renderer。

当前 material_graph_ 静态定义84＝83普通＋1原ignored，全lib87中另3项不匹配；未Rust类型检查/发现/执行。原#391身份/POST不改、不重投，10:09:40Z认证仍building=true/result=null、receipt waiting/management_git_busy。原30声明当前27匹配：source35的两搜索文件及本次lib后继透明登记，新增两模块另附当前指纹；旧75/77不得改成84通过，也不得把当前源塞回旧终态。实际managed capture/发现数须以server结果为准。详细before/validation/guard/patch/handoff位于原E根 material-node-pin-autowire-37-*；host仍须同source/revision/geometry消费建议并用34plan、原一次prepare/retention/permit/history，真实落源/焦点/保存/renderer/EXE144DPI继续open。未改Penpot、fork对象、public或服务/锁/策略，无新编译身份，goal active，passed/accepted=false，释放0B。

## 作者语义接续38：已连接输入的插入与显式替换

本轮仅改本域 lib.rs、connections.rs，并新增 node_insertion.rs / tests/node_insertion.rs，交付 plan_material_graph_node_insertion 与借用式 MaterialGraphNodeInsertionPlan。实际 anchor 必须是当前 MaterialGraphAsset 中唯一节点的真实已占用 Input/schema key，且恰好一条旧边；先拒绝失效方向/节点/端口/多边和损坏旧来源，再复用33创建预检、当前 descriptor 顺序和原 typed analyzer。新计划借用候选节点及 exact old edge，返回有界的一条或两条边，不复制整个图、不分配作者ID。

对齐只读 MaterialGraphNode_Base.cpp:187–218 的 InsertNewNode：先选合法新输出，再按注册顺序尝试旧来源→新输入，并共同检查两条新边及剩余整图。存在合法接收端时返回两边；没有合法接收端时，明确返回新输出→原输入的一边替换，reconnects_previous_source()=false，原来源节点和其它 fanout 保留。这包含 scalar→color 的既有 RGB 广播/alpha=1 行为，不改参数ABI；纹理法线仍要求真实纹理采样，不把Math画成后端支持。37原单边suggest接口仍拒绝已占用输入；实际host须对此分支选择38接口，不能默认单边调用便宣称完成插入。

七项新增回归定义覆盖 exact borrow/其它 fanout、两边共同作用的标量约束与显式fallback、不完整Math的真实诊断、中文稳定ID/位置的实际TOML重开、进入现有PBR编译器、纹理法线拒绝、六类失效anchor先于资源解析，以及断来源/断端口/非输出分量循环。当前 material_graph_ 静态定义91＝90普通＋1原ignored，全lib94中3项不匹配过滤器；这不是Rust发现数或执行结果。精确四路径 rustfmt/diff-check exit0，final guard 0错误/0警告/0抑制；19保护输入（原33/34/35/37、公共Graph/原ZUI/manifest/lock/UE参考）全部未改。最小增量patch SHA5fccae44fe7bad123fa965a84ebdf8c406586ce88e91f1d40c440971f20e5e4f；初稿guard另存，最终版本明确补齐UE的一边fallback，未将初稿当最终验收。

原request ue-material-graph-20261024-01 / operation matgraph-autowire-01a16d50-20261025-34 / queue16229/#391：10:29:16Z认证API已 building=false/result=FAILURE，归档仍waiting/management_git_busy。实际console保留240次acquiring management-git.lock、0条validation_with_cache returned；不能以job终态填造compileStarted=false、测试0或native/资源释放。本轮不重投该operation；后继如实际提交，另用当前源码与新operation透明登记。旧31/#386、r10/#370及其它失败/native缺证均保留。

宿主必须把节点插入、exact旧边移除和新的一/两边在同一真实Document/source/revision下，一次prepare/permit与一次inverse-history发布；现计划不授予执行权限。真实源绑定输入/取消/焦点、事务/撤销/保存、正式codec导出、实际renderer与正常EXE/150%DPI/双端1DIP仍open。fork30四Instance SearchInput/Hierarchy、本域三个搜索板、Shared/catalog/theme/runtime/原ZUI本轮均未写；只追加fresh文档，完整保留两侧已有内容。回执沿原reports material-node-insertion-38-*，整族passed/accepted仍false，goal继续active。

## Penpot 接续39：插入/替换的实际可编辑设计

原file b4a1e3ce-9e6a-81fe-8008-d69d6092d704 / page ...d705追加4板、213对象、15个原材质组件实例；rev156→157保存，局部修正后rev161/2380对象。没有新增主件/库或改Shared/catalog/theme/runtime/原ZUI，fork30四SearchInput/Hierarchy和原Node Search31三板保持。该设计对应源38实际DTO/编译器用例；填齐B后的Multiply及0.8替换结果文字是预期值，不是实时GPU输出或Rust通过证据。

| 状态/逻辑尺寸 | 实际board UUID | 逐图复核 |
| --- | --- | --- |
| Multiply插入后1280×800 | 00233501-f375-4251-b690-cae0eef921dd | 4节点/4边、Emissive分支、连接/未连接端口状态完整 |
| 同图900×620 | ac36f020-e5b9-4d26-a0de-158098b86baf | 同逻辑布局和身份，文字及节点无裁切 |
| 同图640×520 | c43b9bad-1873-4dc7-bf1d-ed209854ef7e | 缩放0.7189873418一次，中文/值完整，不删除节点 |
| 明确替换900×620 | bb767f77-a1d0-46b2-b08c-4dc76626ed33 | New Value→Base Color，旧来源节点及其Emissive边保留 |

实例保留真实componentId/componentFile/shapeRef。Parameter、Math、Output组件分别904a343b-18f1-4af9-928f-b7d5d028a69a / f9ea0da8-6a97-4ccb-b57d-b31b273ea4f0 / d7173cd6-1da7-4897-be89-08583d32f598，仍为本域proposal，不冒充公共Graph library。新框架/文字/线及端口覆盖消费真实Shared颜色/Body引用，不把本地覆盖当全局继承/reset或双族A/B。材质线依据MaterialGraphConnectionDrawingPolicy构造不画箭头，数据线1.5依据GraphEditorSettings。

保留全部失败：原Parameter copy隐藏正文、文字缺原生测量、同SPA换frame-id产生空图、API缺Accept时返回Transit；第二轮发现保留边穿节点及pin实心状态错误；第三轮验证器误将规范化SVG path字符串当命令数组。只在213集合内修正16个hidden文字、共同world布局与实际pin shapeRef对应的连接状态/描边，通过workspace原生图层选择/Shift+2和字体测量得到真实行框，保存关闭后每板独立新context重开。未改旧主件或重签codec。初稿API追加时除根children增加新板外，原2167对象逐对象无差异；最终原生测量/重开前后集合外变化0，原5主件身份和15实例源引用保持。

最终四PNG已逐一打开复核：浏览器DPR=1，真实画板client边界对应上述四尺寸，文字越板均0，API线端点相对真实pin中心最大差0.000449逻辑像素；这仅为Penpot设计几何，不是双端1DIP或150%DPI验收。所有自有browser已退出，auth未复制/改写。逐图行框、路径、哈希及失败批次沿原reports material-node-insertion-39-verification.json和verified PNG保留。

源38有限validate实际POST为原request ue-material-graph-20261024-01 / matgraph-insertion-01a16d50-20261025-38，identity ab538339…、input f31cc819…、queue16803→Jenkins#396；10:52:15Z认证API仍running，归档waiting/checkout_source_busy及确切blockingReadOwner已交原owner，不重投/释放未知reader。91静态定义不是发现/执行，compile/tests/native/resource unknown。源38同哈希guard复用；本轮只检查两文档结构/链接/静态守卫，不另跑Rust。

真实Document/source/revision/retention下输入与一次prepare/permit/inverse-history、正式owner codec/原ZUI导出、实际renderer、正常EXE/焦点/捕获/150%DPI/双端差异仍open。Material Instance/Function未完项沿原台账；整族passed/accepted=false，goal active。


## 作者适配接续41：真实材质源到共享 GraphNodeView / GraphEdgeView

本轮在原材质叶包新增 `MaterialGraphPresentation` / `MaterialGraphEdgeProjection`，精确五路径是 lib.rs、connections.rs、node_creation.rs、新 graph_presentation.rs 及 tests/graph_presentation.rs。它借用当前 MaterialGraphAsset 和现有索引，复用当前连接分析器一次得到类型结果；没有创建 GraphModel 的平行实现、Document/store/clock/history 或 mutation permit。三个原文件的修改前像已在内存逆转并逐字节匹配；十六保护输入（公共 Graph、宿主三行面板、编译器、原 ZUI、manifest/lock、dist 等）当前全部未改。原八项草稿及 guard 保留，最终十项版本另存 final 回执，不能用草稿覆盖最终源。

真实 source ID、parameter name、作者 position 和当前六类节点的注册 schema key 进入既有 GraphNodeView。节点尺寸由宿主传同代际 retained 实测逻辑尺寸；零/负/NaN/Inf、加法溢出及浮点分辨率已令边界坍缩时返回该真实节点诊断，不填默认截图尺寸，不再乘 DPI。当前实际输出类型来自原连接分析器，Multiply 的 vector×scalar 不被 palette 的旧 float 字样误标；未接齐的 Math 可以呈现 undetermined，required-input 与完整编译诊断保持。类型错误不把整图隐藏：仍返回可定位的真实节点和端点有效的原 wire，另留真实 type_diagnostic 供修复；缺失节点/端口不造替代几何。无节点无边也为合法可见作者空态，真正编译仍报缺少 Output。

`connection_plan(from, to)` 直接消费共享 GraphPortRef 的方向和实际 node/schema pin key，调用原 can_connect_material_graph，保留材质的 scalar/vector/texture 约束与 exact old edge 的 Replace/Unchanged 结果；不能用共享默认字符串比较代替该域规则。edge_view 保留原 link 引用及同源 source_index，并投影真实 output→input PortRef。现持久 MaterialGraphLinkAsset 仍无独立 edge UUID，index/地址只是本次 source borrow 的 occurrence，不得在后继 revision 当稳定身份，也不将此差异隐去。

十个新增回归定义实际消费现有 GraphNodePresentation、GraphCanvasState、GraphPortRef、真实 TOML codec 和 PBR compiler：六类节点/Math混合类型、选择与同一变换、未接齐输入、错误 wire 可选修复、非法/溢出行框、失效/歧义 ID、损坏端点、中文同名不同 ID、合法/拒绝/原边替换，以及无假节点的空图。当前 material_graph_ 共101静态定义＝100普通＋1原ignored，全 lib104另3不匹配；均不是 Rust 发现或执行结果。最终五路径格式/diff-check exit0、guard 0错误/0警告/0抑制，最终最小增量 patch SHA1587c3242bb3632d79a8afefc0e9c73f8655776911ffc63cde82284d6b1b0691；证据沿原 reports/material-graph-presentation-41-final-*。

原 request ue-material-graph-20261024-01 / operation matgraph-insertion-01a16d50-20261025-38 / queue16803/#396 保持，11:27:16Z 认证 building=true/result=null，receipt checkout_source_busy 与原 blockingReadOwner 不变。原 identity ab538339… 的34声明当前31匹配，三处变化就是本轮 lib/connections/node_creation；两新增文件未塞入旧 payload。旧91定义/旧源 guard 不签本轮101定义，实际编译/测试/native/资源仍 unknown；此文不重投原 operation。

给 Parent 的精确生产接续见 [共享合同41](shared-node-graph-contract.md#接续41实际材质源投影与可修复错误图)。目前只有域 adapter 与静态消费者代码，真实宿主仍是三行固定表格和 Not implemented compile feedback；没有把单元夹具或 DTO 编译值当实时 UI / renderer。Parent 需在原 source/document/revision/retention 与当前 pane/geometry 下接 read-only projection、实际 paint/hit/选择与输入，再由原 prepare/一次 permit/inverse-history 发布作者修改。原官方 codec/资产导出、实际 GPU 预览、正常 EXE/150%DPI/双端1DIP继续 open。Penpot39 rev161四板复用原保存/关闭/重开和逐图证据，本轮未伪称新图/新视觉通过；fork30四 SearchInput/Hierarchy、Shared/catalog/theme/runtime/原 ZUI 均未写。goal active，整族 passed/accepted=false，释放0B。


## 作者适配接续42：多条真实输入边的歧义类型与41实际收件

- owner=`01a16d50-6c7f-7370-853d-4581184b66a1`。41正式entry已一次提交：request `ue-material-graph-20261024-01`，operation `matgraph-presentation-01a16d50-20261025-41`，queue17524／`zircon-management #400`。2026-10-25T11:43:40Z认证API为building=true/result=null；原receipt为waiting/`management_git_busy`，operationKey=`50b0882a6868a4fbf5154141973d6388ee0ee1491de40baf8d647345e0c2644d`。compileStarted、实际Rust发现/执行及native/resource仍未知，不重复投41，不复用旧失败关闭它。
- 41提交身份`3d05ed48…c3355`、提交前39声明输入及101静态定义原字节保留。实际收件/观察见[delivery41](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-graph-presentation-41-delivery.json)和[observation01](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-graph-presentation-41-build-observation-01.json)。#396的FAILURE/checkout_source_busy与原未知native独立保留。
- 本次42仅改本域`graph_presentation.rs`及其原测试文件。旧实现对多条incoming先取第一条的类型，与真实analyzer的歧义诊断矛盾；现在只有incoming count=1才派生Math输入类型，多条显示undetermined。真实Input(tint,a)诊断、原links顺序、每条原source引用和source_index均保留，不去重、不自动修图。
- 一项新回归覆盖不同来源类型的两种顺序，以及相同端点的两个真实occurrence的两种顺序；同时检查未确定类型、未受影响的另一个输入、编译仍拒绝、两条边可分别投影、源不变。当前`material_graph_`静态定义102=101普通+1原ignored；未Rust发现/编译/执行，不将102塞入41旧payload。
- 两Rust的rustfmt/diff-check与精确change guard均exit0，guard errors/warnings/suppressed=0/0/0，其他37声明输入全部匹配。增量patch SHA`8ce5e411392ad16ec23184221493e1d819a4c4a40e0e1820a96eefe3a3c71559`，并在内存逆推重建41原字节/哈希；未建源码副本。完整[42交付](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-graph-ambiguity-42-handoff.json)，SHA`9719bfa4023e9458e8f5efba2e450bee1a7c1fb9be0c9619c2f2cc9fc4cd657f`。
- 本轮无新managed派发、Penpot/四Instance SearchInput/公共Graph/theme/catalog/runtime/原ZUI修改。41的正常执行不因42被取消或重放；真实host retained几何/命中/焦点、一次permit/undo接线、官方codec/export、renderer与正常EXE/实际150%DPI仍open。goal active，passed=false，accepted=false，清理0B。


## 材质连线接续43：UE 切线修正、原生保存重开与公共最小补丁

- 本次只改材质原 file `b4a1e3ce-9e6a-81fe-8008-d69d6092d704` / page `b4a1e3ce-9e6a-81fe-8008-d69d6092d705` 内39的四板14条既有path，实际rev161→162，2380对象不变，新增/删除ID=0。四板为1280×800 `00233501-f375-4251-b690-cae0eef921dd`、900×620 `ac36f020-e5b9-4d26-a0de-158098b86baf`、640×520 `c43b9bad-1873-4dc7-bf1d-ed209854ef7e`、900×620 replacement `bb767f77-a1d0-46b2-b08c-4dc76626ed33`。端点、原15个native node copies、共享stroke引用、原节点/作者位置及所有非14对象均保留；fork30四SearchInput/Hierarchy未触碰。
- 只读UE `GraphEditorSettings.cpp:27`／`:155` 的forward tangent为clamp(abs(dx),1000)+clamp(abs(dy),1000)，`ConnectionDrawingPolicy.cpp:310`至`:352`消费Hermite travel tangents。对应cubic controls为source+tangent/3、target−tangent/3，不是既有共享max(abs(dx)/2,48)。材质`MaterialGraphConnectionDrawingPolicy.cpp:31`至`:40`关闭箭头，保1.5线宽。14条现有forward wire均未达clamp；以实际端点计算，不按截图挪节点或做障碍绕线。完整3份UE源码哈希见[源与最小提案](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-43-host-proposal.json)。
- 初次把read API的normalized SVG字符串当authoring content被HTTP500拒绝，fresh GET仍rev161。随后使用原生move-to/curve-to command array实际HTTP200保存到rev162；原验证对double的1e-6假设失败单独保留。新只读核验确认原生float32归一化结果逐坐标精确相同、端点完全保留，最大double→native量化8.1625e-05。没有为了验证重放第二次写入；两份失败回执不覆盖。
- 已打开实际workspace并保存、关闭原context，四板各用新独立context重开截图；之后GET仍rev162，workspace/view期间对象变化0。四PNG已各自人工逐图复核：4/4无观察到的文本裁切/端点脱离，640保原0.718987图缩放而非删节点。截图仅设计图表面，底部数值来自39真实域用例的设计投影，不冒充引擎GPU预览。原生细节见[关闭重开](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-43-reopen.json)和[逐图复核](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-43-review.json)。浏览器/全部context已关闭，auth字节未变，own tmp为空。
- 公共扩展未抢写：交付[可应用性检查通过的5287B补丁](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-43-routing-proposal.patch)，SHA`89bced7589380225de6e4b1b06e4619fd0c8c1ed846eefaa69c0552d7d29cc24`。仅建议routing.rs新增optional显式Hermite切线构造，mod.rs导出、cases.rs三项有意义回归；旧route/default/cardinal路由不改。精确三preimages仍匹配、git apply --check exit0，未应用/未编译/未执行候选三测试。Parent持公共集成，不能用提案关闭shared host或UE全线型差异。
- 当前真实host仍是material_workspace_state.rs固定三行，dist native invoke_command=None/empty manifest；六operation descriptor是metadata，不是已绑定NativeEndpoint。需在既有command registry/原Document/source+revision/request/view上下文中接typed有界payload/result，再经原ResourceManager/renderer发布。不得从effective MaterialAsset重建ZMaterialDocument、伪造编译成功或加第二store/backend。精确路径及保存/editor metadata/一次permit/history限制写入同一43提案。
- 原41唯一producer queue17524/#400在2026-10-25T12:12:38Z认证API仍building=true/result=null；receipt仍waiting/management_git_busy、SHA`4a9b78736057f69a5bd4bad1f731f79fee74c107f4b0a7010e200cf214205f75`。03字节序列化包装错误保留，04仅从其原body解码、无第二HTTP。[04原观察解码](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-graph-presentation-41-build-observation-04-decoded.json)不补造compile/native/测试终态。42当前两源和102静态定义保留，不改41身份/不重投；原ZUI29nodes/27controls/13events不变。
- 正式codec/export、源ZUI落地、host paint/hit/selection/focus/history、真实renderer修改、正常EXE/实际150%DPI/双端1逻辑像素仍open。整族goal active、passed/accepted=false；本轮无公共源/Shared/theme/catalog/runtime/桌面/服务/锁写，释放0B。


## 材质连线接续44：真实 edge 的 UE 切线投影与有限受管后继

继续原 active goal，唯一 Node/Graph managed producer 不变，fork30 的 Instance/Hierarchy 与四 SearchInput copies 不动。沿43已保存重开的四板，把 UE 默认材质切线 policy 落实到原 `MaterialGraphPresentation`：`edge_spline_view` 先校验真实 source occurrence/node/pin，再对调用者实测 graph-local 锚点使用同一 `GraphCanvasState` 一次变换，按绘制空间 dx/dy 计算默认 forward/backward travel tangent。保留借用的原 link/source_index；重复、类型错误但可修复的真实 wire 可投影，失效 pin/索引、NaN/Inf/端点或差值溢出拒绝，不修改作者图、selection/camera/history，不发明 edge UUID、锚点或第二 store。

UE 参考为原只读 `GraphEditorSettings.cpp` 155–170 和 `ConnectionDrawingPolicy.cpp` 的绘制空间契约；默认 forward=`min(abs(dx),1000)+min(abs(dy),1000)`，backward=`2*min(abs(dx),200)+1.5*min(abs(dy),200)`，零跨度保零导数。材质本身无 ArrowImage；本轮不为其虚构箭头。公共 `GraphConnectionRoute` 私有构造未绕过，43 optional explicit-tangent constructor 补丁仍待 Parent 集成；本轮结果只是已有真实域 edge 的逻辑几何策略投影，paint/hit/input/geometry-generation/焦点及正式 host 接线仍 open。

实际增量四源：`graph_presentation.rs`、`lib.rs`、原 `tests/graph_presentation.rs` 的 fixture 可见性，以及新叶子 `tests/graph_spline.rs`。6 项回归覆盖正/反向权重与 clamp/零值、一次变换后 clamp、有限值和溢出、原 occurrence/修复诊断、三组43设计曲线 policy。当前过滤器下 108 个静态定义（107普通+1既有 ignored），尚非 Rust 发现或执行数。[最终4源/精确逆补丁](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-policy-44-source-final.json)、[验证](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-policy-44-validation-final.json)：readonly rustfmt、diff-check、精确 reverse apply 皆 exit0；范围 guard 四路径0错误/0警告/0suppressed，36项原41其他输入及10项只读参考不变。

保留本轮失败：带写入 rustfmt 从 lib 跟随子模块导致25个域文件换行变化，已逐项证明 LF 逆变换等于原输入 hash 后恢复到原字节；另首版补丁被 Windows write_text 转为 CRLF 导致 reverse check 失败，失败补丁和检查独立保留，最终补丁用精确 LF 且无需忽略空白。详见 [format repair](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-policy-44-format-repair.json)，不以失败首版背书当前源。未创建源码副本，未触公共/Runtime/原ZUI/Shared/catalog/theme。

原41 operation `matgraph-presentation-01a16d50-20261025-41` / queue17524 / management#400 于12:29:22Z认证读回 building=false/FAILURE，归档仍 status=waiting/reason=management_git_busy/Another process owns this operation；compile、实际测试及 native/资源仍 unknown，不据API失败释放锁。原 [observation05](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-graph-presentation-41-build-observation-05.json) 保留，不重投41/r10。按用户短空档请求已给原管理 owner 12:30:28Z–12:31:13Z 的45秒，只推迟本 producer 下一次 management dispatch，到期自动恢复；不 freeze 源、不取消在途，消息只算 queued。

44 透明受管后继仍在原 request `ue-material-graph-20261024-01`，operation=`matgraph-spline-01a16d50-20261025-44`；当前16 owned +25 dependency hash匹配，身份 SHA3db4e59f9e5c8c43c231e1a724485ce38c76d990b5837941ca7f5ee31441680d。已作编译声明并启动一次正式 `tools.dev.local_cargo` client，唯一 nested manifest/单材质包/features[]/defaults-off/locked/lib/profile test/jobs1/material_graph_/test-threads1/物理 E cargo-targets，传递 Runtime native-host/Editor target-editor-host。正常 prepare 取得本次 source/writer/resource/pool admission；不先索手工grant，不清原锁。[44 managed](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-policy-44-managed.json) 与 [delivery](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-policy-44-delivery.json) 随实际结果更新；没有重复旧 terminal 操作。正式 export/真实renderer/正常EXE/150%DPI/双端1DIP与整族 passed/accepted 继续 false/open。


## 节点搜索接续45：三张原生实例的横向流式约束与真实重开

本轮只修改原 NodeSearch31 三张400×400板中 SearchInput 的 background/text 共6对象：`constraintsH=left` 改为 `leftright`，由 Penpot 原生 touched 记录保留现有 content/geometry override。三个 root 已是 Fill；没有新增 root、master、UUID、主题或节点业务模型。保存162→163；原2374个非目标对象、3个 copy root 和 fork30 的4个 Instance SearchInput/Hierarchy 内容完整保留。

实际 file=`b4a1e3ce-9e6a-81fe-8008-d69d6092d704` / page=`b4a1e3ce-9e6a-81fe-8008-d69d6092d705`；三板为 all-actions=`cbe04791-e3e8-4068-8570-ce848ad5032f`、math=`af647700-59ca-4b2a-b508-825290869480`、中文无结果=`f977dac8-a7c2-4fa7-9916-3eed79ebd040`。复用 Shared SearchInput component=`6ddc248b-bd35-80c5-8008-d6c8c3ce892e` / main=`6ddc248b-bd35-80c5-8008-d6c8c0f9c196`；sourceRef 和3个实例身份保留，未 detach。完整6对象与 override diff 见 [45设计保存](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-node-search-layout-45-design.json)。

通过真实 Penpot 宽度输入框做400→320→400诊断：服务器确认rev166时 root/background=304、text=268，rev167恢复384/384/348，icon宽16保持。首次固定延迟读取晚于 UI 保存，读回旧版本，原失败保留；随后按服务器实际宽度等待确认。恢复后全部2380对象与rev163逐对象比较一致，没有留窄板或截图坐标补丁。关闭旧context，分别在3个新context重开、核对400×400 SVG与实际文本，保存3张原viewer全图及3张板内图，另有原编辑画布和窄宽诊断，共8图已逐图打开复核。全部记录在 [45重开](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-node-search-layout-45-reopen.json) 与 [45逐图复核](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-node-search-layout-45-review.json)。自有浏览器exit0、auth不变、browser-search临时目录空，桌面输入0、手动清理0B。

这是可编辑设计的 consumer layout 修正。`search_input.zui` 和现有插件 field controls 已有对应布局原语；没有改 Shared/master/public source，没有冒称 Update/reset/全局样式传播或官方 asset-document/source export。三板仍是已标明的动作列表/查询/选中设计状态；原 `MaterialGraphNodeSearch`、prepared domain delta 与宿主真实输入/焦点/Document/history 接线分别保留其状态，不能从这三张图声称搜索、创建或 renderer 已接通。

唯一managed producer仍是44：`ue-material-graph-20261024-01` / `matgraph-spline-01a16d50-20261025-44`，queue18117/#403。13:19:28Z认证GET仍 building=true/result=null，490B原归档为 waiting/management_git_busy/Another process owns this operation；compileStarted、Rust发现/执行数及native/资源仍unknown。详见 [44原构建观察03](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-policy-44-build-observation-03.json)。不重投同operation，不复用旧r10或创建第二producer。45无Rust变更，无额外Cargo。官方导出→原ZUI→正常EXE/实际150%DPI/双端≤1DIP继续open，passed=false/accepted=false，完整goal保持active。


## 节点搜索接续46：当前可见动作与真实节点创建预检接通

本轮仅改材质插件3个自有Rust路径：`editor/src/search_creation.rs`、`editor/src/tests/search_creation.rs` 和 `editor/src/lib.rs` 的结构接线。新增 `plan_material_graph_node_search_creation`，接受现有 `MaterialGraphNodeSearch`、当前原 MaterialGraphAsset、稳定 action_id、typed candidate、可选原 autowire 和真实 ResourceKind resolver。只从当前打开菜单的可见结果取动作，不把旧行号或键盘选中项替代鼠标目标；关闭菜单、过滤后旧key、显示名/未知key在资源查询前拒绝。随后复用原 `plan_material_graph_node_creation`，保留节点种类、ID、位置、参数、纹理解析、连线类型/循环/占用输入替换的既有语义；返回原 borrowed plan 与精确旧edge，不克隆全图或造第二模型。

预检对 graph/menu 均无写入，成功也不自动关闭。原host仍须在真实 popup incarnation/source/document/revision 与当前 measured graph-local anchor 下 prepare，一次 retention permit/CAS/primary history 接受实际操作，才进入原激活/焦点流程。UE `SGraphEditorActionMenu.cpp:86–106` 区分 mouse/key 激活、有效action与实际 PerformAction；本预检不是输入事件、修订权限或事务实现，不宣称UE关闭/外点/Esc/焦点已还原。测试中的 DTO投影仅调用现有真实TOML codec/compiler，明确不作为生产 apply/undo 或 renderer 证据。

7新增回归定义覆盖：可见动作借用且菜单不消耗、鼠标Multiply与键盘Add不同、过滤后旧key/未知key、关闭菜单仍保留行缓存、错误kind/非法wire、纹理真实完整引用resolver及失败保持、占用roughness输入精确旧edge与中文ID/位置/TOML/真实MaterialAsset编译值。当前全插件 `material_graph_` 为115静态定义＝114普通+1原ignored；尚未Rust发现/编译/执行。3路径格式/diff/reverse patch为0，42个非目标源/依赖前后hash不变，静态guard3路径0error/0warning/0suppression。完整 [46源码修订](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-search-creation-46-source.json)、[验证](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-search-creation-46-validation.json)、[guard](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-search-creation-46-guard.json) 和 [精确增量补丁](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-search-creation-46-patch)。root仅新增module/test/export，之前全部代码字节由逆向重建hash核对保留。

46是44之后的透明源后继，不改44的identity/payload、108历史定义或原失败。唯一在途仍 `matgraph-spline-01a16d50-20261025-44 / queue18117 / #403`；115不能塞旧声明或借旧回执称执行。新的有限受管验证由本唯一producer在合法后继准备时声明当前源/原request及同一nested单材质包配方；此阶段未提交第二Cargo。Host生产调用、原Document/history/retention、官方导出/源ZUI、renderer/正常EXE/150%DPI继续open，完整goal active，passed=false/accepted=false。


46实际受理更新（2026-10-25T13:44:55Z）：原44 queue18117/#403已由认证API确认 building=false/result=FAILURE，490B原归档仍waiting/management_git_busy，compile/tests/native/resource未知，旧输入与历史不变。给原管理owner明确45秒接收配置发布空档13:39:22.990512Z–13:40:07.990512Z；只缓下一次本producer派发，期间0派发，已自动到期，未freeze源码/取消任务/清锁。原回执 [46有界发布空档](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-receiver-publication-gap-46.json)。

空档后按已公开精确配方发起唯一合法后继 `matgraph-search-create-01a16d50-20261025-46`，原request不变，18 owned+25 dependency均按当前源hash声明，identity SHA289ab6aba07cba5a26449e793c6aa9309f25d7cf6033bc9783538ee32baa007c。client39060/orchestrator30480均exit0，双次真实capture后POST实际queue18854→zircon-management#407，inputDigest c2eedec6d3205e361781ff2682906750dceb0c34199ab799279d6b206dfa1be4。认证API #407当前 building=true/result=null，497B原归档SHA54325fe6f77a71157729ba98c1ee03769b113a5d84e33b9a79a047c397ffc673为waiting/management_git_busy，operationKey cefeef6f07c852e677296ad0d6985f53b891b2f49d74d0878429e79b7a2615d2。compileStarted/实际Rust发现执行/native/资源未知；115只属静态定义。见 [46 managed](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-search-creation-46-managed.json)、[client实际收件](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-search-creation-46-delivery.json) 与 [原queue/build观察](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-search-creation-46-build-observation-01.json)。正常prepare用于取得实际准入，不事先要求完整writer/resource回执；不以所有旧unknown当新后继全局前置，也不因API受理释放旧锁或声称Rust/产品通过。未重复派发同46/44/r10。


## 节点图接续47：真实 ShaderAsset 契约编译与宿主接线边界（2026-10-25）

本轮仅改材质 plugin 私有三个 Rust 路径，新增 `compile_material_graph_with_shader_contract`：沿原 CPU 图编译，核对所供 ShaderAsset 的源 locator、原导入诊断及 Runtime `shader_contract_diagnostics`。图 node/pin、Shader 导入消息及 Runtime source/path 保持不同的真实定位；不伪造图节点跳转。有效结果保持原 MaterialAsset/PBR descriptor，不变更原图/Shader、不写文件、不应用 renderer。六项新回归覆盖正常 PBR/原源不变、图错误优先和原定位、不同 Shader 源、Compute 非 Surface、多个 required property/texture 的真实路径，以及中文导入诊断保留。当前 filter 静态定义121＝120普通＋1旧 ignored；不是发现/执行数。

三路径格式、精确反向补丁、guard 0errors/0warnings/0suppressed，49个 scoped 保护输入未变。第一次误用2024格式 edition 和两次 lib.rs 截断写失败已保留；按实际 workspace2021修正检查，root以现有可写句柄增量增长，精确还原 source46 哈希，不改变任何锁/属性/进程。没有改 public Graph/Runtime/manifest/lock/原ZUI、Shared 或 fork30四个Instance搜索copy。

实际宿主核对：Material注册六个操作描述但本文件没有操作 factory；dist空 command_manifest/invoke_command=None，所查 CoreEditContext未提供Material作者源入口。SaveCtx并非公共写盘API，现有源write authority与receipt为Editor私有。Parent需把本族域能力接到原 per-view document/source/revision/lease 与 ResourceManager active version，再走原prepared事务/history及Toolkit保存。Shader URI相等本身不证明UUID/资源当前版本/ABA/ready。保留原ZMaterialDocument editor与presence，不从展平MaterialAsset重建作者文档。详见[精确宿主合同](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-host-integration-47-handoff.md)、[源码47](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-shader-contract-47-source.json)及[检查](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-shader-contract-47-guard.json)。

原 managed46 request不变，operation `matgraph-search-create-01a16d50-20261025-46` / queue18854 / #407；14:01Z仍building/resultnull，497B原归档waiting/management_git_busy。当前47是透明源后继，不塞旧115项payload、不新增投递，编译/真实执行/native/resource未知。接续45的Penpot可编辑/保存重开证据仍只覆盖三张Search板；本轮无新浏览器或桌面操作，正式导出/真实Apply-save/renderer/normalEXE/实际150%DPI/双端1DIP全部open，passed/accepted=false。旧#352/#353/#370/#403及全部原失败保持；发布空档46已到期，不冻结源码或无限等待。

## 材质 Palette48：原生联动弹层、三尺寸重开与逐图复核

本轮在本域原 file `b4a1e3ce-9e6a-81fe-8008-d69d6092d704` / page `b4a1e3ce-9e6a-81fe-8008-d69d6092d705` 继续真实可编辑设计。新增 Palette 板 `4cc0e0d4-ef9f-4ff0-98bc-88a9f4c34be8`，使用原 Search31 全动作板的实际 linked SearchInput 与六个 TreeRow copy，保留 Shared source/component/shapeRef；未新增 master。布局为 280×380 flex 纵向容器，六项仅对应已注册的 Output、Texture Sample、Scalar Parameter、Vector Parameter、Add、Multiply。UE 依据为 `SMaterialPalette.cpp:112` 的 Palette/ActionMenu 与 `MaterialEditor.cpp:576` 的独立 Palette 页签；UE 的 Functions/分类筛选/拖放创建/快捷键仍未接入，不能以六行设计代替这些语义。

保存从 rev167/2380 对象，经过 rev168，最终 rev170/2514 对象，原五个业务 main 不变。新增134个真实对象、7个 linked copies；既有2376个非授权对象逐项一致，原ID无丢失。只修改本域三个既有窗口板的 Palette 侧栏交互与侧栏 paint；未触及 fork30 的 Instance/Hierarchy 或四个 SearchInput copy。Shared 于15:04:35Z独立只读实测 rev5/isShared=true、4主件/34颜色/3字体样式；本轮仅复用真实库身份，没有发布/修改 Shared，也没有证明样式继承或跨文件 reset。原 catalog/tokens/Starship/根 ZUI 的当前字节指纹保存在48回执，根仍为 `5856c88655aafcae3b8f8af2d4ed7786ed89e61551610324faa51ce767af0c12`。

三个既有窗口板的侧栏实际使用 Penpot native toggle-overlay、top-right 与各自 positionRelativeTo，启用 closeClickOutside。首次手工定位在嵌套 frame 中重复叠加 parent offset，1280 板实际弹层越界；保留失败截图、真实 DOM bounds 和修订记录。按 Penpot 实际 `interactions.cljc` 与 `viewer/shapes.cljs` 的锚点规则改为 native top-right，不以截图偏移补偿。首次检查脚本精确 SVG 标题/复用旧 SPA index 也失败；这些失败保留，最终以独立新 context 的真实 native overlay 容器、尺寸与交互结果验证。

| 逻辑窗口 | 原窗口 board | 弹层实际边界（浏览器逻辑坐标） | 新 context 结果 |
| --- | --- | --- | --- |
| 1280×800 | `9fce0704-72a3-4f09-84e2-00e0d9efab78` | x1158/y253/280×380 | 完整位于 client，侧栏打开、外点关闭 |
| 900×620 | `46039af0-ecf6-4d12-a29c-60c09d698950` | x968/y343/280×380 | 完整位于 client，侧栏打开、外点关闭 |
| 640×520 | `722ca546-223f-4c7f-9e52-a37348a0250e` | x838/y393/280×380 | 完整位于 client，侧栏打开、外点关闭 |

8张最终 PNG 已实际逐图打开复核；六项文字、搜索 copy、浮层边界均可见。仍明确记录旧 Details 页签在900/640截断、Nodes Overview 在1280也被裁切；原“Compile material to preview”仍是占位文字，Not compiled 仍是未编译状态。青色侧栏提示属于 Penpot interaction hint，不当作产品焦点证据。弹层仅证明设计端打开/外点关闭；Esc、IME、焦点返回、搜索创建命令、真实 renderer、正式 codec/export、原 ZUI 回写、正常 EXE/实际150%DPI/双端≤1DIP 全部 open。资源/捕获门未满足，passed=false/accepted=false，完整 goal 保持 active。

[实际保存与身份](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-palette-48-design.json)、[修订与首轮失败](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-palette-48-correction.json)、[三新context交互](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-palette-48-reopen-r3.json)、[逐图与2376对象核对](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-palette-48-review.json)、[Shared/主题/源指纹](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-palette-48-current-fingerprints.json) 均保留，原错误回执不覆写。

托管验证与设计分开记录：原 source46 operation `matgraph-search-create-01a16d50-20261025-46` / queue18854 / #407 在14:30Z实际认证 API 已 FAILURE，548B归档仍 waiting/checkout_source_busy，不能据此补造 native EOF 或0执行。新 source47 已透明声明当前20 owned+25依赖，原 request 下只正式提交一次 `matgraph-shader-contract-01a16d50-20261025-47`，queue19353/#410，inputDigest `957747e9f5e2d0a314a4e600f24b8e22669ef89b2631c1a110452065f6292438`。限定 nested manifest/单 material 包/defaults off/features[]/locked/lib/test/jobs1/material_graph_/threads1/E根配方不变。15:03:06Z认证 API 为 building=true/result=null，499B归档SHA `e4c1f5bb67f71bf9ad5d2502132b4cf4e2b786ee0cf7ccb52c17df883f2d5024` 为 waiting/management_git_busy；compileStarted/真实Rust发现和执行/native/resource仍 unknown。121仅为120普通+1原ignored静态定义，不当作运行数。见[当前47观察02](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-shader-contract-47-build-observation-02.json)。

按实际407终态向原管理owner提供的接收配置发布空档为14:32:16.885361Z至14:33:01.885361Z，45秒、只暂缓本producer下一次管理派发，已自动到期；47派发在到期后，未停止在途任务或源码/设计/文档。空档不代替发布、source freeze或资源准入。两个本域客户端与自有浏览器均已退出，browser显式close、原认证文件未变，48临时目录实测为空；释放0B，保留全部失败/PNG/回执。没有重放r10/#370或其它终结operation，没有改服务、gate、State、锁、公共源或实际桌面。

## 材质接续49：原391终态与锁路径勘误，真实节点测量边界适配

已实际读取原 Jenkins owner 的 `lock37-followup-r03-result.json`（25751B/SHA `e145fac634d24f4c5b41e65ab027b055a61920660ccc49fbbda0f35341447377`）及短说明（SHA `724fa59ead6ed4f83e748459d0905adcedee2480d298f50ac667ad3f21f5edbb`）。原 request/operation/queue16229/#391/key `9adab3d26a57bd0f49442ed4926e5aa57d5f677235c63511a176b2679f1de8bb` 不变；15:03:56Z认证 API 为 FAILURE，timestamp+duration对应10:18:28.090Z结束，State仍v480/waiting、492B原archive SHA `649b9d103fe9cb53a6a55a54f7cf923285c42955968691d6932daecc78b80840`。240响应中232 mutex busy、8 checkout_source_busy，后者指向原r19 reader；这是重试耗尽证据，不能改写为budget拒绝。

锁路径勘误单独保留：原13:20 RestartManager读的是 `.jenkins/state/management/management-git.lock`，实际运行路径为 `.jenkins/state/locks/management-git.lock`，撤回由前者推断实际锁无holder的结论。后者15:03:53Z报告PID10176/birth134374141171019850（15:01:57.101985Z），15:04:24精确CIM匹配同birth/parent35880；命令/路径不可读，没有归属chat或operation。10:12 PID37752只作历史观察，后续无row不证明EOF。原r19 Cargo exit101/Job0/双EOF不关闭reader unknown/pinsReleased=false、active pool/reservation或负向source连续性；原server failed-checkout-read/native/resource CAS仍待权威收口。原source37/84定义只归10:15历史，不签当前源码。没有重投391或手改State/锁；见[本域49消费记录](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-autowire34-lock37-consumption-49.json)。

本轮继续实际域源码，范围仅 `editor/src/graph_presentation.rs`、`editor/src/tests/graph_bounds.rs` 与 `editor/src/lib.rs` 的测试接线。已有 `MaterialGraphPresentation` 新增 `graph_bounds(measure_size)`：逐一借用真实源节点，位置沿原author数据，尺寸由宿主同代际retained几何提供；缺少任一测量原样返回错误，不跳过节点或代入模板尺寸；空图返回None，未编译/不完整图仍可为修复定位计算边界。联合范围O(nodes)/常量额外存储，没有clone全图或节点view；向外取整跨度，拒绝非有限/零/负/坐标精度塌缩及联合溢出。原node_view复用相同节点边界检查，语义保持。

消费路径为当前宿主的同源/同revision/同layout generation测量 → 本域graph_bounds → 原 `GraphCanvasState::fit_graph_bounds`。没有新相机/store/clock，未改shared Graph/model/commands/retention/permit，也不写节点坐标、选择或history；DPI仍仅宿主转换一次。UE实际参考为 `SNodePanel.cpp:1821` 的真实节点位置+DesiredSize联合、`:1953` ZoomToFit与 `SGraphEditorImpl.cpp:582` 转发；本切片只支持全图边界，不声称UE选择子集、deferred zoom动画、输入/焦点已接通。7项有意义回归定义覆盖实际借用节点、空图、缺失retained测量、非法尺寸/位置/联合溢出、向外舍入、最小zoom不能全显、895/629/447逻辑viewport的既有Canvas/选择和坐标逆变换；这些数值为单元测试输入，不是原生窗口测量。

最终3路径rustfmt2021、增量reverse与diff-check均exit0，最终guard-r2为0 errors/0 warnings/0 suppressed。审阅中已修正有限MAX宽度误列非法、数值边缘断言，以及测试误引用未公开的GRAPH_MIN_ZOOM；保留首版patch/guard/validation，未改shared出口来迁就测试。当前128静态定义=127普通+1原ignored，实际Rust发现/执行仍null。source49是透明后继：相对原source47改lib/presentation并新增测试，未篡改原47的121定义payload/identity；当前没有新operation或Cargo提交，在途queue19353/#410及全部旧失败仍按原链。见[源变更](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-frame-bounds-49.patch)、[最终静态验证](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-frame-bounds-49-validation-r2.json)、[最终guard](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-frame-bounds-49-guard-r2.json)。

本轮未写Penpot/Shared/主题/catalog/原ZUI/Runtime/锁/服务/gates，没有desktop操作或新增producer。复用Palette48的rev170设计证据，不将源码边界方法冒充Frame All按钮、paint/hit/focus、正式codec、实际renderer或正常EXE/150%DPI/双端1DIP完成。goal active、passed=false/accepted=false；释放0B，原391/source37/r19及首版静态回执全部保留。

15:19:28.789346Z 补充原47终态：认证 API 已读 queue19353/#410 building=false/result=FAILURE，duration2573483ms；499B归档仍SHA e4c1f5bb67f71bf9ad5d2502132b4cf4e2b786ee0cf7ccb52c17df883f2d5024、waiting/management_git_busy。该pipeline终态与原生/reader/resource收口分开；compileStarted与真实Rust发现/执行继续unknown。保留15:03 building观察，未重投47或创建49 operation。见[47终态观察03](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-shader-contract-47-build-observation-03.json)。
