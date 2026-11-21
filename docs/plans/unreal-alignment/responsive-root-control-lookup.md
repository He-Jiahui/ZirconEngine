# 响应式工作区测试把控件 ID 当成根节点路径

ZE 基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。这是**现有验收测试自身的独立维护项**，不是生产布局引擎不支持缩放。首提交只提供调查与回归方案；如果修正查找后还出现其他错误，按其实际发生的位置处理，不删断言制造通过。

## 1. 用户与工程影响

维护者运行 [真实作者文档的响应式工作区合同](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/tests/zui_reactbits_agent_workspace_contract.rs#L12-L35)，本应检查同一表面在窄、中、宽窗口下的侧栏、主区与上下文摘要。[原用例先从真实文档构建表面和唯一根](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/tests/zui_reactbits_agent_workspace_contract.rs#L181-L211)，此时编译、展开、节点数、槽数和根的方向/间距已有自己的断言。问题发生在接下来的**测试查找**，不是截屏或用户冷启动失败，也不能说明线上控件不可见。

## 2. 两份已发生的证据

Windows x64 / MSVC、`zircon_runtime` 关闭默认、显式 `native-host,ui`、`test` profile。在 [#232](https://github.com/He-Jiahui/ZirconEngine/pull/232) 的 tests-only `25569db598005184535f0c7b075607056bee7d76` 和生产修订 `a781b6e90a4f18f0bfe836fd0006b5e37e26da6a` 上，未修改的 `reactbits_agent_workspace_builds_a_responsive_flow_surface` 各为 **0 过 / 1 败**。两个实际进程都在 `zircon_runtime/tests/zui_reactbits_agent_workspace_contract.rs:25` 报 `responsive shell control should be retained`，均发生在 [五次按 ID 查找以及第一次 640×520 布局](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/tests/zui_reactbits_agent_workspace_contract.rs#L213-L225) 之前或当中，未进入任何 `compute_layout`。这不等于三个尺寸都出现了布局错误。原测试与夹具在两端都与原 `main` 是相同 Git blob：`40613d3a6c3276d690eb63ef2e366c7f4214f459`、`84318220764ad451939b822784c340b6b443afdc`。

## 3. 哪一层把两种身份混在一起

- [原夹具的根声明](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/tests/fixtures/ui/reactbits_agent_workspace.zui#L22-L35)：节点源 ID 是 `root`，作者声明的交互 ID 是 `ReactBitsAgentWorkspaceRoot`；另四个控件 `Sidebar / Main / ContextPanel / ContextSummary` 各自也只有一份作者声明。
- [生产树用 stable_node_path 建根](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/v2/surface_tree/node.rs#L25-L39)，而 [children 走 stable_child_path](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/v2/surface_tree/node.rs#L55-L66)。[两个路径构造器](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/v2/surface_tree/node.rs#L231-L241)分别用源 ID 与非空的子控件 ID。因此此根路径按实现为 `v2/root`，不是 `v2/ReactBitsAgentWorkspaceRoot`；别因为几个子项的路径碰巧等于 `v2/<control_id>` 就把它当普适 ID 合同。
- [所有节点的作者 ID 独立进入 template_metadata](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/v2/surface_tree/node.rs#L132-L151)。[公开的单义控件查找](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/surface.rs#L214-L218)通过 [只接受一个且仍匹配的节点](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/control_index.rs#L59-L76) 给出实际 `UiNodeId`。但 [测试 helper](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/tests/zui_reactbits_agent_workspace_contract.rs#L15-L26)扫描物理树里的 `node_path` 后取第一个。原测试第一次调用传入的就是上述根控件名。这是确定性的测试前置矛盾，而不是随机性能或渲染问题。

## 4. UE 的对应做法和原测

`AutomationDriver::By::Id`（`Engine/Source/Developer/AutomationDriver/Private/LocateBy.cpp:31–55`） 与路径定位是不同的查询表达方式，控件元数据身份不等于层级文本。UE 已有 `TestFramework.Driver` 原测试分别验证 `FindElement / By::Id(KeyA)`（`Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/AutomationDriver.spec.cpp:89–98`）、`FindElement / By::Path`（`Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/AutomationDriver.spec.cpp:119–128`）；`FindElements / 重复 Id 返回两项`（`Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/AutomationDriver.spec.cpp:186–194`）是复数集合的判定，不能偷换成 ZE 的唯一控件索引应该选第一项。它们不是 ZE 这个 ReactBits 文档的三种窗口验收；没有找到完全同形的 UE 原测，也没有执行 UE。使用本地 6.0.0 / UE5 / CL0 的源码文件及其指纹，没有可靠的上游提交号。

## 5. 最窄 Rust 改法与防误修边界

若真实观测证实根的 authored ID 被正确保留，修正本测试的定位器，使用公共的唯一 ID 查找；缺失或重名应失败而不能 `.find()` 取首个。原五个 lookup 的名字、控制组件、夹具、布局尺寸、属性/可见性、发布子节点路径及后续组件断言均保留。原 [重复 ID 不会解析成任意控件的对照](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/ui/surface/tests/control_index.rs#L77-L104) 说明这是已有语义。不改生产树、样式、UI ABI、虚拟列表或 [#232 的滚动范围](https://github.com/He-Jiahui/ZirconEngine/pull/232)。未发现同因历史问题；按全部状态 233 个 PR 标题及正文去重，对相关已关闭/已合变更和全部开放路径再核查。这个 `.zui` 是仓库内的作者测试样本，不是已经观察到的游戏运行故障。

## 6. 等价回归与验收顺序

1. 在本 PR 固定的未改生产树上，加载真实 `.zui`、编译、建立原表面，用另一个小用例对照根 `source_id`、真实 `node_path`、`template_metadata.control_id`、该 ID 的候选数和公开 lookup。四个子控件、缺失 ID 和已知重复场景分别说明；不能通过复制表面或造假节点表示原文档。
2. 同一份测试产物实际运行原用例，区分“前置查找没走通”和“某尺寸的真实断言失败”。首次仍不改原断言；只有定位证据成立，才修测试 helper。
3. 再运行原用例和身份对照；原三次布局应按顺序在同一表面做到窄 `640×520`、中 `900×620`、宽 `1280×800`，以原断言为准。若新的真实失败浮现，要公布其确切边界，不得临时改 `.zui`、删断言、伪造视口或宣称整个编辑器/UE 已通过。

本地 UE 文件 SHA-256：

- `Engine/Plugins/Tests/AutomationDriverTests/Source/AutomationDriverTests/Private/AutomationDriver.spec.cpp`：`62a0e0a384be81fae4e10ec29925fd6058332fe850842daed0aa853163422da6`
- `Engine/Source/Developer/AutomationDriver/Private/LocateBy.cpp`：`809bce4751b4fa11872cdf438290fc40776bbbf479ca33eb804d7da2af222065`

## 7. 原定位器与独立真实身份观测

实际编译 `71768e68a01834f5952845248e0f12471cdbd08b`；同一个原 `.zui` 文档与原 `UiV2SurfaceBuilder`，不依赖 #232/#233 合并。Windows x64 MSVC，`zircon_runtime --no-default-features --features native-host,ui --test zui_reactbits_agent_workspace_contract --profile test --locked --offline`；真实 EXE `2c5bb4f3e6497237c94d215e56eb891e8620ce82e0e0746a9bc99e055e7ccb80`，执行回执 `72a9a8115b2ce67c733d0d5c7e83fb5a32d2b3d268b706559277ae89599984a5`，`reactbits_ --test-threads 1 --show-output`。**1 过 / 1 败**；编译与运行分别计。

| 作者节点 | 实际 retained path | metadata / 公开唯一 ID 查找 | 旧路径查找 |
|---|---|---|---|
| `root` | `v2/root` | `ReactBitsAgentWorkspaceRoot` → `1` | `[]` |
| `sidebar` | `v2/ReactBitsAgentSidebar` | `ReactBitsAgentSidebar` → `2` | `[2]` |
| `main` | `v2/ReactBitsAgentMain` | `ReactBitsAgentMain` → `22` | `[22]` |
| `context` | `v2/ReactBitsAgentContextPanel` | `ReactBitsAgentContextPanel` → `70` | `[70]` |
| `context_summary` | `v2/ReactBitsAgentContextSummary` | `ReactBitsAgentContextSummary` → `56` | `[56]` |

源/compiled 根为 `root`，实际 metadata 索引与 retained 根一致。用例还在明确隔离的派生文档让 `main` 和根重名：单义 API 返回 None，旧路径反而指向 `main`；已移除的原 `main` 名称也返回 None。派生文档仅用于负控，任何尺寸结论都来自未编辑的原文档。

两个测试分开判定：身份观测通过；原 `reactbits_agent_workspace_builds_a_responsive_flow_surface` 仍在 helper 第 25 行失败，未执行任何 `compute_layout`。没有用测试维护冒充生产 UI 改进。

## 8. 只修验收查找，保留原响应式断言

实际编译 `1df3f8f859b0db34812870cc4ae48f0fc31f68f1`；同一个原 `.zui` 文档与原 `UiV2SurfaceBuilder`，不依赖 #232/#233 合并。Windows x64 MSVC，`zircon_runtime --no-default-features --features native-host,ui --test zui_reactbits_agent_workspace_contract --profile test --locked --offline`；真实 EXE `1bbb330bf206a0e25e57a3b4c5791684011c6405fe9c633752d466ffb2978eb6`，执行回执 `6ab28890f5e0dc83aa2ba49dd034c05ac364fb5cb3559672d4314966ef1eae94`，`reactbits_ --test-threads 1 --show-output`。**2 过 / 0 败**；编译与运行分别计。

| 作者节点 | 实际 retained path | metadata / 公开唯一 ID 查找 | 旧路径查找 |
|---|---|---|---|
| `root` | `v2/root` | `ReactBitsAgentWorkspaceRoot` → `1` | `[]` |
| `sidebar` | `v2/ReactBitsAgentSidebar` | `ReactBitsAgentSidebar` → `2` | `[2]` |
| `main` | `v2/ReactBitsAgentMain` | `ReactBitsAgentMain` → `22` | `[22]` |
| `context` | `v2/ReactBitsAgentContextPanel` | `ReactBitsAgentContextPanel` → `70` | `[70]` |
| `context_summary` | `v2/ReactBitsAgentContextSummary` | `ReactBitsAgentContextSummary` → `56` | `[56]` |

源/compiled 根为 `root`，实际 metadata 索引与 retained 根一致。用例还在明确隔离的派生文档让 `main` 和根重名：单义 API 返回 None，旧路径反而指向 `main`；已移除的原 `main` 名称也返回 None。派生文档仅用于负控，任何尺寸结论都来自未编辑的原文档。

只把 helper 从扫描 `v2/<control_id>` 换为正式的唯一控件入口；原聚合函数体、三次尺寸（640×520、900×620、1280×800）、后续子节点路径/样式与夹具未改。这一次真实原测试返回成功；不等于 GPU、桌面互动或整编辑器验收。
