use std::collections::HashSet;
use std::fs::write;
use std::path::PathBuf;
use std::time::Instant;

use arboard::Clipboard;
use colored::*;

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
}
fn main() {
    let cli_args = CLIArgs::parse();

    let args = walker::Args {
        url: cli_args.url,
        search_relative: cli_args.relative,
        debug: cli_args.debug,
    };

    let mut set: HashSet<String> = HashSet::new();
    println!("Running...");
    let now = Instant::now();
    let links = args.recursively_get_links_from_website(None, &mut set);
    let get_elapsed = now.elapsed().as_secs().to_string().bright_magenta();

    if links.urls.len() == 0 {
        eprintln!("It looks like the site is probably client-side rendered. In this case, something like puppeteer would be needed.")
    } else {
        let now = Instant::now();
        println!("Received {} links. Iterating now...", links.urls.len());
        let mut response = String::new();
        for link in &links.urls {
            if !link.starts_with("mailto") {
                let status = args.is_broken(link.to_string());
                let msg = match status.as_str() {
                    "200 OK" => "✅".to_string(),
                    "URL Error" => "CANNOT RESOLVE ❌".bright_red().to_string(),
                    _ => "❌".to_string(),
                };

                if cli_args.construct {
                    response.push_str(format!("{}: {}\n", link, msg).as_str());
                }

                println!("{}: {}", link, msg)
            }
        }
        let mut clipboard = Clipboard::new().unwrap();

        let loop_elapsed = now.elapsed().as_secs().to_string().bright_magenta();

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
                Err(e) => println!(
                    "{}",
                    format!(
                        "Some error occurred while copying to clipboard: {}",
                        e.to_string().bright_red()
                    )
                ),
            };
        }

        pipe_output(
            links.urls,
            args.remove_trailing_slashes(args.url.to_string()),
        )
        .unwrap();
    }
}

fn pipe_output(set: HashSet<String>, url: String) -> std::io::Result<String> {
    let links = serde_json::to_string(&set)?;
    let save_path = format!("{}.json", url);

    let cl = save_path.clone();

    let readable_file_path = PathBuf::from("data/").join(save_path);
    let rd_cl = readable_file_path.clone();
    match write(readable_file_path, links) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Tried to save at {:#?}", rd_cl);
            eprintln!("{}", format!("Some error occurred: {}", e));
        }
    }

    let success_message = format!("Saved to {}", cl);
    Ok(success_message)
}
