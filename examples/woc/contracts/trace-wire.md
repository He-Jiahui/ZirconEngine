# WOC authoritative trace wire (`WTR1`)

`WTR1` carries complete ZrVM-owned scenario state to the generic Rust parity
adapter. It is not a gameplay protocol: Rust may validate, canonicalize, hash and
format these values, but it may not infer or replace gameplay outcomes.

All integers are little-endian. The fixed prefix is:

1. magic `WTR1` (`4 × u8`);
2. version (`u16`, currently `1`);
3. symbol dictionary fingerprint (`u64`, currently the 60-bit contract value
   generated in `reference/trace_symbols.json`);
4. scenario symbol (`u16`), seed/sample cadence/ticks (`u32` each);
5. coverage count (`u16`) followed by that many symbol ids;
6. total RNG draws/digest (`u32` each);
7. frame count (`u16`) followed by frames.

A frame contains tick (`u64`), a typed time value, next entity id (`u64`), an
optional label symbol (`u16`, zero means absent), full-checkpoint flag (`u8`),
cumulative RNG draws/digest (`u32` each), then typed arrays for players, entities
and the authoritative ordered event-window digest (`u32`). The Rust adapter
computes the state digest from the state arrays and formats the supplied event
digest. Player/entity arrays are always present on the wire; they are included
verbosely in JSON only for full checkpoints. Event bodies are deliberately not
part of the parity JSON contract and can contain arbitrary runtime text, so they
are not constrained by the finite canonical symbol dictionary.

Typed values use one-byte tags:

| Tag | Payload | Meaning |
| --- | --- | --- |
| `0` | none | null |
| `1`, `2` | none | false, true |
| `3` | `u64` | unsigned JavaScript-safe integer |
| `4` | sign `u8` + magnitude `u64` | signed integer |
| `5` | sign `u8` + micro-unit magnitude `u64` | six-decimal number |
| `6`, `7`, `8` | none | `Infinity`, `-Infinity`, `NaN` |
| `9` | symbol `u16` | string |
| `10` | length `u32` + values | array |
| `11` | length `u32` + (`key u16`, value) pairs | object |

Decoder defaults cap the envelope at 16 MiB, nesting at 64, collection entries
at 1,000,000, and frames/coverage rows at 4,096. Unknown symbols/tags, duplicate
object keys, unsafe integer magnitudes, malformed booleans, truncation,
dictionary drift and trailing bytes are hard failures.

The ZrVM writer intentionally finishes a trace inside one high-level call. This
avoids relying on module-global container mutation or same-signature private
helper binding, both of which are known experimental-backend defects owned by
the Plugins 08 handoff.

The dictionary preserves the 965 golden-visible IDs and appends 13 values found
only by full-frame reference recording. Hidden state must remain symbolizable
even when the readable golden stores only its digest. Appending hidden values
must never renumber an existing ID.
