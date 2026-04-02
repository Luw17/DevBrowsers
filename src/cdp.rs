use serde::Deserialize;
use tungstenite::{connect, Message};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct CdpTarget {
    pub title: String,
    pub url: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    pub web_socket_debugger_url: Option<String>,
}

pub fn get_targets(port: u16) -> Result<Vec<CdpTarget>, String> {
    let url = format!("http://localhost:{}/json", port);
    let mut response = ureq::get(&url)
        .call()
        .map_err(|e| e.to_string())?;
        
    let body_str = response
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;
        
    let targets: Vec<CdpTarget> = serde_json::from_str(&body_str)
        .map_err(|e| e.to_string())?;
        
    Ok(targets)
}

pub fn send_cdp_command(ws_url: &str, method: &str, params: serde_json::Value) -> Result<String, String> {
    let (mut socket, _) = connect(ws_url)
        .map_err(|e| e.to_string())?;

    let payload = serde_json::json!({
        "id": 1,
        "method": method,
        "params": params
    });

    socket.send(Message::Text(payload.to_string().into()))
        .map_err(|e| e.to_string())?;

    if let Ok(msg) = socket.read() {
        return Ok(msg.into_text().unwrap_or_default().to_string());
    }

    Ok(String::new())
}