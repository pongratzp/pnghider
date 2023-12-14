#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),

    #[error("File is not a valid PNG: {0}")]
    NotPNG(String),

    #[error("Chunk not found: {0}")]
    ChunkNotFound(String),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}
