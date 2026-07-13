---
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_editor/src/lib.rs
  - zircon_plugins/plugin_sdk/src/lib.rs
  - .github/workflows/ci.yml
plan_sources:
  - docs/plans/zircon_runtime/frameworks/index.md
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/engine-architecture/workspace-root-rules-and-hard-cutover.md
  - docs/engine-architecture/large-file-ownership-m1.md
  - docs/engine-architecture/hard-cutover-migration-smells-m1.md
  - docs/engine-architecture/non-network-server-naming-m1.md
  - docs/engine-architecture/generated-code-boundary.md
  - docs/engine-architecture/plugin-optional-feature-bundles.md
  - docs/runtime-plugins/profile-selection.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
---

# Zircon 开发规范准则（总纲）

本文是 ZirconEngine 的开发规范单一权威入口（计划 06 M0 交付物），覆盖 runtime、editor、plugins 三个开发域与全部横切面。既有专项文档（workspace-root-rules、hard-cutover-smells、structure-convention 等）保留为细节论证，与本文冲突时以本文为准并回写勾稽。

**级别定义**：
- **MUST**：违反即阻断合入；每条 MUST 必须有守卫（编译器 / 守卫测试 / CI 步骤 / 审计脚本），守卫列标注 G1–G7（见计划 06 §4）或"评审"（守卫落地前的过渡态）。
- **SHOULD**：默认遵守；偏离需在 PR/计划文档中记录理由。
- **豁免流程**：MUST 的局部豁免必须在代码处标注 `// EXEMPT(<规则ID>): <理由>` 并可被守卫统计；无标注的违规一律视为缺陷。

---

## 一、通用规范（GEN：三域共用）

### GEN-S 结构与依赖

| ID | 级别 | 规则 | 守卫 |
|----|------|------|------|
| GEN-S1 | MUST | 公开形态固定为 `zircon_app` / `zircon_runtime` / `zircon_editor` / `zircon_runtime_interface`（+`zircon_hub` 启动器、`zircon_plugins` 独立 workspace）。新增顶层 crate 须先修订 frameworks/index 决策记录 | 评审 |
| GEN-S2 | MUST | 依赖方向遵守 `architecture-overview.md` 图 2 分层：只准上层依赖下层；app/editor/插件禁止直连 `zr_*` 内部 crate | G1 |
| GEN-S3 | MUST | 根文件（lib.rs/mod.rs/main.rs）只含子模块声明、curated re-export、最小入口接线；禁止行为逻辑与跨域编排 | G2 |
| GEN-S4 | MUST | 生产 Rust 文件 <1000 行；超限按 owner 分组拆分，禁止按行数机械切块 | G2 |
| GEN-S5 | MUST | 重型依赖（wgpu/winit/naga/gltf/image 等）只允许出现在批准的宿主 crate（图 2）；新增外部依赖走 `[workspace.dependencies]` 单源并说明选型 | G6 |
| GEN-S6 | SHOULD | 目录即架构：新子系统先画目录/文件角色再写代码，对照参考引擎目录形态（zr-reference-engine-routing） | 评审 |

### GEN-M 迁移与演进

| ID | 级别 | 规则 | 守卫 |
|----|------|------|------|
| GEN-M1 | MUST | 硬切换：新 owner 路径落地时同批迁移全部调用方并删除旧路径；禁止 `legacy/compat/shim`、alias crate、迁移语境 bridge、"暂时的" re-export | G2 |
| GEN-M2 | MUST | 非网络语义禁用 `server` 命名（crate/trait/handle/注释） | G2 |
| GEN-M3 | MUST | 生成代码只能是叶子数据/表/manifest/schema/adapter；禁止生成物持有 bootstrap、插件注册、模块解析、ECS 变更等架构行为 | G2 |
| GEN-M4 | SHOULD | 破坏性变更集中成批（一个里程碑内切完一个面），避免长期双态 | 评审 |

### GEN-Q 代码质量

