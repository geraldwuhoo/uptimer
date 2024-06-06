use thiserror::Error;

#[derive(Error, Debug)]
pub enum UptimersError {
    #[error("IO error\n{0}")]
    Read(#[from] std::io::Error),

    #[error("askama templating error\n{0}")]
    Askama(#[from] askama::Error),

    #[error("reqwest error\n{0}")]
    Parse(#[from] reqwest::Error),

    #[error("serde_yaml error\n{0}")]
    SerdeYaml(#[from] serde_yaml::Error),
}

impl actix_web::error::ResponseError for UptimersError {}
