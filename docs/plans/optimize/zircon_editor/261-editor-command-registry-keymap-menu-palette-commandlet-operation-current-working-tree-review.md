---
title: Editor Command Registry、Keymap、Menu、Palette、Commandlet 与 Operation Dispatch 当前工作树增量复审
category: zircon_editor
report_id: Editor261
review_date: 2026-08-31
baseline_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
verification_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/178-editor-command-registry-keymap-menu-palette-context-routing-remote-automation-current-source-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/260-editor-extension-contribution-store-toolkit-reload-lifecycle-current-working-tree-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/199-runtime-plugin-profile-catalog-provider-resolution-current-working-tree-review.md
related_plugin_owner:
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
related_code:
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/commandlet
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_control_requests.rs
  - zircon_editor/src/ui/retained_host/app/command_palette_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/command_palette.rs
  - zircon_editor/src/tests/workbench/reflection/remote_routes.rs
  - zircon_plugins/plugin_sdk/src/editor_contribution.rs
  - zircon_editor/src/core/plugin/materializer.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandList.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandInfo.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/InputBindingManager.h
  - dev/UnrealEngine/Engine/Source/Editor/ToolMenus
  - dev/godot/editor/editor_settings.cpp
  - dev/godot/editor/editor_command_palette.cpp
  - dev/Fyrox/editor/src/command/mod.rs
  - dev/Fyrox/editor/src/settings/keys.rs
  - dev/bevy/crates/bevy_ui_widgets/src/menu.rs
  - dev/Graphics/com.unity.graphtools.foundation/Editor/ContextualMenuDispatcher.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 1. 结论

本报告是对 Editor178 的当前工作树增量复审，不是把旧报告重新抄一遍。Editor178 的三个 P0（远程 InvokeBinding/InvokeRoute 的等价授权、公开 command id 与 host operation id 语法不一致、序列化 Command 没有可执行 factory）仍然是产品级阻塞；本轮不重复计数，而是验证它们在当前分支的状态，并补充命令注册、扩展投影、执行器、Palette、Keymap、Menu、Commandlet 和 Operation Dispatch 之间的新证据。

本轮没有发现可以取代 Editor178 三个 P0 的新 P0，但登记了 26 个当前仍为 Open 或 Partial 的 P1 追踪项（其中部分承接 Editor178 的现状）、10 个 P2 产品化缺口和 24 个工程闸门失败。最危险的当前工作树差异有四类：

1. `EditorCommandRegistry` 的 `Clone`/Serde 快照只保存描述符和工厂，丢弃 Native executor；同一个公开类型可以表现为“可发现但不可执行”。
2. `ContributionStore::active_batches()` 按 ticket 顺序投影，跨扩展命令引用没有依赖闭包或拓扑排序，结果随加载顺序改变。
3. SDK `EditorContributionBuilder::command()` 产生的 DTO 被 materializer 统一当成 `NativeEndpoint`，无 execution contract 的合法 SDK 命令会稳定地在发布阶段失败。
4. Palette 窗口虽然返回 `catalog_generation`，提交 callback 却只携带字符串 command id，旧窗口可以把同一 id 重新解析到新一代描述符/工厂。

这条链路目前仍是“多个局部注册表 + 多套路由 + String 错误/收据 + 调用方自带上下文”的临时拼接，尚未达到 Unreal Slate command list/ToolMenus、Godot editor command/shortcut、Fyrox command stack 或 Unity contextual menu 所要求的稳定 owner、代际、授权、可撤销和可诊断模型。建议先完成本报告的架构收敛，再进行任何新的 UI command 数量扩张。

# 2. 审查范围与证据边界

本轮扫描的当前工作树范围为 38 个 Rust 文件：

| 范围 | 文件数 | 总行数 | 非空行 | 字节 | 测试标记 |
| --- | ---: | ---: | ---: | ---: | ---: |
| `zircon_editor/src/core/commands` 全部文件 | 32 | 7,357 | 6,626 | 245,998 | 64 |
| `core/commandlet` runner/tests | 2 | 1,199 | 1,092 | 38,667 | 15 |
| operation/event/Palette host 与反射测试 | 4 | 1,238 | 1,190 | 48,432 | 1 |
| **合计** | **38** | **9,794** | **8,908** | **333,097** | **80** |

