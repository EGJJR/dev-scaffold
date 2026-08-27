use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("project name is required")]
    NameRequired,
    #[error("{0}")]
    InvalidName(String),
    #[error("{0}")]
    InvalidOutput(String),
    #[error("unknown template '{got}'; expected one of: {expected}")]
    UnknownType { got: String, expected: String },
    #[error("--type is required when stdin is not a terminal")]
    TypeRequired,
    #[error("directory '{}' already exists", .0.display())]
    AlreadyExists(PathBuf),
    #[error("template '{0}' has no files")]
    EmptyTemplate(String),
    #[error("{0}")]
    Dialog(String),
    #[error(transparent)]
    Render(#[from] minijinja::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Error {
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::NameRequired
            | Error::InvalidName(_)
            | Error::InvalidOutput(_)
            | Error::UnknownType { .. }
            | Error::TypeRequired => 2,
            _ => 1,
        }
    }
}
