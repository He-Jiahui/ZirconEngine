use super::*;

#[test]
fn windows_recycle_command_uses_shell_recycle_bin_api() {
    let command = RecycleDeleteCommand::windows_delete_directory(Path::new("E:/Projects/My Game"));

    assert_eq!(command.program, "powershell");
    assert_eq!(command.args[0], "-NoProfile");
    assert!(command.args[3].contains("Microsoft.VisualBasic.FileIO.FileSystem"));
    assert!(command.args[3].contains("SendToRecycleBin"));
    assert!(command.args[3].contains("E:/Projects/My Game"));
}

#[test]
fn recycle_command_rejects_empty_path() {
    assert!(RecycleDeleteCommand::for_project("").is_err());
}

#[test]
fn windows_recycle_script_escapes_quotes_spaces_unicode_and_newlines() {
    for (raw, expected_fragment) in [
        ("E:/Projects/Designer's Game", "Designer''s Game"),
        ("E:/Projects/My Game", "My Game"),
        ("E:/项目/我的 游戏", "我的 游戏"),
        ("E:/Projects/Line1\nLine2", "Line1\nLine2"),
        (
            "E:/Projects/It's '; Remove-Item x",
            "It''s ''; Remove-Item x",
        ),
    ] {
        let command = RecycleDeleteCommand::windows_delete_directory(Path::new(raw));
        let script = &command.args[3];

        assert!(script.contains(expected_fragment), "raw={raw}");
        assert_eq!(script.matches('\'').count() % 2, 0, "raw={raw}");
        assert!(script.contains("SendToRecycleBin"));
    }
}
