---
related_code:
  - .gitignore
  - examples/vampire
  - examples/vampire/README.md
  - examples/vampire/LICENSES.md
  - examples/vampire/zircon-project.toml
  - examples/vampire/assets/animation
  - examples/vampire/assets/data/balance.toml
  - examples/vampire/assets/data/enemy_behavior_tree.toml
  - examples/vampire/assets/materials
  - examples/vampire/assets/models
  - examples/vampire/assets/navigation/main.navmesh.toml
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/assets/shaders
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml
  - examples/vampire/assets/textures/jungle_ground_albedo.png
  - examples/vampire/screenshots
  - examples/vampire/scripts/vampire_game/main.zr
  - examples/vampire/scripts/vampire_game/plugin.toml
  - examples/vampire/scripts/vampire_game/plugin.zrp
  - examples/vampire/scripts/vampire_game/bin/.zr_cli_manifest
  - examples/vampire/scripts/vampire_game/bin/main.zro
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/dynamic_api/session
tests:
  - zircon_runtime/src/asset/tests/project/example_vampire.rs
  - zircon_runtime/src/asset/tests/project/example_vampire/manifest_scene_imports.rs
  - zircon_runtime/src/asset/tests/project/example_vampire/third_person_render_extract.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_gameplay.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_hud.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_menu.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_runtime_support.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_app/02-pbr-viewer-tool-runtime-evidence-renderdoc-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08g-gameplay-ability-effect-attribute-tag-cue-prediction-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/zircon_plugins/08/failure-2026-08-01-zrvm-vampire-behavior-test-ownership-gap.md
reference_engines:
  - dev/UnrealEngine/Samples/Games/Lyra/Lyra.uproject
  - dev/UnrealEngine/Samples/Games/Lyra/Source/LyraGame.Target.cs
  - dev/UnrealEngine/Samples/Games/Lyra/Source/LyraGame/GameModes/LyraExperienceDefinition.h
  - dev/UnrealEngine/Samples/Games/Lyra/Source/LyraGame/GameModes/LyraExperienceManagerComponent.cpp
  - dev/UnrealEngine/Samples/Games/Lyra/Plugins/GameFeatures/TopDownArena/TopDownArena.uplugin
  - dev/bevy/examples/2d/2d_viewport_to_world.rs
  - dev/bevy/examples/state/states.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/Fyrox/template-core/src/lib.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/godot/main/main.cpp
  - dev/godot/editor/export/editor_export_platform.cpp
  - dev/godot/editor/export/editor_export_platform_pc.cpp
  - dev/Graphics/Tests/SRPTests/Packages/com.unity.testing.hdrp/TestRunner/HDRP_GraphicTestRunner.cs
  - dev/Graphics/Tests/SRPTests/Packages/com.unity.testing.urp/Scripts/Runtime/UniversalGraphicsTests.cs
  - dev/Graphics/Tests/SRPTests/Packages/com.unity.testing.urp/Scripts/Runtime/UniversalGraphicsTestSettings.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 06 · Vampire Roguelite 样例项目、资产、脚本、玩法与证据产品工程化差距

## 1. 结论

`examples/vampire`不是一个只有占位README的空样例。它包含110个scene entity、90个mesh component、24个有效GLB、52份可解析且UUID唯一的`.zmeta`、一份ZrVM脚本、导航/地形/材质/着色器资产、Start/Retry菜单、玩家移动、自动攻击、三名活动敌人、导航追逐、world HUD和多批历史截图。当前局部基础足以保留为一个引擎产品回归场景。

但仓库中的Vampire不能从clean clone还原。根`.gitignore`以`examples/vampire/*`忽略整个样例；虽然173个文件被force-add，场景直接引用的4份生成模型TOML以及另3份同类模型源只存在于当前工作区的ignored/untracked状态。主import test又先`assert!(exists)`这些文件。52份`.zmeta`指向的本地`.zircon/cache`产物也全部被忽略，仓库既没有完整source closure，也没有可安装cooked artifact closure。当前机器“108个`res://`引用都存在”不能转化为clean clone产品资格。

玩法层更接近scripted smoke scene，而不是README声明的10分钟roguelite slice。唯一192行脚本把全部玩法塞进`onUpdate`，`onFixedUpdate`为空；玩家每帧固定平移1单位且不使用`dt`或`move_speed`，自动攻击每帧造成1点伤害，敌人接触每帧造成0.2点伤害。`balance.toml`的伤害、冷却、无敌帧、XP、等级、波次、最大敌人数、宝箱、升级、Boss和胜利时间都没有consumer；`enemy_behavior_tree.toml`也没有运行时consumer，脚本只是手写几个branch code。场景没有Boss、spawn/progression/win/save恢复、碰撞或物理，6个敌人binding仅3个启用。README中“Boss loop”“enemy waves”“progression”和“accepted slice”因此是设计意图，不是当前产品事实。

