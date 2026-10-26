# UE 工作窗口：Penpot 设计与 Editor 验证

## 当前要求

用户指定以 `E:/Git/ZirconEngine/dev/UnrealEngine` 为参考，覆盖 ZirconEngine 用得到的 UE 工作窗口。每个窗口先形成真实可编辑的 Penpot 设计，再将导出的 ZUI 应用到实际 Editor 验证。

基础组件与样式需要复用，支持后续统一设置和自定义样式。尽量一比一还原；占位界面、静态预览图和无操作效果的按钮不能算完成。无法完全还原的部分保留具体差异、证据和待用户设计方案。

用户说明已手动接管独立 session。Parent 为 `codex://threads/01a16d4e-0491-7353-84fa-038bd7706546`。原先创建的两个子会话已停止，未继续作为独立 session 替代物；其未验证的初步发现仅作后续核查线索。九个独立聊天及实际 Penpot 文件/页面已由 Parent 分配，身份、共享 owner 与桌面交接见 [主统筹记录](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)。主工作台当前证据和下一步见 [交接记录](main-workbench-handoff.md)。

## 窗口族与交接边界

下表为盘点入口，尚未宣称覆盖全部适用 UE 窗口。各域须从 Toolkit、TabFactory、Slate 布局及 Zircon 功能/路线继续补齐。

| 窗口族 | 初始范围 | 共享依赖 | 当前验收状态 |
| --- | --- | --- | --- |
| 材质 | Material、Material Instance/Function及适用材质工具 | 共享图表面由材质 owner 牵头；属性行、资产选择、实时视口 | 尚无本轮 Penpot/Editor 验收 |
| 动画 | Sequence、Montage、BlendSpace、动画状态机及适用动画工具 | 共享时间轴、曲线与播放控件由动画 owner 负责 | 尚无本轮 Penpot/Editor 验收 |
| 角色与骨骼 | Skeleton、Skeletal Mesh、Persona/Character、Physics Asset、Retarget、Control Rig | 骨骼树、属性事务、真实角色视口；消费共享时间轴与图表面 | 尚无本轮 Penpot/Editor 验收 |
| 粒子与效果 | Niagara System/Emitter及适用粒子作者工具 | 参数树、图/曲线、播放时钟、真实效果预览 | 尚无本轮 Penpot/Editor 验收 |
| Widget/UI | Widget Designer及适用动画/预览；Graph须有实际模型和执行能力 | Palette/Hierarchy、Designer、Details、共享图/曲线与预览 | 尚无本轮 Penpot/Editor 验收 |
| AI | Behavior Tree、Blackboard及适用关联调试工具 | 共享图节点/连接、属性、选择焦点与真实调试状态 | 尚无本轮 Penpot/Editor 验收 |
| 设置与插件 | Editor Preferences、Project/World Settings、Plugin Browser及适用配置窗口 | 分类导航、搜索、属性表单、重置、持久化反馈 | 尚无本轮 Penpot/Editor 验收 |
| 资源抽屉与资源工具 | Content Browser/Drawer、Picker/Import、Static Mesh、Texture、Sound、DataTable及独立 Curve 工具 | 文件树、网格/列表、真实缩略图、详情、文档；曲线/播放复用动画 owner | 尚无本轮 Penpot/Editor 验收 |
| Actor/Blueprint | Actor、Blueprint及适用组件作者工具 | 共享图、组件树、Details、文档与命令/撤销 | 尚无本轮 Penpot/Editor 验收 |
| 公共窗口框架 | 菜单、工具栏、文档页签、Dock、状态栏、弹层、全局主题与组件库 | 已有 catalog、tokens、Starship 样式及真实 host | 主窗口已有独立证据；不扩展为其它窗口通过 |

Static Mesh 与 Texture 独立资产编辑器归第 8 族资源工具；材质域只拥有材质/实例/函数及其资产引用适配。Skeletal Mesh、Skeleton 与 Physics Asset 归角色骨骼域。跨域关联跳转通过实际 registry/Toolkit 注册对接，由 Parent 协调公共宿主；本次归属修正不表示缺失的 Toolkit 已实现。资源族共享消费者合同见 [RT-S01～09](08-resource-tools-shared-contracts.md)。

Actor/Blueprint 额外盘点的 45 项窗口候选已逐项登记 [Parent 归属对账](parent-window-coverage.md)。未有独立实现聊天的世界创作、导航、诊断、渲染/设备等工作保持 Parent 待接续；不将建议 owner、盘点或区域布局草稿计为完成。

## 每个独立 session 的交付

1. 适用窗口清单：具体 UE 源文件/版本、Zircon 当前能力、计划能力、排除或待定理由。
2. 公共组件映射及新增能力提案；遵循 [组件复用约定](component-reuse.md)。
3. Penpot 文件链接、可编辑组件/实例、各主要状态、保存关闭重开及官方导出证据。
4. 导出的 ZUI 与组件依赖、实际 Editor 窗口入口及真实数据/命令绑定。
5. 同尺寸、同状态、同实际 DPI 的对照图、差异图与实际操作记录。
6. 未还原项：参考与实际、具体差异、原因/未知、依赖、待用户方案、未完成状态。

状态按“盘点 / Penpot 设计 / 待实现 / 待验证 / 验证失败 / 指定范围验证通过”记录。仅有外观不得升级为功能完成。

## 公共修改与产物

公共组件、token、样式、catalog、菜单/窗口注册和跨窗口协议需要明确归属后集成，避免各域各建一套。独立 session 先在自己的域记录提案，再按归属合入。生产源码、共享依赖和其它聊天的产物继续遵守仓库规则。

本轮产物登记：`E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/artifacts.json`。目前没有本窗口套件的正式验收，也没有释放磁盘空间。

可复用技能入口：[UI Layout Reference](../../../.codex/skills/zircon-project-skills/zr-ui-layout-reference/SKILL.md)，详细流程：[UE/Penpot fidelity](../../../.codex/skills/zircon-project-skills/zr-ui-layout-reference/references/ue-penpot-fidelity.md)。
