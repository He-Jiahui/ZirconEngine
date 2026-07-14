---
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/reflection_macros/Cargo.toml
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime_interface/src/reflect/type_registration.rs
  - zircon_runtime_interface/src/reflect/script_visibility.rs
  - zircon_runtime/reflection_macros/src/lib.rs
  - zircon_runtime/reflection_macros/src/args.rs
  - zircon_runtime/reflection_macros/src/attrs.rs
  - zircon_runtime/reflection_macros/src/derive_type.rs
  - zircon_runtime/reflection_macros/src/function.rs
  - zircon_runtime/reflection_macros/src/module.rs
  - zircon_runtime/reflection_macros/src/tokens.rs
  - zircon_runtime/reflection_macros/src/tests.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/vm/plugin/mod.rs
  - zircon_runtime/src/script/vm/host/mod.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/script/vm/runtime_context.rs
  - zircon_runtime/src/script/vm/scene_hook.rs
  - zircon_runtime/src/script/vm/scene_hook/error.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/error.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/garbage_collection.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/memory.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/policy.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/mod.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/options.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/markdown.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/writer.rs
  - zircon_runtime/src/script/vm/host/plugin_host_driver.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_host_context.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/script/vm/reflection/mod.rs
  - zircon_runtime/src/script/vm/reflection/catalog.rs
  - zircon_runtime/src/script/vm/reflection/error.rs
  - zircon_runtime/src/script/vm/reflection/schema.rs
  - zircon_runtime/src/script/vm/reflection/tests/schema_invariants.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/unload_atomicity.rs
  - zircon_runtime/src/script/vm/backend/mod.rs
  - zircon_runtime/src/script/vm/backend/backend_registry.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/package.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/reflection_host.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/script_call_table.rs
  - zircon_plugins/zr_vm_language/runtime/src/reflection_host/reflection_host_module.rs
  - zircon_runtime/src/dynamic_api/session/tests/mod.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/mod.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/policy.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/hot_reload.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/garbage_collection.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/memory.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_slot_record.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_slot_state.rs
  - examples/vampire/scripts/vampire_game/plugin.toml
  - examples/vampire/scripts/vampire_game/main.zr
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml
  - examples/vampire/assets/navigation/main.navmesh.toml
  - examples/vampire/assets/animation/vampire_idle.graph.zranim
  - examples/vampire/assets/animation/vampire_move.graph.zranim
  - examples/vampire/assets/animation/vampire_attack.graph.zranim
  - examples/vampire/assets/animation/vampire_locomotion.state_machine.zranim
  - zircon_runtime/src/bin/zircon_host_reflection_docs.rs
  - zircon_runtime/src/bin/zircon_host_reflection_docs/args.rs
  - zircon_runtime/src/bin/zircon_host_reflection_docs/error.rs
  - zircon_runtime/src/bin/zircon_host_reflection_docs/run.rs
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.toml
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.zrp
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/main.zr
implementation_files:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/reflection_macros/Cargo.toml
  - zircon_runtime/src/core/framework/script.rs
  - zircon_runtime_interface/src/reflect/type_registration.rs
  - zircon_runtime_interface/src/reflect/script_visibility.rs
  - zircon_runtime/reflection_macros/src/lib.rs
  - zircon_runtime/reflection_macros/src/args.rs
  - zircon_runtime/reflection_macros/src/attrs.rs
  - zircon_runtime/reflection_macros/src/derive_type.rs
  - zircon_runtime/reflection_macros/src/function.rs
  - zircon_runtime/reflection_macros/src/module.rs
  - zircon_runtime/reflection_macros/src/tokens.rs
  - zircon_runtime/reflection_macros/src/tests.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/vm/plugin/mod.rs
  - zircon_runtime/src/script/vm/host/mod.rs
  - zircon_runtime/src/script/vm/host/host_export_registry.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/host/builtin_host_modules.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs
  - zircon_runtime/src/script/vm/runtime_context.rs
  - zircon_runtime/src/script/vm/scene_hook.rs
  - zircon_runtime/src/script/vm/scene_hook/error.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/mod.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/options.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/markdown.rs
  - zircon_runtime/src/script/vm/host/reflection_docs/writer.rs
  - zircon_runtime/src/script/vm/host/plugin_host_driver.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_host_context.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/script/vm/reflection/mod.rs
  - zircon_runtime/src/script/vm/reflection/catalog.rs
  - zircon_runtime/src/script/vm/reflection/error.rs
  - zircon_runtime/src/script/vm/reflection/schema.rs
  - zircon_runtime/src/script/vm/backend/mod.rs
  - zircon_runtime/src/script/vm/backend/backend_registry.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/host_modules.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/instance.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/package.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend/reflection_host.rs
  - zircon_plugins/zr_vm_language/runtime/src/call_site/script_call_table.rs
  - zircon_plugins/zr_vm_language/runtime/src/reflection_host/reflection_host_module.rs
  - zircon_runtime/src/dynamic_api/session/tests/mod.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/mod.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/policy.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/hot_reload.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/garbage_collection.rs
  - zircon_runtime/src/script/vm/plugin/management_policy/memory.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs
  - zircon_plugins/zr_vm_language/runtime/src/real_backend.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_slot_record.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_slot_state.rs
  - examples/vampire/scripts/vampire_game/plugin.toml
  - examples/vampire/scripts/vampire_game/main.zr
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml
  - examples/vampire/assets/navigation/main.navmesh.toml
  - examples/vampire/assets/animation/vampire_idle.graph.zranim
  - examples/vampire/assets/animation/vampire_move.graph.zranim
  - examples/vampire/assets/animation/vampire_attack.graph.zranim
  - examples/vampire/assets/animation/vampire_locomotion.state_machine.zranim
  - zircon_runtime/src/bin/zircon_host_reflection_docs.rs
  - zircon_runtime/src/bin/zircon_host_reflection_docs/args.rs
  - zircon_runtime/src/bin/zircon_host_reflection_docs/error.rs
  - zircon_runtime/src/bin/zircon_host_reflection_docs/run.rs
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.toml
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/plugin.zrp
  - docs/zircon_runtime/script/vm/examples/zr_vm_minimal/main.zr
