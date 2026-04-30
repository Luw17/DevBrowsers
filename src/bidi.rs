use tungstenite::{connect, Message};
use tungstenite::client::IntoClientRequest;

pub struct Target {
    pub id: String,
    pub url: String,
}

fn get_bidi_ws_url(port: u16) -> Result<String, String> {
    let version_url = format!("http://127.0.0.1:{}/json/version", port);
    let mut resp = ureq::get(&version_url)
        .header("Host", &format!("127.0.0.1:{}", port))
        .call()
        .map_err(|e| format!("HTTP erro: {}", e))?;
    
    let body_str = resp.body_mut().read_to_string().map_err(|e| e.to_string())?;
    let version_info: serde_json::Value = serde_json::from_str(&body_str).map_err(|e| e.to_string())?;
    
    let ws_url = version_info["webSocketDebuggerUrl"].as_str().ok_or("webSocketDebuggerUrl ausente")?;
    Ok(ws_url.replace("localhost", "127.0.0.1"))
}

pub fn get_targets(port: u16) -> Result<Vec<Target>, String> {
    let ws_url = get_bidi_ws_url(port)?;
    
    let mut request = ws_url.into_client_request().map_err(|e| e.to_string())?;
    request.headers_mut().insert("Host", format!("127.0.0.1:{}", port).parse().unwrap());

    let (mut socket, _) = connect(request).map_err(|e| format!("Handshake: {}", e))?;

    let payload = serde_json::json!({
        "id": 1,
        "method": "browsingContext.getTree",
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
                    if let Some(contexts) = parsed["result"]["contexts"].as_array() {
                        for ctx in contexts {
                            let id = ctx["context"].as_str().unwrap_or("").to_string();
                            let url = ctx["url"].as_str().unwrap_or("").to_string();
                            targets.push(Target { id: id.clone(), url: url.clone() });
                            
                            if let Some(children) = ctx["children"].as_array() {
                                for child in children {
                                    let cid = child["context"].as_str().unwrap_or("").to_string();
                                    let curl = child["url"].as_str().unwrap_or("").to_string();
                                    targets.push(Target { id: cid, url: curl });
                                }
                            }
                        }
                    }
                    return Ok(targets);
                }
            }
        }
    }
}

pub fn inject_js(port: u16, context_id: &str, js_code: &str) -> Result<String, String> {
    let ws_url = get_bidi_ws_url(port)?;
    
    let mut request = ws_url.into_client_request().map_err(|e| e.to_string())?;
    request.headers_mut().insert("Host", format!("127.0.0.1:{}", port).parse().unwrap());

    let (mut socket, _) = connect(request).map_err(|e| e.to_string())?;

    let payload = serde_json::json!({
        "id": 1,
        "method": "script.evaluate",
        "params": {
            "expression": js_code,
            "target": { "context": context_id },
            "awaitPromise": false
        }
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