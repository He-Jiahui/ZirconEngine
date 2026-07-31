pub trait PreferenceStorage {
    type Error;

    fn read(&self, key: &str) -> Result<Option<String>, Self::Error>;
    fn write(&self, key: &str, value: &str) -> Result<(), Self::Error>;
}
