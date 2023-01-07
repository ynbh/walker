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
}

impl Urls {
    fn push(&mut self, value: String) {
        self.urls.push(value)
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

    fn filter_a_tags(&self, url: String) -> Vec<String> {
        let mut v = vec![];
        let document = self.parse_html(url);

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

    pub fn recursively_get_links_from_website(
        &self,
        url: Option<String>,
        map: &mut HashMap<String, i32>,
    ) -> Urls {
        let effective_url = match url {
            Some(k) => k,
            None => {
                let cloned_url = &self.url;

                cloned_url.to_string()
            }
        };

        let a_tags = self.filter_a_tags(effective_url);

        let mut urls = Urls { urls: vec![] };

        for href in a_tags {
            if self.is_relative_url(&href) {
                let effective_href = self.get_effective_href(href);

                let cl1 = effective_href.clone();
                let cl2 = effective_href.clone();

                if !map.contains_key(&effective_href) {
                    let current_tags = self.filter_a_tags(cl1);

                    for link in current_tags {
                        if self.is_relative_url(&link) {
                            let fixed_link =
                                self.remove_trailing_slashes(self.get_effective_href(link));
                            let nested_urls =
                                self.recursively_get_links_from_website(Some(fixed_link), map);

                            for url in nested_urls.urls {
                                if !map.contains_key(&url) {
                                    let cl = url.clone();
                                    urls.push(url);
                                    map.insert(cl, 1);
                                }
                            }
                        } else {
							let fixed = self.remove_trailing_slashes(link);
                            if !map.contains_key(&fixed) {
								let cl = fixed.clone();
                                urls.push(fixed);
								map.insert(cl, 1);
                            }
                        }
                    }
                }

                // let clone_1 = effective_href.clone();
                // let clone_2 = effective_href.clone();
                // if !map.contains_key(&effective_href) {
                //     {
                //         let current_tags = self.filter_a_tags(clone_1);
                //         for href in current_tags {

                //             // let eff = {

                // 			// 	{
                // 			// 		if self.is_relative_url(&href) {

                // 			// 		}
                // 			// 	}
                //             //     // let current = if self.is_relative_url(&href) {
                //             //     //     let fixed =
                //             //     //         self.remove_trailing_slashes(self.get_effective_href(href));

                //             //     //     fixed
                //             //     // } else {
                //             //     //     let fixed = self.remove_trailing_slashes(href);
                //             //     //     fixed
                //             //     // };

                //             //     current
                //             // };

                //             map.insert(eff, 1);
                //         }
                //     }
                //     urls.push(effective_href);
                //     map.insert(clone_2, 1);
                // }
            } else {
                let cloned = href.clone();
                if !map.contains_key(&href) {
                    urls.push(cloned);
                    map.insert(href, 1);
                }
            }
        }

        return urls;
    }

    pub fn get_effective_href(&self, href: String) -> String {
        let parsed = Url::parse(&self.url).unwrap();
        let base_url = self.base_url(parsed).unwrap().to_string();

        let relative_split = href.split("https://ynb.sh").collect::<Vec<&str>>()[0];

        format!(
            "{}{}",
            self.remove_trailing_slashes(base_url),
            relative_split
        )
    }
}
