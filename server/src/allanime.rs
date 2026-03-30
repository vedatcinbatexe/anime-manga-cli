use anyhow::Result;
use serde::{Deserialize, Serialize};

const API: &str = "https://api.allanime.day/api";
const REFERER: &str = "https://allmanga.to";
const AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/121.0";

// ── Search ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnimeResult {
    pub id: String,
    pub name: String,
    pub episodes: u64,
}

#[derive(Debug, Deserialize)]
struct SearchGqlResponse {
    data: SearchData,
}

#[derive(Debug, Deserialize)]
struct SearchData {
    shows: ShowsEdges,
}

#[derive(Debug, Deserialize)]
struct ShowsEdges {
    edges: Vec<ShowEdge>,
}

#[derive(Debug, Deserialize)]
struct ShowEdge {
    #[serde(rename = "_id")]
    id: String,
    name: String,
    #[serde(rename = "availableEpisodes")]
    available_episodes: AvailableEpisodes,
}

#[derive(Debug, Deserialize)]
struct AvailableEpisodes {
    sub: Option<u64>,
    dub: Option<u64>,
}

pub async fn search(client: &reqwest::Client, query: &str) -> Result<Vec<AnimeResult>> {
    let gql = r#"query($search: SearchInput $limit: Int $page: Int $translationType: VaildTranslationTypeEnumType $countryOrigin: VaildCountryOriginEnumType) { shows(search: $search limit: $limit page: $page translationType: $translationType countryOrigin: $countryOrigin) { edges { _id name availableEpisodes __typename } }}"#;

    let variables = serde_json::json!({
        "search": {
            "allowAdult": false,
            "allowUnknown": false,
            "query": query
        },
        "limit": 40,
        "page": 1,
        "translationType": "sub",
        "countryOrigin": "ALL"
    });

    let raw = client
        .get(API)
        .header("Referer", REFERER)
        .header("User-Agent", AGENT)
        .query(&[
            ("variables", variables.to_string()),
            ("query", gql.to_string()),
        ])
        .send()
        .await?
        .text()
        .await?;

    println!("RAW SEARCH: {}", &raw[..raw.len().min(500)]);

    let response: SearchGqlResponse = serde_json::from_str(&raw)
        .map_err(|e| anyhow::anyhow!("Parse error: {}\nRaw: {}", e, &raw[..raw.len().min(200)]))?;

    let results = response
        .data
        .shows
        .edges
        .into_iter()
        .map(|e| AnimeResult {
            id: e.id,
            name: e.name,
            episodes: e.available_episodes.sub.unwrap_or(0),
        })
        .collect();

    Ok(results)
}

// ── Episodes ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct EpisodesGqlResponse {
    data: EpisodesData,
}

#[derive(Debug, Deserialize)]
struct EpisodesData {
    show: ShowEpisodes,
}

#[derive(Debug, Deserialize)]
struct ShowEpisodes {
    #[serde(rename = "availableEpisodesDetail")]
    available_episodes_detail: EpisodesDetail,
}

#[derive(Debug, Deserialize)]
struct EpisodesDetail {
    sub: Option<Vec<String>>,
}

pub async fn episodes(client: &reqwest::Client, show_id: &str) -> Result<Vec<String>> {
    let gql = r#"query ($showId: String!) { show( _id: $showId ) { _id availableEpisodesDetail }}"#;

    let variables = serde_json::json!({ "showId": show_id });

    let response = client
        .get(API)
        .header("Referer", REFERER)
        .header("User-Agent", AGENT)
        .query(&[
            ("variables", variables.to_string()),
            ("query", gql.to_string()),
        ])
        .send()
        .await?
        .json::<EpisodesGqlResponse>()
        .await?;

    let mut eps = response
        .data
        .show
        .available_episodes_detail
        .sub
        .unwrap_or_default();
    eps.sort_by(|a, b| {
        let a: f64 = a.parse().unwrap_or(0.0);
        let b: f64 = b.parse().unwrap_or(0.0);
        a.partial_cmp(&b).unwrap()
    });

    Ok(eps)
}

// ── Stream URL ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct EpisodeGqlResponse {
    data: EpisodeData,
}

#[derive(Debug, Deserialize)]
struct EpisodeData {
    episode: EpisodeEmbed,
}

#[derive(Debug, Deserialize)]
struct EpisodeEmbed {
    #[serde(rename = "sourceUrls")]
    source_urls: Vec<SourceUrl>,
}

#[derive(Debug, Deserialize)]
struct SourceUrl {
    #[serde(rename = "sourceUrl")]
    source_url: String,
    #[serde(rename = "sourceName")]
    source_name: String,
}

