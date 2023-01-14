use std::collections::HashSet;
use std::time::{Duration, Instant};

use futures::future::join_all;

use reqwest::{StatusCode, Url};

use arboard::Clipboard;
use colored::*;
use std::net::ToSocketAddrs;

use clap::Parser;
use utils::{get_domain_name, is_valid_url, store_output};

mod utils;
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
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    store_output(
        links.urls.clone(),
        args.remove_trailing_slashes(get_domain_name(args.url.to_string())),
    )
    .unwrap();

    // let status_spawn = get_status(
    //     sing,
    //     links
    //         .urls
    //         .clone()
    //         .into_iter()
    //         .filter(|x| !x.starts_with("mailto"))
    //         .collect(),
    //     Some(cli_args.debug),
    // )
    // .await;

    let status_futures = check_status(
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
        let now = Instant::now();
        println!("Received {} links. Iterating now...", links.urls.len());
        let mut response = String::new();

        for t in status_futures {
            // let t = href.await.unwrap();

            let (k, v) = match t {
                Ok((url, status_code)) => {
                    let code = if status_code.to_string() == "200 OK" {
                        status_code.to_string().bright_green()
                    } else {
                        status_code.to_string().bright_red()
                    };
                    println!("{}", format!("{}: {}", url, format!("{}", code)));

                    (format!("{url}: "), format!("{status_code}"))
                }

                Err((k, e)) => {
                    println!(
                        "{}",
                        format!("{}: {}", k, format!("{:#?}", e.to_string()).bright_red())
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

async fn check_status(
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
    let futures = correct.iter().map(|url| {
        let req = client.head(url);
        if debug.unwrap() {
            println!("Verifying {}", url.bright_yellow());
        }
        async move {
            let cl = url.clone();
            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();

                    Ok((cl, status))
                }
                Err(err) => Err((cl, err.to_string())),
            }
        }
    });
    let results = join_all(futures).await;

    results
}
