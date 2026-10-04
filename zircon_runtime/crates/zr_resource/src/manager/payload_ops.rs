use std::sync::Arc;

use crate::{
    ResourceData, ResourceHandle, ResourceId, ResourceMarker, ResourceMutationBatch,
    ResourceRecord, ResourceResult, ResourceSnapshot, UntypedResourceHandle,
};

use super::resource_manager::ResourceManager;

// 元数据可以先于载荷存在；按需加载只填充已声明的版本，缓存消费则按具体类型取回共享载荷。
impl ResourceManager {
    pub fn register_ready<TData>(
        &self,
        record: ResourceRecord,
        payload: TData,
    ) -> ResourceResult<UntypedResourceHandle>
    where
        TData: ResourceData,
    {
        let id = record.id;
        let receipt = self.commit(ResourceMutationBatch::new().upsert_ready(record, payload))?;
        Ok(receipt
            .handle(id)
            .expect("a committed ready upsert produces a handle"))
    }

    /// 读取当前存储的类型擦除载荷；不触发加载，也不要求目录处于 Ready，重载期间可能得到最后有效载荷。
    pub fn get_untyped(&self, id: ResourceId) -> Option<Arc<dyn ResourceData>> {
        self.lock_authority_read().payloads.get(&id).cloned()
    }

    /// 仅为仍处于 Ready 的指定版本填充载荷；异步任务必须传入开始加载时记录的版本，避免晚到结果覆盖新发布。
    pub fn store_payload<TData>(
        &self,
        id: ResourceId,
        expected_revision: u64,
        payload: TData,
    ) -> ResourceResult<()>
    where
        TData: ResourceData,
    {
        self.commit(ResourceMutationBatch::new().store_payload(id, expected_revision, payload))?;
        Ok(())
    }

    /// 同时检查句柄种类与载荷实际类型，缺失或不匹配返回 `None`；只需要载荷、不依赖版本时使用。
    pub fn get<TMarker, TData>(&self, handle: ResourceHandle<TMarker>) -> Option<Arc<TData>>
    where
        TMarker: ResourceMarker,
        TData: ResourceData,
    {
        let authority = self.lock_authority_read();
        if authority.registry.get(handle.id())?.kind != TMarker::KIND {
            return None;
        }
        let payload = authority.payloads.get(&handle.id())?.clone();
        Arc::downcast::<TData>(payload.into_any_arc()).ok()
    }

    /// 在同一权威读锁内取得记录与载荷，供动画和渲染缓存绑定确切版本；不会主动加载或建立驻留租约。
    pub fn snapshot<TMarker, TData>(
        &self,
        handle: ResourceHandle<TMarker>,
    ) -> Option<ResourceSnapshot<TData>>
    where
        TMarker: ResourceMarker,
        TData: ResourceData,
    {
        let authority = self.lock_authority_read();
        let record = authority.registry.get(handle.id())?.clone();
        if record.kind != TMarker::KIND {
            return None;
        }
        let payload = authority.payloads.get(&handle.id())?.clone();
        let resource = Arc::downcast::<TData>(payload.into_any_arc()).ok()?;
        Some(ResourceSnapshot::new(record, resource))
    }
}