plan_sources:
  - user: 2026-05-15 implement ZrVM language plugin and reflection registration plan
  - user: 2026-05-16 continue precise VM host reflection macro implementation
  - user: 2026-05-18 modular reflection content/generated reflection interface documentation
  - user: 2026-05-20 continue ZrVM host reflection follow-up with macro modularity
  - user: 2026-05-21 continue ZrVM lane 1 real backend hardening
  - .codex/plans/Zircon Runtime 独立 3D 游戏能力与 Vampire 示例计划.md
  - docs/superpowers/specs/2026-05-20-zrvm-reflection-macro-modularity-design.md
  - docs/superpowers/plans/2026-05-20-zrvm-reflection-macro-modularity.md
  - docs/superpowers/plans/2026-05-18-zrvm-host-reflection-docs.md
  - docs/superpowers/specs/2026-05-21-zrvm-real-backend-hardening-design.md
  - docs/superpowers/plans/2026-05-21-zrvm-real-backend-hardening.md
  - user: 2026-06-10 vampire roguelite animation state-machine follow-up
  - user: 2026-06-10 vampire graphical HUD, terrain-backed jungle, screen-space health HUD, and buff particles
  - user: 2026-06-12 remove runtime Vampire fallback backend
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_plugins/08-zr-vm.md
tests:
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_runtime/src/script/vm/reflection/tests.rs
  - zircon_runtime/src/script/vm/reflection/tests/schema_invariants.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/reflection.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/unload_atomicity.rs
  - zircon_plugins/zr_vm_language/runtime/src/reflection_host/tests.rs
  - vm_backend_registry_accessors_recover_poisoned_family_lock
  - host_registry_accessors_recover_poisoned_handle_lock
  - host_export_registry_accessors_recover_poisoned_module_lock
  - hot_reload_coordinator_accessors_recover_poisoned_slot_table_lock
  - runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries
  - runtime_15_script_vm_hot_reload_coordinator_tests_are_folder_backed
  - vm_plugin_manager_selected_backend_accessors_recover_poisoned_lock
  - runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_guard_covers_manager_selector
  - "cargo test -p zircon_runtime script::vm: passed 2026-05-15"
  - "cargo test -p zircon_runtime script::vm --locked --target-dir target\\codex-reflection-macros: passed 2026-05-16"
  - "cargo fmt --manifest-path zircon_runtime/reflection_macros/Cargo.toml --check: passed 2026-05-16"
  - "cargo test -p zircon_runtime script::vm --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: passed 2026-05-16"
  - "cargo test -p zircon_runtime script::vm --locked --jobs 1: attempted 2026-05-16 in E:\\cargo-targets\\zircon-zrvm-continue; local machine remained saturated by concurrent cargo jobs before completion"
  - "cargo test -p zircon_runtime script::vm --locked --offline --jobs 1 -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm and CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue: passed 2026-05-16; 16 passed, 0 failed, 1487 filtered out"
  - "cargo check -p zircon_runtime --lib --locked --offline --jobs 1 with CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-runtime-check: passed 2026-05-16"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm --locked --offline --jobs 1 real_backend_loads_documented_minimal_example -- --nocapture --test-threads=1 with CARGO_HOME=D:\\cargo-home-zrvm, CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-zrvm-continue, ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug: passed 2026-05-16; 1 passed, 0 failed, 5 filtered out"
  - "cargo check --workspace --locked: passed 2026-05-15"
  - "cargo test --workspace --locked: attempted 2026-05-15; stopped by no space on device"
  - "cargo test -p zircon_runtime script::vm --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: attempted 2026-05-18; blocked before reflection tests by unrelated graphics test compile errors in zircon_runtime/src/graphics/tests/render_product_ui.rs missing RenderStats UI fields"
  - "cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: passed 2026-05-18"
  - "cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: passed 2026-05-18; 0 unit tests and 0 doc-tests"
  - "rustfmt --edition 2021 --check zircon_runtime/src/core/framework/script.rs zircon_runtime/src/script/vm/host/host_export_registry.rs zircon_runtime/src/script/vm/tests.rs zircon_runtime/reflection_macros/src/lib.rs zircon_runtime/src/script/vm/host/builtin_host_modules.rs zircon_plugins/zr_vm_language/runtime/src/real_backend.rs: passed 2026-05-18"
  - "cargo fmt --all --check: attempted 2026-05-18; blocked by unrelated unformatted asset/render/scene files owned by concurrent sessions"
  - "cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: attempted 2026-05-18 after enum/default-type-ref hardening; blocked by unrelated graphics compile error E0061 in zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs"
  - "cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: passed 2026-05-18 after enum/default-type-ref hardening; 0 unit tests and 0 doc-tests"
  - "rustfmt --edition 2021 --check zircon_runtime/src/core/framework/script.rs zircon_runtime/src/script/vm/host/host_export_registry.rs zircon_runtime/src/script/vm/tests.rs zircon_runtime/reflection_macros/src/lib.rs zircon_runtime/src/script/vm/host/builtin_host_modules.rs zircon_plugins/zr_vm_language/runtime/src/real_backend.rs: passed 2026-05-18 after enum/default-type-ref hardening"
  - "cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: red 2026-05-18 for unsupported-input macro tests before guards; async/generic function and generic type tests failed because macros still emitted descriptors"
  - "cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macros: passed 2026-05-18 after unsupported-input guards; 3 passed, 0 failed, 0 doc-tests"
  - "rustfmt --edition 2021 --check zircon_runtime/reflection_macros/src/lib.rs zircon_runtime/src/script/vm/tests.rs zircon_runtime/src/core/framework/script.rs zircon_runtime/src/script/vm/host/host_export_registry.rs zircon_runtime/src/script/vm/host/builtin_host_modules.rs zircon_plugins/zr_vm_language/runtime/src/real_backend.rs: passed 2026-05-18 after unsupported-input guards"
  - "cargo test -p zircon_runtime host_reflection_docs --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-docs -- --nocapture --test-threads=1: passed 2026-05-18 during Milestone 2; 4 host_reflection_docs tests passed"
  - "cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-docs: passed 2026-05-18 during Milestone 2"
  - "cargo run -p zircon_runtime --bin zircon_host_reflection_docs --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-docs -- F:\\cargo-targets\\codex-reflection-docs\\host-interface.md: passed 2026-05-18 during Milestone 3; generated explicit-output host interface Markdown"
  - "Test-Path -LiteralPath 'F:\\cargo-targets\\codex-reflection-docs\\host-interface.md': passed 2026-05-18 during Milestone 3; generated file existed"
  - "Grep tool search for 'zr\\.zircon\\.math' in F:\\cargo-targets\\codex-reflection-docs\\host-interface.md: passed 2026-05-18 during Milestone 3; generated file included zr.zircon.math at line 76"
  - "rustfmt --edition 2021 --check zircon_runtime/src/script/vm/host/reflection_docs/mod.rs zircon_runtime/src/script/vm/host/reflection_docs/options.rs zircon_runtime/src/script/vm/host/reflection_docs/markdown.rs zircon_runtime/src/script/vm/host/reflection_docs/writer.rs zircon_runtime/src/script/vm/host/mod.rs zircon_runtime/src/script/vm/mod.rs zircon_runtime/src/script/mod.rs zircon_runtime/src/script/vm/tests.rs zircon_runtime/src/bin/zircon_host_reflection_docs.rs: passed 2026-05-18 final validation"
  - "cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-docs: passed 2026-05-18 final validation; 3 passed, 0 failed, 0 doc-tests"
  - "cargo test -p zircon_runtime host_reflection_docs --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-docs -- --nocapture --test-threads=1: passed 2026-05-18 final validation; 4 host_reflection_docs tests passed, 1561 filtered out"
  - "cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-docs: passed 2026-05-18 final validation"
  - "cargo run -p zircon_runtime --bin zircon_host_reflection_docs --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-docs -- F:\\cargo-targets\\codex-reflection-docs\\host-interface.md: passed 2026-05-18 final validation; generated explicit-output host interface Markdown"
  - "Test-Path -LiteralPath 'F:\\cargo-targets\\codex-reflection-docs\\host-interface.md': passed 2026-05-18 final validation; generated file existed"
  - "Grep tool search for 'zr\\.zircon\\.math' in F:\\cargo-targets\\codex-reflection-docs\\host-interface.md: passed 2026-05-18 final validation; generated file included zr.zircon.math at line 76"
  - "rustfmt --edition 2021 --check zircon_runtime/reflection_macros/src/lib.rs zircon_runtime/reflection_macros/src/args.rs zircon_runtime/reflection_macros/src/attrs.rs zircon_runtime/reflection_macros/src/derive_type.rs zircon_runtime/reflection_macros/src/function.rs zircon_runtime/reflection_macros/src/module.rs zircon_runtime/reflection_macros/src/tokens.rs zircon_runtime/reflection_macros/src/tests.rs: passed 2026-05-20 final validation"
  - "cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macro-modularity: passed 2026-05-20 final validation; 10 passed, 0 failed, 0 doc-tests"
  - "cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macro-modularity --verbose: passed 2026-05-20 evidence run; 2 existing warnings in scene ECS helpers"
  - "cargo test -p zircon_runtime --lib rust_reflection_macros_generate_type_function_and_module_descriptors --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macro-modularity --verbose -- --nocapture --test-threads=1: passed 2026-05-20 final validation; 1 passed, 0 failed, 1745 filtered out"
  - "cargo test -p zircon_runtime --lib host_reflection_docs_include_macro_generated_builtin_math_module --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macro-modularity --verbose -- --nocapture --test-threads=1: passed 2026-05-20 final validation; 1 passed, 0 failed, 1746 filtered out"
  - "F: free space check before closeout validation: passed 2026-05-21; 66.86 GB free, no target cleanup required for F:\\cargo-targets\\codex-reflection-macro-modularity"
  - "rustfmt --edition 2021 --check zircon_runtime/reflection_macros/src/lib.rs zircon_runtime/reflection_macros/src/args.rs zircon_runtime/reflection_macros/src/attrs.rs zircon_runtime/reflection_macros/src/derive_type.rs zircon_runtime/reflection_macros/src/function.rs zircon_runtime/reflection_macros/src/module.rs zircon_runtime/reflection_macros/src/tokens.rs zircon_runtime/reflection_macros/src/tests.rs: passed 2026-05-21 closeout validation"
  - "cargo test --manifest-path zircon_runtime/reflection_macros/Cargo.toml --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macro-modularity: passed 2026-05-21 closeout validation; 10 passed, 0 failed, 0 doc-tests"
  - "cargo test -p zircon_runtime --lib rust_reflection_macros_generate_type_function_and_module_descriptors --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macro-modularity --verbose -- --nocapture --test-threads=1: passed 2026-05-21 closeout validation; 1 passed, 0 failed, 1746 filtered out"
  - "cargo test -p zircon_runtime --lib host_reflection_docs_include_macro_generated_builtin_math_module --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-reflection-macro-modularity --verbose -- --nocapture --test-threads=1: passed 2026-05-21 closeout validation; 1 passed, 0 failed, 1746 filtered out"
  - "F: free space check before ZrVM real-backend hardening validation: passed 2026-05-24; 93.32 GB free, no target cleanup required for F:\\cargo-targets\\codex-zrvm-real-backend-hardening"
  - "rustfmt --edition 2021 --check zircon_plugins/zr_vm_language/runtime/src/lib.rs zircon_plugins/zr_vm_language/runtime/src/backend.rs zircon_plugins/zr_vm_language/runtime/src/real_backend.rs: passed 2026-05-24"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-zrvm-real-backend-hardening: passed 2026-05-24; 3 passed, 0 failed, 0 doc-tests"
  - "cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_zr_vm_language_runtime --features backend-zr-vm --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-zrvm-real-backend-hardening real_backend -- --nocapture --test-threads=1 with ZR_VM_RUST_BINDING_LIB_DIR=E:\\Git\\zr_vm\\build\\codex-msvc-debug\\lib\\Debug and PATH including E:\\Git\\zr_vm\\build\\codex-msvc-debug\\bin\\Debug: passed 2026-05-24; 11 passed, 0 failed, 3 filtered out"
  - "rustfmt --edition 2021 --check zircon_runtime/src/script/vm/plugin/management_policy/mod.rs zircon_runtime/src/script/vm/plugin/management_policy/hot_reload.rs zircon_runtime/src/script/vm/plugin/management_policy/garbage_collection.rs zircon_runtime/src/script/vm/plugin/management_policy/memory.rs zircon_runtime/src/script/vm/plugin/management_policy/policy.rs zircon_runtime/src/script/vm/plugin/mod.rs zircon_runtime/src/script/vm/plugin/vm_plugin_manifest.rs zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs zircon_runtime/src/script/vm/runtime/mod.rs zircon_runtime/src/script/vm/runtime/vm_plugin_slot_record.rs zircon_runtime/src/script/vm/runtime/vm_plugin_slot_state.rs zircon_runtime/src/script/vm/mod.rs zircon_runtime/src/script/mod.rs zircon_runtime/src/script/vm/tests.rs: passed 2026-06-04"
  - "git diff --check -- zircon_runtime/src/script/vm/plugin/management_policy zircon_runtime/src/script/vm/plugin/mod.rs zircon_runtime/src/script/vm/plugin/vm_plugin_manifest.rs zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs zircon_runtime/src/script/vm/runtime zircon_runtime/src/script/vm/mod.rs zircon_runtime/src/script/mod.rs zircon_runtime/src/script/vm/tests.rs: passed 2026-06-04 with only expected LF-to-CRLF warnings"
  - "cargo test -p zircon_runtime hot_reload_policy --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-vm-management-policy --message-format short --color never -- --nocapture --test-threads=1: attempted 2026-06-04; timed out during compilation while other workspace/render Cargo and rustc processes were active"
  - "cargo test -p zircon_runtime discovery_parses_vm_management_policy_from_manifest --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-vm-management-policy --message-format short --color never -- --nocapture --test-threads=1: attempted 2026-06-04; timed out during compilation while other workspace/render Cargo and rustc processes were active"
  - "cargo test -p zircon_runtime default_management_policy_preserves_state_and_defers_gc_to_backend --locked --offline --jobs 1 --target-dir F:\\cargo-targets\\codex-vm-management-policy --message-format short --color never -- --nocapture --test-threads=1: attempted 2026-06-04; timed out during compilation while other workspace/render Cargo and rustc processes were active"
  - "cargo test -p zircon_runtime --lib vm_plugin_manager_calls_exports_by_loaded_package_name --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=E:\\cargo-targets\\zircon-vampire-runtime: passed 2026-06-09"
  - "cargo test -p zircon_runtime --lib builtin_host_modules_register_gameplay_capabilities --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=E:\\cargo-targets\\zircon-vampire-runtime: passed 2026-06-09"
  - "cargo check -p zircon_runtime --lib: passed 2026-06-09"
  - "cargo test -p zircon_runtime --lib gameplay_pose_exports_update_entity_transform -- --nocapture: passed 2026-06-09 after vampire pose host additions"
  - "cargo test -p zircon_runtime --lib vampire_project_session --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-vampire-app: passed 2026-06-10; 4 project-session tests passed, covering WASD, Blood Bolt damage, nav chase, HUD text, and dynamic enemy spawning"
  - "CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-vampire-app cargo test -p zircon_runtime --lib vampire_project_session --locked --message-format short -- --nocapture --test-threads=1: passed 2026-06-10; 10 project-session tests passed, including WASD movement, Blood Bolt damage, HUD/capture, pickups, choices, boss spawn, enemy chase, dynamic spawn, and animation state-machine parameter writes"
  - "cargo test -p zircon_runtime --lib vampire_project_session_w_key_moves_player_before_input_clear --locked --message-format short -- --nocapture --test-threads=1: passed 2026-06-10; verifies W input moves the player, writes moving=true, and makes vampire body node transforms change through the animation state-machine hook"
  - "cargo test -p zircon_runtime --lib vampire_project_session_ --locked --message-format short -- --nocapture --test-threads=1: passed 2026-06-10; 10 project-session tests passed with the built-in animation module/hook path"
  - "ZR_VAMPIRE_CAPTURE_PNG=E:\\Git\\ZirconEngine\\examples\\vampire\\screenshots\\vampire-runtime-frame.png ZR_VAMPIRE_CAPTURE_WIDTH=1280 ZR_VAMPIRE_CAPTURE_HEIGHT=720 ZR_VAMPIRE_CAPTURE_TICKS=60 CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-vampire-app cargo test -p zircon_runtime --lib vampire_project_session_capture_frame_draws_hud_panel --locked --message-format short -- --nocapture --test-threads=1: passed 2026-06-10; exported a non-empty runtime frame PNG"
  - "CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-vampire-app cargo build -p zircon_app --bin zircon_runtime --locked --message-format short: passed 2026-06-10; existing warnings only"
  - "D:\\cargo-targets\\zircon-vampire-app\\debug\\zircon_runtime.exe --project E:\\Git\\ZirconEngine\\examples\\vampire: launched 2026-06-10 and remained running after an 8 second startup check; no stderr/stdout and no runtime-session creation error"
  - "cargo check -p zircon_runtime --lib --locked --message-format short with CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-vampire-app: passed 2026-06-10 after switching vampire health presentation to screen-space HUD; existing zircon_runtime warnings only"
  - "cargo test -p zircon_runtime --lib vampire_project_session_ --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-vampire-app: passed 2026-06-10; 10 project-session tests passed after graphical HUD update"
  - "ZR_VAMPIRE_CAPTURE_PNG=E:\\Git\\ZirconEngine\\examples\\vampire\\screenshots\\vampire-runtime-frame.png ZR_VAMPIRE_CAPTURE_WIDTH=1280 ZR_VAMPIRE_CAPTURE_HEIGHT=720 ZR_VAMPIRE_CAPTURE_TICKS=60 cargo test -p zircon_runtime --lib vampire_project_session_capture_frame_draws_hud_panel --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-vampire-app: passed 2026-06-10; exported final PNG with graphical HUD"
  - "cargo test -p zircon_runtime --lib gameplay_host_current_hp_and_particle_sprites_use_dynamic_components --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-vampire-app: passed 2026-06-10; validates gameplay.current_hp and gameplay.set_particle_sprites dynamic component behavior"
  - "cargo test -p zircon_runtime --lib vampire_project_session_buffed_attack_particles_use_buff_palette --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\\cargo-targets\\zircon-vampire-app: passed 2026-06-10; validates combined attack/haste/shield particle style and palette"
  - "rustfmt --edition 2021 --check zircon_runtime/src/script/vm/backend/mod.rs zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs zircon_runtime/src/script/vm/tests.rs: passed 2026-06-12 after removing runtime project fallback backend"
  - "git diff --check -- zircon_runtime/src/script/vm/backend/mod.rs zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs zircon_runtime/src/script/vm/tests.rs docs/zircon_runtime/script/vm/zr_vm_host_reflection.md docs/zircon_runtime/dynamic_api/session.md: passed 2026-06-12 with LF-to-CRLF warnings only"
  - "cargo test -p zircon_runtime --lib discovery_rejects_zr_vm_project_fallback_backend --locked --jobs 1 --target-dir E:\\cargo-targets\\zircon-runtime-no-fallback-0612 --message-format short -- --nocapture --test-threads=1: timed out after 604s on 2026-06-12 during Windows test-target compilation; orphaned cargo/rustc processes were stopped and no Cargo pass is claimed"
