---
related_code:
  - zircon_runtime/src/script
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_plugins/zr_vm_language/runtime/Cargo.toml
  - zircon_plugins/zr_vm_language/runtime/src/real_backend
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/package.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/runtime_owner.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/lock.rs
  - zircon_plugins/zr_vm_language/runtime/src/backend.rs
  - zircon_plugins/zr_vm_language/dist/Cargo.toml
  - zircon_plugins/Cargo.toml
  - .github/workflows/ci.yml
  - examples/woc/scripts/woc_game
  - examples/woc/scripts/woc_game/woc_game.zrp
  - examples/woc/scripts/woc_game/bin/.zr_cli_manifest
  - examples/woc/scripts/woc_game/bin/main.zro
  - examples/woc/scripts/woc_game/woc_m4_power_echo_heal_state_tests.zrp
  - examples/woc/tools/m4_power_echo_heal_source_check.mjs
  - examples/vampire/scripts/vampire_game
  - examples/vampire/scripts/vampire_game/plugin.zrp
  - examples/vampire/scripts/vampire_game/bin/.zr_cli_manifest
  - ../zr_vm/CMakeLists.txt
  - ../zr_vm/.github/workflows/cmake-multi-platform.yml
  - ../zr_vm/zr_vm_common/include/zr_vm_common/zr_io_conf.h
  - ../zr_vm/zr_vm_common/include/zr_vm_common/zr_version_info.h
  - ../zr_vm/zr_vm_parser/src/zr_vm_parser/writer/writer_binary.c
  - ../zr_vm/zr_vm_parser/src/zr_vm_parser/type_system.c
  - ../zr_vm/zr_vm_parser/src/zr_vm_parser/compiler/compile_expression.c
  - ../zr_vm/zr_vm_parser/src/zr_vm_parser/parser/parser_ast_free.c
  - ../zr_vm/zr_vm_core/src/zr_vm_core/io.c
  - ../zr_vm/zr_vm_core/src/zr_vm_core/constant_reference.c
  - ../zr_vm/zr_vm_core/src/zr_vm_core/execution/execution_dispatch.c
  - ../zr_vm/zr_vm_core/src/zr_vm_core/gc/gc_cycle.c
  - ../zr_vm/zr_vm_core/src/zr_vm_core/global.c
  - ../zr_vm/zr_vm_cli/src/zr_vm_cli/compiler/compiler.c
  - ../zr_vm/zr_vm_cli/src/zr_vm_cli/project/project.c
  - ../zr_vm/zr_vm_library/src/zr_vm_library/project/project.c
  - ../zr_vm/zr_vm_library/src/zr_vm_library/project/project_manifest_v2.c
  - ../zr_vm/zr_vm_language_server/src/zr_vm_language_server/incremental_parser.c
  - ../zr_vm/zr_vm_rust_binding/include/zr_vm_rust_binding.h
  - ../zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs
  - ../zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding_sys/build.rs
tests:
  - zircon_runtime/src/script/vm/tests
  - zircon_plugins/zr_vm_language/runtime/src/tests
  - ../zr_vm/tests/CMakeLists.txt
  - ../zr_vm/tests/core
  - ../zr_vm/tests/parser
  - ../zr_vm/tests/language_server
  - ../zr_vm/zr_vm_aot/tests
  - ../zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/12-woc-zrvm-package-kernel-world-state-schedule-serialization-runtime-review.md
  - docs/plans/optimize/zircon_editor/31-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-authoring-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
  - docs/plans/optimize/zircon_tooling/05-woc-content-codegen-build-scripts-generated-artifact-incremental-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/10-test-architecture-partition-selection-isolation-fixture-flake-results-review.md
  - docs/plans/optimize/zircon_tooling/11-woc-parity-oracle-trace-golden-differential-replay-evidence-review.md
  - docs/plans/optimize/zircon_tooling/17-repository-content-source-set-ignore-generated-vendor-license-distribution-review.md
reference_engines:
  - dev/godot/modules/gdscript/gdscript_cache.cpp
  - dev/godot/modules/gdscript/gdscript_analyzer.cpp
  - dev/godot/modules/gdscript/gdscript_byte_codegen.cpp
  - dev/godot/modules/gdscript/gdscript_tokenizer_buffer.cpp
  - dev/godot/modules/gdscript/gdscript_tokenizer_buffer.h
  - dev/godot/modules/gdscript/tests/test_gdscript.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/VerseCompiler/Private/uLang/Toolchain/ProgramBuildManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/VerseVM/VVMBytecodeAnalysis.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/VerseVM/VVMBytecode.h
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
  - dev/bevy/crates/bevy_reflect/src/serde/de/deserializer.rs
  - dev/Fyrox/fyrox-impl/src/script/mod.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 21 · Zr Language Parser、Type System、SemIR、Bytecode、Package Loader、VM Runtime 工程化差距

## 1. 结论

Zr语言实现不是一个临时解释器壳。当前外部`zr_vm`仓库包含完整的parser、type inference、compiler、SemIR、interpreter、AOT C/LLVM、LSP、CLI、project resolver、Rust binding和大规模测试体系。本轮静态快照中，`zr_vm_parser`有382个C/H/Rust源码文件、约204,239行；`zr_vm_core`有200个、约109,481行；`zr_vm_aot`有227个、约105,666行；language server有162个、约77,621行。外部`tests/CMakeLists.txt`有8,040行、128个`add_test`和10个`add_executable`，独立CI在Windows Debug、Linux GCC Release和Linux Clang Release运行CTest。compile-time executor已有fuel、call-depth、heap、aggregate、generated declaration与diagnostic预算；artifact signature parser有深度/计数限制；comptime cache使用临时文件加rename；import dirty会向依赖者传递；Rust native callback trampoline会捕获unwind并保持host argument root。这些都是真实基础，不能在重构时删除或退回简单实现。