| ID | 级别 | 规则 | 守卫 |
|----|------|------|------|
| GEN-Q1 | MUST | 错误处理：每个域一棵 thiserror 错误树；错误必须携带上下文（谁、期望什么、实际什么）；跨边界错误转换保留源错误链 | 评审→G3 |
| GEN-Q2 | MUST | 运行时路径（帧循环、加载链、生命周期推进）禁止 `panic!/unwrap/expect`；仅允许于：测试、不变量被内核已验证的私有函数（须注释 invariant）、进程启动前配置阶段 | G3(clippy 配置) |
| GEN-Q3 | MUST | `unsafe` 只允许在 FFI 边界（dynamic_api、native loader、interface）与 RHI 层；每处必须注释安全前提（SAFETY:） | G3 |
| GEN-Q4 | MUST | 日志分级纪律：默认/Minimal profile 安静；周期性 cadence 日志仅 Dev profile 默认开启；禁止 println!，统一走诊断设施 | G3 |
| GEN-Q5 | SHOULD | 注释只写代码本身讲不出的东西：关键数据结构、不变量、非显然状态迁移、决策理由；不注释显然赋值与机械转发 | 评审 |
| GEN-Q6 | SHOULD | 魔法常量收敛到域内常量模块并命名（zr-magic-constant-convergence）；禁止同一常量多处字面量 | 评审 |
| GEN-Q7 | MUST | `cargo fmt --all --check` 通过；clippy 按 allowlist 递减制收紧至全 workspace 零警告 | G3 |

### GEN-T 测试与验证

| ID | 级别 | 规则 | 守卫 |
|----|------|------|------|
| GEN-T1 | MUST | 遵循 milestone-first 节奏：实现切片不逐片编译；里程碑测试阶段统一跑该计划声明的命令集并记录证据 | 流程 |
| GEN-T2 | MUST | 测试分层齐备才算"完成"：内核/单元测试（逻辑）、契约测试（跨域/ABI 面）、守卫测试（结构规则）、集成/冒烟（启动路径）各就其位；上层测试失败先修最低共享支撑层 | 流程 |
| GEN-T3 | MUST | 触碰生命周期、依赖排序、序列化、状态迁移、失败边界时，必须补对应契约测试 | 评审 |
| GEN-T4 | SHOULD | 故障注入测试优先于快乐路径覆盖率：加载失败、坏输入、能力缺失都要有断言错误码的测试 | 评审 |

### GEN-D 文档

| ID | 级别 | 规则 | 守卫 |
|----|------|------|------|
| GEN-D1 | MUST | 有意义的模块新建/重组必须创建或更新 `docs/` 源路径镜像文档，维护 `related_code/implementation_files/plan_sources/tests` 头部 | G7 |
| GEN-D2 | MUST | 文档头部引用的路径必须真实存在 | G7 |
| GEN-D3 | MUST | 权威计划集（frameworks/runtime/render/shader/text/editor_layout）之间的交叠先勾稽后动代码；规则修改只改本总纲并同步守卫 | 评审 |
| GEN-D4 | SHOULD | 公共 API（门面 re-export 面、interface 全部、plugin_sdk 全部）有 rustdoc；示例优先于形容词 | 评审 |

---

## 二、Runtime 开发规范（RT）

| ID | 级别 | 规则 | 守卫 |
|----|------|------|------|
| RT-1 | MUST | core 脊柱角色固定：`runtime`=生命周期/调度内核，`manager`=服务名/resolver/handle 访问层，`framework(zr_contracts)`=纯契约零实现零重依赖，`math/resource`=共享地基。放错层的代码按 owner 迁移 | G2 |
| RT-2 | MUST | 新模块必经内核：声明 `ModuleDescriptor`（name/init_level/dependencies/drivers/managers/systems/features），实现 build/ready/finish/cleanup 四阶段；禁止绕过 CoreRuntime 自行构造与手工排序 | 评审→G2 |
| RT-3 | MUST | 依赖规则：Driver 只依赖 Driver；Manager 可依赖 Driver/Manager；业务对象不缓存跨生命周期强引用；服务命名 `ModuleName.ServiceKind.ServiceName` | 内核 KernelError + 单测 |
| RT-4 | MUST | 跨域引用只准三形态：extract/snapshot DTO（数据面）、registry 注册（扩展面）、handle+resolver（服务面）；禁止 `use crate::<邻域>::` 触内部类型（计划 05） | G1 |
| RT-5 | MUST | 可选子系统必须 feature 门控且满足可加性：任一域单开/单关都可编译；cfg 门开在模块声明与组装表，不开在业务逻辑深处 | G4 |
| RT-6 | MUST | world 权威在 scene(ECS)：`WorldTransform` 等派生数据由系统计算；序列化不含作者态；渲染侧只见 RenderExtract 产物 | 契约测试 |
| RT-7 | MUST | 热路径零意外开销：extract、文本 shaping、draw 组装等每帧路径禁止新增堆分配/动态派发/锁竞争，契约设计用泛型或批量 DTO；性能敏感改动附 stats 证据（勾稽 render/17） | 评审 + perf 计数断言 |
| RT-8 | MUST | wgpu 类型不出 RHI/graphics 依赖树；platform 类型（winit）不出 platform 层；上层用句柄（index+version）与描述符 | G6 |
| RT-9 | SHOULD | System/调度阶段语义遵循 runtime/03 计划（PreUpdate/Update/LateUpdate/FixedUpdate/RenderExtract）；新增阶段须修订该计划 | 评审 |
| RT-10 | SHOULD | 诊断先行：新子系统落地时同步注册其 stats/诊断通道，"不可观测的系统"不算完成 | 评审 |

