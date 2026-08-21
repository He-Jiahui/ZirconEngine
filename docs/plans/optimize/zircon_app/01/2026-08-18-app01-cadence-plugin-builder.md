Plan: docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
Milestone: P1-19 unfocused cadence and production plugin-builder panic surface
Status: validated

# App01 unfocused cadence and plugin builder

## Delivered

- An unfocused game window now uses a 100 ms low-power interval. Explicit frame requests and an
  earlier runtime deadline still wake the host before that interval; focused and occluded policies
  keep their existing behavior.
- The infallible `PluginGroupBuilder::finish` helper is test-only. Production callers use the typed
  `try_finish` result and cannot hide descriptor-sort failure behind a panic.

## Performance evidence

- Deterministic timer wake bound: `59.9999988 Hz -> 10 Hz`, an `83.333333%` reduction in default
  timer-driven unfocused pumps.
- This slice does not claim App01 P1-19 headless completion. A configurable server scheduler,
  simulation/network cadence separation, overrun reporting and a server exit owner remain open.

## Validation

- Source-bound snapshot: `1846`; copy `4872e9b11aa74bcc80da787a06404441` passed the exact
  post-materialization audit `217/217`.
- The preceding run `a1d800fd78dd4c43a2a8896c0dccad0a` stopped on an Editor fallback-line
  collection inference error before this focused test ran. The collection is now explicitly
  `Vec<RuntimeTextLine>` and passes exact rustfmt/diff checks.
- Successor managed batch request `f71486fef10b4dbf899843f0b885bcac` completed naturally as run
  `5a1ee52cdb27449b86878a76ee712792` with exit 101. The first stage completed a cold build in
  30m45s, then the 10 Hz behavior test exposed that a Continuous pump had not consumed the
  constructor's initial frame request before transition to LowPower. No later product,
  configuration or performance stage started.
- Continuous pumps now consume any already-satisfied frame request. A real focus/occlusion
  transition still requests its own immediate runtime frame through the window lifecycle owner;
  only the stale cross-mode request is removed.
- Current-source validation copy `ceff69e18e804765bc173ac13b1a5312`, input manifest
  `6d54e5a7541aa149cf42a852bcdffa99779b3344cc2c692efeffffb023aa70cd`, completed run
  `f8f8e174ed384165b847933cbe91a548` through the App01 gate. The preceding profile-capture Pester
  contracts passed `63/63`; then
  `unfocused_game_cadence_caps_default_wake_rate_at_ten_hz` passed in the exact current-source
  `zircon_app` binary test stage (`1834.886s`). Later App02 input-closure failure does not invalidate
  these already-terminal stage receipts.
