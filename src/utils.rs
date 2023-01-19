use futures::StreamExt;
use reqwest::{StatusCode, Url};
use std::fs::{create_dir, write};

pub fn is_valid_url(url: String) -> bool {
    url.starts_with("http") || url.starts_with("https")
}

pub fn save(str: String, url: &String, key: &str, format: &str) -> std::io::Result<()> {
    let working_dir = format!("./data/{url}");
    let save_path = format!("/{}-{key}.{format}", url);

    match create_dir(format!("{working_dir}")) {
        Ok(_) => {
            println!("Creating directory...");
            let cl = working_dir.clone() + &save_path;
            let links_cl = str.clone();

            match write(cl.clone(), links_cl) {
                Ok(_) => {
                    println!("{}", format!("Saved {key} to {cl}"))
                }
                Err(e) => println!("Some error occurred: {}", e),
            }
        }
        Err(_e) => {
            println!("Directory already exists. Writing to file now.");
            let cl = working_dir.clone() + &save_path;
            let links_cl = str.clone();

            match write(cl.clone(), links_cl) {
                Ok(_) => {
                    println!("{}", format!("Saved {key} to {cl}"))
                }
                Err(e) => println!("Some error occurred: {}", e),
            }
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
) -> Vec<Result<(String, StatusCode), (String, String)>> {
    let correct = urls
        .clone()
        .into_iter()
        .filter(|x| {
            let is_valid_url = is_valid_url(x.to_string());
            let matchable = match Url::parse(x) {
                Ok(_) => true,
                Err(_) => false,
            };

            if !is_valid_url || !matchable {
                return false;
            }

            true
        })
        .collect::<Vec<String>>();

    let statuses = futures::stream::iter(correct.into_iter().map(|url| {
        if debug.unwrap() {
            println!("Verifying {}", url);
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
    .buffer_unordered(50)
    .collect::<Vec<_>>();

    statuses.await
}