但Zircon产品并没有消费一份可复现、可验证、可治理的Zr语言工具链。`backend-zr-vm`通过`../../../../zr_vm/...`依赖工作区外的兄弟仓库，Cargo lock无法固定其C源码commit、submodule、CMake选项或动态库。当前外部仓库位于commit `8a843bdd7a5aadbbf2deac7242a825cf64c084c8`，且parser、type system、LSP和Rust binding有大量未提交修改；本报告只记录这一份in-flight source snapshot，不能把结果外推为clean-head验收。Zircon required CI只对dist包执行`--no-default-features --features dist`，没有启用`backend-zr-vm`、checkout该源码、构建CMake、设置`ZR_VM_RUST_BINDING_LIB_DIR`或运行外部CTest。因此“dist包能构建”与“真实语言后端可从clean clone构建并运行”是两件事。

更严重的是，当前`.zro`写入器直接打开最终路径并连续`fwrite`host-native整数；大部分写入与`fclose`结果不检查。读取器虽然读出signature、major/minor/patch、native width、instruction width和endian字段，却不验证signature、major/minor、width或endian，也没有转换；源码仍写着`todo: different endianness`和`todo: check signature`。文件控制的长度会直接进入递归分配和`sizeof * length`，没有统一总字节、节点数、深度、乘法溢出或wall-time budget；`ZrCore_Io_ReadSourceFree`还是空函数。header也没有payload length、checksum、compiler/build/options/target/host ABI identity。损坏、截断、跨平台或过期bytecode可以进入allocator、对象图和执行器，而不是在admission阶段被拒绝。

编译器发布链同样不闭合。incremental manifest仅用64-bit FNV-1a source/zro hash与imports判断复用，没有把compiler commit/version、format、options、target、host ABI、dependency lock和native surface纳入key。dirty module逐个直接写最终`.zro/.zri/AOT C`，之后再写cache与非原子manifest，没有generation staging、fsync、整批commit或rollback。仓内709个`.zr`、38个`.zro`、277个`.zrp`没有唯一`ProductScriptBuildSet`：两份tracked manifest共登记7个`.zro`，同时列出14个不存在的`.zri/AOT C`绝对路径；其余31个`.zro`不在这两份manifest中。276份JSON `.zrp`都只有`name/source/binary/entry`四字段并走legacy v1语义；另有一份82字节的TOML-like测试文本，只被Node源码检查读取，不是CLI可执行project。当前资产集合无法回答“哪一份source由哪一版compiler、按什么target/options/ABI生成了哪一组可安装artifact”。

最后，语言语义内部仍存在会静默改变程序含义的临时实现：type system允许所有整数类型互转且共同类型固定`int64`；复杂generic/tuple cast默认发出`TO_STRING`；`INT16`提升没有对应指令；constant reference把部分路径“暂时假设为最终结果”并按上下文假定child function；runtime struct转换找不到prototype时直接复制对象；global `zr`对象创建后仍标记“后续注册”；GC在发现PIC slot count或slot pattern损坏时清空缓存自愈。它们跨越type checker、codegen、object model、inline cache与GC，不能作为独立TODO逐个打补丁，必须由一份canonical type/layout/opcode contract共同修复。

本篇拥有Zr source/project/module语义、parser/type system/SemIR/codegen、`.zro`格式与verifier、compiler artifact transaction、VM执行预算及interp/binary/AOT语义一致性。Runtime07继续拥有package/plugin lifecycle、hot reload和process-wide backend并发；Plugins01拥有foreign ABI/package trust；Editor31拥有code editor/LSP/debugger产品；Tooling05/10/11/17分别拥有通用codegen、测试架构、parity evidence和SourceSet；App03拥有WOC产品host。本轮登记 **4项P0、60项P1和14项P2**。

## 2. 审查边界与来源状态

### 2.1 当前物理覆盖

| 范围 | 文件/规模 | 本轮读取重点 |
|---|---:|---|
| `zircon_runtime/src/script` | 102个Rust文件，约17,307行，653,951 bytes | VM public contract、package discovery、manager、hot reload、GC、tests |
| `zr_vm_common` | 28个C/H/Rust文件，约3,023行 | version、format、IO constants、sentinel |
| `zr_vm_parser` | 382个，约204,239行 | lexer/parser、type inference/type system、compiler、SemIR、binary writer |
| `zr_vm_core` | 200个，约109,481行 | binary reader、object/value、execution dispatch、GC、module/global |
| `zr_vm_cli` | 38个，约12,563行 | project、module closure、incremental compile、manifest、runtime |
| `zr_vm_aot` | 227个，约105,666行 | C/LLVM backend与cross-backend tests |
| `zr_vm_language_server` | 162个，约77,621行 | incremental parser、semantic query、diagnostics |
| Rust binding | 13个，约9,656行 | Runtime/Compile/Run options、FFI ownership、native callback |
| external tests | 8,040行CMake，128个CTest registration | core/parser/AOT/LSP/binding测试可达性 |
| Zircon脚本资产 | 709 `.zr` / 38 `.zro` / 277 `.zrp` / 0 `.zri` | source-project-artifact closure与portable manifest |