行数以当前工作树文件为准，目录中没有被忽略的 Rust 文件。工作树存在其他会话的未提交修改，本报告只读取并引用这些修改，不覆盖或回滚它们。基线和复核头均记录为 `14c89f9776bed828cc85e05e4b9914b3f8d1e784`；本轮是 review-only，没有 Cargo build/test、插件加载、UI 自动化、故障注入或性能压测。

Editor178 仍是命令平面 P0/P1 的历史 owner；Editor260 负责扩展 contribution、toolkit 和 reload 的跨 manager 原子性。本报告只接管命令平面在当前工作树上的新增证据，避免把同一个撤销/热重载问题重复登记。

# 3. 当前调用链

```text
SDK/serialized contribution
        |
        v
materializer -> ContributionStore -> contribution projection
                                    |
                                    v
                         EditorCommandRegistry
                          /       |        \
                    factories  executors  palette catalog
                       |           |             |
                       v           v             v
                 Operation      Native       Palette callback
                 dispatch       invoke       -> binding dispatch
                       |
                       v
                 event / commandlet / remote control
```

这张图中的每个箭头目前都有独立的 owner、generation、权限和错误语义。没有一个不可变 snapshot 同时约束“描述符、factory、executor、keymap、menu、palette”和调用上下文，因此热重载、序列化恢复和异步 UI 事件容易落到不同的代际。

# 4. 直接源码证据

