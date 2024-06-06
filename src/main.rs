mod structures;

use std::{fs::File, io::BufReader, time::Duration};

use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer};
use askama::Template;
use clap::{command, Parser};
use futures::{stream, StreamExt};
use log::{info, warn};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::structures::errors::UptimersError;

#[derive(Debug, Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    status: Vec<(&'a str, StatusCode, bool)>,
}

#[get("/")]
pub async fn index_handler(
    sites: web::Data<Vec<String>>,
    client: web::Data<Client>,
) -> Result<HttpResponse, UptimersError> {
    let status = connect_sites(&client, &sites).await;
    info!("{:?}", status);
    let index = IndexTemplate { status };
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(index.render()?))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    sites: Vec<String>,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to config file
    #[arg(long, env, default_value = "./config.yaml")]
    config_path: String,
}

async fn connect_sites<'a>(
    client: &'a Client,
    sites: &'a Vec<String>,
) -> Vec<(&'a str, StatusCode, bool)> {
    stream::iter(sites)
        // Attempt to connect to all URLs
        .map(|url| async move {
            match client
                .get(url)
                .timeout(Duration::from_secs(10))
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    info!("Connected to {}: {}", url, status);
                    (url.as_str(), status, status.is_success())
                }
                Err(e) => {
                    warn!("Failed to connect to {}: {}", url, e);
                    (url.as_str(), StatusCode::INTERNAL_SERVER_ERROR, false)
                }
            }
        })
        .buffered(10)
        .collect::<Vec<(&str, StatusCode, bool)>>()
        .await
}

#[actix_web::main]
async fn main() -> Result<(), UptimersError> {
    let args = Args::parse();
    info!("Started with args: {:?}", args);

    info!("Reading config from {}", args.config_path);
    let config: Config = serde_yaml::from_reader(BufReader::new(File::open(&args.config_path)?))?;

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let sites = config.sites;
    let client = Client::new();

    // Soon...
    // actix_web::rt::spawn(async move {
    //     loop {
    //         let l = connect_sites(&client, &sites).await;
    //         println!("{:?}", l);

    //         sleep(Duration::from_secs(60)).await;
    //     }
    // });

    Ok(HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(sites.clone()))
            .app_data(web::Data::new(client.clone()))
            .service(index_handler)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?)
}
