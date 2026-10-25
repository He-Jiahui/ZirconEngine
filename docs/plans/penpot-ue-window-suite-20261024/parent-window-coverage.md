# 其它工作窗口：Parent 归属对账

本表接续 [Actor/Blueprint 窗口盘点](actor-blueprint-window-inventory.md) 的 OVW-01～45，统筹 owner 为 `01a16d4e-0491-7353-84fa-038bd7706546`。现有九个独立聊天、实际 Penpot 文件和桌面入口见 [主统筹记录](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)。

“牵头收件”指盘点、设计/业务适配及后续具体范围的接续责任；不是已实现、共享源码独占权或验收通过。适用性、源码与能力差异仍以原盘点逐项证据为准。当前未另建世界创作、导航、诊断、设备或 Runtime 实现聊天；表中 Parent 项保留待接续，不能写成已分配给不存在的会话。

Static Mesh/Texture、Audio 全族、Font/Font Face、String Table 的资产作者入口归资源；Widget 消费字体和 Utility UI。Sound Cue/Class/Submix 的真实模型/编译/执行尚未证明，共享图表面不能关闭该后端缺口。Actor 的类/接口/构造/事件图由本域提出真实模型、保存/命令合同，Parent 对接现有脚本/VM/场景模块；当前没有已确认的完整 Blueprint 编译后端或独立实现 owner。内嵌 Timeline 复用动画表面，业务时间和回调仍归 owning Actor 文档。

