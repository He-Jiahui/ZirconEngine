---
related_code:
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/host/plugin_host_driver.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_editor/20-ai-behavior-tree-blackboard-perception-eqs-debug-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Public/AbilitySystemComponent.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Public/Abilities/GameplayAbility.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Public/GameplayEffect.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Public/GameplayPrediction.h
  - dev/UnrealEngine/Engine/Source/Runtime/GameplayTags/Classes/GameplayTagContainer.h
  - dev/UnrealEngine/Engine/Source/Runtime/GameplayTags/Classes/GameplayTagsManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/GameplayTags/Classes/GameplayTagsSettings.h
  - dev/UnrealEngine/Engine/Source/Runtime/GameplayTags/Private/GameplayTagRedirectors.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayAbilitiesEditor.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayAbilityGraph.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayAbilityGraphSchema.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayAbilitiesBlueprintFactory.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayAbilityAudit.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayEffectDetails.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilitiesEditor/Private/GameplayEffectExecutionDefinitionDetails.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor/Source/GameplayTagsEditor/Private/SGameplayTagWidget.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor/Source/GameplayTagsEditor/Private/SAddNewGameplayTagWidget.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor/Source/GameplayTagsEditor/Private/SRenameGameplayTagDialog.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/GameplayTagsEditor/Source/GameplayTagsEditor/Private/SCleanupUnusedGameplayTagsWidget.cpp
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: false
---

# 08G · Gameplay Ability / Effect / Attribute / Tag / Cue / Prediction Runtime 工程化差距

## 1. 结论

当前 ZirconEngine 没有 Gameplay Ability System。全仓 production source 中没有 `GameplayAbility`、`GameplayEffect`、`GameplayTag`、`AbilitySystem`、`AbilitySpec`、`PredictionKey`、Attribute Set、Gameplay Cue 或等价的一等领域类型；runtime asset、scene component、plugin id 和 first-party runtime catalog 也都没有对应入口。当前唯一名为 gameplay 的运行时代码是脚本 VM 的通用宿主模块 `zr.zircon.gameplay`：它提供输入、场景切换、transform、动态component JSON、spawn/despawn、navigation，以及把 `script.bindings[*].properties.hp` 当作生命值的damage/heal函数。它是通用脚本场景操作桥，不是Ability/Effect/Tag runtime，不能用模块名称代替产品能力。

这不是纯粹的“以后再加一些节点”问题。Editor第一屏的Workbench已经把Effect设为默认选中模块，并展示 `GE_HealthRegen`、`GE_DamageFire`、`GA_DashAttack`、`DefaultGameplayTags.ini`、`Server Initiated` 和 `predicted activation` 等产品语义；但这些名称在runtime没有可保存、cook、加载、激活、复制或调试的artifact。当前UI可以声称compile、apply、playtest和prediction，runtime却没有接收命令的controller/provider，更没有权威结果。这一跨层断裂必须先硬切为显式Unavailable/Prototype，或同步建立真实runtime contract；继续增加静态反馈会扩大错误产品承诺。

通用脚本宿主本身已有可保留基础：versioned host module descriptor、typed host value kind、函数级required capability、borrowed transient string、有限JSON解析错误、SceneError到Host错误转换、world-generation-backed scene transition request，以及Navigation弱集成。问题在于 `gameplay.entity` 一个粗粒度capability同时允许任意裸u64实体的transform/component读写、spawn/despawn、damage和heal；`damage_entity`克隆完整动态JSON、修改第一个enabled binding中字符串键 `hp`，并在小于等于 `f32::EPSILON` 时直接删除实体；`heal_entity`又信任调用者给出的 `max_hp`。这些操作绕过typed attribute owner、effect execution、免疫/标签要求、死亡策略、事件、transaction、authority和generation。`builtin_host_capabilities()`虽列出全部四项gameplay能力，但当前production caller搜索没有证明它自动授予每个脚本；因此本篇不声称“所有脚本已全局越权”，而将问题准确登记为公开能力粒度和操作合同不可用于工程级gameplay安全边界。

Unreal参考实现把Ability grant/spec/activation/commit/end/cancel、Effect spec/context/capture/execution/duration/period/stacking/inhibition、Attribute aggregator、Gameplay Tag hierarchy/query/source/redirect/net index、Gameplay Cue、Prediction Window/Key和server reconciliation放在同一套可组合合同中。Zircon不应复制UObject层次或全部历史包袱，但至少必须吸收：prepared immutable definition、per-owner generational instance、typed mutation transaction、authority-scoped target、deterministic aggregation、显式latent task、可取消prediction、server receipt/reconciliation和按需debug delta。Fyrox、Bevy、Godot主仓以及Unity `dev/Graphics`没有可作为同级Ability System的first-party参考；它们的缺席不能降低用户要求的Unreal级工程基线。