本轮不是按TODO数量定性。外部production源码中约有88个TODO/todo命中，其中很多只是演进说明；报告只登记能沿caller、artifact或runtime状态证明影响的项目。相反，没有TODO但会静默接受错误输入的signature/version/endian/length读取被列为更高严重度。

### 2.2 外部源码不是Zircon可复现输入

`E:/Git/zr_vm`是独立Git仓库，当前branch为`main`，HEAD为`8a843bdd...`。它的docs、LSP、parser/type inference/type system、Rust binding和native argument view存在未提交或未跟踪修改；Zircon工作树不会记录这些bytes。后续实施前必须重新固定commit、submodule和dirty state，本篇所有外部源码finding均为`source_recheck_required: true`。

外部根CMake声明project version `0.0.1`，公共VM version当前为`0.0.25`，Rust binding crates为`0.1.0`，IO format patch为41。CMake默认启用shared、network、debug、thread、language server、Rust binding、CLI和tests；Windows使用`/W4`，Unix使用`-Wall -Wextra -Wpedantic`但没有`-Werror`。这些独立机制存在，却没有一份被Zircon消费的toolchain identity。

### 2.3 动态验证边界

本轮没有在dirty外部源码上重跑CMake/CTest或real backend，也没有把历史build目录当current evidence。Zircon当前还存在其他session正在修改的源码，完整Editor、Hub与WOC lanes已有独立已知阻断；重复运行不能提高本篇静态结论。实施验收必须从固定clean source、空build/cache目录开始，产生可引用的BuildSet和test receipt。

## 3. 当前可保留的工程基础

1. parser、type inference、compiler、SemIR、interpreter、AOT和LSP已经分层，重构应收敛合同而不是重新写一个小解释器。
2. project manifest v2已验证required fields、package/export、dependency source/version、alias与重复项；dependency lock已有读写与解析基础。
3. compile-time executor已有fuel、call depth、heap、aggregate、generated declaration和diagnostic预算，证明预算模型可以进入公共编译合同。
4. artifact metadata signature parser已有depth/count validation，AOT descriptor会验证backend/input/export/token/remap等结构。
5. import dirty能沿依赖反向传播，适合作为canonical dependency graph的起点。
6. comptime cache采用临时文件和rename，说明原子发布模式已有局部实现可复用。
7. Rust callback trampoline捕获unwind，host arguments保持root，session/registration/runtime有显式drop顺序。
8. Runtime option已有heap limit、GC pause/remark budget与worker count，GC治理不是从零开始。
9. external CI跨Windows/Linux及MSVC/GCC/Clang运行CTest；测试体量足以承接新的corruption/conformance matrix。
10. Zircon package discovery已有深度、条目、字节、wall-time和cancellation预算，可作为project/source admission外层。

这些正向机制目前属于不同仓库、不同入口或不同阶段，不能单独生成Zircon产品资格，但应作为新架构的迁移输入。

## 4. 参考实现给出的工程边界

### 4.1 Godot GDScript

`GDScriptParserRef::raise_status()`把处理分成`PARSED -> INHERITANCE_SOLVED -> INTERFACE_SOLVED -> FULLY_SOLVED`单调阶段；cache记录正向与反向依赖，并在owner变化时递归移除依赖parser。analyzer在type仍处于resolving或继承链回到自身时产生明确cyclic diagnostic。Zircon不需要复制Godot类结构，但应拥有同样明确的phase state、cycle result和反向失效，而不是由runtime module lookup和manifest imports共同猜测。

Godot binary tokenizer先要求至少12 bytes，检查`GDSC` magic和精确`TOKENIZER_VERSION`，再读取decompressed size并要求decompress结果完全相等。它仍不是完整的敌对bytecode verifier，且测试runner中的`TEST_BYTECODE`当前也写着`Not implemented`；因此本篇只把它作为最小header/admission基线，不把参考引擎理想化。

### 4.2 Unreal VerseCompiler / VerseVM

`ProgramBuildManager`通过可替换的Parser、SemanticAnalyzer、IR generator、post filters、Assembler和Linker pass构建project，并保留统一semantic program context。VerseVM的`MakeBytecodeCFG()`枚举所有jump/unwind target，要求新增branch opcode同步analysis，构建basic block并验证每条incoming edge拥有相同failure-context stack，同时映射task/yield结构。这给Zircon的核心启示是：opcode schema、decoder、CFG verifier、stack/type verifier和executor必须由同一生成源驱动，未知或结构非法bytecode要在执行前拒绝。

### 4.3 Bevy Reflect 与 Fyrox Script

Bevy `TypeRegistry`以`TypeId + full type path`保存`TypeRegistration`，单独维护short-name歧义集合，注册类型时递归注册field/variant依赖，并通过显式TypeData提供deserialize等能力。Zr需要stable TypeId/TypePath/SchemaId/LayoutId和显式capability，不能让字符串名、value enum顺序或“找不到就TO_STRING”成为类型身份。

Fyrox `ScriptTrait`把init、start、deinit、OS event、fixed update、update和message生命周期放进typed context，并有clone/serialization/type support。该对照主要支持Zircon host lifecycle；语言内部仍以Godot/VerseVM和Zr自身测试为主。Unity Graphics参考树没有语言parser/VM产品，本篇不为了覆盖清单制造RenderGraph与compiler的错误类比。

## 5. P0：发布、信任与语义硬阻断

### ZR-LANG-P0-001 · 真实后端依赖未固定的兄弟仓库和本机动态库，clean clone无法重建同一语言实现