doc_type: module-detail
---

# ZrVM Host Reflection

The VM host surface is split into three ownership layers:

- `zircon_runtime_interface::reflect` owns the authoritative type path, ordered field schema, visibility, and documentation in `ReflectTypeRegistration`.
- `zircon_runtime::core::framework::script` owns neutral host ABI descriptors and values. `ScriptHostTypeDescriptor` is a fallible projection of the unified registration plus ABI-only value/prototype/construction options; it is not a second field schema. VM backends can read `ScriptHostModuleDescriptor`, `ScriptHostFunctionDescriptor`, `ScriptHostTypeDescriptor`, `ScriptHostValueKind`, `ScriptHostCallContext`, and `ScriptHostResult` without depending on concrete runtime managers.
- `zircon_runtime::script::vm::host` owns registration, handle allocation, validation, capability checks, and callback dispatch.

VM code never receives Rust object pointers. Host objects are represented as `HostHandle` values, and framework-level values carry those handles as `u64` so the neutral contract does not depend on the VM subsystem.

`zircon_runtime_reflection_macros` is the convenience layer for Rust-authored host libraries. `ZirconScriptType` emits a script-public `ReflectTypeRegistration`, then projects it into the neutral ABI descriptor; field names, field type paths, ordering, and documentation therefore have one source of truth. `zircon_host_function` and `zircon_host_module` add function/module ABI metadata and propagate projection failures as typed host-module registration errors. Function parameters derive their exported type names from `ScriptHostFromValue::script_host_type_ref`, so Rust `f64` exports as the VM-facing `float` type instead of leaking a Rust-only spelling into ZrVM native module metadata.