1. [registry.rs:22-43](../../../../zircon_editor/src/core/commands/registry.rs) 将 `operation_factories`、`palette_catalog` 和 `executors` 都标记为运行时字段；`Clone` 重新创建空的 `EditorCommandExecutorRegistry`。直接 clone 的注册表保留了 Native 描述符，却失去所有执行入口。
2. [registry.rs:59-80](../../../../zircon_editor/src/core/commands/registry.rs) 的 `default_workbench()` 使用 `expect`，普通 `register()` 接受所有 action，包括没有 factory 的 `Operation`。
3. [registry.rs:83-122](../../../../zircon_editor/src/core/commands/registry.rs) 只有显式 `register_operation()` 才会把 descriptor 与 factory 同步插入；调用方仍可先 `register()` 再在执行时得到 `MissingFactory`。
4. [registry.rs:182-192](../../../../zircon_editor/src/core/commands/registry.rs) 与同文件的 `advance_generation()` 用 `checked_add(...).expect(...)`，generation 溢出会终止编辑器进程，且 executor 变更没有独立 generation。
5. [execution.rs:18-188](../../../../zircon_editor/src/core/commands/execution.rs) 只在 Native callback 返回后测量 elapsed time；超时没有取消、隔离线程、deadline 传播或强制终止语义。
6. [execution.rs:192-277](../../../../zircon_editor/src/core/commands/execution.rs) 的执行 receipt 只有 command/plugin/status/payload/diagnostics 字符串，没有 registry generation、owner ticket、session/document、principal、授权决策或结构化失败阶段。
7. [contribution.rs:124-179](../../../../zircon_editor/src/core/commands/contribution.rs) 从 `active_batches()` 逐批读取 command、factory、Native binding，并以当前 `available_operations` 验证菜单/视图/资产引用；没有 provider dependency graph 或拓扑排序。
8. [contribution.rs:273-280](../../../../zircon_editor/src/core/commands/contribution.rs) 允许菜单引用默认 operation 或此前已投影扩展的 operation，后加载扩展的同一引用会因 ticket 顺序变成 order-dependent 行为。
9. [contribution_store.rs:87-93](../../../../zircon_editor/src/core/extension/store/model/contribution_store.rs) 的 `active_batches()` 明确按 ticket 顺序返回活动批次，没有稳定 provider identity、依赖版本或 owner generation。
10. [descriptor.rs:24-34,67,350-352](../../../../zircon_editor/src/core/commands/descriptor.rs) 将 `callable_from_remote` 作为可序列化 bool，默认值为 `true`；它不是带 principal、scope、审计 reason 的授权策略。
11. [materializer.rs:189-245](../../../../zircon_editor/src/core/plugin/materializer.rs) 把序列化 `Command` 统一物化为 `NativeEndpoint`，缺少 execution contract 直接返回 registry error，缺少 binding context 返回 `MissingExecutor`。
12. [plugin_sdk/editor_contribution.rs:85-118,218-235](../../../../zircon_plugins/plugin_sdk/src/editor_contribution.rs) 的 `command()` 可以合法构建没有 contract 的 `SerializedEditorContribution::Command`，SDK 测试也只验证 DTO 排序成功；发布端随后必然拒绝它。
13. [runner.rs:422-520](../../../../zircon_editor/src/core/commandlet/runner.rs) 每次解析 commandlet 都新建 `EditorCommandRegistry::default_workbench()`，只能发现默认 command，项目/插件 command 不会进入 commandlet 解析。
14. [editor_operation_dispatch.rs:101-185](../../../../zircon_editor/src/ui/host/editor_operation_dispatch.rs) 分开 clone descriptor 与 factory，再按来源检查 remote bool、enablement 和 Native executor；这不是单一 immutable command snapshot，期间可发生代际漂移。
15. [editor_operation_dispatch.rs:460-530](../../../../zircon_editor/src/ui/host/editor_operation_dispatch.rs) 把结构化 `EditorCommandDispatchError` 转成 `ControlFailure` 的 String；journal 无法按错误码、阶段、generation 或 principal 聚合。
16. [editor_operation_dispatch.rs:520-560](../../../../zircon_editor/src/ui/host/editor_operation_dispatch.rs) 的 `ListOperations` 返回完整 Vec，没有 cursor、snapshot generation、session scope 或 disabled reason；`QueryOperationHistory` 使用全局固定窗口。
17. [command_palette_actions.rs:120-141](../../../../zircon_editor/src/ui/retained_host/app/command_palette_actions.rs) 会检查请求窗口的 `catalog_generation`，说明 UI 已承认代际概念，但这个字段没有传到执行 callback。
18. [callback_dispatch/workbench/command_palette.rs:21-44](../../../../zircon_editor/src/ui/retained_host/callback_dispatch/workbench/command_palette.rs) commit 只携带 `command_id: String`，成功后直接 dispatch 并记录 MRU；没有重验 generation、owner 或 descriptor digest。
19. [menu.rs:11-19,90-136](../../../../zircon_editor/src/core/commands/menu.rs) 使用固定根菜单和 `BTreeMap<String, MenuItemModel>`；重复 leaf 通过 `or_insert_with` 静默保留第一个，leaf action 为 `None`，下游再次按 id 查找。
20. [menu.rs:90-136](../../../../zircon_editor/src/core/commands/menu.rs) 的 shortcut 读取 descriptor 的 `default_chord`，不读取 keymap override；用户看到的菜单快捷键可能不是实际解析快捷键。
21. [keymap.rs:27-31,100-143](../../../../zircon_editor/src/core/commands/keymap.rs) 默认 keymap 使用 `expect`；`resolve_keyboard_input_when()` 返回 `Option<&str>`，多个 enabled 或 disabled/stale 情况没有 typed outcome。
22. [keymap.rs:145-190](../../../../zircon_editor/src/core/commands/keymap.rs) 以 pairwise `can_overlap_in_interactive_context` 做 O(n^2) 冲突检查；`with_overrides()` 整体 clone/rebuild，未绑定 registry generation、owner 或 atomic revision。
23. [key_chord.rs:1-90](../../../../zircon_editor/src/core/commands/key_chord.rs) 类型可直接 serde deserialize，`is_valid()` 没有成为反序列化/注册的硬闸门；非法或未规范化 chord 可进入外部配置。
24. [when.rs:35-230](../../../../zircon_editor/src/core/commands/when.rs) 能力条件是任意字符串，`CommandEvalCtx` 由调用者直接组装，包含文档/场景/选择 revision，却没有不可伪造的 session/document authority。
25. [editor_event_control_requests.rs:157-197](../../../../zircon_editor/src/ui/host/editor_event_control_requests.rs) 已将 EditorOperation binding 路由到 `EditorOperationSource::Remote`，这是对 Editor178 P0-01 的部分修复；但仍没有 principal、授权票据、deadline、idempotency 或审计上下文。
26. [remote_routes.rs:11-73](../../../../zircon_editor/src/tests/workbench/reflection/remote_routes.rs) 继续断言一批 workbench、viewport、inspector、asset import action `callable_from_remote == true`，证明当前产品面仍有宽泛的远程可调用面。

# 5. 与参考引擎的结构差异

