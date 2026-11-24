use std::{
    io::Write,
    process::{Command, Stdio},
};

use anyhow::Context;
use niri_ipc::{Action, Request, Response};

fn main() -> anyhow::Result<()> {
    let mut niri_socket = niri_ipc::socket::Socket::connect()?;

    let windows = match niri_socket.send(Request::Windows)? {
        Ok(Response::Windows(w)) if !w.is_empty() => w,
        _ => anyhow::bail!("no windows to show"),
    };

    let mut fuzzel = Command::new("fuzzel")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args(["--counter", "--dmenu", "--index", "-p", "󱂬 "])
        .spawn()?;

    let output = {
        let mut stdin = fuzzel.stdin.take().context("cannot access fuzzel stdin")?;

        for window in &windows {
            let app_id = window.app_id.as_deref().unwrap_or("");
            let title = window.title.as_deref().unwrap_or("");

            let entry = format!("{app_id} - {title} \0icon\x1f{app_id}\n");
            stdin.write_all(entry.as_bytes())?;
        }
        drop(stdin);
        fuzzel.wait_with_output()?
    };

    let selected = str::from_utf8(&output.stdout)
        .context("fuzzel returned invalid UTF-8")?
        .trim();

    if selected.is_empty() {
        return Ok(());
    }

    let window_index = selected.parse::<usize>()?;

    let selected = windows
        .get(window_index)
        .context("window index out of range")?;

    niri_socket
        .send(Request::Action(Action::FocusWindow { id: selected.id }))?
        .map_err(anyhow::Error::msg)?;

    Ok(())
}
