---
related_code:
  - templates/projects/renderable-empty
  - templates/projects/renderable-empty/.gitignore
  - templates/projects/renderable-empty/zircon-project.toml
  - templates/projects/renderable-empty/.zircon/settings.toml
  - templates/projects/renderable-empty/assets/materials/default.zmaterial
  - templates/projects/renderable-empty/assets/materials/default.zmaterial.zmeta
  - templates/projects/renderable-empty/assets/models/cube.obj
  - templates/projects/renderable-empty/assets/models/cube.obj.zmeta
  - templates/projects/renderable-empty/assets/scenes/main.scene.toml
  - templates/projects/renderable-empty/assets/shaders/pbr_shader.zmeta
  - templates/projects/renderable-empty/assets/shaders/pbr_shader/pbr.wgsl
  - templates/projects/renderable-empty/assets/shaders/pbr_shader/pbr.zshader
  - templates/projects/renderable-empty/export/desktop_windows.zpreset
  - zircon_runtime_interface/src/project/template_pack
  - zircon_runtime_interface/src/project/manifest_summary
  - zircon_editor/src/core/project/authority.rs
  - zircon_editor/src/core/project/filesystem.rs
  - zircon_editor/src/core/project/error.rs
  - zircon_hub/src/projects/create_project.rs
  - zircon_hub/src/projects/create_project_request.rs
  - zircon_app/src/entry/cli/launch_args.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_runtime/src/asset/project
  - tools/mvp/Stage-MvpProducts.ps1
  - tools/mvp/Invoke-MvpAcceptance.ps1
tests:
  - .github/workflows/mvp-editor-windows.yml
  - zircon_runtime_interface/src/project/tests/template_pack.rs
  - zircon_editor/src/core/project/tests/template_creation.rs
  - zircon_editor/src/core/project/tests/directory_transaction.rs
  - zircon_editor/src/tests/workbench/project/renderable_template.rs
  - zircon_runtime/src/asset/tests/project/template_contract.rs
  - zircon_runtime/src/dynamic_api/session/tests/foundation_render.rs
  - zircon_app/tests/editor_mvp_authoring.rs
  - zircon_hub/tests/project_management_contract.rs
  - tools/tests/mvp-staging.Tests.ps1
  - tools/tests/render-extract-baseline-capture.Tests.ps1
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
reference_engines:
  - dev/UnrealEngine/Templates/TP_ThirdPerson/TP_ThirdPerson.uproject
  - dev/UnrealEngine/Templates/TP_ThirdPerson/Config/TemplateDefs.ini
  - dev/UnrealEngine/Templates/TP_ThirdPerson/Config/DefaultEngine.ini
  - dev/UnrealEngine/Templates/TP_ThirdPerson/Source/TP_ThirdPerson.Target.cs
  - dev/UnrealEngine/Templates/TP_ThirdPerson/Source/TP_ThirdPersonEditor.Target.cs
  - dev/bevy/Cargo.toml
  - dev/bevy/examples/README.md
  - dev/Fyrox/template/src/main.rs
  - dev/Fyrox/template-core/src/lib.rs
  - dev/Fyrox/project-manager/src/manager.rs
  - dev/godot/editor/project_manager/project_dialog.cpp
  - dev/godot/editor/export/editor_export_platform.cpp
  - dev/Graphics/TestProjects/PostProcessing_Tests/Packages/manifest.json
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 07 · Renderable Empty 项目模板、创建、导入、渲染、导出与证据产品工程化差距

## 1. 结论

`templates/projects/renderable-empty`是当前唯一启用的用户项目模板，也是一个高扇出产品输入。17个文件、217行、4,867 bytes被`zircon_runtime_interface`逐项`include_bytes!`内嵌；Editor、Hub、Runtime、MVP staging、UI/profile scale fixture和101个非reference文件直接或间接依赖`renderable-empty`身份。28个测试文件覆盖模板内容、创建事务、registry恢复、Editor打开、动态Runtime、Hub请求和staging。它不是边缘fixture，任何隐式合同都会复制到所有新项目和大量引擎证据中。