脚本工件也不能作为可发布包。tracked `.zr_cli_manifest`记录当前开发机的绝对`E:\Git\ZirconEngine\...`路径，并引用不存在的`main.zri`与AOT C文件；tracked `main.zro`没有source/toolchain/ABI/BuildSet/target receipt。项目要求Rendering、glTF、Navigation和ZrVM，但README必须手工追加五组Cargo feature才能运行；Plugins06已证明标准Client profile/catalog不能闭合required provider。Vampire的asset test又调用`register_first_wave_plugin_fixture_importers_for_test()`，所以“测试可导入”并不证明标准产品host能materialize相同provider。

验收证据已经发生明显代际漂移。10个real-ZrVM gameplay/menu/HUD/performance test全部被`#[ignore = "real ZrVM coverage moved to the zr_vm_language plugin owner"]`跳过，而目标plugin树没有接手Vampire行为测试。主import test对当前27行`default_pbr.wgsl`断言20个已经不存在的Vampire/material/lighting marker，源码本身已静态不一致。README声明三张latest accepted图，但仓库只有`start-menu-640`，缺少`ground-fixed-640`与`game-over-640`；54个evidence文件没有结构化sidecar，18个日志为空，截图跨多个UI/场景/分辨率代际。README还把2026-06-11的单帧60.87 FPS诊断称为latest，而Runtime07在2026-07-12记录同116 mesh draws只有30.89/33.98 FPS。当前不能对Vampire声明clean-clone、playable loop、visual parity、performance或release readiness。

本篇只拥有Vampire产品样例的repository closure、project composition、游戏规则与数据真值、脚本/资产产品工件、样例级测试和证据资格。Runtime04继续拥有通用资产/meta/cache/import合同，Runtime07拥有通用ZrVM生命周期，Runtime08F/08G拥有AI与Gameplay系统，Plugins06拥有通用provider catalog/profile，Tooling03/07/10拥有通用打包、性能证据和测试基础设施。本轮登记 **5项P0、80项P1和16项P2**。

## 2. 审查边界与物理清单

### 2.1 仓库内容

| 子域 | 物理规模 | 结论 |
|---|---:|---|
| tracked sample | 173文件 / 8,880,831 bytes / 约11,458文本行 | 内容量真实，但根ignore规则与force-add形成不可维护发布模型 |
| metadata | 52 `.zmeta` / 7,359行 | TOML可解析、root UUID唯一、format 7、preview ready；不能替代缺失source/artifact |
| scene | 110 entity / 110唯一ID | parent无缺失/自环；90 mesh、14 point light、9 script binding、1 camera |
| active gameplay | player + 3 enemy | 另3个敌人binding disabled；没有Boss或spawn owner |
| GLB corpus | 24 GLB / 1,399,492 bytes | glTF 2 header/chunk有效；42 mesh primitive、128 animation、0 skin |
| ignored local state | 578文件 / 1,727,504 bytes | cache 563、registry 1、models 14；clean clone不可依赖 |
| screenshots/logs | 54 tracked evidence文件 | 34 PNG、20文本；18文本0 byte，23个唯一PNG hash，3组重复 |
| script package | 1 source / 192行 + tracked `main.zro` | manifest使用绝对路径并引用两个不存在产物 |

### 2.2 Clean-clone closure

场景共有108个唯一`res://`引用，在当前工作区都能解析；其中4个直接指向被ignore且未tracked的模型源：`jungle_terrain.model.toml`、`grass_billboard_static_batch.model.toml`、`jungle_broadleaf.model.toml`和`jungle_fern_cluster.model.toml`。同目录还存在3份未tracked的arena/player/enemy模型TOML及7份对应`.zmeta`。这些TOML直接存储vertices/indices，不是能由tracked生成器和recipe确定重建的临时中间物。

主import test同样硬编码检查前4个文件存在。当前`.zircon/cache`恰好含52个artifact，只证明本机历史导入曾经完成；cache、registry和缺失source都不在tracked closure内。资格测试必须从临时clean checkout开始，不能在开发者已有cache和ignored文件的仓库中运行。

### 2.3 Scene、资产与玩法可达性

scene entity ID和name均唯一，parent关系无直接结构错误。组件分布为90 mesh、14 point light、9 script binding、1 camera、1 animation skeleton、1 animation state machine、1 terrain、1 ambient light、1 directional light、1 post-process volume。文本中没有collision、rigid body、collider、audio、sound或listener组件；导航资产没有作为scene resource引用，依赖host侧另行定位。