本轮登记5项P0、30项P1、5项P2。P0先停止虚假产品能力、隔离通用实体写权限、禁止HP字符串旁路和补齐artifact/world/network最小闭环；P1再建立Tag、Attribute、Effect、Ability、Cue、prediction、replication、asset/scene/plugin、预算和验证全链路。完成32条门禁以前，不得把Workbench中的compile/apply/playtest/prediction反馈、现有14个gameplay host单元测试或通用宿主模块名称解释为Gameplay Ability功能已完成，更不能据此声称性能或多人表现优于Unreal。

## 2. 审查边界与证据

### 2.1 已读范围

| 范围 | 文件 / 行数 / bytes | `#[test]` | ignored | 证据等级 |
|---|---:|---:|---:|---|
| `script/vm/gameplay_host`及根模块 | 16 / 2,875 / 105,895 | 14 | 0 | E3：descriptor、callback、world/entity/component/combat/navigation/transition及全部focused tests逐文件 |
| host注册与公开导出 | 3 / 668 / 28,710 | 0 | 0 | E3：builtin registration、module export、VM export边界 |
| asset/scene absence anchors | 3 / 529 / 19,070 | 0 | 0 | E3：authoring asset和scene module完整导出表 |
| plugin assembly absence anchors | 3 / 629 / 23,948 | 8 | 1 | E3：RuntimePluginId、first-party runtime catalog和manifest依赖 |
| selected combined scope | 25 / 4,701 / 177,623 | 22 | 1 | 当前工作树fingerprint `0094f455e66b890392bba9d636899e917f6d71dfac1b7d6a2c2ecb2d20e1c141`，范围内无在途source |

行数为物理文本行；fingerprint按相对路径排序，为每个当前工作树文件计算SHA-256，再对 `path<TAB>hash<LF>` 清单计算SHA-256。唯一ignored test位于RuntimePluginId测试，理由属于既有 `PERF-MVP-43`，与Gameplay Ability功能无关；这里只作为范围库存，不作为本报告动态失败。当前仓库存在其他Session和用户修改，本轮没有修改任何production source。

全仓production名称与调用搜索覆盖 `zircon_app`、`zircon_editor`、`zircon_runtime`、全部 `zircon_plugins` 和 `tools`。精确Ability/Effect/Tag/Prediction领域名仅在Workbench生成UI、route binding和静态反馈中出现；runtime未发现对应domain owner。runtime asset导出仍是Terrain、TileMap、Prefab、Material等既有类型，scene asset导出是animation/camera/entity/lighting/mesh/physics/postprocess/script/terrain/tilemap，RuntimePluginId和first-party runtime catalog没有Gameplay/Ability provider。该负证据说明当前first-party产品闭环不存在，不排除未来第三方自行实现另一套系统。

现有14个focused test证明通用宿主在mock world中能够进行component JSON、property animation、spawn/transform、scene transition、navigation adapter和HP damage/heal；它们没有Ability grant/activate/commit/end/cancel、Effect duration/period/stack、Attribute aggregation、Tag query、Cue、prediction、replication或产品asset链断言。尤其combat tests直接构造 `script.bindings` JSON和 `CapabilitySet::default().with("gameplay.entity")`，不能证明真实项目内容、权限配置或多人authority成立。

### 2.2 参考边界

- Unreal `AbilitySystemComponent`把ability spec container、active Gameplay Effects、attributes、tag count、Gameplay Cue、prediction、replication和owner/avatar lifecycle聚合为明确的runtime owner。Zircon不必做同名组件，但必须有唯一权威owner，禁止脚本、Editor、scene动态属性和网络层各维护第二份效果/属性状态。
- Unreal `GameplayAbility`区分CanActivate、TryActivate、CommitCost、CommitCooldown、End、Cancel、NetExecutionPolicy和NetSecurityPolicy。Zircon可以采用data-oriented prepared program，但必须保留“检查、预测启动、权威提交、latent执行、终止/取消”各阶段及其失败结果，不能把一次按钮点击等价为成功激活。
- Unreal `GameplayEffect`覆盖ScalableFloat/SetByCaller、attribute capture、execution calculation、duration、period、stacking、inhibition、application/removal tag requirements和Gameplay Cue。Zircon的首版可缩小表达式面，但必须先把确定性aggregation和生命周期做对，不能继续直接写JSON `hp`。
- Unreal GameplayTags管理器维护分层tag树、source、redirect、native tag、net index和container/query语义；Editor同时提供add/rename/cleanup/query/picker/settings。Zircon必须让runtime registry/cooked dictionary成为唯一truth，Editor不能独自解析一个名为 `DefaultGameplayTags.ini` 的静态样例。
- Unreal prediction contract包含generational key、scoped prediction window、server receipt、dependent rejection/catch-up和side-effect rollback。Zircon可以采用自己的网络协议，但UI出现Predicted/Server Initiated之前，至少要有相同级别的ownership、receipt和reconciliation可证明性。
- Fyrox、Bevy、Godot主仓没有first-party Ability System，Unity本地参考仅有Graphics；本篇只把它们作为“无同域对照”的负参考，不用缺失项平均或稀释Unreal基线。

### 2.3 明确未做

