# Runtime Text secure event projection

Date: 2026-08-27

Status: `opaque_secure_event_reference_implemented / dispatch_result_redaction_implemented /
latest_reference_fence_implemented / clone_and_serde_lease_reset_implemented /
authored_text_route_identity_implemented_unvalidated /
public_plaintext_resolver_closed /
trusted_host_session_open / retained_state_history_open / secure_ime_session_open /
static_checks_complete / managed_validation_pending`

## Finding

The shared secure classifier closed the WOC `input_kind=password` front-door bypass, but editable
text Change and Submit still cloned the current password into generic `UiValue::String` component
events. Property mutation binding reports copied both previous and next values. The returned input
event, IME surrounding-text effects, rejected clipboard effects, accessibility request and template
action payload provided additional serialization paths for the same value.

Masking the generic event value was not valid: current WOC authentication routes ultimately accept
`Option<&str>`, so a display mask would silently replace the credential. WOC also does not yet
consume Runtime `UiInputDispatchResult`, which means there was no existing trusted host session to
receive an out-of-band secret.

## Architecture and implementation

- Runtime Interface now defines `UiSecureTextValueRef`, an opaque UUID-bearing tree/node/property
  capability with no text payload. `SecureValueChanged` and `SecureCommit` retain the existing
  authored Change/Submit event kinds while replacing generic values at the secure field boundary.
- `UiSurfaceSecureTextValueStore` is the sole lease owner. It retains only the newest reference per
  node/property and fences resolution with the current non-exhausted text-layout revision. A later
  Change/Submit, property mutation, cross-tree/cross-surface reference, forged token, policy change,
  missing node or revision exhaustion fails closed.
- Lease state is skipped by serialization and deliberately reset by `UiSurface::clone`; it does not
  participate in persisted surface equality. The retained plaintext remains in the existing trusted
  surface state until the document/secret-store migration, but is not copied into the public result.
- `redact_secure_text_dispatch_result` is the single exit projection for normal dispatch and direct
  reply application. It clears raw Text/Keyboard/IME/accessibility request fields, binding previous/
  next values, IME surrounding text, clipboard write text, generic component values and template
  action payload values across reply/applied/rejected effects and host/component reports. The typed
  `secure_text_redacted` receipt makes this transformation observable without text-derived labels.
- Keyboard/Text/IME Change, single-line Submit, focus-loss composition commit, accessibility
  SetValue and ReplaceSelectedText use the same secure event factory. Plain fields preserve their
  existing `ValueChanged`/`Commit` values.
- 2026-08-28 review found that editable-text reports discarded their authored template action.
  The shared text event projection now preserves the exact Change/Submit route or action identity
  while secure redaction keeps its payload content-free. Compiled surfaces use the event index and
  visit only matching bindings; a missing binding produces neither a report nor a secure lease.
  Runtime dynamic still reduces the whole dispatch result to a handled boolean, so this is only the
  trusted-session prerequisite documented in
  `2026-08-28-secure-text-trusted-session-architecture-review.md`.

## Static evidence and open work

- Regressions cover WOC-shaped password Change and Submit, no serialized source substring, newest
  reference resolution, stale reference rejection, cross-surface rejection, clone lease reset,
  accessibility SetValue and rejected clipboard effect redaction. Interface tests cover opaque
  reference round-trip and backward-compatible diagnostics defaults.
- Rust 2024 formatting, focused diff checks and trailing-whitespace checks pass. The interface
  capability owner is 79 lines; the surface lease/redaction owner is 299 lines.
- Managed Cargo, WOC integration, fault injection, real IME, WGPU capture, performance and power
  evidence remain pending. No screenshot is produced by this non-rendering slice.

This is an M0 prerequisite, not M0 acceptance. Runtime retained widget/component state, composition
restore/history, crash dumps, diagnostic/export/plugin paths and memory zeroization still need a
classified secret owner. WOC needs a window/seat/session-qualified trusted consumer that resolves a
reference only for the matching authored route and immediately transfers it to the authentication
owner. Secure input still disables IME instead of negotiating a platform-qualified minimum-
disclosure session. Runtime11B secure-text P0 therefore remains open.