9个script binding中只有player、skeleton、zombie和ghost启用。camera/player aura与3个重复敌人binding disabled；scene没有`role = "boss"`实体。只有player拥有animation state machine；enemy脚本虽调用animation bool，但scene没有给enemy配置对应state-machine组件。GLB共声明128个animation、0个skin，README也承认clip import仍是placeholder，所以不能把动画文件数量计为骨骼动画可用性。

### 2.4 测试与证据可达性

asset import/render-extract测试只在graphics/script feature组合下进入，并注册test-only first-wave importer fixture。动态session侧有4个gameplay、3个HUD、2个menu和1个frame diagnostic等10个real-VM测试，全部ignore；唯一未ignore的玩法检查只匹配WASD源码字符串。`zircon_plugins/zr_vm_language`没有Vampire项目行为consumer，已有failure handoff也明确记录ownership cutover未完成。

README列出的三张latest accepted图片只有`vampire-runtime-start-menu-640.png`存在。34张PNG中10张`current-04..13`完全相同，`current-01/02`相同，`current-00/current`相同；没有任一JSON/TOML/YAML/CSV sidecar绑定source、binary、BuildSet、GPU、resolution、frame、input、threshold或结果。截图既不能自动判定，也不能证明来自当前source generation。

## 3. 参考引擎约束

- Unreal Lyra用`.uproject`声明modules/plugins，用独立Game/Client/Server/Editor Target表达产品角色，并通过Experience definition/manager和Game Feature plugin把可用玩法、加载、activation与deactivation组成可观察生命周期。Zircon无需复制Lyra类层次，但样例required provider、玩法feature、entry scene和artifact target必须形成同一个可验证产品闭包。
- Bevy把连续运动放入`FixedUpdate`并从`Time<Fixed>`读取delta；状态示例把进入、退出和更新挂到显式schedule。Vampire不能继续用render/update调用次数定义速度、DPS、接触伤害和冷却，也不能用几个自由字符串代替typed run state与transition。
- Fyrox game template通过`Plugin::register/init/on_loaded/update/on_os_event`和Script lifecycle形成明确composition root，Executor负责持续loop与资源/插件生命周期。Vampire项目不能把一份脚本文件、手工feature命令和本地cache拼成隐式产品入口。
- Godot `Main`从project settings/main scene启动，export platform枚举project files并通过`save_pack`产生目标包；PC export把project data与binary组合并返回Error。样例资格必须来自一次clean cook/pack/install/run，而不是源码树中恰好存在的import cache和绝对路径编译清单。
- Unity Graphics的HDRP/URP test runner加载明确scene，等待受控frame，固定capture条件，以`ImageAssert.AreEqual`和平台阈值比较reference，并可检查render allocation。Vampire截图文件名和人工说明不能替代reference identity、capture recipe、threshold、machine-readable result与failure artifact。

## 4. 可保留的正确基础

1. 项目manifest已经显式声明default scene、script package和七个plugin selection。
2. scene的entity ID/name/parent结构在当前文件中自洽，组件数量足以作为综合回归场景。
3. 52份tracked `.zmeta`均可解析且UUID唯一，asset kind覆盖模型、材质、动画、shader、terrain、navmesh与scene。
4. 24份GLB具有有效glTF 2容器和完整chunk length，第三方license文件已入库。
5. Start/Game Over/Retry和player/enemy分支为未来typed game flow提供了最小行为骨架。
6. Navigation chase使用`dt`，说明脚本host已有时间参数和导航调用，不需要另造样例专用移动API。
7. 当前scene明确关闭重复enemy binding以控制real-VM回调预算；该预算意识应保留，但要进入profile/receipt而不是README口述。
8. 已有asset import、render extract和real-VM测试源码可迁移成产品测试；问题是fixture、ignore和owner，不是从零创建场景。

## 5. P0：产品资格硬阻断

### APP-VAMPIRE-P0-001 · Clean clone缺失场景入口可达的required模型源

根ignore规则覆盖整个样例，7份项目自有`.model.toml`和对应`.zmeta`仅存在于ignored local state。场景直接引用其中4份，主import test也要求它们存在；tracked cache又不提供可安装替代物。当前clone无法证明默认scene能导入，任何基于现有工作区的绿色结果都会被本地残留污染。

建立`VampireSourceManifest`和clean-checkout closure gate。项目自有source、生成recipe与license必须tracked；若模型是generated artifact，则tracked recipe/tool digest必须能在空cache确定重建，并把artifact receipt纳入cook。禁止以force-add局部文件维持一个整体被ignore的样例目录。

### APP-VAMPIRE-P0-002 · Required provider只在手工feature与test fixture中闭合

项目required Rendering/glTF/Navigation/ZrVM，但标准`target-client`不能保证catalog包含这些provider；README要求开发者手工追加五组feature。asset测试通过`register_first_wave_plugin_fixture_importers_for_test()`注入importer，绕过产品source catalog。测试可导入与用户命令可启动不是同一BuildSet，无法证明样例在支持矩阵内。

