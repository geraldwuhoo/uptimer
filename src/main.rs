mod structures;

use std::{fs::File, io::BufReader, time::Duration};

use actix_web::{get, middleware::Logger, rt::time::sleep, web, App, HttpResponse, HttpServer};
use askama::Template;
use clap::{command, Parser};
use futures::{stream, StreamExt, TryStreamExt};
use log::{debug, error, info, warn};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::structures::{
    errors::UptimersError,
    model::{SiteFactModel, SiteStatModel},
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    sites: Vec<String>,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Postgres username
    #[arg(long, env, default_value = "uptimers")]
    postgres_username: String,

    /// Postgres password
    #[arg(long, env, default_value = "password")]
    postgres_password: String,

    /// Postgres host
    #[arg(long, env, default_value = "localhost")]
    postgres_host: String,

    /// Postgres DB name
    #[arg(long, env, default_value = "uptimers")]
    postgres_db: String,

    /// path to config file
    #[arg(long, env, default_value = "./config.yaml")]
    config_path: String,
}

#[derive(Debug, Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    sites: Vec<SiteStatModel>,
}

#[get("/")]
pub async fn index_handler(
    sites: web::Data<Vec<String>>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, UptimersError> {
    // Select the most recent timestamps and 24 hours average uptime from each site
    let status = sqlx::query_as!(
        SiteStatModel,
        r#"SELECT
            t1.site,
            t1.tstamp,
            t1.success,
            t1.status_code,
            t2.avg
        FROM (
            SELECT
                site,
                tstamp,
                success,
                status_code
            FROM site_fact s1
            WHERE
                tstamp = (SELECT MAX(tstamp) FROM site_fact s2 WHERE s1.site = s2.site)
            AND
                site = ANY($1)
            ORDER BY site, tstamp
        ) t1
        INNER JOIN (
            SELECT
                site,
                AVG(success::int::float)
            FROM
                site_fact WHERE tstamp >= (NOW() - INTERVAL '1 day')
            GROUP BY
                site
        ) t2
        ON
            t1.site = t2.site;"#,
        sites.as_ref(),
    )
    .fetch_all(pool.as_ref())
    .await?;

    debug!("{:?}", status);
    let index = IndexTemplate { sites: status };
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(index.render()?))
}

async fn connect_sites(
    sites: &Vec<String>,
    client: &Client,
    pool: &PgPool,
) -> Result<(), UptimersError> {
    // Truncate the current timestamp to minute accuracy
    let now = time::OffsetDateTime::now_utc()
        .replace_nanosecond(0)
        .unwrap()
        .replace_second(0)
        .unwrap();

    // Connect to all user-supplied URLs and write results into the database
    stream::iter(sites)
        // Attempt to connect to all URLs
        .map(|url| async move {
            let status_code = match client
                .get(url)
                .timeout(Duration::from_secs(10))
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    info!("Connected to {}: {}", url, status);
                    status
                }
                Err(e) => {
                    warn!("Failed to connect to {}: {}", url, e);
                    StatusCode::BAD_GATEWAY
                }
            };
            Ok::<SiteFactModel, UptimersError>(SiteFactModel {
                site: url.to_string(),
                tstamp: now,
                success: status_code.is_success(),
                status_code: status_code.as_u16() as i16,
            })
        })
        // Run 10 parallel at a time
        .buffered(10)
        // Try to update the DB with the connection data
        .try_for_each_concurrent(5, |site| async move {
            sqlx::query_as!(
                SiteFactModel,
                r#"INSERT INTO site_fact(
                        site,
                        tstamp,
                        success,
                        status_code
                    )
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT (site, tstamp)
                    DO NOTHING"#,
                site.site,
                site.tstamp,
                site.success,
                site.status_code,
            )
            .execute(pool)
            .await?;
            Ok(())
        })
        .await
}

#[actix_web::main]
async fn main() -> Result<(), UptimersError> {
    let args = Args::parse();
    info!("Started with args: {:?}", args);

    info!("Reading config from {}", args.config_path);
    let config: Config = serde_yaml::from_reader(BufReader::new(File::open(&args.config_path)?))?;
    debug!("Config: {:?}", config);

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Both these use Arc internally, so clones are cheap
    let sites = web::Data::new(config.sites);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&format!(
            "postgres://{}:{}@{}/{}",
            args.postgres_username, args.postgres_password, args.postgres_host, args.postgres_db,
        ))
        .await?;
    sqlx::migrate!().run(&pool).await?;

    // Spawn background loop to check on all user-supplied URLs every minute
    actix_web::rt::spawn({
        let sites = sites.clone();
        let client = Client::new();
        let pool = pool.clone();
        async move {
            loop {
                if let Err(e) = connect_sites(&sites, &client, &pool).await {
                    error!("Failed to get site status: {}", e);
                };

                sleep(Duration::from_secs(60)).await;
            }
        }
    });

    Ok(HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(sites.clone())
            .app_data(web::Data::new(pool.clone()))
            .service(index_handler)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?)
}
