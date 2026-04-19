use slint::ModelRc;

use crate::ui::layouts::common::model_rc;
use crate::ui::slint_host::{NewProjectFormData, RecentProjectData, WelcomePaneData};
use crate::ui::workbench::startup::RecentProjectValidation;
use crate::WelcomePaneSnapshot;

pub(crate) struct WelcomePresentation {
    pub pane: WelcomePaneData,
    pub recent_projects: ModelRc<RecentProjectData>,
}

pub(crate) fn welcome_presentation(snapshot: &WelcomePaneSnapshot) -> WelcomePresentation {
    WelcomePresentation {
        pane: WelcomePaneData {
            title: snapshot.title.clone().into(),
            subtitle: snapshot.subtitle.clone().into(),
            status_message: snapshot.status_message.clone().into(),
            form: NewProjectFormData {
                project_name: snapshot.form.project_name.clone().into(),
                location: snapshot.form.location.clone().into(),
                project_path_preview: snapshot.form.project_path_preview.clone().into(),
                template_label: snapshot.form.template_label.clone().into(),
                validation_message: snapshot.form.validation_message.clone().into(),
                can_create: snapshot.form.can_create,
                can_open_existing: snapshot.form.can_open_existing,
                browse_supported: snapshot.browse_supported,
            },
        },
        recent_projects: model_rc(
            snapshot
                .recent_projects
                .iter()
                .map(|recent| RecentProjectData {
                    display_name: recent.display_name.clone().into(),
                    path: recent.path.clone().into(),
                    last_opened_label: recent.last_opened_label.clone().into(),
                    status_label: recent_validation_label(recent.validation).into(),
                    invalid: recent.validation != RecentProjectValidation::Valid,
                })
                .collect(),
        ),
    }
}

fn recent_validation_label(validation: RecentProjectValidation) -> &'static str {
    match validation {
        RecentProjectValidation::Valid => "",
        RecentProjectValidation::Missing => "Missing",
        RecentProjectValidation::InvalidManifest => "Manifest Error",
        RecentProjectValidation::InvalidProject => "Invalid Project",
    }
}
