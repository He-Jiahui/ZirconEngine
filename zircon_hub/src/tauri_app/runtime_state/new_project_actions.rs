//! 保存创建表单的草稿与最近一次提交字段，方便失败后仍保留用户输入。
//! 草稿更新不创建目录；真正创建走后台编辑器准入和项目生命周期校验。

use std::path::PathBuf;

use crate::error::HubError;
use crate::projects::project_template_catalog;
use crate::state::ProjectSubpage;
use crate::tauri_app::action_request::{CreateProjectActionPayload, NewProjectDraftActionPayload};

use super::HubRuntimeSession;

impl HubRuntimeSession {
    /// 动作入口用于保存可不完整的表单输入并切到创建流程；保存失败会向调用方返回错误。
    pub(super) fn update_new_project_draft(
        &mut self,
        payload: NewProjectDraftActionPayload,
    ) -> Result<(), HubError> {
        self.apply_new_project_draft_fields(
            payload.name,
            payload.location,
            payload.template,
            payload.engine_id,
        );
        self.project_subpage = ProjectSubpage::NewProject;
        self.pending_delete_project_path = None;
        self.persist()
    }

    /// 在后台创建校验之前保留此次提交字段；失败记录之后的保存负责持久化。
    pub(super) fn remember_create_project_payload(&mut self, payload: &CreateProjectActionPayload) {
        self.apply_new_project_draft_fields(
            payload.name.clone(),
            payload.location.clone(),
            payload.template.clone(),
            payload.engine_id.clone(),
        );
    }

    /// 只做草稿的有限规范化和注册表成员筛选；模板启用状态、绝对路径和引擎就绪由创建路径校验。
    fn apply_new_project_draft_fields(
        &mut self,
        name: String,
        location: PathBuf,
        template: String,
        engine_id: Option<String>,
    ) {
        self.new_project_name = name.trim().to_string();
        self.new_project_location = location;
        if project_template_catalog()
            .iter()
            .any(|candidate| candidate.id == template)
        {
            self.selected_template_id = template;
        }
        self.new_project_engine_id =
            engine_id.filter(|id| self.config.engines.iter().any(|engine| engine.id == *id));
    }
}
