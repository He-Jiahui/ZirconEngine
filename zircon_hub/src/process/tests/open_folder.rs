use super::*;

#[test]
fn open_folder_command_line_preserves_program_and_path() {
    let command = OpenFolderCommand {
        program: "xdg-open".to_string(),
        args: vec!["/tmp/Zircon Output".to_string()],
    };

    assert_eq!(
        command.command_line(),
        vec!["xdg-open".to_string(), "/tmp/Zircon Output".to_string()]
    );
}
