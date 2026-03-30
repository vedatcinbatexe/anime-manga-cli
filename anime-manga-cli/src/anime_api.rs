use serde::Deserialize;

const SERVER: &str = "http://localhost:8080";

#[derive(Debug, Deserialize, Clone)]
pub struct Anime {
    pub id: String,
    pub name: String,
    pub episodes: u64,
}

impl Anime {
    pub fn title(&self) -> &str {
        &self.name
    }

    pub fn info(&self) -> String {
        format!("{} eps", self.episodes)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Episode {
    pub number: u64,
    pub title: Option<String>,
    pub episode_id: String,
    pub filler: Option<bool>,
}

impl Episode {
    pub fn label(&self) -> String {
        let title = self.title.as_deref().unwrap_or("No title");
        format!("Ep.{:<4} {}", self.number, title)
    }
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<Anime>,
}

#[derive(Debug, Deserialize)]
struct EpisodesResponse {
    episodes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct StreamResponse {
    url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct StreamLink {
    pub quality: String,
    pub url: String,
}

#[derive(Debug, serde::Deserialize)]
struct LinksResponse {
    links: Vec<StreamLink>,
}

pub async fn fetch_links(
    client: &reqwest::Client,
    show_id: &str,
    episode: &str,
) -> anyhow::Result<Vec<StreamLink>> {
    let url = format!("{}/links?id={}&episode={}", SERVER, show_id, episode);

    let resp = client
        .get(&url)
        .send()
        .await?
        .json::<LinksResponse>()
        .await?;

    Ok(resp.links)
}

pub async fn search_anime(client: &reqwest::Client, query: &str) -> anyhow::Result<Vec<Anime>> {
    let url = format!("{}/search?q={}", SERVER, urlencoding::encode(query));

    let resp = client
        .get(&url)
        .send()
        .await?
        .json::<SearchResponse>()
        .await?;

    Ok(resp.results)
}

pub async fn fetch_episodes(
    client: &reqwest::Client,
    anime: &Anime,
) -> anyhow::Result<Vec<Episode>> {
    let url = format!("{}/episodes?id={}", SERVER, anime.id);

    let resp = client
        .get(&url)
        .send()
        .await?
        .json::<EpisodesResponse>()
        .await?;

    let episodes = resp
        .episodes
        .into_iter()
        .enumerate()
        .map(|(i, ep_num)| Episode {
            number: ep_num.parse().unwrap_or((i + 1) as u64),
            title: Some(format!("Episode {}", ep_num)),
            episode_id: ep_num,
            filler: Some(false),
        })
        .collect();

    Ok(episodes)
}

pub async fn fetch_stream_url(
    client: &reqwest::Client,
    show_id: &str,
    episode: &str,
) -> anyhow::Result<String> {
    let url = format!("{}/stream?id={}&episode={}", SERVER, show_id, episode);

    let resp = client
        .get(&url)
        .send()
        .await?
        .json::<StreamResponse>()
        .await?;

    Ok(resp.url)
}
