use super::super::fetch::fetch_bytes;
use super::*;
use crate::asset::artifact::read_raw_asset;
use crate::core::runtime::{TaskCancellationPolicy, TaskDescriptor, TaskId, TaskPoolKind};

pub(super) async fn fetch_catalog(
    scope: &TaskGraphScope,
    root: Arc<str>,
) -> Result<Arc<[u8]>, CoreError> {
    fetch_metadata(
        scope,
        0,
        format!("{}/catalog.json", root.trim_end_matches('/')),
        None,
        RAW_CATALOG_MAX_BYTES,
    )
    .await
}
pub(super) async fn fetch_metadata(
    scope: &TaskGraphScope,
    id: u64,
    url: String,
    expected: Option<u64>,
    limit: u64,
) -> Result<Arc<[u8]>, CoreError> {
    let value = AsyncValue::new();
    let result = value
        .wait()
        .map_err(|e| asset_error_message(e.to_string()))?;
    let sender = value.sender();
    let task = scope
        .submit_local_future(
            TaskDescriptor::new(TaskId::new(id), TaskPoolKind::Io, "browser-bundle-metadata")
                .with_cancellation_policy(TaskCancellationPolicy::CancelOnDrop),
            move |token| async move {
                sender.send(
                    fetch_bytes(&url, expected, limit, &token)
                        .await
                        .map_err(Arc::<str>::from),
                );
            },
        )
        .map_err(|e| asset_error_message(e.to_string()))?;
    let result = result.await;
    drop(task);
    result.map_err(|e| asset_error_message(e.to_string()))
}
impl BrowserAssetSource {
    pub(super) fn request(&self, id: AssetId) -> Result<Arc<Request>, CoreError> {
        let payload = self
            .payloads
            .get(&id)
            .ok_or_else(|| asset_error_message(format!("missing cooked resource {id}")))?
            .clone();
        let (request, task_id) = {
            let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if !state.accepting {
                return Err(asset_error_message("browser asset source is closed"));
            }
            if let Some(request) = state.requests.get(&id) {
                if request.done.result().is_none()
                    || request.done.result().is_some_and(|r| r.is_err())
                    || self.resources.get_untyped(id).is_some()
                {
                    return Ok(Arc::clone(request));
                }
            }
            let active = state
                .requests
                .values()
                .filter(|r| r.done.result().is_none())
                .count();
            let bytes = self
                .payloads
                .iter()
                .filter(|(key, _)| {
                    self.resources.get_untyped(**key).is_some()
                        || state
                            .requests
                            .get(key)
                            .is_some_and(|r| r.done.result().is_none())
                })
                .fold(0_u64, |n, (_, p)| n.saturating_add(p.bytes));
            if active >= self.limits.max_in_flight
                || bytes.saturating_add(payload.bytes) > self.limits.max_resident_payload_bytes
            {
                return Err(asset_error_message(
                    "browser asset request or resident payload budget reached",
                ));
            }
            let task_id = state.next_task;
            state.next_task = task_id
                .checked_add(1)
                .ok_or_else(|| asset_error_message("browser asset task sequence exhausted"))?;
            let request = Arc::new(Request {
                done: AsyncValue::new(),
                task: Mutex::new(None),
            });
            state.requests.insert(id, Arc::clone(&request));
            (request, task_id)
        };
        let sender = request.done.sender();
        let root = Arc::clone(&self.root);
        let resources = self.resources.clone();
        let limit = self.limits.max_payload_bytes;
        let task = self.scope.submit_local_future(
            TaskDescriptor::new(
                TaskId::new(task_id),
                TaskPoolKind::Io,
                "browser-cooked-resource",
            )
            .with_cancellation_policy(TaskCancellationPolicy::CancelOnDrop),
            move |token| async move {
                let result = async {
                    let url = format!(
                        "{}/assets/{}.zr-raw",
                        root.trim_end_matches('/'),
                        blake3::Hash::from_bytes(payload.hash).to_hex()
                    );
                    let bytes = fetch_bytes(&url, Some(payload.bytes), limit, &token).await?;
                    if blake3::hash(&bytes).as_bytes() != &payload.hash {
                        return Err("cooked resource content hash mismatch".to_string());
                    }
                    let imported = read_raw_asset(&bytes, limit).map_err(|e| e.to_string())?;
                    drop(bytes);
                    if crate::asset::asset_kind_for_imported_asset(&imported) != payload.kind {
                        return Err("cooked resource kind mismatch".to_string());
                    }
                    if token.is_cancellation_requested() {
                        return Err("cooked resource request cancelled".to_string());
                    }
                    resources
                        .commit(ResourceMutationBatch::new().store_payload_erased(
                            id,
                            payload.revision,
                            imported.into_resource_data(),
                        ))
                        .map_err(|e| e.to_string())?;
                    Ok(())
                }
                .await;
                sender.send(result.map_err(Arc::<str>::from));
            },
        );
        match task {
            Ok(task) => {
                *request.task.lock().unwrap_or_else(|e| e.into_inner()) = Some(task);
            }
            Err(error) => {
                request.done.complete(Err(error.to_string().into()));
                return Err(asset_error_message(error.to_string()));
            }
        }
        Ok(request)
    }
}