| 能力 | Zircon 当前状态 | 参考实现提供的工程约束 | 差异后果 |
| --- | --- | --- | --- |
| command identity | `EditorOperationPath` + 可选字符串别名；owner/generation 不在 id 中 | Unreal `UICommandInfo` 由命令上下文、稳定 name、输入绑定和 style 共同定义；Godot command palette 以 editor command 注册表为单一来源 | 同一 id 在 reload/clone 后可能指向不同可执行对象 |
| executable binding | descriptor、factory、Native executor 三张表分开；serde/clone 丢运行时入口 | Unreal `FUICommandList` 把 action、can execute、can execute context 绑定到 command info；Fyrox command stack 的命令对象本身可执行/撤销 | “可发现但不可执行”、MissingFactory/MissingExecutor 延迟到点击时才暴露 |
| extension ordering | active batches 按 ticket 顺序；跨 batch 引用无闭包 | Unreal module/plugin dependencies、Godot extension registration 和 Bevy PluginGroup 都要求显式依赖顺序 | 加载顺序改变菜单、factory 和 capability 结果 |
| keymap | chord 解析返回 Option，覆盖整体重建，无代际 | Unreal `InputBindingManager`/Godot shortcut settings 提供冲突、默认值、持久化和上下文层级 | 冲突原因不可诊断，旧 UI 输入可能命中新 command |
| menu projection | 固定根菜单，重复 leaf 静默 first，shortcut 取默认值 | Unreal ToolMenus 有 owner、section、startup/shutdown、merge/rebuild；Bevy menu 组件有 typed action | 热插拔和多插件贡献无法可靠撤销/替换 |
| palette | 查询窗口带 generation，但 commit 丢失；locale cache 缺 digest | Godot editor command palette 以 command/shortcut/visibility 统一刷新 | stale window 可执行新一代同名 command |
| remote/commandlet | remote bool default allow；commandlet 只看默认 registry | Unreal commandlet/module startup 与权限边界分离；Godot editor settings/commands 不把远程调用当默认属性 | headless、remote、CLI 被混成同一授权面 |
| diagnostics | receipt/journal 以 String 为主 | Unreal logging/trace、Fyrox command errors、Unity contextual menu validation 均保留结构化上下文 | 无法按 owner、generation、principal 做故障定位和回放 |

# 6. Editor178 继承项与本轮状态

以下问题在 Editor178 已登记，本轮仅复核当前证据，不新增编号：

| 历史项 | 当前状态 |
| --- | --- |
| 远程 InvokeBinding/InvokeRoute 等价授权缺失 | EditorOperation 路径现在映射为 Remote，属于部分修复；principal、scope、deadline、审计仍缺失，保持 Open |
| public command id 与 host operation id 语法分裂 | 当前 registry/dispatch 仍使用两套路径约束，保持 Open |
| serialized Command 没有可执行 factory | 当前 materializer 统一 NativeEndpoint，仍保持 Open；本轮补充 SDK no-contract 的确定性失败证据 |
| registry 直接 Deserialize/Clone 后 factory/executor 缺失 | 当前代码仍把运行时字段 serde skip，Clone 还主动丢 executor；本轮拆分为 ED-CMD-02/03 追踪 |
| palette stale generation、keymap Option、remote default allow、commandlet defaults-only | 当前代码仍可重现，分别在 ED-CMD-14、18、12、10 追踪 |

# 7. P1 差异与重构要求

状态定义：`Open` 表示当前没有实现闭环；`Partial` 表示已有局部保护但跨层契约仍断裂；`Inherited` 表示由 Editor178 继续拥有，本报告只提供当前证据。

