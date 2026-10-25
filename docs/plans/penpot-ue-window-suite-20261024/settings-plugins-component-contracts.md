# 设置与插件：实际需要的公共组件合同

r55 公共消费合同已收敛到 [真实 typed provider 与源调用点](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/settings-semantic-subtree-r55/semantic-subtree-contract.json)，而非 role 清单。现有 PropertyRow/Field/Number/Checkbox/Dropdown/TreeRow 原源均可复用；没有等待 catalog 再造 primitive。value 槽必须恰好一项，Runtime virtual_rows 当前每 sequence 一个 prototype，混合类型保持原字段顺序需要实际 source materializer 合同，不能塞多个 hidden value 或编造 repeat kind。

需要 Parent／原 owner 接线的范围明确为：同一私有 bridge 的 typed capture/call、混合 Row source prototype/slot、真实页面 scope/document 与 commit currentness、scope 目标层显示/default catalog、原 typed numeric setter 的真实 surface callback。schema 没有通用 units／precision／large_step；plugin_pages 仅 metadata；复杂类型与完整 Input Bindings provider 未完成。旧 string/chord 等入口按 definition.scope，coordinator.set/clear 没有 expected_generation，不能称页面 scope 或 CAS 已贯穿。原 r42 wrapper 不重放，原 mutation/receipt 保留。

行／文字几何目前仍来自手工 SettingsWindowLayout 与 root painter；新逐字段证据须接同一真实 UiSurfaceFrame 的 arranged node、render command/text line、hit/focus、window metrics 及源身份，Parent 负责测量/painter/Settings native case，原 public owner 负责 slots/codec/ACK/export。只读再次确认 source PropertyRow 5ab622…／r392 staged bf29… 不一致；未改 F 盘产品、LevelEditor guard、nonce、公共 Rust、窗口、主题、store 或服务。 own rev54／5页720对象、Shared2／4masters／34colors／3typo 仍真实存在；本轮未改 Penpot，r54九图和 r53 whole-page hash 失败均保留。

本阶段只有文档／能力接入合同，无 Rust／正式插件测试或导出、无 managed resource prepare、正常 EXE／actual150%DPI／双端验收，passed=false、accepted=false，释放0 B。

r54 最小公共交接口已具体化为 [Card 源桥接合同](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/repeat-prototype-handoff-r54/source-bridge-contract.json)／[方案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/repeat-prototype-handoff-r54/source-bridge-proposal.md)。SP-C6 现有 status→typed payload→metadata TOML 链完整保留，消费者从同份 typed payload 取原动作和 locale/generation。缺口是唯一 canonical Card／真实 repeat prototype、持久 pane surface 及通用重复项绑定承载；不加 plugin DTO/revision/store，不改原 operation tuple，不给 ModulePluginListPanel 根伪造 Card metadata。

Parent 待核对的共享可选字段为 repeated_item_binding: Option<(SharedString, UiVirtualListNodeBinding)>。完整 plugin_id 是逻辑键，canonical instancePath 保留原作者调用点，物理池槽及其 assignment generation 不冒充业务 identity。复用现有 sequence／surface currentness／cache、实际文本测量及单 scroll；Runtime 只有 fixed extent，第一版统一实测最大 desired height 的视觉差异保持待验，不新增 repeat kind 或硬编码高度。

公共 catalog 当前 e4bba09c3ba16d55b90eabb5a821ab3e76eb0606f9afc17e9f656e2d16fe0a99，68 components，NumberField 10 props／PropertyRow 4 slots／Field 描述符与起始 4c2f849c… 相同。20 源输入仅 19 当前匹配，未声称 source-ready；无公共文件补丁。现有 i64／f64 schema 只有 min/max/step，没有 precision/large_step，也无 u64 provider；完整整数沿 Field 原 typed commit，Number 需真实值和显示策略。Shared rev2 的 4 masters 没有 Field／Number，等待原 owner 正式导入身份，不复制假控件。

34 colors／3 typographies 已发布，21 个 master style ref ID 全为 null，不能以键存在声称共享绑定已证明。原 trial 两个显式颜色引用及 33 copies／419 Shared 后代源 refs 保留。同五页新 9 图、实际源 SVG 身份和完整被动前后对账见 [逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/repeat-prototype-handoff-r54/visual-review.json)／[readback](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/repeat-prototype-handoff-r54/post-capture-readback.json)。本轮不改 Shared／主题，不使用桌面，不跑 Rust／正式插件 suite／generator；r53 whole-page hash 失败、CJK／正常 EXE／actual150%DPI／跨两族主题 A/B／reset 仍 open，passed/accepted=false。

r53 公共样式消费已实际发生：Shared rev2现有4个原生主件、34颜色、3文字样式；本族仅在原受控板和原默认Button leaf用原生picker链接`editor.surface.0`/`editor.surface.1`，局部覆盖后重选同一原style，保留原component/shapeRef/source metadata。34颜色及alpha均匹配`editor_tokens.zui` SHA`dc23135b1e349773266e09cfb354c01bf047774ac0beb70a6a129e5ab4f66743`。未生成本地style、复制main或修改Shared；主件源码页hash仍未改变，不能宣称公共master已绑定全局style。

新增待原公共owner处理的具体合同：①原生库同步导致本族三业务页全hash变化，虽然25标签/18旧动作/419源引用仍有效，完整保全失败不能关闭；②`penpot-capture-validation.ts::singleSolidFill`拒绝直接fill引用，需要正式入口证明解析/校验/源映射责任，不据静态路径伪造导出失败；③已发布Typography是`gfont-roboto`，现有源使用`custom-495ff285-7c71-49bc-bc4e-df20017e2d66`，资源字节/度量未对账，暂不套用。具体前后字段和失败见[同步审计](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/shared-style-linked-r53/library-sync-delta-audit.json)。这三项不授权本族修改公共guard、font、master、主题或codec。

