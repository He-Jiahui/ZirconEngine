---
related_code:
  - tools/cargo-zircon/Cargo.toml
  - tools/cargo-zircon/src/lib.rs
  - tools/cargo-zircon/src/main.rs
  - tools/cargo-zircon/src/plugin/check.rs
  - tools/cargo-zircon/src/plugin/diagnostic.rs
  - tools/cargo-zircon/src/plugin/manifest_sync.rs
  - tools/cargo-zircon/src/plugin/manifest_sync/declaration.rs
  - tools/cargo-zircon/src/plugin/scaffold/mod.rs
  - tools/cargo-zircon/src/plugin/scaffold/templates.rs
  - tools/cargo-zircon/src/plugin/validate.rs
  - tools/cargo-zircon/src/plugin/validate/native_artifact.rs
  - tools/zircon_export/plugin_command.py
  - tools/zircon_export/plugin_validate.py
  - tools/zircon_export/plugin_validate_target_discovery.py
  - zircon_plugins/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests/generated_manifest.rs
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_plugins/plugin_sdk/src/manifest/defaults.rs
  - zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs
  - zircon_app/Cargo.toml
tests:
  - tools/cargo-zircon/tests/manifest_sync.rs
  - tools/cargo-zircon/tests/plugin_commands.rs
  - tools/zircon_export/tests/test_plugin_validate_all_targets.py
  - .github/workflows/ci.yml
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
  - docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md
  - docs/cli-and-tooling/cargo-zircon-plugin-workflow.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/PluginDescriptor.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/PluginDescriptor.cpp
  - dev/godot/editor/plugins/plugin_config_dialog.cpp
  - dev/godot/editor/plugins/plugin_config_dialog.h
  - dev/Fyrox/fyrox-impl/src/plugin/mod.rs
  - dev/bevy/crates/bevy_app/src/plugin.rs
  - dev/Graphics/.yamato/wrench/wrench_config.json
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 02 · Cargo Zircon Plugin Scaffold、Manifest Validation 与 Native Probe 工程化差距

## 1. 结论

`cargo-zircon`不是空命令包装。4,328行Rust实现已经尝试把插件ID、能力、module、distribution和native entry从`declare_plugin!`投影到`plugin.toml`，脚手架能生成runtime/editor/dist crate并修改workspace/catalog/App feature，validator也有稳定diagnostic code与repair hint。这些方向应保留。

但是当前开发入口本身不可用。clean tracked source在`check.rs:70`把`Option<&Path>`传给要求`&Path`的`native_artifact_path`，`cargo test -p cargo-zircon --locked`和真实`cargo run --locked -p cargo-zircon -- plugin check --root .`均以`E0308`失败。即使修掉这个编译错误，生产catalog已经把静态manifest inventory迁到`src/tests/generated_manifest.rs`，而脚手架/check仍硬编码要求`src/lib.rs`内的marker与`include_str!`；所以`plugin new`会立即拒绝当前仓库，`plugin check`会把39个root manifest全部报告为未接入catalog。

native artifact probe还把校验边界反转成安全风险。它在CLI进程中直接`LoadLibrary/dlopen`候选DLL、调用descriptor函数，再把一个没有`struct_size`和字符串长度的外部指针强转为本地ABI结构并执行`CStr::from_ptr`。候选库的loader initializer和descriptor都能执行任意代码，畸形/旧版结构又能让validator越界读、崩溃或进入未定义行为。文档虽然提醒只加载trusted source，却又把`validate <third-party package> --artifact`列为产品入口；工程级validator不能把“信任调用者”当隔离机制。

Python发布validator当前以exit 0通过39个root manifest加2个feature extension，`target_count=41`、`failed_count=0`、`diagnostics=[]`。这证明当前发布manifest并非整体损坏，也证明Rust与Python已经形成两套能力、schema和target口径不同的authority：Rust约4.3K行，Python plugin validator相关production/tests已达139文件、约21,764行。当前计划把Rust工具标为`implementation-complete / validation-pending`已经过时，应回退到`broken / architecture-convergence-required`。

