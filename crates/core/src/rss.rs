use crate::db::{Db, RssFeedRecord, RssRuleRecord};
use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RssMatchResult {
    pub feed_id: String,
    pub rule_id: String,
    pub title: String,
    pub magnet_link: String,
    pub size: u64,
}

pub struct RssManager {
    db: Db,
    client: reqwest::Client,
}

impl RssManager {
    pub fn new(db: Db) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        Ok(Self { db, client })
    }

    pub async fn get_feeds(&self) -> Result<Vec<RssFeedRecord>> {
        self.db.get_rss_feeds()
    }

    pub async fn add_feed(&self, name: &str, url: &str) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let record = RssFeedRecord {
            id: id.clone(),
            name: name.to_string(),
            url: url.to_string(),
            last_polled_at: None,
            last_etag: None,
        };
        self.db.insert_rss_feed(&record)?;
        Ok(id)
    }

    pub async fn delete_feed(&self, id: &str) -> Result<()> {
        self.db.delete_rss_feed(id)
    }

    pub async fn get_rules(&self) -> Result<Vec<RssRuleRecord>> {
        self.db.get_rss_rules()
    }

    pub async fn add_rule(
        &self,
        name: &str,
        pattern: &str,
        feed_id: Option<String>,
        category: &str,
        save_path: &str,
    ) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let record = RssRuleRecord {
            id: id.clone(),
            name: name.to_string(),
            pattern: pattern.to_string(),
            feed_id,
            category: category.to_string(),
            save_path: save_path.to_string(),
            last_matched_at: None,
        };
        self.db.insert_rss_rule(&record)?;
        Ok(id)
    }

    pub async fn delete_rule(&self, id: &str) -> Result<()> {
        self.db.delete_rss_rule(id)
    }

    pub async fn poll_all_feeds(&self) -> Result<Vec<RssMatchResult>> {
        let feeds = self.db.get_rss_feeds()?;
        let rules = self.db.get_rss_rules()?;
        let mut all_matches = Vec::new();

        for feed in &feeds {
            match self.poll_single_feed(feed).await {
                Ok(items) => {
                    for item in &items {
                        for rule in &rules {
                            if rule.feed_id.is_some() && rule.feed_id.as_deref() != Some(&feed.id) {
                                continue;
                            }
                            if Self::matches_pattern(&item.title, &rule.pattern) {
                                all_matches.push(RssMatchResult {
                                    feed_id: feed.id.clone(),
                                    rule_id: rule.id.clone(),
                                    title: item.title.clone(),
                                    magnet_link: item.magnet_link.clone(),
                                    size: item.size,
                                });
                            }
                        }
                    }
                    self.db.update_rss_feed_polled(&feed.id, None)?;
                }
                Err(_) => {
                    continue;
                }
            }
        }

        Ok(all_matches)
    }

    async fn poll_single_feed(&self, feed: &RssFeedRecord) -> Result<Vec<RssItem>> {
        let response = self.client.get(&feed.url).send().await?;
        let body = response.text().await?;
        let items = Self::parse_rss_xml(&body)?;
        Ok(items)
    }

    fn parse_rss_xml(xml: &str) -> Result<Vec<RssItem>> {
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut items = Vec::new();
        let mut in_item = false;
        let mut current_title = String::new();
        let mut current_link = String::new();
        let mut current_enclosure_url = String::new();
        let mut current_size: u64 = 0;
        let mut current_tag = String::new();
        let mut in_title = false;
        let mut in_link = false;

        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match name.as_str() {
                        "item" => {
                            in_item = true;
                            current_title.clear();
                            current_link.clear();
                            current_enclosure_url.clear();
                            current_size = 0;
                        }
                        "title" if in_item => {
                            in_title = true;
                            current_tag = "title".to_string();
                        }
                        "link" if in_item => {
                            in_link = true;
                            current_tag = "link".to_string();
                        }
                        "enclosure" if in_item => {
                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let val = String::from_utf8_lossy(&attr.value).to_string();
                                if key == "url" {
                                    current_enclosure_url = val;
                                } else if key == "length" {
                                    current_size = val.parse().unwrap_or(0);
                                }
                            }
                        }
                        "torrent:magnetURI" if in_item => {
                            current_tag = "magnet".to_string();
                        }
                        _ => {
                            current_tag = name;
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if in_title {
                        current_title.push_str(&text);
                    } else if in_link {
                        current_link.push_str(&text);
                    } else if current_tag == "magnet" && in_item {
                        current_link = text;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match name.as_str() {
                        "item" => {
                            let magnet = if current_link.starts_with("magnet:") {
                                current_link.clone()
                            } else if !current_enclosure_url.is_empty() && current_enclosure_url.starts_with("magnet:") {
                                current_enclosure_url.clone()
                            } else {
                                String::new()
                            };

                            if !magnet.is_empty() {
                                items.push(RssItem {
                                    title: current_title.clone(),
                                    magnet_link: magnet,
                                    size: current_size,
                                });
                            }
                            in_item = false;
                        }
                        "title" => in_title = false,
                        "link" => in_link = false,
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        Ok(items)
    }

    pub fn matches_pattern(title: &str, pattern: &str) -> bool {
        let wildcard_pattern = pattern
            .replace('.', "\\.")
            .replace('*', ".*")
            .replace('?', ".");
        if let Ok(re) = regex::Regex::new(&format!("(?i){}", wildcard_pattern)) {
            if re.is_match(title) {
                return true;
            }
        }
        if let Ok(re) = regex::Regex::new(pattern) {
            if re.is_match(title) {
                return true;
            }
        }
        title.to_lowercase().contains(&pattern.to_lowercase())
    }
}

struct RssItem {
    title: String,
    magnet_link: String,
    size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_pattern_regex() {
        assert!(RssManager::matches_pattern("Ubuntu 24.04 LTS", ".*Ubuntu.*"));
        assert!(RssManager::matches_pattern("Movie.2024.1080p.BluRay", ".*1080p.*BluRay.*"));
        assert!(!RssManager::matches_pattern("Movie.2024.720p", ".*1080p.*"));
    }

    #[test]
    fn test_matches_pattern_wildcard() {
        assert!(RssManager::matches_pattern("Ubuntu 24.04 LTS", "*Ubuntu*"));
        assert!(RssManager::matches_pattern("test.file.mkv", "test*mkv"));
        assert!(!RssManager::matches_pattern("other.file.mkv", "test*"));
    }

    #[test]
    fn test_parse_rss_xml() {
        let xml = r#"<?xml version="1.0"?>
        <rss version="2.0">
            <channel>
                <title>Test Feed</title>
                <item>
                    <title>Test Torrent</title>
                    <link>magnet:?xt=urn:btih:abc123</link>
                </item>
            </channel>
        </rss>"#;

        let items = RssManager::parse_rss_xml(xml).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Test Torrent");
        assert!(items[0].magnet_link.starts_with("magnet:"));
    }
}
