use std::collections::HashMap;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::core::framework::events::{
    EngineEvent, EngineEventDeliveryPolicy, EngineEventSubscription, EventBusDiagnosticsSnapshot,
};
use crate::core::CoreError;

use super::CoreHandle;

impl CoreHandle {
    pub fn try_publish_event(
        &self,
        topic: impl Into<String>,
        payload: Value,
    ) -> Result<
        crate::core::framework::events::EngineEventPublishReceipt,
        crate::core::framework::events::EngineEventPublishRejected,
    > {
        self.inner.event_bus.try_publish(EngineEvent {
            topic: topic.into(),
            payload,
        })
    }

    /// 按主题和投递策略建立订阅；调用者须保留返回的订阅对象以接收后续事件。
    pub fn subscribe_events(
        &self,
        topic: impl Into<String>,
        policy: EngineEventDeliveryPolicy,
    ) -> Result<
        Box<dyn EngineEventSubscription>,
        crate::core::framework::events::EngineEventSubscribeError,
    > {
        self.inner.event_bus.subscribe(topic, policy)
    }

    pub fn close_event_admission(&self) -> crate::core::framework::events::EventBusCloseReceipt {
        self.inner.event_bus.close_admission()
    }
    pub fn event_bus_retention(&self) -> crate::core::framework::events::EventBusRetentionSnapshot {
        self.inner.event_bus.retention_snapshot()
    }

    pub fn event_bus_diagnostics(&self) -> EventBusDiagnosticsSnapshot {
        self.inner.event_bus.diagnostic_report()
    }

    pub fn store_config_value(&self, key: impl Into<String>, value: Value) {
        self.inner.config_store.store_value(key, value);
    }

    pub fn load_config_value(&self, key: &str) -> Option<Value> {
        self.inner.config_store.load_value(key)
    }

    pub fn snapshot_config_values(&self) -> HashMap<String, Value> {
        self.inner.config_store.snapshot_values()
    }

    // 配置持久化工作线程延迟读取此快照；仅捕获配置存储，避免服务反向持有整个运行时。
    pub(crate) fn config_snapshot_source(
        &self,
    ) -> Arc<dyn Fn() -> HashMap<String, Value> + Send + Sync> {
        let config_store = self.inner.config_store.clone();
        Arc::new(move || config_store.snapshot_values())
    }

    pub fn store_config<T: serde::Serialize>(
        &self,
        key: impl Into<String>,
        value: &T,
    ) -> Result<(), CoreError> {
        self.inner.config_store.store(key, value)
    }

    pub fn load_config<T: DeserializeOwned>(&self, key: &str) -> Result<T, CoreError> {
        self.inner.config_store.load(key)
    }
}