本轮记录3个P0、36个P1和7个P2。没有修改Rust、Python、catalog、manifest、lockfile或CI；export/package的实现质量将在下一份报告继续深审。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/规模 | 本轮状态 |
|---|---:|---|
| `cargo-zircon` production | 12文件 / 3,173行 | E3：CLI、check、diagnostic、sync/parser、scaffold/templates、static/native validate逐文件读取 |
| `cargo-zircon` tests | 2 integration files / 1,155行，加2个inline test | E3：共21个`#[test]`源码读取；实际Cargo运行被production compile P0阻断 |
| First-party catalog/App wiring | 7个核心manifest/source | E3：marker、registration、manifest inventory与feature链 |
| Python validator对照面 | 139 production/test files / 21,764行 | E1-E2：本轮只盘点owner并运行真实`--all`，其内部算法留给export报告 |
| Plugin inventory | 39个root `plugin.toml`、42个含声明macro的Rust文件、139个plugin workspace member | E2：目录/声明/catalog统计；feature extension由Python当前结果确认2个 |

clean scoped set fingerprint为`c36994bad774189c97afd5302f563df7fe204285d714115b9c7aeb0138aba821`。输入覆盖全部`cargo-zircon`文件、两份workflow/plan文档和当前runtime/editor catalog owner；实施前必须重取指纹。

### 2.2 动态验证

以下两个命令均在clean tracked source上失败：

```powershell
cargo test -p cargo-zircon --locked
cargo run --locked -p cargo-zircon -- plugin check --root .
```

共同错误为：

```text
error[E0308]: mismatched types
tools\cargo-zircon\src\plugin\check.rs:70:50
expected `&Path`, found `Option<&Path>`
```

因此21个Rust test没有任何一项开始执行，真实workspace check也没有产生manifest诊断。此失败独立于上一报告的stale plugin lock，因为`cargo-zircon`从root lock构建；也独立于Hub编译P0。

以下发布validator命令通过：

```powershell
python -m tools.zircon_export plugin validate --all --repo-root . --json
```

耗时约33.4秒，输出`target_count=41`、`failed_count=0`、`fatal=false`、`diagnostics=[]`。41项包括39个root plugin package和sound的2个feature extension。这只证明Python静态发布合同通过，不证明Rust声明同步、脚手架、Cargo build、DLL probe、加载或运行时行为。

### 2.3 当前产品链

1. 开发者安装root workspace内的`cargo-zircon`，再调用`cargo zircon plugin ...`。
2. `plugin new`读取六个共享文件，渲染package文件，向plugin workspace、runtime/editor catalog和`zircon_app`写入feature/registration。
3. `sync-manifest`递归寻找`capability.rs`或`lib.rs`中的全限定`zircon_plugin_sdk::declare_plugin!`，用定制`syn::Parse`把部分字段写回TOML。
4. `plugin check`递归寻找39个`plugin.toml`，静态验证字段，再用Cargo TOML与catalog Rust源码字符串做wiring检查。
5. 指定artifact时，check/validate在当前进程加载动态库并读取ABI v3 descriptor。
6. 发布侧另有`python -m tools.zircon_export plugin validate/build`，其target发现、schema和package materialization与Rust工具没有共享model。

## 3. 当前P0

### TOOL-PLUGIN-P0-001 · `cargo-zircon`当前无法编译

`check_plugin_workspace_with_artifact_root`在构造tuple时仍把外层`artifact_root: Option<&Path>`直接传给`native_artifact_path(..., &Path)`，尚未进入`if let Some`解包。所有package test、真实CLI、root workspace build和CI中的`cargo run -p cargo-zircon`都会被同一`E0308`阻断。修复必须补一条`artifact_root=None/Some`行为测试并在CI把`cargo test -p cargo-zircon --locked`设为独立快速门，不能只依赖晚阶段workspace build。

### TOOL-PLUGIN-P0-002 · Scaffold/check与生产catalog拓扑硬漂移

