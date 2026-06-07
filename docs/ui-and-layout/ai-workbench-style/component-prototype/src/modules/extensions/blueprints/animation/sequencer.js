import { blueprint, checkValue, inputValue, selectValue, timelinePrimary, tree } from "../helpers.js";

export const sequencerBlueprint = blueprint({
  status: "Cinematic sequence timeline selected",
  actions: [["play", "Preview Sequence"], ["plus", "Add Track"], ["target", "Key Selection"], ["check", "Validate Sequence"]],
  tools: ["Camera Cut", "Actor Track", "Audio Track", "Event Track", "Curve Editor", "Shot Marker"],
  assets: tree("Cinematics", "history", ["SEQ_Intro", "Camera_A", "Hero_Actor", "Audio_Theme", "Shot_003"]),
  metrics: [["Shots", "12"], ["Tracks", "34"], ["Keys", "428"], ["Gaps", "1", "warning"]],
  detailTabs: ["Tracks", "Curves", "Validation"],
  settings: [["Sequence", selectValue("SEQ_Intro")], ["Frame Rate", selectValue("24 fps")], ["Work Range", inputValue("0100-1460")], ["Snap", checkValue(true)], ["Auto Key", checkValue(false)]],
  primary: timelinePrimary("Sequencer Timeline", ["Track", "Range", "State"], [["Camera Cut", "0000-0180", "Ready"], ["Hero Transform", "0180-0620", "Selected"], ["Audio Theme", "0000-1460", "Ready"], ["Event Cues", "0520-0860", "Warning"]])
});
