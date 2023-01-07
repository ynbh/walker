use scraper::{Html, Selector};
use std::collections::hash_set::HashSet;
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
        set: &mut HashSet<String>,
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
			
			let parent_url = self.remove_trailing_slashes(self.get_effective_href(href));

			let nested_a_tags = self.filter_a_tags(parent_url);

			println!("{:#?}", nested_a_tags);
		} else {
			let cl = href.clone();
			let fixed = self.remove_trailing_slashes(cl);

			let cl_fixed = fixed.clone(); 

			urls.push(fixed);
			set.insert(cl_fixed);
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
