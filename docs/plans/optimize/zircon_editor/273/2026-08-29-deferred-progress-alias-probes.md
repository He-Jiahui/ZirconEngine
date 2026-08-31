# Editor273 Deferred Progress Alias Probes

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime328-editor273-performance-batch-20260829ba-v1`

## Scope

Button-style alias projection previously resolved progress track and fill sources even for ordinary
buttons, sliders, and other non-progress roles. Progress source resolution now runs only inside the
progress-role branch. Alias ownership, progress state overrides, and borrowed common-path results
remain unchanged.

## Static Evidence

- BTreeMap probes on an alias-free non-progress style: `8 -> 3`.
- Progress-role source and state selection remain unchanged.
- Alias-free non-progress maps remain borrowed through `Cow`.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR273_DEFERRED_PROGRESS_ALIAS_PROBES_BENCH_V1`.
It compares eager and role-gated source probing over 8,192 checks against a 1,024-attribute map
across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