- 本篇没有实现Gameplay Ability代码，没有修改脚本宿主、asset、scene、plugin或Editor，没有运行Cargo、client/server、network、performance、soak、fuzz或跨平台测试。所有结论来自current-source静态证据。
- 没有把 `builtin_host_capabilities()`清单误当成实际grant链。它目前只有公开导出和tests命中；未来实施必须继续追踪project/plugin manifest如何请求、批准、持久化和审计能力。
- 没有要求删除通用 `zr.zircon.gameplay`。目标是把它更准确地定位为scoped scene/gameplay bridge，并把typed Ability System建立在专用owner/broker上；兼容迁移不能继续把HP JSON当shipping核心。
- Gameplay Workbench的详细surface、route、transaction、compiler diagnostics和debugger问题由后续 `zircon_editor/21` 报告拥有；本篇只定义它必须消费的runtime truth。

## 3. 必须保留的现有基础

### 3.1 Host descriptor和capability检查可作为脚本适配层

模块与函数descriptor已经声明value kind、参数、返回值和required capability，调用路径能拒绝缺失能力。后续Ability脚本扩展应复用这套host registry，但能力必须细分到self/target/query/command、world和authority，且脚本只能拿到generational handle或capability-scoped facade，不能直接获得Ability owner内部容器。

### 3.2 有限JSON和错误映射意识值得保留

动态component入口检查有限数字并把SceneError映射为Host错误；borrowed string避免部分返回值复制。迁移期可保留这些防线，但Ability/Effect/Attribute/Tag热路径必须使用prepared typed data，JSON只允许authoring/import/debug边界，不能在每次damage中clone和遍历整份bindings。

### 3.3 Scene transition与Navigation已开始使用明确系统边界

scene transition request携带world generation，Navigation通过runtime bridge而非在脚本宿主内实现寻路。这说明跨系统请求可以形成typed broker。Ability task、targeting、animation、physics和audio cue都应沿用request/result/cancel/generation模式，而不是新增更多字符串字段和同步World写入。

## 4. P0 阻断项

### P0-1：Workbench已把Ability/Effect/Tags作为真实产品模块，但runtime没有任何对应domain、artifact或provider

Effect是Workbench top toolbar默认选中项，三个surface展示真实资产名、编译、应用、playtest和预测结果；runtime全仓却没有Ability/Effect/Tag类型、asset kind、manager、plugin或command endpoint。目标在实施前先硬切：未接入runtime provider时模块必须明确Unavailable且不能输出成功；接入后每条Editor命令必须返回runtime/compiler生成的operation ID、generation和diagnostic，静态样例文本不得作为成功证据。

### P0-2：UI声称 `Server Initiated` 和 `predicted activation`，runtime没有authority、prediction key、receipt、rollback或reconciliation

当前net policy和prediction只是ZUI字符串与固定feedback。没有client/server role、owner connection、activation RPC、prediction key、server acceptance/rejection、dependent side effect、attribute delta reconciliation或cue dedup。目标在网络合同完成前移除或禁用这些选择；shipping启用时必须通过丢包、乱序、重复、拒绝、late result和host migration门禁，不能用本地按钮动画模拟多人正确性。

### P0-3：`damage_entity`以字符串HP和首个enabled binding作为权威状态，并在归零时直接despawn实体

函数clone `script.bindings`，选择第一个enabled entry，读取/写入 `properties.hp`，再按 `f32::EPSILON`直接删除entity。它绕过health attribute owner、pre/post execute、resistance/immunity、shield、death state、downed/respawn、event ordering、network authority和despawn policy；binding顺序改变还会改变被写对象。目标停止将该API用于shipping gameplay，迁移为typed damage request -> authoritative effect execution -> attribute transaction -> death policy event。兼容入口只能委托新系统并返回明确结果，不能继续自行写JSON或删除entity。

### P0-4：一个 `gameplay.entity` capability允许对任意裸u64实体执行transform、component、spawn/despawn、damage/heal，缺少对象与动作级授权

函数接受调用者提供的entity ID，没有self/owner/target lease、world/entity generation、relationship、authority role、operation class或rate/bytes budget；heal还接受调用者给出的 `max_hp`。即使capability不是自动授予所有脚本，一旦manifest获得它就拿到过大的写面。目标拆成只读query、self command、validated target command、spawn lease、despawn authority和admin/tooling能力；每个handle绑定world/entity generation和owner，跨实体写入必须经过policy/broker，审计拒绝原因和来源。

### P0-5：Ability内容没有asset/cook/load/scene/world/plugin生命周期，任何实现都无法成为普通项目可启动的能力

runtime asset、scene component、RuntimePluginId和first-party catalog都无对应注册；没有项目依赖、cook产物、load generation、world owner、component activation、unload、hot reload或teardown。目标先定义最小产品闭环：Tag Dictionary、Attribute Set、Effect、Ability和Cue资产被import/cook为immutable artifact；scene/entity组件绑定Ability owner；first-party plugin按项目启用；world activation建立实例；asset/world/plugin retire通过generation fence取消任务、prediction和晚到结果。

## 5. P1 工程化差距