本轮接续已核对资源域 [RT-AUDIO01](08-resource-tools-shared-contracts.md#rt-audio01cueclasssubmix图消费合同)：既有 SoundMixerGraph、验证器和 Kira compile 路径可复用；它们不等价于独立 Cue 程序资产、Class 继承或完整 Submix 作者链。当前 Kira `validate_m1_surface` 对效果链/高级 controls/pre-effect sends 的限制须保留，配置被接收不能单独证明可播放。Audio owner 已明确，不再列为待用户分配；实际能力、保存/undo、正常窗口与声音输出证据仍分别开放。Actor 的 rev12 Penpot 业务数据切片、只读 Prefab adapter 及导出/验证阻塞见 [Parent 最新接续](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md#actor-第二次接续rev12-数据切片与三个实际接入边界)，未扩展下表45项的实现范围。

| 候选 | 窗口 | 牵头收件 | 消费/协作 | 接续边界与未完成项 |
| --- | --- | --- | --- | --- |
| OVW-01 | Landscape Manage / Sculpt / Paint | Parent | — | 主场景/世界创作待接续；没有独立世界创作聊天 |
| OVW-02 | Foliage Paint / Select / instance list | Parent | — | 主场景/世界创作待接续；没有独立世界创作聊天 |
| OVW-03 | Navigation 可视化/数据切换 | Parent | — | Navigation 作者/作业入口待接续；AI 消费调试与导航结果 |
| OVW-04 | NavMesh/NavigationSettings 属性与 bake | Parent | — | Navigation 作者/作业入口待接续；AI 消费调试与导航结果 |
| OVW-05 | Levels / Level Details | Parent | — | 主场景/世界创作待接续；没有独立世界创作聊天 |
| OVW-06 | World Composition | Parent | — | 主场景/世界创作待接续；没有独立世界创作聊天 |
| OVW-07 | World Partition Editor | Parent | — | 主场景/世界创作待接续；没有独立世界创作聊天 |
| OVW-08 | Content Bundle Browser | Parent | 资源工具 | 主场景/世界创作待接续；没有独立世界创作聊天 |
| OVW-09 | Layers | Parent | — | 主场景/世界创作待接续；没有独立世界创作聊天 |
| OVW-10 | Data Layers Outliner | Parent | — | 主场景/世界创作待接续；没有独立世界创作聊天 |
| OVW-11 | Level Instance 编辑模式 | Parent | Actor/Blueprint（Prefab 边界） | 主场景/世界创作待接续；没有独立世界创作聊天 |
| OVW-12 | Actor Merge | 资源工具 08 | — | 资源文档/作者适配；数据/执行后端逐项核实 |
| OVW-13 | Output Log / console | Parent | — | 真实数据 provider/窗口/trace 待接续；没有独立诊断聊天 |
| OVW-14 | Device Output Log | Parent | — | 真实数据 provider/窗口/trace 待接续；没有独立诊断聊天 |
| OVW-15 | Message Log | Parent | — | 真实数据 provider/窗口/trace 待接续；没有独立诊断聊天 |
| OVW-16 | Statistics / scene resource statistics | Parent | — | 真实数据 provider/窗口/trace 待接续；没有独立诊断聊天 |
| OVW-17 | Session Frontend | Parent | — | 真实数据 provider/窗口/trace 待接续；没有独立诊断聊天 |
| OVW-18 | Automation 测试窗口 | Parent | — | 真实数据 provider/窗口/trace 待接续；没有独立诊断聊天 |
| OVW-19 | Insights Session Info / Trace Control | Parent | — | 真实数据 provider/窗口/trace 待接续；没有独立诊断聊天 |
| OVW-20 | Timing Insights | Parent | — | 真实数据 provider/窗口/trace 待接续；没有独立诊断聊天 |
| OVW-21 | Memory Insights | Parent | — | 真实数据 provider/窗口/trace 待接续；没有独立诊断聊天 |
| OVW-22 | Asset Loading Insights | 资源工具 08 | Parent 诊断 | 资源文档/作者适配；数据/执行后端逐项核实 |
| OVW-23 | Networking Insights | Parent | — | 真实数据 provider/窗口/trace 待接续；没有独立诊断聊天 |
| OVW-24 | Reference Viewer | 资源工具 08 | — | 资源文档/作者适配；数据/执行后端逐项核实 |
| OVW-25 | Size Map / Asset Audit / disk size | 资源工具 08 | — | 资源文档/作者适配；数据/执行后端逐项核实 |
| OVW-26 | Derived Data Resource Usage / Cache Statistics | 资源工具 08 | Jenkins 缓存/清理；窗口只读展示真实数据 | 资源文档/作者适配；数据/执行后端逐项核实 |
| OVW-27 | Localization Dashboard | 设置与插件 07 | — | 配置/本地化入口；复用真实字段与持久化 |
| OVW-28 | Device Manager | Parent | 设置（profiles 字段） | 设备/构建/部署能力待接续 |
| OVW-29 | UE 设备专用部署、Zen/DDC 服务管理、Kismet 内部对象工具 | Parent | — | 按原盘点审查 UE 专属适用性；不新增无效入口 |
| OVW-30 | Undo History | Parent | Actor 等各文档 history context | 真实数据 provider/窗口/trace 待接续；没有独立诊断聊天 |
| OVW-31 | Render Resource Viewer | Parent | 渲染 | 真实数据 provider/窗口/trace 待接续；没有独立诊断聊天 |
| OVW-32 | Pixel Inspector | Parent | 材质（真实帧/像素/颜色空间） | 真实数据 provider/窗口/trace 待接续；没有独立诊断聊天 |
| OVW-33 | Environment Light Mixer | Parent | — | 主场景/世界创作待接续；没有独立世界创作聊天 |
| OVW-34 | World Bookmark Browser | Parent | — | 主场景/世界创作待接续；没有独立世界创作聊天 |
| OVW-35 | Struct Viewer | 资源工具 08 | 设置/Actor 类型选择；Parent reflection catalog | 资源文档/作者适配；数据/执行后端逐项核实 |
| OVW-36 | Sound Cue / Sound Class / Sound Submix | 资源工具 08 | 材质（仅共享 graph 表面） | Audio 全族作者业务归资源；Cue/Class/Submix 后端未证明，不以播放服务冒充图作者能力 |
| OVW-37 | Font / Font Face | 资源工具 08 | Widget（字体消费）/Parent 字体与媒体就绪 | 资源文档/作者适配；数据/执行后端逐项核实 |
| OVW-38 | String Table Editor | 资源工具 08 | 设置（Localization 入口） | 资源文档/作者适配；数据/执行后端逐项核实 |
| OVW-39 | Source Control Changelists / 文件变更 | Parent | Jenkins | 版本信息窗口/请求回执；Git 写操作仍沿 Jenkins |
| OVW-40 | Config Editor / Device Profiles / Input Bindings | 设置与插件 07 | — | 配置/本地化入口；复用真实字段与持久化 |
| OVW-41 | User Asset Tags 管理 | 资源工具 08 | — | 资源文档/作者适配；数据/执行后端逐项核实 |
| OVW-42 | Gameplay Debugger 工具模式 | AI 06 | Parent 诊断/PIE 生命周期 | 真实 gameplay/agent 调试投影；共用宿主 |
| OVW-43 | Mass Entity Debugger | Parent | — | Zircon ECS 数据与 UE Mass 的适用性待定；不复制无来源统计 |
| OVW-44 | NNE Model Data Editor | 资源工具 08 | Parent neural/runtime | NNE/模型资产适用性及真实类型/后端待核实 |
| OVW-45 | Editor Utility Blueprint / Utility Widget | Actor/Blueprint 09 | Widget（Utility UI 内容）/Parent VM 和宿主接入 | Editor Utility 脚本/图合同；UI 内容消费 Widget |

本次不创建这些窗口的产品入口、伪造数据或平行组件。每项仍需实际 Penpot 可编辑设计/实例、真实数据/命令/撤销和保存、官方 ZUI 回写、正常 EXE/项目/case 与逐图验收。未确认适用的条目保留待定；原失败、历史回执和共享桌面交接保持原样。归属对账不会扩大已有材质测试配方，也不触发构建。