| ID | 当前状态 | 证据 | 工程化差异与重构要求 |
| --- | --- | --- | --- |
| ED-CMD-01 | Open | `registry.rs:66-80`; `editor_operation_dispatch.rs:300-350` | `Operation` descriptor 可绕过 `register_operation` 单独注册，点击时才 MissingFactory。把 command descriptor、operation factory、Native binding 合并为 typed `ExecutableCommandRecord`，注册时一次性验证。 |
| ED-CMD-02 | Open | `registry.rs:22-43` | `Clone` 丢 executor，Serde 丢 factories/executors/cache。定义显式 `CommandRegistrySnapshot` 与 `RuntimeCommandRegistry`，禁止公开 Clone 伪装成可执行快照；恢复必须返回 typed `RebindRequired`。 |
| ED-CMD-03 | Open | `registry.rs:225-252` | executor 注册/撤销不改变公开 registry generation，也不和 descriptor projection 原子提交。executor binding 必须属于同一 owner ticket、snapshot generation 和 commit receipt。 |
| ED-CMD-04 | Open | `execution.rs:151-188` | execution budget 是 callback 后测量，没有 deadline/cancel。采用可取消执行上下文、隔离线程/任务、deadline propagation、超时状态机和 shutdown drain。 |
| ED-CMD-05 | Open | `execution.rs:192-277` | receipt 只含 String diagnostics。增加 typed stage/error code、registry generation、provider/package、owner ticket、session/document、principal、authorization decision、started/finished time 和 payload digest。 |
| ED-CMD-06 | Open | `contribution.rs:124-179,273-280`; `contribution_store.rs:87-93` | 跨批次菜单、view、asset operation 依赖依赖加载顺序。发布前构建 provider graph，校验 missing/duplicate/version/cycle，按拓扑序 materialize；禁止隐式依赖 earlier ticket。 |
| ED-CMD-07 | Open | descriptor/factory/ContributionSnapshot | descriptor 和 factory 没有 provider identity、package version、owner generation。所有可执行记录都必须携带不可变 provenance，并由 ContributionStore 统一分配。 |
| ED-CMD-08 | Open | `plugin_sdk/editor_contribution.rs:85-118`; `materializer.rs:189-245` | SDK `command()` 产生无 contract DTO，materializer 必拒绝。二选一：为 DTO 增加 typed event/operation/headless action；或在 builder API 层禁止无执行语义的 command 构建。不能让合法 builder 在发布末端才失败。 |
| ED-CMD-09 | Open | `materializer.rs:189-245`; `editor_contribution.rs` DTO | 序列化模型只有 metadata/optional contract，没有 action kind、factory key、permission、remote policy、headless route、undo/redo contract。设计版本化 command schema，并为每种 action 提供明确 restore/rebind 协议。 |
| ED-CMD-10 | Inherited/Open | `runner.rs:422-520` | commandlet parser 每次只创建 default workbench registry。注入 project/session command snapshot，明确 commandlet startup、插件依赖、headless capability 和 deterministic mode。 |
| ED-CMD-11 | Open | `runner.rs:450-560` | commandlet report 缺 provider/generation/session/principal/receipt，只返回字符串/少量业务数组。改为复用 `EditorCommandExecutionReceipt` 的 typed report envelope，支持机器解析和回放。 |
| ED-CMD-12 | Inherited/Open | `descriptor.rs:24-34,67`; `remote_routes.rs:11-73` | `callable_from_remote` 默认 true，bool 不能表达 principal、scope、consent、audit。改为 default deny 的 capability/policy object，并在 Remote/Cli/Headless 进入统一 authorization service。 |
| ED-CMD-13 | Inherited/Open | `editor_operation_dispatch.rs:460-530` | ControlFailure 把 typed dispatch error 变成 String。journal 事件必须保存 code/stage/command/provider/generation/source/principal 和可选 cause chain。 |
| ED-CMD-14 | Inherited/Open | `command_palette_actions.rs:120-141`; callback `command_palette.rs:21-44` | 查询窗口校验 generation，commit 不传 generation。Palette item 应携带 snapshot handle/descriptor digest，执行前做 generation+owner+enabled 三重重验，返回 typed stale result。 |
| ED-CMD-15 | Open | palette catalog cache | locale cache 只按 locale/generation，未绑定 i18n bundle/source digest；seed 只有 category source tag。缓存 key 增加 registry/i18n/keymap revision、provider provenance 和 deprecation state。 |
| ED-CMD-16 | Open | `menu.rs:90-136` | 重复 leaf 静默保留 first，leaf action 为 None。菜单构建必须返回 duplicate/conflict diagnostics，使用 typed action reference，携带 owner token 和 contribution id。 |
| ED-CMD-17 | Open | `menu.rs:90-136`; `keymap.rs` | 菜单显示 descriptor default chord，不显示有效 override。Menu、Keymap、Palette 必须从同一个 command snapshot 生成 effective shortcut，并在 override 变更时原子重建。 |
| ED-CMD-18 | Inherited/Open | `keymap.rs:100-143`; `key_chord.rs` | resolver 用 Option 丢失 disabled/ambiguous/stale/invalid 原因，chord 可直接 deserialize。引入 `KeyResolution` typed enum、严格 serde validation、snapshot generation 和 conflict explanation。 |
| ED-CMD-19 | Open | `keymap.rs:145-190` | 冲突检测 O(n^2)，override 全量 clone/rebuild，无 revision/owner。使用按 normalized chord/context 的索引、增量更新和 atomic revision，保留冲突图用于 UI/诊断。 |
| ED-CMD-20 | Open | `when.rs:35-230` | arbitrary capability strings + caller-built context 允许伪造 capabilities/revisions。由 session/document authority 签发 `CommandEvalContext`，capability 使用 catalog ids，revision mismatch 必须 typed fail。 |
| ED-CMD-21 | Open | `registry.rs:182-192` | generation overflow `expect` 终止进程。采用 checked error/epoch rollover 或不可回收 128-bit revision；任何 rollover 要使旧 handle 明确 stale。 |
| ED-CMD-22 | Open | `contribution.rs:157-179,273-280` | menu capability map 只检查同 extension pending command/view，operation 可来自 defaults/previous extension，未检查 owner/dependency closure。统一所有 command/view/menu/asset reference 的 provider admission。 |
| ED-CMD-23 | Open | `editor_operation_dispatch.rs:520-560` | ListOperations 无 cursor、generation、session scope；history 固定全局 128。改为分页 snapshot API，按 session/document/principal 隔离，返回 disabled reason、owner、generation。 |
| ED-CMD-24 | Open | Native dispatch result codec | Native result 依赖精确字符串 `zircon.editor.command-result.v1`，无 negotiation/compatibility matrix。定义 versioned codec registry、size limits、typed decode errors 和 forward-compatible envelope。 |
| ED-CMD-25 | Open | registry/contribution/keymap/menu/palette 分散缓存 | command metadata、factory、executor、keymap、menu、palette 没有统一 commit boundary。构建 immutable `EditorCommandSurfaceSnapshot`，由一次 receipt 发布并让所有 UI/remote/commandlet consumer 持有同一 generation。 |
| ED-CMD-26 | Open | SDK/materializer/i18n paths | 当前 extension registration 流程未显示 command localization bundle 的同一 commit/install/source map；palette/menu 可看到 key 但找不到 bundle provenance。把 i18n bundle、source digest、fallback 和 unregister 绑定到 provider transaction。 |

