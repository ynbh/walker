use std::collections::HashMap;

mod walker;

fn main() {
    let args = walker::Args {
        url: "https://ynb.sh".to_string(),
    };

    let mut hashmap: HashMap<String, i32> = HashMap::new();
    let links = args.recursively_get_links_from_website(None, &mut hashmap);

    println!("{:#?} {:#?}", links, hashmap)
    
}