当前本族rev54／5页／720对象、33外部根／419源引用；局部重选style不等于组件Reset，保存重开不等于两族全局A/B或正常EXE。长中文/空value、正式控件、Dialog主/禁用变体、canonical wrapper及正常producer等原合同继续open，passed=false、accepted=false。以下r52及更早状态保留历史。

r52 当前消费状态为本族rev49／5页／720对象、33个真实external native copies／419源引用节点，Shared仍rev1／4 masters／0 colors／0 typographies。本轮4个Button实例及其8个节点均保存真实source file/component/library revision/shapeRef和原source metadata，关闭重开两次通过身份与覆盖值核对；不改变Shared或原业务组件。详见 [r52保存回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/dialog-linked-r52/save-reopen.json)。下文rev35/29 copies是历史轮次，不是当前状态。

新增的精确公共变体依赖仍归原owner：UE `Dialogs.cpp:194` YesNo使用primary Yes再No，右对齐UniformGrid（150–160行，slot padding8和StandardDialog min尺寸）；`SNewPluginWizard.cpp:280–290` 使用primary Create Plugin、右对齐、padding10并绑定CanCreatePlugin。现有Button固定240×32且没有已发布primary/disabled/AutoWidth，不能把本族Create opacity0.5当作这些变体交付；不修改源master或另造Button。通用Dialog需沿既有SP-C4承载真实模态、默认焦点、键盘、关闭/取消、clipboard及焦点返回；Directories关闭沿Parent Dock。四板的正文及动作语义仍由本域提供，真实enablement/生成/metadata提交能力与正式源导出均独立保持open。

接续 [本族设计](settings-plugins-design-contract.md) 与 [Parent 交接](settings-plugins-parent-handoff.md)。本族聊天 `01a16d53-33db-7fb2-8f59-11118c3ec933`；Parent 分工依据 [当前统筹记录](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)。Parent followup-03 已应用 r34/r39 的 raw getter 与 host metadata 源扩展；下列公共 UI/宿主接口和验收约束仍待交付，不能把新增源字段当作公共控件发布。本族未改 UI catalog、公共 master、tokens、Starship 或插件导出。

r43的Shared已发布为rev1/isShared=true/四主件，本族已正式连库并在受控页保存8个native linked copies；r48在原Plugin Browser业务板保存7个，r49在原Preferences/Project板再保存14个。当前文件rev35/5页/712对象，合计29根/411复制节点，原业务组件及受控页保持身份。r49原生flow刷新回到原row-gap0后，两页标签/引用/连续排布各经两次fresh断言核对。旧rev0/空库/未发布以及rev22/rev27描述是对应时点历史。colors/typographies仍0、canonical sourceHash/owner和正式codec/产品合同仍待原owner；不能将试接作为产品实现。

## 实际分配与当前阻塞边界

