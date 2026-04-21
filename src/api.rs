use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

const API_BASE: &str = "https://api.svt.se/video";

#[derive(Debug, Deserialize)]
pub struct VideoInfo {
    #[serde(rename = "videoReferences", default)]
    pub video_references: Vec<VideoReference>,
    #[serde(default)]
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Deserialize)]
pub struct VideoReference {
    pub format: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Chapter {
    pub position: u64,
    pub title: String,
}

fn fetch_body(video_id: &str) -> Result<String> {
    let url = format!("{API_BASE}/{video_id}");
    let resp = reqwest::blocking::get(&url)
        .with_context(|| format!("nätverksfel vid anrop mot {url}"))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(anyhow!("SVT-API svarade med status {status} för videoId {video_id}"));
    }
    resp.text()
        .with_context(|| format!("kunde inte läsa body från {url}"))
}

pub fn fetch(video_id: &str) -> Result<VideoInfo> {
    let body = fetch_body(video_id)?;
    serde_json::from_str::<VideoInfo>(&body)
        .with_context(|| format!("kunde inte tolka JSON-svar för videoId {video_id}"))
}

pub fn fetch_raw(video_id: &str) -> Result<serde_json::Value> {
    let body = fetch_body(video_id)?;
    serde_json::from_str::<serde_json::Value>(&body)
        .with_context(|| format!("kunde inte tolka JSON-svar för videoId {video_id}"))
}

impl VideoInfo {
    pub fn hls_url(&self) -> Result<&str> {
        self.video_references
            .iter()
            .find(|r| r.format == "hls")
            .map(|r| r.url.as_str())
            .ok_or_else(|| {
                let formats: Vec<&str> = self
                    .video_references
                    .iter()
                    .map(|r| r.format.as_str())
                    .collect();
                anyhow!(
                    "inget HLS-format i svaret (tillgängliga format: {})",
                    if formats.is_empty() {
                        "inga".to_string()
                    } else {
                        formats.join(", ")
                    }
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hls_url_picks_hls_entry() {
        let json = r#"{
            "videoReferences": [
                {"format": "dash", "url": "https://example.com/manifest.mpd"},
                {"format": "hls",  "url": "https://example.com/master.m3u8"},
                {"format": "hls-cmaf-live", "url": "https://example.com/other.m3u8"}
            ]
        }"#;
        let info: VideoInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.hls_url().unwrap(), "https://example.com/master.m3u8");
        assert!(info.chapters.is_empty());
    }

    #[test]
    fn hls_url_reports_missing_formats() {
        let json = r#"{"videoReferences": [{"format": "dash", "url": "x"}]}"#;
        let info: VideoInfo = serde_json::from_str(json).unwrap();
        let err = info.hls_url().unwrap_err().to_string();
        assert!(err.contains("dash"), "felmeddelandet ska lista tillgängliga format: {err}");
    }
}
