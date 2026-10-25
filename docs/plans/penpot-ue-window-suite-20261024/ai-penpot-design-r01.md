# AI 的实际 Penpot 设计与逐图记录

Owner `01a16d53-0693-72e2-bd58-d34a65b938fa`，2026-10-24。接续 [源码盘点](ai-windows.md)、[设计合同](ai-design-and-acceptance.md) 和 [Parent 交接](ai-parent-handoff.md)。这里记录实际持久化结果；窗口族仍未完成，所有产品 LayoutReviewCase 的 passed/accepted 都是 0。


最新恢复记录：2026-10-25 原 file/page 已保存到 rev47/863对象，监督恢复后的实际修改、失败与关闭重开见 [监督恢复52](#监督恢复52原640板空状态与真实保存重开)。下述 rev14/19/28 等保留原时点，不作为当前空库、未打开插件或当前验收结论。

## 已保存身份及验证范围

真实入口：[06 AI Editors](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60dbec5a&page-id=b4a1e3ce-9e6a-81fe-8008-d69d60dbec5b)。file `b4a1e3ce-9e6a-81fe-8008-d69d60dbec5a`，page `b4a1e3ce-9e6a-81fe-8008-d69d60dbec5b`，页面名 `AI — source layouts and states`。十三业务案例的 manifest version=4；最新实读 revn 28/vern 0，另有公共实例受控验证板和原型菜单板。revn 20～28 只更新本域创建菜单及默认图的候选原型交互，旧 UUID、业务组件及来源保留，详见下文。该文件由 Parent 分配，没有另建平行文件。

- API 使用现有真实团队认证，未创建账户、打印凭据或复制认证状态。复用已安装 Playwright 及现存 Edge，以隔离 profile/cache 在 `E:/cargo-targets` 运行，没有共享桌面输入、DPI 修改或服务生命周期操作。
- 第一次原生 update-file HTTP 200 持久化 revn 1；后续使用官方 `mod-obj` 的 `assign` 形状解码保存 revn 2，硬校验保持启用。其间两次原始 `set` 的 JSON 嵌套类型校验 HTTP 500，失败请求/响应均保留，复读仍为 revn 1，没有伪造成功或回滚其他修改。
- revn 14 保存 783 个对象（含页面根），782 个可编辑对象；13 块尺寸/状态板，4 个 AI 业务原生组件，58 个业务链接实例，357 个 native flex 容器，401 个 text 对象。主实例/副本通过真实 componentId、componentFile、shapeRef 关联；文本使用原生 text-content-text/attribute 覆盖。前版 revn 2 为 711 对象/12 板/3 组件，回执原样保留。下述 revn 19 新增 34 对象后为 817 对象，没有把受控验证板计成第十四个窗口案例。
- 原上下文打开、读取实际文件/manifest、截图后关闭；全新认证上下文再开同一 file/page/board，revn 2、3、5 及最终 14 均复读一致，无 pageerror。最终复读另核对 783 个 ID、父子关系、组件引用、metadata、原型导航及 82 个业务文本覆盖；不是同一未保存页面的 reload。真实 Roboto/Noto Sans SC 字体请求 HTTP 200。
- revn 2 修正 137 个 flex 原生子项顺序并保存 29 处 Penpot navigation；最初 128 处组件覆盖缺少文本子类，原生同步后确实失效，后文保留失败及修复。实际 Viewer 已点击 BT→BB→类型菜单设计板→BT；这仅是设计导航，不是引擎 handler、真实弹层或焦点返回。
- 按 Parent 已记录的 P05 决定，revn 3 追加 72 个本域对象及 Decorator 组件/板。当时原 710 个 editable ID 全保留，709 个原对象逐字段相同，仅组件展示容器和页面根增加 child；该历史范围由 `decorator-extension-preservation.json` 证明。后续原生字体/流式重排改变了几何属性，不能再以该历史回执声称最终十二板逐字段未变。
- 最终 401 个文本均为 Penpot 原生 DOM 字体等待/测量后的持久化行框；91 个菜单标签使用 auto-width/原生 flex，207 个容器用原生 padding/align 表达标签内边距与垂直居中。六块 BT 右列以 UE 60/40 为依据定义固定上区 basis 与余量 fill，并保留 1 px 间隔；BB 两列原生等分，非截图坐标补丁。组件副本保留真实 text-content-text/attribute 覆盖，未 detach。

回执均在 `E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-native-r01/`：原 author/refinement/decorator 请求及回执保持原样；最终身份为 `design-current.json`、`ai-file-readback-revn14.json`、`verified-api-readback.json`、`revn14-{original,fresh}-browser-readback.json`。原生测量保存的独立 update-file 响应监听未抓到 ACK，故 `saves=[]` 如实保留；实际新 revision 的 get-file、401 项数据及关闭/fresh 重开证明已持久化，不能捏造保存回执。原 `initial-ai-file.json` 后续曾写为 revn 1，不能用文件名声称完整初始 revn 0。

## 原生渲染和组件同步失败的核查

1. revn 3 的 Viewer 真实跳转成功，但全部文本未绘制。实际源码 `ui/shapes/text.cljs` 的 Viewer SVG 分支依赖 `position-data`；此前 render.html 的 foreignObject 图仍有文字，不能把两路径当作同一视觉结果。失败导航图保留于 `prototype-revn3-step*.png`；通过 `wasm=true` 原生工作区初始化持久化 401 行框得到 revn 4，文字恢复，未手写 baseline。
2. revn 5 的中文 SVG 被压缩；“目标实体”原生 WASM 行框仅 19.46997 px。只读加载既有 TTF/WOFF1 字节到隔离浏览器 FontFace，两者量得相同的 53.31995 px，排除字体文件字宽差异；Roboto Window 两者均 47.65079 px，亦不同于 WASM 的约 45.16 px。`native-font-asset-metrics-comparison.json` 记录六份真实响应/hash/测量。问题边界为当前运行 Penpot WASM 的字体测量/就绪路径，其内部根因未修复，交 Parent/原 owner；没有改服务、字体库或 Penpot 源码。
3. 直接以 `wasm=false` 重开不会重测已有行框，45 秒条件等待未获得新保存，原失败 `native-dom-position-data-readback.json` 保留。依据 `container.cljc` 的派生属性移除合同，仅清除五个中文 position-data，再由原生 DOM `calc-position-data` 等待字体并保存 revn 7，实际四字宽 53.328125 px。随后其余 396 文本也重测；91 个菜单标签采用 auto-width，避免 46 px 固定框截断真实 47.65 px 字宽。
4. revn 9 的实际重排揭示本次作者元数据缺陷：仅有 content-group 不足以保持文本副本，65 处业务内容被主实例覆盖。`logic/libraries.cljc#text-change-value` 要求 text-content-text/attribute 子类。恢复同一原 revn 5 持久化内容，按与各 master 的真实差异登记子类，保留全体 ID/组件引用；复测 revn 11 及最终 14 的 82 个文本覆盖全部匹配原业务语义。revn 9 原图仍是失败证据，不可作为目标设计。
5. 菜单自然宽度触发原生 flex 重排后，原先未表达的标签 padding/居中及 BT 右列分配暴露出来。revn 13 已保存约束，但 API 修改不会主动调用前端 layout engine，实际 Blackboard 子区仍停在旧位置，与 Details 重叠；原图保留。隔离浏览器在本文件原生图层树逐板选中 13 个 board，经官方宽度输入框执行原宽→原宽+1→原宽，触发正常前端重排并保存 revn 14。不是改写截图、手填子节点坐标或共享桌面操作。`native-ui-layout-reflow-{intent,readback}.json` 保留步骤与结果；13 板原 x/y/width/height 全部恢复，六块 BT 分区均为 1 px 间隔且无重叠，278 个居中文本子项实际中心误差小于 0.1 px。最终关闭/全新上下文重开、13 图捕获和三次 Viewer 导航均保留相同 revision 与 82 处业务文本。原生字体/组件合同检查是本地作者验证，不能代替插件、Jenkins Rust 或产品验收。

## 业务组件、依赖及源资产映射

| 业务内容组合 | native component UUID / main instance UUID | 版本及依赖/事件 |
| --- | --- | --- |
| AI Behavior Composite Data | `7f3c68cb-7424-4694-9fc8-0cf60ff65cde` / `b97a2d23-dfb8-4f34-8876-c9c0a1305879` | 本文件 draft v1；显示真实 node ID/name/有序 children 的设计投影。图宿主、pin/命中、事务仍由公共 graph 合同接入 |
| AI Behavior Task Data | `031d4a34-2b4e-4a84-8d85-990c4c7b62df` / `228e4fa9-7624-42d4-becf-5a1949cd0a0e` | 本文件 draft v1；显示 implementation/typed parameters。选择/详情/打开资产/参数提交/Undo 事件仅为合同，产品未实现 |
| AI Blackboard Key Data | `8d700050-79be-45eb-9821-6c4776299a98` / `256238fe-58c4-4ce6-a7ef-c3634b2a3e54` | 本文件 draft v1；key 身份为精确字符串，不伪造键 UUID；创建/引用改名/删除/保存依 AI-BB01..07 |
| AI Behavior Decorator Data | `e850f27b-c1b1-4a57-b169-96eec373d46f` / `0dd3b72c-8642-46f0-9e00-72fb49a58d15` | 本文件 draft v1；明确保留 target_visible→chase_move 的单子 wrapper 与各原 ID。复用既有 Composite/Task master，不复制图底座；实体参数绑定仍未就绪 |

它们是本域业务内容组合，没有发布成公共 library，也不实现另一套通用 GraphCanvas、Tree、Field 或主题。公共 graph 由材质 `01a16d50-6c7f-7370-853d-4581184b66a1` 牵头；动画 `01a16d51-741b-73b3-a61a-2189cd38f81e` 拥有 transport；通用组件/主题/壳由 Parent 集成。当前普通菜单/工具栏/属性框是带 catalogTarget 或 pending 标记的可编辑设计投影，尚未换成已发布公共实例；不能将 58 个业务实例计为公共 catalog 复用完成。

公共 file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` 在原 revn 14 设计时曾为 root-only；该观察已过期。最新实读 revn 1/isShared=true/41 对象，已有下述四个原生主件，colors/typographies 仍为 0。色值仍投影现有 `editor_tokens.zui` 及 UE `BehaviorTreeColors.h` 的正确 LinearColor→sRGB，没有新增 palette/token 系统或共享 style。Roboto normal 400/700 的 font ID `495ff285-7c71-49bc-bc4e-df20017e2d66`、Noto Sans SC normal 400 的 `2ddd793a-5f89-40b0-aec1-fbe879e37a57` 复用现有团队注册。公共 fill/text style 引用及 BT 图标/序号/边框优先级仍待接入，不能宣称≤1 DIP 对齐。

## revn 19 的公共原生实例受控验证

已按 Parent followup03 核对真实 source/slot 合同，通过正式 `link-file-to-library` 连接上述公共文件。只在本域增加验证板 `5f166ebc-d847-4685-b682-b99489c9bc7c`，640×360，原生 vertical flex、padding 16/gap 12。revn 15 保存空板，随后四次真实 Penpot Assets 拖放分别保存 revn 16～19。未手工克隆组件树、detach、创建公共主件或覆盖有效 source metadata。

| 公共组件 / version 1 | component UUID / source main UUID | 本域 copy UUID / shapeRef |
| --- | --- | --- |
| WorkbenchButton | `6ddc248b-bd35-80c5-8008-d6c8c01096f3` / `6ddc248b-bd35-80c5-8008-d6c8bd9b057f` | `973488f0-586e-807e-8008-d6cdeb160dee` / source main |
| WorkbenchSearchInput | `6ddc248b-bd35-80c5-8008-d6c8c3ce892e` / `6ddc248b-bd35-80c5-8008-d6c8c0f9c196` | `973488f0-586e-807e-8008-d6cdef89f540` / source main |
| WorkbenchTreeRow | `6ddc248b-bd35-80c5-8008-d6c8c7a79944` / `6ddc248b-bd35-80c5-8008-d6c8c452bf0c` | `973488f0-586e-807e-8008-d6cdf485d489` / source main |
| WorkbenchPropertyEditorRow | `6ddc248b-bd35-80c5-8008-d6c8cab1e7d0` / `6ddc248b-bd35-80c5-8008-d6c8c84d848f` | `973488f0-586e-807e-8008-d6cdf95392b3` / source main |

四个副本共 33 个形状，componentFile 指向公共 file、shapeRef 指向各真实 source main，组件 modifiedAt 均为 `2026-10-24T13:21:31.125Z`。canonical-record、sourcePath/sourceNodeId/instancePath 等公共 metadata 原样保留，内容 overrides=[]。公共文件数据仍为原 revn 1；原 AI 的 782 个非根对象、四个业务主件、manifest、业务副本逐字段不变。原上下文关闭后，新进程/全新认证 context 实际重开 Penpot UI，再核对全部引用和 linked library，得到相同 revn 19/817 对象。

逐图打开 [原上下文图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-shared-r01/native-linked-instances-original-context.png) 与 [fresh 重开图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-shared-r01/native-linked-instances-fresh-context.png)：四副本按原生插入顺序 Property/Tree/Search/Button 流式排列，无重叠。Property.value 是 required/single，公共默认槽仍为空，未绑定真实 AI 参数；Axis 仍是单 InputField，不塞三轴或任意 Container。该图含 Penpot chrome、显示设计缩放，deviceScaleFactor=1；不是 Windows 150% DPI 或 Engine 原生图。

首次对外层 viewport 的普通 dragTo 没有保存实例，失败与当时 revision 原样保留。核对实际 `viewport/actions.cljs` 后，改为资产真实 dragstart 与 `svg.viewport-controls` 的原生 dragenter/drop，触发正常 instantiate-component/undo/save 路径。认证仅从 Parent 指定现存文件读取，前后 hash 相同，未打印、复制、覆盖其 secret；没有共享桌面输入或服务操作。

完整证据在 `E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-shared-r01/`：`source-contract-inspection.json`、正式 link/add ACK、`native-instance-steps.json`、`saved-closed-fresh-reopen-verification.json`、`visual-review.json`。这只证明四种 linked native copies 的受控消费；未替换原十三板全部通用控件，未证明官方 codec/ZUI 导出、全局样式传播、产品加载或交互。该阶段4213拒绝连接是历史状态；本轮接续15已实测当前19002插件manifest/bundle可用并核对字节，正式codec消费仍由公共owner交接，19001 API可用不等于导出通过。

| 原始 owner / 稳定身份 | Penpot 映射及当前限度 |
| --- | --- |
| `zircon_plugins/ai/editor/behavior_tree.zui`，asset/template/view `ai.behavior_tree` | 13 个 board 的原源资产候选；保持原文件和身份，未生成替代 ZUI 或写回产品。BT/BB 是同文件两模式的设计目标，独立 Blackboard Toolkit 尚缺后端 |
| `AiBehaviorTreeRoot` | board/content shell 的 sourceControlId metadata；实际 DocumentHost/Toolkit/Dock 事件由 Parent 合同接入 |
| `AiBehaviorTreeGraphCanvas` | graph content 和 topology 路径的 sourceControlId metadata；引用真实 fixture 的 parent/child/ordinal，不具产品连接编辑或执行能力 |
| `AiBehaviorTreeInspector` / `AiBehaviorTreeBlackboard` | Details/keys 的 sourceControlId metadata；保留未来 binding/事件的落点，当前没有虚构 binding namespace |
| `AiBehaviorTreePalette` | 原 ID 仍在源文件；设计目标是 UE 上下文创建，不再作为默认永久左栏；迁移绑定与焦点的可逆导出未执行 |
| `zircon_plugins/ai/editor/perception_debug.zui`，`AiPerceptionDebugRoot` / `AiPerceptionDebugAgents` | 本轮尚未设计或导出；不能从 BT/BB 设计完成度推算感知窗口完成 |

完整 native object→sourceControlId/component/fixture metadata 可从持久化文件的 `dev.zircon.ai.design` 读取，JSON API 返回命名空间为 `devZirconAiDesign`。该命名空间明确是设计身份，不伪造 `dev.zircon.zui` 的可逆 bridge 元数据。官方 import/export owner 需在同一原资产上建立受支持的组件/ID 映射并保存导出回执，才能进入产品应用。

BT 数据是现有 `zircon_plugins/ai/runtime/src/behavior_tree/nodes/tests/integration_ai_effect.rs#effect_then_running_tree` 的设计夹具投影：`root` Sequence 有序连接 `set_score`、`emit_guard_event`、`wait`，参数为 score Integer42、ai.guard_passed、duration_seconds Scalar10；图中没有 Running、成功高亮或模拟 tick。它尚未作为真实资产打开/保存/执行。Blackboard 的 6 行英文及 4 行中文是 schema 设计样例，不能视为引擎加载结果。默认 BT 如实显示未绑定 Blackboard、无选中 debug object/session。

新增 Decorator 板来自 `zircon_plugins/ai/runtime/src/tests/scenarios/patrol_detect_chase/fixtures.rs#patrol_detect_chase_tree(target)` 及同级 `patrol_detect_chase.rs` 的目标常量：root Selector 按顺序连接 target_visible、patrol_move，target_visible 的单子节点是 chase_move；target_visible 采用真实默认 implementation=blackboard_condition 的感知条件分支，sense=sight，两个 MoveTo 的 target 分别为 (0,0,-8)、(4,0,0)。函数的 target 实体参数未绑定，metadata 明确为 source-function template、不能序列化/编译冒充实际 DTO；界面显示 Runtime entity required。没有给这些 Task 配置 result 参数，也没有使用测试宿主返回状态作为真实引擎执行证明。

## 十三张原尺寸图的逐图查看

`POST /api/export` 实际返回 404 空响应，保留 `official-export-0f83b39b-a455-4d08-8c04-7517f31e3472-response.json`，官方 Shape.export 服务成功图数为 0；该初次请求不能证明完整官方导出服务是否可用。没有为此启动/重启 exporter 或 Penpot。后续依据现有官方 `exporter/src/app/renderer/bitmap.cljs` 的 `render.html?route=objects` 及 `#screenshot-<id>` 路径，读取真实已保存文件、等待全部声明字体加载、以 deviceScaleFactor=1 截取原尺寸元素。最终 13 张 revn 14 PNG 的 IHDR 和 DOM bounds 都与板尺寸一致，字体响应有效、pageerror 为空，结束复读 revn 14 未变。

这些是**实际 Penpot 渲染路径的浏览器捕获**，不是 `/api/export` 成功回执、可逆 ZUI 导出、Windows 实际 96/144 DPI 证明或正常 Engine 原生捕获。每个 PNG 都有同名 JSON 的 board/revision/bounds/font/方法/hash，最终汇总为 `penpot-render-revn14-summary.json`，逐图复核为 `visual-review-revn14.json`。revn 2/3 的 foreignObject、revn 3 Viewer 缺文字、revn 5 字宽、revn 9 副本覆盖及 revn 13 重叠的历史图全部保留，不能复用为最终版本通过证据。下表最终十三张均由本会话逐张打开；三张最终 `prototype-revn14-step*.png` 也已查看，实际跳转只覆盖设计导航。所有双端差异、产品文本行框和原生交互结论仍 pending。

| 板 / 实际 UUID | 原尺寸渲染图 | 本图所见及未完成项 |
| --- | --- | --- |
| BT default 1280×800 / `0f83b39b-a455-4d08-8c04-7517f31e3472` | [图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-native-r01/penpot-render-revn14-0f83b39b-a455-4d08-8c04-7517f31e3472.png) | 70/30 与右 60/40、四节点/三条真实夹具拓扑可见。UE Root 辅助节点、图标/序号、曲线/选中框、grid、运行反馈仍缺，不计图编辑完成 |
| BB selected 1280×800 / `661d8bba-4bb9-408e-9ff9-5ca4341d8fc3` | [图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-native-r01/penpot-render-revn14-661d8bba-4bb9-408e-9ff9-5ca4341d8fc3.png) | 两列与六行、score 选择和三字段一致；缺真实树行/类型图标、checkbox/dropdown 实例及 schema 事务，字段文本不是可用控件 |
| BT selected 1280×800 / `d1c4ba25-8f69-4526-8499-bdfb80f18e0e` | [图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-native-r01/penpot-render-revn14-d1c4ba25-8f69-4526-8499-bdfb80f18e0e.png) | Set Blackboard 边框和右侧 Node Name/Key/Value 对应；选中框仅现 token 投影，UE 边框/原生 hit/撤销待验证 |
| BB focused 1280×800 / `8678f615-530c-4fa3-b718-06fd11e0deaa` | [图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-native-r01/penpot-render-revn14-8678f615-530c-4fa3-b718-06fd11e0deaa.png) | Entry Name 边框有设计焦点，文字可见；尚未文本输入、Tab/IME/失焦提交、Esc 取消或焦点返回 |
| BT popup 1280×800 / `e82481b7-f06b-4ba2-a712-305142723055` | [图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-native-r01/penpot-render-revn14-e82481b7-f06b-4ba2-a712-305142723055.png) | 菜单覆盖图并可读；仅部分目录项的设计样例，18 项目录、搜索/创建/锚定/焦点返回未接，不以静态菜单算操作通过 |
| BB popup 1280×800 / `b287d79c-34b5-492c-9424-e78a857778d8` | [图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-native-r01/penpot-render-revn14-b287d79c-34b5-492c-9424-e78a857778d8.png) | 仅列六个实际 runtime 类型，不虚构 UE Object/Enum；焦点、键盘选择、创建 key/详情与撤销待实现 |
| BB empty 1280×800 / `0a916870-b06d-402e-9ec7-1874eab038ae` | [图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-native-r01/penpot-render-revn14-0a916870-b06d-402e-9ec7-1874eab038ae.png) | 空列表与空详情没有假值；实际从空 schema 创建第一键及 dirty/保存未验证 |
| BB 中文长键 1280×800 / `72bd2ea2-947e-4124-9568-0bb2c201c49f` | [图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-native-r01/penpot-render-revn14-72bd2ea2-947e-4124-9568-0bb2c201c49f.png) | Noto Sans SC 无方框，四行样例可读；未覆盖超过列宽的截断/tooltip/搜索与 IME，也未验证精确 UTF-8 键保存 |
| BT default 900×620 / `95436d80-3a07-4a52-bda7-df890c13d4d8` | [图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-native-r01/penpot-render-revn14-95436d80-3a07-4a52-bda7-df890c13d4d8.png) | 保持 70/30；左侧 task 部分裁切，graph pan/zoom 到完整节点和端口的实际可达性尚不能证明；不得按截图逐点缩字移位关闭 |
| BB selected 900×620 / `82777aac-d8d9-4a5e-ac89-f4647506531d` | [图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-native-r01/penpot-render-revn14-82777aac-d8d9-4a5e-ac89-f4647506531d.png) | 两列及字段可读，无控件重叠；min-size、滚动首尾与真实焦点/类型菜单仍 pending |
| BT default 640×520 / `2d753d90-cf25-4b32-b443-276a6f06e8b9` | [图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-native-r01/penpot-render-revn14-2d753d90-cf25-4b32-b443-276a6f06e8b9.png) | 左 task 标签被裁掉，右 task 部分裁切；debug selector 当前省略而非可达 overflow，与同操作可达目标有差异。须公共 graph 与 toolbar overflow 合同，保持未完成 |
| BB selected 640×520 / `99cc55d7-ac24-4a5b-96bb-b288f8119fd5` | [图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-native-r01/penpot-render-revn14-99cc55d7-ac24-4a5b-96bb-b288f8119fd5.png) | 两列/三字段和六键仍可读；共享 toolbar 的 debug selector 可达性同样未完成，键类型/详情输入的真实行为与更多行滚动待验证 |
| BT Decorator selected 1280×800 / `540b7174-a916-4425-81c4-d20306f051e9` | [图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-native-r01/penpot-render-revn14-540b7174-a916-4425-81c4-d20306f051e9.png) | 四节点/三条拓扑全部可见，Decorator 蓝色类别来自实际 UE LinearColor，单子 wrapper 身份与 Selector 顺序保留；与 UE 附着式外观差异按 P05 保留。实体未绑定、图编辑/执行/保存仍 pending，没有假运行或 result 参数捷径 |

## 继续开放的交付门

1. 公共组件/共享样式、图节点/端口/连线路由与命中、Dock/Tab/Menu/toolbar overflow 合同接入；空白图十一项创建菜单已形成设计，Decorator/Service 的上下文与真实语义继续开放，不绘制假运行。
2. 由官方插件/导出 owner 在上述原始 `.zui v2` 建立可逆映射，实际 export delta 后保留 control ID、组件引用、bindings/events。当前没有适用 bridge 元数据，也没有产品写回。
3. 实现真实文档/资产、schema 引用事务/保存/Undo，编译注册及 session/world/entity 对应执行反馈；P05 已由 Parent 决定首阶段保留真实 Decorator wrapper，但设计/接入仍未完成；EQS/StateTree 路线和 Service 后端继续 open。
4. 插件检查与聚焦 Rust 仍未运行。原十一项输入的 source drift/auth pending 和历史材质策略拒绝保留。恢复后最新 9c684aef… 已实际支持 AI 四 filters 的 client/server 配方，原 bt01/validate 与 wt01/create QueryOnly 仍 session pending，见 ai-recovery-07；不重放旧源输入。当前 20+4 静态定义未运行，十二源仍匹配 bt-graph-adapter-01 的格式/guard；共享锁实际 diff/after/receipt 仍空。Penpot 新菜单切片另记于下文，十三业务案例保持原身份，没有冒充 OVW-42 专用设计、正常 EXE 或产品通过。
5. 正常 EXE/资源闭包指纹尚无，未获得桌面时段。四个尺寸/DPI、所有交互/滚动首尾/焦点返回、一次真实样式→导出→产品新图、原生捕获入口、并排/差异图和≤1 DIP 逐图验收全部 pending。不得把这十三张设计图算作正常 Engine 预览。

本轮没有源码副本、编译或自有常驻服务；实际释放 0 B，保留临时 recipe/browser cache 和失败/设计证据，清理仍 open。真实 Penpot 文件由 Parent 原服务继续持有。

## revn 28 的真实目录创建菜单

同一 file/page 最终保存并由新进程/新认证 context 实际重开为 revn 28/855 对象、四个 AI 业务 mains、58 个业务 copies 加六个公共 copies、430 个 text。十三业务案例的 manifest 和身份保持；本轮在原 1280 弹层 `e82481b7-f06b-4ba2-a712-305142723055` 内更新菜单 `0aa29b62-45fd-4a94-9481-2ce6765c55f9`，另加辅助原型菜单板 `425e56e2-02fc-444f-9642-21fb66b7baad`，256×408。辅助板不算新的窗口验收案例。

创建项取自真实 `nodes/{composite,task}.rs`：Selector、Sequence、Parallel、Random Selector，以及 Wait、Move To、Play Animation、Set Blackboard、Emit Event、Run Subtree、Script Task。完整十八项源目录 hash 与类别/semantics/creation kind 同时留在 `penpot-node-menus-r01/intent.json`；本菜单只显示 UE 空白图创建上下文适用的十一项。Run Subtree 的创建 kind 是 Subtree，不能由 Task 类别推导。六个 Decorator 及一个 Service 没被塞进空白图菜单，也没有伪造 Service 周期反馈。UE 的 Auto Arrange/Add Comment 保留可见禁用外观；它们缺真实 AI 命令/评论保存，不挂无效果的成功 handler。

两个 SearchInput 都由现存公共库的真实 Assets dragstart→viewport drop→instantiate/save 产生，组件 `6ddc248b-bd35-80c5-8008-d6c8c3ce892e`、main/shapeRef `6ddc248b-bd35-80c5-8008-d6c8c0f9c196`，copy 分别是 `c0666b74-40d0-8045-8008-d6e6e5344305` 与 `248874fc-6d04-8070-8008-d6e720db825c`。14 个副本形状的 canonical source metadata 逐字段相同，根 touched=[]，公共文件 revn 1/data 未变，未 detach 或新增公共 main。普通菜单标签仍待 Parent 的正式 MenuRow 实例；共享样式、主题传播和 codec 不因此通过。

菜单采用原生 column flex、padding 8、SearchInput 32 高、现有 24 高菜单标签，不逐图偏移。旧搜索占位文本的 UUID 保留，已被原生搜索实例取代；原 817 个 UUID 全部保留，十个预定原对象以外逐字段不变，全部旧 component refs、既有交互和来源 metadata 保持。原生工作区完成文本行框持久化，再关闭原 context，由独立新进程重开同一文件验证。API 修改 ACK 与 native 保存后的 revision 读回分开记录；没有把未捕获的 native HTTP response 补写成 ACK。

已直接打开并逐图复核 [1280×800 完整板](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-node-menus-r01/penpot-revn28-e82481b7-f06b-4ba2-a712-305142723055.png) 与 [256×408 菜单板](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-node-menus-r01/penpot-revn28-425e56e2-02fc-444f-9642-21fb66b7baad.png)。十一项及两个禁用外观动作完整可见；Search/组名/条目按同一 flow 排列。三十个菜单行框的本端容纳检查无文字溢出。这是 Penpot render.html 的真实原尺寸对象渲染，非官方 shape export、Engine 窗口或双端 ≤1 DIP 结论；图中原拓扑仍是有标识的设计 fixture。

此前原型查看器验证失败：首次未出现旧 CSS 所指目标 DOM；改用当时查询与 wasm=false 后同样未出现目标 DOM，页面随后失去响应。已核对 PID/parent/创建时间/唯一 profile，仅结束自有隔离 Edge 及两个遗留子进程；未触碰桌面或共享服务。素材查询超时、重复 DOM ID 定位失败、首次宽度恢复输入显示 257 的部分结果、两次查看器失败均原样保留；最终 fresh 读回菜单宽度为 256。后续相同 revision 的实际交互接续如下，不覆盖这些失败。

本轮证据目录 `E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-node-menus-r01/` 保存实际 API、native steps、fresh reopen、渲染、逐图复核及失败。15:45Z 实测 19001/readyz HTTP200、4213/manifest.json Win10061，没有重启服务。四个原始 ZUI 的初始 hash 均未变化，十二候选源仍匹配 `bt-graph-adapter-01`，24 项 Rust 未运行。900/640 裁切、中文/长文/滚动、键盘和焦点返回、正式插件、正常 EXE/150% DPI 与双端比较保持 open，passed/accepted=false。

## revn 28 原生查看器的鼠标与键盘实测 10

原 API `get-view-only-bundle` 实际返回完整 855 对象、公共库 revn 1 和四份已登记字体，不是空文件或认证失败。自然 100%/wasm=false 的 Blackboard 原图已渲染；先前 `.first()` 查询命中了隐藏同 ID 副本，将选择收敛到可见实例后，原 BT 默认图的 native open-overlay 已完成打开、外部关闭、原触发点再次打开。上述只读交互前后 file/data 均未改，revision 仍为 28；没有改 Penpot 源码或覆盖组件。

| 实际操作 | 本端结果 | 产品仍需完成 |
| --- | --- | --- |
| 左键点原图空白处 | 打开原辅助 overlay；十一目录项、两分组及两个禁用外观动作可见 | UE 左键选择/平移与右键上下文的区分 |
| 点击弹层外部再点原触发处 | 菜单关闭、仍为原 BT 图，再次打开；打开与重开 PNG hash 相同 | 正常宿主的取消/焦点返回与事务未执行证明 |
| 实际右键点空白处 | 也能打开；当前原型左右键均触发 | UE 上下文对象、pin、选区和 enabled 状态规则 |
| 点击共享 SearchInput 并输入 Wait | 可编辑元素数 0，文本保持 Search，焦点 BODY | 真正搜索输入、过滤、中文输入及无匹配状态 |
| Esc | 弹层仍可见 | 取消弹层并恢复先前图控件焦点 |
| 方向上键 | 由 BT 默认图 14/14 切到 Blackboard 13/14，菜单消失 | 在菜单项间移动且不触发画板导航 |

鼠标三张 [打开](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-node-menus-r01/prototype-open.png)、[关闭](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-node-menus-r01/prototype-dismissed.png)、[重开](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-node-menus-r01/prototype-reopened.png) 及搜索/Esc/方向键三张均已逐张打开复核。它们含 Penpot chrome、查看器 100% 和设计 fixture，1600×1000 为浏览器捕获尺寸；原业务 frame 为1280×800。没有 Windows 实际 DPI 或 Engine 捕获含义。回执为 `prototype-result.json`、`prototype-keyboard-1792858212349.json` 及 `prototype-visual-review.json`，所有步骤焦点均为 BODY，productCommandExecuted=false。

诊断途中 CDP Debugger.enable 超时且 detach 未完成，保留原进程/终止回执和退出 1；约 1.67 GB renderer 观察不是产品内存预算或已确定的根因。仅核实归属后结束该诊断的自有 Edge/renderer，后续自然缩放与交互进程正常退出。可见实例修正不能解释全部历史无响应。公共 codec、共享菜单键盘/焦点实现和 Engine 接线仍由原 owner 接续，不修改公共代码绕过；本轮没有导出或产品测试，passed/accepted=false。

## 当前发行物与窄窗实际读回15

本轮沿原P21 storage路径只读复用认证，16:58:43Z实际请求19001/16060 readyz及19002/manifest.json均HTTP200；AI原file/page仍rev28/855对象/4业务主件，Shared仍rev1/shared/4主件，没有改文件或库。认证文件前后SHA相同，未打印、复制、覆盖其secret。现存4213错误继续保留，不能作为当前19002不可用的依据。

17:01:42Z实际下载19002/assets/plugin.js计算摘要（未保存源码副本），485232B/SHA `442bed70a38d28c86a702002658c8d77598ac6205fe325b1696aedbd27b7b47d`与原owner的本地served-dist完全匹配。原built runner `26d2e4fc…`及worker-r3 compile receipt `06746e78…`实际存在/摘要匹配，但没有两者在该dist下兼容运行的回执。未重启服务/重建worker/安装账号，未调用正式import/export，不将HTTP200计作codec通过。

新API几何读回：1280/900/640三原board的graph canvas宽分别895/629/447；三个190宽task在原数据中的union宽650，900左右各裁10.5，640左右各裁101.5。它们是Penpot图形几何，不是正常Engine实际DPI或文字行框对齐证据。原640 toolbar只有Save/Browse/Validate/Behavior Tree/Blackboard五项，没有debug selector及可证明的overflow；宽版有额外debug文本。保留所有UUID、节点、children顺序、toolbar操作及失败原图，未做坐标修图、删除或缩小fixture。

原业务板根只有devZirconAiDesign身份；公共SearchInput副本则保留devZirconZui的真实sourcePath/sourceNodeId/baseline/shapeRef。不能通过给设计图补source元数据或重签guard冒充导出。已向原public owner成功入队 `01a16edd-dd47-7b30-9717-8961f552fbf1`，交原private behavior_tree.zui（v2/12稳定control IDs/无events）及原file/page，请其提供当前已构建工具的正式无损source-import/save-close-reopen/export消费方法，限定追加独立验证板、replaceSelection=false，不动现有业务mains/copies/公共源；尚未确认读取或取得执行合同。

向graph owner成功入队 `01a16edd-dd29-7631-a9d2-f743541d48db`，给出上述真实尺寸及frame-all/source-focus消费请求。现有GraphCanvasState只有pan/zoom/selection等helper，本域不另建相机；须共享owner/Parent交同代际retained bounds/viewport/transform及输入路径。没有真实pan/hit/focus前不据设计缩放图关闭裁切或操作缺口。

证据位于 [penpot-ai-resume-r01/findings.json](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-ai-resume-r01/findings.json) 及同目录actual API/bundle/owner消息记录。十二AI源/八依赖仍匹配b00543e0/28定义，四原始ZUI未改；无Cargo、resolver、新测试/生成提交、桌面/DPI或正常EXE捕获，official exports=[]，passed/accepted=false。仅创建必要API/hash证据，无新browser/helper/build，保留历史失败与原helper/cache开放项，释放0B。

## 已注册插件的官方 Open 实测16

17:15:37～17:15:47Z，在隔离无头 Edge 中实际重开原 AI file/page，从官方插件管理器点击现有 `cce74285-44c0-8011-8008-d6ea3345bdbe` 的“打开”。真实 iframe 地址为 `http://127.0.0.1:19002/#/?theme=dark`，Import .zui 控件已挂载，状态显示 Ready / No ZUI asset selected；Export selected 与 Export Canvas PNG 均为禁用。未选择或上传文件、导入、导出、安装、Allow 或编辑；这不是 codec、原 ZUI 写回或产品验收。

前后 API 均为 rev28/855 对象，file data、objects、components 及83项插件 registry 的 SHA256 分别保持一致，认证原文件 SHA 未变；没有保存认证副本。实际截图已打开复核：[官方插件窗口](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-ai-resume-r01/plugin-inspection-r02.png)。画面含 Penpot chrome、17%工作区缩放及未完成设计，不是 Engine/DPI/1DIP 证据。首次尝试因英文 Open 定位与实际中文“打开”不符而超时，原失败及其截图保持；第二次依据实际文字定位成功，没有改账号语言。

通用源码 runner 当前 SHA `d11a9e80…`，265行无条件 server.listen(0)、350～351行 Install/Allow、811行空页前提、815行认证另存、830行创建 host，故不原样用于已有业务页。只读 Open 已验证现有服务可消费，但有界原资产导入/保存重开/导出仍待原公共 owner 的兼容合同；不创建平行生产 runner 或补签业务板源元数据。检查后 context/browser 正常 close，17:16:46Z记录 node PID及带本域browser路径的根进程均未找到；这是本次浏览器退出检查，不是产品 native lifecycle。

完整[只读实测](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-ai-resume-r01/plugin-inspection-r02.json)和[runner边界核查](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/penpot-ai-resume-r01/runner-compatibility-audit.json)保留。业务板、公共库、四原 ZUI、Rust、联合锁生成与正常 EXE 均未推进为通过；passed/accepted/captureReady=false。保留本次必要截图、失败及browser cache，未执行清理，实际释放0B。

## Blackboard 类型菜单：共享搜索与保存重开21

在原 AI file `b4a1e3ce-9e6a-81fe-8008-d69d60dbec5a` / page `b4a1e3ce-9e6a-81fe-8008-d69d60dbec5b` 的既有受控状态板 `b287d79c-34b5-492c-9424-e78a857778d8` 中补齐类型搜索设计。原 `New Key / six runtime types` frame `eb1f55df-c18c-40e4-ac22-70b1fe21d43c`、六行 UUID 和 200×240 尺寸均保留，不另建菜单或公共主件。UE 依据是 `SBehaviorTreeBlackboardEditor.cpp:409–448` 的 class filter/200×240 picker，以及 `SClassViewer.cpp:1701` 的真实 `SSearchBox`、changed/committed 回调；UE 创建键的 transaction、Modify、选择及请求改名不是本轮设计已实现的功能。布局映射仍采用 [Forms](https://pro.reactbits.dev/docs/app-ui/forms/forms-1)，辅以 [App Dialog](https://pro.reactbits.dev/docs/app-ui/app-dialog/app-dialog-1) 和 [Command Menu](https://pro.reactbits.dev/docs/app-ui/command-menu)，视觉与行为目标以本地 UE 源为准。

从实际 Shared rev2 原生 Assets 拖入 component `6ddc248b-bd35-80c5-8008-d6c8c3ce892e`，新 copy 为 `03859a83-eba9-8011-8008-d70ef551bade`，source main/shapeRef 仍为 `6ddc248b-bd35-80c5-8008-d6c8c0f9c196`。该新实例七个 source refs 和七份来源 metadata 均逐项匹配真实 Shared 对象；当时回执只统计六个样式字段，接续26撤回据此声称“已继承六处样式”的结论，原始计数/回执保留。没有 detach、复制 main、修改公共文件或重签 source/export guard。当前 Shared 是 rev2、四主件、34 colors/3 typographies，rev1/styles=0 只保留作历史。当前文件的库关系 API 已直接返回 Shared rev2；关系路由省略的样式数据不能当作空库，样式库存取实际 get-file。

本实例有明确局部覆盖：root 横向 fill，背景和文本的水平约束由 left 改为 leftright。Shared 默认宽 240 且这两子形状固定 left，不能直接适配菜单 184 的内部宽度；公共投影/伸展和官方 codec 兼容性提案 `AI-C01-search-resize-21` 交 Parent/public owner。布局由 Penpot 原生高度编辑 240→241→240 触发，最终菜单恢复 200×240，padding 8，搜索框 184×32、文本约148宽，六行仍各24高。没有手填逐图位置。新实例的 touched 及保留的来源摘要都记录在[回执21](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-type-menu-21.json)。

实际保存关闭后，在新 Edge 进程中重开同 file/page/board，前后 rev34/vern0、862对象、data/objects/components 摘要一致。四业务 mains 保持，65 copies=58业务+7 Shared，194 shapeRefs；原855 UUID全保留，旧对象只有菜单容器和六行位置发生变化，错误落点引起的键列表重排已经原生恢复。六类型仍是 Bool→bool、Integer→integer/i64、Scalar→scalar/Real、String→string、Vector→vec3、Entity→entity/u64；Entity没有冒充UE Object/Class，也未启用 Name/Rotator/Enum/Struct 或继承 schema 的创建。i64/u64 的生产编辑仍须保真字符串和范围验证。

已分别打开复核 [1280×800 原生板渲染](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-type-menu-21-board.png)、[200×240 菜单](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-type-menu-21-menu.png) 和[新进程完整编辑器](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-type-menu-21-fresh-context.png)：搜索及六行完整可见，底层键标签由菜单正常覆盖。前两图来自实际 Penpot render.html，后一图包含 Penpot chrome 和64%画布缩放；三者都不是官方导出或 Engine 捕获。UE filter/view controls、键类型图标/正式 MenuRow、真实可编辑搜索、方向键/Enter/Esc/焦点返回、创建与保存、其它尺寸/中文/滚动、实际150%DPI及双端≤1DIP仍open。

失败原样保留：素材/图层定位超时、首次 native drag 落到下层 key scroll 的[中间原图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-type-menu-21-after.png)、JSON set 枚举导致500、JSON UUID字符串的 move 未生效及其200/readback不匹配、旧 SVG class 定位超时。之后使用实际类型解码、Transit UUID 和真实 SVG id，未 bypass schema/skip validation。原始 payload、响应和 native 步骤见[变更记录](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-type-menu-21-mutation.json)；文件名 after.png 表示失败中间态，最终态见 fresh-context/board/menu。

本轮17候选源仍匹配候选18，复用其 rustfmt/源码 guard；34 authoring+4 gameplay 共38项仍仅静态定义、0执行。四原 ZUI/锁未写，没有 Cargo、生成、原请求重投、服务恢复、共享桌面/DPI或正常 EXE。浏览器正常关闭且本轮专属 cache 路径未发现残留进程；必要失败/图片及可复用缓存保留，清理仍open、释放0B。插件正式导入/导出、生产host/事务/save、真实执行及全族 passed/accepted/captureReady 均保持 false。

## 共享 Button 的样式覆盖、原生重置与重开23

在原 file/page 的既有受控板 `5f166ebc-d847-4685-b682-b99489c9bc7c` 上验证局部样式覆盖，不修改业务板或公共 main。该板明确标注 `Property value unbound / not product`，640×360 仅是组件试接板，不代表640×520窗口案例。原 Button copy `973488f0-586e-807e-8008-d6cdeb160dee` 继续引用 Shared component `6ddc248b-bd35-80c5-8008-d6c8c01096f3` 和 source main/shapeRef `6ddc248b-bd35-80c5-8008-d6c8bd9b057f`。

本轮实际 rev34→35→36：先把该 copy 的 fill 局部引用到 Shared 已发布的 `editor.accent`（`b3f68044-6a40-566a-9d34-fdb415e552e8`，#0070e0），再从 Penpot 原生组件菜单点击“重置覆盖项”。只有原 Button root 产生临时 fill override；重置后 touched=[]，root JSON 摘要与覆盖前相同。已逐张打开复核[覆盖前](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-style-consumer-23-before.png)、[蓝色覆盖](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-style-consumer-23-override.png)和[重置后](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-style-consumer-23-reset.png)，前后两张 PNG 字节相同。三图来自真实 Penpot render.html 的原生对象渲染，没有绘图修补；不属于官方导出或 Engine 截图。

关闭原浏览器后，新 Edge 进程/context 重开同 file/page/board，仍为 rev36/vern0/862对象，data、objects、四业务 components 摘要前后一致。原862 ID、65 copies（58业务+7 Shared）、194 shapeRefs和全部来源 metadata 保留；只发生既有 Button 文本的原生 positionData 测量刷新。[新进程完整编辑器](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-style-consumer-23-fresh-context.png)已实际打开复核：灰色 Button、未绑定 Property value 和通用搜索/树均可见，含 Penpot chrome 与120%画布缩放，不是实际150%DPI。

确认的共享缺口是源主件没有默认样式引用：本轮观察 Shared rev2/41对象/4main/34 colors/3 typographies，其 Button root 的 fills/strokes 和原文本 `6ddc248b-bd35-80c5-8008-d6c8bdb3e2f8` 均没有 native style RefId。重置正确恢复了字面颜色/字体，不能据此宣称默认共享样式继承。提案 `AI-STYLE-23-Button-source-links` 交 Parent/public owner：按[原 canonical Button](../../../zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/button.zui) 的 surface.1、border、text.primary 及 strong-family/weight/body-size 语义评审样式投影，保持现有 UUID、字体度量、bindings/events 和 source/export guard。不得凭相同色值猜 token、在本域重建公共 main 或重签来源。局部 override/reset 已得到证据；跨文件及双窗口族的共享样式修改→正式导出→产品新图仍未完成。

插件当前 HTTP 字节另列版本边界：19002/assets/plugin.js 为348732B / `6ded04494d4126e9540b6b225ff0c5ecd61b9772408d8ac2d2e80f1d6c3701f3`，不能沿用实测16的485232B / `442bed70…` Open/worker兼容结论。本轮原生管理器有83个“打开”，后端 registry.ids/data 也有83项；目标UUID仍登记在原19002 manifest，registry摘要不变。两次有界检查分别停于标签歧义和无法由当前DOM证明UUID对应行，没有点击其它条目、安装、Allow或导入/导出。两张管理器原图及启动前脚本模块格式错误均保留；第一次把registry外层两个字段计为2的诊断计数也保留并明确修正，不把它算插件数量。当前新包的官方Open与codec资格待原owner提供；现有原file/page源导入导出请求不重提。

完整[接续23回执](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-style-consumer-23.json)区分实际覆盖/重置、默认继承缺口、插件诊断和产品门。17源与38项静态定义保持、0 Rust执行；四原ZUI和锁未写。正式codec、生产资产/事务/save/执行、所有窗口尺寸和状态、正常EXE/150%DPI/双端≤1DIP仍open，passed/accepted/captureReady=false。自有浏览器已退出，必要失败与可复用缓存保留，清理open，实际释放0B。

## 原生共享颜色消费与同步证据边界26

独立API确认Shared仍为rev2/shared、四main、34 colors/3 typographies，库已真实发布；不再以rev0/无样式为阻塞。Parent `settings-plugins-followup-05.json` 的37个样式UUID/值与实际库一致。不过其 `styleReferencesInMasterShapes` 的21项 `refId/refFile` 全为null，当前41个Shared对象也没有非空颜色/typography RefId；21是空字段出现次数，不是21个有效引用。原Button默认字面样式缺口仍open，延续 `AI-STYLE-23-Button-source-links`，不在本域重做公共master或主题。

本域接续21也曾将 `copyStyleRefFields=6` 写成“已继承样式”，此结论同样撤回：字段数量不能证明RefId非空、指向存在样式及实际消费。旧回执及其SHA不改，现有source refs/metadata证据仍有效；真实颜色引用以本次原生操作及读回为准。

在原类型菜单SearchInput copy `03859a83-eba9-8011-8008-d70ef551bade` 的原text `03859a83-eba9-8011-8008-d70ef551bae4` 上，从Penpot原生素材面板点选实际 `editor.text.primary`（Shared `e022fcac-39df-51eb-8515-4723f9932060`）。原file/page保存至rev37；只有该text的content/positionData/touched变化，持久化了同一颜色的三处有效引用。原root、另外两个子形状、184宽fill及leftright覆盖、全部862 UUID和四业务main不变；custom Roboto字体属性原样保留。已打开复核[前图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-published-consumer-26-before.png)与[实际链接后图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-published-consumer-26-linked.png)：仍是200×240菜单和原六类型，引用前后同为#c0c0c0，不能据相同外观宣称全局传播。

原生管理库界面实际显示4组件/34颜色/3排版；新进程最初出现“共享库有更新”。确认只有assigned Shared这一份外部库后，点击唯一原生“更新”，提示消失；关闭并再用独立Edge进程重开，仍rev37，data/objects/components不变，提示未再显示，局部颜色及全部几何覆盖仍在。已逐张打开[更新前](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-published-consumer-26-sync-before.png)、[更新后](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-published-consumer-26-sync-after.png)及[再次重开](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-published-consumer-26-sync-fresh.png)。它们含Penpot chrome/64%画布缩放，不是Engine或实际DPI。API库关系的syncedAt仍为13:39:25Z、modifiedAt为17:37:45Z，本轮未捕获同步HTTP ACK；因此只验证点击、提示状态和覆盖保留，服务器同步持久化/默认继承及跨窗口A/B继续open，不把无数据delta当写回。

字体边界另留：实际主件/本copy使用 `custom-495ff285-7c71-49bc-bc4e-df20017e2d66`，三份已发布typography则声明 `gfont-roboto`。family相同不能证明同资源或行框相同；本轮没有应用该typography、上传或替换字体，更没有CJK/引擎字体ready结论。待public owner核对canonical字体投影及真实source master样式链接，不给本域copy补造默认继承。

Parent catalog原候选4c2f849c…与本次实读9556e66d…不同（当前解析71组件）；仅核对两项目标契约：NumberField确有10 props，原value_text/placeholder及新增8项保持；PropertyEditorRow四slots，原required/single value及accepts保持。该观察不以旧25/25回执资格化当前全部catalog，也不归责新增差异。42/0..100/precision0仍是模板默认，未绑到真实AI值；i64/u64须精确保真文本与域校验，描述/source/reset slots不制造默认值、资产关联或有效命令。没有新NumberField Penpot master或原ZUI写回。

首次颜色完整文字定位因后缀#c0c0c0超时、库页“更新”与背景banner重名，以及banner精确文字定位失败均原样保留；修正依据实际截图和唯一button role，不改服务/源码。完整[回执26](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-published-consumer-26.json)分开记录发布、有效引用、局部覆盖、原生更新与未取得的服务器证据。所有自有浏览器正常关闭，认证原文件SHA不变；未占桌面/改DPI/运行Cargo或重投原请求。正式导出、产品加载、真实AI数据/save/执行及逐图≤1DIP继续pending，passed/accepted/captureReady=false，缓存与失败保留，释放0B。

## 原业务主件消费共享字样式31

在原 AI file/page 上用原生 Assets 操作链接已发布字样式，保存 rev37→38→39。原 Composite 业务 component `7f3c68cb-7424-4694-9fc8-0cf60ff65cde`、main `b97a2d23-dfb8-4f34-8876-c9c0a1305879` 保持；标题 text `6ea54bd1-7edb-4c12-9158-d8a6763c81e6` 应用 Shared rev2 的 strong `9ac86733-8214-5a04-820a-9074b5c68310`，副标题 `204d11b3-0021-466f-8401-32de92de06ca` 应用 caption `c0224e78-f918-526f-8026-a76cd8f347af`。两项 typographyRefFile 都是真实 Shared file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12`。没有复制公共 main、detach、修改公共样式或补签 canonical source。

真实变更仅涉及两个 main text 和其十二个本域 copy text；其余848对象摘要、原862 ID、四业务 component 身份、七个 Shared copy 的身份和引用保持。copy 的 `Root`、`selector / 2 ordered children` 等业务覆盖仍实际存在；main 的文字、白色填充、174×20行框、位置和父子关系不变。原生样式操作将这两种文本字体从 custom495ff285…换为发布的 gfont-roboto；不能把同名 Roboto 当相同资源。实际 main 文本测量宽度分别58.609375→59.21875、78.8125→78.046875逻辑单位，x/y/height不变。这只是两条英文设计文本的前后测量，不是全图≤1DIP或引擎字体验收。

关闭操作浏览器后，另一个 Edge 进程/context 重开原 file/page，API仍 rev39/862对象且完整 data 摘要与保存后相同，实际 Shared data 未变。新进程加载了 Roboto 400/700，并在[原生重开图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-typography-consumer-31-fresh.png)的右侧显示 strong 引用。先于截图取样的 DOM 计数曾返回样式名0/更新提示false；原数值保留为该时刻观测，截图实际可见样式名和“共享库有更新”，不能据早期计数宣称同步完成。本轮未再点库更新；server syncedAt/默认 source 样式继承继续未验收。

已逐张打开[修改前](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-typography-consumer-31-before.png)、[修改后](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-typography-consumer-31-after.png)及重开原图。修改后图右栏仍处于 caption 的即时选择，重开图为 strong；不把不同选择瞬间当像素差分。均含 Penpot chrome/61%画布缩放，不是 Engine、真实150%DPI或官方 codec 导出。首次大小写严格匹配共享库按钮超时及最初 JSON/Transit 读取不匹配均保留诊断，没有写坏设计或换认证。

完整[回执31](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-typography-consumer-31.json)保留原生动作、精确引用、前后测量和新进程重开；[检查31](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/shared-typography-consumer-31-checks.json)分开记录源码保护与未执行范围。17源/38静态定义、四原ZUI、原validate/create和联合锁身份保持；0 Rust执行。正式导出/产品加载/真实AI数据与文档save/执行、中文/窄窗/弹层交互、正常EXE/150%DPI/双端逐图仍pending，passed/accepted/captureReady=false。自有浏览器关闭，认证未复制/覆盖，必要失败与可复用cache保留、释放0B。

## Task、Decorator、Blackboard 主件字样式消费33

原 AI file `b4a1e3ce-9e6a-81fe-8008-d69d60dbec5a` / page `b4a1e3ce-9e6a-81fe-8008-d69d60dbec5b` 经五次原生 Assets 操作保存 rev39→44；[原始动作与变更](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/business-typography-consumer-33.json)保留每次点击、保存读回及原形状片段。本次 Shared 实读 rev4，四原主件、34色、3字样式仍存在；本次前后 Shared data 完全相同。此前 rev2、rev39 证据留在原时点，不用它们替代本次版本。

| 原业务 main | 原 text ID | 实际链接 Shared 字样式 |
| --- | --- | --- |
| Task `228e4fa9-7624-42d4-becf-5a1949cd0a0e` | `836c9a22-bcf9-412d-97d9-d185cda6a992` | strong `9ac86733-8214-5a04-820a-9074b5c68310` |
| 同一 Task main | `b4b9da9f-601d-4108-ab30-210480a6780c` | caption `c0224e78-f918-526f-8026-a76cd8f347af` |
| Decorator `0dd3b72c-8642-46f0-9e00-72fb49a58d15` | `95dcb77d-2fd3-4b32-ab0c-957ac20a38a3` | strong，同上 UUID |
| 同一 Decorator main | `973d3b41-76b1-44be-841d-9b111792eac5` | caption，同上 UUID |
| Blackboard Key `256238fe-58c4-4ce6-a7ef-c3634b2a3e54` | `c4623537-4f40-45c1-8e28-e5a9d48ee092` | body `99eb86f9-12ff-5935-96ad-3d1a985e61f0` |

五个 main text 及36个 Task/Decorator copy text 变化，另外821对象摘要保持；全部862 ID、文字内容及布局几何保持。连同31，四个本域业务 main 的七个文字节点均已有真实 Shared typographyRefFile/refId；48个 copy text 也已链接。34个 Blackboard copy text 保留 `text-content-attribute` 等原本地样式覆盖，仍未链接 body，其中30条为原 Roboto、4条为原中文字体资源。没有重置实例、抹去中文字体或假称全实例继承通过；这些覆盖的兼容迁移仍 open。

即使字号、字重与行框位置保持，custom Roboto→gfont-roboto 的实际字宽仍改变。Decorator 说明 main/copy 各142.171875→139.90625，Chase Target 副行79.046875→76.828125，三处宽度差绝对值超过1逻辑单位（最大2.265625）。这是 Penpot 前后测量，不能据此认定与 UE/Engine 的误差达标，也不能用移动文字或改截图隐藏差异。中文实例未更换字体，浏览器可见中文字不证明 CJK 引擎资源或导出 ready。

关闭修改进程后独立 Edge 重开，rev44/862对象与保存 data 摘要 `71c3da5e0334a25f744990fb2880e131ade2e388982bfa3b172d95d2dc53dd09` 完全一致；4业务 main、58本域copy、7 Shared copy、194 shapeRef 均有真实源。首次右栏精确文字定位15秒超时保留；仅重试只读重开，没有重放五次编辑。[重开原图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/business-typography-consumer-33-fresh.png)可见 body 引用和“共享库有更新”，故不计跨库同步完成。首次把所有 copy 纳入“七个主件”断言所得 false 保留，后续明确分别核对7个main与82个copy text；没有以改字段名称掩去34个未链接实例。

已用真实 Penpot render 页面以scale=1捕获完整画板并逐张打开；原SVG边界分别1280×800、900×620、640×520，无移动节点或坐标修图。以下只核查设计图；fixture文案、连线与属性格均不充当真实数据、运行或操作证据。

| 原始画面 | 逐图复核与仍缺内容 |
| --- | --- |
| [BT默认1280](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/business-typography-consumer-33-bt-default-1280.png) | 三个Task可见，原业务文字保留；端口边框、UE图标/序号、真实文档/Blackboard绑定与调试宿主未验收。 |
| [BT默认900](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/business-typography-consumer-33-bt-default-900.png) | 左Task标题起始及右Task边界仍裁切；未消费真实 retained bounds 的 fit，不用挪节点关闭。 |
| [BT默认640](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/business-typography-consumer-33-bt-default-640.png) | 左Task文字完全裁掉，右Task裁切；debug selector缺失且无overflow入口，保持失败。 |
| [Decorator1280](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/business-typography-consumer-33-bt-decorator-1280.png) | 原真实单child wrapper目标拓扑保留；UE附件外观、selection边框、真实pin/命中/运行ACK及abort/subtree证据仍缺。 |
| [BB选中1280](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/business-typography-consumer-33-bb-selected-1280.png) | 六键、选中框及右侧三属性可见；34个key copy样式覆盖未迁移，Search未证明可编辑，属性非真实Schema。 |
| [BB选中900](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/business-typography-consumer-33-bb-selected-900.png) | 键名/属性文本可见；未证明键盘、滚动、焦点或产品尺寸响应。 |
| [BB选中640](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/business-typography-consumer-33-bb-selected-640.png) | 键名和属性格可见；debug selector仍缺失且无overflow，窄窗产品交互未验收。 |
| [BB中文长键1280](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/business-typography-consumer-33-bb-zh-long-1280.png) | 四条中文键及长键完整可见，仍使用原中文资源；没有真实中文输入、搜索、引用跳转、导出或引擎行框证据。 |

沿用本文既有 app-shell/data/forms/command-menu 的 ReactBits 映射和 UE 源码依据；本轮不改变布局模型。官方 owner 的 `public-component-closure-01/ai-family-recipe-r01.json` 已有四原ZUI闭包候选，但 `recipeReadyForExecution/sourceOwnerExportVerified/normalCaseCaptureReady=false`，`ai.behavior_tree.open` 的 typed payload/handler 和普通窗口 producer 未交付。该 recipe 不是本域正式导出或EXE通过。本轮未写公共主题/catalog/graph/插件、原四ZUI、Cargo锁或原validate/create输入；17源/38定义仍0 Rust执行。完整[检查33](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/business-typography-consumer-33-checks.json)保留断言失败、文本测量、引用闭包、逐图记录与边界；官方导出→产品加载→变化证明、真实数据/save/执行、正常EXE/实际150%DPI/双端≤1DIP仍pending，passed/accepted/captureReady=false。自有浏览器已关闭，认证原文件未改，必要证据与复用cache保留，释放0B、清理open。

## 英文 Blackboard 实例链接 body 字样式34

原 AI file/page 在 rev44 上继续原生编辑；预先核对30条英文 key copy 均保留 Roboto/custom495ff285…、13.333333/400，字号、字重、family、style、lineHeight及letterSpacing与已发布 body 一致，文字和源textRef也与33记录相同。按真实父链展开 Layers，以Control逐项选中；原生 `aria-checked=true` 的30个ID须与计划完全一致才应用一次 Shared body `99eb86f9-12ff-5935-96ad-3d1a985e61f0`。没有用shift范围选择、detach、reset整个实例、API更新或改源main。原生多选行为依据只读 `dev/penpot/frontend/src/app/main/ui/workspace/sidebar/layer_item.cljs` 的mod分支，而非修改DOM。

真实保存 rev44→45，data SHA `bb1d001589401944a0b5793a688b781c219604cd3a25e5e55674985d3b234cc2`；仅这30个text对象改变，其余832对象哈希相同，全部文字、填充、几何、父子/组件/shapeRef及4个业务component记录保持。Shared本轮实读rev5，body样式内容摘要与33时相同，本轮前后 Shared data 一致；不覆盖此前rev4或声称由本域发布rev5。四条原Noto Sans SC/custom2ddd793a…中文copy逐字节未变，继续作为明确的字体覆盖，不能用gfont-roboto替掉以凑齐引用数。

连同31/33，当前七个业务main文字和78个copy文字有真实共享typography引用，剩余4个中文copy仍用原字体。30条英文copy是直接链接共享body并保留业务文字/填充的局部覆盖，不等同于已证明公共source默认继承、reset或全局A/B传播。本域不新增中文基础master或平行主题；沿原公共字体/codec交接，由public owner给出body与Noto Sans SC覆盖的兼容投影，实际CJK引擎资源和测量仍open。

关闭修改浏览器后另一个进程/context重开原文件，API仍rev45且完整data一致；58本域copy、7 Shared copy、194 shapeRef均有真实源。原生重开图右栏显示body。提前DOM采样曾给updateBannerVisible=false，但[原始重开图34](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-typography-consumer-34-fresh.png)实际仍可见“共享库有更新”，该false只保留为采样时刻值，不推定服务端同步完成。

保留30条文本的实际前后positionData：15处英文key宽度变化绝对值超过1逻辑单位，最大1.765625；x/y/height不变。字体资源与全双端行框误差是不同测量，不据此声称UE/Engine≤1DIP，也不调文字坐标掩去。已逐张打开[多选前图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-typography-consumer-34-selection.png)、[应用后图](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-typography-consumer-34-after.png)、重开图和以下完整scale1画板；native图含Penpot chrome/59%画布缩放，不是实际DPI证据。

| 完整原画板 | 本轮逐图观察及未通过范围 |
| --- | --- |
| [BB选中1280](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-typography-consumer-34-bb-selected-1280.png) | 六个英文键与右侧属性可见，文字内容保持；真实Schema/读写/引用索引/资产跳转与保存未接入。 |
| [BB选中900](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-typography-consumer-34-bb-selected-900.png) | key/属性可见；只是原布局渲染，不是产品resize/键盘/滚动验证。 |
| [BB选中640](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-typography-consumer-34-bb-selected-640.png) | key/属性可见，debug selector仍缺失且无overflow；未用删操作关闭。 |
| [BB焦点1280](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-typography-consumer-34-bb-focused-1280.png) | Entry Name有原蓝框；该框是设计态，不证明真实输入、IME、Tab或焦点返回。 |
| [BB弹层1280](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-typography-consumer-34-bb-popup-1280.png) | 原六种key类型及Search文字可见；Search编辑、上下键、Esc、选择/取消及焦点返回仍未实现/验证。 |
| [BB中文长键1280](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-typography-consumer-34-bb-zh-long-1280.png) | 原四中文键及长键可见，保持原Noto Sans SC；本轮未改其字样式、文字或实例数据。 |

完整[原生动作34](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-typography-consumer-34.json)与[检查34](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/blackboard-typography-consumer-34-checks.json)保留精确选择、源身份、文本量度及逐图记录。本轮沿原ReactBits和UE映射，不另改布局；原行为树900/640裁切未修，正式导出→产品加载、真实AI/事务/save/执行、正常EXE/150%DPI/双端比较仍pending。17源码与四原ZUI摘要保持，38候选定义仍0发现/编译/执行；没有新validation身份、query、生成、锁写入或桌面。passed/accepted/captureReady=false，自有浏览器退出、认证原件未改；证据与可复用cache保留，实际释放0B、清理open。

## 监督恢复52：原640板空状态与真实保存重开

本轮沿监督恢复授权保持 goal active，未等待通用Field/Toolkit或完整公共交付。真实 file `b4a1e3ce-9e6a-81fe-8008-d69d60dbec5a` / page `b4a1e3ce-9e6a-81fe-8008-d69d60dbec5b`，修改前独立 API rev45/862对象。900板原有该提示，首轮检测到后未保存重复对象；640板原五命令右边剩余190逻辑像素，补一个原生流式Text，最小宽155/fill、32高，内容沿原1280板的未绑定调试对象状态。

新对象 `3ae54be5-f0f4-4daf-839a-3f3a276da8a7` 仍在原 toolbar `66beae8a-9689-4ace-815c-7d14091f3986`、原640板 `2d753d90-cf25-4b32-b443-276a6f06e8b9`。实际引用 Shared body `99eb86f9-12ff-5935-96ad-3d1a985e61f0` 和 `editor.text.secondary` `7c88169d-5f57-51e1-98d0-a76d6d08c0eb`，Shared读取rev5，写入0。没有新建公共main、detach、增加运行状态或修改源ZUI。API update-file HTTP200保存rev46，原862 IDs全部保留，除原toolbar孩子列表外所有既有对象逐字段相同，四业务component记录原样。

随后真实隔离Penpot工作区选择原board，通过原生Width 640→641→640触发布局，收到原生update-file HTTP200，保存rev47；原宽恢复640，提示实际行框宽150.8125，在190宽内完整显示。关闭原认证context，新context重新打开同file/page/board，rev47及完整data摘要一致。原认证只读且SHA不变，浏览器均关闭；无共享桌面输入、系统DPI或服务操作。

逐图复核：

| 原图 | 复核 | 结论 |
| --- | --- | --- |
| [修改前](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/supervision-52-before.png) | 实际Penpot工作区112%缩放，640板五命令存在，右侧空白，无调试空状态提示；两侧任务裁切 | 原失败保留 |
| [原生重排后](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/supervision-52-after.png) | 提示完整进入同一工具栏行，原节点/连线未挪动；截图含编辑选择标记和Penpot chrome | 只证明设计修改 |
| [关闭重开](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/supervision-52-reopen.png) | 新context显示同提示，原main/组件/图拓扑仍在；数据摘要一致 | 保存重开通过本域范围 |

保留失败：两次render.html使用不存在的`screenshot`定位器超时、一次render路由未绘制正文；之后改用真实workspace入口捕获，没有伪造官方PNG导出。一次900提示已存在检查使全批操作在保存前退出，再限定640保存。修复前原文输出/所有失败记在 [实际回执52](E:/cargo-targets/zircon-local/reports/ue-ai-window-suite-20261024/supervision-resume-52.json)。这些工作区图不是Engine、正式owner export或实际150% DPI，不能用于≤1DIP双端比较。

未完成：UE实际debug combo及列表、overflow/长文本、菜单输入与焦点返回、图fit/pan/端口宿主、两个任务裁切、生产文档事务/save/运行trace、官方ZUI单gap导出和正常EXE。保留原四ZUI/12control IDs/0events；不把补回一个状态Text当功能选择器或整个窗口完成。passed/accepted/captureReady继续false。


## 菜单53领域消费边界

原节点创建菜单与Decorator设计现在有本域typed资格/搜索adapter的源码消费者，见[53领域合同](ai-design-and-acceptance.md#菜单作者语义53真实选择目录搜索与单候选)。11个Composite/Task和6个Decorator来自真实catalog，不由UI标题猜类型；无选择/失效节点/只读/多选与Service/断点/步进缺口有typed禁用原因。此为源码变化，没有新的Penpot保存或官方export，不改变52的rev47/863原回执，不把Search/ListRow数据投影算真实可编辑输入或菜单键盘/焦点通过。旧fixture设计、四业务mains/原实例和640/900裁切、中文/长文/滚动及真实DPI差异继续保留。


## 黑板54的数据消费

原六类型New Key设计现在有typed schema创建候选和原reference plan改名菜单的数据适配。它保留真实type alias/required/key次序，不发明默认值/producer，引用状态区分Unknown与仅输入scope的count；Type/Delete及运行值编辑缺口明确。见[54领域合同](ai-design-and-acceptance.md#黑板菜单54创建候选语义改名与引用未知边界)。本轮没有新的Penpot读写或截图，52的原rev47/863仍是固定回执，13业务板/原4mains/实例与status ID保护交原public owner；不将源码借用行计作原生Search输入、菜单IME/焦点、正式export或Engine画面。
