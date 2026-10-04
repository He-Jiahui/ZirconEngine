import argparse
import json
from pathlib import Path


def run(
    idle_pointer_move_count: int,
    idle_event_batch_count: int,
    capture_pointer_move_count: int,
) -> dict:
    if idle_pointer_move_count < 0:
        raise ValueError("idle_pointer_move_count must be non-negative")
    if idle_event_batch_count < 0:
        raise ValueError("idle_event_batch_count must be non-negative")
    if capture_pointer_move_count < 0:
        raise ValueError("capture_pointer_move_count must be non-negative")

    idle_dispatched = min(idle_pointer_move_count, idle_event_batch_count)
    idle_coalesced = idle_pointer_move_count - idle_dispatched
    total_received = idle_pointer_move_count + capture_pointer_move_count
    single_hit_queries = total_received
    mailbox_queries = idle_dispatched + capture_pointer_move_count
    original_double_hit_queries = total_received * 2
    idle_batches_with_coalescing = min(idle_dispatched, idle_coalesced)
    old_capture_recorder_lock_count = (
        idle_pointer_move_count + (idle_coalesced * 2) + idle_dispatched
    )
    batched_capture_recorder_lock_count = idle_dispatched
    batched_capture_counter_sample_count = (
        idle_dispatched * 3 + idle_batches_with_coalescing * 3
    )
    old_idle_window_id_allocation_count = idle_pointer_move_count
    mailbox_idle_window_id_allocation_count = idle_dispatched
    old_idle_platform_translation_count = idle_pointer_move_count
    mailbox_idle_platform_translation_count = idle_dispatched

    return {
        "schema": "zircon.editor.idle-pointer-move-mailbox-pressure.v3",
        "inputs": {
            "idle_pointer_move_count": idle_pointer_move_count,
            "idle_event_batch_count": idle_event_batch_count,
            "capture_pointer_move_count": capture_pointer_move_count,
        },
        "original_double_hit_path": {
            "total_spatial_queries": original_double_hit_queries,
        },
        "single_hit_path": {
            "idle_dispatched_moves": idle_pointer_move_count,
            "capture_dispatched_moves": capture_pointer_move_count,
            "total_spatial_queries": single_hit_queries,
        },
        "mailbox_path": {
            "idle_dispatched_moves": idle_dispatched,
            "idle_coalesced_moves": idle_coalesced,
            "capture_dispatched_moves": capture_pointer_move_count,
            "total_spatial_queries": mailbox_queries,
        },
        "profiling_capture_path": {
            "old_per_event_recorder_lock_count": old_capture_recorder_lock_count,
            "old_per_event_counter_sample_count": old_capture_recorder_lock_count,
            "batched_recorder_lock_count": batched_capture_recorder_lock_count,
            "batched_counter_sample_count": batched_capture_counter_sample_count,
            "modeled_batches_with_coalescing": idle_batches_with_coalescing,
        },
        "input_metadata_path": {
            "old_idle_window_id_allocation_count": old_idle_window_id_allocation_count,
            "mailbox_idle_window_id_allocation_count": (
                mailbox_idle_window_id_allocation_count
            ),
            "avoided_idle_window_id_allocation_count": (
                old_idle_window_id_allocation_count
                - mailbox_idle_window_id_allocation_count
            ),
            "old_idle_platform_translation_count": old_idle_platform_translation_count,
            "mailbox_idle_platform_translation_count": (
                mailbox_idle_platform_translation_count
            ),
            "avoided_idle_platform_translation_count": (
                old_idle_platform_translation_count
                - mailbox_idle_platform_translation_count
            ),
        },
        "delta": {
            "single_hit_to_mailbox_avoided_spatial_queries": (
                single_hit_queries - mailbox_queries
            ),
            "single_hit_to_mailbox_spatial_query_ratio": (
                single_hit_queries / mailbox_queries if mailbox_queries else 0.0
            ),
            "original_to_mailbox_spatial_query_ratio": (
                original_double_hit_queries / mailbox_queries
                if mailbox_queries
                else 0.0
            ),
            "capture_recorder_lock_ratio": (
                old_capture_recorder_lock_count / batched_capture_recorder_lock_count
                if batched_capture_recorder_lock_count
                else 0.0
            ),
            "capture_counter_sample_ratio": (
                old_capture_recorder_lock_count / batched_capture_counter_sample_count
                if batched_capture_counter_sample_count
                else 0.0
            ),
            "idle_window_id_allocation_ratio": (
                old_idle_window_id_allocation_count
                / mailbox_idle_window_id_allocation_count
                if mailbox_idle_window_id_allocation_count
                else 0.0
            ),
            "idle_platform_translation_ratio": (
                old_idle_platform_translation_count
                / mailbox_idle_platform_translation_count
                if mailbox_idle_platform_translation_count
                else 0.0
            ),
        },
        "scope": {
            "latest_value_idle_mouse_move_only": True,
            "capture_moves_coalesced": False,
            "event_batch_count_is_a_model_input": True,
            "cpu_measured": False,
            "allocator_or_rss_measured": False,
            "input_latency_measured": False,
            "gpu_work_measured": False,
            "recorder_lock_counts_are_source_structure_model": True,
            "counter_sample_counts_are_source_structure_model": True,
            "window_id_allocation_counts_are_source_structure_model": True,
            "platform_translation_counts_are_source_structure_model": True,
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--idle-pointer-moves", type=int, default=65_536)
    parser.add_argument("--idle-event-batches", type=int, default=256)
    parser.add_argument("--capture-pointer-moves", type=int, default=4_096)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        idle_pointer_move_count=args.idle_pointer_moves,
        idle_event_batch_count=args.idle_event_batches,
        capture_pointer_move_count=args.capture_pointer_moves,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is None:
        print(encoded, end="")
        return
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(encoded, encoding="utf-8")


if __name__ == "__main__":
    main()