`ZrReflect` derive output also supplies `u32` field-slot accessors. `ScriptCallTable` uses names only while compiling a module, then retains numeric type/member slots and invokes `ReflectComponent` dense callbacks. The regression test installs both named and slot callbacks and asserts that repeated runtime read/write calls increment only the dense callback counter.

## Gameplay Host Surface

`builtin_host_modules.rs` now registers `zr.zircon.gameplay` beside foundation, asset, scene, render, and math host modules. The gameplay module is intentionally narrow and script-frame scoped: `runtime_context.rs` stores the current `CoreWeak`, `LevelSystem`, entity id, and delta time while a scene script export is running, and host functions reject calls when that context is absent.

The implemented capabilities are:

- `gameplay.input`: `key_pressed(key)` reads the current input snapshot.
- `gameplay.entity`: entity id, world position, translate/set-position, face-direction yaw, scale updates, position following, camera follow, dynamic component JSON helpers, simple entity find/spawn/despawn helpers, nearest script-property targeting, script-HP damage resolution, current script HP lookup, and dynamic particle-sprite authoring.
- `gameplay.navigation`: next-point query and `nav_move_towards_entity`, which asks the navigation manager for a path and falls back to direct steering if no loaded navmesh is available.

`scene_hook.rs` contributes fixed-update and update hooks under plugin-prefixed ids `zr_vm_language.script.scene.fixed_update` and `zr_vm_language.script.scene.update`. It reads the `script.bindings` dynamic component imported from `SceneAsset.script_bindings`, calls `onStart(entity, dt)` once per binding on the first update, then calls `onFixedUpdate(entity, dt)` or `onUpdate(entity, dt)` as the scene tick runs. Bindings can opt out of a phase with `fixed_update = false` or `update = false`; when no binding is active for a phase, the hook returns before resolving the VM manager. Export calls go through `VmPluginManager::call_package_export`, so script-bound scene entities can target packages by manifest name rather than a transient slot index.

