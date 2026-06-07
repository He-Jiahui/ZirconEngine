import { checkbox, input, select } from "../../../../components/inputs/atoms.js";

export const onlineRecipe = {
  detailTabs: ["Sessions", "Rules", "Telemetry"],
  actions: (subject, shortLabel) => [["play", `Simulate ${shortLabel}`], ["target", `Match ${shortLabel}`], ["check", `Validate ${shortLabel}`], ["save", `Publish ${shortLabel}`]],
  tools: (subject) => ["Queue Rules", "Party State", "Region Map", "Latency Buckets", `${subject} Preview`, "Failure Report"],
  metrics: () => [["Players", "128"], ["Queues", "6"], ["Latency", "42 ms"], ["Failures", "2", "warning"]],
  settings: (subject) => [["Region", select("Auto")], ["Rule Set", select(subject)], ["Max Wait", input("", { value: "90" })], ["Crossplay", checkbox("", true)], ["Backfill", checkbox("", true)]],
  table: () => [["NA-East", "Open", "42 ms"], ["EU-West", "Open", "58 ms"], ["Asia", "Limited", "84 ms"], ["Backfill", "Queued", "12 jobs"]]
};
