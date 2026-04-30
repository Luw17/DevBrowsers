use serde::Deserialize;
use tungstenite::{connect, Message};
use tungstenite::client::IntoClientRequest;

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct Target {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub target_type: String,
    pub url: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    pub web_socket_debugger_url: Option<String>,
}

pub fn get_targets(browser_ws_url: &str, port: u16) -> Result<Vec<Target>, String> {
    if browser_ws_url.is_empty() { return Err("URL do WebSocket não capturada no lançamento".into()); }
    
    let host_port_str = browser_ws_url.replace("ws://", "").replace("wss://", "");
    let host_port = host_port_str.split('/').next().unwrap_or("127.0.0.1");

    let mut request = browser_ws_url.into_client_request().map_err(|e| e.to_string())?;
    request.headers_mut().insert("Host", host_port.parse().unwrap());

    let (mut socket, _) = connect(request).map_err(|e| e.to_string())?;

    let payload = serde_json::json!({
        "id": 1,
        "method": "Target.getTargets",
        "params": {}
    });

    socket.send(Message::Text(payload.to_string().into())).map_err(|e| e.to_string())?;

    loop {
        let msg = socket.read().map_err(|e| e.to_string())?;
        if let Message::Text(text) = msg {
            let text_str = text.to_string();
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text_str) {
                if parsed.get("id").and_then(|i| i.as_u64()) == Some(1) {
                    let mut targets = Vec::new();
                    if let Some(infos) = parsed["result"]["targetInfos"].as_array() {
                        for info in infos {
                            let id = info["targetId"].as_str().unwrap_or("").to_string();
                            let t_type = info["type"].as_str().unwrap_or("").to_string();
                            let url = info["url"].as_str().unwrap_or("").to_string();
                            let title = info["title"].as_str().unwrap_or("").to_string();
                            let ws_url = format!("ws://127.0.0.1:{}/devtools/page/{}", port, id);
                            
                            targets.push(Target {
                                id,
                                title,
                                target_type: t_type,
                                url,
                                web_socket_debugger_url: Some(ws_url),
                            });
                        }
                    }
                    return Ok(targets);
                }
            }
        }
    }
}

pub fn send_cdp_command(ws_url: &str, method: &str, params: serde_json::Value) -> Result<String, String> {
    let safe_ws = ws_url.replace("localhost", "127.0.0.1");
    
    let host_port_str = safe_ws.replace("ws://", "").replace("wss://", "");
    let host_port = host_port_str.split('/').next().unwrap_or("127.0.0.1");
    
    let mut request = safe_ws.into_client_request().map_err(|e| e.to_string())?;
    request.headers_mut().insert("Host", host_port.parse().unwrap());

    let (mut socket, _) = connect(request).map_err(|e| e.to_string())?;

    let payload = serde_json::json!({
        "id": 1,
        "method": method,
        "params": params
    });

    socket.send(Message::Text(payload.to_string().into())).map_err(|e| e.to_string())?;

    loop {
        let msg = socket.read().map_err(|e| e.to_string())?;
        if let Message::Text(text) = msg {
            let text_str = text.to_string();
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text_str) {
                if parsed.get("id").and_then(|i| i.as_u64()) == Some(1) {
                    return Ok(text_str);
                }
            }
        }
    }
}