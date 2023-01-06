mod walker;



#[tokio::main]
async fn main() {
    let args = walker::Args {
        url: "https://ynb.sh".to_string()
    };


    let links = args.recursively_get_links_from_website(None).await;

    println!("{:#?}", links)
}
