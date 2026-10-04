---
related_code:
  - zircon_runtime/tests/runtime_shader_pbr_realtime_ibl_export.rs
implementation_files:
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/scene_fixtures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_shader_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_graph_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_compute_executor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_wgpu_dispatch/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/ibl_irradiance_sh.wgsl
plan_sources:
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
tests:
  - realtime_ibl_export_contract_uses_requested_matrix_and_unreal_slice_count
  - export_procedural_realtime_ibl_pbr_matrix_png
  - export_procedural_realtime_ibl_mirror_cardinal_120deg_png
---

# Procedural realtime IBL PBR export

This product-level integration test renders the standard 8x8 metallic/smoothness matrix through `SceneRenderer` with a procedural sky.

The ignored export first renders one complete real-time IBL publication. It then changes the procedural sky bake identity and renders the same snapshot for 21 compute frames: three sky-capture batches, seven source-mip batches, ten PMREM batches, and one terminal `ProjectDiffuseSh9` batch. A twenty-second presentation-only frame samples the newly published ready slot and is saved only below `docs/tests/runtime/shader`.

The CPU companion record reports wall time for the initial full update and each sliced update frame. The GPU record comes from production WGPU timestamp queries around the realtime IBL command batch; it reports the complete publication and all 21 sliced batches independently from CPU submission and polling time. The two generations therefore provide 42 timestamp samples when the product export runs successfully.

The product gate also compares the heaviest sliced GPU batch with the initial complete publication. A sliced batch must remain below 75% of the full update, preventing the final SH9 state from silently serializing the complete projection and recreating a frame-time spike.

Setting `ZR_RENDERDOC_CAPTURE_REALTIME_IBL_FINAL_SH9=1` wraps only the twenty-first update batch in the graphics-debugger capture API. That batch owns terminal `ProjectDiffuseSh9` and publication, so a resulting RDC cannot be mistaken for a capture of an unrelated presentation frame.

The dated evidence sections below are historical 16-batch baselines. They remain useful for regression context but do not attest the current 21-batch product contract; a current screenshot, timing report, and RDC are required for that claim.

## External glTF zero-roughness acceptance launch

`export_realtime_ibl_external_gltf_zero_roughness_mirror_png` is the current-source product fixture for the glTF `roughnessFactor = 0` path. It creates its project only under `docs/tests/runtime/shader/.work`, writes its PNG and per-slice GPU timestamp sidecar below `docs/tests/runtime/shader`, and arms RenderDoc only for terminal `ProjectDiffuseSh9` when `ZR_RENDERDOC_CAPTURE_GLTF_ZERO_ROUGHNESS_FINAL_SH9=1`.

Run it only through the coordinator-managed Windows validation owner. When a capture is authorized, launch the managed Cargo child through the locally installed `D:\Tools\renderdoc\renderdoccmd.exe capture` with `--working-dir E:\Git\ZirconEngine`, `--capture-file E:\Git\ZirconEngine\docs\tests\runtime\shader\runtime_shader_pbr_realtime_ibl_external_gltf_zero_roughness_mirror_current_source`, `--wait-for-exit`, and `--opt-hook-children`; set `ZR_RENDERDOC_CAPTURE_GLTF_ZERO_ROUGHNESS_FINAL_SH9=1` in the child environment and execute the exact ignored test `gltf_zero_roughness::export_realtime_ibl_external_gltf_zero_roughness_mirror_png`. RenderDoc appends the frame suffix and `.rdc` itself. This launch contract keeps the capture on E: and prevents generic startup captures from being accepted as the terminal SH9 evidence.

This fixture and its launch recipe are source-level readiness only until the managed run produces fresh PNG, TXT, and replayable RDC evidence.

## Render graph lifetime regression

The product run originally failed before its first draw because all resident A/B cubemap aliases were inserted into render-graph materialization, while a time-sliced compiled graph retains lifetimes only for the aliases used by that frame. `RealtimeIblGpuResources` now keeps the complete A/B allocation and view set resident but binds only resource names present in the current compiled lifetime set. The integration export runs through this strict materialization path, so it guards the production behavior rather than a test-only resource path.