pub async fn stream_links(
    client: &reqwest::Client,
    show_id: &str,
    episode: &str,
) -> Result<Vec<StreamLink>> {
    let gql = r#"query ($showId: String!, $translationType: VaildTranslationTypeEnumType!, $episodeString: String!) { episode( showId: $showId translationType: $translationType episodeString: $episodeString ) { episodeString sourceUrls }}"#;

    let variables = serde_json::json!({
        "showId": show_id,
        "translationType": "sub",
        "episodeString": episode
    });

    let raw = client
        .get(API)
        .header("Referer", REFERER)
        .header("User-Agent", AGENT)
        .query(&[
            ("variables", variables.to_string()),
            ("query", gql.to_string()),
        ])
        .send()
        .await?
        .text()
        .await?;

    let decoded_raw = raw.replace("\\u002F", "/").replace("\\", "");
    let response: EpisodeGqlResponse =
        serde_json::from_str(&decoded_raw).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let source = response
        .data
        .episode
        .source_urls
        .iter()
        .find(|s| s.source_name == "Default")
        .or_else(|| {
            response
                .data
                .episode
                .source_urls
                .iter()
                .find(|s| s.source_name == "Luf-Mp4")
        })
        .or_else(|| response.data.episode.source_urls.first())
        .ok_or_else(|| anyhow::anyhow!("No sources found"))?;

    let encoded = source.source_url.trim_start_matches("--").to_string();
    let decoded = decode_url(&encoded);

    let clock_url = if decoded.starts_with("/") {
        format!("https://allanime.day{}", decoded).replace("/clock", "/clock.json")
    } else {
        decoded
    };

    let clock_resp = client
        .get(&clock_url)
        .header("Referer", REFERER)
        .header("User-Agent", AGENT)
        .send()
        .await?
        .text()
        .await?;

    #[derive(Deserialize)]
    struct ClockResponse {
        links: Vec<ClockLink>,
    }
    #[derive(Deserialize)]
    struct ClockLink {
        link: String,
    }

    let clock: ClockResponse =
        serde_json::from_str(&clock_resp).map_err(|e| anyhow::anyhow!("Clock parse: {}", e))?;

    if clock.links.is_empty() {
        return Err(anyhow::anyhow!("No links found"));
    }

    let master_url = clock.links[0].link.clone();

    // Fetch the m3u8 playlist to extract quality variants
    let m3u8 = client
        .get(&master_url)
        .header("Referer", REFERER)
        .header("User-Agent", AGENT)
        .send()
        .await?
        .text()
        .await?;

    let base_url = master_url.rsplit_once('/').map(|(b, _)| b).unwrap_or("");

    let mut links = Vec::new();

    // Parse m3u8 for quality variants
    let lines: Vec<&str> = m3u8.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("#EXT-X-STREAM-INF") {
            // Extract resolution
            let resolution = line
                .split(',')
                .find(|p| p.contains("RESOLUTION="))
                .and_then(|p| p.split('=').nth(1))
                .unwrap_or("unknown")
                .to_string();

            // Next line is the URL
            if let Some(url_line) = lines.get(i + 1) {
                let url = if url_line.starts_with("http") {
                    url_line.to_string()
                } else {
                    format!("{}/{}", base_url, url_line)
                };

                // Extract height from resolution (e.g. "1280x720" -> "720p")
                let quality = resolution
                    .split('x')
                    .nth(1)
                    .map(|h| format!("{}p", h))
                    .unwrap_or_else(|| resolution.clone());

                links.push(StreamLink { quality, url });
            }
        }
    }

    // Fallback — if no variants found, return master as "auto"
    if links.is_empty() {
        links.push(StreamLink {
            quality: "auto".to_string(),
            url: master_url,
        });
    }

    // Sort best quality first
    links.sort_by(|a, b| {
        let a_h: u32 = a.quality.trim_end_matches('p').parse().unwrap_or(0);
        let b_h: u32 = b.quality.trim_end_matches('p').parse().unwrap_or(0);
        b_h.cmp(&a_h)
    });

    Ok(links)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamLink {
    pub quality: String,
    pub url: String,
}

fn decode_url(encoded: &str) -> String {
    let mut result = String::new();
    let mut i = 0;
    let bytes = encoded.as_bytes();

    while i + 1 < bytes.len() {
        let pair = &encoded[i..i + 2];
        let ch = match pair {
            "17" => '/',
            "59" => 'a',
            "5a" => 'b',
            "5b" => 'c',
            "5c" => 'd',
            "5d" => 'e',
            "5e" => 'f',
            "5f" => 'g',
            "50" => 'h',
            "51" => 'i',
            "52" => 'j',
            "53" => 'k',
            "54" => 'l',
            "55" => 'm',
            "56" => 'n',
            "57" => 'o',
            "48" => 'p',
            "49" => 'q',
            "4a" => 'r',
            "4b" => 's',
            "4c" => 't',
            "4d" => 'u',
            "4e" => 'v',
            "4f" => 'w',
            "40" => 'x',
            "41" => 'y',
            "42" => 'z',
            "79" => 'A',
            "7a" => 'B',
            "7b" => 'C',
            "7c" => 'D',
            "7d" => 'E',
            "7e" => 'F',
            "7f" => 'G',
            "70" => 'H',
            "71" => 'I',
            "72" => 'J',
            "73" => 'K',
            "74" => 'L',
            "75" => 'M',
            "76" => 'N',
            "77" => 'O',
            "68" => 'P',
            "69" => 'Q',
            "6a" => 'R',
            "6b" => 'S',
            "6c" => 'T',
            "6d" => 'U',
            "6e" => 'V',
            "6f" => 'W',
            "60" => 'X',
            "61" => 'Y',
            "62" => 'Z',
            "08" => '0',
            "09" => '1',
            "0a" => '2',
            "0b" => '3',
            "0c" => '4',
            "0d" => '5',
            "0e" => '6',
            "0f" => '7',
            "00" => '8',
            "01" => '9',
            "15" => '-',
            "16" => '.',
            "67" => '_',
            "46" => '~',
            "02" => ':',
            "07" => '?',
            "1b" => '#',
            "63" => '[',
            "65" => ']',
            "78" => '@',
            "19" => '!',
            "1c" => '$',
            "1e" => '&',
            "10" => '(',
            "11" => ')',
            "12" => '*',
            "13" => '+',
            "14" => ',',
            "03" => ';',
            "05" => '=',
            "1d" => '%',
            _ => '\0',
        };
        if ch != '\0' {
            result.push(ch);
        }
        i += 2;
    }

    result
}
