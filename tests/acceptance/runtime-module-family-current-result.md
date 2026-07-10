# Runtime 14 module-family current result

Date: 2026-07-10

Status: in progress

| Filter/evidence | Result | Meaning |
|---|---:|---|
| `module_family` old binary | 2 passed / 2 failed | two stale plan guards |
| current module-family suite | 6/6 | root seats, counts, docs, split and JSON guards pass |
| `diagnostic_log` | 15/15 | executable family gate passes in available binary |
| `engine_module` old binary | 6 passed / 1 failed | real missing `ServiceFactory` surface detected |
| current target-client `engine_module` filter | 7/7 | canonical `ServiceFactory` re-export is compiled and the complete family filter passes |
| `navigation` old binary | 109 passed / 4 failed | one stale guard, three external UI behaviors |
| `animation` old binary | 39 passed / 6 failed | all six stale guards reconciled in current source |
| module-family structure audit | risks = [] | counts 28/9/7/8 and missing lists empty |

Status anchors are recorded in the Runtime14 numbered result and Runtime15 reconciliation record. Broader acceptance remains pending until fresh binary reruns and the external UI navigation behaviors close.