The shared scene fixture also writes generated model, material, and scene assets through `to_project_toml_string(...)` and `PersistedAssetReference`. It does not restore the removed integration-only `to_toml_string()` compatibility surface.

## 2026-07-12 evidence

- Image: `docs/tests/runtime/shader/runtime_shader_pbr_procedural_realtime_ibl_8x8_reflection_20260712.png`
- Image size: 1600x1200, 193192 bytes
- Image SHA256: `360A5020781A21F8BB728BB7FF69358D1320EB689848367DDFBA0504F9293F83`
- Timing: `docs/tests/runtime/shader/runtime_shader_pbr_procedural_realtime_ibl_8x8_timing_20260712.txt`
- Timing SHA256: `F09943A40C943CC43934190157E4D097004A884DDDF92C23F6FA295FD347A653`
- Initial complete publication CPU wall time: 796.750 ms
- Sixteen sliced update frames: 316.679 ms average, 356.000 ms maximum
- Final-source product test: 1 passed, 0 failed, 40.99 seconds

The matrix uses metallic 0 to 1 from left to right and smoothness 0 to 1 from top to bottom. Visual inspection confirms that the warm horizon is reflected in every row, dielectric diffuse color recedes toward the metallic columns, and the reflected horizon becomes more concentrated as smoothness increases. A fixed center-disk probe measured left-to-right saturation falling from 50.6 to 3.4 in the roughest row and from 53.1 to 11.7 in the smoothest row; the rightmost disk luma range increased from 23.6 to 60.9 from rough to smooth. These probes are supporting observations, not replacements for the existing Shader 06 quantitative HDRI gates.

The exact image name was scanned under the repository `target` and the active external cargo target with zero matches. The export therefore leaves render evidence only in `docs/tests/runtime/shader`.

The original EC-M4i image is retained as pre-SH9 comparison evidence. EC-M4j closes multi-view, and the direct-SH9 section below closes the remaining realtime diffuse and GPU timestamp gates.

## Directional procedural-sky multi-view evidence

The procedural gradient was previously rotationally symmetric around the world Y axis, so a yaw validation could only prove camera movement, not cubemap orientation. The production procedural source now has an optional directional sun disk. Sun direction, color, intensity, and angular radius are part of `IblBakeKey`, enter the realtime source capture before source mip, GGX PMREM, and SH9 work, and use the same world direction in the analytical skybox. Sky intensity and rotation remain final-sampling parameters and are not baked twice.

The ignored product test renders one perfect-metal, zero-roughness sphere from five exact perspective camera views: front, pitch +120, pitch -120, yaw -120, and yaw +120 degrees. It asserts nonblank continuous output, a distinguishable mirror region, per-view difference from front, left/right difference, and a visible sun marker that moves to a viewport boundary under the 120-degree orbit.

- Contact sheet: `docs/tests/runtime/shader/runtime_shader_pbr_procedural_realtime_ibl_mirror_cardinal_120deg_contact_sheet_20260712.png`
- Contact sheet: 4000x600, 663618 bytes, SHA256 `B41F470CA6119405AAFB8B5441C0276258F6680353381BFB4230C5FB67BCE9FF`
- Front: SHA256 `0E025C7F02BEB5F0F790C92FFDACC41D461C458526B60C856BC861FF04EF190E`
- Pitch -120: SHA256 `0FA7630B8AB8E7D3282B9DC5CA19E4C61FA6FE9FDA05C4370EC6561FBB37719A`
- Pitch +120: SHA256 `38A8327FB938EBA5BAE3D724287267A257B40B1F71752C4755F2FFB76EC6B294`
- Yaw -120: SHA256 `A34B85605EC58C2799161A4709496AAC48DD5AABE8E379CC28E447CFE4A7DCCB`
- Yaw +120: SHA256 `14A74C8029A3C1976DDFFEBF8B7BFE6EB6FBC219E7C703C1A280651BB7CA2017`
- Final-source regression product test: 1 passed, 0 failed, 94.07 seconds