`backend-zr-vm`的两个Rust path dependency越过Zircon仓库边界；binding sys只读取`ZR_VM_RUST_BINDING_LIB_DIR`并链接`dylib=zr_vm_rust_binding`，没有源码commit、submodule、CMake cache、compiler、target、library digest或ABI fingerprint。Cargo.lock无法描述这条依赖，required CI也不构建它。开发者可在相同Zircon commit下链接完全不同的parser/VM DLL，产生不可复现编译结果和运行语义。

建立canonical `ZrToolchainSourceReceipt`和`ZrToolchainBuildReceipt`。选择vendored/submodule、受签名source package或版本化binary SDK中的一种明确模式；receipt至少固定repo commit、submodules、dirty=false、source archive digest、CMake preset/options、C/C++ compiler、target triple、feature set、generated headers、library/import-lib/PDB digest、Rust binding version与ABI hash。`backend-zr-vm`只能消费receipt内artifact，CI从空目录重建并验证digest；任意本机环境路径只能用于显式developer-unqualified profile。

### ZR-LANG-P0-002 · `.zro`读取不验证signature/platform/宽度/endianness/长度预算，损坏输入可进入分配器和执行器

writer写出signature、版本、native widths、instruction width和endian，但reader只读取；`ReadSourceNew`没有检查signature，版本只拒绝patch高于current，major/minor未形成兼容判定，endianness仍是TODO。大量file-controlled count进入分配和递归读取，string用`length + 1`，没有统一checked arithmetic、总量/depth/node budget或完整EOF校验。header也没有payload size、section table/checksum/content digest。当前loader不能区分合法旧格式、跨架构artifact、截断文件、恶意长度和位翻转。

设计新的`ZroContainerV2`并硬切loader admission：固定little-endian与固定宽度primitive；magic、container major/minor、minimum reader、target/ABI/opcode/type-schema/compiler identities；section table含offset/size/alignment/count/digest；whole-file或signed package digest；所有offset/count用checked arithmetic并受`BytecodeLoadBudget`约束。先在bounded byte slice上验证header、sections、references、UTF-8、tokens、CFG、stack/type/layout和debug maps，成功后一次性materialize immutable module。legacy V1只能进入显式migration tool或隔离兼容reader，不得由shipping runtime静默接受。

### ZR-LANG-P0-003 · incremental key与artifact发布不完整且非原子，可复用错误bytecode或暴露混代产物

manifest复用条件只有source/zro FNV hash与imports；compiler/options/target/host ABI/dependency lock/feature switches没有进入key。compiler逐module覆盖最终文件、删除stale output，再保存cache与manifest；writer本身也直接打开final path。崩溃、磁盘写满、杀进程或中途诊断可留下新旧`.zro/.zri/AOT C/manifest`混合。仓内两份manifest保存绝对路径并引用14个不存在output，证明artifact catalog已无法承担generation truth。

建立`ScriptBuildKey = H(source closure, normalized project v2, dependency lock, compiler+format+opcode+type schema, target, options, host ABI, native surface, generated input)`，使用碰撞安全digest。所有module、interface、AOT、debug/source map、manifest和diagnostic先写入content-addressed staging generation，逐文件校验writer result/digest，再原子发布一个package-relative`ScriptBuildReceipt`；失败保留last-good且不删除旧generation。runtime、editor、export、test只按receipt打开，不自行从路径猜测或在load中另编译。

### ZR-LANG-P0-004 · type checker、codegen、object model与GC cache没有统一语义，当前临时分支可静默错编译并触发内存安全征兆

type system允许所有整数互转、共同整数类型总是`int64`；复杂cast统一发`TO_STRING`；INT16 promotion没有完整opcode；constant reference把路径终点/child index设为临时假定；execution dispatch在struct conversion缺prototype时复制源对象；GC检测到PIC slot count或slot内容损坏后清空。这里既有错误程序被接受，也有正确程序被改变含义，还有cache/object layout corruption迹象。仅为每个TODO添加一个分支无法证明跨interpreter/AOT/GC一致。

先定义canonical `TypeSchema + ConversionMatrix + LayoutDescriptor + OpcodeSchema + ConstantRefSchema + InlineCacheSchema`，从同一数据生成parser/type checker、SemIR validation、bytecode encode/decode/verifier、interpreter dispatch、AOT lowering、reflection/native binding和tests。非法implicit conversion必须带稳定diagnostic拒绝；explicit narrowing定义overflow policy；generic/tuple/nominal cast拥有结构化SemIR；object conversion必须materialize正确prototype/layout并执行barrier。PIC corruption在debug/required lane必须fail-fast并产出最小artifact，不得长期靠清空掩盖；修复后以stress、sanitizer和cross-backend differential证明无损坏。

## 6. P1：Project、Compiler 与 Module Graph