定义versioned `VampireProductProfile`，由product composer把manifest selection解析成compiled provider set、packaging mode与startup receipt。相同profile必须驱动build、test、cook、packaged run；test-only importer只能用于importer单测，不能作为产品资格证据。通用catalog修复仍归Plugins06，本篇验收其产品消费结果。

### APP-VAMPIRE-P0-003 · README承诺的roguelite规则没有运行时authority

`balance.toml`和behavior tree没有consumer，脚本在`onUpdate`按帧平移与伤害；没有cooldown、invulnerability、XP、level、wave、spawn、upgrade、chest、boss entity、win、durable save或完整retry。帧率改变会直接改变移动速度、DPS和contact damage，所谓10分钟目标甚至没有authoritative clock。当前行为既不稳定，也与公开玩法说明不一致。

建立`VampireRunState`、`VampireSimulationConfig`、fixed simulation schedule和typed systems：输入采样、movement/collision、targeting/cooldown/projectile、damage/invulnerability、AI、spawn/wave、XP/upgrade、boss/win、save/restore分别拥有状态与事件。`balance.toml`必须经schema加载成为唯一配置权威，未实现字段要删除或显式标为future，不能继续宣称可用。

### APP-VAMPIRE-P0-004 · Tracked脚本工件不可迁移、不可重建、不可发布

`.zr_cli_manifest`写入开发机绝对路径，并引用不存在的`main.zri`和`bin/aot_c/src/main.c`。`main.zro`虽tracked，却没有编译器、source hash算法合同、ABI、imports closure、target、profile、BuildSet或签名receipt。另一个checkout无法验证它与`main.zr`一致，也无法判断应加载解释器、binary还是AOT形态。

将source包与derived artifact分离。clean build输出content-addressed Zr package，manifest只存包内相对路径和versioned schema；receipt绑定source tree、compiler/toolchain、host ABI、imports、capabilities、target/profile、artifact digest与reproducibility。发布测试从installed package加载，源码树不保留机器绝对路径的CLI中间清单。

### APP-VAMPIRE-P0-005 · 测试、视觉和性能证据不能证明当前产品

10个real-VM测试全部ignore且新owner无等价测试；主import test对当前WGSL断言20个不存在marker；README缺两张宣称accepted图片；已有截图无sidecar且跨代重复；performance从单帧60.87 FPS口径漂移到后续30.89/33.98 FPS而README仍称latest；README还记录1280x720一tick access violation。没有一条source-bound lane同时证明build、start、play、render、performance和clean exit。

建立`VampireEvidenceSet`：clean checkout build/cook/install后运行确定性input trace，验证menu、movement、combat、death/retry和完整run；每个artifact绑定commit/source tree、BuildSet、binary/plugin/script/asset digest、GPU/driver、resolution、frame/input、threshold、metrics和terminal status。ignored owner迁移必须原子完成；不存在或失败的图片不能标accepted。

## 6. P1：Repository、Project 与 Source Closure

| ID | 当前差距 | 需要重构 |
|---|---|---|
| APP-VAMPIRE-P1-001 | `.gitignore`整体忽略`examples/vampire/*` | 使用精确derived-output规则；sample source默认tracked |
| APP-VAMPIRE-P1-002 | force-added文件与普通新增文件行为不同 | pre-commit/CI检查sample下required source不得ignored |
| APP-VAMPIRE-P1-003 | 无versioned source inventory | manifest列出entry scene、script、asset roots、licenses与required生成recipe |
| APP-VAMPIRE-P1-004 | 7份模型源没有tracked provenance | 每份generated model记录generator、inputs、parameters、tool digest和license |
| APP-VAMPIRE-P1-005 | `.zmeta` preview ready未绑定可取artifact | meta readiness与source/artifact digest、platform variant及store receipt绑定 |
| APP-VAMPIRE-P1-006 | cache/registry残留可污染开发验证 | qualification总在空cache、隔离registry和临时install root运行 |
| APP-VAMPIRE-P1-007 | 108个resource引用只做本机存在性观察 | build graph递归解析并给出missing/ignored/cycle/orphan report |
| APP-VAMPIRE-P1-008 | license只覆盖Kenney说明文件 | source inventory逐asset记录license、origin、modification和redistribution资格 |
| APP-VAMPIRE-P1-009 | README是产品状态唯一总表 | 机器可读capability/evidence receipt生成简短文档状态，禁止手工漂移 |
| APP-VAMPIRE-P1-010 | 无sample schema/version migration lane | project/scene/meta/data/script package升级均需old fixture与migration receipt |

## 7. P1：Script Package、Build 与 Artifact