现有底座有值得保留的工程化内容。`RelPath`在模板条目进入filesystem前做相对路径约束；Runtime Interface测试比较embedded pack与source tree的完整文件集合；Editor创建在同一parent内写staging并用rename发布，对空target、commit、open和post-commit failure有backup/rollback测试；template scene引用与三个固定UUID一致；Editor测试会删除cache/registry后从source/meta重建；F2动态测试真实导入scene/model/material/shader，用WGPU捕获RGBA并要求超过100个非背景pixel、mesh draw、directional light、零material fallback；Windows workflow还用source fingerprint串联F1-F5并保留诊断artifact。

但“可渲染空项目”仍不是一个自描述、可升级、跨产品面一致的产品包。唯一`ProjectTemplateId::RenderableEmpty`没有template schema/version/content digest/engine compatibility；每次修改17个embedded bytes都沿用同一ID。创建后的`zircon-project.toml`也不记录template ID、version、source digest、creator build或migration baseline。用户报告问题、Hub重新打开、项目升级和CI复现都无法回答“这个项目由哪一代模板生成”。

项目manifest没有任何plugin selection、render provider、backend或capability要求。相同项目在Editor、`target-client`、不同first-party feature组合和export pipeline中依赖host恰好编入的provider；Plugins06已经证明标准profile与source catalog的required Rendering闭包并不自洽。模板测试能在测试binary中导入/渲染，不等于新项目能从manifest解析出相同BuildSet。

创建语义也分裂。Editor `ProjectAuthority`在发布后打开`ProjectManager`，并对open/finalize failure执行rollback；Hub维护另一份约260行创建/commit实现，复制17个文件后立即返回`CreateProjectReport`，不打开project、不scan/import、不验证default scene/asset Ready/render profile，也没有Editor同等post-commit rollback。Hub可以显示“创建成功”，随后Editor才发现import、scene或provider失败。共享pack只统一bytes，没有统一创建operation或Ready定义。

模板还默认暴露`desktop_windows` release export profile和`.zpreset`。Tooling03已经动态证明当前export链可把无效pack与placeholder host报告为成功，generated host启动后立即释放runtime owner，preset字段也没有完整consumer。App07不重复拥有通用export P0，但模板产品必须在Tooling03资格通过前隐藏或标记该profile不可发布；把它放进每个新项目会把未实现流水线包装成默认能力。

当前Windows MVP workflow是重要基础，但还不能给模板永久shipping资格。F1主要验证source/schema，F2直接render embedded pack并走offscreen dynamic session，F5通过Editor CLI创建并运行staged product；Hub创建路径、installed/exported game、Linux/macOS、template升级和长期baseline仍未覆盖。CI artifact只保留7天，仓库没有与当前template digest绑定的accepted reference/receipt。更直接的是当前workspace的Editor compile lane已有239个既存error，当前HEAD不能复用历史workflow成功结论。

本篇只拥有project template package identity、创建面一致性、模板默认project capability/export truth和template级资格消费。Runtime04拥有通用asset/meta/import/cache，Interface02拥有通用project/schema DTO，Editor02/04拥有document/import transaction，Hub01拥有Hub process/persistence，Plugins06拥有provider catalog/profile，Tooling03拥有export实现，Tooling10拥有test architecture。本轮登记 **4项P0、72项P1和16项P2**。

## 2. 审查边界与物理清单

### 2.1 Template source pack

| 类型 | 数量 | bytes | 当前语义 |
|---|---:|---:|---|
| 全部文件 | 17 | 4,867 | Runtime Interface编译期逐项内嵌 |
| `.gitignore` | 6 | 345 | root加5个derived子目录keep-file |
| project/settings/TOML | 3 | 1,485 | project manifest、Editor settings、scene |
| source assets | OBJ 1 / material 1 / WGSL 1 / zshader 1 | 1,643 | cube + project PBR surface |
| metadata | 3 | 1,039 | shader/model/material固定UUID；dirty/empty digest/importer |
| export preset | 1 | 355 | Windows client release、binary asset、zstd、deterministic声明 |

scene包含Camera、Sun、Cube三个entity；entity ID 1..3，Cube引用model UUID `...0002`与material UUID `...0003`，material引用shader UUID `...0001`。两个`path_hint`均存在。camera/frustum test同时验证初始Cube与F4把X改为42后的中心仍在16:9视锥内。模板没有scene `.zmeta`，首次scan会创建/补全registry与artifact状态。