### P1-1：通用脚本宿主命名与Ability产品边界混淆，缺少唯一领域owner

`zr.zircon.gameplay`当前同时承载input、scene、navigation和任意entity mutation，名称容易被误认为完整gameplay framework。目标保留兼容模块但文档和capability改为明确的scene/gameplay bridge；新增唯一 `GameplayAbilityWorldRuntime`及per-entity `AbilityOwner`，Effect、Attribute、Tag、Cue和prediction状态只由它们拥有。Editor、脚本、网络和AI都通过typed command/query接口访问，不建立影子容器。

### P1-2：缺少稳定、分层、可cook的Gameplay Tag字典

当前没有tag ID、parent chain、dictionary generation、source或cooked index。目标定义规范化文本名、stable content ID、dense runtime index、parent closure和dictionary hash；import/cook拒绝非法段、重复、大小写冲突、空父级和超限深度。runtime container使用排序small-vector/bitset混合表示，禁止热路径反复字符串split/hash。

### P1-3：缺少Tag source、redirect、rename/delete和跨资产迁移合同

Editor样例显示 `DefaultGameplayTags.ini`、rename和migration preview，但runtime没有source precedence或redirect consumer。目标建立project/plugin/native/generated source、优先级、owner、只读性和provenance；rename生成versioned redirect并由cook解析所有Ability/Effect/scene引用。循环、歧义、失效终点和过期redirect必须诊断；删除前提供引用扫描和可回滚迁移记录。

### P1-4：缺少Tag Container、owned/blocked tag count和确定性增减语义

Effect和Ability常需要多个来源共同授予同一tag；简单bool无法正确移除。目标按source/spec handle维护reference count与explicit/parent aggregate count，提供Has/HasAny/HasAll/Exact和changed delta；overflow/underflow、duplicate source和owner retire必须可诊断。变更在attribute/effect transaction中按固定阶段发布，listener不能观察半提交状态。

### P1-5：缺少prepared Gameplay Tag Query和复杂条件表达

没有all/any/none嵌套query、compiled token stream、hash或版本。目标authoring AST经校验和深度/节点/string budget编译为immutable program；runtime evaluation无分配、确定性、可短路，提供required/blocked/source-aware variant。invalid generation和redirect后hash变化必须使依赖artifact重新cook。

### P1-6：缺少Attribute Set定义、stable attribute ID和per-owner存储布局

当前生命值只是任意JSON键，无法表达base/current、min/max、metadata、replication和owner。目标Attribute Set asset/schema编译stable ID、dense slot、value type、clamp、serialization/replication policy和hook binding；owner实例初始化默认值并携schema generation。属性引用不得只靠display string，reload必须有兼容迁移或显式重建策略。

### P1-7：缺少Attribute aggregator、modifier channel、source provenance和可逆计算

工程级属性不能靠每次直接覆写。目标为每个active attribute维护base、add/multiply/override等明确channel、qualifier、source spec handle和evaluation metadata；apply/remove/inhibit能重算相同结果并支持rollback。浮点顺序、NaN/Inf、clamp和overflow策略固定，所有delta包含old/new/base、cause和transaction ID。

### P1-8：缺少Attribute pre/post change、pre/post execute和跨属性约束

damage、heal、max health变化、shield和death需要所有权明确的拦截阶段。目标定义纯校验/调整hook与提交后事件，限制重入和副作用；跨属性修改进入同一command buffer或嵌套transaction并有最大深度。hook owner reload/revoke时使用generation fence，不能在持有全局锁时执行用户脚本。

### P1-9：缺少Gameplay Effect Definition、Spec、Context和稳定handle

Definition应是共享immutable asset，Spec是按等级/source/target/SetByCaller构造的实例，Context携instigator/causer/hit/ability/prediction provenance。目标分别定义asset ID、compiled generation、spec handle和active effect handle，所有entity/resource引用带world generation。禁止把整份动态JSON当Effect payload；网络、save和debug只投影明确字段。

### P1-10：缺少瞬时、无限、持续时间与周期Effect的统一生命周期

当前只有同步damage/heal，没有duration、period、start/end time、pause/inhibit和catch-up policy。目标world scheduler按确定性simulation time管理Effect；定义duration refresh、period execute-on-apply、missed tick cap、time dilation和world pause语义。remove、expire、owner retire、prediction reject必须通过同一terminal路径撤销modifier、tag、cue和granted ability。

### P1-11：缺少Effect stacking、overflow、duration/period refresh和inhibition policy

Editor样例展示stacking却无runtime含义。目标stack key可由source/target/definition/custom key决定，配置limit、overflow apply/deny/clear、duration refresh、period reset和expiration removal count。每次stack变化返回typed outcome并保持source provenance；多线程/网络重复apply必须幂等或有request ID。

### P1-12：缺少Effect application/removal requirements、immunity和query-based removal

目标在Spec构造后、提交前评估source/target tag、attribute、authority和custom predicate；被免疫、blocked、invalid target、stale generation分别报告。Effect可按tag/query/source/handle移除，但必须限定owner与最大条目，确定性排序并返回实际移除集合。免疫变化与inhibition重评估不能靠每帧扫描全部effect。

