# 角色与骨骼：已分配身份及共享组件消费合同

当前接续第38节 Physics r05：为现有RagdollProfile补受校验约束的TOML写出接口，新增9项编解码回归定义，0执行；当前Physics源17项。r04骨骼typed key/整批候选、Skeleton r05/32项和Penpot rev76保留。共享缺件不阻止域源码工作，goal active；原Document/CAS/history/save、真实预览、正式导出、normal EXE/实际150%DPI与整族验收仍open。 TreeRow公共源码复核、正式导出身份及UTF-8版本链见第39节；compact尚未发布。

glTF托管历史接续见[第21节](character-skeleton-design.md#21-gltf限定入口交付与角色owner三路径审阅)：本轮先消费28698交付，后实际发现同owner的Prefab限定资格使installed推进为20b3bf85…/entry4e95a18e…；原版6 native/archive保留，当前11 sealed匹配，两条角色test纯parse通过，glTF条件及source preparation/guard/wrapper未改。角色owner完成同身份三路径/18哈希和recipe审阅；当前profile=test，第20节dev观察和本轮首版指针漂移均保留历史。仍待原执行owner当前writer/受管资源和Parent有界源码时段后正常准入；capture-ready=false，CPU/preview/GPU/normal EXE未通过，不将公共组件发布或helper管理资格当产品验收。

本文件接续 [窗口族盘点](character-skeleton.md)、[具体设计](character-skeleton-design.md) 和 [Parent 交接](character-skeleton-parent-handoff.md)。Parent 的最新权威记录为 [.codex/sessions/ue-window-suite-parent-01a16d4e.md](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)。本域仍负责 Skeleton/Skeletal Mesh/Socket/Pose/Physics Asset/Retarget/适用 Control Rig 的业务语义，不接管其他域文件。

第20节前轮对账保留历史：原Jenkins owner实读r4且14哈希一致，当时Parent profile=dev、r4提案test，route/三路径envelope/完整source准备未到。当前由顶部及第21节更新为已交付test限定route、三路径输入已审阅，仍待writer/受管资源和Parent时段，capture-ready=false。持续guard到EOF由原执行owner/Parent统一，不把五漂移路径当完整写集或以公共组件发布关闭production preview/EXE门。

## 实际身份与当前阶段

| 身份 | 实际值 | 证据范围 |
| --- | --- | --- |
| Parent | `01a16d4e-0491-7353-84fa-038bd7706546` | 统筹跨域合同、公共主题/框架、catalog、导出与原生证据集成 |
| 本域 | `01a16d52-6b1e-7543-82f0-4f05ca92ad58` | 四份计划/合同文档、独立任务证据及分配的真实 Penpot 文件；公共源未改 |
| Team / Project | `822661f1-7ba3-81f7-8008-c6dcb4ae6a8f` / `b4a1e3ce-9e6a-81fe-8008-d69d606f6160` | 来自 Parent 的真实创建/第二认证上下文回读回执 |
| 本域 file / page | `b4a1e3ce-9e6a-81fe-8008-d69d60adb48e` / `b4a1e3ce-9e6a-81fe-8008-d69d60adb48f` | [03 Character and Skeleton](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60adb48e&page-id=b4a1e3ce-9e6a-81fe-8008-d69d60adb48f) |
| 公共目标 file / page | `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` / `b4a1e3ce-9e6a-81fe-8008-d69d607aac13` | [00 Shared Components](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d607aac12&page-id=b4a1e3ce-9e6a-81fe-8008-d69d607aac13)；URL 不是已发布库或组件版本 |
| 时间轴/曲线/播放 owner | `01a16d51-741b-73b3-a61a-2189cd38f81e` | 复用 [AN-P01/02/03/05](animation-design.md)，不复制 timeline/curve/transport |
| 节点图表面 owner | `01a16d50-6c7f-7370-853d-4581184b66a1` | 复用 [MAT-SHARED-001](material-family.md)，材质仅牵头共享表面；角色域持有 Rig/Physics 图语义 |

初次身份核实时，本域用原 P21 认证 storage 的 cookie **仅在内存中**独立调用真实 `POST /api/rpc/command/get-file`，未创建账户、未输出 cookie。以下保留初次空文件的历史响应，不代表实际设计后的现态：

- 本域 file：`revn=0, vern=0`，目标 page object count=1，components/colors/typographies 各 0。响应 SHA256 `54ed7f877dfba1eeb2aa811bcd90c8ce9ac36901bb3466fdad3ba10b75fbb868`。
- 公共 file：`revn=0, vern=0`，目标 page object count=1，components/colors/typographies 各 0。响应 SHA256 `fe0d8e5c3ff7850da27eade99425f4192fa7f8665047b43a506167ff4afcc734`。

初次消费脚本误按kebab-case找 `is-shared`，已依真实 `isShared` 更正；初次空文件响应保留为历史。该次本域为 **rev22/1538对象/102flow/1组件**，既有 `SkeletonBoneReferenceDetails` component=`c8bdd097-2862-4d7e-926d-887a559bd5ff` / main=`c888ed9b-7d6f-40f9-a87e-f52437abc603` 保持，原3个同文件实例另增1个；新增60个真正跨文件Copy。公共库现rev1/isShared=true/4组件，已通过实际get-file-libraries核对链接，公共文件响应哈希未变。原Retarget及旧317个非根对象未变。[设计第18节](character-skeleton-design.md#18-共享原生实例的完整源骨骼树与head内容设计)列明Copy/源shapeRef、属性覆盖、保存关闭fresh重开与逐图证据。colors/typographies仍0，未证明全局样式/override-reset传播、官方导出、完整UE Toolkit或产品/原生双端验收；`passed/accepted=false`。

当前公共native合同：TreeRow=`6ddc248b-bd35-80c5-8008-d6c8c7a79944` / main=`6ddc248b-bd35-80c5-8008-d6c8c452bf0c`，Button=`6ddc248b-bd35-80c5-8008-d6c8c01096f3`，SearchInput=`6ddc248b-bd35-80c5-8008-d6c8c3ce892e`。PropertyEditorRow=`6ddc248b-bd35-80c5-8008-d6c8cab1e7d0` 当前未用于Euler/Axis；真实slot合同与typed事务未接，不能把只读TRS文字塞进临时props。TreeRow设计仅覆盖source label、12逻辑像素depth、Head选择色、叶chevron及未支持eye/lock action-slot可见性；公共main/目录/主题未修改。CS-D01的collection/真实scroll、CS-D03的作者/事务、CS-D05的canonical新源owner与样式/导出仍open。

补充第四项的最小消费边界（13:55Z实际回读，见[设计第19节](character-skeleton-design.md#19-四个公共主件的source与必填value槽核对)）：PropertyEditorRow main=`6ddc248b-bd35-80c5-8008-d6c8c84d848f`，必填 `value`槽不接受当前4个已发布主件中的任何一项；SearchInput不是Field替代。当前最小公共依赖是已在catalog登记的 `res://ui/editor/components/workbench/primitives/inputs/field.zui#WorkbenchField` 的正式库identity/sourceHash和只读/slot覆盖合同，暂无UUID，不新造master。先推进真实Head的Bone Name/Parent两个只读字段，Euler/space/Axis和作者事务分别保持open。仅有 `editable_text=false` 不能代替现有host的typed read_only/focus/click同步；在合同交付前保留现有详情，不做空required-slot板或非法Text子项。原库4主件/前三项60Copy及8图证据有效，未宣称官方export/override-reset/全局A/B/EXE通过。

## CS-AN01：共享播放与时间轴

现有 owner：`zircon_editor/src/ui/timeline`、`workbench/animation/transport_controls.zui` 的 `WorkbenchTransportControls`、动画域的实际文档/播放适配。现有 `TimelineModel` 明确由域持有源模型和 inverse delta；现有 `PreviewPlayback` 字段为 `playing/looping/rate/time_seconds`。本域消费同一时间和反馈，不另建“角色播放时钟”。

| 合同项 | 本域要求与提供 | owner/边界 |
| --- | --- | --- |
| 活动 context | 当前 Toolkit/文档/preview group、实际 mesh/skeleton/clip 身份、document revision、preview session 与 subject generation；多个工具组不能被同一全局按钮串控 | Parent 统筹 host/preview；动画 owner 维护同一 transport context；本域提供角色 subject 和合法对象键 |
| 时间 | seconds 是求值权威；display FPS 只负责显示/步长，来源必须明确，不从 glTF 最后采样数猜帧率，也不硬编码 30 FPS | 动画 owner 维护 ruler/view/play range/scrub；业务适配决定 clip 合法时域、loop/rate 能力与诊断 |
| 事件 | 保留共同前缀 `WorkbenchExtension/AnimationTransport` 加 `Record`、`Play`、`Pause`、`Previous`、`Next`、`Loop` 六个后缀组成的稳定事件 ID；源 route、安装 payload、handler 和活动 context 分开记录 | Previous/Next 不擅自等同首/尾跳转；需要首尾命令时由同一 owner 的实际合同增加，不改原 ID |
| 骨骼/网格预览 | 同一 mesh/skeleton 的真实 clip seek/play/pause，返回实际播放状态与已求值 pose generation | 本域不把 Entity 属性 Sequence 当作骨骼 Clip；缺生产 backend 时禁止把图标切换当播放成功 |
| Retarget 对照 | source 是一个权威时钟；target 使用同次 retarget 求值输出，源/目标 subject 与 pose revision 可追溯 | 角色域持有 retarget 时间映射/求解语义；共享控件不启动两个各自漂移的定时器，不伪造重定向输出 |
| Control Rig | Rig 编辑中的播放、当前 time 和 graph 求值 revision 与同一 preview session 对齐 | Rig 图/solver/record/bake 能力由角色域实际后端声明，组件不因有 Record 图标就提供录制成功 |
| Physics 模拟 | 动画预览播放与物理 simulate/pause/stop/step 是不同的业务操作，必须由当前模式明确适配 | 共用控件表面可以复用；物理固定步长/模拟推进归 Runtime，停止恢复编辑姿势归角色作者状态。没有可逆模拟快照时不启用任意时间 scrub |
| 生命周期 | 切文档、切 subject、关闭最后消费者后，停止旧时钟/seek 请求并失效旧 ready/frame | 不拿旧文档的已完成播放反馈更新新角色；遵守 shared PreviewScene 原生命周期 |

共享控件的必须反馈是实际 playing/time/rate/looping、加载/错误/禁用原因、请求关联及结果 revision；“鼠标点过”不算执行完成。真实文本字段/IME 优先处理编辑键，Space/方向键等由当前模式及聚焦区域决定，不能注册成全局抢键。

## CS-AN02：共享曲线

复用 `zircon_editor/src/ui/curve` 的 `CurveModel`、`CurveView`、选择/画布变换和动画 owner 表面。`CurveModel::apply` 返回域 inverse；共享 renderer 不成为骨架或姿势资产的第二 authority。

| 输入/操作 | 角色域责任 | 公共消费要求 |
| --- | --- | --- |
| 曲线身份 | 实际文档/资源 revision + skeleton + bone/property/channel/component 的稳定键；显示名和行号不是身份 | 切骨骼、重导入或过滤后重新投影，保留合法选中键或清空，不编辑旧 generation |
| 时间/空间/单位 | 与 CS-AN01 同源的时间范围；区分 local/reference/evaluated pose，单位与转换规则由原数学/资产 owner 提供 | 不用一个纯 float “value” 隐藏空间和角度单位；轴标签、曲线与数值框读同一源 |
| 值/切线/插值 | 真实源通道提供读数与合法可逆命令；key drag、value/tangent/interpolation 修改由本域/动画文档 owner 执行 | drag 只预览，release 一次提交，Escape 取消；失败/资源换代不部分修改；undo 后重新求值 |
| 旋转 | Skeleton local_rotation 当前是 quaternion；真正旋转编辑需要域层归一化、插值及表示转换合同 | 共享 scalar curve 能显示 component 不代表四条任意 scalar 就是正确旋转作者；不把该显示当 quaternion 编辑已完成 |
| 只读/缺后端 | 已求值骨骼读数与可保存作者曲线分开；Socket/rig curve 源尚未支持时提供真实能力状态 | 不创建固定示例 key 或无效果 Add Key；共享组件 UUID 不等于该曲线可写 |

角色域不修改动画域的公共 timeline/curve 源文件。涉及新 curve 命令或共享实例/slots/events 的修改先经 Parent 交接给动画 owner；域内骨骼/Socket/姿势的序列化和事务仍由本域负责，不转移给共享 painter。

## CS-GR01：共享节点图表面

复用 `zircon_editor/src/ui/graph/{model,canvas,routing,commands,node_widget}.rs` 和材质 owner 的 MAT-SHARED-001 表面。`GraphModel` 的 NodeId/Delta/Error 归域，已有 retention/revision-bound prepare/apply 合同，禁止另建无限制整图快照历史或平行 graph renderer。

| 使用面 | 角色域实际语义与当前缺口 | 对共享表面的要求 |
| --- | --- | --- |
| Physics Asset Graph | 当前 RagdollProfile 每 bone 一 body/shape 和 parent constraint；未有 UE 每骨多 body/任意 joint graph 的完整作者模型。profile bone_path/parent 与实际 spawn entity 映射由本域管理 | 消费实际 body/constraint 节点/关系、selection、focus、picking；只读关系图与可编辑接线有明确能力边界，不能因画布允许连线就修改物理拓扑 |
| Control Rig 图 | 真实 Rig graph/控制/solver/执行顺序与可保存资产由本域实现；当前只核实数学 IK，完整 RigVM/Modular 图语义未证明 | 节点/引脚/连线的视觉、pan/zoom/box select/drag、文本字段、菜单和命中复用公共层；不能导入材质节点语义或把材质 compiler 当 Rig 编译 |
| Retarget OpStack | 当前 UE 参考是有序 OpStack + Hierarchy + pose 工作区，并非固定四链表或一张任意节点图 | 使用真实 ordered-list/tree 表面；没有实际图语义的区域不为了复用 graph 而画装饰连线 |
| 连接合法性 | 域提供 typed port、方向、兼容/拒绝、替换语义、环/拓扑约束；`StructureConstraint` 有 FreeGraph/Tree/Dag，不能对全部角色工具一概指定 Dag | shared `ConnectVerdict` 显示实际拒绝原因；连接预览、取消与提交命中分开；模型不变失败和一个可撤销域事务 |
| 位置 | 图坐标属于真实 graph authoring/layout owner；Physics profile 当前没有 graph position 字段 | 不把 UI 节点位置写进 body_offset/bind pose；无资源位置字段时由现有 Editor view-layout 状态保存/推导，依赖明确后再接持久化 |
| 几何/DPI | 同 MAT-SHARED-001 的 pane-local logical→graph transform；节点内容与端口由 flow/文字测量确定 | DPI 只缩放一次；pin anchor 和 edge hit geometry 来自同一实际布局，不取截图坐标或给普通面板套 canvas 例外 |
| 文档与命令 | 本域提供真实资源、能力、原子变更/undo/保存、关联资产打开和实际求解诊断 | 共享表面只持 view state 与投影，不私藏另一份角色/physics 资产；文本快捷键优先，取消菜单回原 node/pin |

Physics collider overlay 继续复用现有 provider/mirror 和中性 render 提取。节点图命中不能绕过该层直接拿主 PIE entity 充当 preview entity；两者必须归到同一 subject/session 及实际对象映射。

## 原始事件与实际处理路径核查

只读解析两份原始 v2 资产及 `ui/runtime/builtin/bindings/gameplay_animation.rs`：Retarget、ControlRig 各 15 个事件，30 项 event ID 与 Click/Change/Submit 类型均有对应 builtin descriptor。每份有 9 项源 `route` 与安装 action 名不同，adapter 按稳定 ID/事件类型解析，所以不能据此批量改 ID/route。以下记录当前处理器实际能力，不把“事件存在”升级为业务完成。

| 原 ID 范围（共同前缀 `WorkbenchExtension/`） | 当前实际路径/效果 | 接入真实业务所缺合同 |
| --- | --- | --- |
| `RetargetMannequinRow`、`RetargetRobotRow`、`RetargetQuadrupedRow`、`RetargetRootChainTableRow`、`RetargetSpineChainTableRow`、`RetargetArmChainTableRow`、`RetargetLegChainTableRow` | navigation spec 映射至固定 control，`apply_workbench_extension_action` 只更新同行组的 selected；部分行另外返回未实现反馈 | 实际 skeleton/mesh/op/chain 对象键、选择与详情/预览联动、关联资产跳转；样例名称不能充当资产身份 |
| `RetargetSourceEdit`、`RetargetSourceCommit`、`RetargetTargetEdit`、`RetargetTargetCommit`、`RetargetSolverEdit`、`RetargetSolverCommit` | `edit_workbench_module_field` 校验 control 拥有绑定及 Change/Submit，再写 control 的 `value/value_text`；有 options 的 `.edit` 可打开 dropdown | 真正的资源引用或 solver schema 校验、draft/commit/cancel、文档 revision 和可逆事务；不是输入字符串后自动成为有效 retarget 资源 |
| `RetargetPreview`、`RetargetApply` | feedback 的 output/status 是 `Not implemented (preview)`；可附最多三个当前文本输入 | Preview 需要真实求值/预览帧；Apply 需要真实域修改或导出语义、校验、undo/save，不得把反馈文字当成功 |
| `ControlRigHeroRow`、`ControlRigSpineRow`、`ControlRigHandRow`、`ControlRigSpineControlTableRow`、`ControlRigArmIkTableRow`、`ControlRigHandIkTableRow`、`ControlRigOutputPoseTableRow` | 固定 control 选择与部分未实现反馈，未在这条路径提供真实控制/solver 选择 | 实际 rig/control/solver/output 身份与稳定选择、图/树/详情/pose 同步，动态行使用真实对象键与 instancePath |
| `ControlRigControlEdit`、`ControlRigControlCommit`、`ControlRigSpaceEdit`、`ControlRigSpaceCommit`、`ControlRigWeightEdit`、`ControlRigWeightCommit` | 同一通用字段处理器维护 UI 字符串，Submit 也是被接受的事件类型 | 真实 control、坐标空间与数值域、合法范围、逆操作、求值诊断；不能把可输入等同可保存/可求解 |
| `ControlRigPreview`、`ControlRigValidate` | 明确返回 `Not implemented (preview)`，没有这里已证明的编译或求解结果 | 真实图/控制资产与 validator/evaluator 结果及 revision；不借用材质 compiler |
| CS-AN01 的六项 `AnimationTransport…` | `reference_menu_actions` 调到 `apply_blend_space_transport_action`，只维护模板按钮/文字；Previous 写 0.0、Next 写 3.0，目标是固定 BlendSpace timeline/status control | 由动画 owner 把同一稳定事件连接活动 subject 的真实时域、操作和反馈；不能把模板 0/3 秒沿用为角色 clip 范围 |
| Physics `asset.physics.ragdoll_profile.create`（插件 operation） | 注册的 event 为 `WorkbenchMenu(OpenView(physics.ragdoll_profile))`，creation template 指向原 ZUI；不是生成 profile | 接实际 Skeleton 数据、现有 generator、文档创建/校验/保存/undo 和失败返回；不重建 generator/overlay |

上述工作台动作还在 `preview_actions/extensions.rs` 的预览名单中。`common/dispatch.rs::dispatch_editor_binding` 会把预览动作交到 host 的 preview 记录路径；`editor_event_dispatch.rs::record_component_lab_preview_action` 实际记录 `Transient::PressNode`，`operation_id/transaction_id/save_generation=None`、`effects=[]`，结果虽为 success，但 `changed=false`。此类 journal 条目仅证明点击被观察到，不能用它验收 Apply、Validate、播放或撤销；Physics 的创建缺口来自其单独的 OpenView 注册，不将该插件 operation 混入这份 preview 名单。

当前 `preview_features_enabled=false` 时，通用扩展处理还会直接显示未实现状态；打开模板预览开关也不会补出域后端。接入时保留现有原始 ID、事件类型和兼容投影，新增真实 typed 命令/对象参数并退出相应 preview-only 路径；涉及共享 action registry/host 的具体 patch 先交 Parent 及相应 owner，不由本域单独放宽预览名单或伪造 effects。真实操作证据应关联资源和 document revision、事务/结果及实际已求值预览帧。

核查范围是以上实际源调用链，未执行这些动作、Jenkins 或产品测试；30 项 ID/kind 闭包只是结构检查。一次搜索使用了不存在的候选目录、另一次使用了 Windows 不展开的路径通配符，均已按现有路径及 `rg --glob` 重查；没有据失败搜索推断能力缺失。

## Parent 需要统筹的接入项

沿用 CS-D01–05；本表按最新 owner 分配收窄需求，未授予跨域写入。

| Owner | 本域实际需要的最小交接 | 当前完成情况 |
| --- | --- | --- |
| 动画 `01a16d51…` | CS-AN01/02 的组件 UUID/版本、原源 owner、真实 slots/props/events、活动 context 与实际播放/曲线能力；原 transport 六项事件保留 | 动画owner已交付Timeline/Curve/Transport三个原生草稿身份，见下段；尚未跨文件接入角色/产品 |
| 材质 `01a16d50…` | CS-GR01 共享图表面组件/渲染/输入/几何合同，域模型适配入口及 native sourceRef/instancePath | 已核对现有 GraphModel 与 MAT-SHARED-001；未取得可交互 retained 图或组件版本 |
| Parent | 公共 TreeRow/PropertyEditorRow/Menu/Dock/UiHostWindow 样式及库映射、真实 PreviewScene/GPU image owner、导出与 capture 协议集成 | 真实 file/page 已分配；公共原生组件/颜色/typography 库仍为空；不能引用伪造组件 ID |
| 角色域 | 本域 mesh/skeleton/Socket/Pose/Profile/Retarget/Rig 数据投影、真实能力、typed mutation、稳定身份、原 ZUI 业务事件及差异 | 已保存本域只读详情原生组件/三实例；新增唯一真实角色CPU回归，尚未运行；原业务ZUI和生产Rust未改 |

不同时改写公共组件文件、动画表面或材质 graph 源；重叠路径在明确交接后才写。基础库未发布时可以继续独立设计语义/布局和本域状态，但不复制按钮/字段/时间轴/图表面/主题以伪装复用，也不把页面根节点算作可编辑设计完成。

动画owner的当前[共享控件合同](animation-shared-controls.md)已交付三项草稿：Timeline component=`8cf70bfd-9275-4bac-849b-b79148b8a4fd` / main=`4db0829c-abc4-482d-b7b3-155a5d8d75c2` / v2；Curve=`78492c86-3bb4-47dd-97c6-752d6ff37e3d` / main=`599963dc-b91b-4740-926b-d38154aa8f3a` / v1；Transport=`982ef940-2239-4167-a24a-56c80ec35fbf` / main=`44eb5ccd-8ac2-44f2-a4ad-d0d618daa096` / v1。实际file/page=`b4a1e3ce-9e6a-81fe-8008-d69d60a13f99` / `b4a1e3ce-9e6a-81fe-8008-d69d60a13f9a`；其owner先前记录为rev8/302对象/9实例；Parent在12:34:03Z独立API核实已到rev12/533对象，仍为同3个主件、linkedLibraries=[]，尚未发布。三尺寸PNG为rev8记录，不能作为rev12画面；Parent交接给公共pipeline的Transport/BlendSpaceDetails/SampleWeights三项catalog登记也不等于本表Timeline/Curve/Transport三设计主件已经注册或可跨文件消费。角色域未复制这些main或迁移其domain模型；跨文件库发布、slots/export和真实backend仍由原owner/Parent接续，不将草稿UUID当作功能已接通。

## CS-XR01：九族归属及跨域资源打开

Static Mesh、Texture 独立工具由资源会话 `01a16d53-60db-7d02-b03e-0042f277783d` 持有；Skeletal Mesh、Skeleton、Physics Asset 属本域。Retarget/Control Rig 与既有动画、材质共享表面边界不变。CS-D02 与资源域 [RT-S06](08-resource-tools-shared-contracts.md) 共用 Parent 单一 PreviewSession/GPU image/media/input/ready/frame/teardown 合同；本域只提供真实角色资源、node↔skin↔skeleton、pose/picking 和 Physics entity↔profilebone 语义，不创建第二个 backend 或角色时钟。

实际 [asset_type_id_for_locator](../../../zircon_editor/src/ui/workbench/project/asset_workspace_state.rs) 目前从 catalog kind 生成 AssetTypeId，`ResourceKind::Mesh` 单独不足以区分静态与骨骼网格。由 Parent 与资源 owner 在现有 catalog/registry 边界明确真实类型和 Toolkit 解析，不能双方各自覆盖通用 `mesh`。原 [AssetToolkitOpenRoute](../../../zircon_editor/src/core/asset/toolkit_route.rs) 使用规范 locator/open operation；[OpenAsset](../../../zircon_editor/src/ui/host/editor_event_execution/asset_event.rs) 核对实际 registry、command 和 enabled 后开视图并聚焦。`ActivateReference` 仅导航资源；状态文字、打开视图和资源 ready 仍是不同证据。

真实 Wolf 的 Node48/Greyboy→Mesh0→Skin0；Knight 的 Node23～31 分别关联 Mesh0～8、Skin0～8，均为 Material0。引用必须由真实 importer/catalog 返回的 locator 解析，不用本地路径、名称或推测 UUID。`MeshSkinAsset` 当前仅携带逆绑定矩阵，插件按 mesh index 保留首个 node skin；不能单靠这项载荷、骨骼数或显示名恢复完整关联。复用现有 SkinN Data 的 joints/skeleton_asset 及 primitive/material 引用；映射缺失或多义时保留诊断，交公共资源接口 owner 处理。

跨域 material/texture 关联打开须携带源文档/选中对象/焦点键及 revision，完成目标真实 Toolkit 打开；返回时重新核对源代际与稳定对象身份。目标未注册/禁用/缺资源保持明确失败，不能用无效按钮或假成功替代。切换关联页签不复制 preview world；旧帧与最后消费者释放继续由 Parent 统一处理。

精确源哈希、10项真实 mesh/skin 关联及9项待执行场景见[源关联与导航合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/cross-domain-navigation/source-and-navigation-contract.json)。这不是运行导入或导航测试结果；原 Rust 请求仍 pending，产品 `passed/accepted=false`。

## CS-TP01：typed subject / committed pose / frame资源生命周期

最小方案和两个真实未应用diff见[设计第16节](character-skeleton-design.md#16-typed-subject已提交pose与真实frame最小适配)。本域继续消费既有PreviewScene生命周期；不占公共入口，不创建backend/clock。Parent整合的Editor候选将locator收束到现有ResourceLocator，并要求调用方subject generation；同时堵住成功close后的同值setter提前返回。Runtime候选从现有提交快照导出animation_generation，原pose/骨架/实体和renderer求值复用，不制造新pose或frame。

本域提供真实root/part/mesh/skin/skeleton/material/clip引用、revision与选择/picking目标映射。动画owner掌管time/参数/evaluator；Parent负责gateway/document/subject/World/pose/render-frame/ready/image身份关联、实际帧显示和最后消费者释放。现有capture帧没有完整subject/pose关联，picking的generic subobject也不是骨骼槽位；这些仍是具体生产接线缺口。

取消/重导入/换subject后，消费者重新核对当前doc、subject代际及已显示帧再接受原route回执。Retiring期间拒绝新输入；原端点取消pick、解绑image和释放frame引用后才retire App lease，失败保留owner重试。SharedPreviewScene的Rc或Drop本身不是teardown成功证明；不复用PlayController主PIE或假PlayInstanceId作为Persona。

[完整30源哈希与生命周期合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/typed-preview-adaptation/typed-subject-pose-frame-contract.json)中明确区分已存在接口、未应用提案和未实现字段。两diff应用检查通过但未编译/运行；Fake只用于既有单位测试，不是生产preview证据。原Penpot实例/官方导出/GLB ready/正常EXE及原生矩阵继续pending。

## CS-G10：原导入policy修复与受限Jenkins配方

Parent已交两原owner路径，plugin白名单增加meshopt、negative改未支持Draco，两CPU integration正例未变；三源guard/格式保持，未改decoder或preview backend。第17节旧cdd client的material-only拒绝保留为历史。当前28698支持两条精确test配方，本域已审阅三路径输入/18哈希，待当前writer/资源/Parent时段后正常准入，见[第21节](character-skeleton-design.md#21-gltf限定入口交付与角色owner三路径审阅)。本域不修改管理客户端/manifest/lock或借材质范围；实际3项Rust和产品验收继续pending。

## 验证与桌面请求条件

当前没有桌面时段。已在真实文件使用隔离 headless browser 和现有官方插件形成可编辑 Retarget 基线、实际保存关闭重开；临时插件静态 host 随自有 browser 退出。未启动/恢复 Penpot/Jenkins/Engine 服务，没有共享桌面输入或系统 DPI 操作。该基线不是 UE 目标内容，不能用 semantic materialization 替代 native instances 或资源就绪。

待真实可编辑设计和生产资源/EXE 就绪后，向 Parent 排队提交 file/page/board、真实场景/交互、1280×800/900×620/640×520 @1 和 1280×800 @实际1.5、EXE/依赖指纹、nonce/generation（已有时）、预计用时和原请求。只有 Parent 明确回复时段后才执行共享桌面输入/原生捕获；完成后报告进程实际退出或交接、输入释放、证据和失败路径。队列 ACK 不授予桌面。

本阶段完成源码/身份/Markdown/JSON/TOML指纹核查、原基线保存重开子项及上述CS-G10两文件最小修复的三文件静态guard/格式；尚无Rust执行。UE 目标设计、共享组件实例/样式、官方 ZUI/PNG 导出、Jenkins 插件/Rust 聚焦测试、真实预览和原生双端逐图验收均未完成。导出失败已交 Parent，未修改公共 pipeline。产品 `passed/accepted=false`，整体 goal active；释放空间 0 字节，任务浏览器临时目录仍按任务归属保留，未宣称生命周期关闭。

CS-D05早期插件导入失败继续open：第12节Wolf Tree（26 projected nodes、17初始可见joint）触发SDK missing→null可选paint的painter snapshot比较；整窗145节点另有超时。该两次失败当时未形成保存板，transient UUID不复用；后续第18节已通过正式共享库linked Copy保存真实48骨骼树，两种链路分别记录。旧guard未由本域放松，公共组件/主题/导出仍单一owner，不以源JSON TRS替代Engine pose/mesh preview。

## CS-D05 原生更新与文字覆盖消费修订（rev54）

共享库当前实读rev5/4原master/34颜色/3字体；Button默认fill真实绑定599586e3-022c-500b-ac80-5d9c490139af。原生UPDATE曾覆盖本域94树标签，本域旧authoring仅content-group，缺当前Penpot文字细分text-content-text；归属本域作者数据修复，不要求公共owner改Shared来掩盖。原生Undo恢复，再6个native text先证及112个typed官方set-touched完成118个文字保护，rev54保存关闭fresh重开一致。原ID/componentFile/componentId/shapeRef、内容/样式/几何均保留。

该修复不证明下一次发布同步通过；library syncedAt未随Undo回退，禁止强写时间、detach或用新master绕过。Reset overrides实际还清尺寸覆盖，控制板New77.333→240与Search覆盖，不能把它当只重置颜色的合同。请公共owner在下一次正常有意义的Shared发布后消费精确源身份，验证本地文字/样式覆盖保留和reset支持范围，勿为演示单独改Shared。具体证据与最小复现见[第26节](character-skeleton-design.md#26-原生库同步失败撤销与本域文字覆盖修复当前-rev54)及[本轮交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-library-sync-reset-r01/parent-handoff.json)。Field/typed只读value槽、真实引用的canonical样式解析、官方PNG/export及两族全局A/B依旧由公共owner/Parent交付，验收不提升。

## CS-D01/04：骨骼 reveal 消费补充（r04）

本域已提供 `SkeletonReference::reveal_for_name_query(&SkeletonBoneKey, query)`。使用与树行一致的字面筛选规则；Included给同一资源版本的原目标和根到父的祖先键，FilteredOut明确保留被隐藏的原目标，None表示该引用不能解析这个键。它不把名称或可见行号作为身份，不返回 Socket/virtual bone 假键。旧snapshot可供只读查看，不代表当前预览资源或显示帧可接受输入。

Parent继续拥有展开偏好/force、collection generation/实体化、scrollIntoView、选择集合与事件方向、焦点返回、document命令和frame admission。本域不向共享TreeIndex塞新store，不持久化进程内键；应用父链后仍需核对同代文档/资源/collection和原目标，失败保留诊断。完整[消费与拒绝合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-reveal/consumer-contract.json)列出边界、现有接口和未实现host行为。源码11路径/24项定义0执行，公共接线与EXE验证未完成；不扩展原glTF范围或声明产品通过。

## CS-PA：未打开Physics配置的设计消费边界（rev63）

本域原file/page的三尺寸新板复用现有Shared rev5 Button6ddc248b-bd35-80c5-8008-d6c8c01096f3及Search6ddc248b-bd35-80c5-8008-d6c8c3ce892e，各3只；主件/源节点身份不变，没有新增公共组件或主题。新增文本直接引用body99eb86f9-12ff-5935-96ad-3d1a985e61f0及实际颜色，CJK字体/行框仍非EXE已验证。原asset六control ID映射、全部新节点与组件refs见[精确合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-physics-empty-design-r01/design-contract-placement-r02.json)。

CS-D04由Parent提供单一Toolkit/菜单/Dock及typed registry打开/焦点返回，支持UE上层级/下图表、中Viewport、右Details/AdvancedPreview及Tools/Profiles页签。当前设计的标题/页签文本只是区域目标，没有伪造published Dock UUID或可用事件。打开需要当前profile资源版本与skeleton/mesh/skin实际关联，不能按.preview字符串批改stable binding IDs。原create命令仍只OpenView；本域后续把真实skeleton输入generator、作者数据和有效非空profile持久化交document/history合同，不能用空profileReady或临时Sequence/props替代。

CS-D02/Physics继续沿既有PreviewSubject/Playback及唯一secondary backend。profilebone路径要与版本、skeleton key和该session body实体映射核对；UE body/shape/constraint是不同选择，现profile一骨一个shape的数据限制保持显式gap。主PIE overlay/mirror不作为secondary图像，未交付生产backend前不翻capability。没有新增图表backend，Physics专属图表语义后续复用材质牵头公共表面而非材质domain模型。

公共C6新增三个实际consumer：Search根宽238.4/162.4/110.4，既有内部surface240，几何溢出1.6/77.6/129.6；640文字和900边框实际裁切。本域保留原shapeRef和内部源几何，没有改Shared/组件子约束。原asset左右min200/240与窄比例目标的合同差异也未擅自消除；未请求新master或接受值扩展。证据见[当前结构](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-physics-empty-design-r01/structural-review.json)及[逐图复核](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-physics-empty-design-r01/visual-review.json)。官方export/资源闭包、正常EXE/DPI/1DIP和整族验收保持open。


## CS-PA02：Physics 配置候选与现有文档事务（r01）

领域交付`zircon_plugin_physics_editor::prepare_ragdoll_profile_edits`：输入host当前RagdollProfile与按精确作者bone_path的typed edits，修改已有Shape/Mass/BodyOffset/Constraint/BlendWeight，返回None或完整有效候选。无store/clock/history/文件I/O/World/backend；Runtime复用既有manager validator，新增InvalidConstraint并拒绝原先漏检的constraint/compound数值。当前8源/3新增文件、6＋12项定义与精确hash见[源交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-edit-r01/source-handoff.json)。格式/guard静态通过，0项Rust执行。

Parent继续持有document/resource/profile的版本与CAS、一次before/after历史事务、保存及focus-return；纯API无ABA/旧版本权限。角色typed ResourceActiveVersionKey+原joint slot和实际mesh/node/skin应与preview generation、entity↔profilebone匹配，主场景PIE镜像不授权secondary工具编辑或picking；无新增backend/clock。原create仅OpenView、RagdollBoneProperties无typed消费、Field正式UUID/slots及真实profile lifecycle仍缺，[合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-edit-r01/consumer-contract.json)逐项保留。one-shape-per-bone/compound子shape无稳定ID、单位转换、资源引用/backend支持与clone成本仍未验证，不以局部数值有效称ready。

此scope不追加原glTF请求或骨骼Editor24计划；两个Physics单包配方仅草案、入口资格未核验，未发新执行/准备请求。完整member/writer/resource/pool/native guard与EOF、产品消费者/undo/save/实时preview、正常EXE/实际DPI与双端逐图均open，passed/accepted/captureReady=false。


## CS-RTG：Retarget未配置状态与真实命令准入（rev66）

UE v1.022新增OpStack/Hierarchy、AssetBrowser/Overrides/Variables、Viewport/OutputLog及Details/AdvancedPreview区域，当前三尺寸空态设计只描述真实无资源/0映射条件，没有processor、链模型或可用命令。原ZirconPreview/Apply仍只是Not implemented (preview)，原.zui三rig/四链/Ready原件保持且未算通过。新6只Button复用Shared rev5原UUID/shapeRef，主件、207处样式引用、外部root/source闭包经保存关闭fresh重开核对，不另建Field/Dock/Menu或主题。

Parent继续持有Toolkit/Dock/菜单溢出、registry资源picker与focus-return、现有document/history及唯一secondary backend；本域持有实际skeleton/mesh/pose/joint keys与Retarget定义/链业务。新源/目标选择caption不冒充原field事件实现，不能把原稳定TextChanged/Commit改解释为Click。640“资源 / …”、原min270/320、真实资源与retarget processor.ready、官方export/原source映射及loaded/selected/focus/scroll/native验证均pending；[精确合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-retarget-capability-state-r01/design-contract.json)和[逐图记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-retarget-capability-state-r01/visual-review.json)供共享owner消费。本轮不重复同pending消息、不扩原glTF身份/配方，Skeleton24与Physics6＋12均0运行，passed/accepted/captureReady=false。


## CS-PA03：Skeleton 资源到 Physics 生成候选（r02）

本域新增generate_ragdoll_profile_from_skeleton(resources, expected, profile_id, source_paths)，复用现有snapshot_expected与原generator/validator。source_paths按原joint slot显式给全作者路径，不从name、filter row或mesh第一skin猜引用。输出只读profile、source ResourceActiveVersionKey/revision及exact profile path→original_joint_slot；不新增共享SkeletonBoneKey、document store/clock/history/World/backend。Parent原editing root声明不重放。

Parent在prepare与commit持有当前资源/文档检查、真实full-path producer、一次before/after事务/保存/focus-return；原asset.physics.ragdoll_profile.create仍仅OpenView。唯一secondary host再绑定真实session/mesh/node/skin/pose与entity↔profilebone，当前候选不是current pick或frame权限，runtime name/leaf fallback未自动变成typed slot。详见[最小消费合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-skeleton-source-r02/consumer-contract.json)。既有capsule heuristic非UE网格拟合，finite TRS非pose有效，one-shape限制和真实preview仍open。

当前10源静态通过，Physics Runtime6＋Editor22仅定义0执行；新文件membership透明交接，不扩原glTF或Skeleton Editor scope、不发准备/执行请求，无新freeze/window。官方导出/公共Field及生产backend、正常EXE/DPI/逐图验收分别保留门槛，passed/accepted/captureReady=false。


## CS-CR：Control Rig源状态与Shared TreeRow窄宽消费（rev72）

本域复用原Shared rev5的TreeRow/SearchInput/Button，共21只新增native linked copy，shapeRef与原child顺序不变；不detach、不复制main、不造图表面/主题/Field。实际源选中Knight_Head node27/mesh4/skin4的head joint slot14，过滤保留原祖先；这是GLB JSON源状态，不是持久RigHierarchy或求解pose。Graph仍归材质公共表面，时间/evaluator归动画，Parent继续持有Toolkit/Dock/registry/focus-return/history和单一secondary host。

C6有具体受影响行：900板ec82e74c-393d-498a-a802-d66cbe3178c0与640板a5ee9c22-c321-48c5-a73a-388a56543655的选中head，原caption glyph宽29.1875、裁后可见宽0；1280可见29.1875。root分别收至138.4/86.4，内部仍是240px主件caption约束，未在本域改Shared子几何或改UE比例掩盖。公共owner需在原TreeRow source/slot及导出约束闭包内修复再同步，不另发UUID或拓accepts；SearchInput内部约束、骨骼专用glyph、canonical Field及真正scroll/focus/菜单仍pending。[结构与节点度量](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-control-rig-source-state-r01/structural-review.json)、[12张逐图记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-control-rig-source-state-r01/visual-review.json)保留实际失败。

原ControlRig 31节点/29ID/15绑定不改，源Edit/Commit不解释成Click；新“打开Rig…”按钮只是设计，需真实registry/Toolkit命令接入。只有本域三个新只读说明的content和派生文字缓存被修正，Shared/data/source refs保持；不是公共style A/B或官方导出/产品加载证明。33保护源和9参考输入保持，无新prepare/request/query/capture/freeze/Cargo/桌面，实际Rig author/solver、生产preview/正常EXE/DPI及全族验收继续open，passed/accepted/captureReady=false。


## CS-SKIN：显式SkinN Data与原joint key（r05）

本域交付GltfSkinReference::read(resources, expected_skin, &SkeletonReference)，现有snapshot_expected<DataMarker,DataAsset>提供单资源配对；验证现gltf_skin schema并输出原序GltfSkinJoint、key_for_node/joint_for_key、Skin version/record和可选inverse-bind locator/count。既有SkeletonReference新joint_count为只读访问，无独立loader/store/clock/history/后台或共享schema变更。[消费合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-skin/consumer-contract.json)与[源交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/skeleton-reference-skin/source-handoff.json)列明精确版本和错误边界。

Parent须先给真实node/mesh/primitive/skin绑定与资源组版本准入，再将原joint key接动画pose、同secondary session的frame/picking和Physics entity↔profile path。SceneMesh当前漏掉node.skin()仍是具体共享DTO依赖，不让两个族各造持久映射。两个保留snapshot并非原子多资源publication；骨名/count相同也非当前pose/pick权限，matrix引用/count非解码、finite或GPU palette证明。纯glTF SkinN producer合同不推广为任意Skeletal Mesh格式，未改PreviewSubject、公共graph/timeline/Field或生产backend。

15源静态通过，Editor32/Physics6＋22全0运行；同一Editor filter不能再描述为24或9项。原glTF同request/operation三路径/两recipe不扩，原legacy32ce及r04指针/失败原件归档保留，新membership不是完整writer/资源准入。无新执行/准备身份、query/POST/capture/freeze、服务/桌面或正常EXE。Penpot72、C6裁切、official export、author/history/save/focus/真实preview/DPI/双端验收独立open，passed/accepted/captureReady=false。


## 35. Physics 世界平移到 body offset（r03）

prepare_ragdoll_body_translations 读取真实 RagdollProfile、每个目标的实际 bone_to_world 与 world_delta，复用 checked affine inverse，将世界增量按向量转换到骨骼局部空间，再一次调用 prepare_ragdoll_profile_edits。actor 平移不参与向量变换；仅修改 body_offset.translation，保留旋转、缩放、shape、mass、constraint、blend 和骨骼顺序。整批失败不改源；空批次或净零变化返回 None。

UE依据为 PhysicsAssetEditorEditMode.cpp 的 StartTracking/InputDelta：停止模拟后编辑，并通过原事务提交。该切片只覆盖 Zircon 现有每骨单 body offset 的平移；UE aggregate primitives、旋转/缩放 gizmo、CoM 和网格拟合仍缺。r03为12源、8项新增定义，相关 Editor 作者回归30项、Runtime作者回归6项均未执行；不是整个包测试总量。格式和源码guard通过只绑定r03当时字节，后继源以第37节为准。

[原r03交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-body-translation-r03/source-handoff.json)与[消费合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-body-translation-r03/consumer-contract.json)保留。原Jenkins的8项配方交接不等于正式收件或执行；glTF及Skeleton Editor身份保持独立。没有新增store、clock或backend。

## 36. TreeRow 窄板原生布局修正（rev76）

角色 file b4a1e3ce-9e6a-81fe-8008-d69d60adb48e/page b4a1e3ce-9e6a-81fe-8008-d69d60adb48f 的三张 Control Rig 源状态板中，15个既有 native linked TreeRow 已通过原生 flow override 重新布局。保留 component/file/shapeRef、child顺序、业务main及原metadata/guard；不detach、不复制共享main、不改Shared。Shared当次实读rev5/4主件/34颜色/3文字样式，hash259ddb2aec9c0a16f3a36755fa4a0a991b96116ac66401b24f230a2ea8e12132保持。

API的rev73只保存属性，首次fresh读回未证明reflow。后经原生inspector修改gap 6→7→6取得真实布局，再依据children反向z顺序将15行设为row-reverse，保存至rev76。1280×800、900×620、640×520关闭并fresh重开，6张原图逐图打开。图标left12/宽16，caption left34.00000048；选中head字形可见宽29.1875，caption宽依次168.39999952、92.39999952、40.39999952，修正900/640原宽0。3367对象中的105处变化均在15个copy子树内。

[逐图记录](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-character-tree-overrides-r01/visual-review-r08.json)、[布局合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-character-tree-overrides-r01/current-layout-contract-r08.json)及[640实图前后差异](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-character-tree-overrides-r01/penpot-before-after-640.png)保留。中断于rev漂移的geometry helper未写入手工坐标，失败诊断原样保留。浏览器77/107/145%画布缩放及DPR1不等于Windows实际DPI。

当前本地native flow与官方TreeRow painter/导出合同仍存在差异：[source-contract-audit.json](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/penpot-character-tree-overrides-r01/source-contract-audit.json)记录15处caption几何与旧metadata guard不符，以及无expander的content变体仍预留18px的具体源位置；这是源码算术检查，未执行官方导出。已交原public owner；不自行重签guard、不写公共源或.zui。640根行在viewport顶部裁切、真实首尾scroll/focus、骨骼glyph、SearchInput fit、Field、同步/reset、正式导出/产品加载/EXE/DPI仍open。

## 37. Physics typed key 与整批编辑候选（r04）

本轮在原Physics Editor内新增binding.rs：RagdollSkeletonBoneKey由已安装骨骼生成候选提供，保留ResourceActiveVersionKey、原revision、原joint slot和完整作者路径。prepare_bound_ragdoll_profile_edits及prepare_bound_ragdoll_body_translations先用现有snapshot_expected重核真实skeleton载荷，再逐项检查key与source路径/slot，最后复用原批量编辑或世界平移函数。重复属性编辑按手势顺序，空或净零返回None；任何错误不发布局部结果。候选仅持私有before/after和来源版本，validate_current拒绝骨骼替换/ABA及profile任意内容变化。

此处没有伪造profile资源UUID或修订号，也没有新文档store/history/clock/backend。当前profile尚无已发布的通用宿主文档载体；完整值相等不能证明文档ABA安全，snapshot读锁在返回后不保留。Parent仍须在原document/source/revision/edit lease下做资源准入和最后CAS，一次history操作、原save token保存、undo/redo及focus-return。真实作者路径producer、同secondary session的entity↔profilebone/frame/pose generation与共同资源生命周期仍是明确依赖，不由本候选授权当前pick。

11项新增回归定义覆盖重复显示名与原slot、整批顺序及不变字段、no-op、异资源key、同版本不同路径映射、profile缺骨、无效profile、等值载荷替换、同ID同revision重建、任意profile变化及多骨affine平移/重复目标。相关作者Editor定义共41项、Runtime6项均0执行；Skeleton Editor仍32项，三类身份不混用。

本轮scoped rustfmt和源码guard通过，0错误/警告/抑制；静态证明不等于Rust运行。现行53172128… capsule的真实parser函数已在内存检查精确Physics单包配方，返回build_scope_policy，因为当前nested plugin白名单不含Physics Editor；未dispatch、QueryOnly或启动Cargo。精确声明为zircon_plugin_physics_editor/defaults off/features[]/locked/profile test/jobs1/lib/filter ragdoll_bound_edit_tests::/threads1，物理E:/cargo-targets。交原Jenkins owner补受限资格，不借glTF、Editor或旧8项输入身份，不自行改parser或扩大包。

[本轮源码交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-bound-edits-r04/source-handoff.json)、[宿主消费合同](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-bound-edits-r04/consumer-contract.json)、[精确测试计划](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-bound-edits-r04/test-plan.json)与[入口检查](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-bound-edits-r04/route-scope-check.json)分别记录当前源及未完成门。r03原12源中仅两处域root接线后继，其余10源保持；manifest/lock/plugin注册/原ZUI/Parent root/Skeleton r05保持。未创建源码副本、执行身份或九root冻结窗口。goal active，passed/accepted/normalExe=false。

文档修复说明：上轮第35/36节和顶部摘要在PowerShell到Python管道中丢失非ASCII字符。此轮以真实r03与Penpot r08证据重写可读内容，旧回执和前像SHA保留，不把历史静态回执改签为本轮结果。后继写入前已验证UTF-8码点，避免再次以问号替换中文。

公共Document/Toolkit/registry/focus-return与唯一secondary backend仍由Parent统筹；本域仅作者路径、typed keys、业务before/after与诊断。Graph与Timeline/Curve/Transport继续由原owner持有。

## 38. Physics Profile 持久化编码（r05）

当前Runtime已有RagdollProfile::from_toml与完整作者校验，新增codec.rs提供to_toml：先复用同一validate，再以既有toml1.1.2/serde schema写出字符串。RagdollProfileEncodeError区分Profile准入失败与Toml序列化失败，保留原错误source。不改RagdollProfileError既有变体、字段/schema、原读取器、manifest/lock或capability；不写文件、不发布资源、不建立第二store/clock/history。

宿主可在候选准备阶段对candidate.after().to_toml()做编码预检，但不能据此标记已保存。实际保存必须从原Document取得真实已提交revision与原save token，编码该revision的完整profile，使用原持久化流程写入；仅在保存成功且token仍匹配时标记该revision已保存。编辑或资源变化、编码/写盘失败都须保留dirty和原history。原document/source/revision/lease的一次CAS与undo/redo、同secondary session实体/骨骼/pose/frame关联仍由Parent集成；此API不提供这些权限。

新增9项Runtime编解码回归定义：既有稀疏TOML默认值、中文/转义/长作者路径和父后子前顺序、全部8种既有碰撞形状及嵌套compound与UUID/url配对、稀疏轴与全部drive字段、Some(0)与None及锁轴区别、有限浮点边界/TRS/负零、不合法profile原错误、legacy三轴数组和root/bone未知字段拒绝。测试仅用内存单元数据调用真实codec，不注册生产夹具或宣称真实角色/资源ready。相关Runtime作者定义由6增至15，Editor作者41项、Skeleton Editor32项不变，全部仍0执行。

同源scoped rustfmt、4源码及4文档guard、增量patch反向check和diff检查记录在[本轮交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-codec-r05/source-handoff.json)。当前17源为r04十四源中一个Runtime mod后继、其余13源保持，另显式纳入Runtime lib接线与2新文件；r03/r04旧交接及旧过滤器输入原字节保留。托管最小新范围为zircon_plugin_physics_runtime/defaults off/features[]/locked/profile test/jobs1/lib/filter skeletal::codec_tests::/threads1，物理E:/cargo-targets；不启用backend-jolt，不借glTF、Skeleton或Editor11项身份。现行53172128…真实parser对此Runtime单包配方仍返回build_scope_policy，零dispatch/Cargo，限定资格交原Jenkins owner接续正常prepare/intake。

编码保留当前typed schema能表达的数据；注释、格式和已被宽松nested decoder丢弃的字段无法从DTO恢复。静态源码确认root/bone拒绝未知字段，而共享PhysicsColliderShape、PhysicsJointConstraintMetadata与PhysicsJointDrive没有deny_unknown_fields。原始文档完整保存须选择严格拒绝未知nested字段，或由原Document保存未建模原文；不以writer存在宣称整份原始文本无损。[三处共享小补丁提案](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/physics-profile-codec-r05/strict-nested-fields-proposal.json)及精确diff已通过非修改apply --check；共享源保持，兼容性决策与其独立回归待Parent/共享owner评审。

本轮没有写Penpot、原ZUI/公共主题/控件/Shared、桌面或服务，也没有发出全局冻结时段或执行身份。正常受管准备沿任务授权继续，不要求人类补grant；真实source/resource/pool/cache/native与Rust/产品证据仍分别pending，goal active，passed/accepted=false。

## 39. TreeRow compact 提案与业务导出身份

当前native与插件content均隐藏disclosure glyph/actions并保留其对齐槽。公共owner的[精确前像与提案](<E:/cargo-targets/zircon-local/reports/penpot-unreal-01a0f04d/ue-penpot-resume-20261023/local-execution/public-component-closure-01/character-tree-content-lane-r01/proposed-scope.json>)要求先由Parent统一native/catalog公开范围，再以既有component_variant中的明确opt-in选择紧凑布局；content-compact只是候选，不能据此填写新公开属性或复制新主件。外层缩进只计算一次，原source/main/child/shapeRef稳定，inactive五part需source-owned visibility且保留拓扑。

归档15行的90个直接part sourceRef保留，不能因继承part-ID不是copy ID而判丢引用。15 caption guard、75 hidden/original-visibility和15 allocation parent差异仍须通过新鲜精确前像迁移解决；不得直接layoutTreeRowText、full Reset、删part、detach或重签读回baseline。默认/对齐content不应被公式变更影响；横向opt-in与Y8/Y6静态差异分别待验证。

三张业务板无正式role=asset/source envelope/export guard，行canonical metadata不代表业务资产owner。正式路径需原Shared单TreeRow源面先验证，再消费来自原业务资产的真实invocation owner/node/instancePath。本域此轮不写公共源、Shared、metadata或guard，也不发SDK/export/Jenkins/request/host/desktop操作。

review.domainDocEncoding所列r03乱码SHA已沿r04修复和r05后继链匹配当前文档；本轮严格UTF-8扫描连续ASCII问号/U+FFFD均0，解释只来自原始审计/源码/公共审阅。旧失败与source/test身份保留；[本轮文档交接](E:/cargo-targets/zircon-local/reports/ue-character-skeleton-20261024/tree-content-public-review-r02/handoff.json)不授予共享写权限或产品passed/accepted。goal active。