三份tracked meta均为format 7、固定全零前缀UUID、`preview_state = "dirty"`、空`importer_id`、空`source_digest`、`source_mtime_unix_ms = 0`和`importer_version = 0`。这适合“首次打开后导入”的source seed，却不能当作ready artifact receipt；模板创建与首次import是两个不同产品阶段。

### 2.2 Pack、Editor 与 Hub 实现

| Owner树 | 规模 | 结论 |
|---|---:|---|
| Runtime Interface `template_pack` | 7文件 / 237行 / 7,811 bytes | 单enum ID、17 entry、name rewrite、RelPath与manifest summary；无version/digest/compatibility |
| Editor `core/project` | 18文件 / 3,064行 / 104,784 bytes | 有较完整path identity、staging、backup、open与rollback基础 |
| Hub `projects` | 13文件 / 2,659行 / 85,917 bytes | catalog/create/request混合；create复制后即成功，不做project Ready资格 |

Runtime Interface的`render_project_template`只trim project name、clone entry bytes、用TOML重写`name`并解析summary。它不生成新asset identity、不记录template receipt，也不验证scene依赖、meta/source一致、provider selection或export compatibility。Editor/Hub虽消费同一个rendered pack，却复制了不同的transaction/validation代码。

### 2.3 测试与CI

仓内至少28个test文件直接命中模板身份。强证据包括embedded/source集合相等、unsafe name/path、empty/non-empty target、commit/rollback fault、cache/registry重建、scene reference/asset Ready、F2 WGPU capture、F4 authoring restart和Hub contract。`.github/workflows/mvp-editor-windows.yml`在Windows构建Editor/Runtime，运行F1-F4 exact test，再执行profile/workspace gates和source-bound staged F5。

证据仍有边界：F2 helper直接把rendered entries写入test-binary旁fixture，未经过Editor或Hub创建；F5覆盖Editor CLI但不覆盖Hub创建；workflow只在Windows，artifact retention为7天；没有template version迁移、跨project copy/remap、installed export launch或当前digest对应的长期reference registry。当前local Editor compile blocker未变化，本轮没有重复运行。

## 3. 参考引擎约束

- Unreal Third Person template用`TemplateDefs.ini`声明localized name/description、ignore、folder rename、filename/content replacement与shared content packs；`.uproject`声明Runtime module和enabled plugins，Game/Editor各有Target，`DefaultEngine.ini`声明game/editor map、game mode和target renderer。Zircon无需复制内容体量，但template package、project module/provider、target/profile和生成替换必须是显式合同。
- Bevy不是Editor模板系统；其`Cargo.toml`对example声明required features，example源码显式组装`DefaultPlugins`与schedule。这里的约束只是“样例/模板运行能力由build feature与composition明确声明”，不能从测试binary的feature反推用户项目能力。
- Fyrox template generator生成game、executor、editor、export-cli、wasm/android target与workspace manifests，并提供upgrade入口；project manager读取Cargo metadata并实际运行/编辑项目。其实现仍有可改进处，但证明template generation要拥有依赖版本、role artifact与升级路径。
- Godot project dialog在创建/导入前验证path、project.godot或ZIP结构，并让用户选择renderer；export再枚举project files并生成pack。Renderer选择与项目身份在创建时落盘，不能推迟到某个host binary恰好带什么feature。
- Unity Graphics test projects通过`Packages/manifest.json`固定包依赖，测试runner加载scene并做reference image comparison。这里仅借鉴test project dependency pinning与机器oracle，不把Graphics仓库外推为完整Unity project generator。

## 4. 可保留的正确基础

1. 单一source template tree与embedded entry集合有exact test，不存在手写两份payload。
2. `RelPath`约束每个embedded path，模板source tree测试拒绝symlink/reparse point。
3. project name通过TOML parser重写，能正确处理引号而非字符串替换。
4. Editor创建使用同parent staging/rename，拒绝非空target并覆盖多种rollback failure。
5. source scene/entity/asset reference和meta UUID之间有typed解析与registry一致性测试。
6. cache/registry被定义为可再生状态，删除或corrupt后可从source/meta重建。
7. F2不是“非空颜色”测试：它检查changed pixels、graph pass、mesh draw、light与material fallback。
8. F4验证一次真实application authoring/restart，F5有source fingerprint和staged product输入。
9. Hub catalog只启用真实存在的一个template，另外三个reserved项没有被允许创建。