## 三、Editor 开发规范（ED）

| ID | 级别 | 规则 | 守卫 |
|----|------|------|------|
| ED-1 | MUST | 顶层三域固定：`core/`（命令/历史/意图/事件 journal/项目会话/纯编辑器状态机）、`scene/`（选择集/viewport 工具/gizmo/世界桥）、`ui/`（workbench/布局/绑定/宿主）；新代码先归域再落文件 | G2 |
| ED-2 | MUST | 作者态只在 editor：选择集、viewport 工具状态、相机 override、gizmo/handle/overlay 生成不得进入 runtime world 或其序列化 | 契约测试 |
| ED-3 | MUST | editor 消费 runtime 只走契约/句柄/快照投影；禁止持有 world 内部对象、直接改 ECS 组件（一律经编辑命令意图 DTO） | G1 + 评审 |
| ED-4 | MUST | 一切用户可撤销操作走命令/undo/journal 管线；禁止绕过命令系统直接落状态（回放/协作依赖此不变量） | 评审 |
| ED-5 | MUST | editor UI 遵循 `docs/plans/zircon_editor/editor_layout` 规范层（设计令牌/声明式布局/提交契约）；不得另立 UI 提交口径；Slint 不回流 editor | G2 |
| ED-6 | MUST | overlay/gizmo 渲染只产出中立 overlay packet DTO（zr_contracts::render），graphics 不依赖 editor crate | G1 |
| ED-7 | SHOULD | editor 扩展面对齐 Fyrox EditorPlugin 语义（on_start/on_sync_to_model/on_mode_changed/...），与 runtime 插件分离；编辑器工具不污染游戏构建物 | 评审 |
| ED-8 | SHOULD | 长任务（导入/烘焙/构建）必须异步 + 可取消 + 进度上报，不得阻塞 UI 线程帧预算 | 评审 |

## 四、Plugins 开发规范（PL）

| ID | 级别 | 规则 | 守卫 |
|----|------|------|------|
| PL-1 | MUST | 元数据单源：一切插件元数据（id/能力/模块/目标/成熟度/打包）只在 `declare_plugin!` 声明一次；plugin.toml 是生成物；禁止手写重复常量 | G5 |
| PL-2 | MUST | 新插件用 `cargo zircon plugin new` 脚手架生成骨架；目录形态固定 `<id>/{runtime,editor?,dist?}/` + 生成的 plugin.toml | G5 |
| PL-3 | MUST | 能力命名走既有 namespace（`runtime.plugin.*`、`runtime.asset.importer.*` 等）；feature bundle 规则遵守 owner 唯一 + all-of 依赖语义 | G5 + 审计 |
| PL-4 | MUST | ABI 纪律：dist(cdylib) 边界只传 `#[repr(C)]` 值、句柄、`ZrByteSlice/ZrOwnedByteBuffer` 序列化载荷；禁止跨界传 Rust trait 对象/集合/字符串切片；符号与 ABI 版本由宏生成，不手写 | G5 + 契约测试 |
| PL-5 | MUST | 插件访问宿主只经 host API 函数表与扩展槽注册（RuntimeExtensionRegistry）；禁止链接期直连 `zircon_runtime` 内部路径；`zircon_runtime` 反向只消费注册报告与 manifest，不依赖插件实现 crate | G1 |
| PL-6 | MUST | 版本与成熟度诚实申报：`sdk_api_version` 语义化；maturity（experimental/beta/stable）如实标注——stable/默认 profile 不接受 required 的 experimental 插件 | 审计 |
| PL-7 | MUST | 生命周期：默认 `InitLevel::Post`；声明更早层级需在描述符注明依据；实现四阶段钩子，支持热重载的插件必须实现 save_state/restore_state 且状态可序列化 | 评审 + 契约测试 |
| PL-8 | MUST | 每个插件最低测试面：注册报告快照测试 + 能力协商测试；importer 类另需样例资产端到端导入测试；dist 形态需 `cargo zircon plugin check` 符号探测通过 | G5 |
| PL-9 | MUST | 失败必须可诊断：插件内部错误经 diagnostics 通道结构化上报，禁止吞错静默降级；panic 不得越过 FFI 边界（catch_unwind 收口） | 契约测试 |
| PL-10 | SHOULD | 第一方插件同批适配 SDK 变更（同仓硬切换）；SDK 对外破坏集中发生并在 README 迁移表记录 | 流程 |
| PL-11 | SHOULD | 插件间不直接通信，经宿主 broker（事件/bridge）；确需共享数据结构时先落 interface 契约 | 评审 |

