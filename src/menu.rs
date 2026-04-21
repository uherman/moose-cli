use anyhow::{anyhow, Result};
use inquire::Select;

use crate::{api, player, scrape};

const LIVE: &str = "Live just nu (huvudkamera)";
const EXTRA: &str = "Extrakameror (live)";
const CHAPTERS: &str = "Hoppa till klipp…";
const EARLIER: &str = "Tidigare dygn… (ej implementerat)";
const QUIT: &str = "Avsluta";

pub fn run(detach: bool) -> Result<()> {
    let choice = Select::new(
        "Vad vill du titta på?",
        vec![LIVE, EXTRA, CHAPTERS, EARLIER, QUIT],
    )
    .prompt()?;

    match choice {
        LIVE => watch_page(scrape::LIVE_URL, 0, detach),
        EXTRA => watch_page(scrape::EXTRA_URL, 0, detach),
        CHAPTERS => chapter_menu(detach),
        EARLIER => {
            eprintln!("Inte implementerat i v1. Skrapa programsidan manuellt och kör `moose-cli play <videoId>`.");
            Ok(())
        }
        QUIT => Ok(()),
        other => Err(anyhow!("okänt menyval: {other}")),
    }
}

fn watch_page(page_url: &str, start_sec: u64, detach: bool) -> Result<()> {
    let video_id = scrape::video_id_from_page(page_url)?;
    let info = api::fetch(&video_id)?;
    let hls = info.hls_url()?;
    player::play(hls, start_sec, detach)
}

fn chapter_menu(detach: bool) -> Result<()> {
    let video_id = scrape::video_id_from_page(scrape::LIVE_URL)?;
    let mut info = api::fetch(&video_id)?;

    // Kapitel finns normalt i API:et (enligt plan.md), men SVT levererar dem
    // just nu bara som HTML-länkar på episod-sidan. Falla tillbaka till
    // HTML-scrape om API-listan är tom.
    let mut chapters = std::mem::take(&mut info.chapters);
    if chapters.is_empty() {
        chapters = scrape::chapters_for_video(&video_id)?;
    }

    if chapters.is_empty() {
        eprintln!(
            "Inga kapitel hittades för dagens avsnitt ({video_id}). \
             De läggs till löpande under dygnet — prova igen senare."
        );
        return Ok(());
    }

    let labels: Vec<String> = chapters
        .iter()
        .map(|c| format!("{}  —  {}", format_timestamp(c.position), c.title))
        .collect();

    let picked = Select::new("Välj klipp:", labels.clone()).prompt()?;
    let idx = labels
        .iter()
        .position(|l| l == &picked)
        .ok_or_else(|| anyhow!("kunde inte matcha valt klipp"))?;
    let chapter = &chapters[idx];

    let hls = info.hls_url()?;
    player::play(hls, chapter.position, detach)
}

fn format_timestamp(total_seconds: u64) -> String {
    let h = total_seconds / 3600;
    let m = (total_seconds % 3600) / 60;
    let s = total_seconds % 60;
    if h > 0 {
        format!("{h:02}:{m:02}:{s:02}")
    } else {
        format!("{m:02}:{s:02}")
    }
}