## 5. P0：项目模板产品资格硬阻断

### APP-TEMPLATE-P0-001 · Template ID可变但没有版本、内容身份与兼容范围

所有17个文件都映射到永恒的`RenderableEmpty`/`renderable-empty`。pack没有schema version、content digest、engine min/max、migration ID或deprecation状态；创建后的project也不记录template provenance。同名template跨commit改变后，用户项目、support日志和CI无法识别生成代际，无法选择migration或证明复现。

建立`ProjectTemplateDescriptor`与`ProjectTemplateReceipt`：qualified ID + version、content root digest、engine/API range、target/capability matrix、migration predecessor、license和evidence set。创建时把receipt写入project manifest的provenance段；后续升级只通过显式migration，不修改旧版本语义。

### APP-TEMPLATE-P0-002 · 新项目不声明provider/BuildSet，Renderability依赖host偶然feature

manifest只声明default scene、asset root、library version与Windows export strategy，`plugins`命中为0。它不要求Rendering、OBJ/material/shader importer、platform/window或backend，也不绑定compiled catalog。F2测试binary拥有的provider不等于`target-client`、Editor、Hub launch和export host都有相同闭包；Plugins06已经证明standard profile required closure可断裂或静默丢selection。

模板必须声明minimum product capability和qualified provider selection；创建时ProductComposer解析为immutable BuildSet receipt。Editor open、native runtime、F2/F5和export只能消费同一receipt；required provider缺失时在创建/打开preflight阻断，不能生成“可渲染”项目后再降级。

### APP-TEMPLATE-P0-003 · Hub 与 Editor 创建成功定义分裂

Editor创建发布后会canonicalize并`ProjectManager::open_resolved`，open/finalize失败则回滚；Hub复制同一pack后立即返回report，不open、不scan/import、不加载default scene、不验证asset Ready或provider，也没有post-commit product rollback。两套create/commit helper独立演化，Hub成功只代表17次write/rename成功。

建立共享`CreateProjectOperation`：ValidateRequest -> RenderPack -> Stage -> ValidateProject -> ResolveBuildSet -> Import/Compile -> LoadDefaultScene -> optional render probe -> Commit -> Ready receipt。Editor与Hub只投影operation progress/result，不维护第二事务。失败零发布或恢复empty target，并保留typed recovery artifact。

### APP-TEMPLATE-P0-004 · 默认项目暴露未达到发布资格的Windows release export

每个新项目都带`desktop_windows` release profile和preset，声明binary assets、zstd、deterministic及三种packaging strategy。Tooling03已经证明这些字段没有完整consumer，placeholder host与无效pack仍可被PlatformBundle判成功，generated runtime host又立即退出。模板把“可选择的配置”呈现为默认发布能力，用户无法区分schema valid与artifact runnable。

在Tooling03的Build/Cook/Pack/Install/Run gates完成前，将profile状态设为`unavailable`并给出缺失capability receipt，或从shipping template移除。恢复时必须由目标platform provider生成，完成real host/object/dependency/sign、zrpack parse、installed launch/first frame/clean exit；App07只消费结果，不复制export实现。

## 6. P1：Template Package、Identity 与 Schema

| ID | 当前差距 | 需要重构 |
|---|---|---|
| APP-TEMPLATE-P1-001 | enum只有一个无version variant | qualified template ID包含namespace/name/version |
| APP-TEMPLATE-P1-002 | descriptor由Rust match隐式决定 | versioned manifest声明entry、capability、target、compatibility与evidence |
| APP-TEMPLATE-P1-003 | embedded entries无per-file digest | build生成sorted file manifest、bytes、mode、digest与root hash |
| APP-TEMPLATE-P1-004 | created project不记录creator build | provenance保存engine BuildSet、template receipt与creation operation ID |
| APP-TEMPLATE-P1-005 | 没有template migration graph | old descriptor保留，upgrade声明from/to、preflight、backup与rollback |
| APP-TEMPLATE-P1-006 | 无deprecation/withdrawal状态 | catalog区分available/deprecated/unsupported/revoked与replacement |
| APP-TEMPLATE-P1-007 | project name只trim，不统一display/filesystem identity | shared validated ProjectName同时拥有display与safe target segment |
| APP-TEMPLATE-P1-008 | Runtime Interface public render只拒绝空名 | API返回validated request/receipt，禁止绕过caller policy |
| APP-TEMPLATE-P1-009 | TOML re-encode可能改变canonical字节形状 | manifest writer定义canonical format与semantic digest，不依赖原排版 |
| APP-TEMPLATE-P1-010 | pack无license/provenance清单 | descriptor逐entry声明origin、license、generated/source状态 |
| APP-TEMPLATE-P1-011 | pack无locale/preview metadata | catalog内容与template version绑定并有fallback、thumbnail digest |
| APP-TEMPLATE-P1-012 | pack更新没有compatibility review gate | CI比较descriptor/schema/capability/asset delta并要求migration decision |

