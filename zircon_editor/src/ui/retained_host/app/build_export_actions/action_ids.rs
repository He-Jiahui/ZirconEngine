pub(in crate::ui::retained_host::app) enum BuildExportAction<'a> {
    GeneratePlan {
        profile_name: &'a str,
    },
    Execute {
        profile_name: &'a str,
    },
    Cancel {
        profile_name: &'a str,
    },
    SetOutput {
        profile_name: &'a str,
        output_root: &'a str,
    },
    ChooseOutput {
        profile_name: &'a str,
    },
    ClearOutput {
        profile_name: &'a str,
    },
    RevealOutput {
        profile_name: &'a str,
    },
}

pub(in crate::ui::retained_host::app) fn parse_build_export_action(
    action_id: &str,
) -> Option<BuildExportAction<'_>> {
    let action = action_id.strip_prefix("workbench.build_export.")?;
    if let Some(profile_name) = action
        .strip_prefix("plan.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::GeneratePlan { profile_name });
    }
    if let Some(profile_name) = action
        .strip_prefix("execute.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::Execute { profile_name });
    }
    if let Some(profile_name) = action
        .strip_prefix("cancel.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::Cancel { profile_name });
    }
    if let Some(profile_name) = action
        .strip_prefix("output.clear.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::ClearOutput { profile_name });
    }
    if let Some(profile_name) = action
        .strip_prefix("output.reveal.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::RevealOutput { profile_name });
    }
    if let Some(profile_name) = action
        .strip_prefix("output.choose.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::ChooseOutput { profile_name });
    }
    action
        .strip_prefix("output.set.")
        .and_then(|rest| rest.split_once('|'))
        .and_then(|(profile_name, output_root)| {
            if profile_name.trim().is_empty() || output_root.trim().is_empty() {
                None
            } else {
                Some(BuildExportAction::SetOutput {
                    profile_name,
                    output_root,
                })
            }
        })
}

#[cfg(test)]
#[path = "action_ids/common_prefix_tests.rs"]
mod common_prefix_tests;