脚手架强制在`first_party_runtime_catalog/src/lib.rs`寻找`@cargo-zircon:static-manifest-end`，check也要求该文件包含每个`../../<package>/plugin.toml`。生产代码已把`STATIC_PLUGIN_MANIFESTS`和marker迁到`src/tests/generated_manifest.rs`，`lib.rs`里0个`include_str!`，而integration fixture仍模拟旧拓扑。编译修复后：

- `plugin new`在任何写入前因缺marker失败；
- `plugin check`对39个manifest各产生`plugin.catalog.manifest.missing`；
- 测试继续对旧fixture全绿，无法发现真实仓库失配。

这不是小型文档漂移，而是两条公开developer workflow全部不可用。应先定义catalog manifest inventory究竟是production authority、generated test fixture还是build-time artifact，再让生成器消费typed owner，而不是移动marker。

### TOOL-PLUGIN-P0-003 · Native validator在同进程执行并无界解引用候选DLL

`validate_native_artifact`直接加载路径并调用用户指定descriptor symbol。加载动作本身会执行DLL initializer；descriptor函数同样是任意native代码。随后本地代码假定返回值完整匹配`NativePluginAbiV3Projection`，在没有`struct_size`、data model、build ID或pointer/length pair的情况下读取五个外部指针并用`CStr::from_ptr`寻找未知位置的NUL。旧版、恶意或损坏artifact可以在“验证”阶段终止/控制CLI进程。

最低重构要求是先用object-file parser做非执行preflight，再在受限child process中加载；parent设置timeout、memory/output budget并把crash/timeout转为typed result。ABI carrier必须有size/version、显式长度、最大预算和安全copy协议。该P0与Plugin 01报告的native ABI P0同源，但这里的owner是开发者校验进程与不可信artifact admission。

## 4. Authority、Schema 与 Target Model 差距

### TOOL-PLUGIN-P1-001 · Rust/Python/structure audit形成三套validator authority

Rust validator、Python `zircon_export plugin validate`和`audit_plugin_structure.py`分别维护字段集合、枚举、target discovery、diagnostic与测试。CI同时运行其中部分，但没有共享schema IR或conformance corpus。应由版本化plugin schema生成解析/校验模型，语言实现只扩展各自owner的行为门。

### TOOL-PLUGIN-P1-002 · Rust target inventory漏掉feature extension

Rust只递归root `plugin.toml`，声明发现又要求crate parent直属含该manifest；sound的两个feature extension因此不进入sync/check/native probe。Python当前识别41项。所有工具必须输出同一`PackageTargetId { package, feature?, form, target }`集合，并对集合diff失败。

### TOOL-PLUGIN-P1-003 · 没有显式manifest schema version与migration graph

`sdk_api_version`和engine compatibility不能代替file schema version。当前validator不知道某字段是未知未来扩展、拼写错误还是旧schema；sync也没有from/to migration。应单独版本化descriptor schema，保留可逆/有损migration诊断与最低/最高reader版本。

### TOOL-PLUGIN-P1-004 · Workspace check使用无界递归文件扫描

`collect_plugin_manifests`与declaration discovery递归`read_dir`，只按basename跳过`target/.git`，没有symlink/reparse containment、cycle detection、file/depth/count/byte budget或取消。对第三方树和大型monorepo会产生escape、DoS或不可预测延迟。应从canonical package inventory开始，并对显式content roots使用有预算walker。

### TOOL-PLUGIN-P1-005 · Catalog correctness依赖Rust源码字符串匹配

registration通过查找`zircon_plugin_...::plugin_registration()`文本与相邻`#[cfg(feature=...)]`substring推断；改成import alias、格式化、多行、helper或generated table都会误报/漏报。Catalog应由同一typed registry数据生成Cargo feature和Rust dispatch，校验编译后的descriptor inventory而不是源码词法形状。

### TOOL-PLUGIN-P1-006 · Artifact定位只支持当前host的平面profile目录

