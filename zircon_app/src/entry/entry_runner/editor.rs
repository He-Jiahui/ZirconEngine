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
            run_editor(core)?;
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
        Ok(EditorOperationControlRequest::InvokeOperation(
            EditorOperationInvocation::new(operation_id).with_arguments(self.arguments),
        ))
    }

    fn parse<I>(args: I) -> Result<Option<Self>, Box<dyn Error>>
    where
        I: IntoIterator<Item = String>,
    {
        let mut args = args.into_iter();
        let mut operation_id = None;
        let mut arguments = Value::Null;
        let mut headless = false;
        let mut list_operations = false;
        let mut query_operation_stack = false;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--operation" => {
                    let Some(value) = args.next() else {
                        return Err("--operation requires an operation id".into());
                    };
                    operation_id = Some(EditorOperationPath::parse(value)?);
                }
                "--args" => {
                    let Some(value) = args.next() else {
                        return Err("--args requires a JSON value".into());
                    };
                    arguments = serde_json::from_str(&value)?;
                }
                "--list-operations" => list_operations = true,
                "--operation-stack" => query_operation_stack = true,
                "--headless" => headless = true,
                other => return Err(format!("unknown editor argument `{other}`").into()),
            }
        }

        if operation_id.is_none() && !list_operations && !query_operation_stack {
            return Ok(None);
        }
        Ok(Some(Self {
            operation_id,
            arguments,
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
