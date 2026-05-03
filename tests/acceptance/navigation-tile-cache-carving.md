# Navigation TileCache Carving

## Scope
- `zircon_plugin_navigation_recast` now exposes a private DetourTileCache path-query owner behind `RecastBackend::find_path_with_obstacles(...)`.
- `zircon_plugin_navigation_runtime` now collects carving `NavMeshObstacle` dynamic components and routes agent path queries through the carved backend path when a loaded navmesh and carving obstacles are present.
- The affected layers are native Recast/Detour C ABI, plugin-local Rust backend wrappers, and runtime navigation manager obstacle query flow.

## Baseline
- The previous Detour query milestone passed focused native and runtime navigation Cargo tests before this TileCache slice.
- Current focused Cargo validation is blocked before navigation tests execute by unrelated active renderer compile drift in `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs`.
- Windows PATH only has GCC 4.8.3, which cannot compile the C++17 native boundary used by existing navigation C++ files. WSL has `g++ 11.4.0` and was used for native TileCache evidence.

## Test Inventory
- Native C ABI positive/blocking case: a three-square corridor with one box obstacle spanning the middle corridor should create a TileCache query and return `NoPath`.
- Rust native regression: `tile_cache_carved_obstacle_blocks_corridor_path` should assert the same behavior through `RecastBackend::find_path_with_obstacles(...)` once Cargo reaches test execution again.
- Runtime regression: `carved_runtime_obstacle_blocks_agent_path_on_loaded_navmesh` should assert a loaded-navmesh agent is blocked by a carving world obstacle once Cargo reaches test execution again.
- Boundary case covered in code review: the original test obstacle left edge strips walkable for a zero-radius agent, so the regression obstacle now fully spans corridor width.

## Tooling Evidence
- Tool: WSL `g++ 11.4.0`.
- Reason: Windows Cargo validation is blocked in unrelated Rust renderer code, and Windows `g++ 4.8.3` cannot compile the C++17 native boundary.
- Command shape: `wsl.exe --cd /mnt/e/Git/ZirconEngine --exec sh -lc "g++ -std=c++17 -DDT_VIRTUAL_QUERYFILTER -I. -Izircon_plugins/navigation/native/vendor/recastnavigation/Recast/Include -Izircon_plugins/navigation/native/vendor/recastnavigation/Detour/Include -Izircon_plugins/navigation/native/vendor/recastnavigation/DetourCrowd/Include -Izircon_plugins/navigation/native/vendor/recastnavigation/DetourTileCache/Include zircon_plugins/navigation/native/tests/tile_cache_smoke.cpp zircon_plugins/navigation/native/native/detour_tile_cache.cpp zircon_plugins/navigation/native/native/detour_query.cpp zircon_plugins/navigation/native/vendor/recastnavigation/Recast/Source/*.cpp zircon_plugins/navigation/native/vendor/recastnavigation/Detour/Source/*.cpp zircon_plugins/navigation/native/vendor/recastnavigation/DetourCrowd/Source/*.cpp zircon_plugins/navigation/native/vendor/recastnavigation/DetourTileCache/Source/*.cpp -o /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_tile_cache_smoke && /mnt/c/Users/HeJiahui/AppData/Local/Temp/opencode/zircon_tile_cache_smoke"`.
- Observed output: `create status=1 polygons=3 obstacles=1 message=TileCache query created` and `path status=2 points=0 visited=0 length=0.000000 message=TileCache path query found no complete path`.

## Results
- Passed: WSL native TileCache smoke harness built and returned `NoPath` for the carved corridor case.
- Blocked: `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_navigation_recast tile_cache_carved_obstacle_blocks_corridor_path --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never -- --exact --nocapture` stopped during `zircon_runtime` compilation before navigation test execution.
- Blocked: `cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_navigation_runtime carved_runtime_obstacle_blocks_agent_path_on_loaded_navmesh --locked --jobs 1 --target-dir E:\cargo-targets\zircon-navigation-validation --message-format short --color never -- --exact --nocapture` stopped during `zircon_runtime` compilation before navigation test execution.
- Fixed in response: the TileCache corridor regression obstacle was widened from `[0.45, 1.0, 0.45]` to `[0.55, 1.0, 0.6]` so a zero-radius query cannot legally route around the obstacle through edge strips.

## Acceptance Decision
- Native C ABI behavior is accepted with direct WSL evidence.
- Rust/native and runtime Cargo acceptance are blocked by unrelated `zircon_runtime` renderer compile drift, not by a navigation test failure.
- Remaining risks: TileCache data is transient per query, obstacle updates are not incremental across frames, and DetourCrowd simulation is not implemented yet.
