use std::collections::HashSet;
use std::net::ToSocketAddrs;
use std::time::{Duration, Instant};

use arboard::Clipboard;
use clap::Parser;
use colored::*;
use futures::future::join_all;
use reqwest::{StatusCode, Url};
use rayon::prelude::*;

use parse::parse;
use stats::Stats;
use utils::{get_domain_name, save, get_status_buffer_unordered};
use walker::Args;

mod parse;
mod stats;
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

    /// Checks if the domain of the URL is resolvable.
    #[arg(short, long, default_value_t = false)]
    singular: bool,
}

#[tokio::main]
async fn main() {
    let cli_args = CLIArgs::parse();

    let mut args = Args::new(
        cli_args.url,
        cli_args.relative,
        cli_args.debug,
        reqwest::Client::new(),
        HashSet::new(),
    );

    if cli_args.singular {
        let parsed_response = Url::parse(&args.url).unwrap();
        let base_url = args._base_url(parsed_response).unwrap().to_string();
        let domain: String = args.remove_trailing_slashes(
            base_url.split("//").into_iter().collect::<Vec<&str>>()[1].to_string(),
        );

        return println!("{:#?}", dbg!((domain, 80).to_socket_addrs()));
    }

    println!("Running...");
    let now = Instant::now();
    let links = args.walk_sync(None).await;
    let get_elapsed = now.elapsed().as_secs().to_string();

    let sing = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(8))
        .build()
        .unwrap();
    let file_path = args.remove_trailing_slashes(get_domain_name(args.url.to_string()));

    save(
        serde_json::to_string_pretty(&links.urls).unwrap(),
        &file_path,
        "links",
        "json",
    )
    .unwrap();

    let status_vec = get_status_buffer_unordered(
        sing,
        links
            .urls
            .clone()
            .into_par_iter()
            .filter(|x| !x.starts_with("mailto") || !x.starts_with("file:"))
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

        for t in status_vec {
            let (k, v) = match t {
                Ok((url, status_code)) => {
                    let code = if status_code.to_string() == "200 OK" {
                        status_code.to_string().bright_green()
                    } else {
                        status_code.to_string().bright_red()
                    };

                    let display_message = format!("{url}: {code}");
                    println!("{}", display_message);

                    (format!("{url}: "), format!("{status_code}"))
                }

                Err((k, e)) => {
                    println!("{k}: {}", format!("{}", e.to_string().bright_red()));

                    (format!("ERROR: "), format!("{:#?}", e.to_string()))
                }
            };

            if cli_args.construct {
                response.push_str(format!("{k}{v}\n",).as_str());
            }
        }

        let loop_elapsed = now.elapsed().as_secs().to_string();

        let link_count = links.urls.len();

        let stats = Stats::new(get_elapsed, loop_elapsed, link_count);

        println!("{stats}");

        let jsonified_response = parse(response.clone());

        save(jsonified_response.clone(), &file_path, "status", "json").unwrap();

        save(stats.to_markdown(), &file_path, "stats", "md").unwrap();

        if cli_args.construct {
            let mut clipboard = match Clipboard::new() {
                Ok(clipboard) => clipboard,
                Err(e) => {
                    eprintln!(
                        "{}",
                        format!(
                            "Some error occurred while initializing clipboard: {}",
                            e.to_string().bright_red()
                        )
                    );
                    return;
                }
            };

            match clipboard.set_text(jsonified_response) {
                Ok(_) => println!("Copied JSON response to clipboard."),
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


#[allow(dead_code)]
async fn check_status(
    client: reqwest::Client,
    urls: Vec<String>,
    debug: Option<bool>,
) -> Vec<Result<(String, StatusCode), (String, String)>> {
    let correct = urls
        .into_par_iter()
        .map(|url| {
            let mut parsed_url = Url::parse(&url).unwrap();

            // just making sure for now.
            parsed_url.set_fragment(None);

            parsed_url.to_string()
        })
        .collect::<HashSet<String>>();

    let futures = correct.iter().map(|url| {
        let req = client.clone().head(url);
        if debug == Some(true) {
            println!("{} {}", "[Verifying]".bright_magenta(), url.bright_yellow());
        }
        async {
            let cl = url.clone();
            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    Ok((cl, status))
                }
                // @TODO: Perform a GET request if HEAD fails.
                Err(err) => Err((cl, err.to_string())),
            }
        }
    });
    let results = join_all(futures).await;

    results
}
