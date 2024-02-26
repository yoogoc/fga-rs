#[derive(Debug, Clone, PartialEq, Copy)]
pub struct ResolutionMetadata {
    pub depth: u32,
    pub datastore_query_count: u32,
}

impl Default for ResolutionMetadata {
    fn default() -> Self {
        Self {
            depth: 25,
            datastore_query_count: 0,
        }
    }
}
