use scraper::{Html, Selector};
use std::collections::HashMap;
use url::Url;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Args {
    pub url: String,
}

#[derive(Debug)]
pub struct Urls {
    pub urls: Vec<String>,
    pub map: HashMap<String, i32>,
}

impl Urls {
    fn push(&mut self, value: String) {
        self.urls.push(value)
    }

    fn add_map(&mut self, value: String) -> Option<i32> {
        self.map.insert(value, 1)
    }
}

impl Args {
    pub fn get_html(&self, url: String) -> Result<String, Box<dyn std::error::Error>> {
        let res = reqwest::blocking::get(url)?.text()?;

        Ok(res)
    }

    pub fn parse_html(&self, nested_url: String) -> Html {
        let html = self.get_html(nested_url).unwrap();
        let document = Html::parse_document(html.as_str());

        return document;
    }

    pub fn get_tag_by_name(&self, tag: &str) -> Selector {
        Selector::parse(tag).unwrap()
    }

    pub fn is_relative_url(&self, url: String) -> bool {
        url.starts_with("/")
    }

    pub fn base_url(&self, mut url: Url) -> Result<Url, &str> {
        match url.path_segments_mut() {
            Ok(mut path) => {
                path.clear();
            }
            Err(_) => {
                return Err("Cannot base URL");
            }
        }

        url.set_query(None);

        Ok(url)
    }

    pub fn recursively_get_links_from_website(&self, url: Option<String>) -> Urls {
        let effective_url = match url {
            Some(k) => k,
            None => {
                let cloned_url = &self.url;

                cloned_url.to_string()
            }
        };

        let document = self.parse_html(effective_url);

        let a_tags_selector = self.get_tag_by_name("a");
        let a_tags = document.select(&a_tags_selector);

        let mut urls = Urls {
            urls: vec![],
            map: HashMap::new(),
        };

        for tag in a_tags {
            let value = tag.value();
            let href = match value.attr("href") {
                Some(n) => n.to_string(),
                None => break,
            };

            if href != "" {
                urls.add_map(href.clone());
                urls.push(href)
            }
        }

        return urls;
    }
}