| ID | 当前差距 | 需要重构 |
|---|---|---|
| ZR-LANG-P1-001 | 276份tracked JSON `.zrp`仍是无`manifestVersion`的v1四字段项目 | 全部迁移到v2 schema，固定name/version/kind/source/binary/entry/package/dependency与target profile |
| ZR-LANG-P1-002 | 一份Power Echo `.zrp`实际是TOML-like文本，只被Node substring检查读取 | 改名为非project fixture或迁移为合法project；validator拒绝extension与schema不一致 |
| ZR-LANG-P1-003 | project v1缺少明确retirement/migration期限 | 提供只读migrator、compat telemetry和硬切版本，shipping compiler只接受current major |
| ZR-LANG-P1-004 | CMake/project/Rust crate/IO patch有四套版本数字 | 建立单一toolchain release identity并生成C/Rust/CMake/package constants |
| ZR-LANG-P1-005 | source encoding、BOM、newline、Unicode normalization与case policy没有BuildKey合同 | 规范化规则写入project/profile，保留原始bytes digest与canonical text digest |
| ZR-LANG-P1-006 | module identity可由path、module name、dependency selector和runtime cache多种方式表示 | 建立canonical `ModuleId(package, normalized path, version)`，parser到runtime只传typed ID |
| ZR-LANG-P1-007 | constant reference的module fallback会遍历registry并按字符串比较查找 | compile/link阶段解析精确ModuleId/ExportId；runtime不做模糊全表搜索 |
| ZR-LANG-P1-008 | parser/compiler/project/runtime各自持有部分import graph | 生成唯一`ModuleDependencyGraph`，包含value/type/comptime/native/build edge与cycle class |
| ZR-LANG-P1-009 | cycle handling没有统一的允许/拒绝分类与稳定diagnostic | 区分type-only、declaration、module init、comptime与runtime cycle，给出最小cycle path |
| ZR-LANG-P1-010 | compile binding只有`emit_intermediate`和`incremental`两个bool | 引入versioned `CompileRequest`，包含profile、target、optimization、debug、budget、cancel和artifact policy |
| ZR-LANG-P1-011 | compiler diagnostics没有绑定build generation和source snapshot | 每条diagnostic携BuildKey、ModuleId、source digest、stable code、phase、span和related spans |
| ZR-LANG-P1-012 | parser AST free遇到未知node type会跳过children | node kind由schema生成visit/free/clone/serialize；unknown kind在debug fail-fast并有exhaustiveness gate |

## 7. P1：Type System、SemIR 与 Codegen

| ID | 当前差距 | 需要重构 |
|---|---|---|
| ZR-LANG-P1-013 | 所有整数类型都被视为可转换，没有位宽/符号检查 | 定义widen/narrow/sign-change matrix、constant-range proof与overflow policy |
| ZR-LANG-P1-014 | 二元共同整数类型固定为`int64` | 使用规范usual arithmetic conversion并保留literal range与target type |
| ZR-LANG-P1-015 | generic/tuple等复杂cast默认lower为`TO_STRING` | 为每类cast生成typed SemIR；unsupported必须在compile phase拒绝 |
| ZR-LANG-P1-016 | INT16 promotion缺少完整指令支持 | opcode schema覆盖所有numeric widths，或在SemIR阶段规范化为少量明确machine types |
| ZR-LANG-P1-017 | type identity混合value enum、名称、prototype index和metadata token | 建立stable TypeId/TypePath/SchemaId/LayoutId，禁止以显示名承担身份 |
| ZR-LANG-P1-018 | short type/module name歧义没有集中registry | 像Bevy一样维护full path索引与ambiguous short-name集合，诊断列出候选 |
| ZR-LANG-P1-019 | generic实例化、约束、variance和canonical substitution没有统一owner | 建立interned canonical type arena与substitution cache，限制实例化爆炸 |
| ZR-LANG-P1-020 | tuple/struct/object的nominal与structural兼容规则不清 | 在type relation中显式定义identity、shape、prototype、mutability和ownership要求 |
| ZR-LANG-P1-021 | nullable、readonly view和ownership flags未证明进入所有conversion/call boundary | 把qualifier纳入TypeId relation、SemIR verifier、ABI lowering和reflection schema |
| ZR-LANG-P1-022 | constant reference path部分step被临时当终点或child function index | 版本化step enum与typed operands，linker验证完整路径和目标kind |
| ZR-LANG-P1-023 | compile-time与runtime可调用/导出类型合同可能走不同metadata路径 | 统一callable signature、effect、capture、native ABI与export descriptor生成源 |
| ZR-LANG-P1-024 | 类型错误、implicit conversion和unsupported lowering缺稳定conformance catalog | 建立positive/negative spec tests，固定diagnostic code而不固定易变文案 |

## 8. P1：Bytecode Container、Verifier 与 Loader

| ID | 当前差距 | 需要重构 |
|---|---|---|
| ZR-LANG-P1-025 | format只用patch阈值分支，major/minor兼容没有policy | 定义reader/writer compatibility matrix、minimum reader和migration path |
| ZR-LANG-P1-026 | native int/size/instruction按host layout直接序列化 | container统一fixed-width LE；runtime layout通过明确section解码 |
| ZR-LANG-P1-027 | reader读signature但不比较 | admission第一步constant-time magic check，错误返回typed format diagnostic |
| ZR-LANG-P1-028 | reader不校验native width/instruction width/endian | 与target descriptor逐项比较或执行显式转换，禁止静默继续 |
| ZR-LANG-P1-029 | header没有payload/section size和trailing-data policy | section table约束完整覆盖、无重叠、alignment、EOF和可选extension规则 |
| ZR-LANG-P1-030 | count/string/array递归读取缺统一budget和checked arithmetic | `BytecodeLoadBudget`限制bytes/nodes/strings/functions/constants/depth/time并使用checked math |
| ZR-LANG-P1-031 | writer忽略多数`fwrite`和`fclose`结果 | 使用checked sink，传播short write/flush/close错误并删除staging temp |
| ZR-LANG-P1-032 | `ZrCore_Io_ReadSourceFree`为空 | 明确parsed-source ownership arena/RAII，成功转移与失败回收都由测试证明 |
| ZR-LANG-P1-033 | 没有pre-execution opcode/operand/CFG/stack/type verifier | 按VerseVM类机制构建CFG，验证target、stack merge、exception/task region、token与layout |
| ZR-LANG-P1-034 | unknown opcode到dispatch default才报`Not implemented` | decoder/verifier先拒绝未知opcode；executor default只作为unreachable fail-fast |
| ZR-LANG-P1-035 | 没有checksum/digest/signature或package trust绑定 | artifact receipt保存section/whole-file digest，分发层可选签名并绑定BuildSet |
| ZR-LANG-P1-036 | 未发现signature/endian/width/truncation/oversize header corruption tests | 增加table-driven corruption corpus、property tests、coverage-guided fuzz与cross-version fixtures |

