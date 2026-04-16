#[derive(Debug, Clone, PartialEq, Default)]
pub enum DataFormat {
    #[default]
    Ascii,
    Binary16,
    Binary32,
    Float32,
}
