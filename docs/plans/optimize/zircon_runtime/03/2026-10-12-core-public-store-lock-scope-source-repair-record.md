# Runtime 03 Core public-store lock-scope source repair

**Status:** Source adoption recorded. Native/type-check, performance, product, and Jenkins acceptance remain pending.

## Repair scope

`ConfigStore::store_value` now converts the caller-provided key and creates the new `Arc<Value>` before acquiring the values-map mutex. It performs the map insertion inside an inner guard scope, then retires the replaced `Arc` after that guard has dropped. This prevents caller-controlled key conversion and destruction of the replaced value from running under this store's mutex. The public API and owner type remain unchanged; the existing `if_changed` decision and publication barrier are preserved.

One ordinary regression name is now present in the source:

- `core::runtime::config_store::tests::config_store_public_store_converts_reentrant_keys_before_locking`

It exercises a real nonblocking `try_lock` in the reentrant key-conversion path. Its presence is a Source fact only; it was not executed. Weak references alone do not demonstrate mutex acquisition or guard-drop ordering, and this record does not treat them as such. The previous test-file prefix remains byte-exact.

## Applied source evidence

The actual canonical apply receipt is [`actual-canonical-apply-v1.json`](../../../../../.codex/outbox/runtime03-core-store-retirement-and-key-conversion-repair-v1/actual-canonical-apply-v1.json), SHA-256 `8d799aa23ed19cda65cd209306a7975c7ea4fadae09474e7990b62c594e58f15` (12,878 bytes). The joint strict check and scoped apply both recorded exit code 0; the receipt records four canonical paths across the two adopted packets, an unchanged index, exact physical postimages, and zero source copies.

For this packet, `config_store.rs` is 3,772 bytes (raw and LF SHA-256 `88b32d70613f3815d68a72b01fd1a76027f815b4d87145068213ee781e842d80`). `tests/config_store.rs` is 7,017 bytes (raw and LF SHA-256 `6b56908c4501cf86bb3868af03a5b570d4aa42c2e983706072accdeba56f5327`). The actual receipt records each postimage's exact mtime and no-reparse check.

The author's [binding](../../../../../.codex/outbox/runtime03-core-store-retirement-and-key-conversion-repair-v1/binding-v1.json), [functional proof](../../../../../.codex/outbox/runtime03-core-store-retirement-and-key-conversion-repair-v1/proof-v1.json), and [entry contract](../../../../../.codex/outbox/runtime03-core-store-retirement-and-key-conversion-repair-v1/entry-contract-v1.json), plus the [independent Source review](../../../../../.codex/outbox/runtime03-core-store-retirement-and-key-conversion-repair-v1/independent-review-v1.json), document this limited change. Source checks do not establish type checking or test execution.

## Validation and limits

No Rust type-check, Native test, or performance sample ran as part of adoption. One ordinary case is present in Source and zero were executed. The batch adds no ignored producer.

The prior 15-command contract remains intact: 14 original producers plus two Runtime 03 matrix producers, 16 total. Conditional future floors are 64 unique ordinary cases for command #2, 193 for command #15, and 12 ignored-library matches for command #9. These are selection requirements, not run counts. The earlier Native07 recipe remains frozen and refused; a future validation run needs a new coordinator and Source identity. The user-authorized Jenkins handoff is being prepared; it has not been submitted to a queue and is not Jenkins acceptance.

The matching [Runtime 03 completion list](../../../astra/features/runtime/2026-10-12-runtime03-core-public-store-lock-scope-source-completion-list.md) keeps the Source change separate from pending compiler, Native, performance, product, Jenkins, and milestone gates.
