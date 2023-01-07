use std::collections::HashSet;

mod walker;

fn main() {
    let args = walker::Args {
        url: "https://prf.ink".to_string(),
    };

    let mut set: HashSet<String> = HashSet::new();
    let links = args.recursively_get_links_from_website(None, &mut set);

    println!("{:#?} {:#?}", links, set)
    
}