Runtime 15 F5 script scene hook typed errors (`runtime_15_script_scene_hook_typed_errors_static_passed_cargo_deferred`) keeps those hook ids, phase filters, lifecycle export names, and `script.bindings` schema unchanged while moving hook-local failures into `script/vm/scene_hook/error.rs`. `ScriptSceneHookError` / `ScriptSceneHookResult` preserve manager resolve, `script.bindings` JSON parse, and VM export-call sources until `SceneRuntimeHook::run(...)` converts the final diagnostic into `CoreError::Initialization`; `review_f5_script_scene_hook_uses_typed_errors_before_core_boundary` locks that boundary and rejects `Result<_, String>` rollback inside `scene_hook.rs`.

The `examples/vampire/scripts/vampire_game` package demonstrates this surface. `main.zr` moves the player with WASD through `gameplay.key_pressed` and `gameplay.translate`, updates a third-person camera with `gameplay.camera_follow`, lets enemies chase the player with `gameplay.nav_move_towards_entity`, and runs Blood Bolt auto-targeting through `gameplay.nearest_by_script_property` plus `gameplay.damage_entity`. The example also uses `gameplay.face_direction`, `gameplay.set_scale`, `gameplay.follow_position`, `gameplay.current_hp`, and `gameplay.set_particle_sprites` for visible action-state feedback: moving and attacking actors face their target direction, pose scale changes distinguish idle/run/attack states, the HUD displays real player HP from script bindings, attacks emit buff-colored particle sprites, and the player blood-aura point light follows the player entity.

## Project Backend Boundary

`VmPluginManager::with_plugin_context_and_host_exports` registers the `builtin` backend family for mock/unavailable testing paths and the real `zr_vm` backend family for project packages. Project script manifests that carry a `[zr_vm]` section must use `backend = "zr_vm:project"`; `zr_vm_fallback:project` and other project fallback selectors are intentionally rejected during package discovery.

The runtime VM subsystem must not contain example-specific Rust gameplay fallbacks. The `examples/vampire` package is expected to run through its authored ZR script plus the generic `zr.zircon.gameplay` host API surface. Any Vampire-specific behavior belongs in the example project assets/scripts or in external first-party plugin code, not in `zircon_runtime/src/script/vm/backend`.

The vampire player scene now binds `vampire_locomotion.state_machine.zranim`, which switches between idle, move, and melee attack graph assets that reference imported Kenney glTF animation clips. The authored script updates movement and attack parameters through generic gameplay host calls, so the animation manager can evaluate the state machine alongside normal transform/action-state feedback. Kenney's graveyard character GLBs are node-transform animation packs rather than skinned meshes; the runtime animation hook maps sampled pose bones back onto descendant scene nodes by name while the skinned-mesh raster path remains a separate renderer concern.

## Type Reflection Model