| ID | 当前差距 | 需要重构 |
|---|---|---|
| APP-VAMPIRE-P1-011 | `.zr_cli_manifest`存绝对workspace路径 | package manifest只允许规范相对路径或content URI |
| APP-VAMPIRE-P1-012 | `main.zri`登记但不存在 | 每种execution mode声明required artifact，missing为build failure |
| APP-VAMPIRE-P1-013 | AOT C路径登记但目录不存在 | 不生成的variant不得进入manifest；生成则纳入包与receipt |
| APP-VAMPIRE-P1-014 | tracked `main.zro`无compiler identity | receipt绑定compiler version/config/target/ABI/source/import digests |
| APP-VAMPIRE-P1-015 | source与binary freshness不可验证 | build/load前验证source graph fingerprint，不接受未知代际artifact |
| APP-VAMPIRE-P1-016 | imports只列字符串 | resolve为qualified module ID、version、capability和artifact digest |
| APP-VAMPIRE-P1-017 | capability表不含实际host calls完整集合 | 编译器生成capability use set，与declared grant做双向校验 |
| APP-VAMPIRE-P1-018 | interp mode与binary/AOT残留混在同目录 | variant使用独立target/profile目录和统一package index |
| APP-VAMPIRE-P1-019 | `saveState`返回常量且restore丢弃输入 | 定义versioned save schema、migration、validation和round-trip behavior |
| APP-VAMPIRE-P1-020 | activate/deactivate无资源清理合同 | package lifecycle拥有subscription/entity/UI/particle/nav handle并验证幂等释放 |

## 8. P1：Simulation、State 与 Gameplay Loop

| ID | 当前差距 | 需要重构 |
|---|---|---|
| APP-VAMPIRE-P1-021 | `onFixedUpdate`为空 | authoritative gameplay进入fixed schedule，presentation留在variable update |
| APP-VAMPIRE-P1-022 | 玩家每update固定平移1单位 | 归一化input vector并使用speed、fixed dt和collision result |
| APP-VAMPIRE-P1-023 | 对角线速度高于轴向 | input magnitude clamp/normalize并测试8方向等速 |
| APP-VAMPIRE-P1-024 | scene `move_speed=5.2`未消费 | typed player config成为唯一速度来源并在加载时验证范围 |
| APP-VAMPIRE-P1-025 | 自动攻击每update伤害1 | cooldown/projectile/damage通过simulation clock和event pipeline驱动 |
| APP-VAMPIRE-P1-026 | 无目标仍永久attacking并发particle | action state由target/ability phase/outcome决定并在结束时清理 |
| APP-VAMPIRE-P1-027 | contact damage每update 0.2 | attack cadence、hit event、invulnerability与damage receipt显式化 |
| APP-VAMPIRE-P1-028 | Start/Retry用自由字符串component JSON | typed run state与validated command/event transition |
| APP-VAMPIRE-P1-029 | Retry只复位player | checkpoint重建spawn、enemy、projectile、buff、RNG、timer与UI全状态 |
| APP-VAMPIRE-P1-030 | 没有pause/focus/time-scale policy | menu、window focus、capture、debug pause与simulation clock有明确合同 |

## 9. P1：Data、AI、Scene 与 World Semantics

| ID | 当前差距 | 需要重构 |
|---|---|---|
| APP-VAMPIRE-P1-031 | `balance.toml`没有production consumer | typed schema加载、版本检查、默认值策略和consumer coverage |
| APP-VAMPIRE-P1-032 | damage/range/cooldown与脚本literal冲突 | ability definition引用balance ID，禁止复制裸数值 |
| APP-VAMPIRE-P1-033 | XP/level/upgrade字段无状态系统 | XP事件、level threshold、choice/upgrade transaction与UI投影 |
| APP-VAMPIRE-P1-034 | wave/spawn/max80字段无owner | deterministic spawn director、population budget与despawn policy |
| APP-VAMPIRE-P1-035 | boss配置与脚本分支无scene entity | timed boss spawn/archetype、health、AI、reward和win pressure测试 |
| APP-VAMPIRE-P1-036 | behavior tree资产无人解释 | AI provider加载compiled tree，binding引用qualified asset/version |
| APP-VAMPIRE-P1-037 | AI branch code由脚本手写11/31/41 | debug state来自tree node identity/receipt，不重复实现规则 |
| APP-VAMPIRE-P1-038 | AI plugin optional却被README宣称使用 | 未使用则删除selection/声明；使用则required并进入product receipt |
| APP-VAMPIRE-P1-039 | enemy动画bool无state machine组件 | archetype validator检查script host calls所需组件/capability |
| APP-VAMPIRE-P1-040 | navigation asset不在scene dependency graph | world/scene显式引用nav data与agent profile，加载失败阻止ready |

## 10. P1：Asset、Rendering 与 Content Quality

