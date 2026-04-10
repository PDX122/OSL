use std::fs;
use std::process::Command;

pub fn send_message(webhook_url: &str, content: impl AsRef<str>) {
    let payload = format!(r#"{{"content":"{}"}}"#, escape_json(content.as_ref()));
    send_webhook(webhook_url, &payload);
}

pub fn send_embed(webhook_url: &str, title: impl AsRef<str>, description: impl AsRef<str>, color: i64) {
    let embed = format!(
        r#"{{"title":"{}","description":"{}","color":{}}}"#,
        escape_json(title.as_ref()),
        escape_json(description.as_ref()),
        color
    );
    let payload = format!(r#"{{"embeds":[{}]}}"#, embed);
    send_webhook(webhook_url, &payload);
}

pub fn send_as(webhook_url: &str, content: impl AsRef<str>, username: impl AsRef<str>, avatar_url: impl AsRef<str>) {
    let payload = format!(
        r#"{{"content":"{}","username":"{}","avatar_url":"{}"}}"#,
        escape_json(content.as_ref()),
        escape_json(username.as_ref()),
        escape_json(avatar_url.as_ref())
    );
    send_webhook(webhook_url, &payload);
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn send_webhook(webhook_url: &str, payload: &str) {
    let temp_file = "/tmp/oslc_discord_msg.json";
    
    if let Err(e) = fs::write(temp_file, payload) {
        eprintln!("Failed to write payload: {}", e);
        return;
    }
    
    let output = Command::new("curl")
        .args(&["-fsS", "-X", "POST"])
        .args(&["-H", "Content-Type: application/json"])
        .args(&["-d", &format!("@{}", temp_file)])
        .arg(webhook_url)
        .output();
    
    let _ = fs::remove_file(temp_file);
    
    match output {
        Ok(out) => {
            if !out.status.success() {
                eprintln!("Discord webhook error: {}", String::from_utf8_lossy(&out.stderr));
            }
        }
        Err(e) => {
            eprintln!("Failed to execute curl: {}", e);
        }
    }
}
