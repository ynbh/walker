use colored::*;
use regex::Regex;
use reqwest::Url;
use reqwest::{header::USER_AGENT, Client};
use scraper::{Html, Selector};
use rayon::prelude::*;

use std::collections::hash_set::HashSet;
use std::process;

use async_recursion::async_recursion;

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
    pub fn new(
        url: String,
        search_relative: bool,
        debug: bool,
        client: Client,
        set: HashSet<String>,
    ) -> Args {
        Args {
            url,
            search_relative,
            debug,
            client,
            set,
        }
    }

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

        if debug == Some(true) {
            println!(
                "{}",
                format!("{} Fetching {url}", "[DEBUG]".bright_magenta()).bright_yellow()
            )
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
        let html = match self.get_html(nested_url).await {
            Ok(n) => n,
            Err(_) => "".into(),
        };

        let document = Html::parse_document(html.as_str());

        return document;
    }

    pub fn get_tag_by_name(&self, tag: &str) -> Selector {
        Selector::parse(tag).unwrap()
    }

    // @credit https://github.com/sindresorhus/is-absolute-url/blob/main/index.js
    pub fn is_absolute_url(&self, url: &str) -> bool {
        let windows_regex = Regex::new(r"^[a-zA-Z]:\\").unwrap();
        let absolute_regex = Regex::new(r"^[a-zA-Z][a-zA-Z\d+\-.]*?:").unwrap();

        if windows_regex.is_match(url) {
            return false;
        }

        absolute_regex.is_match(url)
    }

    // Checks if encountered URLs follow format like `/walker` and `../walker`
    pub fn is_relative_url(&self, url: &str) -> bool {
        !self.is_absolute_url(url)
    }

    // Functions to fix URL
    pub fn remove_fragment(&self, url: String) -> String {
        let mut parsed = match Url::parse(&url) {
            Ok(n) => n,
            Err(e) => {
                println!("COULD NOT parse {url}: {e}");
                process::exit(1)
            }
        };

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

    pub async fn filter_hrefs(&mut self, url: &str, tag: &str, attribute: &str) -> Vec<String> {
        let document = self.parse_html(url.to_string()).await;
        let selector = self.get_tag_by_name(tag);
        let tags = document.select(&selector);

        let mut hrefs = Vec::new();

        for tag in tags {
            let value = tag.value();
            let value_at_attr = match value.attr(attribute) {
                Some(n) => {
                    let effective_href = match n {
                        "" => &self.url,
                        _ => n,
                    }
                    .to_string();
                    effective_href
                }
                None => String::from(""),
            };

            hrefs.push(value_at_attr)
        }

        hrefs.into_par_iter().filter(|k| k != "").collect()
    }

    // Get all anchor tags from the parsed HTML
    pub async fn get_all_links(&mut self, url: String) -> Vec<String> {
        let url = self.remove_fragment(url);

        let mut v = vec![];

        if self.set.contains(&url) {
            return v;
        }

        let anchor_tags = self.filter_hrefs(&url, "a", "href").await;
       
        // This does not filter out image tags. Will need to investigate _why_.
        // let img_tags = self.filter_hrefs(&url, "img", "src").await;

        for a in anchor_tags {
            if !a.starts_with("#") {
                v.push(a);
            }
        }


        // for img in img_tags {
        //     v.push(img)
        // }

        v
    }

    // Main Function

    #[async_recursion]
    /**
     * Get a list of all URLs found on the page at the provided URL.
     * @param url The URL to crawl.
     * @returns A list of URLs found on the page.
     */
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
            .get_all_links(self.remove_trailing_slashes(effective_url.clone()))
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
}
