use colored::Colorize;
use futures::StreamExt;
use reqwest::{StatusCode, Url};
use std::fs::{create_dir_all, write};

pub fn save(str: String, url: &str, key: &str, format: &str) -> std::io::Result<()> {
    let working_dir = format!("./link-walker/data/urls/{url}");
    let save_path = format!("/{}-{key}.{format}", url);

    let final_destination = working_dir.clone() + &save_path;

    match create_dir_all(format!("{working_dir}")) {
        Ok(_) => {
            let links_cl = str.clone();

            match write(final_destination.clone(), links_cl) {
                Ok(_) => {
                    println!(
                        "{}",
                        format!("Saved {key} to {final_destination}").underline()
                    )
                }
                Err(e) => println!("Some error occurred: {}", e),
            }
        }
        Err(e) => {
            eprintln!(
                "Some error occurred while trying to save {key} at {final_destination}: {}",
                e.to_string().bright_red()
            );
        }
    }

    Ok(())
}

pub fn get_domain_name(url: String) -> String {
    let url = Url::parse(url.as_str()).unwrap();

    match url.domain() {
        Some(n) => n.to_string(),
        None => format!("Cannot resolve domain for {:#?}", url),
    }
}

#[allow(dead_code)]
pub async fn get_status_buffer_unordered(
    client: reqwest::Client,
    urls: Vec<String>,
    debug: Option<bool>,
    threads: Option<usize>,
) -> Vec<Result<(String, StatusCode), (String, String)>> {
    let statuses = futures::stream::iter(urls.into_iter().map(|url| {
        if debug == Some(true) {
            println!("{} {}", "[Verifying]".bright_magenta(), url.bright_yellow());
        }
        let href = url.clone();

        let head = client.head(url);
        async move {
            match head.send().await {
                Ok(resp) => {
                    let status = resp.status();

                    Ok((href, status))
                }
                // @TODO: Perform a GET request if HEAD fails.
                Err(err) => Err((href, err.to_string())),
            }
        }
    }))
    .buffer_unordered(threads.unwrap_or(500))
    .collect::<Vec<_>>();

    statuses.await
}
