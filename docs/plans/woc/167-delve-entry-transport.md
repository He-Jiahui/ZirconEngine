---
title: WOS167 Delve entry transport
status: implemented_static_dynamic_validation_deferred
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS167 Delve Entry Transport

## Source contract

Source command id 115 is the fire-and-forget client send
`enter_delve { delveId, tierId }`. `OnlineGame.enterDelve` forwards both caller
strings without local admission checks. The server only requires both values to
be strings before it checks that the player and delve exist, rejects a dead player
or one farther than 12 units from that delve's board door, invokes
`sim.enterDelve(delveId, tierId, pid)`, and resyncs delve state. The reducer owns
all further admission: conflicting dungeon/arena/delve/trade/duel state, tier,
level, party size, run claim or reuse, teleport, companion/pet handling and
events.

## Typed ingress scope

Schema 56 adds `enter_delve` as `utf8_id_pair`:
`u32_le_utf8_delve_id + u32_le_utf8_tier_id`, with the established 256-byte
per-identity WOC transport bound and 8..520 encoded bytes. The native
`EnterDelveCommandPayload` and `ClientGameplayIntent::EnterDelve` preserve field
order, whitespace and unknown valid UTF-8 ids. They do not resolve a delve/tier,
check distance, player state, level, party or instance capacity, emit errors, or
resync a view.

The source has no per-field id limit; its 16 KiB limit applies to the outer JSON
WebSocket frame. The generic Plugins08 WOS166 raw-text and frame-ingress handoff
therefore remains the dependency for exact JavaScript string and serialized-frame
boundary parity. This slice uses the same bounded typed-id convention as existing
`delve_buy`; it does not claim the broader raw-frame domain.

## Deferred authority and projection

WOC already has scalar delve-admission rules, but no accepted authoritative
fixed-tick command reducer, server geospatial ingress adapter or lossless
resync/event projection for this command. WOS167 only advances typed client
ingress. It must not add a native admission or teleport fallback, and it must not
claim a completed delve-entry gameplay loop.

## Verification

- Static source contract anchors the client send, string guard, 12-unit door gate,
  reducer call and resync.
- Native codec tests round-trip known and unknown id pairs, reject malformed or
  trailing input and retain generic payload bounds.
- Client intent tests map exactly command id 115 without local admission policy.
- Regenerate and check command payload schema/coverage. Cargo and ZrVM dynamic
  execution remain deferred under the open Plugins08 runtime handoff.