`ScriptHostValueKind` remains the coarse ABI-lowering category used by host calls. `ScriptHostTypeRef` carries the VM-facing type name beside that value kind, allowing a host function to lower as `Float` while still registering a semantic type such as `Vec3`, `ColorRgba`, or `float` with a backend. Function and parameter descriptors default primitive type refs from their value kind, while type descriptors default the type ref name to the descriptor name so handwritten semantic types do not accidentally collapse back to `float` or `int`.

`ScriptHostPrototypeKind` describes the VM prototype that should be used for a host type: module, class, interface, struct, enum, or native. The derive macro defaults Rust structs to `Struct` and Rust enums to `Enum`; callers can still override the prototype with `#[zircon_script(prototype = ...)]` when a host type intentionally maps to another VM shape. `ScriptHostTypeDescriptor::allow_value_construction` records whether the VM may construct values directly from the reflected descriptor. These fields are intentionally descriptor data only; scripts still receive values or `HostHandle` identifiers, not Rust object pointers.

The conversion traits are the Rust-side source of default type refs:

- `ScriptHostFromValue` converts script arguments into Rust parameters and exposes the exported argument type ref.
- `ScriptHostIntoValue` converts Rust return values into `ScriptHostValue` and exposes the exported return type ref.
- `ZirconScriptType` produces a complete `ScriptHostTypeDescriptor` for Rust-authored value types.

The macro entry points reject unsupported Rust shapes instead of emitting descriptors that would fail later through trait-bound or runtime errors. `zircon_host_function` supports synchronous, non-generic free functions with simple identifier parameters. `ZirconScriptType` supports non-generic structs and enums; unions are rejected. Async functions, generic functions, and generic script types must be wrapped in a concrete host export before reflection.

The proc-macro crate is a separate workspace member because Rust procedural macros must live in a `proc-macro` crate. `zircon_runtime` re-exports the macros so runtime-owned host modules can write `#[crate::zircon_host_function]`, `#[crate::zircon_host_module]`, and `#[derive(crate::ZirconScriptType)]` without depending on the macro crate directly.

The macro crate is split by code-generation responsibility. `lib.rs` contains only the Rust-required proc-macro entry points and module declarations. `args.rs` owns attribute argument parsing, `attrs.rs` owns `#[zircon_script]` parsing and item discovery, `derive_type.rs` emits `ZirconScriptType` descriptors, `function.rs` emits host function descriptors and callbacks, `module.rs` emits host module descriptors and registration functions, `tokens.rs` owns shared token helpers, and `tests.rs` covers unsupported input plus descriptor metadata generation. Runtime validation remains in `HostExportRegistry`; the macro crate only rejects Rust shapes that cannot be represented correctly as host descriptors.

Runtime 15 F12 script reflection macro fixture dead-code cleanup keeps the runtime-side macro descriptor fixtures from hiding unused fields behind `#[allow(dead_code)]`. `script/vm/tests/reflection_docs.rs` now constructs and reads the TestVec3 fields, exercises `TestEnum::A` with `matches!`, and reads the nested Point fixture through `macro_math::point_fixture_x()` while still validating the emitted `ZirconScriptType` and `zircon_host_module` descriptors. Status: `runtime_15_script_reflection_macro_fixture_dead_code_cleanup_static_passed_cargo_deferred`; guard: `runtime_15_script_reflection_macro_fixtures_do_not_suppress_dead_code`.

## Registry Behavior

### VM generation reflection catalog

`VmReflectionSchema::from_state_schema` is the single projection from lifecycle state metadata into callable runtime reflection. It selects only public component-only registrations and validates their standalone schema rules. The coordinator asks `VmReflectionCatalog` to prepare a generation from a clean runtime-owned builtin registry plus the candidate before activation; it never samples a managed World's local registry and no public API can manufacture or commit a candidate. Preparation also binds every registration to the trusted package manifest name instead of accepting a self-reported foreign namespace. World-local direct registrations participate only in collision validation across every managed World and cannot silently become process-wide dense ABI metadata.

`VmReflectionCatalog` owns the process-wide slot/generation view and serializes every prepare/commit/discard transaction. Every changed candidate receives a unique checked epoch plus the committed epoch it was based on. Name resolution may consume that exact prepared snapshot while activation runs, but dense read/write stays closed until its epoch is the committed epoch. Commit first proves that the prepared handle carries the same catalog-owned epoch capability, including before no-op early returns, then rechecks the base epoch and rejects a candidate if another transaction won first; a prepared generation from another catalog can never be consumed even when both catalogs happen to use the same numeric epochs. Two different candidates can never become current merely because they share the same future revision number. Equal generations are idempotent only when owner and schema are identical; conflicting metadata is a typed generation error and reuses neither revision nor epoch. `HotReloadCoordinator` serializes its load/reload/unload lifecycle mutations, reads `state_schema` exactly once per generation, installs the prepared candidate before activation, and commits that same prepared handle to Worlds before the slot becomes active. The level manager locks the level set and all Worlds, validates retained payloads, synchronizes every World, and only then release-publishes catalog state, revision, and epoch. Unload keeps its slot in an explicit unloading state until deactivate and catalog discard both succeed; failures restore the prior generation/interface snapshot and never let the manager discard host interfaces unconditionally.

Plugin `ScriptCallTable` tokens are monotonic opaque identifiers, not packed type/member slots. Each table owns the token-to-dense-site map, so a token from an older table cannot resolve in a replacement table even when slot numbers coincide. Production tables retain the exact prepared/current epoch guard: the public `ScriptCallTable::resolve` owner itself checks the name-resolution capability, a prepared table can resolve package-loading names, dispatch fails closed before commit, exact commit activates the same token, and a foreign commit revokes both direct name resolution and dispatch even when its numeric revision matches. `ReflectionHostModule` refreshes only from the committed catalog snapshot after that stale transition.

The concrete ZrVM implementation is plugin-owned. `zircon_runtime` retains only backend-neutral contracts, lifecycle coordination, the canonical reflection catalog, and host registries; it no longer declares a `backend-zr-vm` feature or a duplicate concrete backend module.

Private VM types are intentionally absent from this catalog even when they participate in state migration. Script visibility is an authorization boundary, not an editor-only presentation hint.

`HostExportRegistry` validates a module before it becomes visible:

