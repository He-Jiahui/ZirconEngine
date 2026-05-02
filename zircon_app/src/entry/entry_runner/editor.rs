use std::error::Error;

#[cfg(feature = "target-editor-host")]
use serde_json::Value;
#[cfg(feature = "target-editor-host")]
use zircon_editor::{
    core::editor_event::EditorEventRuntime,
    core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
        EditorOperationSource,
    },
    run_editor,
    ui::host::EditorManager,
    ui::workbench::state::EditorState,
    EDITOR_MANAGER_NAME,
};
#[cfg(feature = "target-editor-host")]
use zircon_runtime::{core::math::UVec2, scene::DefaultLevelManager};

#[cfg(feature = "target-editor-host")]
use crate::entry::{EntryConfig, EntryProfile};

#[cfg(feature = "target-editor-host")]
use super::super::runtime_library::{LoadedRuntime, RuntimeSession};

use super::EntryRunner;

impl EntryRunner {
    pub fn run_editor() -> Result<(), Box<dyn Error>> {
        Self::run_editor_with_args(std::iter::empty::<String>())
    }

    pub fn run_editor_with_args<I, S>(args: I) -> Result<(), Box<dyn Error>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        #[cfg(not(feature = "target-editor-host"))]
        {
            let _ = args;
            return Err("run_editor requires the `target-editor-host` feature".into());
        }
        #[cfg(feature = "target-editor-host")]
        {
            let request = EditorCliOperationRequest::parse(args.into_iter().map(Into::into))?;
            if let Some(request) = request {
                let response = Self::run_editor_operation(request)?;
                println!("{}", serde_json::to_string(&response)?);
                return Ok(());
            }
            let core = Self::bootstrap(EntryConfig::new(EntryProfile::Editor))?;
            let runtime = LoadedRuntime::load_default()?;
            let runtime_client =
                std::sync::Arc::new(RuntimeSession::create_with_profile(runtime, b"editor")?);
            run_editor(core, runtime_client)?;
            Ok(())
        }
    }

    #[cfg(feature = "target-editor-host")]
    fn run_editor_operation(
        request: EditorCliOperationRequest,
    ) -> Result<zircon_editor::core::editor_operation::EditorOperationControlResponse, Box<dyn Error>>
    {
        let core = Self::bootstrap(EntryConfig::new(EntryProfile::Editor))?;
        let state = EditorState::with_default_selection(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
        );
        let manager = core.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)?;
        let runtime = EditorEventRuntime::new(state, manager);
        Ok(runtime.handle_operation_control_request_from_source(
            EditorOperationSource::Cli,
            request.into_control_request()?,
        ))
    }
}

#[cfg(feature = "target-editor-host")]
#[derive(Clone, Debug, PartialEq)]
struct EditorCliOperationRequest {
    operation_id: Option<EditorOperationPath>,
    arguments: Value,
    operation_group: Option<String>,
    headless: bool,
    list_operations: bool,
    query_operation_stack: bool,
}

#[cfg(feature = "target-editor-host")]
impl EditorCliOperationRequest {
    fn into_control_request(self) -> Result<EditorOperationControlRequest, Box<dyn Error>> {
        if self.list_operations {
            return Ok(EditorOperationControlRequest::ListOperations);
        }
        if self.query_operation_stack {
            return Ok(EditorOperationControlRequest::QueryOperationStack);
        }
        let Some(operation_id) = self.operation_id else {
            return Err(
                "--operation is required unless --list-operations or --operation-stack is set"
                    .into(),
            );
        };
        let mut invocation =
            EditorOperationInvocation::new(operation_id).with_arguments(self.arguments);
        if let Some(operation_group) = self.operation_group {
            invocation = invocation.with_operation_group(operation_group);
        }
        Ok(EditorOperationControlRequest::InvokeOperation(invocation))
    }

