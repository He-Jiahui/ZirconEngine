use super::EditorSceneMode;

pub trait SceneModeFactory: Send + Sync {
    fn create(&self) -> Box<dyn EditorSceneMode>;
}

impl<F> SceneModeFactory for F
where
    F: Fn() -> Box<dyn EditorSceneMode> + Send + Sync,
{
    fn create(&self) -> Box<dyn EditorSceneMode> {
        self()
    }
}
