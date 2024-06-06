#[derive(Debug, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct SiteFactModel {
    pub site: String,
    pub tstamp: time::OffsetDateTime,
    pub success: bool,
    pub status_code: i16,
}

#[derive(Debug, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct SiteStatModel {
    pub site: String,
    pub tstamp: time::OffsetDateTime,
    pub success: bool,
    pub status_code: i16,
    pub avg: Option<f64>,
}
