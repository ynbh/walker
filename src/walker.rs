use reqwest::{header::USER_AGENT, Client};
use scraper::{Html, Selector};
use std::collections::hash_set::HashSet;
use reqwest::Url;

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

        match self
            .client
            .get(url)
            .header(USER_AGENT, "Walker - Recursive link checker.")
            .send()
            .await
        {
            Ok(n) => Ok(n),
            Err(e) => Err(format!("ERROR: cannot fetch URL: {}", e)),
        }
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

    pub fn get_domain_name(&self, url: Url) -> String {
        match url.domain() {
            Some(n) => n.to_string(),
            None => format!("Cannot resolve domain for {:#?}", url),
        }
    }

    pub async fn filter_a_tags(&self, url: String) -> Vec<String> {
        let mut v = vec![];
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
    pub fn get_effective_href(&self, href: String) -> String {
        let parsed = Url::parse(&self.url).unwrap();

        let base_url = match self.base_url(parsed) {
            Ok(n) => self.remove_trailing_slashes(n.to_string()),
            Err(e) => format!("An error occurred: {}", e),
        };

        let relative_split = href.split(self.url.as_str()).collect::<Vec<&str>>()[0];

        format!("{}{}", base_url, relative_split)
    }

    #[async_recursion]
    pub async fn recursively_get_links_from_website(&mut self, url: Option<String>) -> URLs {
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

        if !self.set.contains(&effective_url) {
            let a_tags = self
                .filter_a_tags(self.remove_trailing_slashes(effective_url))
                .await;

            for href in a_tags {
                if self.is_relative_url(&href) {
                    let parent_url = self.remove_trailing_slashes(self.get_effective_href(href));
                    if self.search_relative {
                        let nested_a_tags = self
                            .filter_a_tags(parent_url)
                            .await
                            .into_iter()
                            .map(|x| {
                                let curr = if self.is_relative_url(&x) {
                                    self.remove_trailing_slashes(self.get_effective_href(x))
                                } else {
                                    x
                                };
                                if !self.set.contains(&curr) {
                                    return curr;
                                } else {
                                    return "".to_string();
                                }
                            })
                            .collect::<Vec<String>>();

                        for tag in nested_a_tags {
                            // println!("diagnosis: 133 {}", tag);
                            let cl = tag.clone();
                            let cl2 = tag.clone();
                            urls.push(cl2);
                            if tag.starts_with(&self.url) {
                                if !self.set.contains(&tag) {
                                    let a_tags = self.filter_a_tags(tag).await;

                                    self.set.insert(cl);

                                    for href in &a_tags {
                                        if self.is_relative_url(&href) {
                                            let fixed = self.remove_trailing_slashes(
                                                self.get_effective_href(href.to_string()),
                                            );
                                            let cl = fixed.clone();
                                            let cl2 = fixed.clone();
                                            urls.push(cl);
                                            if !self.set.contains(&fixed) {
                                                let recursed_urls = self
                                                    .recursively_get_links_from_website(Some(fixed))
                                                    .await;

                                                for link in recursed_urls.urls {
                                                    let cl = link.clone();
                                                    urls.push(link);
                                                    if !self.set.contains(&cl) {
                                                        let link_cl = cl.clone();

                                                        self.insert(link_cl);
                                                    }
                                                }

                                                self.insert(cl2);
                                            }
                                        } else {
                                            let fixed =
                                                self.remove_trailing_slashes((&href).to_string());
                                            let fixed_cl = fixed.clone();
                                            urls.push(fixed);
                                            self.insert(fixed_cl);
                                        }
                                    }
                                }
                            } else {
                                let cl = tag.clone();

                                urls.push(tag);
                                self.insert(cl);
                            }
                        }
                    } else {
                        let cl = parent_url.clone();
                        urls.push(parent_url);
                        self.insert(cl);
                    }
                } else {
                    let cl = href.clone();
                    let fixed = self.remove_trailing_slashes(cl);

                    let cl_fixed = fixed.clone();
                    urls.push(cl_fixed);
                    if !self.set.contains(&fixed) {
                        self.insert(fixed);
                    }
                }
            }
        }

        urls
    }

    fn insert(&mut self, value: String) {
        if value != "" {
            if !self.set.contains(&value) {
                self.set.insert(value.clone());
            }
        }
    }
}
