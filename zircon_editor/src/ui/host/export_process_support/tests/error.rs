use super::*;

#[test]
fn cleanup_retains_typed_termination_error() {
    let error = ExportProcessError::io(
        "read process output",
        "typed cleanup test",
        Some("stdout"),
        None,
        io::Error::new(io::ErrorKind::UnexpectedEof, "primary source"),
    )
    .with_cleanup(
        "termination command could not start".to_string(),
        Some(ExportProcessTerminationError::CommandSpawn {
            program: "taskkill",
            source: io::Error::new(io::ErrorKind::PermissionDenied, "cleanup source"),
        }),
    );

    match error {
        ExportProcessError::Cleanup {
            source,
            cleanup_error: Some(cleanup_error),
            ..
        } => {
            assert!(matches!(*source, ExportProcessError::Io { .. }));
            assert!(matches!(
                *cleanup_error,
                ExportProcessTerminationError::CommandSpawn {
                    program: "taskkill",
                    source,
                } if source.kind() == io::ErrorKind::PermissionDenied
            ));
        }
        other => panic!("expected typed cleanup error, got {other:?}"),
    }
}
