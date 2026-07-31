# WOC M3 authoritative scenario drivers

The four M3 drivers execute the source-pinned gameplay branches inside ZrVM.
They are not expected-value replayers: each driver constructs mutable state,
runs the same ordered operations as the corresponding upstream parity scenario,
and only then exports scalar observations for the WTR1 writer.

The built-in world construction order is part of the RNG contract. Its 306
non-dummy camp mobs each consume five Mulberry32 draws (scatter angle, scatter
radius, level, facing and wander timer), so the parity observer begins after
1,530 unobserved draws. `kernel/rng.prepareSharedAfterConstruction` preserves
that advanced state while resetting the observed count to zero and the rolling
FNV-1a digest to `811c9dc5`.

## Pinned scenario envelopes

| Scenario | Seed | Cadence | Ticks | Frames | Next id | Observed draws | Final digest |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `entity_roster` | 1012 | 2 | 7 | 8 | 411 | 4 | `7ba9533d` |
| `mob_targeting` | 1014 | 2 | 0 | 11 | 412 | 0 | `811c9dc5` |
| `mob_locomotion` | 7777 | 1 | 0 | 9 | 415 | 16 | `4b7ddb75` |
| `mob_lifecycle` | 1015 | 1 | 0 | 7 | 417 | 20 | `e612091c` |

`mob_locomotion` pins cumulative digests after draws 4, 8, 13, 15 and
16: `65f0173a`, `284b8111`, `06ceb9a6`, `5adac73c`, `4b7ddb75`.
The common combat profile consumes 3, 3 and 4 draws before Ground Pound,
Shuddering Stomp and Keening Wail; the mechanic draw therefore remains last in
each stage. Idle wander consumes draws 14-15 and evade reset consumes draw 16.

`mob_lifecycle` pins cumulative digests after draws 5, 8, 9, 15 and
20: `ca7bbe9c`, `cac7cfb2`, `88aac2f9`, `ca70052b`, `e612091c`.
The five death/loot/lifecycle stages consume 5, 3, 1, 6 and 5 draws. The direct
death-throes burst is draw 9; the wild-respawn wander timer is draw 15.

## Stable scalar boundary

The scenario implementations keep custom classes private. Their public
`scenarioContractTest` entry executes once and checks a built-in integer vector.
`scenarioMetric` is available where a scalar-only WTR1 adapter needs one value.
This boundary is temporary only in shape, not authority: ZrVM still owns every
gameplay result.

Returning or passing `container.Array<T>` across modules is not considered
reliable yet. A fresh wire-only build rejected an exported array return with
different signature hashes, then rejected an array parameter while printing
identical expected and actual hashes. The raw reproduction is recorded in the
Plugins 08 failure handoff. Until that shared defect is fixed, each high-level
trace writer must keep its byte array and writer implementation in one module.

These drivers prove the ordered gameplay decisions and RNG stream, but do not
by themselves satisfy the M3 exit gate. M3 remains open until each driver emits
the complete WTR1 player/entity state and event digest, executes twice through the real M2
backend and matches its full pinned JSON golden with no exclusions.

## Current complete trace evidence

`mob_targeting` now emits all 11 full WTR1 frames from ZrVM: three player
metadata rows, three player entities, the forest-wolf entity, ordered threat
tables, forced-target fields and empty event-window digests. The payload is 24,900
bytes with byte FNV `3ad2d815`. A structured in-memory decoder using the pinned
978-symbol dictionary reproduced the entire upstream `mob_targeting.json` and
passed deep equality, including every state/event digest; the final state is
`0e924b3b`. Interpreter and newly compiled binary modes produce the same pinned
payload metrics.

`entity_roster` now emits its eight WTR1 frames, including the three cadence
frames hidden in the readable golden, the four-event tick-2 window digest and
the aura/respawn healer-window digest. Its 6,680-byte payload has byte FNV
`ec6b93dc` and final state `ee7066d9`. Both interpreter and newly compiled
binary output deep-equal the complete pinned `entity_roster.json`.

`mob_locomotion` emits all nine full frames in 19,162 bytes with byte FNV
`9eee50ad` and final state `c3c2c925`. `mob_lifecycle` emits all seven full
frames in 19,803 bytes with byte FNV `39eec1e3` and final state `69d03988`.
Both traces deep-equal their complete pinned goldens in interpreter and newly
compiled binary modes.

All four M3 WTR1 goldens are now exact with no excluded fields or frames. M3
remains open only because the real M2 `zr_vm:project` transaction bridge has not
returned from the Plugins 08 dependency; all four scenarios still require two
identical executions through that production backend before acceptance.
