import { blueprint, checkValue, inputValue, selectValue, timelinePrimary, tree } from "../helpers.js";

export const montageEditorBlueprint = blueprint({
  status: "Montage sections and notify tracks visible",
  actions: [["play", "Preview Montage"], ["plus", "Add Section"], ["target", "Add Notify"], ["check", "Validate Montage"]],
  tools: ["Section", "Notify", "Slot Track", "Branch Point", "Root Motion", "Sync Marker"],
  assets: tree("Animation", "play", ["AM_DashAttack", "Dash_Start", "Dash_Loop", "Dash_End", "Notify_HitWindow"]),
  metrics: [["Sections", "4"], ["Notifies", "18"], ["Frames", "240"], ["Root", "OK"]],
  detailTabs: ["Sections", "Notifies", "Blend"],
  settings: [["Montage", selectValue("AM_DashAttack")], ["Slot", selectValue("UpperBody")], ["Blend In", inputValue("0.12")], ["Root Motion", checkValue(true)], ["Loop Preview", checkValue(false)]],
  primary: timelinePrimary("Montage Timeline", ["Section", "Range", "State"], [["Start", "0000-0032", "Ready"], ["Loop", "0032-0140", "Selected"], ["Attack", "0140-0190", "Ready"], ["Recover", "0190-0240", "Ready"]])
});
