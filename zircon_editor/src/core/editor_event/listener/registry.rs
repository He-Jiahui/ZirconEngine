use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::super::{
    EditorEventRetentionBudgets, EditorEventRetentionPolicy, EditorEventRetentionStore,
};
use super::{
    EditorEventListenerDescriptor, EditorEventListenerFilter, EditorEventListenerHandle,
    EditorEventListenerRoute,
};

#[derive(Debug)]
struct EditorEventListenerState {
    descriptor: EditorEventListenerDescriptor,
    inbox: Arc<Mutex<EditorEventRetentionStore>>,
}

#[derive(Debug)]
/// 监听器控制面的登记与配置所有者；按登记顺序提供描述符，投递面使用可共享的路由快照。
/// 禁用或注销只影响后来取得的快照，已经在途的路由和查询句柄仍可持有原收件箱。
pub struct EditorEventListenerRegistry {
    listener_order: Vec<String>,
    listeners: HashMap<String, EditorEventListenerState>,
    retention_budgets: EditorEventRetentionBudgets,
    delivery_routes: Arc<[EditorEventListenerRoute]>,
}

impl Default for EditorEventListenerRegistry {
    fn default() -> Self {
        Self::new(EditorEventRetentionPolicy::default().listeners)
    }
}

impl EditorEventListenerRegistry {
    pub(crate) fn new(retention_budgets: EditorEventRetentionBudgets) -> Self {
        Self {
            listener_order: Vec::new(),
            listeners: HashMap::new(),
            retention_budgets,
            delivery_routes: Arc::from([]),
        }
    }

    pub fn register(
        &mut self,
        listener_id: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Result<(), String> {
        let listener_id = listener_id.into();
        if self.listeners.contains_key(&listener_id) {
            return Err(format!(
                "editor event listener {listener_id} already registered"
            ));
        }
        self.listener_order.push(listener_id.clone());
        self.listeners.insert(
            listener_id.clone(),
            EditorEventListenerState {
                descriptor: EditorEventListenerDescriptor {
                    listener_id,
                    display_name: display_name.into(),
                    enabled: true,
                    filter: None,
                },
                inbox: Arc::new(Mutex::new(EditorEventRetentionStore::new(
                    self.retention_budgets.clone(),
                ))),
            },
        );
        self.rebuild_delivery_routes();
        Ok(())
    }

    pub fn unregister(&mut self, listener_id: &str) -> Result<(), String> {
        if self.listeners.remove(listener_id).is_none() {
            return Err(not_registered(listener_id));
        }
        self.listener_order.retain(|id| id != listener_id);
        self.rebuild_delivery_routes();
        Ok(())
    }

    pub fn set_enabled(&mut self, listener_id: &str, enabled: bool) -> Result<(), String> {
        let listener = self.listener_mut(listener_id)?;
        if listener.descriptor.enabled == enabled {
            return Ok(());
        }
        listener.descriptor.enabled = enabled;
        self.rebuild_delivery_routes();
        Ok(())
    }

    /// 安装并归一化筛选条件；用于重建后续投递快照，不重新筛选已经入队的记录。
    pub fn set_filter(
        &mut self,
        listener_id: &str,
        filter: EditorEventListenerFilter,
    ) -> Result<(), String> {
        let listener = self.listener_mut(listener_id)?;
        let filter = filter.normalized();
        if listener.descriptor.filter.as_ref() == Some(&filter) {
            return Ok(());
        }
        listener.descriptor.filter = Some(filter);
        self.rebuild_delivery_routes();
        Ok(())
    }

    pub fn clear_filter(&mut self, listener_id: &str) -> Result<(), String> {
        let listener = self.listener_mut(listener_id)?;
        if listener.descriptor.filter.is_none() {
            return Ok(());
        }
        listener.descriptor.filter = None;
        self.rebuild_delivery_routes();
        Ok(())
    }

    pub fn listeners(&self) -> Vec<EditorEventListenerDescriptor> {
        let mut listeners = Vec::with_capacity(self.listener_order.len());
        listeners.extend(
            self.listener_order
                .iter()
                .filter_map(|listener_id| self.listeners.get(listener_id))
                .map(|listener| listener.descriptor.clone()),
        );
        listeners
    }

    pub(crate) fn listener_handle(
        &self,
        listener_id: &str,
    ) -> Result<EditorEventListenerHandle, String> {
        let listener = self
            .listeners
            .get(listener_id)
            .ok_or_else(|| not_registered(listener_id))?;
        Ok(EditorEventListenerHandle::new(
            listener.descriptor.clone(),
            Arc::clone(&listener.inbox),
        ))
    }

    pub(crate) fn delivery_routes(&self) -> Arc<[EditorEventListenerRoute]> {
        Arc::clone(&self.delivery_routes)
    }

    fn listener_mut(&mut self, listener_id: &str) -> Result<&mut EditorEventListenerState, String> {
        self.listeners
            .get_mut(listener_id)
            .ok_or_else(|| not_registered(listener_id))
    }

    // 配置更新发布新路由数组；旧数组保留其筛选条件和收件箱所有权，使派发可在注册表锁外完成。
    fn rebuild_delivery_routes(&mut self) {
        let mut routes = Vec::with_capacity(self.listener_order.len());
        routes.extend(
            self.listener_order
                .iter()
                .filter_map(|listener_id| self.listeners.get(listener_id))
                .filter(|listener| listener.descriptor.enabled)
                .map(|listener| {
                    EditorEventListenerRoute::new(
                        listener.descriptor.filter.clone(),
                        Arc::clone(&listener.inbox),
                    )
                }),
        );
        self.delivery_routes = routes.into();
    }
}

fn not_registered(listener_id: &str) -> String {
    format!("editor event listener {listener_id} is not registered")
}

#[cfg(test)]
#[path = "registry/tests/optimization_tests.rs"]
mod optimization_tests;

#[cfg(test)]
#[path = "tests/registry.rs"]
mod tests;