### P1-13：缺少attribute capture、snapshot/live capture和Execution Calculation

damage公式需要明确捕获source/target属性、tag和level，而不是脚本任意读写world。目标compiler预解析capture definition和dependency；Spec creation时完成snapshot capture，execute时读取live capture。Execution Calculation通过纯输入/有预算扩展接口产生modifier/output command，禁止直接despawn或修改容器，随机数来自可记录seed。

### P1-14：缺少Ability Definition、Spec、grant/revoke和stable activation identity

目标Ability asset编译instancing policy、activation group、tags、cost/cooldown、tasks和net policy；grant产生per-owner spec handle、level、input binding、source object和dynamic tags。重复grant、upgrade、remove-on-end、revoke while active都有明确语义；spec handle含generation，不能用数组index或字符串资产名作为网络身份。

### P1-15：缺少CanActivate/TryActivate/Activate/Commit/End/Cancel阶段与失败原因

当前静态Playtest消息跳过所有阶段。目标activation先检查owner/avatar、required/blocked tags、group、cooldown、cost、authority和target data，返回结构化failure tags；成功后建立activation instance。CommitCost和CommitCooldown必须原子或有明确部分失败策略，End与Cancel都清理tasks/effects/cues/prediction并发terminal event，重复终止幂等。

### P1-16：缺少cost/cooldown资产化、commit时机和server validation

Editor样例写死4秒和 `GE_DashAttack_Cost`，runtime无关联。目标cost复用Effect Spec但在commit前做preview/affordability检查；cooldown以tag/effect或专用timer形成可查询authoritative state。客户端预测消费必须可回滚，server重新校验level、owner、target和时间；禁止客户端传入任意max hp、cost delta或cooldown结束时间作为truth。

### P1-17：缺少latent Ability Task、事件等待、取消和资源清理

动画、targeting、delay、network sync和move等能力都不是一次同步函数。目标Task Broker返回generational task handle，支持Start/Poll/Event/Cancel/Finish、deadline、owner/activation/prediction generation和provider revoke。task只通过command/event通道与world交互；Ability终止、world unload、avatar swap、plugin reload和prediction reject必须取消全部子task并等待有界ack。

### P1-18：缺少Gameplay Event、Target Data和target authority validation

目标定义typed event tag、payload schema、instigator/target/context和bounded target data；target actor/entity/location/hit result都携world generation和provider provenance。server不能信任客户端任意entity ID或hit result，必须按ability policy重放/校验range、line-of-sight、team和timestamp。event订阅按owner/activation lease管理，防止结束后回调旧实例。

### P1-19：缺少Gameplay Cue的执行、去重、回收和dedicated-server边界

Effect变化需要可视/音频反馈，但Cue不能成为属性truth。目标Cue tag映射prepared notify/actor/system provider，区分Executed/Added/WhileActive/Removed，携spec/prediction key用于客户端去重。资源预取、实例池、并发/距离/质量预算、late join状态恢复和server no-render policy明确；provider缺失返回diagnostic，不能阻塞effect提交。

### P1-20：缺少Prediction Key、Scoped Window和dependent operation图

目标每个local prediction生成connection-scoped generational key，记录ability activation、cost/cooldown、attribute/effect/cue/task等dependent side effect。窗口结束后禁止无key预测；server receipt接受/拒绝/catch-up均引用key和owner generation。key wrap、reuse、disconnect、avatar change、late packet和nested window必须有测试合同。

### P1-21：缺少预测回滚与权威reconciliation，无法修复已显示的属性、Effect和Cue

目标保留足够的before state或可逆command log，在reject时按依赖逆序撤销；server correction按authoritative sequence合并，而非整容器last-writer-wins。Cue、animation和task收到reject/catch-up事件，重复网络包幂等。预测只允许声明为predictable的calculation，涉及不安全随机、隐藏server信息或不可逆spawn/despawn时必须server-only。

### P1-22：缺少Ability/Effect/Attribute的复制模型、条件、序列和带宽预算

目标定义owner-only、simulated proxy、everyone和server-only字段；使用stable dictionary/spec/effect ID和delta sequence，支持active effect add/change/remove、tag count、selected attributes和ability state。每连接有entries/bytes/time budget、priority和baseline generation；丢包、乱序、重发、late join、relevancy change和dictionary mismatch都有恢复路径，禁止每帧序列化完整JSON或完整容器。

### P1-23：缺少owner/avatar/connection/world/entity generation和权威迁移生命周期

Ability owner与avatar可能不同，possession、respawn、seamless travel、world replacement和entity ID复用都需要 fence。目标 `AbilityOwnerKey`绑定world replacement epoch、entity generation和可选connection generation；avatar变更触发ability policy、task取消/迁移、cue和target刷新。world retire先停admission，再取消activation/effect/task/prediction，最后drop state并拒绝late result。

### P1-24：缺少asset importer/cook依赖图、版本兼容和last-known-good发布