`native_artifact_path`根据当前validator OS拼`.dll/.so/.dylib`并在单一root下找文件，不包含target triple、profile、crate target name、hash、architecture、feature form或per-plugin target目录。它无法验证cross target，也与Python plugin build的per-plugin `.target/<package>`布局不一致。

### TOOL-PLUGIN-P1-007 · Native probe缺少静态格式与依赖preflight

加载前不检查PE/ELF/Mach-O格式、machine architecture、imports/RPATH、required OS version、code signature、hash、Build Set ID或descriptor section。错误只能由OS loader字符串返回，既不稳定也可能先执行代码。应先静态解析并把兼容性差异变成结构化诊断。

### TOOL-PLUGIN-P1-008 · Native probe无进程、时间和资源隔离

即使artifact来自可信source，buggy initializer/descriptor也可能hang、abort、分配无界内存或写stdout/stderr。当前没有child process、timeout、job object/cgroup、signal/exit classification和bounded log。发布门不得让一个插件杀死整个validation batch。

### TOOL-PLUGIN-P1-009 · Rust CLI没有机器可读结果

诊断结构只有`code/message/hint`，CLI只逐条写stderr；没有JSON/schema version、path/span、severity、package/target、causes、run ID或summary。CI、Editor、Hub和IDE无法稳定消费。应共享diagnostic envelope，同时保留human renderer。

### TOOL-PLUGIN-P1-010 · 手写参数解析缺少成熟CLI合同

`main.rs`手动遍历`Vec<String>`，没有`--help`、`--version`、shell completion、response file、`--color`、verbosity、明确冲突/required group和稳定usage tests。错误统一exit 2，drift 3、validation 4却没有版本化exit taxonomy。可采用成熟parser，但重点是声明式command schema与测试，而非库名称。

### TOOL-PLUGIN-P1-011 · Plugin build位于另一套Python命令面

Rust入口有new/check/validate/sync，真正`plugin build`却由`python -m tools.zircon_export`拥有。用户不能从同一tool discovery、config、diagnostic、Build Set和receipt完成new→check→build→package。应收敛到单一frontend和shared service model；不要求把所有实现重写成一种语言。

## 5. Scaffold 工程化差距

### TOOL-PLUGIN-P1-012 · 六个共享文件与package tree不是原子事务

`plugin new`依次创建package文件，再直接覆盖workspace、两个catalog Cargo、两个catalog source与App Cargo。失败后删除package并尝试回写原文，但所有rollback error被丢弃；process crash、断电或磁盘满可留下半接线状态。需要journal/staging、全部preflight、原子replace和可恢复commit marker。

### TOOL-PLUGIN-P1-013 · 并发scaffold会丢更新

两个进程可同时读取相同original、各自生成updated，再后写覆盖前写；package existence check也不是reservation。应取得repository-scoped writer lease，提交前做compare-and-swap source hash，冲突返回可重试plan。

### TOOL-PLUGIN-P1-014 · 没有dry-run、diff、resume或repair

开发者无法预览将修改的共享文件、选择是否加入base profile、在中断后恢复或对已有半生成package执行repair。工程入口应先产生typed mutation plan与diff，再显式commit；repair读取journal而不是猜测当前文本。

### TOOL-PLUGIN-P1-015 · Rust catalog仍靠marker注入源码

Cargo TOML使用`toml_edit`尚能保留大部分格式，Rust source却在第一个marker前插入字符串。marker重复、移动、条件编译层级变化或函数重构都可能生成语法正确但语义错误的代码。应生成独立数据模块或build-time registry，不修改手写函数体。

### TOOL-PLUGIN-P1-016 · 新runtime插件无条件进入`base-runtime-plugins`

`wire_runtime_catalog_cargo`把每个system/importer feature追加到base集合，没有显式profile/size/licensing/platform/experimental决策。新实验插件可能自动进入默认产品闭包。Scaffold应声明profile admission intent，默认保持未启用，并由产品profile owner审批。

### TOOL-PLUGIN-P1-017 · System skeleton注册成功但没有任何系统

