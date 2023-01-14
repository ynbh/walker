use futures::StreamExt;
use reqwest::{StatusCode, Url};
use std::collections::HashSet;
use std::env;
use std::fs::{create_dir, write};
use std::path::PathBuf;

pub fn is_valid_url(url: String) -> bool {
    url.starts_with("http") || url.starts_with("https")
}

pub fn store_output(set: HashSet<String>, url: String) -> std::io::Result<String> {
    let links = serde_json::to_string(&set)?;
    let save_path = format!("/{}.json", url);

    let current_dir = get_current_working_dir();
    let working_dir = current_dir + "/data";

    match create_dir(format!("{working_dir}")) {
        Ok(n) => n,
        Err(_e) => {
            println!("Directory already exists. Writing to file now.");
            let cl = working_dir.clone() + &save_path;
            let links_cl = links.clone();

            match write(cl.clone(), links_cl) {
                Ok(n) => n,
                Err(e) => println!("Some error occurred: {}", e),
            }
            return Ok(format!("Saved to {cl}"));
        }
    }

    let cl = save_path.clone();

    let readable_file_path = PathBuf::from(working_dir).join(save_path);
    let rd_cl = readable_file_path.clone();
    match write(readable_file_path, links) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Tried to save at {:#?}", rd_cl);
            eprintln!("{}", format!("Some error occurred: {}", e));
        }
    }

    let success_message = format!("Saved to {}", cl);
    println!("{}", success_message);
    Ok(success_message)
}

pub fn get_current_working_dir() -> String {
    let res = env::current_dir();
    match res {
        Ok(path) => path.into_os_string().into_string().unwrap(),
        Err(_) => "FAILED".to_string(),
    }
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
                Err(err) => Err((href, err.to_string())),
            }
        }
    }))
    .buffer_unordered(50)
    .collect::<Vec<_>>();

    statuses.await
}
