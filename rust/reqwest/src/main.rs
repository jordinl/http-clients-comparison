use std::time::{Duration, SystemTime};
use std::fs::File;
use std::env;
use std::io::{self, prelude::*, BufReader};
use reqwest;
use reqwest::header::USER_AGENT;
use tokio;
use futures::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::process;

struct Result {
    code: String,
    time: u128
}


fn get_env(key: &str, default: u32) -> u32 {
    env::var(key)
        .ok()
        .and_then(|limit| limit.parse::<u32>().ok())
        .unwrap_or(default)
}

async fn handle_response(response: reqwest::Response) -> String {
    let code = response.status().as_str().to_string();
    match response.text().await {
        Ok(body) => {
            let _ = body.replace("\0", "");
            code
        },
        Err(err) => handle_error(err).await,
    }
}

async fn handle_error(err: impl Error) -> String {
    let mut last_err: &dyn Error = &err;
    while let Some(source) = last_err.source() {
        last_err = source;
    }
    last_err.to_string()
            .split(":")
            .collect::<Vec<&str>>()
            .first()
            .unwrap()
            .to_string()
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let url_limit = get_env("LIMIT", 1000);
    let request_timeout = get_env("REQUEST_TIMEOUT", 5);
    let concurrency = get_env("CONCURRENCY", 10);
    let data_dir = env::var("DATA_DIR").unwrap_or("./data".to_string());

    println!("Starting index.:");
    println!(" * {}: {:?}", "URL_LIMIT", url_limit);
    println!(" * {}: {:?}", "REQUEST_TIMEOUT", request_timeout);
    println!(" * {}: {:?}", "CONCURRENCY", concurrency);

    let time = SystemTime::now();

    let file = File::open(format!("{}/urls.txt", data_dir))?;
    let reader = BufReader::new(file);

    let results = stream::iter(reader.lines().take(url_limit as usize))
        .map(|line| {
            async move {
                let start = SystemTime::now();
                let url = line.unwrap();

                let client = reqwest::Client::builder()
                    .build()
                    .unwrap();

                let response = client.get(&url)
                    .timeout(Duration::from_secs(request_timeout as u64))
                    .header(USER_AGENT, "crawler-test")
                    .send()
                    .await;

                let time = start.elapsed()
                                .unwrap()
                                .as_millis();

                let code = match response {
                    Ok(response) => handle_response(response).await,
                    Err(err) => handle_error(err).await,
                };

                println!("{}: {} -- {:?}ms", url, code, time);
                Result { code, time }
            }
        })
        .buffer_unordered(concurrency as usize)
        .collect::<Vec<Result>>()
        .await;


    let aggregates = results.iter().fold(HashMap::new(), |mut acc, result| {
        *acc.entry(result.code.clone()).or_insert(0) += 1;
        acc
    });

    let mut sorted_aggregates = aggregates.iter().collect::<Vec<(&String, &u32)>>();

    sorted_aggregates.sort_by(|a, b| b.1.cmp(&a.1));

    for (code, count) in &mut sorted_aggregates {
        println!("{}: {}", code, count);
    }

    let total_time = time.elapsed().unwrap().as_secs();
    let total_urls = aggregates.values().sum::<u32>();
    let mut sorted_times = results.iter().map(|result| result.time).collect::<Vec<u128>>();
    sorted_times.sort_by(|a, b| b.cmp(a));
    let avg_time = sorted_times.iter().sum::<u128>() / total_urls as u128;
    let median_time = sorted_times[sorted_times.len() / 2];

    println!("Total time: {:?}s", total_time);
    println!("Total URLs: {:?}", total_urls);
    println!("Average time: {:?}ms", avg_time);
    println!("Median time: {:?}ms", median_time);

    process::exit(0);
}
