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
    fn push(&mut self, value: String) {
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
    pub async fn get(&self, url: String, debug: Option<bool>) -> Result<reqwest::Response, String> {
        if debug.unwrap_or(false) {
            println!("{}", format!("[DEBUG] Fetching {url}").bright_purple())
        }

        if self.set.contains(&url) {
            if self.set.contains(&url) {
                return Err("URL already visited".to_string());
            }
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

    pub async fn _is_broken(&self, url: String) -> String {
        let status = match self.get(url, None).await {
            Ok(n) => n.status().to_string(),
            Err(_) => "URL Error".to_string(),
        };

        status
    }

    pub async fn get_html(&self, url: String) -> Result<String, Box<dyn std::error::Error>> {
        let res = self.get(url, Some(self.debug)).await?.text().await?;

        Ok(res)
    }

    pub async fn parse_html(&self, nested_url: String) -> Html {
        let html = self.get_html(nested_url).await.unwrap();
        let document = Html::parse_document(html.as_str());

        return document;
    }

    pub fn get_tag_by_name(&self, tag: &str) -> Selector {
        Selector::parse(tag).unwrap()
    }

    pub fn is_relative_url(&self, url: &String) -> bool {
        url.starts_with("/") || url.starts_with(".")
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
    pub async fn filter_a_tags(&self, url: String) -> Vec<String> {
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

    pub fn remove_trailing_slashes(&self, mut url: String) -> String {
        while url.ends_with('/') {
            url = url.trim_end_matches('/').to_string();
        }
        url
    }

    pub fn remove_fragment(&self, url: String) -> String {
        let mut parsed = Url::parse(&url).unwrap();

        parsed.set_fragment(None);

        parsed.to_string()
    }

    pub fn get_effective_href(&self, href: String) -> String {
        let mut parsed = Url::parse(&self.url).unwrap();

        parsed.set_fragment(None);

        let resultant = parsed.join(&href).unwrap().to_string();

        resultant
    }

    #[async_recursion]
    pub async fn walk(&mut self, url: Option<String>) -> URLs {
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

        // Get all URLs from current URL
        let anchors = self
            .filter_a_tags(self.remove_trailing_slashes(effective_url))
            .await;

        for href in anchors {
            // Is the URL relative?
            if self.is_relative_url(&href) {
                // prepend parent URl to the relative URL
                let fixed = self.get_effective_href(href);
                if self.search_relative {
                    // filter out the tags according to cache
                    let nested_anchors = self
                        .filter_a_tags(fixed)
                        .await
                        .into_iter()
                        .map(|x| {
                            let curr = if self.is_relative_url(&x) {
                                self.get_effective_href(x)
                            } else {
                                self.remove_fragment(x)
                            };
                            if !self.set.contains(&curr) {
                                return curr;
                            } else {
                                return "".to_string();
                            }
                        })
                        .filter(|blank| blank != "") // apparently pre-filtering out cached URLs stops making this work?
                        .collect::<Vec<String>>();

                    // again iterate over received URLs
                    for nested_href in nested_anchors {
                        let cl = nested_href.clone();
                        let cl2 = nested_href.clone();
                        urls.push(cl2);

                        // check if relative since I already fixed them after mapping them while checking nested_a_tags
                        if nested_href.starts_with(&self.url) {
                            if !self.set.contains(&nested_href) {
                                let deep_nested_anchors = self.filter_a_tags(nested_href).await;

                                self.set.insert(cl);

                                for deep_nested_href in &deep_nested_anchors {
                                    if self.is_relative_url(&deep_nested_href) {
                                        let fixed =
                                            self.get_effective_href(deep_nested_href.to_string());

                                        let cl = fixed.clone();
                                        let cl2 = fixed.clone();
                                        urls.push(cl);
                                        if !self.set.contains(&fixed) {
                                            let recursed_anchors = self.walk(Some(fixed)).await;

                                            for recursed_href in recursed_anchors.urls {
                                                urls.push(recursed_href.clone());
                                                if !self.set.contains(&recursed_href.clone()) {
                                                    self.insert(recursed_href);
                                                }
                                            }

                                            self.insert(cl2);
                                        }
                                    } else {
                                        let fixed =
                                            self.remove_fragment((&deep_nested_href).to_string());

                                        urls.push(fixed.clone());
                                        self.insert(fixed);
                                    }
                                }
                            }
                        } else {
                            urls.push(nested_href.clone());
                            self.insert(nested_href);
                        }
                    }
                } else {
                    urls.push(fixed.clone());
                    self.insert(fixed);
                }
            } else {
                let fixed = self.remove_fragment(href);

                urls.push(fixed.clone());
                if !self.set.contains(&fixed) {
                    self.insert(fixed);
                }
            }
        }

        urls
    }

    fn insert(&mut self, value: String) {
        if value != "" {
            if !self.set.contains(&value) {
                // why does this insert when the URL is already in the set?
                self.set.insert(value.clone());
            }
        }
    }
}
