use super::SourceEngineInstall;

/// 解析当前可用的源码引擎供构建和界面读取；无效选择回退到登记顺序中的首项。
pub fn active_source_engine<'a>(
    engines: &'a [SourceEngineInstall],
    active_engine_id: Option<&str>,
) -> Option<&'a SourceEngineInstall> {
    active_engine_id
        .and_then(|id| engines.iter().find(|engine| engine.id == id))
        .or_else(|| engines.first())
}

/// 与只读解析保持相同回退规则，供更新当前引擎的配置或构建历史。
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

/// 在登记表变化后修复持久化的活动 ID；空表时清除选择，避免界面指向已删除引擎。
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

/// 在引擎删除或迁移后清除项目中的悬空绑定，同时保留钉选等独立元数据。
pub fn prune_project_engine_bindings(
    metadata: &mut crate::projects::ProjectMetadataMap,
    engines: &[SourceEngineInstall],
) -> usize {
    let mut pruned = 0;
    metadata.retain(|_, project| {
        if let Some(engine_id) = project.engine_id.as_deref() {
            if !engines.iter().any(|engine| engine.id == engine_id) {
                project.engine_id = None;
                pruned += 1;
            }
        }
        !project.is_empty()
    });
    pruned
}

/// 删除登记项后同步修复活动选择；调用方仍需持久化配置并刷新项目作用域视图。
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
#[path = "tests/registry.rs"]
mod tests;
