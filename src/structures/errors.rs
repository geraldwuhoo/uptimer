use thiserror::Error;

#[derive(Error, Debug)]
pub enum UptimersError {
    #[error("IO error\n{0}")]
    Read(#[from] std::io::Error),

    #[error("askama templating error\n{0}")]
    Askama(#[from] askama::Error),

    #[error("reqwest error\n{0}")]
    Parse(#[from] reqwest::Error),

    #[error("sqlx error\n{0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("sqlx migreate error\n{0}")]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),

    #[error("serde_yaml error\n{0}")]
    SerdeYaml(#[from] serde_yaml::Error),

    #[error("utf8 error \n{0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("nul error \n{0}")]
    Nul(#[from] std::ffi::NulError),

    #[error("other error \n{0}")]
    Other(String),
}

impl actix_web::error::ResponseError for UptimersError {}
