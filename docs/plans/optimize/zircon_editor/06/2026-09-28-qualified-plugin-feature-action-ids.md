---
title: Editor06 Qualified Plugin Feature Action IDs
category: zircon_editor
date: 2026-09-28
implementation_status: candidate_static_complete
validation_status: managed_cargo_pending
performance_status: no_new_performance_claim
---

# Editor06 qualified package feature action routing

## Defect and scope

E-PLUGIN-UX-P1-13 remains a product validation gate. The Plugin Manager row joined
`plugin_id` and `feature_id` with a dot, while its callback parser split at the first dot.
A valid package such as `third_party.weather_sim` was therefore decoded as plugin
`third_party` and feature `weather_sim.weather.lightning`.

The candidate changes only feature enable, disable, and dependency-enable action IDs.
Simple package IDs retain their existing action strings. Qualified package IDs use
`#<plugin-byte-length>:<plugin_id>.<feature_id>` after the action kind. The length counts
UTF-8 bytes; the parser borrows slices from the incoming action and rejects malformed
lengths or separators. Runtime plugin ID admission excludes `#` as an initial character.

## Regression and remaining acceptance

Producer tests cover all three qualified action kinds and unchanged simple IDs. Parser
tests cover the exact owner/feature for all three kinds, malformed lengths, a Unicode
boundary, and unknown action kind. The retained pane passes the generated action ID as
a string to the existing `module_plugin` callback route; that downstream route is unchanged.

Pinned rustfmt and scoped diff checks pass for the four Rust paths. Managed Cargo tests,
a real retained-host click through the callback, product generation/admission review, and
performance evidence remain pending. This candidate does not mark E-PLUGIN-UX-P1-13 closed.