Tag、Attribute、Effect、Ability、Cue互相引用，不能在运行时用字符串临时解析。目标import生成source diagnostics，cook构建dependency closure、stable hash、size/count budget和target-specific artifact；prepare完成全部引用解析和schema compatibility，atomic publish后才替换generation。失败保留LKG并标记stale，redirect/schema/extension变化精确使依赖重cook。

### P1-25：缺少scene component和runtime system调度阶段，无法自动随entity激活

目标定义最小Ability System component：owner config、attribute sets、startup abilities/effects/tags、replication/prediction profile。component add/change/remove与scene activate/deactivate增量驱动runtime，不允许每帧全scene扫描。system阶段固定为ingest network/input -> activate/commit -> effect periodic/aggregation -> publish events/replication/debug，跨系统command在边界批量提交。

### P1-26：缺少确定性排序、simulation clock、随机种子和并发提交模型

同帧多个damage/heal/effect/activation的结果必须独立于HashMap顺序和worker完成顺序。目标所有command带world tick、producer order/sequence、owner generation和transaction ID；按文档化规则排序并在owner lane单写提交。duration/period/cooldown使用simulation time，浮点与random policy可记录；并行阶段只读immutable artifact和snapshot，不直接改共享容器。

### P1-27：缺少save/load、replay、hot join和schema迁移合同

目标区分可保存的base attribute、persistent effect、granted ability/cooldown与瞬时task/prediction/cue；序列化asset ID、compiled generation和剩余simulation time，不保存裸指针/slot。load先迁移dictionary/schema，再恢复owner并重建aggregator；不兼容项产生可审计drop/fail policy。replay记录输入、activation、authoritative transaction和seed，支持定位分歧。

### P1-28：缺少third-party extension、owner revoke和热重载安全边界

calculation、target provider、task、cue和custom condition都需要plugin扩展，但不能给扩展任意world mutation。目标每类extension有typed descriptor/schema、thread affinity、determinism、budget、owner generation和command sink；prepare artifact记录依赖closure。revoke先停新调用、等待in-flight、取消active task/prediction、迁移或retire依赖artifact，late callback被generation拒绝。

### P1-29：缺少按需诊断、trace、审计和隐私/安全日志

目标runtime生成bounded结构化事件：grant/revoke、activation attempt/failure/commit/end、effect apply/inhibit/stack/period/remove、attribute delta、tag delta、cue、prediction receipt/reject/reconcile和budget overflow。无reader时成本接近零；reader按owner/tag/ability filter取得delta，full snapshot异步分片且有entries/bytes/age预算。security rejection和manifest capability使用单独审计通道，避免日志泄露任意payload。

### P1-30：测试只覆盖通用host fixture，缺少产品、网络、故障、规模和性能资格

目标测试分层覆盖tag/query property tests、aggregator golden、effect/ability state machine、asset round-trip/cook determinism、world lifecycle、plugin reload、script capability denial、client/server prediction、packet chaos、late join、save/replay和Editor-provider integration。规模曲线至少覆盖大量owner、active effect、periodic effect、tag query和并发activation；记录CPU、allocation、bytes、oldest latency和rollback cost。没有Windows client/server与目标平台证据前不得声明优于Unreal。

## 6. P2 扩展能力

### P2-1：缺少面向大型项目的Gameplay Effect表达式、曲线和数据注册表生态

在P1确定性Spec/aggregator完成后，增加level curve、data registry、caller magnitude、conditional modifier、linked/overflow effect、custom calculation library和离线常量折叠。表达式必须prepared、版本化、有复杂度预算和可解释trace，不能把任意脚本eval塞回热路径。

### P2-2：缺少高阶Ability编排、组合、输入触发和可视化状态调试

建立ability set、activation group、combo/window、input trigger、chord、hold/release、cancel relationship和可组合task graph；所有高级语义落到同一activation/prediction合同。Editor timeline/graph只投影runtime trace，不维持独立模拟器。

### P2-3：缺少Mass/ECS批处理与大规模Effect/Tag/Attribute存储策略

大量单位不能为每个owner分配同构HashMap和独立scheduler对象。基于compiled layout建立chunk/dense storage、shared definition、timing wheel、batched aggregation和significance/replication LOD；冷热owner迁移必须保持handle generation和确定性结果。

### P2-4：缺少跨服务器authority transfer、回放取证和反作弊策略

在单server预测正确后，增加shard/host migration、handoff snapshot、prediction drain、server sequence continuation和安全策略。客户端target/cost/timestamp输入需风险分级、限频、签名/nonce或等价防重放机制，并提供不泄露敏感状态的取证trace。

### P2-5：缺少离线平衡分析、仿真、审计和内容质量门禁

建立headless corpus simulation、ability/effect dependency graph、tag/attribute coverage、不可达activation、无限stack/周期、循环grant、数值爆炸、网络带宽和rollback成本分析。结果绑定artifact hash并可由CI/Editor消费，不能继续使用固定 `+50 health` 文案作为平衡或执行证据。

## 7. 目标架构

