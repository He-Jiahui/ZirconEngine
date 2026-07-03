pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    match slice {
        "Runtime 15 M3 core-framework naming camera-controller guard child-owner split"
        | "Runtime 15 M3 core-framework naming render-fixture guard child-owner split"
        | "Runtime 15 M3 core-framework render-layer schema-v1 guard child-owner split"
        | "Runtime 15 M3 graphics naming render-fixture guard child-owner split"
        | "Runtime 15 M3 core-scene naming render-contract guard child-owner split"
        | "Runtime 15 M3 core-scene naming ECS owner guard child-owner split"
        | "Runtime 15 M3 asset-dynamic naming texture-container guard child-owner split"
        | "Runtime 15 M3 asset-dynamic asset-watch guard child-owner split"
        | "Runtime 15 M3 asset-dynamic scene-ECS query guard child-owner split"
        | "Runtime 15 M3 asset-dynamic dynamic-API vampire guard child-owner split"
        | "Runtime 15 M3 graphics render-framework receiver guard child-owner split"
        | "Runtime 15 M3 graphics resource-streamer guard child-owner split"
        | "Runtime 15 M3 graphics offscreen-target guard child-owner split"
        | "Runtime 15 M3 graphics GPU-model guard child-owner split"
        | "Runtime 15 M3 asset-schema material guard child-owner split"
        | "Runtime 15 M3 core-scene render-layer schema-v1 guard child-owner split"
        | "Runtime 15 M3 core-scene runtime-state guard child-owner split"
        | "Runtime 15 M3 scene-tests ECS systems guard child-owner split"
        | "Runtime 15 M3 Net HTTP policy guard child-owner split"
        | "Runtime 15 M3 Hub raw-text policy guard child-owner split"
        | "Runtime 15 M3 input mouse-wheel line-delta guard child-owner split"
        | "Runtime 15 M3 plugin static manifest naming guard child-owner split"
        | "Runtime 15 M3 banned-name scene-dynamic guard child-owner split"
        | "Runtime 15 M3 banned-name graphics construction guard child-owner split"
        | "Runtime 15 M3 banned-name global module guard child-owner split" => Some("2026-06-30"),
        "Runtime 15 M3 UI platform-input guard child-owner split" => Some("2026-07-01"),
        _ => None,
    }
}