| ID | 当前差距 | 需要重构 |
|---|---|---|
| APP-VAMPIRE-P1-041 | GLB有128 animation但0 skin | importer报告clip/skin compatibility；样例不把placeholder计为动画资格 |
| APP-VAMPIRE-P1-042 | actor只有player state machine | player/enemy/boss各有可验证skeleton/clip/state-machine绑定 |
| APP-VAMPIRE-P1-043 | scene无collision/physics | player/world/enemy碰撞、navigation obstacle和penetration policy显式author |
| APP-VAMPIRE-P1-044 | scene无audio/listener | 若产品声明完整slice，建立music/SFX/spatial listener；否则明确capability缺失 |
| APP-VAMPIRE-P1-045 | 90 mesh与116 draw只按数量描述 | authored render budget含visible/draw/triangle/material/light/shadow/overdraw |
| APP-VAMPIRE-P1-046 | static batch只有extract DTO承诺 | renderer提交阶段验证实际merge、draw reduction和fallback correctness |
| APP-VAMPIRE-P1-047 | shader marker test与当前WGSL漂移 | 以shader interface/reflection与image behavior验证，不测私有函数名 |
| APP-VAMPIRE-P1-048 | material/shader alias缺variant receipt | material绑定qualified shader package、layout、permutation与compiled artifact |
| APP-VAMPIRE-P1-049 | 14 point light无profile预算 | light influence/shadow/cluster occupancy与quality tier进入sample budget |
| APP-VAMPIRE-P1-050 | terrain/nav/mesh坐标关系靠截图观察 | bake/import validation检查bounds、height、walkability、spawn与camera framing |

## 11. P1：Product Host、Provider 与 Platform Closure

| ID | 当前差距 | 需要重构 |
|---|---|---|
| APP-VAMPIRE-P1-051 | README命令手工拼Cargo features | `cargo zircon run --project ... --profile vampire-client`解析标准profile |
| APP-VAMPIRE-P1-052 | project selection未绑定compiled catalog | resolution receipt逐selection记录required、provider、version、packaging与结果 |
| APP-VAMPIRE-P1-053 | required provider缺失可被通用resolver跳过 | Vampire host在任一required outcome非Ready时拒绝进入scene load |
| APP-VAMPIRE-P1-054 | optional AI/animation语义与内容声明矛盾 | capability tier定义required、degraded behavior和证据矩阵 |
| APP-VAMPIRE-P1-055 | 没有sample-specific product target | build graph产出明确Vampire client package，不依赖通用dev binary隐式状态 |
| APP-VAMPIRE-P1-056 | 只记录源码树`--project`路径 | packaged smoke从install root启动并禁止访问repository/cache |
| APP-VAMPIRE-P1-057 | 无Windows/Linux/graphics backend支持矩阵 | target/backend/driver profile分别build、start、render、exit并出receipt |
| APP-VAMPIRE-P1-058 | 1280x720 access violation只有README文本 | crash捕获dump/symbol/source/BuildSet并成为阻断lane，不允许口述豁免 |
| APP-VAMPIRE-P1-059 | 无startup/ready/terminal schema | host发project/plugin/asset/script/scene generation和structured exit cause |
| APP-VAMPIRE-P1-060 | 无正常shutdown/resource leak资格 | window/GPU/VM/navigation/asset/task owner按依赖停机并跑leak/handle gate |

## 12. P1：Test、Oracle 与 Evidence Authority

| ID | 当前差距 | 需要重构 |
|---|---|---|
| APP-VAMPIRE-P1-061 | asset test使用test-only importer catalog | product integration test使用标准Vampire product profile |
| APP-VAMPIRE-P1-062 | 10个real-VM测试永久ignore | 新owner先建立等价lane，旧owner同变更删除/迁移，禁止长期ignore |
| APP-VAMPIRE-P1-063 | gameplay未ignore测试只搜WASD字符串 | 运行input trace并断言position、state、time和terminal receipt |
| APP-VAMPIRE-P1-064 | import test约700行混合多域 | 拆成source closure、import、scene binding、shader/material、render product tests |
| APP-VAMPIRE-P1-065 | test直接断言shader私有marker | 断言reflection contract、compiled pipeline与reference image outcome |
| APP-VAMPIRE-P1-066 | 无clean checkout/cache-cold lane | CI在isolated checkout、empty cache/registry中完成import/cook/run |
| APP-VAMPIRE-P1-067 | 无full gameplay deterministic test | scripted 10-minute/accelerated trace覆盖spawn、upgrade、boss、win与retry |
| APP-VAMPIRE-P1-068 | 无save/load/migration test | snapshot round trip、旧版本迁移、corruption rejection和resume determinism |
| APP-VAMPIRE-P1-069 | 无fault injection | missing provider/asset、VM trap、device loss、resize、OOM budget和shutdown测试 |
| APP-VAMPIRE-P1-070 | `.github`/tools没有Vampire lane | required CI job发布JUnit、structured evidence、screenshots、metrics与crash artifact |

