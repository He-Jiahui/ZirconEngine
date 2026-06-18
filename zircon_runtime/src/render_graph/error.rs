use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RenderGraphError {
    #[error("render graph pass `{pass}` is unknown")]
    UnknownPass { pass: usize },
    #[error("render graph resource `{resource}` is unknown")]
    UnknownResource { resource: String },
    #[error("render graph resource name `{resource}` is declared more than once")]
    DuplicateResourceName { resource: String },
    #[error("render graph `{graph_name}` contains a dependency cycle")]
    CycleDetected { graph_name: String },
    #[error(
        "render graph pass `{pass}` reads resource `{resource}` before any producer writes it"
    )]
    ReadBeforeProducer { resource: String, pass: String },
    #[error(
        "render graph pass `{pass}` loads transient attachment `{resource}` before any producer writes it"
    )]
    LoadBeforeProducer { resource: String, pass: String },
    #[error(
        "render graph pass `{pass}` reads transient attachment `{resource}` after producer `{producer}` discarded it"
    )]
    ReadAfterDiscardedStore {
        resource: String,
        pass: String,
        producer: String,
    },
    #[error(
        "render graph resource `{resource}` is written by `{first_writer}` and `{second_writer}` without an ordering dependency"
    )]
    WriteAfterWriteMissingDependency {
        resource: String,
        first_writer: String,
        second_writer: String,
    },
    #[error("render graph `{graph_name}` has no present, readback, persistent, or side-effect cull root")]
    MissingCullRoot { graph_name: String },
}