本族实际 file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402`、初始 page `b4a1e3ce-9e6a-81fe-8008-d69d60e4e403`，归属已与 Parent assignment 对账。[真实文件入口](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60e4e402&page-id=b4a1e3ce-9e6a-81fe-8008-d69d60e4e403)。初始 API 为 revn=0、vern=0、一个 root frame。随后仅在本文件组织四窗口页面及 planning provenance，实际 update-file HTTP 200，独立 get-file 返回 revn=1/vern=0；初始 page ID 保留。

| 已保存页面 | 实际 page ID | 实际结构草稿 board ID | 当前内容 |
| --- | --- | --- | --- |
| Editor Preferences | `b4a1e3ce-9e6a-81fe-8008-d69d60e4e403` | `92bc794d-9419-4284-bb9c-282b99afb641` | 搜索/分类/详情/状态的 native flex；10 个 User 设置键的来源映射。 |
| Project Settings | `49ff1071-f83b-4b72-bb6e-043a3326ec82` | `c5edfcc8-00e6-4419-a463-3324207d5335` | 同类域结构；6 个 Project 设置键，reset scope 明确绑定。 |
| World Settings | `a705d2f3-663d-4a71-9d71-a67cf2e78ab8` | `cc3d2d68-ecfd-44aa-9a7e-fcf2a10b6c27` | 上下文区域和“无对应字段”设计态；未生成假的 World 字段。 |
| Plugin Browser | `c6a45a2a-60ca-41fb-a1c4-c5164da3cad3` | `9d912f75-c86c-4b3a-878d-1007947cbf81` | 分类/列表/事务状态域；五个实际 manifest metadata 设计 fixture。 |

`penpot-page-setup-r7.json` 保留保存 ACK、请求 hash 与页面读回；`penpot-browser-r8/receipt.json` 保留实际无头 Chromium 的原 context 关闭、fresh authenticated context 打开同文件/四页面、两次 UI 截图和进程身份。已人工查看重开图，四页面正确、画布为空，未见 JS pageerror。它验证页面组织与访问，不验证任何窗口设计的保存/重开、组件/样式或正常 EXE。计划元数据使用独立 `dev.zircon.settings-plugins` namespace，不冒充 ZUI 插件导出 metadata。

后续 `penpot-domain-flow-r10` 保留四板真实保存请求及正常验证 ACK，初次独立读回 revn=3/vern=0、native flex 身份均有效。16 个设置行明确 schema-default fixture、写入/重置 scope、来源 key、预期组件和现有/缺失 action；五个 tile 使用源 manifest/hash，enabled/load_state/operation 均为未观测，不制造运行状态。文本和区域是可编辑域草稿，原生公共组件实例、components/colors/typographies 均为 0，没有发布新的 master 或平行主题。

`penpot-domain-flow-r10/browser/receipt.json` 记录原 context 关闭后 fresh context 逐一打开四页，八次 SVG 板身份命中且 JS pageerror 为 0；八张完整 UI 图保留。逐图查看后确认是文字/区域草稿。World/Plugins 的 initial/fresh 图 hash 相同；两个设置页不同，文字栅格观感存在差异，字体就绪尚未验证，不能宣称视觉一致或误差达标。普通 Penpot 字体只是中性结构草稿，未替代 Parent 的公共 typography。r10 最初行计数用错 JSON namespace key 而得到 0；实际 metadata 存在，r11 按真实 `devZirconSettingsPlugins` envelope 检查 16 行/五 tile，保留原结果。

首次实际缩窄暴露草稿排序错误，失败图 `browser/flow-edit-900x620.png` 保留：页脚跑到顶部、分类及属性倒序。已按源码追踪到 Penpot `flex_layout/layout_data.cljc:419` 对普通 row/column 反转输入 children 的原生约定；草稿按视觉顺序直接追加 stored shapes，初始坐标掩盖了缺陷。r12 的 `reorder-children` 虽 HTTP 200，JSON 的 `any` UUID 字符串未转换，读回仍未修复，不能算成功。r13 改用正常 schema 解码的 `mod-obj/assign`，先核验一个根再修复其余容器；最终 revn=6，71 个 stored order 全部正确，所有 ID 与域 metadata 保留。

修复后 fresh 无头 context 的 `browser/fixed-flow-r14-receipt.json` 对 Project 草稿实际编辑 1280×800、900×620、640×520，三态各七项顺序/左右位置检查通过，JS pageerror 为 0，UI 与 API 最后均恢复 1280×800。已查看两张缩窄图，顺序正确、文字随区域缩窄换行；这不是公共 PropertyRow、全部文字行框/裁切、滚动或正常 EXE 的验证。其余窗口的缩窄、中文、所有交互、实际 150% DPI 与正式双端逐图验收继续 pending。浏览器/Node 均关闭，PID 独立复核后登记产物清单。

同一 Plugin Browser page 后续保存四个相关域内容板。它们不是公共 Dialog master，也没有可运行按钮；普通文字/流容器只表达来源、可写性、请求状态和预期槽位。`penpot-dialog-domain-r16` 保留源指纹、请求/ACK/readback 及两 context 的保存重开图。

| 域内容板 | 实际 board ID | 取证边界 |
| --- | --- | --- |
| New Plugin | `0abd721a-403d-43a6-a802-5c2e2f9be55d` | 1000×750 结构草稿；ID、必选 kind、native=false、引擎 checkout 与六文件 write-plan 来源。未调用 scaffold。 |
| Plugin Directories | `af252d36-d7d1-4f0b-91ed-065d36b6b0de` | 750×600 的无项目 fixture；无伪造路径或可用多根 setter，现有固定 native root 的源码合同在 metadata。 |
| Plugin Properties | `858bc131-0a68-47c8-ab3d-8acf82516aa7` | Material Editor manifest 的六个只读 metadata 字段；740×640 是草稿内容尺寸，不是已确认 UE 默认窗口度量。 |
| Beta 启用确认内容 | `b11c37ed-3ccb-4653-876f-451d6af0fb44` | 实际 Animation manifest 的 Beta 标记 fixture；740×360 内容区。没有 live enablement request，confirm/cancel/focus-return 未执行。 |

`run_new` 的 kind 没有默认值，不能把 importer 自动选中；native 默认 false。Properties 不写生成清单，目录页不补不存在的 User/命令行/环境根。生成器仍只适用引擎 first-party checkout；UE 模板缩略图、Content-only/普通用户项目模板与目录编辑的差异继续留在 SP-C7，不以本内容板关闭。

逐图复核又发现标题/版本与确认文案曾用空格分隔。r18 保留旧文本 ID，改为六个横向流组和独立文本节点，图关系及无多空格的 content 检查通过；r19 的真实画面检查仍失败：Beta 旧 `positionData` 继续画原 `Cancel     Enable`，与已存 `content=Cancel` 不一致。该失败图与读回保留，不能用内容检查替代画面。r20 按原生文本失效规则清除这六个改文案节点的缓存，不增加坐标补偿。r21 在两个独立认证 context 读取 revn=11，六组文字内容与相对位置检查均通过，无 JS pageerror，未需强制尺寸重排；四张原图逐图复核确认重复文案消失。浏览器/Node 关闭且 PID 独立复核缺席。此项只关闭草稿的文本缓存缺陷；公共实例/style、Dialog host、真实 mutation 与完整验收仍 pending。

Parent 公共文件 `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` 的同轮只读 API 返回 revn=0、isShared=false、原生 components/colors/typographies 均为 0。它是接入目标，尚不是可实例化的已发布库；不能填入虚构 master/style ID。JSON API 的页面索引字段是 `pagesIndex`，与 Transit 的 `pages-index` 不同，首次 r5 的页面计数解析未完成，后续按实际 envelope 核验，保留原回执。

文件已分配后可用隔离 API/无头浏览器编辑本域文件；共享桌面与系统 DPI 未授予时段。当前页面组织及首批域 region/来源映射已保存重开，下一步接入 Parent 发布的原 master/style 并按同一流布局完成状态和正式设计；中间草稿不计组件实例、导出或画面验收通过。

## 本域 PluginMetadataCard

本族 Plugin Browser page 已有一个本域原生组合，未提升为公共基础组件、catalog entry 或主题。component UUID `81548358-cc2f-47cf-856d-39b395bfecfa` / version 1；main 保留 AI tile `d57fd7f1-bf28-4c97-bc6b-0a52ea4696f1`。四 copies 沿用 Animation `89b0f69b-d824-4358-8eec-cf992fb2c882`、Graph `5efccb17-7874-4bdc-be4b-362a62329bfd`、Audio Importer `4cf5e3e8-2d1c-400b-a37f-25e72745b7e8`、Material Editor `f1cdb958-984b-4669-ab59-b291651d7740`，各 8 节点。全部 40 个原对象保留，32 个 copy nodes 原生 shapeRef 对应主件；只覆盖实际 manifest metadata 文字与本域名称。

engine 原 owner 是 `module_plugins_body.zui` 的动态 `module_plugin_list_slot`，尚无独立 ZUI wrapper 或官方 codec 导出。公共 primitives/style/font 来源、SP-C6 同代 metadata/status/operation 仍 pending；原 namespace 只声明设计 provenance，不冒充官方导出。

`penpot-domain-component-r26` 保留正常请求/ACK、五份 fixture hash 和三次隔离浏览器尝试。第一次在折叠层级定位超时，修正为原生 subtree 展开；其后虽链接检查正确，逐图仍检出四 copies 文字全部变成 AI。实际 `libraries.cljc::text-change-value` 要求 text-content-text 等细分 touched 标记，初始只有 content-group。已恢复 12 字段并仅失效对应文本缓存，保留失败。最终两 context 的 API、实际 SVG 和 UI autosave 后各 20 字段正确，40 个 shape 对应稳定，最终 revn=15、无 JS pageerror；六张修复图逐图复核。它验证本域组件和文字覆盖，不证明公共样式/重置传播、真实启停、官方导出或引擎窗口。

## 最小公共接口

| 合同 | 本族实际消费者和输入 | 需要公共 owner 提供的能力 | 本域承担的语义与拒绝条件 |
| --- | --- | --- | --- |
| SP-C1a：组件和样式身份 | 全部四窗口；现有 component ID/document、原 control/node/slot/binding/event | 可用 library file/component UUID、main instance、版本、实例化入口；token/style ID 与源 token 路径映射；保存关闭重开后身份不变，导出保留引用。 | 本域记录具体实例/变体与源 key，不复制公共 master，不自建 palette、字号或密度 store。尚未发布的条目标 pending。 |
| SP-C1b：PropertyEditorRow | Preferences、Project、World 的标签、说明、单个 value editor、来源与局部 reset | 扩充现有 `WorkbenchPropertyEditorRow`：保留 value 单实例限制；给出 description/source/reset 的 slot 或受支持 variant，label wrap/内在高度、值列分隔与可访问标签。长文本不被固定 max105 与固定行高裁掉。 | Host 决定 schema、可写 scope、实际 source/default、reset 可用性；公共行不自行改配置。继承值不能因 reset 点击而误删另一层覆盖。 |
| SP-C1c：TreeRow / ListRow / section header | 设置分类 All/root/leaf、插件来源/分类、breadcrumb、可折叠属性节 | 稳定 item ID 的 selected/focused/expanded/disabled/count；键盘 disclosure 与 selection；文字按实际字体测量；滚动到选中项。复用 PanelHeader 的 title/actions slot。 | 本域投影 category key path、plugin ID 与真实计数，过滤后恢复合法选择；不把 translated label 或列表索引当 key。来源分组只用已核对 provenance。 |
| SP-C1d：Field / NumberField / Checkbox / Dropdown | typed settings、搜索、枚举、值输入、插件开关 | 明确 edit/commit/cancel、IME composing、disabled/invalid/focus 状态；NumberField 的可键入提交与步进接口；Dropdown 的 popup anchor 与可用 viewport、关闭/返回焦点。事件只发生一次。 | 本域按真实 schema 校验、派发 intent、核对 binding/catalog generation。搜索不持久化；number step 与值默认分开；按键捕获不能重复进入搜索。当前 action 只有 number ±，不能声称已能输入数值提交。 |
| SP-C2：窗口与滚动宿主 | UE EditorSettings / ProjectSettings / WorldSettingsTab / PluginsEditor；两个不同比例侧栏 | 实际菜单/命令/Tab/Dock 注册和 activate/focus 合同；内容可用矩形；Sidebar drawer 停靠/展开、Splitter min size、每区单 scroll owner、弹层 viewport。 | 本域 body 保留 settings 0.20 drawer、plugins 0.35 splitter 与 World 0.5 属性列的源目标；不改系统 DPI，不发明无依据的窄屏断点。新窗口注册前保留旧 Preferences/Module Plugins 消费者。 |
| SP-C3：共享主题及覆盖 | `editor.appearance.design_tokens`，设置和主工作台至少两族 | 现有 `EditorDesignTokens` 的字段范围、实际加载优先级、Penpot shared style/token ID、局部 override/reset 与持久化合同；一次共享 style A/B 的两族捕获时段。 | 本域通过 SettingsMutationCoordinator 更新 typed token 值，不增加主题配置文件。显示 desired/source/persistence，不能把 DesignTokens 强转普通 enum。公共值和覆盖值分别取证。 |
| SP-C4a：通用 Dialog / popup / focus-return | 插件 maturity/依赖确认、批量 reset、enum/chord/color/path 等适用弹层 | 现有模态/弹层 master、焦点栈、Escape 优先级、confirm/cancel 单次返回；发起 source/control/instance 可恢复，目标消失时有合法 fallback。 | 本域携带实际 key/plugin/scope/context generation，确认前重验。取消零 mutation；无真实路径/颜色能力时不伪装完整 picker；提示不靠解析英文错误字符串生成状态。 |
| SP-C4b：持久化/恢复/正常重启 | 设置 health 与插件完整 enablement operation | 公共状态区/错误显示/可执行动作 slot；正常 dirty document close、fence、退出/重启/重开协议，实际进程与 EXE 指纹；同一个操作的 retry/revert/recover/open 继续身份。 | 本域区分 Unavailable/Ready/Queued/Durable/PendingAdmission/Terminal 与完整插件 tuple；不把所有 RestartRequired 画成“保存后重启”。业务恢复参数留在 host，不泄露 admission 内容。 |

这里没有新增公共 palette、列表、输入控件或弹窗的实现请求。优先扩展已有公开组件/宿主；Parent 若已支持某项，交接现有属性、slot、动作及 identity 即可，本域按实际接口适配。

## 已核对的现有输入属性

本轮进一步读取实际源资产，C1c/C1d 不是“公共控件不存在”的请求：

- `workbench/primitives/inputs/number_field.zui` 已有 `editable_text=true`、value/value_text、min/max、step/large_step、precision、validation_level、layout_stepper 及真实 token 引用。本域需绑定实际 schema/value；模板中的 value=42、范围0..100、precision=0 不得带进 viewport 小数设置或充当产品数据。缺口是设置表面尚无 numeric typed commit action 及完整接线/验收，不是要求新建第二个 NumberField。
- `components/workbench/primitives/inputs/field.zui` 已有可编辑文本、validation 与交互/focus 属性；`workbench/primitives/inputs/dropdown.zui` 已有 value/value_text/options、popup_open、selection_state 等。source key 与翻译后的显示文本分开绑定；locale options 来自 `en`/`zh-CN` schema，模板 `default` 不是实际选项。
- `components/workbench/primitives/data/tree_row.zui` 已有 expanded、tree_depth、selected、focused、hovered、inactive-selected 样式和 text_overflow。复用现有属性，补充真实 item key、分类角色/计数与事件交接；不另建 SettingsTreeRow。

以上仅为源属性核查；不能据此推断 IME、键盘提交、popup focus-return 或实际设置持久化已在正常 EXE 验收。

## 数据和业务边界

- **SP-C5 / World**：Parent 协调活动场景/Inspector owner，确认每 World 的 authority、持久化、undo/redo 与运行时应用。`PhysicsSettings` 有真实 manager/ConfigManager，但不因此变成 per-scene WorldGravityZ；scene entity/camera/volume 的后处理继续由原 owner 持有。没有等价字段时记录 gap，不制造表单。
- **SP-C6 / plugin metadata**：请求现有 `EditorPluginStatus` 发布链补带 manifest 已有 version/category/description/maturity 与同次 source/catalog revision，并提供生成来源/可写性，供 Properties 只读状态使用。保留 enabled、load_state、required、features、diagnostics 和 enablement operation；图标/作者/支持 URL 没有来源则保持缺失。`package_company` 是包坐标字段，不充当作者；`content_roots` 已存在，但不等于 UE 的可编辑 CanContainContent bool。本域不自行扫描 TOML 拼另一套 catalog。
- **SP-C7 / 目录和创建**：目录编辑需要现有固定 native root 之外的真实多根权威合同。New Plugin 已有真实 scaffold，但影响六个共享文件且只尽力回滚；由 Jenkins generation owner 提供只读计划、源 revision、完整变更集及恢复回执。UI 创建/校验/聚焦构建/发现/注册是不同阶段，不用创建成功代替 loaded。
- **专用表面**：本族没有需要新时间轴/曲线/播放或节点图的直接设计需求。未来主题/插件预览若确需这些表面，复用动画聊天 `01a16d51-741b-73b3-a61a-2189cd38f81e` 和材质聊天 `01a16d50-6c7f-7370-853d-4581184b66a1` 的现有接口；不在本族创建替代组件。

## 交接顺序与证据

Parent OVW-27/40 覆盖补充的具体合同仍复用 C1/C2/C4：Config Editor 需要原 authority 的逐层 presence/raw value/source/file/writability/health 同代投影；Keyboard Shortcuts 需要同一输入组件的 chord capture、冲突弹层焦点返回及整 map mutation/undo，不能另建 Keymap store。Localization target/culture/job 和可编辑 Device Profiles 暂无等价 authority 合同，须 Parent 对应领域 owner 决定；现有 i18n locale 与不可变 RHI device facts 不能替代。字段/源码依据见设计合同第 9 节，新增 SP-014～017 全部保持 open；没有额外 Penpot board、共享组件或实现通过声明。

先交 C1a 身份/共享 style 和 C1b 行接口，再接 C2/C4 的真实宿主与焦点；本域独立准备四窗口内容、17 keys 的 schema/action 映射、搜索与作用域状态。主题修改和正常重启必须等 C3/C4 的两族/进程合同。C5/C6/C7 保留独立 gap，不能阻止有真实后端的基础设置内容设计，也不能由基础内容的局部通过覆盖。

正式双端 case 继续复用 LayoutReviewCase 与原生 nonce/generation/source/process 证据协议。四尺寸/DPI、通用状态及特有交互见设计合同；资源 readiness 与捕获有效才可 passed，双端比较和逐图复核通过才可 accepted。当前只有源码核查、域结构草稿保存/重开及诊断图，`passed=false`、`accepted=false`。

官方导出还有明确的 family closure 依赖：`zui-layout-real-session.ts` 的 `--source-closure` 仍调用 `prepareRealWorkbenchInput`，其 canonical owner 断言固定 Level Editor；已部署正常 producer 同样固定该 owner。SP-C1/C2 需由原公共 pipeline owner 提供本族受支持的 owner/host/source fingerprint/actual-case 输入，保留既有全主窗口约束，不用改标签、拆叶或 metadata 代替原始资产导出。具体源码指纹、两个实际入口及源资产稳定 ID/五条 Plugin events 见 `source-asset-map-r25.json` 与 `normal-capture-readiness-r25.json`。本族没有修改公共 pipeline、catalog 或捕获协议。

## followup-02 最小域适配

Parent 已移交 SP-C3/C6 最小源码范围。r28 只在原 settings mutation 文件形成显式 scope reset 候选，保留 catalog/coordinator/现有调用签名；六项真实 authority/coordinator 回归尚未执行，不作为四窗口交互或共享主题通过。原请求的 Jenkins 输入捕获失败及原 identity 查询待授权均保留，公共 pipeline/主题/组件没有改动。

SP-C6 公共 DTO 扩展须先协调，具体未应用 patch 及六个构造消费者在本任务 `scoped-reset-metadata-r28/plugin-public-contract-proposal.json`。拟用可缺失的 package_metadata 传递同次 status 的 version/category/description/maturity，missing package=None；不发明 revision、可写性、作者或 URL。官方 queue 第 11 次投递因 100 条上限失败，完整交接见 [Parent 记录](settings-plugins-parent-handoff.md)。

## r30 可消费 mutation 入口

SP-C3 域侧当前候选为 `EditorHostEventController::set_setting_in_scope(key: &str, scope: SettingsScope, value: SettingValue)` 与 `reset_setting_override_in_scope(key: &str, scope: SettingsScope)`；旧六类 setter 签名、默认 scope 和原 action IDs 保留。公共语义子树应提供 page/category/key、typed value、value source、edit scope、locale/代数和存在性；scope 来自宿主 context，不能把 resolved source 当写入目标。仍复用同一 SettingsCatalog/SettingSchema/MutationCoordinator 与原 receipt/persistence 状态，不另建配置或主题。

当前十二项 set/reset 回归只有静态检查，托管 Rust 未执行。UI/项目绑定代数的旧事件拒绝、presence/重置禁用态、原始数值文本提交、undo 与重开需要公共 host/adapter 后续接入；本域没有增加 CAS/fence 参数，不据此关闭 C1/C2/C3。当前补丁、API 和验证范围在本任务 `scoped-set-contract-r30/typed-scope-contract.json`。

## r32 额外覆盖的消费接口

OVW-27/40 继续由本族牵头配置/生产入口；资源 08 持有 OVW-35 Struct Viewer 与 OVW-38 StringTable 文档/作者及引用事务，Parent 持有 OVW-28 设备/部署与 OVW-39 Source Control/Jenkins。准确的输入、确认/取消/失效、保存及焦点要求见 [设计合同第 10 节](settings-plugins-design-contract.md#10-额外覆盖的跨域消费合同)，不扩展共享源码写范围。

仍沿 C1 的同源 TreeRow/PropertyEditorRow/value 与 chord-input、C2 的真实文档/窗口及 focus-return、C3 的原 scope mutation、C4 的原事务/错误弹层消费；不新增公共 master、主题、Keymap/locale/profile store。Config 与快捷键都需要同代的**目标层 raw/presence**；resolved value 只说明 winner。反射类型另需实时 registry generation，目录 fingerprint 不能代替当前性；StringTable 的 asset/key/reference 回传需资源 owner 交付，不能沿当前 editor/default 表字段随意扩展。

本轮没有额外 Penpot board/component/style、ZUI 导出或 native case。沿用 file `b4a1e3ce-9e6a-81fe-8008-d69d60e4e402` 的四页八板、`PluginMetadataCard` UUID 与原四 copy；r31 actual revn15 及 r26 保存/关闭/重开为原有证据，不把新增消费合同声称为真实控件或产品能力。缺 provider 的状态继续 open，插件、托管 Rust、正常 EXE 与双端逐图验收继续 pending。

## r33～r35 的最小域接口交付

SP-C3 增加原 `EditorHostEventController::set_numeric_setting_in_scope(key: &str, scope: SettingsScope, value: &str)`，只解析当前 Int/Float schema 后经原 typed scope 路径写入；不 clamp，不改变现有 setter/事件。数字文本控件可消费该候选，错误保留输入及原焦点；公共 host 仍负责页/作用域、旧事件及 project binding 准入，未实现 expected-generation CAS 或文本事件接线。当前十五项回归仅静态候选，精确源码/patch/guard 与测试身份见 `numeric-scope-adapter-r33`、`jenkins-route-reconciliation-r35`。

SP-C6 的完整精确补丁已扩为 `plugin-metadata-chain-proposal-r34/plugin-metadata-chain.proposed.patch`，SHA-256 `2810e0c2ac85f6ab7e2468d686460ef33c57279acb2b71ae2d52eba35ebb0292`。拟新增公开 `EditorPluginStatus.package_metadata` 及类型/reexport，补原 builtin/native/missing 构造与三组 fixture；实际 status Arc/live clone 路径保留，不另存状态。需要 Parent 协调后才应用；只读适用性和候选格式解析不算编译/guard/引擎支持。旧 r28 提案作为历史保留，缺作者/图标/URL/生成可写性、混合 catalog 与已加载 binary provenance 继续具体记录，UI 组件不补推断值。

## r37 原详情控件的恢复态合同

获准的原 `module_plugin_projection/details.rs` 已形成最小候选：详情 Apply 的投影、Tab 准入及提交均消费同一 status report 的同插件 recovery，恢复期间不产生 intent。保留原 control/event/qualified action 身份、feature 选择与真实状态，追加原事务原因；已聚焦 Apply 时返回原 features 列表。恢复解除后使用新报告/新 intent，另一插件不受影响。无新公共组件、DTO、主题或恢复事务。

此合同可供原列表/详情控件消费，尚无编译或产品输入验收。三项新增回归及五项旧详情回归共八项待 Jenkins，原十五项设置 mutation 也未执行。r34 metadata 提案字节不变，详情 fixture 基线随本次域补丁前移、apply-check 仍有效；具体 current hash/combined diff/guard 在 `plugin-detail-recovery-r37`，公共迁移仍待 Parent 协调。12:40Z 真实共享库仍 rev0、无 component/style；不据此新增 detached 或重复 primitive。

## r39 同源属性值的逐层读取提案

C1/C3 的目标层 presence/raw 与 Config/快捷键消费有同一缺口，现交原 `ResolvedSettingValue` 的精确两文件扩展提案。拟新增 `override_in_scope(SettingsScope) -> Option<&SettingValue>`，同原 batch 一次锁捕获三层原值，保留 key/effective value/source/generation；不另造 SettingsScopeRow、分层 store 或 resolved map 的 User 副本。只有选中 key 的实际存在值被 clone；Parent/core owner 需确认捕获成本和 DTO 私有状态扩充。

精确 patch 为 `scope-projection-proposal-r39/resolved-scope-values.proposed.patch`，SHA-256 `38f618eb703ce78b97dd314b2f8ab901257d17805a14f6fb678a9c92fd88ee2a`。core 源码未改，三项候选回归未执行；此提案不是组件/native 接口发布或调用方完成。公共语义子树仍需同时保留页面 edit scope、source、同代 raw/presence 与事件 currentness；文件/健康/可写性及 mutation generation fence 不由这个只读 getter 提供。

## r41 SP-C7 authority 与不可用状态消费

按原消息 `01a16dbc-2adf-7911-8337-3d49d946952c` 接收原 owner 回执，SHA-256 `2b247a0384791c87cfb6fea5920f70aaba82ebdeece6f91b7788b82d7f8a48f4`。generation adapter/authority 仍归 `01a12af6-7511-70d1-a452-61b80678eb56`，Parent 协调真实 scaffold/catalog 源 owner，本族不写生成器/共享源或派发请求。当前 selected generate 仅 source-index/render-template；原 MUTATIONS/writer_hold 不表示 plugin scaffold 已接通。所需 catalog 标记实际缺失，属于 source-backed 静态前置错误，未运行生成器。

公共 Dialog/字段消费的当前状态只有固定 `<project>/zircon_plugins`、kind 必选无默认、native=false 以及 authority 未就绪的真实不可用原因，不能把草稿 label 映射为可执行 plan/generate/retry。未来计划应携原业务 request、planRef/digest/source/generator 指纹、11/13文件全部 create/modify/touch-unchanged 与目录/absence/diff/结构错误；服务器内部阶段与逐文件 journal 保留原 operation，不能改受理 payload 或重放 pending。恢复只覆盖仍匹配自身 postimage 的路径，逐项 restored/conflicted/unknown 与 native/EOF/未启动状态来自原 authority；未恢复继续 open。本项只是消费合同，未部署组件/绑定/API，也未新增多根 setter、SDK 模板或平行 generator。

## r42 实际消费与待协调的最小接口

原 r34/r39 补丁已由 Parent 应用，12 份实际 after hash 均核对，不再次 apply。域内 reset 已读真实 `override_in_scope(edit_scope)`；目标层为空时不进入持久化准入，目标层存在时沿原 coordinator。两个新增回归及一个加强回归使 mutation 候选共十七项，未执行。此修复不提供 UI/resetEnabled、旧事件 fence、undo 或主题传播；完整证据在 `scope-consumer-r42/consumer-contract.json`。

SP-C1/C2/C3 的新宿主接口精确未应用提案 SHA-256 `423e5651517502c8d940b272038f06758ef7f4d9d8783c273407b09b64034508`：在原 `settings_projection.rs` 给同一 selected-key batch 配对 `category_id` 与显式 `edit_scope`，借用原 row/raw，旧接口不变。没有新增必填共享 DTO/注册字段、事件或第二 store。Parent 须协调 page-scope 调用方、原 catalog/locale/权限、project binding 与 generation admission；当前只有 format/apply-check，canonical 源未应用。

SP-C6 的剩余承载点明确为 `layouts/windows/window/host_data.rs::ModulePluginStatusViewData`（可选 typed metadata）、`app/pane_data/view_rows.rs`（从原 status 的同一行复制）、正式卡片及 `retained/ui/data/module_plugins.rs` 原 native 投影。原行高/滚动须按共享卡片布局容纳描述，不展平为诊断或改名字段。保留所有旧字段、构造点、action、cache、operation tuple；metadata None 不造包，version 不是二进制来源。以上共享路径未写，等 Parent 的具体接口/官方 source owner 交接。

## r43 四个主件实际身份及slot核查

共同来源 file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12`/page `b4a1e3ce-9e6a-81fe-8008-d69d607aac13`，实际库rev1/isShared=true/41objects。所有原main只读；本域未创建新的同名主件。