## 7. P1：Source、Asset、Meta 与 Scene Closure

| ID | 当前差距 | 需要重构 |
|---|---|---|
| APP-TEMPLATE-P1-013 | scene source没有tracked `.zmeta` | 明确scene是generated-meta source并在create receipt列出first-import mutation，或交付完整meta |
| APP-TEMPLATE-P1-014 | 三份meta固定mtime 0/empty digest | seed schema与qualified ready meta分型，禁止把seed字段当current receipt |
| APP-TEMPLATE-P1-015 | importer ID/version为空 | descriptor声明required importer kind/version和fallback policy |
| APP-TEMPLATE-P1-016 | preview dirty没有创建状态投影 | create/import progress区分SourceMaterialized/Imported/PreviewReady |
| APP-TEMPLATE-P1-017 | 所有项目复制相同asset UUID | project scope需进入qualified identity；跨项目copy/import必须remap并测试 |
| APP-TEMPLATE-P1-018 | UUID使用可读全零序列 | 生成稳定template-local ID并明确保留/实例化策略，避免保留值被误当global |
| APP-TEMPLATE-P1-019 | scene只测point center在frustum | 验证AABB、projected pixel footprint、near clipping与多个aspect ratio |
| APP-TEMPLATE-P1-020 | FOV 100度与远置相机无quality profile | camera framing从template visual contract生成并有reference image |
| APP-TEMPLATE-P1-021 | material仅base color slot | 明确minimum PBR contract和missing texture/default resource ownership |
| APP-TEMPLATE-P1-022 | OBJ没有import recipe/settings | template锁定coordinate、normal/tangent/UV、scale与material policy |
| APP-TEMPLATE-P1-023 | root ignore只管理`.zircon` | 加入平台/editor/build/export常见derived output并做source-closure lint |
| APP-TEMPLATE-P1-024 | 无source artifact inventory | recursive dependency graph证明scene->model/material->shader全闭合且无orphan |

## 8. P1：Create Operation、Filesystem 与 Product Surfaces

| ID | 当前差距 | 需要重构 |
|---|---|---|
| APP-TEMPLATE-P1-025 | Editor/Hub复制create实现 | 单一operation owner和共享fault-injected transaction |
| APP-TEMPLATE-P1-026 | transaction ID只是PID+进程内counter | 使用不可冲突operation ID并处理crash残留/stale staging |
| APP-TEMPLATE-P1-027 | Hub cleanup吞掉remove failure | terminal receipt列出cleanup failure与recoverable paths |
| APP-TEMPLATE-P1-028 | Hub backup remove failure静默忽略 | success需说明backup retained并安排bounded recovery |
| APP-TEMPLATE-P1-029 | Hub不做derived layout owner校验 | shared operation创建并验证cache/registry/autosave/play/thumbnail布局 |
| APP-TEMPLATE-P1-030 | Hub不解析published manifest | commit前后重读canonical manifest并核对summary/digest |
| APP-TEMPLATE-P1-031 | Hub不scan/import | required asset全部Ready才返回project-ready，或明确CreatedNotReady状态 |
| APP-TEMPLATE-P1-032 | Hub不load default scene | scene/schema/reference failure进入create terminal result |
| APP-TEMPLATE-P1-033 | Editor create open不等于full application ready | receipt区分Created/Opened/Imported/WorldReady/FirstFrameReady |
| APP-TEMPLATE-P1-034 | CLI、Welcome与Hub没有同一operation schema | 所有surface消费同一request/progress/cancel/result DTO |
| APP-TEMPLATE-P1-035 | create无deadline/cancel | 大template/import/compile可取消且commit点前零发布 |
| APP-TEMPLATE-P1-036 | create无concurrent target lease | parent/target writer lease防止两个Hub/Editor transaction竞争 |