生成的`RuntimePlugin::register`直接`Ok(())`。它能通过manifest/catalog结构检查，却没有system/service/event/resource行为。脚手架可以生成TODO skeleton，但check/文档必须把`structurally generated`与`behavior ready`分开，并提供至少一个可执行sample system或强制删除placeholder capability。

### TOOL-PLUGIN-P1-018 · Editor skeleton同样发布空能力

`register_editor_extensions`固定`Ok(())`，却在descriptor与catalog宣称editor capability。Editor会看到可启用插件而没有command/pane/importer/settings贡献。需要capability-to-contribution completeness gate，或生成最小真实extension示例。

### TOOL-PLUGIN-P1-019 · Importer skeleton只有泛化Data descriptor

所有importer都固定`AssetKind::Data`、priority 100、扩展名等于plugin ID，并只注册descriptor；没有import执行器、settings schema、source sniffing、derived artifact、reimport或error contract。它是metadata skeleton，不是可用importer，文档的“三步可加载”不能升级为“能导入”。

### TOOL-PLUGIN-P1-020 · Native skeleton固定生成空行为表

模板写死`systems=[]`、`events=[]`、`bridge_methods=[]`、command/event manifest为空，`invoke_command/save_state/restore_state/unload/on_host_ready=None`且`is_stateless=true`。这会批量制造Plugin 01报告已识别的native metadata shell。Native模板必须按kind生成真实behavior contract，或默认不宣称native form。

### TOOL-PLUGIN-P1-021 · 没有真实generated package E2E

integration test只在简化fixture中检查文件与substring，不运行生成crate的Cargo metadata/check/test，不加载artifact，也不验证Runtime/Editor能发现和调用贡献。已有计划明确承认E2E pending；当前编译和catalog漂移正是该缺口的结果。

## 6. Manifest Projection 与 Static Validation 差距

### TOOL-PLUGIN-P1-022 · `sync-manifest`直接覆盖文件且丢格式/注释

同步使用`toml::Value`反序列化后`to_string_pretty`，再`fs::write`原路径；它没有atomic replace、fsync、backup、file lock或source hash CAS，并会重排/删除手写comment。生成物仍需确定性，但应先写临时文件、验证、原子交换并保留extension owner策略。

### TOOL-PLUGIN-P1-023 · Declaration discovery依赖固定文件名和完整宏路径

只扫描`capability.rs/lib.rs`，再用精确字符串`zircon_plugin_sdk::declare_plugin!`预筛。合法的`use ...::declare_plugin; declare_plugin!`、其他owner文件或宏格式变化会被静默跳过；反之comment/string命中后才parse。应从Cargo target/module或显式declaration path定位，并由parser确认唯一owner。

### TOOL-PLUGIN-P1-024 · Declaration parser要求字段固定顺序

定制`Parse`逐字段期待`id→display_name→category→...`，不支持可交换顺序、可选字段、default、deprecated alias或unknown future field。SDK宏本身一旦演进，工具必须同步发版，否则旧CLI无法读新声明。需要版本化IR与name-based parser，并有向前/向后兼容测试。

### TOOL-PLUGIN-P1-025 · `module_description`被解析后立即丢弃

parser消费该字符串却不存入`PluginDeclarationProjection`，同步器也不更新root/module description。修改Rust声明中的描述不会进入`plugin.toml`，直接反驳“Rust声明是manifest authority”。每个声明字段必须有projection destination或明确标记runtime-only，禁止silent discard。

### TOOL-PLUGIN-P1-026 · Native projection除entry外被当作opaque token跳过

parser只读取runtime/editor entry，随后把registration manifest、modules、systems、events、extensions逐token吞掉。同步器因此无法证明native capability/behavior与TOML一致。需要共享native ABI schema IR，不应由源码token跳过与另一套Python validator分别推断。

### TOOL-PLUGIN-P1-027 · First-party plugin版本被强制等于engine workspace版本

每次同步都把`version`覆盖为root `workspace.package.version`，engine compatibility也从同一版本生成。这样无法表达插件独立release、backport、security fix、marketplace promotion或同一engine上的多插件版本。First-party lockstep可作为policy，但package version和engine range必须是不同字段与owner。