| 主件 | component UUID | main UUID | 源检查与最小消费 |
| --- | --- | --- | --- |
| WorkbenchButton | `6ddc248b-bd35-80c5-8008-d6c8c01096f3` | `6ddc248b-bd35-80c5-8008-d6c8bd9b057f` | `components/workbench/primitives/inputs/button.zui`；root controlId/catalog一致，两个copy验证text覆盖，不提供域action。 |
| WorkbenchSearchInput | `6ddc248b-bd35-80c5-8008-d6c8c3ce892e` | `6ddc248b-bd35-80c5-8008-d6c8c0f9c196` | `workbench/primitives/inputs/search_input.zui`；两个copy验证placeholder覆盖，真实query/搜索action仍需接线。 |
| WorkbenchTreeRow | `6ddc248b-bd35-80c5-8008-d6c8c7a79944` | `6ddc248b-bd35-80c5-8008-d6c8c452bf0c` | `components/workbench/primitives/data/tree_row.zui`；两个copy验证text，固定行高下长中文换行/溢出open。 |
| WorkbenchPropertyEditorRow | `6ddc248b-bd35-80c5-8008-d6c8cab1e7d0` | `6ddc248b-bd35-80c5-8008-d6c8c84d848f` | `workbench/inputs/property_editor_row.zui`；原`value_slot`/required value真实存在，两个copy仅验证label，accepted editors尚未发布。 |

