---
title: WOS168 Market listing transport
status: implemented_static_dynamic_validation_deferred
source_commit: 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a
owner: woc
---

# WOS168 Market Listing Transport

## Source contract

Source command id 102 is the fire-and-forget client send
`market_list { item, count, price }`. `OnlineGame.marketList` forwards its
string item id and both numeric arguments unchanged. The server only requires a
string item and finite numeric count and price before delegating to
`sim.marketList(item, count, price, pid)`. The market reducer owns every
business rule: player state and merchant distance, known-item eligibility,
quest/soulbound/no-market restrictions, count flooring and availability, price
flooring and bounds, listing capacity, escrow/removal and emitted events.

## Typed ingress scope

Schema 57 will represent `market_list` as `utf8_id_f64_pair`:
`u32_le_utf8_item + f64_le_count + f64_le_price`, with the established 256-byte
typed identity bound and 20..276 encoded bytes. The native
`MarketListCommandPayload` and `ClientGameplayIntent::ListMarketItem` preserve
item text and finite numeric values (including fractional, negative and signed
zero values after canonical finite encoding). They perform no item lookup,
merchant check, rounding, price policy, inventory mutation, escrow, listing
limit or reducer event work.

The source has no item-id field bound; its 16 KiB limit applies to the outer JSON
WebSocket frame. The existing Plugins08 WOS166 raw-text/frame-ingress handoff
remains the dependency for exact JavaScript string and serialized-frame parity.
This slice uses the established bounded typed-transport convention and does not
claim that broader domain.

## Deferred authority and projection

WOC has source-derived market state rules but has no accepted fixed-tick
authoritative command reducer, server ingress adapter, inventory escrow bridge or
listing/event projection for this command. WOS168 advances only typed client
ingress. It must not add a native market fallback or claim a complete marketplace
loop.

## Verification

- Static source anchors cover the client send, server type/finite guards and
  `marketList` reducer entry point.
- Native codec tests round-trip raw ids and finite values, reject non-finite or
  malformed input, and keep the generic descriptor bounds.
- Client intent tests map exactly command id 102 without market policy.
- Regenerate and check command-payload schema/coverage. Cargo and ZrVM dynamic
  execution remain deferred under the open Plugins08 runtime handoff.
