use std::collections::HashSet;

mod walker;

fn main() {
    let args = walker::Args {
        url: "https://ray.so".to_string(),
    };

    let mut set: HashSet<String> = HashSet::new();
    println!("Running...");
    let links = args.recursively_get_links_from_website(None, &mut set);

    if links.urls.len() == 0 {
        eprintln!("It looks like the site is probably client-side rendered. In this case, something like puppeteer would be needed.")
    } else {
        println!("All URLs: {:#?}", links)
    }
}