源路径均位于 `zircon_editor/assets/ui/editor/`；四份当前源 SHA、root字段、canonical-record、catalog props/slots完整记录于 `shared-linked-consumption-r43/source-slot-audit.json`。记录存在不证明sourceHash/owner已封闭；发布canonical-record没有sourceHash。本域保留原sourcePath/metadata，未换映射。PropertyRow value只接受 `WorkbenchCheckbox/WorkbenchDropdown/WorkbenchField/WorkbenchNumberField/WorkbenchRangeSlider/WorkbenchSlider/WorkbenchToggle`；不能拿已发布SearchInput替代，或发明未发布UUID。

受控 page `1ca0591c-2856-49b0-9d7e-eeb076bafe03`/flow board `4829a7b5-8807-4e66-b34d-b94fbb8807fe` 保存8个external copies及全部shapeRef；默认4个和文字覆盖4个经两次fresh context重开核验。own rev22/五页/367objects，原local PluginMetadataCard不变。库记录直接连接isIndirect=false/rev1；无detach/复制main。原生copy的pluginData原样含main辅助ID，正式codec须证明shapeRef解析；不自行重签。跨文件试接成功不关闭正式跨文件导出、native semantic bindings、override-reset、两族全局样式A/B或EXE门。

## r48 原Plugin Browser消费后提出的公共变体需求