### TOOL-PLUGIN-P1-028 · Rust validator允许未知字段且没有migration诊断

root/module/distribution都从开放TOML table取已知字段，拼写错误和废弃字段通常被保留并通过。发布validator已有更严格shape owner，Rust开发门却不能提前报错。应共享allowed/extension namespaces和schema version，第三方自定义字段必须进入明确extension map。

### TOOL-PLUGIN-P1-029 · SemVer与engine range校验过度简化

`valid_semver`只接受三个纯数字分量，拒绝合法prerelease/build metadata；`engine_compat`则只检查非空，根本不parse range。需要标准SemVer/range库、canonical rendering、pre-release policy和compatibility test corpus。

### TOOL-PLUGIN-P1-030 · Distribution合同只做浅层存在性检查

Rust侧不闭合验证`forms`、distribution `default_packaging`、descriptor symbol固定值、engine range、entry identifier、platform/target与form对应关系；Python为这些字段维护大量独立owner。开发门与发布门应消费同一distribution schema，并分离static、build、load三阶段证据。

### TOOL-PLUGIN-P1-031 · Module与Cargo package勾稽不完整

除dist crate外，validator主要检查crate name字符串形状，不用Cargo metadata证明package存在、target kind/feature正确、依赖边界和crate-type匹配。`package_contains_crate`只尝试四个固定子目录并吞掉IO/TOML错误。应通过canonical workspace/package graph验证每个module target。

### TOOL-PLUGIN-P1-032 · Rust check没有全局插件图验证

没有跨manifest重复ID/capability/interface/event namespace、dependency cycle/version range、optional feature provider、asset root冲突或target closure检查。Python当前覆盖其中一部分，但结果没有回流Rust developer diagnostics。共享graph pass应在单包与全仓两种模式复用。

## 7. Developer Workflow 与 Test Architecture 差距

### TOOL-PLUGIN-P1-033 · 缺少remove/rename/upgrade/migrate工作流

CLI只能新增和同步，不能安全移除catalog/workspace/App wiring、重命名ID/crate/capability、升级SDK/schema或迁移旧package。长期工程中这些操作比new更危险，也更需要transaction plan。

### TOOL-PLUGIN-P1-034 · Scaffold硬编码first-party monorepo布局

`new`固定写`repo_root/zircon_plugins`、两个first-party catalog和`zircon_app`，不能创建project-local、external vendor或独立SDK仓库插件。应把package template与first-party admission分成两个命令/阶段，外部插件不应修改engine source。

### TOOL-PLUGIN-P1-035 · 21个Rust测试没有覆盖CLI进程

测试直接调用library API，`main.rs::run`私有且无test。`cargo zircon`前缀剥离、usage、unknown command、flag冲突、exit 2/3/4、stdout/stderr和working directory均未验证。需要process-level CLI snapshot与argv matrix。

### TOOL-PLUGIN-P1-036 · Native tests没有真实或畸形artifact corpus

integration只检查missing file，inline test只给本进程合法`CString`测试entry name；没有真实PE/ELF/Mach-O、wrong arch、missing dependency、old struct、null/unterminated pointer、crashing/hanging descriptor、signature/hash或child containment测试。P0隔离重构必须以故障artifact corpus驱动。

## 8. 次要差距

### TOOL-PLUGIN-P2-001 · 测试临时目录不是RAII资源

测试用时间纳秒拼`temp_dir`并手工`remove_dir_all`；panic会残留，极端并发也没有exclusive create。采用RAII tempdir并不解决事务语义，但能让故障注入测试可靠。

### TOOL-PLUGIN-P2-002 · “禁止panic shortcut”测试锁源码形状

较大测试visitor扫描production Rust是否出现unwrap/expect/panic，却不能证明IO、ABI和rollback在故障下安全，也不检查生成后的代码行为。保留lint即可，测试预算应转向transaction/native/CLI行为。

