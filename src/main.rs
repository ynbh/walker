use std::collections::HashSet;
use std::fs::write;
use std::path::PathBuf;

use clap::Parser;

mod walker;

/// Tool to recursively analyze links from a website.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CLIArgs {
    /// URL of the website to analyze links from.
    #[arg(short, long)]
    url: String,
}
fn main() {
    let url = CLIArgs::parse().url;

    let args = walker::Args { url };

    let mut set: HashSet<String> = HashSet::new();
    println!("Running...");
    let links = args.recursively_get_links_from_website(None, &mut set);

    if links.urls.len() == 0 {
        eprintln!("It looks like the site is probably client-side rendered. In this case, something like puppeteer would be needed.")
    } else {
        for link in &links.urls {
            if !link.starts_with("mailto") {
                let status = args.is_broken(link.to_string());

                println!(
                    "{}: {}",
                    link,
                    match status.as_str() {
                        "200 OK" => "✅".to_string(),
                        "URL Error" => "CANNOT RESOLVE ❌".to_string() ,
                        _ => "❌".to_string(),
                    }
                )
            }
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