原业务page `c6a45a2a-60ca-41fb-a1c4-c5164da3cad3`/board `9d912f75-c86c-4b3a-878d-1007947cbf81` 已保存Button×2、SearchInput×1、TreeRow×4，均引用上表Shared rev1的原component/main。自有文件现为rev27/5页/458对象；原受控页8个copy未动。7个新根及91个节点的shapeRef、源metadata和标签覆盖经过两次fresh重开核对。具体身份、25项源身份/保存值检查及仍缺失的native逐请求网络回执见 [保存重开记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/plugin-browser-linked-r48/save-reopen.json)，原26项总结果false不改。

以下是向Parent/原公共owner提出的行为和布局需求，**不是已发布的props、slots或variant名**；本族没有改Shared、canonical公共资产或native painter。

| 原公共组件 | 实际观察及UE依据 | 最小同源合同与归属 |
| --- | --- | --- |
| TreeRow | 四分类copy仍显示expander/cube/eye/lock；当前原始行宽240，而原board分类区域宽448。Plugin Browser的分类选择不具备对象可见性/锁定操作，叶分类不应出现无效disclosure。 | 原owner在同一TreeRow语义子树中提供适用的导航表现及容器尺寸策略；是否可展开、是否有图标/尾部操作由真实节点能力决定。保留category稳定key、selection/focus/keyboard事件、真实count和locale/generation；不要隐藏有业务意义的操作来冒充实现，也不要新增SettingsTreeRow。 |
| Button | Add与Plugin Directories实例根均固定240×32。实际UE `SPluginBrowser.cpp` 两个SHorizontalBox按钮slot为AutoWidth，Add使用Icons.Plus及SmallButtonText。 | 同一Button须支持按实际字体和图标测量内容宽度及正确padding，并保留原label/icon/action/disabled/focus合同。原owner协调Penpot/component/catalog/codec/native尺寸行为；本域只提供原New Plugin只读入口及固定根目录动作意图，不自行发明icon主件或启动generator。 |
| SearchInput | 实例根仍固定240×32，不能填充header剩余空间。UE对应SHorizontalBox slot采用默认增长、padding(0,7,0,7)，SSearchBox将OnTextChanged接真实过滤；源码没有显式FillWidth调用。 | 沿原SearchInput和外层flow提供增长/收缩、最小宽度与真实query/clear/keyboard/focus事件合同。标签覆盖不是搜索绑定；窄尺寸下的优先级、滚动和长中文行为仍需原owner与本域设计共同验证。 |
| PropertyEditorRow / accepted value editors | 当前Shared四主件不含value_slot接受的编辑器，正文metadata卡片也尚未使用正式公共属性表面。 | 继续SP-C1同源语义子树、明确page-scope/raw/source/presence及description/reset/换行高度合同，等原owner真实发布accepted editors及source/slot/codec/native适配。不得以SearchInput替代value、把属性展平成label或增painter role关闭。 |

