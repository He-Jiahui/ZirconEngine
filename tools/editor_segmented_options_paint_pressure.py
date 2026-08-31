import argparse
import json
from pathlib import Path


def run(
    *,
    stable_paint_count: int = 10_000,
    option_count: int = 3,
    average_option_utf8_bytes: int = 8,
) -> dict[str, object]:
    for name, value in (
        ("stable_paint_count", stable_paint_count),
        ("option_count", option_count),
        ("average_option_utf8_bytes", average_option_utf8_bytes),
    ):
        if value < 0:
            raise ValueError(f"{name} must be non-negative")

    option_visits = stable_paint_count * option_count
    option_payload = option_visits * average_option_utf8_bytes

    return {
        "schema_version": 1,
        "inputs": {
            "stable_paint_count": stable_paint_count,
            "option_count": option_count,
            "average_option_utf8_bytes": average_option_utf8_bytes,
        },
        "retired_owned_option_collection": {
            "modeled_option_model_visits": option_visits,
            "modeled_option_vec_allocation_count": stable_paint_count,
            "modeled_option_string_clone_count": option_visits,
            "modeled_option_utf8_payload_bytes": option_payload,
        },
        "borrowed_two_pass_options": {
            "modeled_option_model_visits": option_visits * 2,
            "modeled_option_vec_allocation_count": 0,
            "modeled_option_string_clone_count": 0,
            "modeled_option_utf8_payload_bytes": 0,
        },
        "delta": {
            "avoided_option_vec_allocation_count": stable_paint_count,
            "avoided_option_string_clone_count": option_visits,
            "avoided_option_utf8_payload_bytes": option_payload,
            "additional_borrowed_option_visit_count": option_visits,
        },
        "model_notes": [
            "Two borrowed passes preserve empty-option filtering while separating count from paint traversal.",
            "The model excludes command-owned display labels, iterator overhead, String headers, capacity slack, and allocator metadata.",
            "The model is not measured CPU time, allocator traffic, RSS, input latency, or GPU evidence.",
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stable-paint-count", type=int, default=10_000)
    parser.add_argument("--option-count", type=int, default=3)
    parser.add_argument("--average-option-utf8-bytes", type=int, default=8)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        stable_paint_count=args.stable_paint_count,
        option_count=args.option_count,
        average_option_utf8_bytes=args.average_option_utf8_bytes,
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