- module, version, capability, type, function, and parameter names must be non-empty and trimmed;
- module names, type names, function names, and parameter names must not duplicate within their scope;
- type, field, parameter, and return `ScriptHostTypeRef` names must be non-empty and already trimmed;
- every type ref value kind must match the descriptor value kind that will be used for call lowering;
- a registered type descriptor's own `type_ref.type_name` must match its descriptor name;
- field names must not duplicate within a reflected type;
- function arity must be coherent with its parameter list;
- function required capabilities must be declared on the module;
- callbacks must exactly match declared function names.
- duplicate callback names are rejected before callback storage, so a later callback cannot silently replace an earlier one.

Each registered module receives a `HostHandle` through the shared `HostRegistry`, using a `host.module.<module>` capability label. This keeps script-visible handles stable and lets existing handle validation continue to work.

Direct `HostExportRegistry` callers go through `call_with_capabilities`: the registry checks arity and required capabilities before building a `ScriptHostCallContext` and dispatching the callback, and such callers pass the package capability set from `VmPluginHostContext`. The production ZrVM reflection backend follows a different hot path: `real_backend/host_modules.rs` installs a prepared `ScriptCallTable`, resolves names once while loading, and dispatches reflected scene fields only through opaque tokens and dense callbacks guarded by the committed catalog epoch.

Runtime 15 M3 script VM registry lock poison recovery status: `runtime_15_script_vm_registry_lock_poison_recovery_static_passed_cargo_deferred`.

The VM registry surfaces now recover poisoned internal mutexes instead of panicking in production paths. `VmBackendRegistry` owns `lock_families()` for backend family registration and selector resolution. `HostRegistry` owns `lock_handles()` for capability handle allocation and lookup. `HostExportRegistry` owns `lock_modules()` for descriptor/callback storage, module snapshots, script call table generation, and capability-checked dispatch. `HotReloadCoordinator` owns `lock_slots()` for package load, hot reload, slot restoration, unload, package-name lookup, export calls, list projection, and debug slot-count projection.

This does not change VM backend traits, host descriptor validation, capability enforcement, script call-table dispatch, slot generation, or hot-reload lifecycle states. The new module-local recovery tests are `vm_backend_registry_accessors_recover_poisoned_family_lock`, `host_registry_accessors_recover_poisoned_handle_lock`, `host_export_registry_accessors_recover_poisoned_module_lock`, and `hot_reload_coordinator_accessors_recover_poisoned_slot_table_lock`. The cross-module guard `runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries` keeps those helpers, production direct-lock scans, and Runtime 15/status anchors synchronized.

Runtime 15 M3 script VM hot-reload coordinator test folder split status: `runtime_15_script_vm_hot_reload_coordinator_tests_folder_split_static_passed_cargo_deferred`.

Runtime 15 M3 script VM hot-reload guard child-owner split status: `runtime_15_script_vm_hot_reload_guard_child_owner_split_static_passed_cargo_deferred`.

The structure guard owner for the hot-reload coordinator split now lives at `tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests/hot_reload.rs`, with `runtime_15_script_vm_hot_reload_guard_is_child_owner` preventing the coordinator checks from returning to the parent script VM test-budget guard.

`HotReloadCoordinator` now keeps production slot lifecycle logic in `hot_reload_coordinator.rs` while module-local tests live in `hot_reload_coordinator/tests.rs`. The child owner contains the policy recording backend, lifecycle-query backend, slot lifecycle fixture, and the five hot-reload/poison-recovery tests. This keeps the production owner focused without changing VM backend traits, slot generation, hot-reload policy semantics, host lifecycle queries, host export dispatch, or poison recovery behavior.

The structure guard `runtime_15_script_vm_hot_reload_coordinator_tests_are_folder_backed` keeps the parent mount, moved-test scan, test count, line budget, Runtime 15/status rows, and this document synchronized.

Runtime 15 M3 VM plugin manager selected-backend lock poison recovery status: `runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_static_passed_cargo_deferred`.

`VmPluginManager` now recovers poisoned selected-backend `RwLock` state through `selected_backend_read()` and `selected_backend_write()`. The public `selected_backend_name()` and `select_default_backend(...)` paths no longer direct unwrap read/write guards, so a panic while updating the selected backend cannot permanently break later selector reads or writes.

This does not change backend family resolution, plugin package discovery, slot lifecycle calls, host export dispatch, or default backend selector values. The module-local `vm_plugin_manager_selected_backend_accessors_recover_poisoned_lock` test poisons the selected-backend lock, verifies the default selector is still readable, then switches to `builtin:mock`. The structure guard `runtime_15_vm_plugin_manager_selected_backend_lock_poison_recovery_guard_covers_manager_selector` keeps the helper shape, production direct RwLock unwrap scan, Runtime 15 plan/status rows, and this document synchronized.

## Built-In Modules

`PluginHostDriver::default()` registers first-wave built-in host modules:

- `zr.zircon.foundation`: time, log, and event helper descriptors.
- `zr.zircon.asset`: locator/status/revision query descriptors.
- `zr.zircon.scene`: default world handle and handle validity helpers.
- `zr.zircon.render`: read-only render metadata descriptors.
- `zr.zircon.math`: pure value descriptors and deterministic vector helpers.

`zr.zircon.math` is registered through the reflection macros. `Vec3` and `ColorRgba` derive `ZirconScriptType`, and pure helpers such as vector length and dot product use `zircon_host_function`, proving that macro-generated descriptors flow through the same registry validation and dispatch path as handwritten modules.

This first wave deliberately favors stable values and handles over concrete manager references. Manager-backed behavior can replace the diagnostic placeholders once the target services expose stable trait-object access through `core::manager`.

## Generated Interface Documentation

Generated ZrVM host interface documentation is descriptor-driven. `ScriptHostModuleDescriptor`, `ScriptHostTypeDescriptor`, `ScriptHostFunctionDescriptor`, `ScriptHostParameterDescriptor`, `ScriptHostFieldDescriptor`, and `ScriptHostTypeRef` remain the source of truth; the Markdown renderer reads those descriptors instead of reflecting Rust implementation details or querying a backend-specific ABI. Built-in documentation uses `builtin_host_module_descriptors()` to register the same first-wave host modules into a local registry and then renders the validated descriptor records.

The renderer keeps output deterministic for review and generated-file comparison. Modules are sorted by module name, capabilities are sorted by capability string, reflected types are sorted by type name, and functions are sorted by function name. Field order and function parameter order stay descriptor-defined because those sequences describe user-facing struct layout and call signatures.

The writer command is explicit-output only:

