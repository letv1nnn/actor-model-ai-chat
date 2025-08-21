use actix::prelude::*;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

// Actor definition and message

// message, that carries session_id, model amd prompt
#[derive(Message)]
#[rtype(result = "Result<String, String>")]
pub struct AskModel {
    pub session_id: String,
    pub model: String,
    pub prompt: String,
}

// Actor that handles that message by trying Ollama. If that's down, you still see a reply (echo).
pub struct ModelActor {
    pub base_url: String,
    pub client: reqwest::Client,
}

impl Actor for ModelActor {
    type Context = Context<Self>;
}

// implement the handler
impl Handler<AskModel> for ModelActor {
    type Result = ResponseFuture<Result<String, String>>;

    fn handle(&mut self, msg: AskModel, _ctx: &mut Self::Context) -> Self::Result {
        let client = self.client.clone();
        let base = self.base_url.clone();

        Box::pin(async move {
            #[derive(Serialize)]
            struct OllamaMessage {
                role: String,
                content: String,
            }
            #[derive(Serialize)]
            struct OllamaRequest {
                model: String,
                messages: Vec<OllamaMessage>,
                stream: bool
            }

            let url = format!("{}/api/chat", base);
            let req = OllamaRequest {
                model: msg.model.clone(),
                messages: vec![OllamaMessage { role: "user".into(), content: msg.prompt.clone() }],
                stream: false,
            };

            let resp = client.post(url).json(&req).send().await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    let v: serde_json::Value = r.json().await.map_err(|e| e.to_string())?;
                    if let Some(s) = v.get("message").and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
                        Ok(s.to_string())
                    } else if let Some(s) = v.get("reply").and_then(|c| c.as_str()) {
                        Ok(s.to_string())
                    } else {
                        Ok(v.to_string())
                    }
                },
                _ => Ok(format!("(dev echo) session={} model={} -> {}", msg.session_id, msg.model, msg.prompt))
            }
        })
    }
}

// http endpoint
// frontend posts json: { "sessionId": "default", "model": "mistral", "message": "..." }
// need to parse that and send an AskModel to the actor
#[derive(Serialize, Deserialize)]
pub struct ChatIn {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub model: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChatOut {
    pub reply: String,
}

pub async fn chat_endpoint(payload: web::Json<ChatIn>, model_addr: web::Data<Addr<ModelActor>>) -> impl Responder {
    let msg = AskModel {
        session_id: payload.session_id.clone(),
        model: payload.model.clone(),
        prompt: payload.message.clone(),
    };

    match model_addr.send(msg).await {
        Ok(Ok(reply)) => HttpResponse::Ok().json(ChatOut { reply }),
        Ok(Err(err))  => HttpResponse::InternalServerError().json(ChatOut { reply: format!("error: {}", err) }),
        Err(mailbox)  => HttpResponse::InternalServerError().json(ChatOut { reply: format!("mailbox error: {}", mailbox) }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        MockServer, Mock, ResponseTemplate,
        matchers::{method, path}
    };

    #[actix::test]
    async fn test_ask_model_success() {
        // Mock server for Ollama
        let mock_server = MockServer::start().await;
        
        // Setup mock response
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": {
                    "content": "Hello, this is a test response"
                }
            })))
            .mount(&mock_server)
            .await;

        // Create actor with mock URL
        let actor = ModelActor {
            base_url: mock_server.uri(),
            client: reqwest::Client::new(),
        };
        let addr = actor.start();

        // Test message
        let msg = AskModel {
            session_id: "test-session".to_string(),
            model: "test-model".to_string(),
            prompt: "Hello".to_string(),
        };

        // Send message and get response
        let result = addr.send(msg).await.unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, this is a test response");
    }

    #[actix::test]
    async fn test_ask_model_fallback() {
        // Create actor with invalid URL to trigger fallback
        let actor = ModelActor {
            base_url: "http://invalid-url-that-wont-work:9999".to_string(),
            client: reqwest::Client::new(),
        };
        let addr = actor.start();

        let msg = AskModel {
            session_id: "test-session".to_string(),
            model: "test-model".to_string(),
            prompt: "Hello".to_string(),
        };

        let result = addr.send(msg).await.unwrap();
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("(dev echo)"));
        assert!(response.contains("test-session"));
        assert!(response.contains("test-model"));
        assert!(response.contains("Hello"));
    }

    #[test]
    fn test_chat_in_deserialization() {
        let json = r#"{
            "sessionId": "test-session",
            "model": "mistral",
            "message": "Hello there"
        }"#;

        let chat_in: ChatIn = serde_json::from_str(json).unwrap();
        assert_eq!(chat_in.session_id, "test-session");
        assert_eq!(chat_in.model, "mistral");
        assert_eq!(chat_in.message, "Hello there");
    }

    #[test]
    fn test_chat_out_serialization() {
        let chat_out = ChatOut {
            reply: "Test response".to_string(),
        };

        let json = serde_json::to_string(&chat_out).unwrap();
        assert!(json.contains("\"reply\":\"Test response\""));
    }
}