### TOOL-PLUGIN-P2-003 · 权威计划状态与当前事实矛盾

`frameworks/04`仍写`implementation-complete / validation-pending`，workflow文档也描述可用五分钟路径；当前编译P0与catalog漂移说明状态必须回退。计划状态应由fresh gate receipt更新，而不是手工长期保留。

### TOOL-PLUGIN-P2-004 · 文档中的39 target锚点已经过期

当前Python validator是41项。历史文档多次把39写成权威数量，新增feature extension后没有统一生成。文档应引用inventory artifact或按类型展示，不硬编码易漂移总数。

### TOOL-PLUGIN-P2-005 · CLI无规模预算或性能基线

没有39/1K/10K package下scan、parse、sync和graph validation的时间/内存预算，也没有增量cache。Python当前全量静态验证已需约33秒；未来Marketplace规模必须避免每次命令全仓重扫。

### TOOL-PLUGIN-P2-006 · 诊断只有英文字符串renderer

code可保留稳定机器身份，但message/hint没有参数化payload或localization key，Editor/Hub只能显示CLI拼好的文本。共享diagnostic应携带typed context，由各surface负责本地化与交互修复。

### TOOL-PLUGIN-P2-007 · Schema、示例和IDE元数据不能从同一来源生成

没有JSON Schema/TOML schema、completion、hover docs、example manifest和migration guide生成。字段在Rust/Python/docs多处手写，开发者只能运行多个validator试错。

## 9. 参考引擎对照

### 9.1 Unreal：descriptor是版本化产品模型

`FPluginDescriptor`区分machine version与用户可见`VersionName`，拥有creator/docs/marketplace/support、engine compatibility、modules、localization、dependencies/disallowed plugins、supported platforms、content/code/sealed/explicit load、pre/post build等字段；descriptor reader还显式维护file version。Zircon不必复制所有字段，但需要同样清晰地区分schema version、package version、engine range和模块/平台/内容策略。

### 9.2 Godot：创建入口包含即时校验和语言模板

`PluginConfigDialog`在创建前验证名称、目录、脚本扩展和已存在路径，按当前script language生成真实`EditorPlugin`模板，保存后刷新Project Settings与Editor filesystem。它的事务能力并不完美，但产品入口至少把用户输入、模板能力和Editor刷新连成一条可见workflow；Zircon当前CLI既不能构建，也没有Editor/Hub集成或repair surface。

### 9.3 Fyrox与Bevy：插件模板必须落到真实生命周期

Fyrox Plugin暴露register/init/on_loaded/on_deinit/update与dynamic prepare/reload状态路径；Bevy Plugin有build/ready/finish/cleanup并由App执行。Zircon scaffold生成的system/editor/native capability没有对应行为或lifecycle smoke，所以“manifest有效”不能被等同为“插件有效”。

### 9.4 Unity Graphics：package validation连接版本矩阵与promotion

Wrench/Yamato把package catalog、schema、Editor/Playmode版本矩阵、pack和promotion作为依赖链。Zircon Python发布层已经比Rust入口覆盖更多字段，但两者没有共享Build Set/package receipt，尚不能形成同类promotion authority。

## 10. 重构路线

### M0 · 恢复诚实的开发入口

1. 修复compile P0，并为`None/Some artifact_root`补行为测试。
2. 将计划状态回退为broken，CI独立执行package test和真实CLI smoke。
3. 选择catalog inventory的canonical owner，消除旧fixture与production拓扑差异。
4. 在当前39 root/41 target上对Rust/Python inventory做diff gate。
5. native artifact选项在隔离实现完成前明确禁用或标为unsafe trusted-only，不进入第三方admission。

### M1 · 统一Plugin Schema IR

1. 定义manifest schema version、package/engine/SDK版本、module/form/feature target ID。
2. 由同一IR生成Rust/Python parser、TOML/JSON schema、diagnostic payload和docs。
3. 建立valid/invalid/migration corpus，两种实现必须产生同一code/path/context。
4. 让Rust声明每个字段明确标注projection owner，删除silent discard和opaque token skip。