## 9. P1：VM Execution、Object、GC 与 Determinism

| ID | 当前差距 | 需要重构 |
|---|---|---|
| ZR-LANG-P1-037 | `RunOptions`只有mode/module/args | 增加fuel、deadline、cancel、stack/call depth、host-call、allocation和output budget |
| ZR-LANG-P1-038 | runtime heap/GC options属于整个Runtime，不是package/session/request | 建立hierarchical quota：process/world/package/session/call，并定义reserve/charge/refund |
| ZR-LANG-P1-039 | 长脚本只能依赖外层锁/线程，不能cooperative preempt | 在dispatch backedge/call/host boundary检查budget与cancel，返回typed termination |
| ZR-LANG-P1-040 | host/native call没有统一wall/output/handle预算 | 所有foreign call通过admission ticket，记录调用者、generation、deadline、bytes和effect |
| ZR-LANG-P1-041 | struct conversion缺prototype时直接复制对象 | verifier保证conversion target存在；runtime按layout构造并执行barrier/rollback |
| ZR-LANG-P1-042 | global `zr`对象创建后未完成global scope注册 | 由bootstrap schema声明内建global，初始化失败阻止Runtime Ready |
| ZR-LANG-P1-043 | PIC损坏时清空slot继续运行 | debug/sanitizer lane fail-fast；shipping隔离module并生成corruption receipt，先修owner/lifetime |
| ZR-LANG-P1-044 | inline cache layout与GC scanner没有共同生成/验证 | 从InlineCacheSchema生成layout、init、trace、rewrite、clear和invariant tests |
| ZR-LANG-P1-045 | module registry fallback遍历全表并依赖字符串对象 | linker生成dense ModuleId/ExportId slot；cache miss有bounded exact lookup和诊断 |
| ZR-LANG-P1-046 | exception/unreachable路径仍有裸`todo`与字符串runtime error | 定义Exception/Trap/Cancel/Budget/HostFailure outcome domain和finally/unwind contract |
| ZR-LANG-P1-047 | clock、random、locale、filesystem/network effect未纳入执行determinism | host capability提供record/replay effect port；deterministic profile禁止隐式process globals |
| ZR-LANG-P1-048 | GC、execution、module load与hot reload缺统一generation fence | object/function/cache/root均绑定RuntimeGeneration，retire前验证无跨代引用 |

## 10. P1：AOT、Native Binding、LSP 与 Qualification

| ID | 当前差距 | 需要重构 |
|---|---|---|
| ZR-LANG-P1-049 | interp/binary/AOT C/AOT LLVM没有required semantic equivalence gate | 同一typed corpus运行四backend，比较value、trap、effect trace、GC roots与state digest |
| ZR-LANG-P1-050 | AOT/input/manifest仍广泛使用64-bit FNV identity | FNV只保留hash table用途；artifact/build/trust identity改用碰撞安全digest |
| ZR-LANG-P1-051 | AOT output不是ScriptBuildReceipt的同代成员 | AOT library、descriptor、symbols、source map和input digest与bytecode一起原子发布 |
| ZR-LANG-P1-052 | Rust sys build只信任环境目录中的同名DLL/import lib | build script读取toolchain receipt并校验library/ABI digest，拒绝裸目录 |
| ZR-LANG-P1-053 | C/Rust CompileOptions和RunOptions没有schema/version/size | 使用versioned ABI struct、size/version negotiation与reserved-zero validation |
| ZR-LANG-P1-054 | Rust error边界没有完整保留phase/module/span/stable code | FFI返回owned structured diagnostic batch，Rust保留source/build identities |
| ZR-LANG-P1-055 | Zircon CI不启用`backend-zr-vm` | 新增固定toolchain source/build、binding tests、real backend compile/run的required lane |
| ZR-LANG-P1-056 | external 128 CTests与Zircon package tests不属于一个BuildSet | CI receipt记录external commit/config/test list/results，并由product qualification引用 |
| ZR-LANG-P1-057 | LSP incremental parser计算简单hash后仍总是full reparse | 建立versioned text snapshot、incremental syntax tree、bounded invalidation与fallback telemetry |
| ZR-LANG-P1-058 | compiler与LSP可能分别实现type/semantic facts | LSP消费compiler service的canonical syntax/type/diagnostic snapshot，不复制规则 |
| ZR-LANG-P1-059 | debugger/source map/stack/local/eval产品链未绑定artifact generation | Editor31实现UI；本篇输出generation-qualified debug metadata和safe eval request |
| ZR-LANG-P1-060 | 没有语言级compile/load/execute/GC/AOT性能资格 | 建立规模梯度benchmark、profile/counters、回归阈值及trace artifact，由Tooling07托管 |

