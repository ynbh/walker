use scraper::{Html, Selector};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Args {
    pub url: String,
}

#[derive(Debug)]
pub struct Urls {
    urls: Vec<String>,
}

impl Urls {
    fn push(&mut self, value: String) {
        self.urls.push(value)
    }
}

impl Args {
    pub async fn get_html(&self) -> Result<String, Box<dyn std::error::Error>> {
        let res = reqwest::get(&self.url).await?.text().await?;

        Ok(res)
    }

    pub async fn parse_html(&self) -> Html {
        let html = self.get_html().await.unwrap();
        let document = Html::parse_document(html.as_str());

        return document;
    }

    pub fn get_tag_by_name(&self, tag: &str) -> Selector {
        Selector::parse(tag).unwrap()
    }

    pub fn is_relative_url(&self, url: String) -> bool {
        url.starts_with("/")
    }

    pub async fn recursively_get_links_from_website(&self) -> Urls {
        let document = self.parse_html().await;

        let a_tags_selector = self.get_tag_by_name("a");
        let a_tags = document.select(&a_tags_selector);

        let mut urls = Urls { urls: vec![] };

        for tag in a_tags {
            let value = tag.value();
            let href = match value.attr("href") {
                Some(n) => n.to_string(),
                None => break,
            };

            if href != "" {
                let href_clone = href.clone();
                if self.is_relative_url(href_clone) {
                    println!("Relative url spotted {}", href)
                }
                urls.push(href)
            }

    
        }

        return urls;
    }
}