### 7.1 Content与prepared artifact层

`GameplayTagDictionaryAsset`、`AttributeSetAsset`、`GameplayEffectAsset`、`GameplayAbilityAsset`和`GameplayCueAsset`由统一compiler产生immutable、versioned、bounded artifact。所有名称引用在cook时解析为stable ID/dense index，artifact记录dictionary/schema/extension依赖和compat hash。Editor只编辑authoring DTO并消费同一compiler diagnostics。

### 7.2 World runtime与owner层

每个world replacement epoch拥有独立 `GameplayAbilityWorldRuntime`，内部按generational `AbilityOwnerKey`分片。owner持Attribute storage/aggregator、Tag counts、Ability specs/activations、Active Effects、Cue state和prediction ledger；单写commit lane保证确定性，worker只处理immutable input并返回commands。

### 7.3 Command、transaction和broker层

外部系统只能提交typed grant/activate/apply/remove/attribute/tag/event command。一次commit产生transaction ID、typed outcome和ordered deltas；Animation/Navigation/Physics/Audio/Script通过generational broker task集成。通用脚本宿主获得self/validated-target facade，不直接操作owner内部或随意despawn。

### 7.4 Authority、prediction和replication层

authority policy决定client是否可预测、server是否重新验证以及哪些side effect可回滚。Prediction Ledger把key、activation、transaction、cue/task依赖关联；Replication按connection baseline和budget发送dictionary-negotiated delta。receipt/reject/correction统一进入reconciliation阶段，并在owner/world generation变化时拒绝晚到结果。

### 7.5 Observability与Editor层

runtime提供versioned catalog、compiler diagnostics、operation API、bounded delta stream和explicit snapshot lease。Editor Gameplay surface、bottom panel、diff、simulate和debugger都消费这些真实接口；未安装provider、artifact stale、runtime offline或权限拒绝必须成为显式状态，绝不回退到静态成功文案。

## 8. 分层实施顺序

### M0：能力真相与危险入口止血

将Workbench Gameplay模块标记Unavailable/Prototype直到provider存在；冻结新的HP JSON caller；把damage/heal/despawn迁移目标和capability拆分写入兼容政策。建立跨层能力truth table和现有调用审计。

### M1：Tag与Attribute基础

实现Tag dictionary/source/redirect/container/query artifact，以及Attribute Set/dense storage/aggregator/transaction。先完成确定性、migration和property/golden tests，再允许Effect依赖。

### M2：Effect runtime

实现Definition/Spec/Context、duration/period、modifier/execution、stack/inhibition/immunity和active handle；接入world scheduler、save和debug delta。用新Effect替代host damage/heal直接写入。

### M3：Ability runtime与任务

实现Definition/Spec、grant/revoke、activation phases、cost/cooldown、event/target data和latent task broker。scene component和first-party plugin建立普通项目启动闭环。

### M4：Cue与跨系统集成

建立Cue生命周期、资源预算和Animation/Audio/VFX适配；Navigation/Physics/Script targeting与task全部使用generational broker。provider缺失和reload有明确结果。

### M5：网络authority与prediction

实现Prediction Key/Window、server request/receipt、rollback/reconciliation和replication delta；通过packet chaos、late join、respawn、disconnect和world replace测试以后才启用Editor中的Predicted/Server Initiated选项。

### M6：资产、cook、reload与产品Editor

完成全部asset importer/cook/dependency/LKG、Editor controller/transaction/compiler/diff/simulation/debug stream。硬删除静态样例成功路径并做catalog/registration parity检查。

### M7：规模、故障与性能资格

覆盖大量owner/effect/query、periodic scheduler、network bytes、consumer stall、plugin reload、save/replay、soak/fuzz和跨平台。输出与Unreal相同场景、相同内容、相同质量和相同硬件的可复现实验，不以微基准外推整机优越性。

### M8：P2高级能力

在M0-M7稳定后再进入复杂表达式、组合Ability、Mass批处理、跨服务器迁移和离线平衡分析，避免高级UI建立在错误runtime上。

## 9. 验收门禁