## 11. P2：工程质量与长期维护

| ID | 当前差距 | 需要重构 |
|---|---|---|
| ZR-LANG-P2-001 | opcode/type/format文档由源码常量和注释分散维护 | 从schema生成规范、tables、disassembler与compat文档 |
| ZR-LANG-P2-002 | CMake option注释与默认值存在漂移 | preset成为真源，configure receipt打印最终值并由test snapshot验证 |
| ZR-LANG-P2-003 | warning严格度没有按required/dev/third-party分层 | first-party C/Rust启用warnings-as-errors，vendor隔离并记录例外 |
| ZR-LANG-P2-004 | compiler/VM日志缺统一category、module/build correlation | 结构化event携phase、ModuleId、BuildKey、RuntimeGeneration和duration |
| ZR-LANG-P2-005 | disassembler不能作为版本化诊断artifact稳定消费 | 输出schema-versioned JSON/text并验证roundtrip与unknown section |
| ZR-LANG-P2-006 | source map/debug info大小和剥离策略未profile化 | Debug/Dev/Shipping定义独立section与strip receipt |
| ZR-LANG-P2-007 | error文案中英文与术语混杂 | stable code独立于localized message，统一术语表和locale catalog |
| ZR-LANG-P2-008 | parser/IR dump可能暴露绝对路径和本机信息 | artifact使用workspace-relative SourceId，diagnostic display单独映射本机路径 |
| ZR-LANG-P2-009 | compile cache缺命中原因与miss taxonomy | 记录input component digest差异、hit level、bytes/time saved和eviction原因 |
| ZR-LANG-P2-010 | bytecode loader没有可观测的拒绝阶段 | 输出header/section/verifier/budget/trust阶段码，不记录不可信payload正文 |
| ZR-LANG-P2-011 | test corpus没有自动最小化与长期seed治理 | fuzz crash自动minimize、去重、固定format/compiler identity并进入regression corpus |
| ZR-LANG-P2-012 | language feature完成度容易按文件/TODO/test数量外推 | capability只能由spec item、implementation、required tests和product consumer共同资格化 |
| ZR-LANG-P2-013 | legacy `.zrp/.zro`清理没有inventory dashboard | 生成按format/build/owner/consumer分类的migration inventory与retirement burn-down |
| ZR-LANG-P2-014 | external repo与Zircon issue/owner映射不清 | 为compiler/runtime/binding/CI指定双仓owner、SLA、release branch和backport policy |

## 12. 目标架构

```text
ZrToolchainSourceReceipt
        |
        v
ZrToolchainBuildReceipt ---- HostAbiSchema / OpcodeSchema / TypeSchema
        |                                  |
        +--------------------+-------------+
                             v
ProjectManifestV2 + DependencyLock + Frozen SourceSet
                             |
                             v
ParseSnapshot -> SemanticSnapshot -> Typed SemIR -> Verified Module Graph
                             |
              +--------------+----------------+
              v                               v
       ZroContainerV2                    AOT C / LLVM
              +--------------+----------------+
                             v
                   ScriptBuildReceipt
                             |
                  loader admission/verifier
                             |
                             v
 RuntimeGeneration + ExecutionBudget + EffectPort + DebugMetadata
                             |
                             v
                 ScriptExecutionReceipt
```

关键不变量：

1. 同一BuildKey只能对应一组不可变artifact digests；任何不同compiler/options/ABI/source closure必须产生不同key。
2. runtime只加载已验证且与当前toolchain/host ABI兼容的ScriptBuildReceipt，不直接编译developer project。
3. parser、semantic、SemIR、bytecode、interpreter、AOT和native binding共享Type/Opcode/Layout schema。
4. bytecode在任何分配大对象、注册module或执行opcode之前完成bounded verification。
5. 执行必须在fuel/deadline/cancel/quota/effect contract内终止，并产生可区分成功、trap、budget、cancel和host failure的receipt。
6. hot reload只在新build generation完整验证后原子切换；旧object/cache/root不得跨generation泄漏。

## 13. 重构里程碑

### M0 · 冻结不可信发布与建立inventory

- 标记当前real backend为developer-unqualified，shipping/profile不得因dist package存在而宣告Ready。
- 枚举全部`.zr/.zrp/.zro/.zri/AOT/manifest`及consumer，记录format、hash、owner、是否tracked、是否可搬运。
- 对两份absolute/incomplete manifest和TOML-like伪`.zrp`建立明确migration issue，不在review阶段修改资产。
- 增加静态gate：required product不得消费未绑定BuildReceipt的`.zro`。

### M1 · 收编并固定toolchain source/build

- 选择外部源码治理模式，固定commit/submodule/archive digest和CMake preset。
- 从clean directory构建C libraries、CLI、Rust binding、symbols并产生ToolchainBuildReceipt。
- Zircon CI启用real backend并运行最小compile/load/run及external CTest receipt。
- 统一CMake/C/Rust/format release identity。

### M2 · 收敛language schema与type semantics

- 建立TypeSchema、ConversionMatrix、LayoutDescriptor、OpcodeSchema、ConstantRefSchema和InlineCacheSchema。
- 删除任意integer conversion、default TO_STRING、ambiguous child-index和prototype-less copy。
- 生成parser/compiler/verifier/interpreter/AOT/binding tables及negative conformance tests。
- required sanitizer/stress lane先复现并消除PIC corruption。

### M3 · 实现ZroContainerV2与bounded verifier

