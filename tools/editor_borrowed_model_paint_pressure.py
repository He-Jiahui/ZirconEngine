import argparse
import json
from pathlib import Path


def run(
    *,
    stable_paint_count: int = 10_000,
    visible_row_count: int = 64,
    owned_text_fields_per_row: int = 2,
    average_text_utf8_bytes: int = 24,
) -> dict[str, object]:
    for name, value in (
        ("stable_paint_count", stable_paint_count),
        ("visible_row_count", visible_row_count),
        ("owned_text_fields_per_row", owned_text_fields_per_row),
        ("average_text_utf8_bytes", average_text_utf8_bytes),
    ):
        if value < 0:
            raise ValueError(f"{name} must be non-negative")

    cloned_rows = stable_paint_count * visible_row_count
    cloned_text_fields = cloned_rows * owned_text_fields_per_row
    cloned_text_payload = cloned_text_fields * average_text_utf8_bytes

    return {
        "schema_version": 1,
        "inputs": {
            "stable_paint_count": stable_paint_count,
            "visible_row_count": visible_row_count,
            "owned_text_fields_per_row": owned_text_fields_per_row,
            "average_text_utf8_bytes": average_text_utf8_bytes,
        },
        "retired_owned_row_read": {
            "modeled_row_clone_count": cloned_rows,
            "modeled_text_field_clone_count": cloned_text_fields,
            "modeled_text_utf8_payload_bytes": cloned_text_payload,
        },
        "borrowed_row_read": {
            "modeled_row_clone_count": 0,
            "modeled_text_field_clone_count": 0,
            "modeled_text_utf8_payload_bytes": 0,
        },
        "delta": {
            "avoided_row_clone_count": cloned_rows,
            "avoided_text_field_clone_count": cloned_text_fields,
            "avoided_text_utf8_payload_bytes": cloned_text_payload,
        },
        "model_notes": [
            "The model covers stable model-row reads only; command-owned text copies remain outside this model.",
            "UTF-8 figures exclude String headers, capacity slack, allocator metadata, and non-text row fields.",
            "The model is not measured CPU time, allocator traffic, RSS, input latency, or GPU evidence.",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stable-paint-count", type=int, default=10_000)
    parser.add_argument("--visible-row-count", type=int, default=64)
    parser.add_argument("--owned-text-fields-per-row", type=int, default=2)
    parser.add_argument("--average-text-utf8-bytes", type=int, default=24)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        stable_paint_count=args.stable_paint_count,
        visible_row_count=args.visible_row_count,
        owned_text_fields_per_row=args.owned_text_fields_per_row,
        average_text_utf8_bytes=args.average_text_utf8_bytes,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is None:
        print(encoded, end="")
    else:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