## 9. P1：Runtime、Provider 与 Renderability

| ID | 当前差距 | 需要重构 |
|---|---|---|
| APP-TEMPLATE-P1-037 | `plugins` selection为0 | minimum render/import/platform providers进入manifest |
| APP-TEMPLATE-P1-038 | library version不能表达provider ABI | BuildSet包含engine/runtime ABI、plugin/package/schema identities |
| APP-TEMPLATE-P1-039 | template不声明render profile | default 3D quality/backend capability明确且可降级策略typed |
| APP-TEMPLATE-P1-040 | Editor与Runtime可能解析不同catalog | 两者验证同一composition receipt/hash |
| APP-TEMPLATE-P1-041 | F2直接写pack绕过create operation | 增加shared operation->native runtime product test |
| APP-TEMPLATE-P1-042 | F2是offscreen dynamic session | 同scene还需native window/swapchain/present/resize/exit资格 |
| APP-TEMPLATE-P1-043 | changed pixels只比较首pixel | image oracle使用reference/threshold/mask并检测fallback/blank patterns |
| APP-TEMPLATE-P1-044 | test只要求mesh draw大于0 | 锁定expected visible set、draw/material/light/pass budget与generation |
| APP-TEMPLATE-P1-045 | 无device/backend matrix | D3D12/Vulkan及software policy分别声明支持/不可用 |
| APP-TEMPLATE-P1-046 | 无resource residency/warm frame门 | 首帧与steady state分别验证upload、cache、RSS/VRAM和无增长 |
| APP-TEMPLATE-P1-047 | input test只验证event drain | project至少提供action mapping或明确“无gameplay input”的template tier |
| APP-TEMPLATE-P1-048 | template无shutdown receipt | session/window/GPU/asset/provider释放与project目录可删除进入产品结果 |

## 10. P1：Export、Target 与 Distribution Truth

| ID | 当前差距 | 需要重构 |
|---|---|---|
| APP-TEMPLATE-P1-049 | manifest/preset双份profile authority | 单一qualified export profile，preset只引用不可复制字段 |
| APP-TEMPLATE-P1-050 | 三种strategy同时列出无选择规则 | profile生成resolved plan并说明顺序/互斥/产物 |
| APP-TEMPLATE-P1-051 | Windows-only profile无support matrix | catalog按host/target/toolchain显示可用状态 |
| APP-TEMPLATE-P1-052 | `release`不等于shipping | shipping policy包含symbols/assert/log/sign/hardening与optimization |
| APP-TEMPLATE-P1-053 | zstd/deterministic只是请求字段 | cook/pack receipt证明codec、order、rebuild hash和reader验证 |
| APP-TEMPLATE-P1-054 | 无entry-scene cook closure | export解析default/extra scene递归依赖并拒绝missing/unknown asset |
| APP-TEMPLATE-P1-055 | 无project-specific host | build输出能持续运行template scene的runtime executable/instance |
| APP-TEMPLATE-P1-056 | 无installed-root isolation test | exported game禁止读取repo、Cargo target、source cache和template tree |
| APP-TEMPLATE-P1-057 | 无object/architecture/dependency gate | Windows host验证PE machine/subsystem/import DLL closure |
| APP-TEMPLATE-P1-058 | 无sign/install/uninstall receipt | package identity、publisher、install root、repair/remove可审计 |
| APP-TEMPLATE-P1-059 | export UI可能把Unavailable显示可选 | admission state驱动按钮、diagnostic与recovery，不显示假成功 |
| APP-TEMPLATE-P1-060 | template未绑定Tooling03 evidence | descriptor只引用未过期、同digest、同target的export qualification |

## 11. P1：Test、Evidence、CI 与 Documentation