Parent followup-04已在9份canonical源中承载同次typed metadata：status→ViewData→Payload→既有TOML package_metadata表；本族已核对after hash，不重复应用r47提案。该链没有实现正式卡片的可变行高、112高旧native行的替换、引擎绑定或binary provenance；这些仍需原公共source owner与Parent的正常窗口producer接续。

真实图 `browser/reopened.png` 与 `browser/overrides-reopened.png` 已逐图复核：白底正文、中性文字、分类多余图标及固定控件宽度仍明显偏离UE。约71%画布缩放的headless图片不证明实际DPI、产品焦点或1逻辑像素精度。公共样式A/B、原始ZUI正式导出与重载、真实EXE及双端差异验收仍open，passed/accepted均false。完整差异在 [逐图复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/plugin-browser-linked-r48/visual-review.json)。

## r50 业务卡片的精确source消费边界

原本域PluginMetadataCard component `81548358-cc2f-47cf-856d-39b395bfecfa`、main `d57fd7f1-bf28-4c97-bc6b-0a52ea4696f1`继续保留。本轮API复核为rev35，整Plugin Browser页不变。组件domain metadata写明`sourceWrapper=null`、`officialCodecMetadata=false`，component/main均无canonical-record；它是可编辑本域组合，尚无正式source wrapper。原owner view的`plugin_list_slot` / `ModulePluginListPanel` / `module_plugin_list_slot`没有对应卡片重复原型；不能把view根当canonical卡片导入、重签来源或另造同名公共组件。

Parent已有optional metadata transport不需要第二DTO/store/revision。原owner需交同源卡片原型/owner-instance路径、稳定plugin_id重复身份、字段与事件/slot映射以及测高/scroll合同；本域按 [r50精确节点映射](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/settings-plugins/card-source-eligibility-r50/eligibility.json) 消费display_name与package_metadata.version/category/description/maturity。None移除metadata表现且保留原operation-only恢复动作，Some空字符串保持真实空值，不推导已加载DLL版本。原main设计116高与native112固定行不构成匹配画面的误差结论；真实长文/中文测量和可变行高仍需原source/native owner。没有新增公共props/UUID、painter role、主题或生成操作，也未声称正式导出已尝试/拒绝。