    fn parse<I>(args: I) -> Result<Option<Self>, Box<dyn Error>>
    where
        I: IntoIterator<Item = String>,
    {
        let mut args = args.into_iter();
        let mut operation_id = None;
        let mut arguments = Value::Null;
        let mut arguments_provided = false;
        let mut operation_group = None;
        let mut headless = false;
        let mut list_operations = false;
        let mut query_operation_stack = false;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--operation" => {
                    if operation_id.is_some() {
                        return Err("--operation was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--operation requires an operation id".into());
                    };
                    operation_id = Some(EditorOperationPath::parse(value)?);
                }
                "--args" => {
                    if arguments_provided {
                        return Err("--args was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--args requires a JSON value".into());
                    };
                    arguments = serde_json::from_str(&value)?;
                    arguments_provided = true;
                }
                "--operation-group" => {
                    if operation_group.is_some() {
                        return Err("--operation-group was provided more than once".into());
                    }
                    let Some(value) = args.next() else {
                        return Err("--operation-group requires a group id".into());
                    };
                    operation_group = Some(value);
                }
                "--list-operations" => {
                    if list_operations {
                        return Err("--list-operations was provided more than once".into());
                    }
                    list_operations = true;
                }
                "--operation-stack" => {
                    if query_operation_stack {
                        return Err("--operation-stack was provided more than once".into());
                    }
                    query_operation_stack = true;
                }
                "--headless" => {
                    if headless {
                        return Err("--headless was provided more than once".into());
                    }
                    headless = true;
                }
                other => return Err(format!("unknown editor argument `{other}`").into()),
            }
        }

        if operation_id.is_none() {
            if arguments_provided {
                return Err("--args requires --operation".into());
            }
            if operation_group.is_some() {
                return Err("--operation-group requires --operation".into());
            }
        }
        let operation_mode_count = usize::from(operation_id.is_some())
            + usize::from(list_operations)
            + usize::from(query_operation_stack);
        if operation_mode_count > 1 {
            return Err(
                "--operation, --list-operations, and --operation-stack are mutually exclusive"
                    .into(),
            );
        }
        if operation_mode_count == 0 {
            if headless {
                return Err(
                    "--headless requires --operation, --list-operations, or --operation-stack"
                        .into(),
                );
            }
            return Ok(None);
        }
        if !headless {
            return Err("editor operation control requests require --headless".into());
        }
        Ok(Some(Self {
            operation_id,
            arguments,
            operation_group,
            headless,
            list_operations,
            query_operation_stack,
        }))
    }
}

#[cfg(all(test, feature = "target-editor-host"))]
mod tests {
    use super::EditorCliOperationRequest;
    use zircon_editor::core::editor_operation::EditorOperationControlRequest;

    #[test]
    fn editor_cli_operation_parser_accepts_operation_args_and_headless() {
        let request = EditorCliOperationRequest::parse([
            "--operation".to_string(),
            "Window.Layout.Reset".to_string(),
            "--args".to_string(),
            r#"{"source":"ci"}"#.to_string(),
            "--headless".to_string(),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            request.operation_id.as_ref().unwrap().as_str(),
            "Window.Layout.Reset"
        );
        assert_eq!(request.arguments["source"], "ci");
        assert!(request.headless);
    }

    #[test]
    fn editor_cli_operation_parser_accepts_operation_group() {
        let request = EditorCliOperationRequest::parse([
            "--operation".to_string(),
            "Viewport.Transform.Apply".to_string(),
            "--operation-group".to_string(),
            "Viewport.TransformDrag.42".to_string(),
            "--headless".to_string(),
        ])
        .unwrap()
        .unwrap();

        assert_eq!(
            request.operation_group.as_deref(),
            Some("Viewport.TransformDrag.42")
        );

        let EditorOperationControlRequest::InvokeOperation(invocation) =
            request.into_control_request().unwrap()
        else {
            panic!("expected InvokeOperation request");
        };
        assert_eq!(
            invocation.operation_group.as_deref(),
            Some("Viewport.TransformDrag.42")
        );
    }

    #[test]
    fn editor_cli_operation_parser_rejects_operation_group_without_operation() {
        let error = EditorCliOperationRequest::parse([
            "--operation-group".to_string(),
            "Viewport.TransformDrag.42".to_string(),
            "--headless".to_string(),
        ])
        .unwrap_err();

        assert_eq!(error.to_string(), "--operation-group requires --operation");
    }

    #[test]
    fn editor_cli_operation_parser_rejects_args_without_operation() {
        let error = EditorCliOperationRequest::parse([
            "--args".to_string(),
            r#"{"source":"ci"}"#.to_string(),
            "--headless".to_string(),
        ])
        .unwrap_err();

        assert_eq!(error.to_string(), "--args requires --operation");
    }

    #[test]
    fn editor_cli_operation_parser_rejects_null_args_without_operation() {
        let error = EditorCliOperationRequest::parse([
            "--args".to_string(),
            "null".to_string(),
            "--headless".to_string(),
        ])
        .unwrap_err();

        assert_eq!(error.to_string(), "--args requires --operation");
    }

    #[test]
    fn editor_cli_operation_parser_rejects_headless_without_control_request() {
        let error = EditorCliOperationRequest::parse(["--headless".to_string()]).unwrap_err();

        assert_eq!(
            error.to_string(),
            "--headless requires --operation, --list-operations, or --operation-stack"
        );
    }

    #[test]
    fn editor_cli_operation_parser_rejects_operation_mixed_with_list_operations() {
        let error = EditorCliOperationRequest::parse([
            "--operation".to_string(),
            "Window.Layout.Reset".to_string(),
            "--list-operations".to_string(),
            "--headless".to_string(),
        ])
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "--operation, --list-operations, and --operation-stack are mutually exclusive"
        );
    }

