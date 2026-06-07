import { blueprint, checkValue, inputValue, selectValue, timelinePrimary, tree } from "../helpers.js";

export const weatherEditorBlueprint = blueprint({
  status: "Weather layers and timeline preview selected",
  actions: [["plus", "Add Weather Layer"], ["play", "Preview Weather"], ["check", "Build Weather"], ["target", "Inspect Region"]],
  tools: ["Weather Preset", "Region Profile", "Cloud Layer", "Wind Curve", "Event Track", "Timeline"],
  assets: tree("Weather", "sun", ["Weather_Storm", "Region_Mountains", "Layer_Clouds", "Layer_Rain", "Curve_Wind"]),
  metrics: [["Layers", "8"], ["Regions", "5"], ["Events", "18"], ["Warnings", "2", "warning"]],
  detailTabs: ["Layers", "Curves", "Timeline"],
  settings: [["Preset", selectValue("Storm")], ["Region", selectValue("Mountains")], ["Blend Time", inputValue("12.0")], ["Loop Preview", checkValue(true)], ["Affect Lighting", checkValue(true)]],
  primary: timelinePrimary("Weather Timeline", ["Layer", "Range", "State"], [["Cloud Build", "00:00-02:00", "Ready"], ["Rain Burst", "02:00-04:00", "Selected"], ["Wind Gust", "03:20-05:00", "Ready"], ["Lightning", "04:00-04:30", "Warning"]])
});