### M2 · 事务化Scaffold与Catalog生成

1. package template与first-party admission分离，支持project/external仓库。
2. 所有mutation先生成plan/diff，取得writer lease并对source hash做CAS。
3. staging/journal/atomic replace提交workspace、catalog、App和package tree。
4. Catalog由typed registry生成，不在手写Rust函数中插marker。
5. 新plugin默认不进入base profile；profile admission单独审批。

### M3 · 行为完整的模板与E2E

1. System生成可执行sample system，Editor生成最小真实contribution，Importer生成backend contract。
2. Native模板按kind生成behavior/lifecycle或不宣称native form。
3. 每种模板执行new→metadata→check→build→package→load→invoke→shutdown→remove。
4. Windows/Linux/macOS至少各验证一个真实artifact，故障corpus覆盖wrong arch/crash/hang/old ABI。

### M4 · 安全Native Admission

1. object parser静态读取format/arch/import/export/signature/hash和只读metadata section。
2. ABI加入struct size、Build Set/SDK/schema identity、pointer-length预算。
3. 动态probe移入受限child，parent拥有timeout、kill tree、memory/output budget和crash report。
4. 发布层只接受静态+隔离probe receipt，不重复加载未绑定artifact。

### M5 · 单一Developer/Release Frontend

1. new/check/sync/migrate/build/package/validate/publish共享command schema和diagnostic。
2. Rust/Python可继续分工，但frontend只暴露一个配置、Build Set与receipt体系。
3. Editor/Hub/IDE消费structured result，不解析stdout。
4. 规模测试覆盖39、1K、10K target及增量变化，建立时间/内存budget。

## 11. 验收门

1. `cargo test -p cargo-zircon --locked`在独立CI job通过，production compile错误在60秒内被发现。
2. 真实`cargo zircon plugin check --root .`对当前仓库exit 0，不依赖旧catalog fixture。
3. Rust与Python对root/feature/form target集合逐项一致；当前应识别41项而非只认39 manifest。
4. Plugin schema有独立file version、package version、engine range和SDK API version。
5. Rust/Python validator共享conformance corpus，code/path/context一致。
6. Unknown/deprecated/extension字段有显式策略和migration诊断。
7. `module_description`等每个声明字段都有projection destination或明确runtime-only标记。
8. Native systems/events/extensions/entries由typed IR解析，不跳过token。
9. Scaffold不直接修改六个共享文件；所有更改经plan、lease、CAS、journal和atomic commit。
10. 进程崩溃、磁盘满、权限失败与并发new不会留下半生成package或丢更新。
11. 新plugin默认不进入base product profile。
12. System/Editor/Importer模板各有一个真实行为，并由产品host E2E调用。
13. Generated package实际通过Cargo metadata/check/test，而非只做substring断言。
14. CLI支持help/version/structured JSON、稳定exit taxonomy和process-level argv tests。
15. Plugin build与validate消费同一个target ID、Build Set和artifact receipt。
16. Static artifact preflight在任何代码执行前验证format/arch/import/export/hash/signature。
17. 动态probe只在受限child运行，crash/hang/OOM/超量输出被parent分类并回收。
18. ABI descriptor有struct size和所有字符串长度/最大预算，validator不做无界`CStr::from_ptr`。
19. Cross-manifest重复ID/capability/interface/event、dependency cycle和feature provider closure有全局graph gate。
20. remove/rename/upgrade/migrate与new使用同一事务模型并有恢复测试。
21. Project-local/external插件不要求修改engine first-party catalog和App manifest。
22. 计划状态与文档target数量由fresh receipt生成，不能保留过期“implementation-complete”。

## 12. 后续边界

本报告只深审Rust开发入口及其与当前catalog/Python validator的交界。下一份报告继续读取`tools/zircon_export`完整pipeline、plugin build/package/signature、platform bundle、Editor export调用点和CI artifact真实性；不会因为Python `validate --all`通过就把build/package/install/promotion视为完成。
