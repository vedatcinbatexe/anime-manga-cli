use serde::Deserialize;
use std::{collections::HashMap, fmt::format};

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub data: Vec<Manga>,
}

#[derive(Debug, Deserialize)]
pub struct ChapterResponse {
    pub data: Vec<Chapter>,
}

#[derive(Debug, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub attributes: ChapterAttributes,
}

#[derive(Debug, Deserialize)]
pub struct ChapterAttributes {
    pub chapter: Option<String>,
    pub title: Option<String>,
    pub pages: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct Manga {
    pub id: String,
    pub attributes: MangaAttributes,
}

#[derive(Debug, Deserialize)]
pub struct MangaAttributes {
    pub title: std::collections::HashMap<String, String>,
    pub description: Option<std::collections::HashMap<String, String>>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PageResponse {
    pub chapter: PageChapter,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct PageChapter {
    pub hash: String,
    pub data: Vec<String>,
}

impl Manga {
    pub fn title(&self) -> &str {
        self.attributes
            .title
            .get("en")
            .and_then(|s| Some(s.as_str()))
            .unwrap_or("Unknown Title")
    }

    pub fn description(&self) -> &str {
        self.attributes
            .description
            .as_ref()
            .and_then(|d| d.get("en"))
            .map(|s| s.as_str())
            .unwrap_or("No description")
    }
}

impl Chapter {
    pub fn label(&self) -> String {
        let num = self.attributes.chapter.as_deref().unwrap_or("?");
        let title = self.attributes.title.as_deref().unwrap_or("No title");
        let pages = self.attributes.pages.unwrap_or(0);
        format!("Ch.{} — {} ({} pages)", num, title, pages)
    }
}

pub async fn search_manga(client: &reqwest::Client, query: &str) -> anyhow::Result<Vec<Manga>> {
    let url = format!("https://api.mangadex.org/manga?title={}&limit=10", query);

    let response = client
        .get(&url)
        .send()
        .await?
        .json::<SearchResponse>()
        .await?;

    Ok(response.data)
}

pub async fn fetch_chapters(
    client: &reqwest::Client,
    manga_id: &str,
) -> anyhow::Result<Vec<Chapter>> {
    let url = format!(
        "https://api.mangadex.org/manga/{}/feed?translatedLanguage[]=en&order[chapter]=asc&limit=50",
        manga_id
    );

    let response = client
        .get(&url)
        .send()
        .await?
        .json::<ChapterResponse>()
        .await?;

    Ok(response.data)
}

pub async fn fetch_pages(
    client: &reqwest::Client,
    chapter_id: &str,
) -> anyhow::Result<Vec<String>> {
    let url = format!("https://api.mangadex.org/at-home/server/{}", chapter_id);

    let response = client
        .get(&url)
        .send()
        .await?
        .json::<PageResponse>()
        .await?;

    // Build full image URLs
    let urls = response
        .chapter
        .data
        .iter()
        .map(|filename| {
            format!(
                "{}/data/{}/{}",
                response.base_url, response.chapter.hash, filename
            )
        })
        .collect();

    Ok(urls)
}

pub async fn download_page_bytes(client: &reqwest::Client, url: &str) -> anyhow::Result<Vec<u8>> {
    let bytes = client.get(url).send().await?.bytes().await?;

    Ok(bytes.to_vec())
}