## 13. P1：Visual、Performance、Release 与 Documentation Truth

| ID | 当前差距 | 需要重构 |
|---|---|---|
| APP-VAMPIRE-P1-071 | 34张PNG无source/build identity | 每张reference/capture配不可分离sidecar与content digest |
| APP-VAMPIRE-P1-072 | 多组重复PNG伪装成时间序列 | evidence store按digest去重，sequence必须记录不同frame/input cursor |
| APP-VAMPIRE-P1-073 | 图片尺寸从640x360到1440x745混杂 | qualification profile固定resolution、aspect、DPI、backend与camera state |
| APP-VAMPIRE-P1-074 | start/ground/game-over accepted set缺2张 | 缺失artifact使对应case失败，README只能引用已发布EvidenceSet |
| APP-VAMPIRE-P1-075 | 截图跨UI/scene代际且无oracle | baseline更新需review receipt，旧generation归档而非混入current目录 |
| APP-VAMPIRE-P1-076 | 18个证据日志为空文件 | capture失败产生structured terminal receipt；空日志不得算evidence |
| APP-VAMPIRE-P1-077 | performance使用单提交帧/current值 | warm-up、采样窗口、percentile、CPU/GPU breakdown、variance与重复次数固定 |
| APP-VAMPIRE-P1-078 | 60.87与30.89/33.98口径冲突 | 同profile paired benchmark建立canonical baseline和regression threshold |
| APP-VAMPIRE-P1-079 | visual quality只有人工营销描述 | 定义可检查场景构图、可见角色/HUD、曝光、非空区域和image thresholds |
| APP-VAMPIRE-P1-080 | README把future目标写成current acceptance | 文档分Current Qualified/Future Target/Known Blocker并从receipt生成状态 |

## 14. P2：成熟度提升

| ID | 提升项 | 目标 |
|---|---|---|
| APP-VAMPIRE-P2-001 | sample content lint dashboard | resource、license、meta、orphan、budget和evidence一屏可查 |
| APP-VAMPIRE-P2-002 | deterministic seed browser | replay任一wave/combat seed并对比state digest |
| APP-VAMPIRE-P2-003 | gameplay telemetry overlay | 显示fixed tick、enemy budget、ability cooldown、nav与VM cost |
| APP-VAMPIRE-P2-004 | automated screenshot storyboard | menu、play、attack、upgrade、boss、game-over固定镜头批量产证 |
| APP-VAMPIRE-P2-005 | sample packaging matrix | debug/development/shipping与backend组合自动比较闭包 |
| APP-VAMPIRE-P2-006 | asset provenance report | 从最终画面反查source/importer/artifact/material/shader代际 |
| APP-VAMPIRE-P2-007 | balance curve simulator | 离线模拟DPS、spawn、XP、boss与10分钟目标并输出置信区间 |
| APP-VAMPIRE-P2-008 | AI trace viewer | 展示tree node、blackboard、target、path和decision时间线 |
| APP-VAMPIRE-P2-009 | run replay artifact | input/RNG/config/BuildSet驱动可移植确定性复现 |
| APP-VAMPIRE-P2-010 | visual backend differential | 同一capture在D3D12/Vulkan等后端做阈值比较 |
| APP-VAMPIRE-P2-011 | performance scalability tiers | Low/Medium/High/Epic各有visual与frame budget |
| APP-VAMPIRE-P2-012 | sample upgrade fixture | 保留旧project/meta/save package验证跨版本升级 |
| APP-VAMPIRE-P2-013 | localization/accessibility sample | 菜单、HUD、字体、缩放、输入重绑和色觉模式进入产品场景 |
| APP-VAMPIRE-P2-014 | content authoring recipe | 从source到import/cook/evidence的可执行而非叙事型教程 |
| APP-VAMPIRE-P2-015 | release provenance bundle | SBOM、licenses、symbols、receipts、baselines与known issues同包发布 |
| APP-VAMPIRE-P2-016 | reference performance history | 只保留同profile可比较趋势并自动标注不兼容代际 |

## 15. 重构所有权与目标架构

| Owner | 本篇责任 | 不在本篇重复实现 |
|---|---|---|
| `VampireProductProfile` | project selection、target、quality、script mode、evidence profile | 通用provider catalog算法 |
| `VampireSourceManifest` | tracked source、recipe、license、entry graph和clean-clone closure | 通用asset registry/cache |
| `VampireRunCoordinator` | run state、fixed schedule、checkpoint、retry、win/terminal | 通用app main loop |
| `VampireSimulationConfig` | balance schema与所有字段consumer映射 | 通用Gameplay Ability框架 |
| `VampireWorldDirector` | spawn/wave/population/boss/progression orchestration | 通用AI/nav/scene system |
| `VampireScriptPackageBuilder` | relocatable project script artifact与receipt | ZrVM compiler/runtime内部 |
| `VampireEvidenceHarness` | deterministic trace、visual/perf/crash/product result聚合 | 通用test/evidence store |

