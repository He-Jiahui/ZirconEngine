"""Export a compact RenderDoc capture audit through QRenderDoc's embedded Python API."""

import json
import os
import sys
from collections import Counter

import renderdoc as rd


def _succeeded(result):
    code = getattr(result, "code", result)
    return code == rd.ResultCode.Succeeded


def _action_rows(controller):
    structured = controller.GetStructuredFile()
    rows = []

    def resource_name(resource_id):
        if resource_id == rd.ResourceId.Null():
            return None
        name = controller.GetResourceName(resource_id)
        return name if name else str(resource_id)

    def visit(action, depth):
        flags = str(action.flags)
        rows.append(
            {
                "event_id": action.eventId,
                "depth": depth,
                "name": action.GetName(structured),
                "flags": flags,
                "num_indices": action.numIndices,
                "num_instances": action.numInstances,
                "dispatch_dimension": [
                    action.dispatchDimension[0],
                    action.dispatchDimension[1],
                    action.dispatchDimension[2],
                ],
                "copy_source": resource_name(action.copySource)
                if "Copy" in flags
                else None,
                "copy_destination": resource_name(action.copyDestination)
                if "Copy" in flags
                else None,
            }
        )
        for child in action.children:
            visit(child, depth + 1)

    for root in controller.GetRootActions():
        visit(root, 0)
    return rows


def _flag_counts(actions):
    categories = {
        "draw": "Drawcall",
        "dispatch": "Dispatch",
        "copy": "Copy",
        "clear": "Clear",
        "resolve": "Resolve",
        "present": "Present",
        "marker": "PushMarker",
    }
    return {
        category: sum(token in action["flags"] for action in actions)
        for category, token in categories.items()
    }


def _gpu_duration_rows(controller, actions_by_event):
    counters = controller.EnumerateCounters()
    descriptions = [
        {
            "id": int(counter),
            "name": controller.DescribeCounter(counter).name,
        }
        for counter in counters
    ]
    if rd.GPUCounter.EventGPUDuration not in counters:
        return descriptions, []

    rows = []
    for result in controller.FetchCounters([rd.GPUCounter.EventGPUDuration]):
        seconds = getattr(result.value, "d", 0.0)
        action = actions_by_event.get(result.eventId)
        rows.append(
            {
                "event_id": result.eventId,
                "seconds": seconds,
                "milliseconds": seconds * 1_000.0,
                "name": action["name"] if action else "<unmapped>",
                "flags": action["flags"] if action else "",
            }
        )
    rows.sort(key=lambda row: row["seconds"], reverse=True)
    return descriptions, rows


def _ranked_counts(values, limit):
    return [
        {"name": name, "count": count}
        for name, count in Counter(values).most_common(limit)
    ]


def main():
    capture_path = os.environ["ZR_RENDERDOC_CAPTURE"]
    output_path = os.environ["ZR_RENDERDOC_AUDIT_OUTPUT"]

    capture = rd.OpenCaptureFile()
    open_result = capture.OpenFile(capture_path, "", None)
    if not _succeeded(open_result):
        raise RuntimeError("RenderDoc could not open capture: {}".format(open_result))
    if not capture.LocalReplaySupport():
        raise RuntimeError("capture does not support local replay")

    replay_result, controller = capture.OpenCapture(rd.ReplayOptions(), None)
    if not _succeeded(replay_result):
        capture.Shutdown()
        raise RuntimeError("RenderDoc could not initialise replay: {}".format(replay_result))

    try:
        actions = _action_rows(controller)
        actions_by_event = {row["event_id"]: row for row in actions}
        counter_descriptions, gpu_durations = _gpu_duration_rows(
            controller, actions_by_event
        )
        debug_messages = [
            {
                "event_id": message.eventId,
                "severity": str(message.severity),
                "category": str(message.category),
                "source": str(message.source),
                "message": message.description,
            }
            for message in controller.GetDebugMessages()
        ]
        copies = [row for row in actions if "Copy" in row["flags"]]
        copy_event_buckets = Counter(
            (row["event_id"] // 500) * 500 for row in copies
        )
        report = {
            "capture": os.path.abspath(capture_path),
            "api": str(controller.GetAPIProperties().pipelineType),
            "action_count": len(actions),
            "max_event_id": max((row["event_id"] for row in actions), default=0),
            "flag_counts": _flag_counts(actions),
            "resource_counts": {
                "resources": len(controller.GetResources()),
                "textures": len(controller.GetTextures()),
                "buffers": len(controller.GetBuffers()),
            },
            "debug_message_count": len(debug_messages),
            "debug_messages": debug_messages,
            "available_counters": counter_descriptions,
            "gpu_duration_total_ms": sum(
                row["milliseconds"] for row in gpu_durations
            ),
            "gpu_duration_top_25": gpu_durations[:25],
            "root_action_count": sum(row["depth"] == 0 for row in actions),
            "command_list_begin_count": sum(
                "CommandBufferBoundary" in row["flags"]
                and "BeginPass" in row["flags"]
                for row in actions
            ),
            "top_action_names": _ranked_counts(
                (row["name"] for row in actions), 50
            ),
            "copy_source_counts": _ranked_counts(
                (row["copy_source"] or "<unknown>" for row in copies), 50
            ),
            "copy_destination_counts": _ranked_counts(
                (row["copy_destination"] or "<unknown>" for row in copies), 50
            ),
            "copy_event_buckets": [
                {"event_start": event_start, "count": copy_event_buckets[event_start]}
                for event_start in sorted(copy_event_buckets)
            ],
            "marker_actions": [
                row
                for row in actions
                if "Marker" in row["flags"] or row["name"].startswith("zircon::")
            ][:250],
        }
        with open(output_path, "w", encoding="utf-8") as output:
            json.dump(report, output, indent=2, ensure_ascii=False)
            output.write("\n")
    finally:
        controller.Shutdown()
        capture.Shutdown()


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        sys.stderr.write("renderdoc capture audit failed: {}\n".format(error))
        sys.exit(1)
    sys.exit(0)
