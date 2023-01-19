use reqwest::Url;
use reqwest::{header::USER_AGENT, Client};
use scraper::{Html, Selector};

use std::collections::hash_set::HashSet;

use async_recursion::async_recursion;
use colored::*;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Args {
    pub url: String,
    pub search_relative: bool,
    pub debug: bool,
    pub client: Client,
    pub set: HashSet<String>,
}

#[derive(Debug)]
pub struct URLs {
    pub urls: HashSet<String>,
}

impl URLs {
    fn insert_and_remove_trailing_slashes(&mut self, value: String) {
        if value != "" {
            if !self.urls.contains(&value) {
                self.urls.insert(self.remove_trailing_slashes(value));
            }
        }
    }

    fn remove_trailing_slashes(&self, mut url: String) -> String {
        while url.ends_with('/') {
            url = url.trim_end_matches('/').to_string();
        }
        url
    }
}

impl Args {
    fn insert(&mut self, value: String) {
        if value != "" {
            if !self.set.contains(&value) {
                self.set.insert(value.clone());
            }
        }
    }

    pub async fn get(
        &mut self,
        url: String,
        debug: Option<bool>,
    ) -> Result<reqwest::Response, String> {
        if self.set.contains(&url) {
            return Err("URL already visited".to_string());
        }

        self.insert(url.clone());

        if debug.unwrap_or(false) {
            println!("{}", format!("[DEBUG] Fetching {url}").bright_purple())
        }

        return match self
            .client
            .get(url)
            .header(USER_AGENT, "Walker - Recursive link checker.")
            .send()
            .await
        {
            Ok(n) => Ok(n),
            Err(e) => Err(format!("ERROR: cannot fetch URL: {}", e)),
        };
    }

    // Helper functions
    pub async fn get_html(&mut self, url: String) -> Result<String, Box<dyn std::error::Error>> {
        let res = self.get(url, Some(self.debug)).await?.text().await?;

        Ok(res)
    }

    pub async fn parse_html(&mut self, nested_url: String) -> Html {
        let html = self.get_html(nested_url).await.unwrap();
        let document = Html::parse_document(html.as_str());

        return document;
    }

    pub fn get_tag_by_name(&self, tag: &str) -> Selector {
        Selector::parse(tag).unwrap()
    }

    // Checks if encountered URLs follow format like `/walker` and `../walker`
    pub fn is_relative_url(&self, url: &String) -> bool {
        url.starts_with("/") || url.starts_with(".")
    }

    // Functions to fix URL
    pub fn remove_fragment(&self, url: String) -> String {
        let mut parsed = Url::parse(&url).unwrap();

        parsed.set_fragment(None);

        parsed.to_string()
    }

    pub fn remove_trailing_slashes(&self, mut url: String) -> String {
        while url.ends_with('/') {
            url = url.trim_end_matches('/').to_string();
        }
        url
    }

    pub fn get_effective_href(&self, href: String) -> String {
        let mut parsed = Url::parse(&self.url).unwrap();

        parsed.set_fragment(None);

        let resultant = parsed.join(&href).unwrap().to_string();

        resultant
    }

    // Get all anchor tags from the parsed HTML
    pub async fn filter_anchors(&mut self, url: String) -> Vec<String> {
        let url = self.remove_fragment(url);

        let mut v = vec![];

        if self.set.contains(&url) {
            return v;
        }
        let document = self.parse_html(url).await;

        let a_tags_selector = self.get_tag_by_name("a");

        let a_tags = document.select(&a_tags_selector);

        for tags in a_tags {
            let value = tags.value();
            let href = match value.attr("href") {
                Some(n) => {
                    let effective_href = match n {
                        "" => &self.url,
                        _ => n,
                    }
                    .to_string();
                    effective_href
                }
                None => break,
            };
            if !href.starts_with("#") {
                v.push(href)
            }
        }
        v
    }

    // Main Function

    #[async_recursion]
    pub async fn walk_sync(&mut self, url: Option<String>) -> URLs {
        let effective_url = match url {
            Some(k) => k,
            None => {
                let cloned_url = &self.url;

                cloned_url.to_string()
            }
        };

        let mut urls = URLs {
            urls: HashSet::new(),
        };

        if self.set.contains(&effective_url) {
            return urls;
        }

        let anchors = self
            .filter_anchors(self.remove_trailing_slashes(effective_url.clone()))
            .await;

        for href in anchors {
            if self.is_relative_url(&href) {
                let fixed = self.get_effective_href(href.clone());
                urls.insert_and_remove_trailing_slashes(fixed.clone());
                if self.search_relative {
                    let recursed_anchors = self.walk_sync(Some(fixed)).await;
                    for recursed_href in recursed_anchors.urls {
                        urls.insert_and_remove_trailing_slashes(recursed_href);
                    }
                }
            } else {
                let fixed_href = self.remove_fragment(href.clone());
                urls.insert_and_remove_trailing_slashes(fixed_href);
            }
        }

        urls
    }

    // Unused Functions

    pub async fn _is_broken(&mut self, url: String) -> String {
        let status = match self.get(url, None).await {
            Ok(n) => n.status().to_string(),
            Err(_) => "URL Error".to_string(),
        };

        status
    }

    pub fn _base_url(&self, mut url: Url) -> Result<Url, &str> {
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
}
