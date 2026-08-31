# Runtime Text secure policy classifier

Date: 2026-08-27

Status: `woc_password_schema_declared / shared_secure_classifier_implemented /
render_a11y_clipboard_policy_reached / malformed_policy_fail_closed /
secure_event_projection_implemented_unvalidated / secure_ime_session_open /
static_checks_complete / managed_validation_pending`

## Finding

Current source had masking, accessibility redaction and clipboard-write denial for metadata with a
boolean `secure*` attribute. The actual WOC authentication and recovery TextFields declare
`input_kind = "password"`, which neither the input classifier nor the separate render classifier
recognized. Those fields could therefore bypass all existing secure projections.

The two classifiers also duplicated TOML interpretation. Adding `input_kind` independently to each
consumer would preserve the architectural defect and make later password/PIN/platform policy drift
more likely.

## Implementation

- `UiSecureTextPolicy::{PlainText, Password}` and `secure_text_policy(...)` are the only Runtime UI
  metadata classifier. Input, accessibility, clipboard and IME consume it through the editable-text
  policy query; render consumes it directly. The render-local classifier was deleted.
- `TextField.input_kind` is now a catalog enum with `text/password/email/search/number/tel/url` and a
  typed `text` default. This covers the existing WOC assets without changing their source format.
- `input_kind=password`, `type=password` and any true secure alias resolve to Password. A password
  signal wins over an explicit false alias. Malformed secure aliases, non-string input kinds and
  unknown input-kind tokens fail closed to Password; known ordinary input kinds remain PlainText.

## Evidence and open work

- Policy tests cover password-vs-false conflict, ordinary email and malformed/unknown fail-closed
  behavior. The catalog test locks enum options/default.
- Existing product-route regressions now use WOC-shaped `input_kind=password` for masked render/glyph
  artifact, accessibility redaction, secure focus/IME denial and clipboard copy/cut denial. Boolean
  secure aliases retain coverage in other existing tests.
- Rust 2024 formatting and focused diff checks pass. The policy owner is 104 lines, input state 197,
  render text field 660, and catalog text-input owner 147. Tests are authored but managed Cargo and
  real WOC rendering have not run.

The follow-up `2026-08-27-secure-text-event-projection.md` replaces secure Change/Commit values with
surface-owned opaque references and redacts input dispatch events, effects, host/component reports,
binding updates and action payloads. This still does not close secure-text P0 or Runtime82 M0. WOC
has no trusted Runtime-result consumer, secure focus still disables IME, and no public/versioned
secure policy or host session exists. Retained state/history, diagnostics/plugin/export review,
zeroization, reveal-last-character, password-manager/autofill, managed Cargo, real IME, capture and
GPU evidence remain open.
