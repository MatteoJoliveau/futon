use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct DatabaseCreationParams {
    pub q: Option<usize>,
    pub n: Option<usize>,
    pub partitioned: bool,
}

impl DatabaseCreationParams {
    pub fn partitioned() -> Self {
        Self {
            partitioned: true,
            ..Default::default()
        }
    }
}