产品资格链必须是：

`Clean Checkout -> Source Closure -> Product Profile Resolve -> Import/Cook -> Script Build -> Package/Install -> Start/Ready -> Deterministic Gameplay Trace -> Visual/Performance/Crash Gates -> Structured EvidenceSet`。

任一环节不得读取repository外绝对路径、开发者历史cache、test-only provider或未声明artifact；最终receipt必须能从package反查所有source、provider和toolchain代际。

## 16. 依赖序里程碑

### M0 · Truth Freeze 与隔离复现

冻结README能力声明；在临时clean checkout/empty cache中复现missing source、provider resolution、stale test和证据缺失，生成首份机器可读gap inventory。

### M1 · Source 与 Product Closure

修正ignore/source/recipe/license，建立`VampireSourceManifest`与标准`VampireProductProfile`；clean import不使用test fixture或本机cache。

### M2 · Relocatable Build/Cook/Package

脚本、资产、provider和scene统一进入content-addressed build；从install root启动，manifest无绝对路径，artifact/BuildSet receipt闭合。

### M3 · Authoritative Gameplay Slice

把移动、攻击、接触伤害、AI和run state迁入fixed/typed边界；使balance/tree成为真实consumer，补齐spawn、XP/upgrade、boss/win、save/retry。

### M4 · Product Tests 与 Failure Matrix

原子接回10个real-VM行为测试，拆分asset测试，增加clean-cache、packaged run、save/replay、missing provider/asset、VM/device/shutdown fault lanes。

### M5 · Visual 与 Performance Qualification

重建三类核心截图及完整storyboard，使用image oracle、固定profile和paired benchmark；解决1280x720 access violation并保存crash evidence。

### M6 · Shipping Sample Acceptance

在支持平台/后端矩阵上完成clean build、package/install、10分钟deterministic run、visual/perf/leak/exit gates；README只从accepted EvidenceSet生成current状态。

## 17. 资格门

1. clean checkout中`examples/vampire`required source closure为100%，ignored required source为0。
2. empty cache/registry完成全部52类meta对应资产导入或确定性cook，不读取开发机残留。
3. 108个scene resource引用全部解析到tracked source或qualified packaged artifact。
4. project required plugin selection在标准Vampire profile中逐项Ready，无test-only provider。
5. script package manifest不含盘符/绝对workspace路径，所有登记artifact存在且digest匹配。
6. source、script、asset、provider、host和package共享BuildSet generation。
7. fixed simulation下不同render FPS产生相同移动、DPS、wave与state digest。
8. `balance.toml`每个current字段至少有一个production consumer和behavior test。
9. behavior tree由qualified AI provider执行，脚本不复制node graph。
10. 10分钟run覆盖Start、movement、combat、spawn、XP/upgrade、boss、win与terminal receipt。
11. Retry/restore重建完整world checkpoint，旧run残留为0。
12. real-VM Vampire行为测试required运行，永久ignore数量为0。
13. asset/import/render product tests使用与shipping相同profile/catalog/package。
14. clean-cache CI、packaged smoke、fault matrix与save/replay lanes均发布结构化结果。
15. start-menu、play/ground、game-over reference图均存在且通过image threshold。
16. 每张accepted图绑定source/BuildSet/GPU/driver/resolution/frame/input/camera与oracle结果。
17. performance使用预热、多帧、重复样本和percentile；与canonical baseline同profile比较。
18. 1280x720及支持分辨率连续运行无access violation、device loss或非结构化退出。
19. 正常/失败停机均释放window/GPU/VM/nav/asset/task资源并返回稳定exit code。
20. README的Current Qualified条目全部可追到未过期EvidenceSet，不存在缺失artifact或future-as-current。

## 18. 本轮验证与限制

本轮执行了tracked/ignored文件枚举、resource引用存在性检查、scene TOML结构统计、52份`.zmeta`解析/UUID检查、24份GLB容器/chunk统计、脚本host-call/数据consumer扫描、测试ignore/owner扫描、截图尺寸/hash/sidecar清单和README/性能文档交叉核对。所有操作为只读审查；没有修改Vampire production、tests、manifest、asset、script binary、cache或截图。

没有把当前工作区“所有resource存在”计为成功，因为其中7份模型源被ignore且未tracked。没有把本地cache计为发布产物，没有把test-only importer计为product provider，也没有把README历史运行日志计为current动态验证。当前workspace仍有既存编译阻断；源码条件未变化，本轮不重复执行同一失败lane。实施前必须从M0隔离复现重新建立可执行基线。
