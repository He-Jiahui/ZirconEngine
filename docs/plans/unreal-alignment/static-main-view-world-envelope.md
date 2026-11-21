# Static main-view prefilter can drop visible geometry under a scaled orthographic camera

## Scope

At 10,000 static mesh instances the main view queries a world-space cell index before testing individual mesh bounds. With a scaled Scene camera, its query sphere can be smaller than the visible world volume. Geometry removed by this first stage cannot be recovered by the subsequent plane tests. This is separate from [#218](https://github.com/He-Jiahui/ZirconEngine/pull/218) (orthographic per-sphere planes), [#138](https://github.com/He-Jiahui/ZirconEngine/pull/138) (perspective per-sphere planes), [#220](https://github.com/He-Jiahui/ZirconEngine/pull/220) (Editor picking rays), and [#223](https://github.com/He-Jiahui/ZirconEngine/pull/223) (previous-pose HZB).

## Measured on the unchanged Windows Runtime package

An authored orthographic Scene camera inherits `Sx=64` from its parent (`half-height=1`, square view, `near=1`, `far=8`). Two root meshes at world `X=±40` are entirely inside the actual projection, with clip depths safely inside both planes. They already pass the old individual-sphere test. Each occupies a frozen, edge-excluding region of 468 pixels in the corresponding frame. A central static mesh, an independently positioned dynamic mesh also at `X=40`, and genuinely out-of-view geometry provide controls. The 9,996 count-filling meshes are far outside the view.

| Real RTX DX12 / public main-view counters | Prefilter used | Static input | Static candidates | Covered pixels, each static witness |
| --- | ---: | ---: | ---: | ---: |
| One distant filler remains dynamic | No | 9,999 | 9,999 | 468/468 |
| Same filler becomes static, otherwise same scene and camera | Yes | 10,000 | 1 | **0/468** |
| Same static scene, only `far=192` (pre-existing cell-budget fallback) | No | 10,000 | 10,000 | 468/468 |

In a separate live run, the same 10,000 static instances and `far=8` stay fixed while the camera parent moves `X=0,+14,−14,0`. Static candidates are `1,2,2,1`. Neither, only the right, only the left, then neither static witness is drawn. Both meshes remain entirely inside the projected view in every pose. Central and dynamic controls stay visible, the outside control stays absent, and observed index rebuild/insert/update/remove counts remain zero after startup. Across both experiments, 70 real hardware frames were captured; fixed tail frames, public per-frame diagnostics, actual cooked scene/mesh/material bindings, and independent geometry agreed. These are packaged baseline observations, not an execution of a new frozen Rust reproduction or a candidate.

## Ownership and intended invariant

[The main-view visibility constructor](../../../zircon_runtime/src/graphics/visibility/context/history/construct.rs#L288-L381) queries a camera-centered sphere with radius `sqrt(far² + halfWidth² + halfHeight²)` in view units. [The grid](../../../zircon_runtime/src/graphics/visibility/static_index/mod.rs#L275-L301) consumes it as a world-space box. In this case the query radius is about `8.124`, touching only 8 cells, although the actual frustum extends `±64` in world X relative to the eye. A sphere around the eye containing the actual eight corners has radius `sqrt(4161) ≈ 64.506`, still within the existing 4,096-cell budget (900–1,000 cells in these measured poses). This is a conservative example, not a mandate to disable prefiltering or multiply all axes by 64.

Use the effective selected view, with its actual projection and coordinate frame, to derive conservative world coverage. The [ZE view projection](../../../zircon_runtime/src/core/framework/render/view_matrix_pair.rs#L12-L33) and [Three.js r180 `matrixWorldInverse`](https://github.com/mrdoob/three.js/blob/0af9729d0c143a86a1d725d6e2c3ad83301f3f34/src/cameras/Camera.js#L92-L104) frame the mathematical comparison. Unreal's optional `CullOctree` tests world node bounds against the same `ViewCullingFrustum` used for primitive tests; compare the division of responsibility, not Unreal camera scale or `FTransform` behavior. Keep the 10,000-instance gate, 4,096-cell budget and conservative fallback; do not alter authored transforms, per-sphere planes, shared pick queries, mesh resource identity, or the ABI. Untrusted/unrepresentable bounds must not create false negatives. This witness does not decide arbitrary projection overrides, shear optics, or perspective policy.

## Before implementation and acceptance

- Freeze a focused native CPU witness on unchanged production with a real Scene Camera and finite prepared mesh bounds. Prove actual visible stable keys, not just counts. The 9,999-instance, unit-pose, dynamic and outside controls must pass; the at-threshold scaled case should reach an enabled prefilter and fail only because its two keys are missing. Include a same-history move/restore and budget-exceeded control. Record the exact executable and late failure before a production change.
- Replay the same CPU selectors and both hardware fixture families with the candidate. All genuinely visible static keys and their pixels must survive **with bounded prefiltering still enabled** in the short-far case; the over-budget case must still fail open. Preserve independent compatibility controls and existing required checks. No CI activation or merge is part of this problem-only Draft.