# 8. P2 产品化缺口

| ID | 缺口 | 目标 |
| --- | --- | --- |
| ED-CMD-P2-01 | Command registry inspector | 展示 descriptor/factory/executor/owner/generation 的一致性和缺口 |
| ED-CMD-P2-02 | Dependency graph viewer | 展示 plugin command provider、依赖、cycle、被阻塞节点 |
| ED-CMD-P2-03 | Command lifecycle waterfall | 记录 DTO、materialize、admit、publish、retire、rebind 时延 |
| ED-CMD-P2-04 | Provenance browser | 从菜单/Palette/remote receipt 反查 package、source file、owner ticket |
| ED-CMD-P2-05 | Reload impact preview | reload/revoke 前列出受影响 command、keymap、menu、Palette、session |
| ED-CMD-P2-06 | Lease and quiescence dashboard | 显示执行中 callback、pending UI event、阻塞 retire 的 owner |
| ED-CMD-P2-07 | Receipt export/replay | 导出 typed command receipt，支持 deterministic headless replay |
| ED-CMD-P2-08 | Capability matrix UI | 按 source/principal/session 显示 command policy 和拒绝原因 |
| ED-CMD-P2-09 | Cache telemetry | 统计 palette locale/query、menu、keymap snapshot 命中率和失效原因 |
| ED-CMD-P2-10 | 100+ provider soak | 长时加载、替换、撤销、重连和随机 query 的内存/句柄/代际稳定性 |

# 9. 工程闸门

