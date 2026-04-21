# moose-cli

Liten Rust-CLI för att titta på SVT:s livesändning av *Den stora älgvandringen*
— Hämtar stream-URL direkt från SVT:s
eget video-API och startar `mpv`.

## Krav

- [`mpv`](https://mpv.io/) i `$PATH`.

## Installation

Från crates.io:

```sh
cargo install moose-cli
```

Eller bygg från källan:

```sh
git clone https://github.com/uherman/moose-cli.git
cd moose-cli
cargo install --path .
```

## Användning

### Interaktiv meny

Utan argument dyker en meny upp med Live / Extrakameror / Hoppa till klipp /
Avsluta:

```sh
moose-cli
```

### Subkommandon

```sh
moose-cli live                          # huvudkameran (live)
moose-cli extra                         # extrakamerorna (live)
moose-cli play <videoId>                # ett specifikt avsnitt
moose-cli play <videoId> --start 5662   # hoppa in 5662 s i avsnittet
moose-cli url <videoId>                 # skriv HLS-URL till stdout (ingen mpv)
moose-cli debug <videoId>               # dumpa rå JSON från api.svt.se
```

Ett `videoId` är den korta strängen i SVT Play-URL:en, t.ex. `KXv169r` i
`https://www.svtplay.se/video/KXv169r/...`. För `live` och `extra` skrapas
id:t automatiskt från programsidan.

### Detach-beteende

Default startar mpv **frånkopplat** från terminalen — du kan stänga fönstret
och mpv fortsätter. Lägg till `-a` / `--attach` för att istället köra mpv i
förgrund (bra när du vill se ffmpeg-loggarna vid felsökning):

```sh
moose-cli --attach live
```

## Att hoppa till klipp

Välj "Hoppa till klipp…" i menyn — den listar dagens höjdpunkter (t.ex.
"Isflaksbuss!", "En älg i utkiken") och `mpv` startar med `--start=<sek>`.

Eftersom SVT:s live-stream är en seekbar `EVENT`-playlist men ffmpeg antar
att det är en vanlig sliding live, lägger CLI:t automatiskt till
`--demuxer-lavf-o-add=live_start_index=0` när ett start-offset används.
Det betyder att seek:en faktiskt landar rätt istället för att spamma
"m3u8 list sequence may have been wrapped".


## Projektstruktur

```
src/
  main.rs     — clap-dispatch, subkommandon
  api.rs      — api.svt.se/video/{id}-klient, HLS-väljare
  scrape.rs   — videoId från programsida + kapitel-HTML-scrape
  player.rs   — mpv-spawn (attach- och detach-lägen)
  menu.rs     — inquire-baserad interaktiv meny
```
