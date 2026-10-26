# AI 窗口的设计、数据和验收合同

本记录接续 [窗口盘点](ai-windows.md)。2026-10-25 监督恢复后继续原文件：最近一次实读（监督恢复52）rev47/863对象；十三业务状态/尺寸板、四业务main、58本域业务copy和七个Shared copy保留。Shared仍用原file/page，原四main、34颜色和3排版，最近读取rev5。640板已补回原本缺失的“No debug object selected”提示；这只修复设计中的空状态显示，不代表调试选择器、overflow或窄窗图节点裁切已完成。保存、原生重排、关闭/新上下文重开及逐图复核见 [设计恢复52](ai-penpot-design-r01.md#监督恢复52原640板空状态与真实保存重开)。

实际入口：[06 AI Editors](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60dbec5a&page-id=b4a1e3ce-9e6a-81fe-8008-d69d60dbec5b)。沿 Parent 原分配及只读认证，不创建账户或复制认证文件。本域候选21在BT菜单上接入黑板创建/改名typed业务候选和引用未知资格；26份作者pin、61项静态测试定义（57 authoring、4 gameplay），仍无Cargo执行回执。监督恢复52的四次输入续接及一次精确QueryOnly原输出保留，最终返回authorization pending；本轮新源没有塞回该原输入。继续独立设计和实现，不再把等待公共完美交付作为 blocked 条件；官方导出、真实文档/history/save、正常EXE/150%DPI和双端验收仍待完成。

## 区域与稳定身份

Behavior Tree 目标依据 `BehaviorTreeEditorModes.cpp:27` 的 `Standalone_BehaviorTree_Layout_v2`：图文档占水平 0.7，右 Dock 占 0.3；右上 Details/Search 占垂直 0.6，右下 Blackboard 占 0.4。Search 初始关闭，Details 与 Blackboard 初始打开，图文档由 Toolkit 恢复。UE 没有这个默认布局中的永久左侧节点 palette 栏。

Blackboard 目标依据同文件 `Standalone_BlackboardEditor_Layout_v2`：Blackboard Editor 与 Blackboard Details 两个水平 stack，没有另设尺寸系数。模式切换保留真实资产、选中身份、图视图位置与正在编辑对象；不能重建另一份数据或丢弃 dirty 状态。

EQS 的 graph/details 是 0.7/0.3，Profiler 与 Details 同 stack。StateTree 的默认布局由 `Standalone_StateTree_Layout_v7` 定义：左 AssetDetails 0.2、中间 0.5、右 SelectionDetails 0.3；中间上方 state view 0.75、下方 compiler/search/debugger/bindings stack 0.25。待定窗口先保留完整目标，不能以占位面板填入产品。

外部框架复用 `UiHostWindow`、`DocumentHost`、`MenuBar`、`StatusBar` 及 Parent 的 Dock/tab/菜单合同。本域只提供内容组合与真实数据。普通区域使用水平/垂直 flow、组件 slot 和明确滚动归属；图才由 graph owner 管理坐标、pan、zoom、clip 与端口命中。文本起始对齐、数值列尾对齐，图标与文本是一组控件内容，不用空格拼列或逐截图偏移。

需要保留的现有身份：

| 源资产 | 身份 | 处理 |
| --- | --- | --- |
| `zircon_plugins/ai/editor/behavior_tree.zui` | asset/view/template `ai.behavior_tree` | 原始 owner，等待 Penpot 后修改，不建替代预览资产 |
| 同上 | `AiBehaviorTreeRoot`、`AiBehaviorTreeGraphCanvas`、`AiBehaviorTreeInspector`、`AiBehaviorTreeBlackboard`、`AiBehaviorTreePalette` | 控件稳定 ID 保留；palette 的默认位置需按 UE 上下文创建交互调整，迁移依赖必须记录 |
| `zircon_plugins/ai/editor/perception_debug.zui` | asset/view/template `ai.perception.debug`、`AiPerceptionDebugRoot`、`AiPerceptionDebugAgents` | 保留身份，连接真实镜像实体数据 |
| `zircon_plugins/ai/editor/src/extension_ids.rs` | `ai.behavior_tree.palette`、`ai.behavior_tree.import/open/validate/compile` | 复用已注册命令与节点目录；descriptor 不等于 handler |
| `zircon_editor/assets/ui/editor/components/catalog.toml` | WorkbenchPanelHeader、WorkbenchPropertyEditorRow、WorkbenchListRow、WorkbenchTreeRow、WorkbenchField、WorkbenchDropdown、WorkbenchButton、WorkbenchIconButton 等 | 使用 catalog 的完整 component ID/document/binding namespace/slots；不 detached 重画 |
| `editor_tokens.zui`、`unreal/starship.zui` | Parent 管理的主题闭包 | 仅继承已批准语义值；缺少 BT 样式先向 Parent 提案 |

尺寸变小时保留同一数据、模式和操作可达性。图负责 pan/zoom；长键列表、详情和搜索各有自己的单一滚动 owner。Dock 的 min-size、overflow、隐藏/恢复须复用共享实现并在 640 宽真实测量；当前没有证据可指定一个“自动改版”或隐藏目标面板来冒充 UE 对齐。

下表使用实际 catalog ID。document 路径相对 `res://ui/editor/`；除 MenuBar 外，所列 binding namespace 与 component ID 相同。AI 文件中的 4 个组件仅为业务内容组合，下表的公共组件实例和 shared style UUID/version 仍未交接，不能用本地业务 UUID 顶替 catalog 身份。

| component ID | document owner | 本窗口使用与 slot 限制 |
| --- | --- | --- |
| WorkbenchPanelHeader | `workbench/composites/chrome/panel_header.zui` | title 必填，接 WorkbenchCaption/SectionTitle；actions 只接 catalog 允许的 Button/Caption/Chip/Dropdown/IconButton/Toggle |
| WorkbenchPropertyEditorRow | `workbench/inputs/property_editor_row.zui` | value 必填，接 Checkbox/Dropdown/Field/NumberField/RangeSlider/Slider/Toggle；不能直接塞自建复合 Vec3 节点绕过 accepts |
| WorkbenchListRow / WorkbenchTreeRow | `components/workbench/primitives/data/list_row.zui` / `tree_row.zui` | 节点目录、黑板键/搜索行及层级状态的候选；实际 row 内容/状态能力仍需对应 owner 证据 |
| WorkbenchField / WorkbenchButton | `components/workbench/primitives/inputs/field.zui` / `button.zui` | 文本编辑和命令入口，使用原事件与焦点合同 |
| WorkbenchNumberField / WorkbenchCheckbox / WorkbenchDropdown / WorkbenchIconButton | `workbench/primitives/inputs/number_field.zui` / `checkbox.zui` / `dropdown.zui` / `icon_button.zui` | 数值、required/bool、类型选择、菜单触发；i64/u64 精度与 Vec3 原子提交不能由组件名称推定支持 |
| WorkbenchAxisValueField | `workbench/primitives/chrome/axis_value_field.zui` | 三分量编辑的现有候选；需要 Parent 确认可用组合及 PropertyEditorRow 的 slot 接入，AI 不另造基础输入组件 |
| WorkbenchContextMenu | `workbench/primitives/feedback/context_menu.zui` | 节点/键/pin 菜单；业务提供真实动作及守卫，弹层/焦点返回沿共享 owner |
| MenuBar | `unreal/components/menus.zui` | binding namespace 是 WorkbenchMenuBar；actions slot 仅接 WorkbenchIconButton，不能把 component ID 和 namespace 混用 |
| DocumentHost | `unreal/editor_shell.zui` | tabs 接 DocumentTabs，content 必填且接 PaneSurface；AI 通过内容宿主接入，不把任意根节点当允许的 slot 类型 |

## 数据、执行与事务归属

| 所有者 | 权威数据/职责 | 禁止混淆 |
| --- | --- | --- |
| AI runtime plugin | 节点目录、编译、真实世界 tick、blackboard/perception/effects | UI 不推进计时器生成假 Running 或 fake 节点结果 |
| AI editor plugin | 资产文档会话、选中、节点/键编辑、视图投影、资产命令处理 | 不能让共享 chrome 或 Penpot metadata 成为 AI 数据权威 |
| Editor 资产/事务系统 | 资产 URI、dirty、保存失败、撤销/重做与资产跳转 | 视觉属性修改不等于资产已保存；失败不得清 dirty |
| 既有 runtime-event 消费者 | play session、sequence、world、entity 的真实调试帧 | 多世界相同 entity ID 不合并，结束会话/删除实体必须清除旧高亮和值 |
| Parent 管理的图组件 | 实际节点/端口/边绘制、输入/焦点、坐标与证据 | UI 连线必须来自已保存拓扑并反映执行顺序 |

Parent 最新 AI 交接已决定首阶段 Decorator 保留真实包装拓扑及 UE 类别样式，并记录与 UE 宿主附加子节点的外观差异。Zircon 当前校验为恰好一个子节点的包装器；后续高保真方案可考虑可逆地把包装链投影成宿主附加区域，仍把每个包装节点及顺序写回原 ID。若不能证明重排、abort 与 subtree 的等价性，不能采用后一方案，也不能合并 wrapper/host 身份。

Service 不能套用这个映射：Zircon 校验要求它零子节点，现有 `evaluate_service` 读取 `service_result` 返回状态；注册名 `update_blackboard_distance` 没有在该路径计算距离或写回黑板。UE 在宿主活动期间执行的附着式 Service 因而仍缺后端，需明确附着、周期、激活/退出、数据与真实写入反馈合同，不能通过装饰性区域或可配置的 Succeeded/Running 冒充完成。

每个编辑操作要在真实文档上验证输入、产生一个可撤销事务、保持未受影响节点 ID、更新详情与 dirty。拒绝操作保留完整文档、选择和历史。节点创建先能保存未连接的创作态，但执行/编译必须报告 unreachable 等真实错误；不能为了让创建通过而放宽 runtime 编译器。编译失败不伪造运行结果。保存须通过现有资产写入入口，重新打开同一资产核对原 ID、参数、拓扑和键类型；具体 handler/存储接口待受管工作树与设计身份就绪后实现。

## 逐控件数据绑定设计

以下是等待实现的投影字段和操作合同，**不是已经注册的 binding namespace、事件或 payload schema**。实际导出只能使用现有 ZUI/组件 owner 支持的绑定语法；新增字段须在 AI editor 文档模型中实现后再声明可用。Penpot 上的样例数据须标为设计夹具，产品加载时由同一投影读真实资产或镜像。

| 内容/保留控件身份 | 权威来源与行身份 | 编辑、空值和状态约束 |
| --- | --- | --- |
| 文档标题、dirty、模式 | 资产 URI + 文档会话；`AiBehaviorTreeDescriptor.id/display_name` 只提供树的业务身份 | URI、文档 revision、dirty、保存结果不在 runtime DTO 中，需接现有资产/事务 owner；不能把 display_name 当文件路径 |
| `AiBehaviorTreePalette` | `standard_node_catalog().descriptors()`；行身份是 implementation ID | category 只用于分组，创建 kind 需下表的语义映射；当前注册没有 inputs/outputs，也没有参数 schema，不可推定能生成完整节点 |
| `AiBehaviorTreeGraphCanvas` | `nodes[].id/kind/implementation/display_name/children/parameters/abort_policy`；节点身份是原 ID | 边身份由实际 parent/child/顺序导出；位置、pan/zoom、折叠和多选不在 DTO 中，需接编辑 metadata owner，不能塞入 runtime 参数改变语义 |
| `AiBehaviorTreeInspector` | 文档 revision + 选中节点 ID；参数以 `parameters[].key` 精确寻址 | 不按列表 index 更新节点；多选只显示共同且可兼容属性；未知 implementation/参数保留原数据并报告目录不匹配，不能在重存时丢弃 |
| `AiBehaviorTreeBlackboard` 创作态 | 当前 schema 的 `keys[]`；身份是精确键名 | schema 未绑定时显示真实未绑定状态；当前树 DTO 没有 schema URI/ID 字段，不能凭两个 display_name 相同自动关联 |
| Blackboard Details | `AiBlackboardKeyDescriptor.key/value_type/required` | 只编辑真实三字段；运行时值不属于 schema 默认值；键改名不能靠改 `row_identity_field` 完成引用迁移 |
| BT Blackboard 运行态 | 选定 session/world/entity 的 `AiBehaviorDebugFrame.blackboard` | 只读；无帧、缺少 optional key、值为 false/0/空串、类型错误分别呈现，不把缺值格式化为 0；schema 的 required 检查仍由 runtime 执行 |
| 调试实例与状态 | `AiPieMirror.agents_in_world`、frame.behavior_tree、report.status/active_node/diagnostic | 用 session + world + entity 选择；`agent(entity)` 在多世界歧义时返回 None，不能用于跨世界列表选择 |
| 感知实体/刺激详情 | 同一 frame 的 perception/perception_debug | 无 spatial debug 时保留真实缺失；overlay 不能拿另一个 entity 的 position 补齐；停止/切世界清选择和 overlay |
| Search | 当前文档节点 ID、名称、implementation、真实参数的可读投影 | 搜索结果携带文档 revision 和原节点 ID；中文/长文本不改变 ID；结果过期先重算，不能按旧 index 跳错节点 |

Blackboard 的 schema 目前没有独立键 UUID。改名后列表选择要由同一事务显式迁移，Undo 恢复原名、原引用和原选择；不能声称键拥有尚不存在的稳定 UUID。`BlackboardLayout` 的 dense slot/generation 是编译结果，不能当作跨编辑版本的持久行身份。当前 schema 验证按原字符串精确判重；UE 使用 FName，大小写身份规则不能直接照搬。是否收紧产品命名规则须另行提案，不能静默 normalize 已有资产。

### 黑板类型与 UE 字段

参考实际 `AIModule/Classes/BehaviorTree/Blackboard/BlackboardKeyType_*.h`、`BlackboardData.h` 及 `BlackboardDataDetails.cpp`。当前参考树还包含 Struct、Name、Rotator 等类型，不能把“UE 常见六类”当完整类型菜单；NativeEnum 标有 HideDropDown/NotEditInlineNew，也不能因头文件存在就加入 New Key。

| Zircon 实际类型 | 控件内容/输入 | UE 关系与边界 |
| --- | --- | --- |
| bool | 勾选框；显示 true/false 与 missing | 对应 Bool 语义；不能用 unchecked 表示缺值 |
| integer / i64 | 整数 Field，保留 i64 精度 | UE Int 数据为 int32；不能套用其范围截断现有 Zircon 值 |
| scalar / Real | 数值 Field；仅提交有限值；保留项目精度/单位 | 对应 Float 的标量用途；不凭 UE float 推定 Zircon Real 的存储精度 |
| string | 文本 Field，支持中文、空串和长文本 | 对应 String；不是 FName，不隐式转为 Name 类型 |
| vec3 | 现有三分量数值编辑器；三个有限值一起提交 | 对应 Vector 用途；按 Zircon 坐标/单位解释，不能拿 Rotator 标签冒充旋转语义 |
| entity / u64 | 真实 world/entity 选择器，保留 u64 精度与失效引用状态 | 只表示实体 ID，不等于 UE Object/Class 的 UObject/BaseClass 或资产选择器 |

Parent/ParentKeys、Entry Category、Entry Description、Instance Synced、各类型 DefaultValue/BaseClass/Enum/Struct 类型信息都不在当前 schema DTO。它们保留为 UE 目标差异；生产产品不能提供会丢数据的可编辑字段。`required` 是 Zircon 的输入完整性约束，不能改名为 Instance Synced。若需要 Category/Description 等编辑 metadata，应复用资产 metadata owner 并补保存/重开合同；本轮没有自行扩展 runtime 格式。

UE New Key 由类型选择后生成唯一名、关闭菜单、选择新项并立即请求 rename；当前源码的 picker 为 200×240，分类菜单最大高度 400。这些是对应弹层的目标逻辑尺寸，不是整个面板的固定高度。重命名/删除在黑板模式、非调试且非继承项时才可用；copy/duplicate 可读取继承项。Zircon 目前无继承后端，不能生成假 ParentKeys。选键菜单参考 `BlackboardSelectorDetails.cpp` 按允许类型过滤，并在 owner/schema 改变时刷新；“None”只有参数语义允许时才可选。

### 黑板引用、事务与保存边界

本节是 AI 域的待实施合同，不是现成的引用索引或跨文档事务。已核对 `manager/blackboard.rs`、`manager/validation.rs`、`manager/validation/runtime_inputs.rs`、`blackboard/observer.rs`、`behavior_tree/executor/{condition.rs,effects.rs}` 和 Runtime 的 `ai/{manager.rs,tick.rs}`。注册 schema 会拒绝重复业务 ID；当前 `AiManager` 只有注册/读取 schema 和设置代理值，没有 schema replace/unregister 接口。`AiAgentTickRequest.blackboard_schema` 是已注册句柄，不是树资产保存的 schema 引用。修改编辑资产不能声称已更新活动代理布局。

引用须以已核验的 schema 资产关联、树 URI/内容 hash/revision、node.id、parameter.key 和旧值组成精确目标。内置 owner 验证允许 Decorator 的 `blackboard_key`，以及 Task + `implementation=set_blackboard` 的同名参数；未知 provider 的参数只能由其已声明语义识别。`equals_string`、节点标题、事件 payload、脚本文本或另一个 schema 的同名键不属于自动改名范围。当前缺少树→schema 持久关联和参数语义索引，UI 应报告无法证明的引用闭包，不能把字符串搜索结果当作完整引用。

| 合同 ID / 操作 | 提交与 UI 结果 | 验证及未完成依赖 |
| --- | --- | --- |
| AI-BB01 New Key / Rename 输入 | 六个真实类型；生成名称按当前精确字符串集合判重，选择新项后进入原 Field 的 rename/IME。Esc 不提交；Enter/失焦按共享编辑器合同提交一次；空白名或真实冲突保留输入并关联具体诊断 | schema 校验只要求非空和类型有效，没有通用 256 字节上限；`set_blackboard` 参数另有 256 UTF-8 字节上限。改名会更新该引用时同时校验，中文长度按字节计算，不静默截断或更改既有命名规则 |
| AI-BB02 Rename 引用闭包 | 先取得同一 schema 的可写依赖、各 revision/hash 和真实语义引用；构造 schema 键+引用值+选中迁移的一次可撤销编辑，保留节点 ID 和无关字符串 | 未绑定 schema、未知 provider、只读/缺失依赖或 stale revision 时拒绝，并用可跳转行列明具体依赖；需要资产/事务 owner 提供真正跨文档准入与原子提交，不能循环调用单文档命令冒充原子操作 |
| AI-BB03 Change Type / Required | 类型改变先检查相关 equals/comparison/value 参数和真实使用方；required 只修改输入完整性约束，不生成运行值或默认值；失败保留原类型/值/选择 | 当前 manager 编译参数检查并不自动证明所有比较与 schema 类型兼容；observer resolve 对启用 abort 的缺键会报错，typed effect 还会校验活动 dense store。不能靠编译成功就关闭类型差异 |
| AI-BB04 Delete / Duplicate | 删除有引用的键先报告真实使用点；无可证明的迁移/删除事务则拒绝，不静默把参数改成 None。复制 schema 项生成独立键名，复制保留真实类型/required，不能复制代理当前值当默认值 | 被删除键的选择回到合法相邻项或列表；Undo 恢复键、引用与选择。外部依赖未知不等于零引用；当前引用索引与键事务均未实现 |
| AI-BB05 Dirty / Save / Close | 沿 `capture_save_token` 固定 history lineage/transaction/generation，在实际写入完成后调用 `mark_saved_if_unchanged`。保存期间继续编辑则保持 dirty；失败、取消、旧 ticket 或别的 history 不能清标记 | 复用 `SaveDirtyViewsRequest` / `SaveDirtyViewsJobAdapter` 的准入和逐文档完成回执。它们不是 AI 序列化器，也不保证多文件保存原子性；引用改名后的持久一致性与中断恢复需要资产 owner 合同 |
| AI-BB06 运行值与编辑锁 | 运行值只读，来自匹配 session/world/entity 的真实帧；schema 编辑锁沿真实调试 context，停止后由 owner 解除并清旧值。保存 schema 不调用 `set_blackboard_entries` 伪造实时生效 | 运行 schema 的业务 ID/句柄/布局代际与资产版本尚无完整桥接；`register_schema` 重复 ID 被拒绝，不能通过重新注册同 ID 伪造热更新 |
| AI-BB07 引用跳转与焦点 | 引用行携带 URI/node/parameter/revision，打开成功后选择真实对象；返回恢复源键/列表滚动/焦点。弹层取消回到原 Field/行，目标已删除时回合法列表 | 复用 Parent 的资产导航/焦点栈；未解析 URI 或 stale 行显示真实失败并重新查询。实际中文/长文本、首尾滚动、Undo/Redo、保存期间修改和只读外部引用均待运行验证 |

黑板证据要额外记录 schema 资产与运行 schema 句柄的不同身份、引用闭包版本、原子编辑结果、保存 ticket/token 对应版本、重开后的真实源 hash，以及每个引用跳转目标。这里只规定所需记录，尚无这些运行回执。

### 18 个内置 implementation 的创建与 Details

源码为 `zircon_plugins/ai/runtime/src/behavior_tree/nodes/{composite,decorator,service,task}.rs`、`manager/parameters.rs`、`manager/validation.rs`、`validation/integration.rs`、`behavior_tree/executor.rs`。下表是创建映射设计，不修改目录或声称每条后端已验证。

| implementation | 创建 kind | Details/真实执行边界 |
| --- | --- | --- |
| selector | Selector | `children` 有序；顺序是执行优先级，重排必须进入同一事务 |
| sequence | Sequence | `children` 有序；不能以屏幕排序替代保存顺序 |
| parallel | Parallel | `success_policy`、`failure_policy` 为 string all/any；不等同 UE SimpleParallel 的主任务/后台分支语义 |
| random_selector | Selector（保守创建映射） | 保留 implementation；权重按 `weight.<child-id>` 优先、`weight_<position>` 兼容回退、默认 1；有限且非负。旧 positional 权重重排会改变语义，不能偷偷改成 ID 权重 |
| blackboard_condition | Decorator | `blackboard_key`、exists/invert、typed equals 与 integer/scalar 比较；类型筛选来自 schema。未知键/比较类型错误必须有真实诊断 |
| cooldown | Decorator | `cooldown_seconds`，兼容读取 duration_seconds；非负 scalar。显示参数缺省不等于往源文件写入默认值 |
| time_limit | Decorator | `time_limit_seconds`，兼容 duration_seconds；非负 scalar；缺省为执行器的无限上限，不把 infinity 写入只接受有限值的参数 |
| loop | Decorator | count 为正 integer，infinite 为 bool；runtime count 转换边界需补覆盖，UI 不擅自截断大 i64 |
| inverter | Decorator | 保留单子节点包装语义，无假条件参数 |
| force_result | Decorator | forced_result 由实际 parser 解释；这是显式改变节点结果的语义，不能当调试器“当前运行状态”控件 |
| update_blackboard_distance | Service | 仅注册名与 service_result 返回路径，距离/写黑板/周期仍缺失；不提供装作已生效的 Interval/Target/Output Key 编辑操作 |
| wait | Task | duration_seconds 为非负 scalar；缺省会持续 Running。已有 result 参数会覆盖计时路径，不能在真实等待验收资产里配置 result=succeeded 走捷径 |
| move_to | Task | target 为 vec3；需真实 Navigation 可用与目标匹配反馈，缺导航返回 Blocked/诊断；不把 result 参数作为到达证明 |
| play_animation | Task | parameter 或 trigger 恰选一个非空 string；parameter 需要 bool/i32 integer/scalar/vec3 的 value，trigger 不允许 value；实际播放依赖宿主与资源 |
| set_blackboard | Task | blackboard_key 非空 string + typed value；禁止 result 占位；以真实写入/效果回执验证，值类型需与 schema 对齐 |
| emit_event | Task | event_name 非空 string，可选 typed payload；禁止 result 占位；需要实际事件通道和效果回执 |
| run_subtree | Subtree | catalog category 是 Task，但 `behavior_tree` 参数的 owner 验证要求 Subtree kind；目标是已注册树 ID，不是任意文件路径；打开资产依赖 ID→URI 解析 |
| script_task | Task | callback 为 provider-qualified `<package>::<node-id>`；缺 VM/bridge 真实报告 Blocked；需调用与返回反馈，不用 result 配置绕过 |

当前 `GraphNodeDescriptor::new` 只接 id/display_name/category，AI palette 没有调用 with_input/with_output；`GraphPinDescriptor` 只有 name/value_type/required，也未描述单父/顺序/执行类别。向 Parent 的 P02 交接必须包括这些约束，不能仅导出两个圆点就声称图连接支持。

保守真实拓扑的 pin 设计：非 root 节点一个 parent input；Selector/Sequence/Parallel 为有序 children output；Decorator 为恰一 child output；Task/Service/Subtree 零 child output。UE 的 root 是特殊 composite-only pin，而 Zircon `root_node` 可指向已存在节点，没有独立 root DTO。是否投影特殊根头须作为可逆视图提案，不能新增虚构运行节点；Decorator 附着投影仍待 P05。连接到已有 parent 的 input 按 UE Schema 的 Replace connection 反馈做**原子重接**，不是先删旧边、失败后留半张图。先检查候选整图的自连、方向、循环、重复边、单父和 child count；成功才提交，Undo 一次恢复原 parent 和有序 children。

### 命令、保存和调试守卫

| 操作 | 提交前与提交结果合同 | 当前缺口 |
| --- | --- | --- |
| Create/Connect/Reparent/Delete | 文档 revision 未变、可写、非调试锁定；按真实 catalog/kind 验证；一次编辑事务，保留未受影响 ID | 文档命令 handler、共享 graph 输入与 metadata 尚未实现 |
| Rename Key / Change Type / Delete Key | 按 schema 与引用索引检查；改名必须原子更新真实 `blackboard_key` 引用；被外部资产引用且无法完成事务时拒绝，列出依赖并可跳转 | 当前树 DTO 不绑定 schema；需要资产引用 owner，不能全仓文本替换同名字符串 |
| Validate/Compile | 绑定同一 URI/revision/source hash；parse→`validate_behavior_tree_descriptor_for_compile`→catalog compile；诊断关联节点 ID/参数 key，失败保持原文档 | 直接调用 `compile_behavior_tree` 不包含所有 manager 参数/child-count 验证；compile_toml 才展示了完整入口顺序 |
| Register/Execute | 独立编译成功仍需目标子树已注册、schema/world/agent 实际可用和真实 tick；保存不自动把 dirty 图热替换进运行世界 | 资产版本→runtime 注册代际及安全替换合同待 owner 确认 |
| Save / Save As / Reopen | 保存当前 revision 的原始 DTO、编辑 metadata 和依赖，保留未知参数；失败不清 dirty；关闭后从存储重开，核对内容 hash、ID/拓扑/键类型/位置；Save As 由资产 owner 定义新身份及引用处理 | 目前只有 import/open/validate/compile 的 `.v1` schema ID 字符串，未发现对应本域 payload 定义或 handler；不能自行猜字段并发出成功提示 |
| Asset Jump | 依据真实 implementation/provider 或 subtree/schema ID→资产 URI；无解析/无资产显示真实失败；Back 恢复原文档视图和选择 | runtime 的字符串引用不天然可由资产浏览器打开 |
| Pause/Resume/Stop | 向实际 PIE owner 提交并等待其状态；pending 时禁重复；失败保持实际状态；Stop 后清 mirror/高亮/运行值 | latest snapshot 不能证明暂停/继续控制已可用 |
| Breakpoint/Step/Current vs Saved Values | 需要 trace/断点控制与帧身份，再改变 UI 和 runtime 状态 | 当前无后端，不做假的本地高亮或历史缓存冒充 UE 调试 |

### AI leaf 的候选资产编辑边界

按 Parent P04 的域分工，首个实现切片仅触及 `zircon_plugins/ai/editor/Cargo.toml`、`src/lib.rs`、`src/authoring/` 及 `src/tests/authoring.rs`，不改公共 core、graph、catalog、主题或导出。AI leaf 复用 runtime 的 DTO 与 `compile_behavior_tree_toml` 完整验证链；原 `compile_behavior_tree` 不是等价的参数/child-count 校验入口。TOML 写回沿同一 v1 DTO，拒绝未知字段，避免反序列化后静默丢失扩展；规范化文本不承诺保留 TOML 注释排版。

候选是不可变源值：名称、类型化参数、abort policy、创建、保留兄弟顺序的重接、真实单子 Decorator 包装/解包、删除子树均在候选整图上完成，成功校验才返回；原源值在失败或 no-op 时不变。已配置 `weight.*`/`weight_*` 的 random_selector 自动拓扑变更先明确拒绝，避免把位置权重静默重新分配到另一子节点，完整带权事务仍 open。

这不是文档宿主、历史栈或实时写入口。Parent 的公共文档生命周期/事务预算准入、`EditorTransactionEngine` 的一次 materialization 与 CAS 发布/Undo、原始 URI/资源保存和失败恢复尚未接入；不得在 UI callback 中绕过它们直接替换源，也不把返回候选计为实际创建、保存、执行或产品通过。新增聚焦用例必须覆盖真实 runtime 校验、子顺序、失败保持、wrapper ID 及中文/类型化参数的往返，托管执行回执仍需 P08 的独立插件 manifest 路由。

Parent `ai-followup-02.json` 已确认上述八文件的本域归属，要求继续补齐正确的 Subtree 创建和参数回归。当前新增三项后共 14 个用例：`run_subtree` 必须为 Subtree kind，合法零 child、目标注册 ID 与编辑后的有序拓扑经 TOML 往返保持；Task kind/缺失、类型错误、空白、自指目标以及给 Subtree 加 child 均须被真实 validator 拒绝且原源值不变；独立编译可接受尚未注册的外部目标，但 manager 必须拒绝，目标真实注册后才可注册调用树。测试调用实际 runtime/manager，不模拟执行结果；14 项均尚未执行。旧 `ueai24-bt01` 输入仍绑定原 11 用例源码，不覆盖其身份或哈希，新修订另记 `bt-authoring-02-*` 证据。

`AiBehaviorDebugFrame` 有 behavior_tree ID，但没有文档内容 hash、注册代际、源资产 URI、运行 tick 时间或完整调用栈；事件 sequence 是投递顺序，不能显示成 runtime tick。`BtNodeResultEvent` 只带 world/entity/node_id/status/diagnostic，没有树版本。运行时 subtree 的 active_node 使用 `target_tree_id::node_id` 拼接，必须经真实树/资产解析后跳转，不能把完整字符串误当当前图的节点 ID；分隔符和同名节点路径歧义仍需合同。当前 node-result mirror 按最新快照 active_node 裁剪结果，不能据此生成全部分支执行历史。

产品投影必须区分 `no-session`、`awaiting-frame`、`no-agent`、`different-tree`、`unresolved-node`、`live-frame` 等实际数据条件；这些是待实现状态名称，不是现有注册枚举。只有匹配选定 frame.behavior_tree 和可证明兼容的图才显示节点执行高亮。仅树 ID 相同不能证明 dirty 文档等于已注册运行版本；待补版本合同前应保留身份不足差异。不能从 active_node 反推整条路径的真实搜索成功/失败、上次活动、abort 区间或 Service 状态。

## 交给 Parent 的具体共享样式/图需求

本域没有改 catalog/tokens/Starship，也不指定平行的 `ai-*` 主题。P02 的表面牵头为材质聊天 `01a16d50-6c7f-7370-853d-4581184b66a1`，Parent 负责跨域合同与集成；P03 仍由 Parent 选择既有组件/类/token 名称并同步 Penpot shared styles。本域随后引用相同身份。

| 内容 | UE 来源与已核对逻辑量 | 共享 owner 的待办 |
| --- | --- | --- |
| 图节点/端口/边 | `SGraphNode_BehaviorTree.cpp`、Schema 与 ConnectionDrawingPolicy；保留真实 ID、pin 类别、顺序、命中和焦点 | 通用 graph 能接业务拓扑与连线预检；AI 不复制 AnimationGraphPaneBody 或自建第二套图底座 |
| 主体/子区域 | `BehaviorTreeColors::NodeBody::{Task,TaskSpecial,Decorator,Composite,CompositeScoped,Service,Root,Error}` 与 `BTEditor.Graph.BTNode.Body` | linear RGBA 进入既有颜色转换；类别与错误状态共享，不能把数值直接写成 sRGB hex；Service 样式可在设计中存在，真实运行态不得伪造 |
| 边框优先级 | `GetBorderBackgroundColor`：条件分支中的 abort0→abort1→quickfind；随后 selected-subnode→broken→disconnected→active→previous→connected-root→inactive | 共享状态投影需区分 selection 和 runtime；无对应数据的状态不合成。普通主节点的选中还需沿其外层 Graph owner 验证，不能将 subnode 分支当完整 selection 合同 |
| 内容流与端口 | 主节点 padding 8；Decorator/Service padding 2；输入端口水平 padding 20；low-detail 标题 placeholder 宽高最少 75/22 | 只作为这些具体元素的目标；中文/长标题按测量流式扩展，不把最小值设成所有节点的固定大小 |
| 序号与文本 | `BTEditor.Graph.BTNode.Index` 20×20；IndexText Bold 8；对应 padding (4,0,4,1)。黑板行 icon + name + debug-value，后两 slot 水平 padding 3，debug-value 右对齐 | 字体匹配/基线来自 Parent 字体闭包；不把 UE 字号直接等同某个未确认的 Penpot CSS px |
| 节点图标 | `BTEditor.Graph.BTNode.*.Icon`：Selector/Sequence/SimpleParallel、Blackboard/Cooldown/Loop/TimeLimit、Service、MoveTo/RunBehavior/Wait 等，多数 24×24 | 沿既有图标资源 owner；Zircon Parallel 不能仅用图标暗示等同 SimpleParallel；无对应实现的 UE 节点不加入可用菜单 |
| 黑板/模式/调试图标 | `BTEditor.Blackboard.NewEntry` 与 SwitchToBehaviorTreeMode/SwitchToBlackboardMode 为 20×20；`.Debugger.*.Small` 为 20×20 | 复用共享 Button/IconButton 的 enabled/hover/focus；图标存在不启用未接后端命令 |

## AI 接入现有共享图基础的合同

下表是 AI 向材质表面 owner 与 Parent 的接入合同，复用 `zircon_editor/src/ui/graph` 并保留 MAT-SHARED-001。当前本域仅实现不可变投影/连接候选的一部分；**未接生产 GraphModel、retained owner、预算及事务**。公共导出已由材质交付，本域不写共享路径。图的业务数据、连接合法性、Decorator/Service 语义和资产编辑仍在 AI 域。

| ID / 输入或结果 | 现有接口与 AI 接入 | 尚需公共合同及验收观察 |
| --- | --- | --- |
| AI-C01 节点/边身份 | `GraphNodeView` / `GraphPortRef` / `GraphEdgeView` 的本域投影复用原 node.id，使用宿主完整 graph-local bounds，保留每个 parent 的真实 child 顺序 | 树 ID/revision/几何节点集合/有限正尺寸先校验；未接生产 GraphModel 或持久化几何。共享表面不保存第二份树；边端点与实例路径须可回原文档 |
| AI-C02 root/Tree 约束 | `StructureConstraint::Tree` 和 `default_connection_verdict` 可复用同向/缺 pin/类型/重复/环检查 | 当前 Tree 只按相同 target port 拒绝已有入边，不等于 root/整树可达/child count/顺序验证。AI 使用唯一 parent input 并校验整图；未连接创作态与可执行树分开 |
| AI-C03 原子替换 | 本域 candidate_for_connection 在同 tree/revision、方向/kind/精确 pin 校验后构造一次 Reparent，index 按移除旧边后的数组解释，复用完整 TOML/manager/catalog 编译链 | 候选成功不代表文档发布；生产表面还须显示旧边替换反馈，经一次预算 permit/prepare/apply、同宿主 live revision 和一次 undo。不得拆成 Disconnect/Connect 两次提交 |
| AI-C04 文档与预算 | `delta_retention_v1`→one-use permit→`prepare_delta_v1`→`apply_prepared_delta_v1`，`retention_revision_v1` | AI delta/inverse/Target/label 都满足现有 retained owner 预算与 revision 校验；不另建无界整图快照历史。stale、超预算、参数/拓扑失败均不改文档，不提前清 dirty |
| AI-C05 外 crate 可用性 | 材质已重导出GraphDeltaProofV1、PreparedGraphDeltaV1及GraphAttachmentView；最初29584afa…记录保留，最新mod.rs为`2a0a4cda94a77b8f2354cf1e87d8c49618f12547ed4f035b73605a3f7e9ffd3b`，另有中立路由/附件辅助API | 源码可命名性缺口已解决；AI不重复修改、不复制类型。当前只消费定向路由；实际编译及retained proof/prepare/apply接入未验证，不能关闭AI-C04 |
| AI-C06 位置与输入 | `GraphCanvasState::graph_to_screen/screen_to_graph`、`GraphNodeDrag::moved_nodes`、marquee/selection | pane-local DIP 输入统一转换：先减物理 pane 原点并除实际 DPI，再 screen_to_graph；绘制反向变换后 host 只施加一次 DPI。拖动预览不修改 DTO，release 一次 metadata delta，Esc 原样取消；pan/zoom 不进节点参数 |
| AI-C07 路由与箭头 | 材质已保留原水平入口并增加`route_connection_with_directions`/`endpoint_tangents`/`target_arrow`；AI的`route_edge`已按真实树/revision/端口/现有边校验后调用Down/Up方向，直接使用宿主传入的graph-local锚点 | 不造BT renderer或横向回退。当前仅为本域源码消费者，未接retained paint/hit；同次测量/transform generation由host校验。共享四向控制点仍不等同UE单位锚点差切线/4.5分离/brush/1.5线宽与缩放，双端测量和动态trace仍缺 |
| AI-C08 调试连线状态 | `BehaviorTreeConnectionDrawingPolicy.cpp`：真实 active/previous 两端路径 10 线宽+bubbles，暂停搜索结果 5 线宽/成功失败色及反向 flag | 当前 AI 帧没有整条 active/previous/search path，不能因单一 active_node 自动放大整条祖先边或生成 bubbles。普通拓扑连线可显示；运行样式等真实 trace，状态由 AI 投影而非共享控件计时 |
| AI-C09 附件身份/命中 | 已有`GraphAttachmentView`及材质新增的`GraphAttachmentPresentation::from_view`，可保留attachment/host ID；原`attachment_labels`兼容接口仍在 | 中立投影没有bounds/instancePath或输入authority，AI仍保留真实包装节点，不据此折叠Decorator。独立命中/焦点/菜单目标和文本行框须host实际测量；P05等价证明及Service后端仍缺 |
| AI-C10 文本/菜单/键盘 | 现 catalog 字段/菜单和 `GraphSelection`；共享 focus/文本编辑 owner 优先 | text/IME/Ctrl+A/Delete/Undo 不误路由图命令；创建菜单接 AI catalog 与 domain preflight；关闭/取消回到触发 pin/node，目标删除则回合法父 canvas；低缩放文字/端口可达性须真实验证 |
| AI-C11 剪贴板/删除 | `GraphClipboardModel::serialize_subgraph/paste_subgraph_delta`，AI 实现实际 ID remap 与参数/引用保存 | 复制需保留子树引用、包装节点/顺序及 positional weight 语义；paste 为新 ID。删除内部选中但未选 wrapper/host 的规则由 AI 明确，undo 一次恢复拓扑、引用与选中；不使用全局字符串替换 |
| AI-C12 证据与导出 | 现 LayoutReviewCase、sourceNodeId/instancePath/native geometry 和 Penpot metadata 合同 | 每个 node/pin/edge/attachment 的几何绑定实际文档/hash/revision、canvas transform、DPI、world/entity（若调试）；pan/zoom/resize 后同 identity 可复核。缺来源的图像或不可编辑折线不算已接入 |

AI-C05 的最小重导出已落实，不改变 retained-memory 机制。AI-C03 当前只提供整候选，retained bound、一次准入/prepare/apply 和发布仍由既有 authority 接续；AI 不私增共享枚举或让 UI 越过事务引擎。AI-C07 的常量来自实际 Slate 源码，不能在产品中反复针对截图偏移；路由、箭头和命中须共享同一测量结果。

### 本域图候选实现与仍缺的宿主合同

[graph.rs](../../../zircon_plugins/ai/editor/src/authoring/graph.rs) 借用真实 immutable Asset 与宿主 revision，投影已有共享 view 类型；宿主提供全量 graph-local bounds，adapter 不发明节点位置或重复施加 pan/zoom/DPI。root 不生成父输入端口，Decorator 保留真实单子包装，Task/Service/Subtree 不生成 child 输出。Service 仅投影现有零 child kind，不声称周期执行；没有添加 helper Root DTO、假附件或运行状态。

连接候选检查 tree/revision/节点/方向/精确 kind 端口；一个 Reparent 在整树候选内同时移除旧边、插入新位置并调用既有完整 compiler。失败和 no-op 保持原源值；带权 random_selector 的保守拓扑守卫继续生效。这些 API 会分配候选，必须在宿主准入后调用，不能从无预算 hover/paint 路径调用。当前没有证明全候选 retained bound、实现 GraphModel、分配 permit、维护历史或写入实际资产；公开返回值也没有这些 authority。

该版六项回归源码位于 [behavior_graph.rs](../../../zircon_plugins/ai/editor/src/tests/behavior_graph.rs)，通过literal module纳入`authoring_tests::`，覆盖真实kind/包装/顺序、宿主几何失配/非有限值、同revision/端口、一次Reparent/index/no-op、环/空Decorator整树拒绝及weighted守卫。原版20authoring+4OVW-42未运行，十二源与patch `1d06a8fd…`按原证据保留。后续四项定向路由回归及当前源见下节12，不以原guard或旧验证输入证明新代码。

保存复用 `EditorTransactionEngine::capture_save_token` 和 `mark_saved_if_unchanged`：写盘对应的 history/generation 与保存 token 一致才清 dirty；保存期间继续编辑时返回 HistoryChangedDuringSave。UI 文案、资产字节写入与事务引擎的 saved mark 必须区分，不能直接调用“标为已保存”掩盖写盘失败。现有 `AnimationAuthoringDocumentStore` 是动画专有模型，不复制或改名为 AI 文档；共用的是事务/历史/资产 owner 合同。

### 与动画时间轴/播放 owner 的边界

共享时间轴、曲线和播放控件 owner 为 `01a16d51-741b-73b3-a61a-2189cd38f81e`。AI 当前没有曲线资产编辑需求，不重建曲线表面。BT 的 Pause/Resume/Stop 可复用共享 transport 外观、focus 和 pending/failed/acknowledged 状态，但实际命令归 PIE/AI；Back/Forward Into/Over、Step Out、断点、Current/Saved 值需要 AI trace 模型，不能把通用播放位置当运行 tick。StateTree debugger/simulation 和 EQS profiler 后端仍待定；本轮不请求播放控件生成一条假历史来填这些窗口。

## 交互复刻合同

| 场景 | 行为目标与可观测结果 | UE 源码 |
| --- | --- | --- |
| 创建节点/连接 | 空白右键或 pin 拖出菜单来自真实 catalog；按类别过滤；拒绝自连、同方向、循环和非法父子；连接顺序与保存拓扑一致 | `EdGraphSchema_BehaviorTree.cpp:129,238` |
| 选中、框选与焦点 | 单选/多选更新真实 Details；Tab/方向键沿可交互 owner；切 tab 后恢复该文档的焦点，不在弹层关闭后落到无关视口 | `BehaviorTreeEditor.cpp:644,798`、共享 focus 合同 |
| 装饰器与服务 | 宿主选择、附加项选择、编辑参数、重排和删除均保持运行语义；abort policy 使用真实值 | `SGraphNode_BehaviorTree.cpp`、Schema 与 runtime DTO |
| 黑板键 | New Key 类型选择；唯一名；改名/类型/required 提交；被引用键删除的可解释拒绝或原子引用修复；撤销恢复引用 | `SBehaviorTreeBlackboardEditor.cpp` 的事务、CanRename/CanDelete 与键类型筛选 |
| 复制/粘贴/删除 | Ctrl+C/X/V/D、Delete、Select All 沿图命令；复制产生新 ID，外部引用处理明确；可撤销 | `Source/Editor/AIGraph/Private/AIGraphEditor.cpp:49` 起 |
| 搜索与跳转 | Ctrl+F 聚焦 Search；中文/长节点名/参数搜索；激活结果定位真实节点、Details 同步；无匹配是真空数据 | `BehaviorTreeEditorCommands.cpp:27`、`FindInBT.cpp` |
| 资产跳转 | 双击 subtree/task 和 Blackboard 引用打开真实资产；Back/恢复选择；不存在资产显示真实错误 | `BehaviorTreeEditor.cpp:1231`、工具栏及资产 registry |
| 保存、撤销/重做 | Ctrl+S 的真实存储回执；保存失败不清 dirty；Undo/Redo 同步图、详情、键引用和选择 | `BehaviorTreeEditor.cpp:194,204`、AIGraph 事务 |
| 运行状态 | 真实 world/agent/tree 选择；读取真实 tick、active_node、blackboard、node result、diagnostic；暂停/停止只在真实命令确认后改变状态 | `BehaviorTreeDebugger.cpp`、现有镜像消费端 |
| 调试历史/断点 | Back/Forward Into/Over、Step Out、当前/保存值和断点动作必须有后端确认；现有 latest-frame mirror 不足以宣称支持 | `BehaviorTreeEditorCommands.cpp:41` 起 |
| 感知调试 / OVW-42 | agent 列表来自真实帧，视野/听觉/刺激 overlay 来自真实世界；切世界/停止会话清除；沿 payload 的真实投递来源，不用全局最新序号重标另一 world 的帧 | `GameplayDebuggerCategory_AI.cpp`、AI overlay/controller；[Gameplay Debugger 合同](ai-gameplay-debugger.md) |

运行调试时允许的编辑与快捷键要匹配实际安全状态。文本框中 Ctrl+A、Delete、撤销由文本编辑 owner 处理，不能误路由为删除图节点。弹层 Escape/选择关闭后要回到触发节点/键，键被删除时落到其合法父列表。

OVW-42 已由 Parent 归为本域 AI-W25：AI 负责数据投影，Parent 负责共享 PIE/diagnostic/trace 生命周期，Actor 仅供真实关联定位。AI editor 的 `agent_frame_in_world` 新增借用的 session/payload delivery sequence/frame 一体读取；四项回归尚未执行，不构成 UI、源版本或运行高亮完成。详细字段、八项接入合同及无 trace/源失配守卫见专项记录。OVW-03/04 Navigation 作者/bake 仍由 Parent 待接续，AI 不用导航窗口替代 EQS/StateTree。

## 尺寸、状态与真实资源

每个适用窗口都覆盖以下四个配置，不能用 125% 诊断图抵扣：

| 配置 | 逻辑客户区 | 物理客户区 | 实际 DPI/scale |
| --- | --- | --- | --- |
| S1 | 1280×800 | 1280×800 | 96 / 1.0 |
| S2 | 900×620 | 900×620 | 96 / 1.0 |
| S3 | 640×520 | 640×520 | 96 / 1.0 |
| S4 | 1280×800 | 1920×1200 | 144 / 1.5 |

每一配置：default、empty、selected、focused、popup-open、popup-focus-return、scroll-start、scroll-end、zh-CN、long-text。场景逐项保持同一逻辑数据和 Penpot/产品状态，不以静态文本覆盖模拟生产结果。

专属功能场景：新增/保存/重开树；断开再连接/非法循环拒绝；添加/修改/重排 Decorator/Service；黑板六种类型与引用的重命名/删除/撤销；subtree/blackboard 资产跳转；真实 patrol→detect→chase 执行反馈；跨世界同实体 ID、旧 sequence、停止并重新开始会话；诊断与保存失败；空图和无 agent。测试夹具允许用于测试，产品证据必须明确绑定本轮真实资产和 world/entity，不能把夹具表格当作生产绑定。

候选案例身份规则为 `ai-{bt|blackboard|perception}-{state}-{width}x{height}-dpi{100|150}`。实际 `LayoutReviewCase` 由现有协议生成和校验：复用 sourcePath、host、viewport、dpi、locale、state、sourceNodeId/instancePath 和真实业务 projection；此处没有伪造任何已执行 case 或 native receipt。

## 门禁与每图记录

真实 Penpot 阶段记录 file/page/board/library/component IDs、版本、instance→catalog document/slot/参数映射、flow、共享 styles，以及服务/插件输入身份。必须见到保存 ACK，关闭原 context，fresh 认证重开同一文件，校验 metadata/来源和官方 PNG/ZUI 导出。只使用插件 JSON 或 mock browser 不能通过这一门。

先验证实际支持的局部样式属性，再在真实 Penpot 做一次可审阅修改：记录 before/after 值与 owner，导出保留 ID 的 delta，写回原始 `.zui`，由产品资源加载闭包证明消费，重启正常 Editor，再以同状态新图证明像素变化。恢复默认也须同一路径验证。共享 token 的跨两窗口族传播与 reset 验收交给 Parent 统筹，AI 提供一个实际消费者，不创建平行主题。

原生证据复用 `tools/analysis/visual/record-editor-normal-window-evidence.py` 与既有 normal-frame 协议，保留原 EXE 启动、PID/birth、EXE/runtime/resource SHA、nonce、generation、frame/readback/source/sequence、ACK/release、资源/字体就绪、实际 `GetDpiForWindow`、Winit scale 和客户区。捕获只在 Parent 协调的桌面时段执行。拒绝 stale case、不同程序、资源未就绪、无效 capture hold 或不同状态关联。

每图记录以下字段：case/hash、窗口/状态/locale、逻辑/物理客户区与实际 DPI、Penpot 官方图/geometry、正常 Editor 原图/geometry、并排图、差异图、关键边界/文本行框误差、逐项人工复核、差异 ledger IDs、结论及证据引用。误差目标不超过 1 DIP；没有对齐身份的文本/节点不能从最大误差统计中静默排除。

`passed` 只在实际资源就绪、捕获有效、该 case 检查通过时填写。`accepted` 还要求双端同状态对比、逐图复核与明确范围通过。当前已有 13 张 revn 14 Penpot 设计图及逐图复核，正常 Editor case/双端对比仍未形成，产品 passed=0、accepted=0。

## 聚焦验证边界

本域已形成真实 Penpot 设计及 AI leaf 的不可变候选源代码；尚未运行 Rust 或插件构建。托管执行前须声明实际模块/包/features/target/filter/profile/命令，再由 Jenkins 执行：

- AI runtime：`zircon_plugin_ai_runtime` 的行为树解析/编译、拓扑、黑板引用/abort、真实执行与 debug event 的相关 `--lib` 过滤器。禁默认 features；依赖 manifest 当前显式引入 runtime 的 ai/net/physics/sound contracts，这些是传递合同，不能声称完全没有其他依赖。
- AI editor：`zircon_plugin_ai_editor` 的 catalog/Toolkit 注册、真实镜像过期/跨世界/停止清理、编辑/撤销/存储回归。禁默认 features。
- 插件：只跑本轮 `.zui`/组件/导出映射涉及的 parser、ID/绑定/事件保留、共享组件实例、flow/style 支持合同；复用 Parent 同源证据，不重建无关整个插件。
- Editor 宿主：只在实际集成后，选 `zircon_app --bin zircon_editor` 与当前主会话批准的最小 Editor host feature 集；禁止全仓、全目标和 all-features。

**入口的历史与当前边界**：AI 两个 leaf package 属于 `zircon_plugins/Cargo.toml` 的独立 workspace；根 manifest 没有它们。较早 `b56a55b8…` runner 无 manifestPath 的缺口保留于历史。Parent route-02 与本域实际 `baac70c9…` pinned parser/recipe 已读回可表达该 manifest 和 AI 单包 `authoring_tests::`，但不代表 Rust 执行。当前 `1e1b939f…` 已核对 capsule 及全部 11 个固定文件；以原 identity 经正式 wrapper 执行 QueryOnly，实际退出 2、`build_scope_policy`：该版本 `entry.py:472–481` 仅准插件 manifest 下的材质包/`material_graph_` 配方，尚未到服务器 query 或 Cargo。详见本域 `bt-authoring-02-route-observation.json` 与原始 query log。旧版接受 AI 预检、源捕获变更拒绝及授权 pending 均作为当时事实保留；不借材质 filter、回退旧 client、重放 create 或换成根/full build。

共享 `zircon_plugins/Cargo.lock` 的 AI editor dependency 尚缺 manifest 已新增的 toml 边。Parent 已交原 Jenkins generation owner `01a12af6-7511-70d1-a452-61b80678eb56` 提供最小锁解析计划/diff/receipt；本域不手改该共享文件、不移除 `--locked`。生产文档/预算/graph 接线、源码稳定窗口、原验证请求授权及精确传递 features 仍须实际 owner 准入。

OVW-42 接续时安装已更新为 `cdd14500…`；核验 capsule 与 11 个固定文件后，对 AI 单包 authoring、新 gameplay debugger、既有 mirror/overlay 四个精确 filter 的非执行 parser 检查均仍被材质专用策略拒绝。没有重放原 query/提交或启动 Cargo。最新源码/路由观察见 `gameplay-debugger-01-source.json`、`gameplay-debugger-01-route-observation.json`；上段 1e1 的 QueryOnly 是保留的前次实际结果。

Parent 再交 route-02 信息后，又实际调用当前正式 `tools/dev/local-cargo.ps1 -QueryOnly` 对账原 AI validation identity。cdd 的 pinned entry 被 selector 选中，实际仍退出 2/build_scope_policy；调用前后 installed/identity 指纹一致，未用 canonical parser、未到服务器 query/Cargo。详见 `ai-p08-selector-02.json` 与原始 query log；baac 的旧能力回执保留，不当作当前 selector 的许可或编译结果。

前次 E 盘实读约 19.62 GiB，最新 2026-10-24T12:41:28Z 只读预检约 18.94 GiB 空闲；当前 runner 源码的 35 GiB 预留阈值尚差约 16.06 GiB，另需所选 recipe 的新增磁盘预算。实际资源门及 policy override 尚未核实，不把潜在差异写成存储拒绝或清理授权。build root 仍为物理 E:/cargo-targets，无 reparse 祖先，不改用路径别名或其它盘。

最新 `ai-authoring-readiness-03.json` 固定十个相关 manifest 和十个候选源文件的哈希。AI leaf 的 defaults off/features=[] 不会移除现有依赖声明的 runtime `native-host`、Editor `target-editor-host`；后者含 animation/graphics/navigation/script/text/ui/平台窗口输入。SDK editor 同时启用 runtime/declaration/editor_contribution。这里只证明局部 manifest 的字面 feature 闭包，未运行 Cargo 解析或编译，不删除依赖 feature 缩小表面范围。正式执行应声明这些传递 host 能力及 AI 单包 lib/test profile，并分别验证 authoring、gameplay debugger、现有 mirror 和 overlay 的精确过滤器；AI 配方准入、受管锁更新、原请求/源窗口和资源门均须先有真实回执。当前无新的服务器请求或 Rust 结果。

原入口 owner 后续提供的 `ai-consumer-reconciliation.json`（SHA `6071fda0a3b26627455cb61f9129476c3a9e99d632d1b69c2651f64ffa3d84aa`）已在本域 `ai-owner-reconciliation-04.json` 核对：当前 client 与 server recipe 两侧均拒 AI，source guard 仍选材质 member/package，没有 AI 源闭包、pool 或验证回执。管理 #400 及四份真实原生管理终态不构成 AI 或 live-host 验收。本域不修改 owner 的 entry/runner/tests、sealed 文件或共享锁；原 owner 本次只读且不另开恢复。旧 create 在 owner 检索的现存记录中未证明实际入队，原 repoId/runtime 不匹配当前，继续保留输入与失败，不换身份或重放。具体责任和精确路径见 [交接记录](ai-parent-handoff.md)。

AI-only 历史锁解析计划 SHA `a4355fd3c40ea9b11fed6d222f11018dae1b05bf20900b22b512329903f48751` 经本域核对且原字节保留。Parent 随后明确授权合并修订 r2，SHA `1f8593faa8f78998914c31ea5fa6cde84db25924ccd87d4a4fe6209b959177dc`：拟写仍仅嵌套锁一个文件，两个已有条目四边，AI 加 toml、particles 加 serde/toml/uuid，复用锁内既有版本；其他字节、版本、域和根锁受保护。本域核对为 `ai-combined-lock-r2-reconciliation.json`。当前 generate 缺 lock-resolution 配方，真实 diff/after SHA/generator receipt 仍为空，captureReady=false。静态提案与 dry-run 不产精确锁字节，旧本域锁 patch 仅作历史预期增量，禁止据此发布；真实候选须来自合法受管 resolver，经范围复核和 before-hash CAS 发布，附原请求/source/resource/native/双 EOF/恢复证据。

合并 generation 计划不合并测试身份或编译：AI 仍为自己的 authoring/gameplay/mirror/overlay 范围，粒子仍为其 owner 的 source_tests::，双方测试 `--locked` 保留。本域原十一项输入、历史十四项 authoring 加四项 gameplay，以及当前新增图候选后的 20+4 分开记录，全部未执行。粒子 owner 报告旧九源 hash 中五项变化及新增可达源、23 个静态定义；这些未由旧输入资格化，尚非 Cargo 发现/执行，由该 owner 重新声明完整 source/features/resources 后续验证。本域不据此扩大 AI 闭包或借粒子身份。

### 最新恢复后的 AI 验证边界

Parent 已独立核实原 Jenkins 恢复 r2 的完整 native 终态；这是管理恢复结果。当前本域实读 installed 已由 Parent 的 20b3 后续变为 `9c684aef96b8b1c94319aa8619207f8b07aefffa71d571d4f58708d2c3408233`，manifest 与 5 server/6 client 全匹配。四个 AI 单包/lib filters 的 pinned parse policy 和实际 Management.validation_recipe 只读构造均成功；当前“仅材质”的能力判断已过期，历史拒绝仍保留。

正式 wrapper 沿原 `ueai24-bt01/validate` 与 `ueai24-wt01/create` 输入分别 QueryOnly，均返回 `Current session authorization is pending`、ordinary_entry_unproven、exit 2。验证首次调用在 PowerShell 参数传递处丢失 harness separator、pre-query 退出，错误保留后用显式 argv 数组更正同一 identity；没有改输入、借他域 identity 或重新提交。由原 Jenkins owner 对账历史收件与自动上下文，不要求手工 grant，不从 pending 推断未曾入队或其他聊天失败。

十二源/20+4 静态定义不等于旧八源/十一项输入；当前源仍匹配既有 guard，编译/执行为零。嵌套锁依旧缺 AI toml 和 particles 三边，当前 generate 尚无 lock-resolution 配方，生成 diff/after/receipt 为空。正式 source closure/pool/writer/resource 准入、锁解析和合法验证上下文仍须真实回执，不移除 --locked 或传递 host features 绕过。完整观察为 `ai-recovery-07.json` 与 `ai-recovery-07-client-recipe.json`。服务恢复不授权生产图预算/undo、真实 doc revision/trace ACK、Desktop/DPI 或正常 EXE；这些产品门继续 open。

原 owner 随后以 implementation-r3 正式交付有限入口：#406 管理 81 checks/267 subtests；本域核对四个给定 SHA、当前 11 固定文件和 26 项 workspace 静态输入均匹配。四 filters 的共同 digest 为 `9fb6c82299deb64a8dfb83f2795e210b90e81bfa6da080c12f16726e803896cd`。这是入口和静态合同，不是当前十二源/20+4 测试的准入；通知中的 14+4 为历史数目。当前 combined 锁仍缺 resolver 完整 diff/CAS/恢复事务、原 server 生成准备、单锁 writer 和可信预算/reservation 四门；result 中生成所有真实 diff/after/native/recovery receipt 为 null。详见 [最新交接](ai-parent-handoff.md) 的“正式交付接收 08”和 `ai-entry-r3-08.json`。本轮不重复原 session pending 查询、不借旧身份/他域 filter、不修改锁或启动 Cargo。

### 创建菜单的实际设计切片 09

按 UE `EdGraphSchema_BehaviorTree.cpp::GetGraphContextActions`，空白图的创建目录只有 Composite/Task 类别，Decorator/Service 属于节点上下文，不把十八个注册项平铺成等价可执行操作。当前 Penpot 将原 1280 弹层补成源码四个 Composite、七个 Task 类别条目；`run_subtree` 位于 Task 类别但创建 kind 明确保留 Subtree，`random_selector` 保守映射 Selector，Parallel 不暗示 UE SimpleParallel 语义。Decorator 仍按 Parent P05 保留真实单子包装，Service 周期执行仍无证明。

重复搜索控件直接由公共 WorkbenchSearchInput 的原生 Assets 拖放产生两个 linked copies，source/main/shapeRef/canonical metadata 保留，未创建菜单行或搜索框的本地公共 main。普通菜单标签目前仍是本域文本，公共 MenuRow 尚未发布；对源 props、enabled、hover、selected、focus、键盘移动和动作提交的正式实例映射由 Parent/原 pipeline owner 交付。共享 colors/typographies 仍为零，不能从搜索框外观一致推断主题传播成功。

UE 同一菜单里的 Auto Arrange/Add Comment 保留可见禁用外观，不挂成功 handler；本域没有相应的布局事务或持久评论数据，不将它们列为完成操作。菜单上下文、禁用理由和十一项实际 implementation/kind 记录在 `penpot-node-menus-r01/intent.json`。主参考仍为 UE 实际 Schema；沿现有 ReactBits Command Menu/App Dialog 的信息分组及弹层归属合同，不引入另一套视觉主题。

此前两次查看器验证未出现目标 DOM，第二次页面失去响应，原失败及自有进程结束记录保留。后续在相同 revn 28、原生查看器 100%/wasm=false 路径，选择可见实例后已实际完成打开→外部关闭→同一触发点重开，三图逐张核对。先前的 DOM 查询命中过隐藏同 ID 副本；这解释该定位误判，不代表全部无响应根因已确定或已修改 Penpot 源码。

实际右键也可打开，但左键同样触发，未复现 UE 右键上下文规则。点击公共 SearchInput 副本并输入 Wait 后仍为 Search，可编辑元素数为 0；Esc 不关闭，方向上键转到 Blackboard 画板；焦点始终为 BODY。故鼠标原型通过不覆盖搜索过滤、菜单键盘选择、焦点返回、节点创建或任何引擎命令。实际回执为 `penpot-node-menus-r01/prototype-result.json` 与 `prototype-keyboard-1792858212349.json`，详见 [逐项记录](ai-penpot-design-r01.md)。官方导出 4213/manifest.json 前次实测 Win10061，19001/readyz HTTP200 不替代插件就绪；没有后续插件运行回执。

### 实际命令路由核查11

本轮只读跟踪当前AI插件→editor_support→贡献注册表→host操作分发；不新增命令、改原ZUI事件或绕过公共owner。以下结果是源码观察/静态推论，未运行EXE或Rust：

| 原有身份/来源 | 当前实际链 | 接入前置与判定 |
| --- | --- | --- |
| `view.ai.behavior_tree.open` | `register_authoring_surface`按view ID生成`view.<id>.open`，绑定`MenuAction::OpenView` | 可用于打开既有表面；不证明资产URI已解析、文档已加载或存在AI编辑能力 |
| `ai.behavior_tree.open` | plugin asset toolkit/graph editor引用该独立operation；`EditorCommandDescriptor::operation`只有schema/capability，无event或factory | 与上行不同；按当前host源码，前置通过后会走MissingFactory，不能静默换成OpenView宣称资产打开 |
| `ai.behavior_tree.import/validate/compile` | 只有descriptor，`register_authoring_contribution_batch`调用register_command，不生成factory | 先补真实资源读取与source身份、参数schema、结果/错误反馈及AI factory；不能把`with_payload_schema_id`当验证器实现 |
| `AiBehaviorTreeAsset`、`AiBehaviorTreeGraphAdapter` | 候选与共享view投影代码存在；当前host/插件贡献链无生产调用点 | 使用真实已准入文档及revision，保留完整compile_toml链；mutation仍须一次预算/事务、undo和save，不从paint或hover直接物化候选 |
| 原插件`AiBehaviorTreePalette/GraphCanvas/Inspector` | Table/Space/Space，原12个控件无events；Palette只提供18目录的名称/类别 | 不把目录条目注册当节点创建；保留原ID，待正式codec及typed事件接线，不增无效果handler |

源码依据：[AI贡献](../../../zircon_plugins/ai/editor/src/plugin.rs)、[support批量注册](../../../zircon_plugins/editor_support/src/lib.rs)、[view路径](../../../zircon_editor/src/core/editor_extension/view_descriptor.rs)、[贡献转命令](../../../zircon_editor/src/core/commands/contribution.rs)、[host分发](../../../zircon_editor/src/ui/host/editor_operation_dispatch.rs)。host先检查command、source、资源写权限及enabled，再处理event/factory，故MissingFactory仅是这些前置通过后的静态结果；本轮没有发出实际操作或注入假的成功反馈。

沿既有AI-P04/C04/C10分工，AI继续持有完整serializer/delta/真实compile结果，Parent接DocumentHost/URI资源authority/revision/history/save及焦点路由，公共pipeline保留原ZUI/事件身份的可逆导出。缺这些真实输入时不把Validate/Compile或Save启用为完成动作。recovery07回传已确认2790fb…仍有有限四filter route，但没有锁生成/源码资源准入；入口支持不关闭上述产品路由缺口。静态证据及文件hash见`ai-operation-routing-11.json`，20+4候选仍未运行。

### 共享定向路由的AI源码消费12

共享owner的最新[四文件合同](shared-node-graph-contract.md#四文件辅助-api-实施边界)已实际存在；本域只改自身lib/mod导出、`authoring/graph.rs`及`tests/behavior_graph.rs`，没有修改共享graph/model/commands/canvas、Runtime DTO、公共主题或原ZUI。新增`AiBehaviorTreeEdgeRouteRequest`带真实tree ID/revision、原GraphEdgeView及宿主测得的graph-local锚点/style；`route_edge`先复用连接候选的真实kind/方向/精确端口校验，再核对当前parent.children确有该child，拒绝虚构边，随后调用共享Down/Up路由。非有限、溢出或无法保留端点方向时报告InvalidRouteGeometry，无横向fallback或替代坐标。

该API仅为现存作者边计算路径，不修改树、不生成连接预览成功、不施加camera/DPI或运行态。Decorator保持真实wrapper，Task/Service/Subtree无child输出，Root不造父输入。同一revision并不能证明几何是当前帧；host必须先做retained测量/transform generation和分配准入，再用同一结果完成paint、arrow、hit、sourceRef。箭头brush/尺寸、UE切线及分离/线宽、命中容差与实际DPI仍由共享owner/Parent接续，不因返回route关闭C06/C07/C12。

新增四项回归仍在`authoring_tests::behavior_graph::`，使用明确的单位几何fixture验证外crate API、上下/反向锚点、stale/foreign/虚构边/非法方向、真实wrapper端口和三类叶节点输出拒绝、非有限/溢出几何与失败不改源；不当作真实测量帧或产品图。当前authoring14+graph10=24，另gameplay4，共28静态定义，均未Cargo发现、编译或执行。新十二源记录为`bt-edge-routing-12-source.json`，76784B patch SHA `b00543e01a118d5109a170b37755b0b6039ce0725a8c59222f65fc839b418559`；11 Rust文件格式、精确12路径guard及diff-check均exit0，0 errors/warnings/suppressions。原1d06a8fd/20+4证据仍保存，新源不借其身份重放。

已读共享owner core #353 的source conflict/compileStarted=false记录，冲突涉及本域设计文档；它不是19项已运行或core失效的证据。本域没有停写/放宽守卫或重投该请求。只有source-preparation owner给出真实ready窗口与资源准入时才按当前候选固定哈希；当前无全局freeze、源capture或新增验证提交。锁生成四门、生产文档/事务/保存、正式codec、Penpot键盘/焦点和正常EXE/150%DPI/双端1DIP继续open，passed/accepted=false。

## 生产适配最小输入包13

已收到并实际读取 Parent `ai-followup-04.json`，SHA `d1a2f06701a52b1d46b4a5e8280dda9a2557a638ba04b17d7691793d9587cee7`。Parent 于16:34:23Z独立读回原 file/page 的rev28/855对象、4业务main、58本域及6公共copy，并打开最新菜单及原型失败图。此前“Parent仅读rev14”和本域20+4是历史观察；当前十二源仍匹配 `bt-edge-routing-12-source.json`、patch `b00543e0…`，24 authoring+4 gameplay共28定义，0执行。本轮只补具体生产输入与接入证据，不改这十二源、共享API或ZUI。

机器输入包位于 [ai-production-input-13.json](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/ai-production-input-13.json)，31份实际源码/资产指纹与定位见 [audit](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/ai-production-input-13-audit.json)。包中的 live document、generation、revision、save token 均未取得，保留空值；它是按现有类型整理的接入要求，不是已注册的新DTO、factory或运行回执。

### 资产与真实打开路径

现有 [AssetActivationIntent](../../../zircon_editor/src/core/asset/activation.rs) 已携带真实 `asset_uuid/asset_locator/catalog_revision/resource_revision/source`，[AssetToolkitOpenRoute](../../../zircon_editor/src/core/asset/toolkit_route.rs) 只携带 locator 和 open operation。这两者都不分配AI文档或授予写入权限。Parent必须从实际活动项目解析同一资源、确认 `ai.behavior_tree` 类型及当前版本，再提供注册的 `DocumentId/ViewInstanceId/ToolkitInstanceId` 与原始源字节/摘要。文档revision、资源revision、catalog revision、retained layout/transform generation不能互相替代；资产UUID不能转成文档ID。

本轮查到独立于 operation dispatch 的另一条生产路径：[OpenAsset事件](../../../zircon_editor/src/ui/host/editor_event_execution/asset_event.rs) 检查资源类型、toolkit描述符及命令enabled，然后调用 `open_asset_document_view`；后者打开view、写入route metadata、focus并更新状态文字。所查branch/helper没有调用AI factory、读取BT源或注册AI文档。因此即使出现“Opened asset toolkit”文字，也不能证明AI文档已载入。与之分开，显式 `ai.behavior_tree.open` 的descriptor仍无AI factory；MissingFactory仍只是在前置gate通过后的静态推断，没有动态EXE报错。

现存 [enemy_behavior_tree.toml](../../../examples/vampire/assets/data/enemy_behavior_tree.toml) 原始SHA256为 `fb2b913c9cc03a5b67be654e1749c24e98b913c3164b228d4999ad2767801b4e`。[原sidecar](../../../examples/vampire/assets/data/enemy_behavior_tree.toml.zmeta) 给出UUID `2f7f0e40-bc1b-4ee0-9a8b-5a1590ed5f31`、URI `res://data/enemy_behavior_tree.toml`，类型为 `Data`、importer为 `zircon.builtin.data.toml`。它缺当前 [BT DTO](../../../zircon_runtime/src/core/framework/ai/behavior_tree.rs) 所需的 `format_version/display_name/root_node`，并含不在当前节点kind枚举内的 `condition`；这只是TOML/源码结构核查，未执行Rust导入。sidecar的16位source_digest不是本次SHA256算法，不比较或覆盖；preview_state=ready也不是AI执行证据。该文件/sidecar保持原样，不能改标签充当可编辑BT资产。所查 `zircon_plugins`、`examples`、`zircon_editor` 的TOML候选未找到含root_node的生产源；[runtime测试](../../../zircon_plugins/ai/runtime/src/tests/behavior_tree_compile.rs) 中有v1字符串fixture，但未复制为产品资产，也不代替实际项目生成/导入准入。

### 四个既有操作的typed输入与禁用条件

复用 [EditorOperationInvocation](../../../zircon_editor/src/core/editor_operation.rs)、[OperationCommandFactory](../../../zircon_editor/src/core/editing/operation/factory.rs) 和原 `ai.behavior_tree.{import,open,validate,compile}.v1` schema ID。当前schema ID没有对应AI decoder/factory；下表是所需语义输入，不声称其已成为wire schema。未知字段、身份或旧revision必须拒绝，不能由客户端提交“已准入/已保存”布尔量建立权限。宿主opaque句柄留在进程内；新浏览器payload必须保真u64，不静默修改既有AssetActivationIntent编码。

| 现有operation | 最小typed输入与真实效果 | 当前禁用/未就绪条件 |
| --- | --- | --- |
| `ai.behavior_tree.import` | 宿主解析的来源locator/原字节/摘要、目标项目/locator/资产类型、存在源的baseline或确认不存在、取消及实际import/publication回执 | 没有AI factory/decoder、目标writer/source/resource准入或真实importer发布；不生成演示资产冒充导入 |
| `ai.behavior_tree.open` | 原AssetActivationIntent及活动项目解析结果、源baseline、宿主分配的document/toolkit/view身份、同资产复用和焦点决定 | 只有route metadata或OpenView，尚无AI源载入/文档注册，不能宣称可编辑打开 |
| `ai.behavior_tree.validate` | 确切live DocumentId、expected文档revision、原始源身份、当前候选与引用闭包、同revision诊断目标及Job准入 | 无同文档typed factory或source/dependency准入；不能把旧文档结果贴到新revision |
| `ai.behavior_tree.compile` | 同上，保持完整TOML→manager验证→catalog编译链及真实结果/错误 | 无实际factory/Job；编译不安装运行时、不执行、不清dirty或保存 |

现有 [OperationCommand](../../../zircon_editor/src/core/editing/operation/command.rs) 装载EditCommand和HistoryContextId，不能把“打开表面”或只读诊断包装成无效果的可撤销编辑。Parent需先交接实际执行/结果通道；本域不增加返回成功的空factory。

### 同一文档的事务、测量与保存

| 输入边界 | 复用的真实合同及AI可交付值 | 尚须宿主交付、缺失时拒绝的条件 |
| --- | --- | --- |
| 文档事务 | [DocumentToolkitDescriptor](../../../zircon_editor/src/core/extension/toolkit/descriptor.rs) 的DocumentId与 `HistoryContextId::Document`；`EditOperationTarget::EditDocument`；现有 `GraphEditContext/GraphDeltaCommand` | 同文档模型lookup、live revision/CAS、完整source+compiled candidate/forward+inverse/selection保留量及峰值证明，再一次permit→prepare→apply；没有生产AI GraphModel/retained owner，不用source字节长度代替完整预算 |
| 单次连接 | 本域一次Reparent、移除旧边后的index、整树编译；真实wrapper与有序children保持 | 不在无预算hover中先构造候选；原子发布、一次undo/redo和selection/focus还缺真实文档事务，不能拆成Disconnect/Connect两笔 |
| 几何与输入 | 原 `AiBehaviorTreeGraphGeometry`/`AiBehaviorTreeEdgeRouteRequest` 接收graph-local bounds/pin anchors及真实边；共享 `GraphCanvasState::fit_graph_bounds` 已提供同一相机的数学fit，源码身份见接续25 | 同DocumentId/source revision/ViewInstanceId、原retained node/pin/wrapper/attachment的instancePath、同layout/viewport/transform generation、真实未裁切文字/节点边界、字体/viewport/DPI、同一paint/arrow/hit路径及焦点返回仍由host证明；document revision和纯fit返回值不能建立这些权限 |
| 保存 | AI仅提供完整引用/编译验证、经准入的canonical v1字节与域诊断；原始import字节/摘要须另外保留，不能用 `to_toml()` 的规范化结果当原始CAS baseline | 宿主注册DocumentToolkit四个hook、源路径/规范化写入租约、DirtyRegistry snapshot及opaque HistorySaveToken；走 `submit_document_toolkit_save` 的Job，不在UI回调同步等待或自行写盘 |

[EditorUiHost](../../../zircon_editor/src/ui/host/editor_ui_host.rs) 及其register/submit是宿主内部入口，`HostDocumentToolkit`使用宿主函数指针；[源writer](../../../zircon_editor/src/core/extension/toolkit/save/source_write_authority.rs) 亦为crate内私有。AI插件不能直接实现这条私有host绑定或伪造source-write receipt。请Parent在既有边界内接入具体AI hook；本包没有要求广泛public re-export、另建事务引擎或另一套writer。

保存时先捕获同文档dirty snapshot/token，真实引用验证和源CAS发布后才由原 [DirtyRegistry](../../../zircon_editor/src/core/asset/dirty/registry.rs) 执行 `mark_saved_if_unchanged/clear_saved_external_effects`。SourceChanged/未发布保持dirty；PublishedNotDurable或已提交但完成冲突保留真实发布事实和诊断，不能声称回滚/clean成功。[DocumentSaveReport](../../../zircon_editor/src/core/extension/toolkit/save/report.rs) 的字节计数或best-effort保证不单独证明文档已清dirty、跨文件原子保存或EXE反馈。

创建/属性/删除还缺同一文档事务；Auto Arrange/Comment还缺真实schema及undo/save；黑板跨文档rename/type/delete还缺键身份和引用闭包。原型搜索、Esc、方向键、左右键规则及焦点失败继续open；640/900裁切保留。周期Service、执行高亮、trace/step必须有真实session/world/entity/source-revision及PIE/trace ACK，EQS/StateTree缺后端不借Nav tab填补。以上禁用矩阵是输入合同，尚未声称产品已经实现禁用。

本轮为文档/只读结构核查，不新增Rust定义或运行测试，不碰锁或重投旧身份。combined-r2 resolver事务/资源准入、官方codec→原ZUI→正常EXE、真实150%DPI/逐图双端≤1DIP仍pending，passed/accepted=false。

## d287有限入口与联合锁边界14

已按 [本域接续14](ai-parent-handoff.md#d287入口交付只读接续14) 核对原owner新result/summary摘要、实际selected d287胶囊及11份pin；AI四filter保留，粒子受限source_tests::属于该域，不合并测试身份。当前十二AI源仍为b00543e0的24+4共28未运行定义，8依赖及5保护lock/manifest匹配；原11项输入不能验证新候选。未重复纯配方测试、QueryOnly或执行Cargo，本轮明确不提交AI测试/生成。

联合锁仍缺四边，generate只有source-index/render-template；受限定向resolver、完整diff/CAS发布/恢复事务、sole canonical lock writer及source/resource/cache/pool准入都不能由新route或管理通过替代。captureReady=false、不freeze、不手改lock或取消locked。原owner粒子源漂移、管理失败/成功、PR51与资源OS观察按其时间和归属保留，本域不改其源或lineage。输入包13、真实backend/PIE trace ACK、正式导出和正常EXE/150%DPI验收仍open，passed/accepted=false。

## 正式导出消费与窄窗测量接续15

当前插件入口以本域实际16:58:43Z/17:01:42Z读回为准：19002 manifest/bundle HTTP200，bundle485232B/SHA442bed70…与原owner本地served-dist逐字节摘要一致；原4213拒绝连接是历史失败。原built runner和worker-r3回执存在/匹配不等于兼容执行。已按公共协议向原owner交原AI file/page、private源及只追加验证板的消费请求，尚无正式import/save-close-reopen/export。原业务板的devZirconAiDesign身份不能代替devZirconZui源owner/guard，不改现有有效源或重签元数据来使其“可导出”。

三原BT板当前graph宽895/629/447，三task union650；900左右各裁10.5、640各裁101.5，640另缺debug selector/overflow。以上是实际Penpot shape几何，不能拿它设原生DPI或runtime hit box。交graph owner消费原pan/zoom或提供最小共享frame-all/source-focus合同；同代际retained bounds/viewport/transform、原实例路径、paint/hit/focus仍按输入包13，不能按截图移动节点、删操作或造AI相机关闭缺口。完整 [设计接续与消息身份](ai-penpot-design-r01.md#当前发行物与窄窗实际读回15) 保存原失败和当前边界。十二源/八依赖/四ZUI未改，28测试未运行，official exports=[]、正常EXE/150%DPI/≤1DIP未通过，passed/accepted=false。

## 四原资产的导出保留项与字体身份17

本轮只读解析四个原ZUI及其显式imports，按实际asset.id/组件fragment查找；全部可在现checkout找到唯一匹配，合并15份源，未创建投影、复制源码、运行canonical preparer或扩大writer。完整节点/children/slot/组件导出及逐事件字段、原字节SHA见[源依赖与保留项](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/ai-source-closure-17.json)。这里没有把“显式依赖齐全”提升为动态宿主、隐式图标/媒体或正式codec闭包。

| 原资产 | 节点 / control IDs | 原events | 显式依赖含自身 | 正式导出须保持 |
| --- | --- | --- | --- | --- |
| `zircon_plugins/ai/editor/behavior_tree.zui` | 12 / 12 | 0 | 4 | 12个控件身份，原Table/Space/分区，不补无效果事件 |
| `zircon_plugins/ai/editor/perception_debug.zui` | 3 / 3 | 0 | 4 | 原agents Table及entity行身份，动态数据仍需真实host |
| `zircon_editor/assets/ui/editor/workbench/modules/core/ai/behavior_workspace.zui` | 28 / 26 | 12 | 10 | 原匿名节点/挂载、WorkbenchBehavior控件及12条事件/route |
| `zircon_editor/assets/ui/editor/modules/core/ai/perception_workspace.zui` | 28 / 26 | 12 | 10 | 原value slots、WorkbenchPerception控件及12条事件/route |

私有两资产的样式链均为 `editor_base → editor_material → editor_tokens`；工作台两个资产复用同八种公共widget与tokens。全部原节点没有显式binding字段，这不构成真实数据绑定证据，也不能把24条既有event声明当作已实现handler。首个消费请求仍只以private behavior_tree为root、该4份源为显式依赖，在原file/page追加独立验证板、replaceSelection=false/createComponentMaster=false；四根盘点不扩大当前源写权，不改13业务cases/4mains/64copies或Shared。

字体核对进一步区分独立TTF、引擎TTC face及实际Penpot注册资源。`editor-ui.font.toml`/13,353,468B的`editor-ui.ttc`（SHA `03564700…`）含12faces，二进制name/OS2 weight均与原manifest一致。rev28的430个text中425个声明Roboto、5个Noto Sans SC。17:29:13Z真实只读get-font-variants仍4条；Roboto400/700及Noto400六份TTF/WOFF资源ID/字节SHA与11:22Z原实测相同，解压WOFF并仅规范化head.checkSumAdjustment后，全部表与TTC face8/10/7逐表相同。仓库独立TTF与上传TTF摘要不同，不能据此更换字体；对应实际TTC face才是本次比较对象。见[实际字体API与表比对](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/ai-source-closure-17-font-api.json)。

本轮没有字体上传、字体二进制副本或浏览器测量重放；只在内存读取既有资源，API context已dispose，认证原文件SHA不变。复用11:22Z六份FontFace/canvas证据，不抹去Penpot WASM行框历史失败。当前Penpot/native实际loaded face、fallback、shaping、字形行框及双端≤1DIP仍需正常产品捕获；字体资源表相同不证明这些门通过。原图/详情/节点创建/事务/save/execute、Service/trace、EQS/StateTree、联合锁生成/资源准入和正式codec仍open，28定义0执行、passed/accepted/captureReady=false。

## 黑板语义引用与改名候选18

接Parent第五次交接，AI-BB01..07仍是业务合同；本轮在本域[blackboard子模块](../../../zircon_plugins/ai/editor/src/authoring/blackboard/mod.rs)实现对**调用方提供的不可变快照**进行语义扫描和改名候选构造。尚无真实catalog引用枚举、持久树—schema关联读取器、独立schema资产格式或跨文档发布接线，UI改名/类型变更/删除继续受这些缺口限制；未把候选API当成引用索引或产品操作完成。

输入明确区分原始源的UUID/URI/revision/SHA256、树和schema业务ID、关联记录自身的owner源版本及字段路径、提供者registry生命周期/revision/module ID，以及依赖catalog身份/revision/显式树UUID集合/未解析资源。字段路径由真实关联owner提供，没有规定虚构生产metadata。扫描保留缺失、重复、额外、stale、未知提供者/语义、schema未绑定键及opaque节点阻塞项；`None`枚举和有阻塞的空引用都不能成为“零引用”。显式关联另一个schema UUID的树不因同业务ID/同键名被改写。

扫描使用完整编译结果的节点ID寻找实际标准实现，不假设作者数组与编译遍历顺序相同，再核对提供者和目录中的完整节点描述符。只收集真实`blackboard_key`参数：BlackboardCondition为条件及观察者槽位，其他Decorator为槽位，SetBlackboard为写入；标题、`equals_string`、事件/载荷字面值和脚本内容不按字符串替换。Script/External、未解析RunSubtree目标及未完整实现的Service保持阻塞，不造执行反馈。原Decorator wrapper/children/abort policy及所有无关参数保持。

`rename_key`只返回schema前后值、每棵树一次完整编译后的候选、精确原参数对应的forward/inverse和全部源/关联/catalog前置条件。schema校验复用新建、隔离的DefaultAiManager注册入口，随后丢弃句柄；不取得live manager或增加replace/unregister。真实schema键没有统一256字节上限；SetBlackboard仍通过原完整`compile_behavior_tree_toml`链拒绝超过256 UTF-8 bytes的effect key。未知、只读、碰撞、stale或校验失败不改变输入。资源预算须在构造前准入，源hash是原始字节摘要，不能替换成候选canonical TOML摘要。

Parent继续负责完整依赖闭包、真实operation factory、文档revision/源CAS、单次跨文档事务/history/save token与资源writer准入；本域正反向编辑不能逐文档提交来假装跨文件原子持久化。`SaveDirtyViews.references_valid`仍只是调用方断言，批量准入和逐文档保存不构成该事务。运行黑板只读，保存schema不等于现有tick句柄热更新。

源码核查同时发现旧`effect_tree`夹具将SetBlackboard参数写作`key`，真实校验器要求`blackboard_key`。已修正并新增旧拼法经完整编译链被拒绝的回归，未弱化校验；此前28项只是定义且未执行，不能称其曾通过。本轮新增9项语义回归及该1项夹具回归，当前34 authoring+4 gameplay=38静态定义，0 Cargo发现/编译/执行。静态检查和新源lineage见[候选18记录](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/bb-reference-plan-18-source.json)，旧b005/28、原validate/create、失败及锁四边计划保持原样；不提交新测试或生成、不freeze。正常EXE/150%DPI/双端逐图、正式codec、保存/执行和全部原视觉差异仍pending，passed/accepted/captureReady=false。

## 受管锁交付的当前门24

原owner已交五阶段 `prepare/admit/resolve/publish/recover` 管理实现；Jenkins #416的110 tests+317 subtests只资格化其原管理范围。原 `jm24/combined-ai-particles-lock-resolution-r2/prepare-lock-resolution-r2` 沿相同输入取得 #360 和真实 generationRequestId，preparation摘要 `fdc33cc1…`、157 manifest/1075解析输入按原回执复用；这些是resolution metadata，不是产品完整源码捕获。cargo 1.94.1的root0/Job0/双EOF只对应 `--version`。

#361实际资源回执 `admitted=false`：policy v11，E盘当时15137222656B，reserve37580963840B；独立resolver估算1 CPU/2GiB RAM/256MiB disk，不是实测峰值。reservation和sole-lock writer均为空；#417撤销本次未启动检查的等待行，原blocked generation准备保留。受信stable>=1.97的独立锁输出能力、实际资源/writer和最新解析输入仍待准入，不降阈值、借源码树或清共享缓存。

未运行resolver/update，实际完整diff/after SHA/发布/生成恢复均为空；嵌套锁04de41d2…及四边计划不改。后续实际capture-ready才由AI提供当时真实候选及合法source/resource context。原owner报告24项作为历史快照保留，不能测试当前17源/38项；原validate/create输入不重放、不补attempt，测试locked不解除。单指针receiver发布与11 job API/Home一致不覆盖外部full-source policy归属/验收，更不授予AI host/history/save或正常EXE权限。完整[接续24](ai-parent-handoff.md#联合锁资源阶段实际回执24)保持所有历史失败，passed/accepted/captureReady=false。

## AI-C06的宿主消费输入25

已核对共享owner的[正式来源回执](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/ai-bounds-fit-contract-07.json) `08028254…`；其canvas/mod/cases分别为`a072722f…`、`0ed32746…`、`9f7e906a…`，与本域候选18已登记依赖相同。本轮只确认来源并补足消费合同，不新增AI wrapper、第二camera/clock或共享实现。实际API是[原CanvasState](../../../zircon_editor/src/ui/graph/canvas.rs)的 `fit_graph_bounds(GraphNodeBounds, GraphPoint, f32) -> Option<GraphBoundsFitResult>`；同一state的pan/zoom一次更新，padding是四侧DIP inset，helper不乘DPI，也不验证host generation。

以下是接续13所缺的真实宿主输入合同，尚无生产provider/factory/运行回执；语义意图名称不冒充新注册command ID或wire DTO：

| 阶段 | 所需真实输入与消费方式 | 缺失或失效处理 |
| --- | --- | --- |
| 意图与焦点 | 原host分派的全部图、当前选择或源定位意图；明确DocumentId、ViewInstanceId/retained incarnation、资产UUID/URI/hash、doc revision、选中语义ID或源node ID。输入须属于该pane，IME、popup和菜单焦点按原owner路由 | 缺真实host/资产关联/源节点映射则不执行；不按标题、Penpot copy ID或全局focus猜目标，不新增成功占位handler |
| 完整测量 | 同一source/selection/layout/viewport/transform generation的未裁切graph-local extent；包含应显示的真实node/text/pin/wrapper及越出节点的实际paint范围，离屏内容不能因clip或虚拟化被漏掉。源定位只消费真实sourceRef→instancePath关联 | `AiBehaviorTreeGraphGeometry`的节点矩形map与document revision不足以证明完整extent；不从650 fixture、截图坐标或少量可见卡片合成生产bounds。测量未就绪沿现有layout-ready生命周期等待或拒绝 |
| viewport与单位 | 同一pane实测viewport及clip、真实DPI/transform来源；物理输入与尺寸只转换一次，调用时bounds为graph-local、viewport/padding为逻辑屏幕单位 | 不传Penpot绝对位置，不再乘DPI。padding由Parent确认实际命令/主题策略，不能将UE的25图单位静默当作25DIP或将owner的零padding算式当UI默认 |
| 调用前再核对 | Parent现有retained owner核对上述代次及资源准入，持同一CanvasState；resize/DPI、doc close/reopen、selection/source变更、gesture/pan/wheel使原请求失效，取消或在新代次重新测量 | 不跨pane复用请求，不建立第二计时器/后台相机，不改变节点源坐标、选择或作者history；helper保留drag对象并不代表宿主可以在旧drag代次执行fit |
| 结果与绘制 | `None`保持原state；`Some`分别消费changed与fully_visible。changed=true后由原host发布transform generation，实际paint、edge/arrow、hit/clip使用同一新transform；源焦点由同一instancePath恢复 | 最小zoom仍装不下或精度不足时fully_visible=false，保留可见缺口；changed=false不创建作者事务。纯返回值、截图缩放或新generation标签不证明真实paint/hit/焦点已接线 |

尚需Parent交付实际输入来源和帧回执，不能由AI填入示例UUID/revision来绕过。共享helper与UE离散fit最大默认档、25图单位padding及deferred/interpolated/cancel策略有明确差异，本轮不增加动画或延时实现。Decorator保持原wrapper，Service/trace不因frame能力获得运行反馈。640 toolbar debug selector/overflow另由Parent统筹，旧900/640裁切、所有DPI/逐图验收继续open。

共享`ui::graph::tests::`的26项（19旧+7新）和AI的38项属于不同package/filter，均未本轮Rust发现/编译/执行，不合并成已跑矩阵。原core #353、material #352及AI旧验证输入不重放；三份共享变化相对其各自旧identity的冲突按owner原记录保留。完整[本域消费核对25](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/graph-bounds-consumer-25.json)只证明依赖身份和合同，正常EXE/150%DPI/正式codec/双端≤1DIP仍pending，passed/accepted/captureReady=false。
## 原始Vampire BT兼容性与关联producer核查27

接Parent AI06，只读核查原[enemy_behavior_tree.toml](../../../examples/vampire/assets/data/enemy_behavior_tree.toml)、sidecar、registry及实际脚本/注册源。原UUID `2f7f0e40-bc1b-4ee0-9a8b-5a1590ed5f31`、URI及原始SHA256 `fb2b913c9cc03a5b67be654e1749c24e98b913c3164b228d4999ad2767801b4e`保持；sidecar的16hex source_digest、TOML version=1、registry format_version均不是运行revision。15节点/14边、ID唯一、全部从root可达是静态数据核查，未执行Rust解析/compile或运行树。

| 原始内容 | 当前合同与未解决语义 |
| --- | --- |
| `version/root/description/blackboard` | [DTO](../../../zircon_runtime/src/core/framework/ai/behavior_tree.rs)的必填顶层为format_version/id/display_name/root_node/nodes，当前缺format_version/display_name/root_node；不能只改字段名便称迁移成功 |
| 每节点id/kind及扁平字段 | id/kind/implementation/display_name必填；children/parameters/abort_policy有serde默认。原15节点均无implementation/display_name；缺省parameters不使原扁平action/result/expected/operator/value自动成为参数，[作者入口](../../../zircon_plugins/ai/editor/src/authoring/asset.rs)还拒绝未知字段 |
| 五个condition叶 | 当前kind枚举没有condition。标准blackboard_condition是Decorator且须一个child，原条件叶均零child；保留ID、顺序、短路、abort语义的迁移方案尚未决策，不能直接把kind改为decorator或造空child |
| 六个task的action/result | 六个action字符串均不是当前标准Task实现ID；脚本动作、导航调用、结果/取消/副作用需要真实provider合同。不能改成wait、script_task或直接认定result=running正在执行 |
| 六项blackboard默认值 | 不是AiBlackboardSchemaDescriptor的id/display_name/keys及key/value_type/required；target_entity=2不能证明entity类型或live EntityId。五个条件读键中的target_distance/attack_cooldown没有本文件默认值，其实际生产/更新来源待提供；这不证明运行时缺键或允许补假值 |

实际中性导入链是[importer注册](../../../zircon_runtime/src/asset/importer/ingest/asset_importer.rs)的 `zircon.builtin.data.toml` → [import_plain_toml_data](../../../zircon_runtime/src/asset/importer/ingest/import_data_asset.rs)：保留原文并生成canonical JSON的DataAsset，不调用AI compiler/schema注册。registry的dependencies=[]只说明当前登记边；其语义覆盖未证明，不能推断零引用或完整关联树集。

[场景](../../../examples/vampire/assets/scenes/main.scene.toml)有六个 `vampire_game/main` binding携带业务字符串graveyard_enemy_bt：entity20/22/24启用、21/23/25禁用，均非本轮live实体身份。实际[main.zr](../../../examples/vampire/scripts/vampire_game/main.zr)通过role分支调用伤害/导航并写vampire.action_state与vampire.behavior_node数值；其源内未找到读取behavior_tree属性/本TOML、schema注册或黑板键生产的调用。数值31/41不是原树节点ID，不映射为该树执行高亮。这里只定位源码producer；已存在zro字节、README或测试名称都不证明本轮实例正在运行这条路径。

所查`examples/vampire`及Runtime dynamic_api/ZrVM生产源的精确ID/URI/属性搜索未定位到从此Data到AiManager树/schema注册的链路；这是有界搜索的未证明状态，不是全仓无consumer断言。现有测试有场景业务字符串断言；旧runtime chase测试带ignore，断言脚本component/移动，不能代替资产compile或当前运行回执，均未执行。

[标准provider注册](../../../zircon_plugins/ai/runtime/src/plugin/registration.rs)实际取得module.owner并绑定标准节点，撤销监听仍保留；离线standard_node_catalog使用bootstrap owner，不是该文档的live registry身份。必须由真实producer提供同生命周期catalog/owner/revision与所需动作provider。真实schema资产及hash/revision、持久关联owner/field_path、完整语义coverage仍未获得，因此不能构造可提交的Schema+Tree输入，更不调用BB18候选充当一次真实改名。

资源owner继续原RT-AIREF01-G01的publish gate→state→index一致只读scope，释放锁后交AI；本域不代写中立适配器或把其直接边当AI语义闭包。Parent仍持asset→document/history/budget/CAS/跨文档save及retained host。此轮未出现新live producer/导出/执行回执，不重复Parent数字ACK或原请求；更新[本域27证据](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/vampire-bt-compatibility-27.json)及原manifest。候选18、原四ZUI/12control IDs/0events与最后真实设计rev37保持；无迁移/源修改/新schema UUID/Rust/QueryOnly/生成/freeze，passed/accepted/captureReady=false。
## 原资源G01接口的AI消费合同29

原资源owner已经交付 `EditorAssetManager::readonly_asset_snapshot(&AssetRef, Option<&EditorAssetReadScope>, Option<AssetSourceWritePolicy>) -> EditorAssetReadSnapshot` 的源码候选；本域此前按AI命名查找未命中，不能继续据此概括为“没有adapter”。[trait/default](../../../zircon_editor/src/ui/host/assets/api.rs)、[Default bridge](../../../zircon_editor/src/ui/host/assets/manager/assets/preview_trait_bridge.rs)、[原query](../../../zircon_editor/src/ui/host/assets/manager/assets/readonly_query.rs)和[snapshot类型](../../../zircon_editor/src/ui/host/assets/manager/assets/readonly_snapshot.rs)已直接阅读。本域不复制或改写这七份公共源码，原12项回归尚未运行，源码候选不代表真实host已经调用。

| 实际返回/字段 | AI侧消费边界 |
| --- | --- |
| Unknown(UnsupportedManager/NoActiveProject/IndexUnavailable/InputUnavailable) | 保留具体未知原因和requested，不生成空schema、空依赖或可提交动作；custom manager默认Unknown不是成功读回 |
| Missing(GuidNotRegistered/GuidSubMismatch) | 保留精确GUID/sub与注册sub诊断，不按hint、同名URI或业务ID替换选中对象；未登记资产不能成为已知零引用 |
| Stale及optional expected | 取消当前源观察/候选发布，由真实host重新取得同一文档的有效输入；不套旧scope重试保存，不改原live文档 |
| Known及scope | Known只证明原registry中精确GUID/sub的内存观察。asset/details仍可缺失；runtime kind=Data不建立AI semantic type或Toolkit。须保留原scope，不通过自身构造/序列化补造它 |
| scope.same_publication | 比较真实activation、input Arc、catalog Arc身份；preview-only publication也会失效。同sequence/根路径/业务ID不替代同publication。query返回已释放manager锁，但后续IO/异步完成/提交仍须host重查当前publication与document incarnation/关闭状态 |
| incoming/outgoing及端点 | 复用原registry全边，Known/Missing端点逐项保留，不从UI行反推引用集。asset_edge_coverage和semantic_coverage当前枚举只有Unknown；不能据空边构造AiBlackboardDependencies的“完整树集合”，更不关闭BB18 blockers |
| source_policy_access | None表示调用方未给策略；Some只计算实际类型policy与locator。它不是OS读写、源锁、源字节校验、AI类型正确或save/CAS/跨文档事务准入；本域不填一个假可写policy启用操作 |
| 原source/version/provider | API不提供原始字节SHA256、asset/document revision、schema UUID/URI、持久关联owner/field_path、provider registry生命周期或完整语义coverage。registry metadata digest、catalog sequence、Arc地址、TOML version都不填入这些缺失字段 |

具体生产顺序沿既有分工：Parent提供真实AssetRef、当前host/外部锁已释放的调用时机与实际source policy；原manager按publish_gate→state→index返回owned snapshot。随后由原source host在锁外读取原始bytes、建立真实source版本/文档关联，并在返回或提交前复核expected publication及同文档authority；AI仅在这些真实前置条件齐备后消费原BB18 schema/tree/provider语义。资源query不会替AI判断schema关系，AI也不把此API包装成新索引/store、源读取器或保存入口。

原Vampire样例仍是Data旧格式，G01源码存在没有使其通过BT DTO、生成schema或成为当前可提交输入。当前覆盖枚举仍Unknown，G02真实asset/type/document关联、G03完整生产覆盖与语义闭包、G04 history/CAS/atomic save以及源码/资源/锁与Rust执行门继续open。七个owner当前source hash、原patch/12未运行定义及本域审阅见[消费记录29](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/resource-readonly-consumer-29.json)。旧[28号负向搜索](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/goal-readiness-28.json)原样保留，29纠正其搜索充分性；没有新live查询、Schema+Tree构造、handler/UI、public写入或产品验收。


## 菜单作者语义53：真实选择、目录搜索与单候选

本轮在原 AI editor owned 路径实现 `AiBehaviorTreeMenuAdapter`，接续候选19的 `AiBehaviorTreeEditCandidate`，形成候选20。候选20登记21份本域输入，新增10项菜单回归定义；当前静态计数为48 authoring + 4 gameplay = 52，实际 Rust 发现/编译/执行仍为0。21份是作者候选 pin 集，不是完整 Cargo/source/member 解析或资源准入；其他会话和 runtime_mirror 的后续成员须由受管 source preparation 一并枚举，不用这组 pin 代签完整闭包。

适配器只借用原 `AiBehaviorTreeAsset`、当前选择、宿主只读状态与原 `AiBlackboardProviderSnapshot`。`rows(query)` 的 text/value 面向既有 WorkbenchListRow，`node_rows(command, query)` 直接借真实 frozen catalog 的 descriptor，面向既有 WorkbenchSearchInput.query；没有新 Field/ListRow/Menu painter、主题、后端、store 或 clock。搜索按空白分词、所有词匹配标签或实现ID、ASCII大小写不敏感；中文按真实UTF-8内容匹配，没有宣称完整UE模糊搜索、编辑输入/IME或本地化标签已实现。

| 操作/上下文 | 当前领域判定与结果 |
| --- | --- |
| 无选择、多选、失效ID、只读 | 分别返回 NoSelection / MultipleSelection / MissingNode / ReadOnly；未把未知选择当空数据或对多选循环提交。 |
| 创建子节点 | 仅当前 Composite 可尝试；11个实际 composite/task catalog 行，完整typed node及index由原宿主提供，不生成asset UUID或参数默认值。Run Subtree必须 `Subtree` / `run_subtree` / 原 `behavior_tree` 注册ID。 |
| 添加/移除Decorator | 六个实际Decorator目录行；使用原 wrapper 拓扑，根包装仍可逆；不绘制UE attachment假投影。单子节点Decorator不能直接删掉必需子节点或追加第二子节点。 |
| 删除子树 | 根删除拒绝；一次 `DeleteSubtree` -> 原完整编译链 -> 一个before/after候选，保留原节点ID、原TOML与次序的inverse。 |
| Random Selector权重 | 复用原拓扑guard的同一只读权重谓词 `has_explicit_child_weights`，创建/删除/包装/解包会改变带权子节点身份时拒绝；未偷偷重映射ID或位置权重。 |
| provider不明/无生命周期/替换语义 | 未知provider、缺registry identity、外部factory或内置ID被不同semantics/category占用均拒绝typed创建；分类标签不能充当创建kind或参数schema。 |
| Add Service | 明确 ServiceAttachmentUnsupported；现零child Service和默认Succeeded不作为UE周期服务执行。 |
| Toggle Breakpoint / Step Into / Step Over | 明确 RuntimeTraceControlUnsupported，不生成回放ACK、执行高亮或成功handler。真实既有mirror只读能力未被替换。 |

`eligibility=Ok` 只允许尝试准备typed候选，不是文档写入许可。`prepare` 在调用方预算准入后只调用一次原 `asset.prepare_edit`；仍经 `compile_behavior_tree_toml` 的manager validation和完整编译，不切换为较弱descriptor-only编译。它没有DocumentId/revision CAS、permit/history、文件保存或runtime权限。宿主须用原真实asset URI/UUID/hash/revision、同view/incarnation/selection/provider lifecycle核对，走原Document一次prepare/apply/history/CAS；双文件Blackboard改名继续原语义计划与Parent跨资产提交准入，不用循环save替代。

本轮沿用[监督恢复52](ai-penpot-design-r01.md#监督恢复52原640板空状态与真实保存重开)的真实设计目标和已留原图；没有新的Penpot API保存、官方导出或EXE图，不将原rev47/863升级为当前live观察。原菜单Search prototype输入、Esc/方向键/焦点返回失败和640两任务裁切仍open。既有空白画布11项设计仍为UE目标；本adapter尚需真实选择/连接目的地，不能在完整有效树内创建未连接孤儿节点。多选批量删除、复制/粘贴、断链、自动布局、评论、资产跳转factory、实际debug/trace/PIE ACK继续open。

10项新回归覆盖真实catalog引用及11/6条目录、多词/中文/无匹配、Subtree kind/参数、错误typed参数/索引/重复ID/已有child、选择与只读、Decorator根包装/解包/删除inverse、带权拓扑、provider替换和明确unsupported；仅定义，未运行。原 `ueai24-bt19 / validate-authoring-inverse-52` 输入与pending日志不改，新源不能塞回这笔或旧bt01/create/jm24身份。通过原Jenkins owner正常后继收件后才用本次完整current revision及实际source/resource/cache/pool做有限 `authoring_tests::` 单包验证，未要求全局freeze或用户手动grant。检查与完整pin/diff见[53报告](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/menu-eligibility-53.json)。passed/accepted/captureReady仍false，goal保持active。


## 黑板菜单54：创建候选、语义改名与引用未知边界

接续候选20的BT菜单，本轮在原owned路径实现 `AiBlackboardMenuAdapter` 与私有before/after的 `AiBlackboardSchemaEditCandidate`，候选21登记26份作者pin。新增9项黑板菜单回归定义；当前 `authoring_tests::` 静态57项（原根19、graph10、BB reference9、BT menu10、BB menu9），gameplay4另列，总61；实际发现尚无回执，已验证执行0。Parent已核恢复52的17pin/38 authoring+4 gameplay以及候选20/52定义均保留各自时点，不拿旧输入测试本轮新源码，不再QueryOnly或重投。

| 命令/数据 | 实际领域合同 |
| --- | --- |
| New Key | 原宿主供给完整 `AiBlackboardKeyDescriptor` 与插入index；保持名称、value_type原alias、required和key次序，沿原fresh isolated manager执行同一真实schema validator；源和候选均须合法。没有默认值字段就不补0/空值，不产生asset UUID。 |
| Rename Key | 原 `AiBlackboardReferencePlan.rename_key` 的语义重写、source/association/provider/dependency前置条件和业务inverse原样复用。无引用闭包/关联/provider证据时拒绝；受影响树只读拒绝，未受影响树不伪造写入。no-op返回None。 |
| Key搜索与选择 | 原schema descriptor借用行，SearchInput.query复用BT同一内部无分配过滤函数；ListRow key/type/selected可投影，不建菜单store或索引。中文/类型别名/多词/ASCII大小写有定义。 |
| 引用状态 | 有任何原plan blocker就返回 `Unknown(blockers)`；否则只返回 `InSuppliedScope(count)`。该count不是全局引用索引、agent/默认值/脚本producer覆盖证明，零数不授权破坏性操作。 |
| Change Type / Delete Key | 引用未知返回UnknownReferences；已知本scope有引用返回KeyReferenced；scope零引用仍SchemaEvolutionUnsupported。缺完整producer/migration和持久化/注册生命周期合同，继续open。 |
| Edit Runtime Value | 一律RuntimeValuesReadOnly，复用真实mirror只读数据；不新建运行值、热替换或伪ACK。 |

新建key只准备schema作者值，既不重写原键，也不宣称未知引用集完整，因此缺依赖枚举时仍可构造合法schema候选；**这不是发布许可**。required/default的真实input producer兼容性、运行注册句柄/活动lease、原source/document/revision/hash、资源预算与原单次CAS/history仍由宿主验证。没有这些实际上下文不得把row.disabled=None变成可执行成功按钮。创建候选持原source version及完整before/after；值相等不能防ABA，原lease/revision/CAS必需。候选无clock/save/runtime权限，不替换AiManager中的schema，不新增replace/unregister API。创建/改名的whole-value no-op/错误不写历史；跨文档改名不得逐文件早保存或用SaveDirtyViews默认references_valid代替原子提交。

UE依据为 `SBehaviorTreeBlackboardEditor.cpp:143–174/324–338/389–403/453–535`：非debug与模式/继承键条件、单FScopedTransaction、Modify、刷新、选中新键和请求改名。本轮尚未复刻UE的自动名/同名后缀、继承键/父schema、popup关闭、键盘/IME和选中后请求改名；原runtime DTO不表达继承或默认值，不能从文案猜字段。共享menu仍LevelEditor-only且request尚缺document/revision，展示menu_items与Runtime options/search_query合同由Parent原scope接，本轮没有去guard、改target文案或第二store。

Parent恢复52的原BT候选消费合同同步适用于这里：业务before/after相等不是ABA防线；document/source/revision/lease、预算准入及原一次CAS/history才是宿主发布边界。原rev47/863与status `3ae54be5-f0f4-4daf-839a-3f3a276da8a7` 继续由原public owner保护接官方流程，投递不等于构建/导出/产品验收。本轮无Penpot写/新图、无共享源或原四ZUI/锁写，无查询/重投/新可执行validation身份或全局freeze。

新增回归覆盖创建六真实type/长UTF-8 key不套effect256字节上限、type alias/required/原次序/source/inverse、重复/空键/未知type/原schema非法/越界/只读、rename no-op/语义inverse、受影响只读树、原借用行及未知引用不当零、类型/删除/runtime明确拒绝、选择失效和ABA边界。均是测试定义，不是Rust执行。原schema validator函数只返回原AiManagerError供创建与改名共同复用，改名仍映射原InvalidSchema错误；BT查询函数提到域内search.rs，两菜单复用同一实现，无公共API/主题变化。完整[54源码与检查报告](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-menu-54.json)。goal active，passed/accepted/captureReady=false。
