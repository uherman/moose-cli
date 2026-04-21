use anyhow::{anyhow, Context, Result};
use std::io::ErrorKind;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

pub fn play(hls_url: &str, start_sec: u64, detach: bool) -> Result<()> {
    let mut cmd = Command::new("mpv");
    if start_sec > 0 {
        cmd.arg(format!("--start={start_sec}"));
        // SVT:s live-HLS är en EVENT-playlist (hela dygnet finns tillgängligt),
        // men ffmpeg tolkar den som vanlig live och börjar vid kanten — det gör
        // `--start` oanvändbart. Detta tvingar ffmpeg att läsa från segment 0 så
        // att seek-offsetten faktiskt landar rätt.
        cmd.arg("--demuxer-lavf-o-add=live_start_index=0");
    }
    cmd.arg(hls_url);

    if detach {
        return spawn_detached(&mut cmd);
    }

    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Err(anyhow!(
                "mpv verkar inte vara installerat. Installera det med t.ex. `sudo pacman -S mpv` eller `brew install mpv`."
            ));
        }
        Err(e) => {
            return Err(e).context("kunde inte starta mpv");
        }
    };

    if !status.success() {
        return Err(anyhow!("mpv avslutades med fel: {status}"));
    }
    Ok(())
}

fn spawn_detached(cmd: &mut Command) -> Result<()> {
    // Kör mpv i ny session + egen process-grupp, utan ärvd stdio, så att
    // terminalens SIGHUP inte når den när fönstret stängs.
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    unsafe {
        cmd.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }

    match cmd.spawn() {
        Ok(child) => {
            eprintln!("mpv startad i bakgrunden (pid {}).", child.id());
            Ok(())
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Err(anyhow!(
            "mpv verkar inte vara installerat. Installera det med t.ex. `sudo pacman -S mpv` eller `brew install mpv`."
        )),
        Err(e) => Err(e).context("kunde inte starta mpv"),
    }
}