All five tiles are 800x600 and are arranged in one 4000x600 row without an unused cell. The image names were scanned under repository `target` and the active external cargo target with zero matches.

## Direct GPU SH9 and timestamp evidence

Scene group 0 binding 6 is a fixed 144-byte SH9 uniform. Offline environments upload their artifact coefficients into an owned scene buffer. Realtime environments bind the selected A/B slot's SH9 buffer directly: compute writes it as storage, then subsequent scene draws read the same allocation as uniform. There is no CPU readback or duplicate coefficient copy in the realtime path.

- Image: `docs/tests/runtime/shader/runtime_shader_pbr_procedural_realtime_ibl_sh9_8x8_reflection_20260712.png`
- Image size: 1600x1200, 272246 bytes
- Image SHA256: `6E060927368C0D75678F115B5D110E536C0ABE2E81BD8FB05CDBEFA129FA62FA`
- CPU timing SHA256: `0CD6DDCDC5DB5064CC04F0501731AD03A8E5D08E9DAB9DA91DF912F172A58D68`
- GPU timing SHA256: `5FB2742C6AF8BD7FEF03FD688FA117FD1B83090E4B5FC865F1538949D6CC6746`
- GPU timestamp samples: 17, all nonzero
- Initial complete publication: 4.981760 ms GPU
- Sixteen sliced updates: 0.321728 ms average GPU, 4.472832 ms maximum GPU
- Final SH9 projection slice: 4.472832 ms GPU
- Final-source product test: 1 passed, 0 failed, 50.10 seconds

The saved frame is intentionally rendered after the state-11 submission publishes the work slot. Compared with the pre-SH9 EC-M4i image, the new frame has RGB channel MAE 27.999296, maximum absolute delta 109, and 5,699,574 changed channels out of 5,760,000. This proves that the accepted image samples the new publication instead of the previous analytical-diffuse frame. The active external Cargo target contains zero PNG files.

## 2026-07-14 parallel SH9 closeout evidence

The SH9 kernel now runs as one 64-lane workgroup reduction instead of dispatching 96 groups and serializing every cubemap sample on global invocation zero. A normal DX12 run produced:

- Image: `docs/tests/runtime/shader/runtime_shader_pbr_procedural_realtime_ibl_sh9_8x8_reflection_20260714.png`
- Image size: 1600x1200, 272485 bytes
- Image SHA256: `3FB37653075AD6A6A0BC5BD7A4F9D72919E59F52AD499287C1D843E66C1E536E`
- CPU timing SHA256: `DB74914C6BCCEBE07E2357B1740B67B68E81BD474CA402707EE90C5A0AE73D92`
- GPU timing SHA256: `6BFC288485614E6ECEA34BD13C08FF48B6FA1D6FD00ECE6B33252F94517644ED`
- Initial complete publication: 4.363264 ms GPU
- Sixteen sliced updates: 0.361920 ms average GPU, 1.819648 ms maximum GPU
- Final SH9 projection slice: 1.819648 ms GPU, 41.7% of the complete publication
- Product test: 1 passed, 0 failed, 91.92 seconds

The exact five-view product was regenerated from front, pitch +/-120 degrees, and yaw +/-120 degrees. Its 4000x600 contact sheet is 663618 bytes with SHA256 `B41F470CA6119405AAFB8B5441C0276258F6680353381BFB4230C5FB67BCE9FF`; the product test passed 1/1 in 72.62 seconds. The individual view hashes remain the directional fixtures recorded above.

RenderDoc captured only the final SH9 slice through the opt-in test hook. `runtime_shader_pbr_realtime_ibl_sh9_final_slice_dx12_renderdoc_20260714_capture.rdc` is 40334552 bytes with SHA256 `F5E5CF3EFFD1D16B789A3C06C5457C88028F0CF75B092DDE54EA0409B0094CA8`. The captured product test passed 1/1, and `renderdoccmd replay --loops 1` completed with exit code 0. Images, timing reports, logs, and the RDC are stored only under `docs/tests/runtime/shader`.
