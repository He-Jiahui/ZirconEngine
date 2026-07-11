import { Chip } from "@mui/material";

const tones: Record<string, "default" | "success" | "warning" | "error" | "info"> = {
  active: "success", running: "info", succeeded: "success", completed: "success", fixed: "success",
  degraded: "warning", stale: "warning", blocked: "warning", failed: "error", invalid: "error", open: "error",
};

export function StatusText({ value }: { value: string }) {
  return <Chip size="small" label={value || "unknown"} color={tones[value.toLowerCase()] ?? "default"} variant="outlined" />;
}
