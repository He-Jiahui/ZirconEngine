# Runtime Text trusted secure-session architecture review

Date: 2026-08-28

Status: `architecture_review_complete / authored_text_route_identity_implemented_unvalidated /
bounded_content_free_action_delivery_implemented_unvalidated /
dynamic_dispatch_result_action_drop_closed / remaining_dispatch_receipts_open /
trusted_session_contract_open /
woc_consumer_integration_open / public_plaintext_resolver_closed /
managed_validation_pending`

## Current-source finding

The secure value reference is content-free and surface-local, but it is not yet a trusted delivery
session. The latest-reference resolver is now crate-local after the audit found no production
consumer outside `zircon_runtime`; no public compatibility alias remains. The public dynamic Runtime
does not expose its Surface. Before the 2026-08-28 action-delivery slice,
`RuntimeUiSurfaceSet::dispatch_input` and pointer/accessibility dispatch reduced
`UiInputDispatchResult` to a handled boolean, so all component reports and template actions were
discarded at the product boundary. Dynamic dispatch now extracts authored template actions through
one shared result collector and leaves them in the existing transactional Host Request stream;
binding reports and the remaining generic UI host requests are still open.

WOC has the matching authored routes and a real authentication owner:

- `auth_form.zui` declares `woc.shell.auth.set_password` and `woc.shell.auth.submit`;
- `password_recovery.zui` declares the reset-password routes;
- `WocShellController::dispatch_shell_route` accepts `Option<&str>` and transfers password values
  into `AuthFlow`;
- the WOC native client does not consume Runtime UI dispatch results today.

Before this review, editable-text Change/Submit reports also set `template_action = None`, so even
an in-process consumer could not prove which authored route produced a secure reference. The
current non-validation slice now projects one report per matching binding, preserves route/action
identity for keyboard, text, IME, clipboard, accessibility and focus-loss Submit, and keeps secure
payloads empty. Compiled surfaces use the `(node, event kind)` binding index, so lookup is `O(k)` for
the matching bindings rather than `O(all node bindings)`; the source-only fallback remains a
construction/test compatibility path. No secure lease is issued when no matching binding exists.

## Reference-engine conclusion

Unreal's editable text owner keeps the actual value and passes it to the bound commit delegate;
password mode changes presentation through the password run/marshaller instead of replacing the
credential with mask text. Zircon must preserve the same separation: display and generic event
projection remain redacted, while a specifically authorized action consumer receives the real
value. A mask, JSON operation argument, plugin-event payload or generic host-request text field is
not an acceptable substitute.

## Rejected directions

- Do not serialize plaintext into `UiTemplateActionInvocation.payload`, Runtime operation JSON,
  plugin-event mirrors, diagnostics or the generic host-request batch.
- Do not treat possession of a node id, route string or `UiSecureTextValueRef` alone as sufficient
  authorization. The reference currently proves freshness, not principal/window/seat ownership.
- Do not make `zircon_runtime` depend on `woc_client`, and do not add a WOC-only callback to the UI
  Surface. Product ownership must enter through a versioned Runtime extension boundary.
- Do not export a reusable `String`. Resolution must be one-shot or transactionally consumed and
  its result must not be Clone/Serialize/Debug plaintext.

## Required contract before implementation

1. Add a versioned trusted text-session handle owned by the Runtime dynamic session and qualified
   by viewport, surface, live window instance, input seat and an exact allowlist of authored route
   ids. Generic Zircon App sessions register no secure routes by default.
2. Retain content-free action deliveries in a bounded, transactional Runtime UI action queue. Each
   delivery binds session, viewport, surface, tree, node, property, exact route and the newest
   secure reference. Queue overflow, supersession, focus loss, surface rebuild and session/window
   teardown revoke the lease.
3. Expose a separate bounded consume operation requiring the Runtime session handle, trusted
   text-session handle and the complete delivery identity. Validate all fields before resolving the
   Surface lease; register output before committing one-shot consumption so allocation failure is
   retry-safe. The generic action/event DTO remains content-free.
4. Implement the product adapter outside the engine core. WOC registers only its static password
   and reset-password routes, consumes during route dispatch, and immediately passes a borrowed
   value to the authentication owner. A later secure-buffer/zeroization milestone must replace the
   current long-lived password `String` storage; that work is not hidden by this bridge.
5. Publish content-free outcomes for accepted, stale, unauthorized, superseded, overflowed and
   disconnected deliveries. Add fault tests for cross-session, cross-window, cross-seat,
   cross-surface, wrong-route, replay and output-registration rollback before managed product
   validation.

## Evidence and remaining gates

Focused source regressions now require secure Change/Submit and focus-loss Commit to retain their
authored route while serialized results contain no credential substring. Rustfmt and scoped
whitespace validation pass. A repository source scan finds no plaintext resolver use outside
Runtime-owned tests/implementation, and the method is crate-local. Managed Cargo and the dynamic WOC
route are still pending, and this non-rendering change intentionally produces no screenshot.

The follow-up product bridge now has a typed `UiAction` Host Request containing viewport, surface,
tree, node, input sequence, action ordinal, authored invocation and optional opaque secure
reference. The Runtime queue is bounded by 256 rows, 240 KiB of admitted encoded requests, 64 KiB
per row and a nesting reserve below the 128-level JSON ceiling. Secure Change supersedes only the
same pending field/route; Submit remains FIFO. Any secure action rejected for overflow, byte/depth
budget, identity mismatch, undelivered status or non-redacted payload returns its reference to the
owning Surface for immediate lease revocation. The existing Host Request page prepare/commit/
rollback transaction owns delivery retry. Generic App sessions install no product adapter: they
validate viewport identity and issue logarithmically bounded, payload-free diagnostics instead of
pretending to execute WOC routes. Static source and queue regressions cover FIFO identity, rollback
stability, row/byte/depth bounds, secure supersession, rejection and payload-free diagnostics.

This review closes the missing route-identity prerequisite and the dynamic template-action drop.
It does not close full `UiInputDispatchResult` publication. The trusted session, one-shot consume,
product adapter, secure-buffer ownership, zeroization, platform secure IME and real WOC
authentication validation remain P0-open.