- **G-01 能力真相**：provider缺失时Editor与runtime返回Unavailable；搜索不到固定compile/apply/playtest/prediction成功路径。
- **G-02 名称边界**：文档、manifest和反射清楚区分通用脚本scene bridge与Gameplay Ability System。
- **G-03 Capability隔离**：self query、validated target command、spawn/despawn authority分离；缺权、跨world、stale entity全部拒绝并审计。
- **G-04 HP旁路清零**：production caller不再直接读写 `script.bindings.*.properties.hp`；damage/heal经typed Effect/Attribute transaction。
- **G-05 Tag字典**：source/parent/redirect/cook/dense index稳定，非法、循环、冲突和dictionary mismatch测试通过。
- **G-06 Tag容器与查询**：reference count、parent aggregate、Any/All/None/Exact和prepared query property tests通过。
- **G-07 Attribute schema**：stable ID、dense layout、default/clamp/generation和reload migration golden通过。
- **G-08 Aggregator确定性**：不同插入/worker完成顺序产生相同base/current/delta，apply/remove/rollback可逆。
- **G-09 Effect Spec**：Definition/Spec/Context/handle分层明确，所有引用带generation且bounded serialization通过。
- **G-10 Effect时间**：instant/infinite/duration/period、pause、time dilation、missed tick和expire在固定时钟测试中正确。
- **G-11 Stacking/Inhibition**：stack key/limit/overflow/refresh/reset/inhibit/remove矩阵和幂等请求测试通过。
- **G-12 Execution安全**：capture/execution只能产生bounded command，NaN/Inf、超预算、重入和provider revoke显式失败。
- **G-13 Ability Spec**：grant/upgrade/duplicate/revoke/remove-on-end和stale handle测试通过。
- **G-14 Activation状态机**：CanActivate/Try/Commit/End/Cancel每个失败分支返回结构化原因，重复terminal幂等。
- **G-15 Cost/Cooldown**：预测与权威commit、部分失败、rollback、server revalidation和时间恢复测试通过。
- **G-16 Latent Task**：Start/Event/Poll/Cancel/Finish、deadline、owner retire和late callback generation fence通过。
- **G-17 Target/Event**：跨world、伪造entity/hit、越距、遮挡、team和stale target均被policy拒绝。
- **G-18 Cue生命周期**：Executed/Added/WhileActive/Removed、预测去重、池回收、provider缺失和server no-render通过。
- **G-19 Prediction Key**：nested window、wrap/reuse、dependent operation、receipt/reject/catch-up和disconnect测试通过。
- **G-20 Reconciliation**：属性、Effect、Tag、Cue和Task在丢包、乱序、重复、晚到和拒绝后收敛到server truth。
- **G-21 Replication预算**：owner/proxy条件正确，每连接entries/bytes/time/oldest-age受限且late join可恢复。
- **G-22 Asset闭环**：Tag/Attribute/Effect/Ability/Cue从authoring到import/cook/load/activate/unload完成，无运行时字符串补解析。
- **G-23 Scene闭环**：普通项目仅靠scene component和plugin依赖即可创建、替换和销毁Ability owner。
- **G-24 World生命周期**：world replace、entity despawn、avatar swap、project close先停admission、取消任务并拒绝晚到结果。
- **G-25 Plugin热重载**：extension dependency closure、in-flight drain、active task迁移/取消、LKG和rollback通过。
- **G-26 Save/Replay**：schema migration、persistent effect/cooldown恢复、seed和authoritative transaction replay无分歧。
- **G-27 Debug按需**：无reader时接近零额外分配；delta与full snapshot受entries/bytes/time/age预算且slow consumer不反压runtime。
- **G-28 Editor真实链**：Save/Compile/Diff/Apply/Playtest/Tags migration都返回真实operation/generation/diagnostic，无样例字符串冒充结果。
- **G-29 规模曲线**：固定内容下记录owner/effect/period/query增长的CPU、allocation、memory和latency曲线，无隐藏全量扫描/clone。
- **G-30 Network chaos**：目标丢包、乱序、重复、延迟、带宽和连接重建矩阵连续通过并报告收敛时间。
- **G-31 跨平台产品证据**：Windows client/dedicated server及目标平台完成asset、scene、network、reload、save和Editor端到端验证。
- **G-32 性能声明**：只有相同硬件、内容、质量、网络和采样窗口的可复现对照达到阈值后，才允许声称优于Unreal。

## 10. 硬切与兼容政策

- 不保留“静态Workbench成功反馈 -> 未来runtime也许实现”的双轨；provider不存在就Unavailable，provider存在就只接受真实operation outcome。
- 不保留以 `script.bindings.properties.hp` 为权威生命值的shipping路径。短期兼容API必须委托typed Effect并发deprecation diagnostic，完成调用迁移后删除旧实现。
- 不保留单一 `gameplay.entity` 作为任意实体写权限。旧manifest需要显式迁移到细粒度能力，无法证明owner/authority的调用默认拒绝。
- 不建立Editor专属Tag/Effect/Ability模型或模拟器。Editor、cook、runtime、network和debug必须共享同一schema、artifact ID和diagnostic code。
- 不以Fyrox/Bevy/Godot/Unity Graphics缺少同域实现为降级理由；本域参考优先采用Unreal本地源码并结合Zircon既有typed/generational/budget原则重新设计。

## 11. 当前验证结论

本轮只完成静态review。没有运行Cargo或动态产品测试，原因不是将现有代码视为通过，而是本篇没有production实现变更，且当前会话先前的 `zircon_editor --lib` 编译验证已经暴露239个既有错误、122个warning；重复同一未变化lane不能增加Gameplay runtime证据。文档中的22个test attribute只是库存，唯一ignored test也不属于本域。

实施前必须重新计算selected scope fingerprint并检查相关source是否被其他Session修改；若发生漂移，应重新核对对应差距、行数和调用搜索。任何后续绿色测试都必须绑定current source或Build Set，不能用本篇静态结论、旧output record或Workbench截图代替runtime资格。
