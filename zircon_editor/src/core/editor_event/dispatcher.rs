use crate::core::editor_event::{
    EditorEvent, EditorEventEnvelope, EditorEventRecord, EditorEventSource,
};
use zircon_runtime_interface::ui::binding::UiEventBinding;

/// 供宿主、无界面入口和回放共用的语义事件执行边界；宿主实现负责执行与记录，执行失败日志由宿主写入。
/// 调用者提交编辑意图，不能把返回记录当作独立于宿主状态的事务或撤销凭据。
pub trait EditorEventDispatcher {
    type Error: std::error::Error + 'static;

    /// 执行已带来源的语义事件；适用于绑定已经归一化的跨入口传递。
    fn dispatch_envelope(
        &self,
        envelope: EditorEventEnvelope,
    ) -> Result<EditorEventRecord, Self::Error>;

    /// 将界面绑定解析为语义事件后执行；绑定解析失败与实际执行失败均通过实现方错误类型返回。
    fn dispatch_binding(
        &self,
        binding: UiEventBinding,
        source: EditorEventSource,
    ) -> Result<EditorEventRecord, Self::Error>;

    /// 直接执行语义事件并保留调用来源；回放使用此入口，重新分配本次执行的序号和记录。
    fn dispatch_event(
        &self,
        source: EditorEventSource,
        event: EditorEvent,
    ) -> Result<EditorEventRecord, Self::Error>;
}
