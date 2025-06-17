/// A StableContainer with 3 optional fields
#[derive(Debug, Clone, PartialEq)]
pub struct MyStableContainer {
    pub a: Option<u32>,
    pub b: Option<bool>,
    pub c: Option<u64>,
}