```powershell
cargo run -p zircon_runtime --bin zircon_host_reflection_docs --locked --offline --jobs 1 --target-dir F:\cargo-targets\codex-reflection-docs -- F:\cargo-targets\codex-reflection-docs\host-interface.md
```

The command requires exactly one output Markdown path, creates missing parent directories through the writer API, and does not commit a machine-specific generated artifact path into the repository. Callers choose where generated interface documentation is emitted.

Runtime 15 F5 host reflection docs CLI typed errors
(`runtime_15_host_reflection_docs_cli_typed_errors_static_passed_cargo_deferred`) keeps that writer
command folder-backed and typed internally. `zircon_host_reflection_docs.rs` is now only the CLI
entry shell; `args.rs` owns the explicit-output argument contract, `error.rs` owns
`HostReflectionDocsError` / `HostReflectionDocsResult`, and `run.rs` owns descriptor collection plus
Markdown writer handoff. `HostReflectionDocsError::CollectBuiltInHostModules` preserves the
`VmError` source from `builtin_host_module_descriptors()`, while
`HostReflectionDocsError::WriteHostInterfaceDocs` preserves the output path and `std::io::Error`
source from `write_script_host_modules_markdown(...)`. The final string is produced only by
`main.rs` for process stderr; `review_f5_host_reflection_docs_cli_uses_typed_errors_before_cli_boundary`
locks the no `Result<_, String>` rollback.

The built-in math module is the proof that handwritten and macro-generated descriptors flow through one documentation path. `zr.zircon.math` is registered through the reflection macros, then rendered from the same descriptor model as the handwritten built-ins; Milestone 3 generated output was inspected and contained `zr.zircon.math` at line 76.

## Real Backend Lowering Boundary

The real `zr_vm` backend treats `HostExportRegistry` records as already validated neutral descriptors, then applies only target-backend lowering checks. Function arity must fit the `zr_vm` native function ABI (`u16` min/max bounds), `min_argument_count` must not exceed `max_argument_count`, and reflected parameter count must fit the maximum arity. These are backend constraints, not shared descriptor constraints for every future VM backend.

Native callbacks convert ZrVM null, bool, int, float, and string arguments into `ScriptHostValue`, then invoke the pre-resolved `ScriptCallSite::call(arguments, capabilities)` captured while the host module is installed. Runtime callbacks therefore do not repeat module/function name lookup. Host return values lower null, bool, int, float, string, bytes as lossy UTF-8 strings, and `HostHandle` as integers. Unsupported ZrVM argument kinds remain errors with module/function context rather than lossy conversions.

## VM Plugin Management Policy

VM plugin manifests now carry a neutral `management` policy block. The default policy keeps existing behavior: hot reload preserves VM state through `save_state` and `restore_state`, garbage collection is backend-managed, and no soft or hard memory limits are declared. This keeps old bytecode and `zr_vm:project` packages compatible while giving project tooling a stable place to declare lifecycle and memory expectations.

The hot-reload policy has three modes. `preserve_state` saves state from the active slot, deactivates it, loads and activates the replacement instance, and restores the saved state. `stateless` deactivates the old instance and activates the replacement without calling state transfer hooks. `disabled` rejects hot reload before deactivation, leaving the active slot untouched.

The garbage-collection policy is descriptive at this layer. `backend_managed` means the backend owns collection timing, `cooperative` can declare an `interval_frames` cadence for future host-driven collection, and `disabled` forbids an interval. Memory policy can declare `soft_limit_bytes` and `hard_limit_bytes`; invalid zero limits or a soft limit above the hard limit are rejected during package discovery before the package reaches a backend.

Runtime 15 F5 VM plugin management policy typed errors (`runtime_15_vm_plugin_management_policy_typed_errors_static_passed_cargo_deferred`) keeps that schema and behavior unchanged while moving policy validation failures into `script/vm/plugin/management_policy/error.rs`. `VmPluginManagementPolicyError` / `VmPluginManagementPolicyResult` now cover disabled GC interval, zero GC interval, zero memory limits, and soft limit exceeding hard limit before package discovery hands the policy to a backend. `review_f5_vm_plugin_management_policy_uses_typed_validation_errors` locks the error owner, validate signatures, script facade exports, and status/docs anchors so `Result<(), String>` does not return to this boundary.

`script/vm/plugin/mod.rs` also re-exports `VmPluginManagementPolicyError` and `VmPluginManagementPolicyResult` from the management-policy owner. That keeps `script/vm/mod.rs` and `script/mod.rs` as typed public facades after the error owner split without adding a compatibility shim or changing the management policy schema.

`VmPluginSlotRecord` exposes the resolved management policy beside each loaded slot, plus a monotonic `generation` and a lifecycle state. Initial loads start at generation 1, and successful hot reload increments the generation. The status projection uses `active` for a running instance, `reloading` while the coordinator is saving state, deactivating, loading, activating, or restoring a replacement, and `failed` when a reload step fails after the original instance has already left the clean active path. Reload hooks run without holding the coordinator slot-table lock, so a VM plugin can safely query its slot lifecycle facade during `activate` or `restore_state` and still observe the transient `reloading` record. This gives editor, Hub, export, and diagnostics surfaces a single read-only status projection without needing to inspect backend internals. Real GC execution and live memory measurements remain backend follow-up work; this slice only establishes the neutral contract and lifecycle bookkeeping that those backends can report through.

Example package policy:

```toml
[management]
hot_reload = "stateless"

[management.garbage_collection]
mode = "cooperative"
interval_frames = 120

[management.memory]
soft_limit_bytes = 104857600
hard_limit_bytes = 268435456
```

## Package Protocol

`discover_vm_plugin_package` still supports bytecode packages. A ZrVM project package uses:

```toml
backend = "zr_vm:project"

[zr_vm]
project = "plugin.zrp"
entry_module = "main"
execution_mode = "binary"
```

Project packages store no bytecode in `VmPluginPackage::bytecode`; instead they populate `VmPluginPackage::zr_vm_project` and `VmPluginPackageSource::zr_vm_project_path`.

The checked-in minimal example lives at `docs/zircon_runtime/script/vm/examples/zr_vm_minimal`. It uses the same package protocol, imports `zr.zircon.foundation`, calls `foundation.time_unix_millis()` and `foundation.log_info()` from `activate()`, and demonstrates hot-reload state through optional `saveState(): string` and `restoreState(state: string)` exports. The real ZrVM plugin tests copy that example to a temporary package root before loading it so validation does not leave compiled artifacts under `docs/`.
