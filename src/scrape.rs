use anyhow::{anyhow, Context, Result};
use regex::Regex;

use crate::api::Chapter;

pub const LIVE_URL: &str = "https://www.svtplay.se/den-stora-algvandringen";
pub const EXTRA_URL: &str = "https://www.svtplay.se/den-stora-algvandringen-extrakameror";

fn fetch_html(url: &str) -> Result<String> {
    reqwest::blocking::get(url)
        .with_context(|| format!("nätverksfel vid hämtning av {url}"))?
        .text()
        .with_context(|| format!("kunde inte läsa HTML-body från {url}"))
}

pub fn video_id_from_page(url: &str) -> Result<String> {
    let html = fetch_html(url)?;

    // SVT bäddar det här fältet både som ren JSON (`"svtId":"..."`) och som
    // sträng-escape:ad JSON (`\"svtId\":\"...\"`) inuti __NEXT_DATA__.
    let re = Regex::new(r#"\\?"svtId\\?":\\?"([A-Za-z0-9_-]+)"#).expect("statisk regex");
    if let Some(caps) = re.captures(&html) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }

    let preview: String = html.chars().take(500).collect();
    Err(anyhow!(
        "kunde inte hitta svtId på {url}. Första 500 tecknen av svaret:\n{preview}"
    ))
}

/// Skrapa kapitel från episod-sidan `https://www.svtplay.se/video/{videoId}`.
/// Kapitlen ligger inte i api.svt.se-svaret utan som HTML-länkar på den sidan.
pub fn chapters_for_video(video_id: &str) -> Result<Vec<Chapter>> {
    let url = format!("https://www.svtplay.se/video/{video_id}");
    let html = fetch_html(&url)?;

    let re = Regex::new(r#"position=(\d+)"[^>]*><p[^>]*>([^<]+)</p>"#).expect("statisk regex");
    let mut chapters = Vec::new();
    for caps in re.captures_iter(&html) {
        let position: u64 = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
        let title = html_decode(caps.get(2).unwrap().as_str()).trim().to_string();
        if !title.is_empty() && position > 0 {
            chapters.push(Chapter { position, title });
        }
    }
    Ok(chapters)
}

fn html_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}
