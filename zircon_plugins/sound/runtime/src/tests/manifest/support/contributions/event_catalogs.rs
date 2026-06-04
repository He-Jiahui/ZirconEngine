use super::StaticEventCatalog;

pub(super) fn event_catalogs_from_plugin_toml(manifest: &str) -> Vec<StaticEventCatalog> {
    let mut catalogs = Vec::new();
    let mut current_namespace = None;
    let mut current_version = None;
    let mut inside_catalog = false;

    for line in manifest.lines().map(str::trim) {
        if line == "[[event_catalogs]]" {
            push_event_catalog(&mut catalogs, &mut current_namespace, &mut current_version);
            inside_catalog = true;
            continue;
        }
        if line.starts_with("[[") {
            push_event_catalog(&mut catalogs, &mut current_namespace, &mut current_version);
            inside_catalog = false;
        }
        if !inside_catalog {
            continue;
        }
        if let Some(value) = line
            .strip_prefix("namespace = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            current_namespace = Some(value.to_string());
            continue;
        }
        if let Some(value) = line.strip_prefix("version = ") {
            current_version = Some(
                value
                    .parse::<u32>()
                    .expect("sound event catalog version should be an integer"),
            );
        }
    }
    push_event_catalog(&mut catalogs, &mut current_namespace, &mut current_version);
    catalogs
}

fn push_event_catalog(
    catalogs: &mut Vec<StaticEventCatalog>,
    namespace: &mut Option<String>,
    version: &mut Option<u32>,
) {
    let Some(namespace) = namespace.take() else {
        return;
    };
    catalogs.push((
        namespace,
        version
            .take()
            .expect("sound event catalog should declare version"),
    ));
}