| Gate | 当前结果 | 通过条件 |
| --- | --- | --- |
| G-CMD-01 | Fail | 公开 registry clone/serde 后 executor/factory 语义明确且可验证 |
| G-CMD-02 | Fail | descriptor 与 executable binding 单次原子注册 |
| G-CMD-03 | Fail | provider dependency closure、版本和 cycle 在 publish 前验证 |
| G-CMD-04 | Fail | SDK 每种 command builder 都能成功 materialize 或在 builder 层拒绝 |
| G-CMD-05 | Fail | command action kind/factory/contract 版本化并可 rebind |
| G-CMD-06 | Fail | Native callback deadline 可取消，超时不会只在返回后标记 |
| G-CMD-07 | Fail | receipt/journal 使用 typed error、stage、generation、owner、principal |
| G-CMD-08 | Fail | remote/CLI/headless 统一 default-deny authorization |
| G-CMD-09 | Fail | session/document authority 签发 eval context，revision mismatch 可检测 |
| G-CMD-10 | Fail | Palette commit 携带并校验 catalog snapshot handle |
| G-CMD-11 | Fail | Menu/Keymap/Palette 使用同一 effective shortcut snapshot |
| G-CMD-12 | Fail | duplicate menu leaf、ambiguous chord 返回可定位诊断 |
| G-CMD-13 | Fail | key chord serde 在 admission 阶段严格规范化 |
| G-CMD-14 | Fail | keymap conflict index 增量更新，不再 O(n^2) 全表扫描 |
| G-CMD-15 | Fail | commandlet 使用 project/plugin command snapshot 而非 defaults-only |
| G-CMD-16 | Fail | ListOperations/history 有 cursor、generation、session/principal scope |
| G-CMD-17 | Fail | result codec 支持版本注册、大小预算和 forward compatibility |
| G-CMD-18 | Fail | i18n bundle/source digest 与 command provider transaction 绑定 |
| G-CMD-19 | Partial | EditorOperation binding source 已映射 Remote，但没有 auth receipt/audit |
| G-CMD-20 | Fail | generation rollover 不以 panic/expect 结束进程 |
| G-CMD-21 | Fail | immutable command surface snapshot 覆盖 metadata/factory/executor/keymap/menu/palette |
| G-CMD-22 | Fail | revoke/reload 可在统一 owner lease 下原子 retire consumer |
| G-CMD-23 | Partial | 已有 80 个测试标记，但缺跨代、跨 provider、stale callback、fault injection |
| G-CMD-24 | Fail | 100+ provider、长时间 reload 和 remote/commandlet soak 通过 |

# 10. 分阶段重构顺序

1. **先定义契约**：冻结 `EditorCommandSurfaceSnapshot`、`ExecutableCommandRecord`、provider identity、owner lease、generation、principal/session authority、typed receipt/error 和 versioned serialized schema。明确 event、operation、native、headless 四种 action，不再把 serialized Command 默认为 NativeEndpoint。
2. **收敛注册与投影**：让 SDK builder、materializer、ContributionStore 和 registry 使用同一 admission pipeline；构建 provider dependency graph，按拓扑序验证并在一个 commit receipt 中发布 metadata/factory/executor/i18n。
3. **收敛执行边界**：Operation dispatch、Native executor、event control、commandlet 和 remote/CLI 全部接收 snapshot handle 与 signed eval context；authorization、deadline、idempotency、quiescence 和 audit 在入口统一处理。
4. **收敛 UI surface**：由 snapshot 生成 effective keymap、menu 和 Palette；Palette callback 必须提交 snapshot handle，菜单显示 override 后 chord；duplicate/ambiguous/stale 都返回 typed diagnostics。
5. **完成生命周期与恢复**：serde 只保存可重建的 declarative state，恢复返回 explicit rebind plan；clone 只允许 immutable metadata snapshot；撤销/热重载与 Editor260 的 ContributionStore、toolkit、runtime consumer 使用同一 owner transaction。
6. **建立验证矩阵**：覆盖非法 chord、duplicate leaf、missing factory/executor、dependency cycle、generation rollover、stale palette/keymap/menu callback、remote denial、deadline cancellation、codec mismatch、session isolation、crash/restart rebind、100+ provider soak 和 allocation/latency budget。

# 11. Review-only 验证记录

- 已扫描本报告范围的 38 个 Rust 文件，并以当前工作树行号建立证据链。
- 已核对 Editor178、Editor260、Runtime199 和 plugin owner 的交接边界；历史 P0 没有被重复编号。
- 已执行文档静态检查：目标文档及索引 `git diff --check` 无 trailing whitespace；报告 frontmatter 中引用的本地路径均存在；索引链接无缺失；P1/P2/Gate/ID 计数无重复。
- 未运行 Cargo、编译器、插件加载、Editor UI 自动化、故障注入或性能测试，因为本轮明确是 review-only 且工作树存在未提交的跨模块修改。
- 未查询、跟踪或等待协调器状态；本报告的证据全部来自本地当前工作树和已存在的参考源码路径。

下一次实现会话应先以 ED-CMD-01、02、03、06、08、14、25 为入口建立最小可运行的统一 snapshot/admission 骨架，再处理 UI polish 和 P2 观察工具。当前状态保持 `implementation_status: pending`。
