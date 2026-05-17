use super::SourceEngineInstall;

pub fn active_source_engine<'a>(
    engines: &'a [SourceEngineInstall],
    active_engine_id: Option<&str>,
) -> Option<&'a SourceEngineInstall> {
    active_engine_id
        .and_then(|id| engines.iter().find(|engine| engine.id == id))
        .or_else(|| engines.first())
}

pub fn active_source_engine_mut<'a>(
    engines: &'a mut [SourceEngineInstall],
    active_engine_id: Option<&str>,
) -> Option<&'a mut SourceEngineInstall> {
    if let Some(id) = active_engine_id {
        if let Some(index) = engines.iter().position(|engine| engine.id == id) {
            return engines.get_mut(index);
        }
    }
    engines.first_mut()
}

pub fn ensure_active_source_engine(
    engines: &[SourceEngineInstall],
    active_engine_id: &mut Option<String>,
) {
    let active_exists = active_engine_id
        .as_deref()
        .is_some_and(|id| engines.iter().any(|engine| engine.id == id));
    if active_exists {
        return;
    }
    *active_engine_id = engines.first().map(|engine| engine.id.clone());
}

pub fn upsert_source_engine(engines: &mut Vec<SourceEngineInstall>, engine: SourceEngineInstall) {
    match engines.iter().position(|existing| existing.id == engine.id) {
        Some(index) => engines[index] = engine,
        None => engines.push(engine),
    }
}

pub fn remove_source_engine(
    engines: &mut Vec<SourceEngineInstall>,
    active_engine_id: &mut Option<String>,
    engine_id: &str,
) -> Option<SourceEngineInstall> {
    let index = engines.iter().position(|engine| engine.id == engine_id)?;
    let removed = engines.remove(index);
    ensure_active_source_engine(engines, active_engine_id);
    Some(removed)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn engine(id: &str) -> SourceEngineInstall {
        SourceEngineInstall {
            id: id.to_string(),
            display_name: format!("{id} Engine"),
            source_dir: PathBuf::from(format!("E:/{id}")),
            output_dir: PathBuf::from(format!("E:/out/{id}")),
            last_build_unix_ms: None,
            build_history: Vec::new(),
        }
    }

    #[test]
    fn active_source_engine_falls_back_to_first_record() {
        let engines = vec![engine("first"), engine("second")];

        assert_eq!(
            active_source_engine(&engines, Some("second")).map(|engine| engine.id.as_str()),
            Some("second")
        );
        assert_eq!(
            active_source_engine(&engines, Some("missing")).map(|engine| engine.id.as_str()),
            Some("first")
        );
    }

    #[test]
    fn upsert_source_engine_replaces_existing_record() {
        let mut engines = vec![engine("local")];
        let mut updated = engine("local");
        updated.display_name = "Renamed".to_string();

        upsert_source_engine(&mut engines, updated);

        assert_eq!(engines.len(), 1);
        assert_eq!(engines[0].display_name, "Renamed");
    }

    #[test]
    fn remove_source_engine_repairs_active_selection() {
        let mut engines = vec![engine("first"), engine("second")];
        let mut active = Some("second".to_string());

        let removed = remove_source_engine(&mut engines, &mut active, "second");

        assert_eq!(removed.map(|engine| engine.id), Some("second".to_string()));
        assert_eq!(active.as_deref(), Some("first"));
        assert_eq!(engines.len(), 1);
    }
}
