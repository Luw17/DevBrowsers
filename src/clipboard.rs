use std::process::{Command, Stdio};
use std::io::Write;
use eframe::egui;

pub fn get_text() -> String {
    if let Ok(out) = Command::new("wl-paste").output() {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout)
                .trim_end_matches('\n')
                .trim_end_matches('\r')
                .to_string();
        }
    }
    if let Ok(out) = Command::new("xclip").args(["-o", "-selection", "clipboard"]).output() {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout)
                .trim_end_matches('\n')
                .trim_end_matches('\r')
                .to_string();
        }
    }
    String::new()
}

pub fn set_text(text: &str) {
    if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
    } else if let Ok(mut child) = Command::new("xclip").args(["-selection", "clipboard"]).stdin(Stdio::piped()).spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
    }
}

pub fn handle_global_events(ctx: &egui::Context) {
    let mut paste_triggered = false;
    ctx.input_mut(|i| {
        if i.modifiers.command && i.key_pressed(egui::Key::V) {
            paste_triggered = true;
            i.events.retain(|e| !matches!(e, egui::Event::Paste(_)));
        }
    });

    if paste_triggered {
        let text = get_text();
        if !text.is_empty() {
            ctx.input_mut(|i| {
                i.events.push(egui::Event::Paste(text));
            });
        }
    }

    let copied_text = ctx.output_mut(|o| std::mem::take(&mut o.copied_text));
    if !copied_text.is_empty() {
        set_text(&copied_text);
    }
}