| ID | 当前差距 | 需要重构 |
|---|---|---|
| APP-TEMPLATE-P1-061 | 28个test文件无统一template test plan | inventory标明owner、layer、profile、platform、required/optional与oracle |
| APP-TEMPLATE-P1-062 | F1命名与F2函数名混杂 | stable gate ID与test identity分离，重命名不丢历史 |
| APP-TEMPLATE-P1-063 | F2不消费create operation | 添加Editor/Hub create后的相同runtime test |
| APP-TEMPLATE-P1-064 | Hub只测文件复制/request | Hub UI action->operation->Ready->Editor open做真实E2E |
| APP-TEMPLATE-P1-065 | F5只覆盖Windows | 支持平台至少做build/create/import/run/exit，unsupported显式skip receipt |
| APP-TEMPLATE-P1-066 | CI artifact retention仅7天 | accepted baseline进入durable evidence store并有retirement policy |
| APP-TEMPLATE-P1-067 | 无template digest绑定 | 每个结果记录descriptor/file-root/engine/BuildSet/toolchain digest |
| APP-TEMPLATE-P1-068 | 无version migration test | 旧template project打开/升级/save/reopen/export保持或明确转换语义 |
| APP-TEMPLATE-P1-069 | 无cross-project copy/remap test | 固定UUID资产复制/合并时验证qualified scope与collision处理 |
| APP-TEMPLATE-P1-070 | 无installed export visual test | release package启动、首帧reference、resize/input/clean exit |
| APP-TEMPLATE-P1-071 | historical pass可被当current | source/template/toolchain漂移自动expire qualification |
| APP-TEMPLATE-P1-072 | catalog描述“current engine runtime”不可审计 | UI显示qualified targets/status/last evidence，不使用无条件营销文案 |

## 12. P2：成熟度提升

| ID | 提升项 | 目标 |
|---|---|---|
| APP-TEMPLATE-P2-001 | versioned template gallery | 预览、capability、target、size与evidence可比较 |
| APP-TEMPLATE-P2-002 | organization template registry | signed internal templates、policy与offline mirror |
| APP-TEMPLATE-P2-003 | parameter schema | renderer、language、target、source control等typed options |
| APP-TEMPLATE-P2-004 | deterministic template diff | 更新前展示source/derived/migration影响 |
| APP-TEMPLATE-P2-005 | template SDK | validate/render/migrate/test/package接口供第三方扩展 |
| APP-TEMPLATE-P2-006 | content-addressed pack cache | 相同version复用且离线可验证 |
| APP-TEMPLATE-P2-007 | sample asset provenance UI | 新项目内可查看origin/license/import recipe |
| APP-TEMPLATE-P2-008 | multi-backend visual matrix | reference image按backend/vendor阈值管理 |
| APP-TEMPLATE-P2-009 | creation telemetry | stage/import/compile/first-frame耗时与失败分布 |
| APP-TEMPLATE-P2-010 | repair operation | 根据receipt恢复缺失source/derived而不覆盖用户修改 |
| APP-TEMPLATE-P2-011 | template conformance fuzz | entry path、manifest、archive、unicode与fault组合 |
| APP-TEMPLATE-P2-012 | package size/budget policy | source、cook、install、first-frame内存和draw预算 |
| APP-TEMPLATE-P2-013 | localization/accessibility seed | 选择性提供最小UI/input/a11y示例而不污染empty tier |
| APP-TEMPLATE-P2-014 | headless/server template tier | 与renderable client分开声明artifact和lifecycle |
| APP-TEMPLATE-P2-015 | upgrade preview sandbox | migration在临时clone运行并比较project graph/evidence |
| APP-TEMPLATE-P2-016 | reproducible support bundle | template/build/project/evidence身份一键导出且敏感字段脱敏 |

## 13. 重构所有权与目标架构

| Owner | 本篇责任 | 依赖/非重复owner |
|---|---|---|
| `ProjectTemplateRegistry` | descriptor/version/catalog/deprecation/compatibility | O00/O03 truth与schema |
| `ProjectTemplatePackBuilder` | sorted entries、digest、license、signature、embedded artifact | O01/O04 artifact |
| `CreateProjectOperation` | shared validate/stage/import/load/commit/rollback/Ready | Editor/Hub只做surface adapter |
| `ProjectTemplateReceipt` | template/engine/BuildSet/source/migration/evidence identity | project manifest provenance |
| `TemplateCapabilityResolver` | required provider/target/profile admission | Plugins06通用resolver |
| `TemplateQualificationHarness` | create/import/native render/export/migration矩阵 | Tooling03/10/11基础设施 |

