# Runtime Text typed input-constraint receipt

Date: 2026-08-27

Status: `typed_constraint_receipt_implemented /
canonical_single_line_separators_implemented /
keyboard_text_ime_accessibility_routes_converged /
zero_max_length_unbounded_contract_restored /
constrained_preedit_edit_mapping_implemented /
single_line_enter_submit_implemented /
incremental_validation_open / platform_clause_producer_open /
static_checks_complete / managed_validation_pending`

## Finding

The product text-input gateway silently removed filtered characters and hard-line separators and
silently truncated replacements at the configured grapheme limit. Keyboard, text, IME and
accessibility routes consumed the sanitized `String` without a shared typed result. The single-line
path also recognized only CR/LF while Runtime Text already owned a canonical separator model for CR,
LF, CRLF, VT, FF, NEL, LS and PS.

The review found a separate contract mismatch in the same gateway. The component catalog and state
reducer define `max_length = 0` as unbounded, but the input sanitizer interpreted zero as no remaining
capacity and could discard all ordinary input for a default TextInput.

## Implementation

- `UiTextInputConstraintReceipt` publishes only low-cardinality counts and a truncation flag. It does
  not retain input text, document identity or dynamic labels. `UiInputDispatchDiagnostics` carries the
  optional receipt with a serde default, and the DTO owner tests both a non-empty round trip and a
  legacy payload with the field missing.
- The sanitizer performs filter and single-line admission in one replacement pass. It uses Runtime
  Text's canonical hard-line predicate and counts CRLF as one removed separator. Grapheme capacity
  truncates the accepted buffer in place rather than allocating a second replacement string.
- Keyboard text, text events, IME preedit/commit, accessibility SetValue and accessibility
  ReplaceSelectedText use the same sanitizer result. Accessibility retains the receipt even if a
  later property mutation rejects the prepared value, because the constraint operation already
  occurred before mutation.
- Constrained preedit maps platform cursor/clause UTF-8 byte boundaries while the sanitizer scans the
  replacement. The map stores only requested cursor/clause endpoints, so auxiliary memory is
  `O(clause count)`, not `O(preedit bytes)`. Cursor ranges then clamp to grapheme boundaries;
  non-empty clauses that collapse completely are dropped with a typed count, while valid empty
  clauses retain their existing contract.
- Constraint limits now accept only positive values. This preserves explicit positive
  `max_graphemes`/`max_chars`/`max_length` limits while restoring the catalog's zero-means-unbounded
  contract.
- Unmodified Enter follows the editable-text command policy before text-payload fallback. Multiline
  input inserts a newline; writable single-line input is handled as Submit without mutating text or
  inventing a constraint receipt. Key repeat remains handled but does not emit repeated commits,
  matching Unreal Slate's `HandleCarriageReturn` behavior.

## Evidence and open work

- Regressions cover filter counts, capacity truncation, canonical single-line separators with CRLF
  counted once, constrained IME cursor/clause mapping, valid empty clauses, accessibility
  SetValue/ReplaceSelectedText, the zero maximum contract, single-line submit/repeat behavior and DTO
  serde compatibility. These tests are authored but have not run through managed Cargo in this slice.
- Rust 2024 formatting passes. Focused `git diff --check` passes with line-ending notices only. The
  sanitizer, mapping, state transition, dispatch and mutation owners are 256, 118, 298, 276 and 408
  lines; accessibility result owners are 142 lines each; the DTO owner is 243 lines. The oversized
  aggregate interface contract test receives only the required default field and does not own the new
  protocol test.
- Source scans find six sanitizer call sites and no remaining string-comparison inference for whether
  a replacement was constrained. The canonical separator predicate has one Runtime Text owner.

Retained grapheme counts still scan the current prefix and suffix for each replacement. Production
winit/dynamic paths still supply no rich clauses, and platform-specific UTF-16/ACP conversion remains
a host-layer responsibility. Incremental document validation, locale-aware input grammar,
byte/work/deadline budgets, managed Cargo, profile/RSS/power, WGPU and PNG remain pending. No
Runtime82 document-authority, platform clause-producer or IME-session gate is closed by this slice.
