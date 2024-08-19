use serde::Serialize;

pub trait Table<const N: usize>: Serialize {
    type Key;
    type Value;
    fn get_keys() -> [Self::Key; N];
    fn get_values(&self) -> [Self::Value; N];
    fn get_entries(&self) -> [(Self::Key, Self::Value); N];
    fn get_value(&self, key: &Self::Key) -> Option<Self::Value>;
}