目标链为：

`Template Descriptor -> Signed/Embedded Pack -> Validated Create Request -> Staging -> Project/BuildSet/Asset Preflight -> Default Scene Load -> Commit -> Project Ready Receipt -> Native/Export Qualification`。

Hub、Editor CLI、Welcome页和自动化不得各自解释“创建成功”；相同operation receipt是recent project、launch、MVP evidence和support的唯一输入。

## 14. 依赖序里程碑

### M0 · Current Truth 与 Descriptor Freeze

为现有bytes生成`renderable-empty@1` descriptor/root digest，冻结当前项目/asset/export声明；历史证据按digest重新分类，当前compile blocker作为failed lane保留。

### M1 · Shared Create Operation

把Editor与Hub创建收敛为共享operation/transaction，统一path lease、staging、open/import/default-scene preflight、commit、rollback、cancel和Ready receipt。

### M2 · Project Capability 与 Provenance

manifest写入template receipt和minimum provider/target profile；标准Editor/Runtime解析同一BuildSet，required missing fail-close。

### M3 · Source/Meta/Migration Closure

明确seed meta与ready artifact状态、scene meta策略、project-qualified UUID与cross-project remap；建立`@1`到后续版本migration fixture。

### M4 · Native Render 与 Surface Matrix

保留F2 strong oracle，增加shared create到native window/present的Windows/Linux backend矩阵与shutdown/resource gate。

### M5 · Export Qualification

等待Tooling03真实build/cook/pack/install/run后恢复Windows profile；installed root首帧、input、resize、clean exit与PE/dependency/sign全部通过。

### M6 · Product Template Release

Hub/Editor/CLI创建等价，version/upgrade/support/evidence闭合；durable EvidenceSet绑定descriptor、BuildSet、platform和reference image。

## 15. 资格门

1. template qualified ID/version/content root在一次release内不可变。
2. 17个source/embedded entry集合、path、bytes和digest完全一致。
3. 创建project记录template、engine、BuildSet、creator与operation receipt。
4. manifest明确minimum renderer/importer/platform capability，required provider全Ready。
5. Hub、Editor CLI、Welcome和API消费同一个CreateProjectOperation。
6. create concurrent/fault/crash测试证明失败零发布或有typed recovery artifact。
7. Project Ready前全部required asset imported、default scene loaded、references resolved。
8. seed meta到ready registry/artifact的mutation可解释、可再生且绑定digest。
9. project-qualified UUID在clone、copy、merge、migration中不碰撞或有确定remap。
10. F2保留changed-pixel/draw/light/material强oracle并绑定template/BuildSet digest。
11. native runtime从创建项目启动窗口/swapchain，first present、resize/input和clean exit通过。
12. Windows/Linux支持矩阵明确；unsupported target无法选择或返回structured unavailable。
13. export profile只在真实host、valid zrpack、object/dependency/sign/install/run全通过后Enabled。
14. exported game从installed root运行且不读取repo/source cache/Cargo target。
15. template version migration完成backup、dry-run、apply、save/reopen和rollback测试。
16. fixed UUID asset的cross-project copy/merge有scope/remap行为测试。
17. CI结果绑定source/template/toolchain/BuildSet，不允许历史pass覆盖current failure。
18. accepted visual/perf/crash evidence进入durable store而非只保留7天。
19. Hub成功状态至少代表Project Ready；只复制完成必须显示非终态。
20. catalog描述、可用target、export按钮和support状态全部由未过期qualification生成。

## 16. 本轮验证与限制

本轮逐文件读取17个template source、7个Runtime Interface pack文件、Editor/Hub create owner、相关tests和Windows MVP workflow；统计101个非reference consumer文件与28个test文件，检查source reference存在、meta状态、embedded entry列表、manifest plugin/export字段以及F1-F5边界。所有操作为只读审查，没有修改template、production、tests、workflow或artifact。

本轮没有重跑Editor/Workspace compile：当前源码此前已稳定产生239个既存Editor compile error，相关source条件没有变化。没有把历史F1-F5或7天artifact当作current pass，也没有把Tooling03的通用export finding重复计入本篇；App07只登记模板继续暴露该能力的产品truth缺口。