- 定义fixed-endian container、section table、compatibility、digests和budgets。
- 实现two-phase decode/verify/materialize及完整ownership cleanup。
- 建立corruption/fuzz/cross-version corpus；legacy V1只由migration tool读取。
- verifier构建CFG并验证jump、stack/type merge、exception/task region、tokens和layouts。

### M4 · 原子compiler artifact generation

- 迁移tracked project到manifest v2和dependency lock。
- 实现完整BuildKey、content-addressed staging、whole-generation commit和last-good。
- 生成package-relativeScriptBuildReceipt，纳入bytecode/interface/AOT/debug/diagnostic。
- Editor、CLI、export、tests共享同一compiler service和receipt，不再各自写final path。

### M5 · 执行治理、object与GC一致性

- 扩展RunRequest/ABI，接入fuel、deadline、cancel、quota、host-call/effect预算。
- module/global/object/prototype/cache/root全部绑定RuntimeGeneration。
- 修复PIC owner/lifetime并建立GC barrier/cache schema generated tests。
- 输出typed execution outcome与receipt，支持record/replay deterministic profile。

### M6 · LSP、Debugger、AOT 与性能资格

- LSP消费canonical compiler snapshot并实现bounded incremental invalidation。
- debug metadata绑定BuildKey/ModuleId/source digest；Editor31消费stack/local/eval接口。
- interp/binary/AOT C/LLVM跑同一differential corpus。
- 建立compile/load/execute/GC/AOT规模梯度、trace和regression gates。

### M7 · 产品迁移与legacy退休

- WOC、Vampire和template从current ScriptBuildReceipt安装并启动，不在package load中compile source。
- clean clone构建、install、launch、save/reload、debug和export均引用同一BuildSet。
- legacy v1 project、V1 `.zro`、absolute manifest和unindexed artifact归零后删除compat reader。
- 通过长时、故障注入、cross-platform、cross-backend和产品acceptance后再宣告qualified。

## 14. 验收矩阵

| 资格面 | 必须证明 |
|---|---|
| clean source | Zircon commit可解析唯一zr_vm source commit/submodules，dirty或缺失立即失败 |
| reproducible build | 两个空目录按同一receipt构建产生等价artifact digest；路径/timestamp不进入语义输出 |
| project graph | dependency lock、cycle、alias、package export、source normalization有positive/negative tests |
| type semantics | conversion/generic/tuple/nullability/ownership/nominal layout在checker、interp和AOT一致 |
| bytecode trust | magic/version/endian/width/truncation/overlap/overflow/oversize/bad opcode/bad CFG全部fail-closed |
| atomic artifact | 任意write/flush/rename/disk-full/kill点只暴露旧完整或新完整generation |
| execution control | infinite loop、deep recursion、allocation storm、host-call storm按budget/cancel有界退出 |
| memory safety | ASan/UBSan或等价lane、GC stress、PIC/cache/root/reload测试无corruption/leak/UAF |
| backend parity | interpreter、binary、AOT C、AOT LLVM的value/trap/effect/state digest一致 |
| tooling parity | CLI、Rust binding、LSP、Editor/debugger消费同一diagnostic/type/build generation |
| product closure | WOC/Vampire clean build-install-launch使用receipt artifact且runtime不打开project source |
| observability | compile/load/verify/execute/GC receipt可关联BuildKey、ModuleId、RuntimeGeneration和target |

任一局部unit test、历史`.zro`、外部CTest exit 0、Zircon dist build或一次脚本输出都不能单独生成产品资格。

## 15. 依赖、所有权与禁止旁路

- Runtime21是Zr语言核心、compiler artifact、bytecode verifier和execution budget的canonical owner。
- Runtime07只拥有package/plugin discovery、instance/hot reload和backend lifecycle；它必须消费Runtime21的receipt，不再在load内另造compiler authority。
- Plugins01拥有foreign ABI、package trust和provider activation；binding ABI schema由Runtime21生成、Plugins01执行admission。
- Editor31拥有code editor、LSP/debugger/visual script产品；不得复制parser/type rules或用编辑器缓存冒充compiler snapshot。
- Tooling05拥有生成入口与artifact orchestration，Tooling10拥有test selection/result，Tooling11拥有parity oracle，Tooling17拥有SourceSet；它们都引用同一BuildKey。
- App03/WOC runtime reports拥有产品world/transaction/domain语义；语言层只提供有界执行与effect receipt，不直接实现gameplay规则。

禁止以下“快速修复”：

1. 只在`ReadSourceNew`补一个signature `memcmp`，却继续接受未验证length/endian/opcode/layout。
2. 只把FNV换成更长hash，仍遗漏compiler/options/target/ABI/dependency inputs。
3. 每个output各自用temp rename，却没有whole-generation manifest commit。
4. 为复杂cast补更多string/type-name分支，而不建立canonical type relation与SemIR。
5. 在GC发现PIC损坏时继续扩大清空/忽略条件，掩盖owner或write barrier问题。
6. 把外部仓库当前dirty目录打包进CI cache，绕过固定source receipt。
7. 用外部128个CTest数量、38个`.zro`或WOC静态source checks宣告语言后端完成。

## 16. 本轮记录

本轮只新增review与索引，不修改Zircon或`zr_vm` production/test/build源码，不生成、删除或迁移任何`.zrp/.zro/.zri/AOT`，也不运行dirty external source的CMake/CTest/real backend。外部仓库commit与dirty状态、Zircon artifact inventory、CI feature可达性和reference source均需在实施前重新核对。