## 五、Interface / ABI 规范（IF）

| ID | 级别 | 规则 | 守卫 |
|----|------|------|------|
| IF-1 | MUST | `zircon_runtime_interface` 只放契约：handles/status/buffer/函数表/manifest DTO/reflect/resource/ui 中立契约；零引擎行为实现 | G2 |
| IF-2 | MUST | 已发布函数表结构 `#[repr(C)]` 字段顺序永冻结；演进只准新增 `V(n+1)` 后缀共存，配 size_bytes/abi_version 校验；禁止原地改布局 | 契约测试 |
| IF-3 | MUST | interface 变更必须同批更新双端（runtime 实现 + 消费方）与契约测试；`cargo test -p zircon_runtime_interface --locked` 是其独立门 | CI |
| IF-4 | SHOULD | 新契约优先复用既有词汇（handle/descriptor/packet/manifest/report）；DTO 命名 `Zr*` 前缀 | 评审 |

## 六、工作流规范（WF）

| ID | 级别 | 规则 | 守卫 |
|----|------|------|------|
| WF-1 | MUST | 实质性工作先计划后代码：落在对应权威计划集（frameworks/runtime/render/...），里程碑分"实现切片 + 测试阶段"，测试阶段声明命令/验收证据/待更新文档 | 流程 |
| WF-2 | MUST | 架构级改动先写架构注记（owner 边界/所需契约/参考引擎先例/深度理由/验证层），过架构深度测试再实现（zr-architecture-first-engineering） | 评审 |
| WF-3 | MUST | 构建纪律：所有 Cargo 构建只允许写入 D/E/F 盘根目录下的 `cargo-targets`、`targets`、`ZirconBuilds`（共九个根及其 WSL 挂载等价路径）；必须由协调器按仓库、Windows/WSL、工具链、目标架构、工作区、构建配置组成兼容键并独占单一主池；无完整键或显式临时产物释放后立即删除；磁盘 ≤50GB 按 LRU 清理空闲池；禁止在其他位置构建或为忙碌兼容键另建目录 | 流程 |
| WF-4 | MUST | 提交面完整：一个里程碑的提交含代码 + 测试 + docs 镜像 + 计划状态回写；禁止"代码先行文档后补"跨里程碑欠账 | 评审 |
| WF-5 | SHOULD | 本地合入前跑 `tools/check-conventions`（守卫聚合脚本，计划 06 M1）；CI 是兜底不是首道门 | 流程 |

---

## 七、快速对照：我要做 X，看哪几条

| 场景 | 必读规则 |
|------|---------|
| 给 runtime 加一个新子系统 | GEN-S2/S3/S6、RT-1/2/3/4/5/10、GEN-T2/T3、GEN-D1 |
| 写一个新插件 | PL-1…PL-9、IF-4、GEN-Q1/Q4 |
| 给 editor 加面板/工具 | ED-1…ED-6、GEN-D1、ED-8 |
| 改跨 crate 边界/搬代码 | GEN-M1、GEN-S2、RT-4、GEN-D3 |
| 改 dynamic_api / 函数表 | IF-1/2/3、PL-4、GEN-Q3 |
| 调渲染/热路径 | RT-7/8、render 计划集、GEN-Q5 |
| 引入新外部依赖 | GEN-S5、G6、WF-2 |

## 八、状态与勾稽

- 本文各 MUST 的守卫落地进度由计划 06 里程碑追踪（M1 CI 基础门 → M2 结构守卫统一 → M3 渐进收紧）；守卫未就位期间"评审"为过渡守卫，规则效力不打折。
- 被收编文档需在头部加勾稽行指向本文（计划 06 M0 切片）。
- 规则新增/修改：改本文 + 更新守卫 + 在本节记录变更日期与理由。初版：2026-07-02，随 frameworks 计划集建立。
