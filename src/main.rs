use std::collections::HashSet;
use std::env;
use std::fs::{create_dir, write};
use std::path::PathBuf;
use std::time::Instant;

use futures::future::join_all;
use reqwest::{Error, StatusCode, Url};

use arboard::Clipboard;
use colored::*;
use std::net::ToSocketAddrs;

use clap::Parser;

mod walker;

/// Tool to recursively analyze links from a website.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CLIArgs {
    /// URL of the website to analyze links from.
    #[arg(short, long)]
    url: String,

    /// Whether to perform a deep search or not.
    #[arg(short, long, default_value_t = false)]
    relative: bool,

    /// Shows what URL walker is currently on.
    #[arg(short, long, default_value_t = false)]
    debug: bool,

    /// Constructs the stream of responses into a string and copies it to the clipboard.
    #[arg(short, long, default_value_t = false)]
    construct: bool,

    #[arg(short, long, default_value_t = false)]
    singular: bool,
}

#[tokio::main]
async fn main() {
    let cli_args = CLIArgs::parse();

    let mut args = walker::Args {
        url: cli_args.url,
        search_relative: cli_args.relative,
        debug: cli_args.debug,
        client: reqwest::Client::new(),
        set: HashSet::new(),
    };
    if cli_args.singular {
        let parsed = Url::parse(&args.url).unwrap();
        let base_url = args.base_url(parsed).unwrap().to_string();
        let domain: String = args.remove_trailing_slashes(
            base_url.split("//").into_iter().collect::<Vec<&str>>()[1].to_string(),
        );

        return println!("{:#?}", dbg!((domain, 80).to_socket_addrs()));
    }

    println!("Running...");
    let now = Instant::now();
    let links = args.recursively_get_links_from_website(None).await;
    let get_elapsed = now.elapsed().as_secs().to_string().bright_magenta();

    let sing = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let statuses = check_status(
        sing,
        links
            .urls
            .clone()
            .into_iter()
            .filter(|x| !x.starts_with("mailto"))
            .collect(),
        Some(cli_args.debug),
    )
    .await;

    if links.urls.len() == 0 {
        eprintln!("It looks like the site is probably client-side rendered. In this case, something like puppeteer would be needed.")
    } else {
        let cl = links.urls.clone();

        let parsed_url = Url::parse(&args.url).unwrap();
        let base_url = args.get_domain_name(parsed_url);

        store_output(cl, args.remove_trailing_slashes(base_url)).unwrap();

        let now = Instant::now();
        println!("Received {} links. Iterating now...", links.urls.len());
        let mut response = String::new();

        for href in statuses {
            let (k, v) = match href {
                Ok((url, status_code)) => {
                    println!(
                        "{}",
                        format!("{}: {}", url, format!("{}", status_code).bright_green())
                    );

                    (format!("{url}: "), format!("{status_code}"))
                }

                Err(e) => {
                    println!(
                        "{}",
                        format!("ERROR: {}", format!("{:#?}", e.to_string()).bright_red())
                    );

                    (format!("ERROR: "), format!("{:#?}", e.to_string()))
                }
            };

            if cli_args.construct {
                response.push_str(format!("{k}{v}\n",).as_str());
            }
        }

        let loop_elapsed = now.elapsed().as_secs().to_string().bright_magenta();
        let mut clipboard = Clipboard::new().unwrap();

        let message = format!(
            "{}\n{}\n{}",
            "Stats".underline().bright_green(),
            format!(
                "{}{} {}",
                "Time to get all links: ".bright_yellow(),
                get_elapsed,
                "seconds".bright_magenta()
            ),
            format!(
                "{}{} {}",
                "Time to verify links: ".bright_yellow(),
                loop_elapsed,
                "seconds".bright_magenta()
            ),
        );
        println!("{}", message);

        if cli_args.construct {
            match clipboard.set_text(response) {
                Ok(_) => println!("Copied response to clipboard."),
                Err(e) => eprintln!(
                    "{}",
                    format!(
                        "Some error occurred while copying to clipboard: {}",
                        e.to_string().bright_red()
                    )
                ),
            };
        }
    }
}

fn store_output(set: HashSet<String>, url: String) -> std::io::Result<String> {
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

fn get_current_working_dir() -> String {
    let res = env::current_dir();
    match res {
        Ok(path) => path.into_os_string().into_string().unwrap(),
        Err(_) => "FAILED".to_string(),
    }
}

async fn check_status(
    client: reqwest::Client,
    urls: Vec<String>,
    debug: Option<bool>,
) -> Vec<Result<(String, StatusCode), Error>> {
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
    let futures = correct.iter().map(|url| {
        let req = client.head(url);
        if debug.unwrap() {
            println!("Verifying {}", url.bright_yellow());
        }
        async move {
            req.send().await.map(|response| {
                let status = response.status();
                let str_url = url.to_string();

                return (str_url, status);
            })
        }
    });
    let results = join_all(futures).await;

    results
}

fn is_valid_url(url: String) -> bool {
    url.starts_with("http") || url.starts_with("https")
}

pub fn base_url(mut url: Url) -> Result<String, String> {
    match url.path_segments_mut() {
        Ok(mut path) => {
            path.clear();
        }
        Err(_) => {
            return Err("Cannot base URL".to_string());
        }
    }

    url.set_query(None);

    Ok(url.to_string())
}