    #[test]
    fn editor_cli_operation_parser_rejects_list_operations_mixed_with_stack_query() {
        let error = EditorCliOperationRequest::parse([
            "--list-operations".to_string(),
            "--operation-stack".to_string(),
            "--headless".to_string(),
        ])
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "--operation, --list-operations, and --operation-stack are mutually exclusive"
        );
    }

    #[test]
    fn editor_cli_operation_parser_rejects_control_request_without_headless() {
        for args in [
            vec!["--operation".to_string(), "Window.Layout.Reset".to_string()],
            vec!["--list-operations".to_string()],
            vec!["--operation-stack".to_string()],
        ] {
            let error = EditorCliOperationRequest::parse(args).unwrap_err();

            assert_eq!(
                error.to_string(),
                "editor operation control requests require --headless"
            );
        }
    }

    #[test]
    fn editor_cli_operation_parser_rejects_duplicate_control_arguments() {
        for (args, expected) in [
            (
                vec![
                    "--operation".to_string(),
                    "Window.Layout.Reset".to_string(),
                    "--operation".to_string(),
                    "Window.Layout.Reset".to_string(),
                    "--headless".to_string(),
                ],
                "--operation was provided more than once",
            ),
            (
                vec![
                    "--operation".to_string(),
                    "Window.Layout.Reset".to_string(),
                    "--args".to_string(),
                    "{}".to_string(),
                    "--args".to_string(),
                    "{}".to_string(),
                    "--headless".to_string(),
                ],
                "--args was provided more than once",
            ),
            (
                vec![
                    "--operation".to_string(),
                    "Window.Layout.Reset".to_string(),
                    "--operation-group".to_string(),
                    "Group.1".to_string(),
                    "--operation-group".to_string(),
                    "Group.2".to_string(),
                    "--headless".to_string(),
                ],
                "--operation-group was provided more than once",
            ),
            (
                vec![
                    "--list-operations".to_string(),
                    "--list-operations".to_string(),
                    "--headless".to_string(),
                ],
                "--list-operations was provided more than once",
            ),
            (
                vec![
                    "--operation-stack".to_string(),
                    "--operation-stack".to_string(),
                    "--headless".to_string(),
                ],
                "--operation-stack was provided more than once",
            ),
            (
                vec![
                    "--list-operations".to_string(),
                    "--headless".to_string(),
                    "--headless".to_string(),
                ],
                "--headless was provided more than once",
            ),
        ] {
            let error = EditorCliOperationRequest::parse(args).unwrap_err();

            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn editor_cli_operation_parser_leaves_empty_args_for_gui_startup() {
        assert!(EditorCliOperationRequest::parse(Vec::<String>::new())
            .unwrap()
            .is_none());
    }

    #[test]
    fn editor_cli_operation_parser_accepts_list_operations() {
        let request = EditorCliOperationRequest::parse([
            "--list-operations".to_string(),
            "--headless".to_string(),
        ])
        .unwrap()
        .unwrap();

        assert!(request.list_operations);
        assert!(request.headless);
    }

    #[test]
    fn editor_cli_operation_parser_accepts_operation_stack_query() {
        let request = EditorCliOperationRequest::parse([
            "--operation-stack".to_string(),
            "--headless".to_string(),
        ])
        .unwrap()
        .unwrap();

        assert!(request.query_operation_stack);
        assert!(request.headless);
    }

    #[test]
    fn editor_cli_operation_stack_query_maps_to_control_request() {
        let request = EditorCliOperationRequest::parse([
            "--operation-stack".to_string(),
            "--headless".to_string(),
        ])
        .unwrap()
        .unwrap();

        assert!(matches!(
            request.into_control_request().unwrap(),
            EditorOperationControlRequest::QueryOperationStack
        ));
    }
}
