---
title: WOS170 Trade offer transport
status: implemented_static_dynamic_validation_deferred
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS170 Trade Offer Transport

## Source contract

Source command id 66 is the fire-and-forget client send `trade_offer { items,
copper }`. The server only requires an array, then delegates its first six slots
and `Number(copper) || 0` to `sim.tradeSetOffer`. The trade reducer ignores
malformed entries, floors each finite count to at least one, merges duplicate
item ids, rejects unknown/quest/soulbound or unavailable stock, bounds copper to
the player's balance, replaces the side's offer and clears both confirmations.
Session ownership, distance, atomic confirmation, capacity and instance transfer
remain in the existing trade system.

## Typed ingress scope

Schema 59 will represent the valid typed client domain as an item count followed
by up to six `u32_le_utf8_item_id + f64_le_count` elements and `f64_le_copper`.
The 256-byte per-id transport bound gives 9..1617 bytes. Native
`TradeOfferCommandPayload` and `ClientGameplayIntent::SetTradeOffer` preserve
duplicate and unknown ids plus finite fractional or negative counts/copper. They
do not merge, clamp, floor, inspect item instance data, resolve an inventory,
mutate an offer or change confirmation state.

The source permits arbitrary array values and JavaScript numeric coercion for
copper. Exact parity for those non-typed raw cases remains under the Plugins08
WOS166 raw-text/frame-ingress handoff; this slice covers valid UTF-8 strings and
finite numeric client inputs only.

## Deferred authority and projection

No accepted fixed-tick authoritative trade reducer, inventory-instance bridge,
multi-player transaction service or event/snapshot projection exists in WOC yet.
This slice must not add a native trade fallback or claim a completed trade loop.

## Verification

- Static anchors cover client send, server array/coercion gate and trade reducer
  normalization ownership.
- Native codecs round-trip six raw elements, reject malformed/oversized/non-finite
  inputs and retain descriptor bounds.
- Client intent tests map id 66 without local trade rules.
- Regenerate/check schema and coverage. Cargo/ZrVM dynamic execution remains
  deferred under Plugins08.
