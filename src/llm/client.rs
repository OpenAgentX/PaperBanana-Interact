use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

use super::models::{ContentPart, ImageGenResponse, LlmResponse, Message, Role};

pub struct LlmClient {
    reqwest_client: Client,
    model_name: String,
}

impl LlmClient {
    pub fn new(model_name: &str) -> Self {
        Self {
            reqwest_client: Client::new(),
            model_name: model_name.to_string(),
        }
    }

    pub async fn generate_text(&self, messages: &[Message]) -> Result<LlmResponse> {
        if self.model_name.starts_with("gemini") {
            self.generate_text_gemini(messages).await
        } else if self.model_name.starts_with("gpt") {
            self.generate_text_openai(messages).await
        } else if self.model_name.starts_with("claude") {
            self.generate_text_anthropic(messages).await
        } else {
            Err(anyhow!("Unsupported text model: {}", self.model_name))
        }
    }

    pub async fn generate_image(&self, prompt: &str) -> Result<ImageGenResponse> {
        if self.model_name.starts_with("gemini") || self.model_name == "imagen" {
             self.generate_image_google(prompt).await
        } else if self.model_name.starts_with("dall-e") {
             self.generate_image_openai(prompt).await
        } else {
             Err(anyhow!("Unsupported image model: {}", self.model_name))
        }
    }

    async fn generate_text_gemini(&self, messages: &[Message]) -> Result<LlmResponse> {
        let api_key = env::var("GOOGLE_API_KEY").unwrap_or_default();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model_name, api_key
        );

        let mut contents = Vec::new();
        let mut system_instruction = None;

        for msg in messages {
            if msg.role == Role::System {
                let parts: Vec<Value> = msg.content.iter().filter_map(|p| match p {
                    ContentPart::Text(t) => Some(json!({"text": t})),
                    _ => None
                }).collect();
                system_instruction = Some(json!({"parts": parts}));
                continue;
            }

            let role_str = match msg.role {
                Role::User => "user",
                Role::Assistant => "model",
                Role::System => "system",
            };

            let parts: Vec<Value> = msg.content.iter().map(|p| match p {
                ContentPart::Text(t) => json!({"text": t}),
                ContentPart::Image { data, mime_type } => {
                    let clean_data = data.split(',').next_back().unwrap_or(data);
                    json!({
                        "inlineData": {
                            "mimeType": mime_type,
                            "data": clean_data
                        }
                    })
                }
            }).collect();

            contents.push(json!({
                "role": role_str,
                "parts": parts
            }));
        }

        let mut body = json!({
            "contents": contents
        });

        if let Some(sys) = system_instruction {
            body["systemInstruction"] = sys;
        }

        let res: Value = self.reqwest_client.post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        let text = res["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Failed to parse Gemini response: {}", res))?
            .to_string();

        Ok(LlmResponse { text })
    }

    async fn generate_text_openai(&self, messages: &[Message]) -> Result<LlmResponse> {
        let api_key = env::var("OPENAI_API_KEY").unwrap_or_default();
        let url = "https://api.openai.com/v1/chat/completions";

        let mut formatted_messages = Vec::new();
        for msg in messages {
            let role_str = match msg.role {
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::System => "system",
            };

            let content: Value = if msg.content.len() == 1 && matches!(msg.content[0], ContentPart::Text(_)) {
                if let ContentPart::Text(ref t) = msg.content[0] {
                    json!(t)
                } else {
                    json!("")
                }
            } else {
                let parts: Vec<Value> = msg.content.iter().map(|p| match p {
                    ContentPart::Text(t) => json!({"type": "text", "text": t}),
                    ContentPart::Image { data, mime_type } => {
                        let data_url = if data.starts_with("data:") {
                            data.clone()
                        } else {
                            format!("data:{};base64,{}", mime_type, data)
                        };
                        json!({
                            "type": "image_url",
                            "image_url": {
                                "url": data_url
                            }
                        })
                    }
                }).collect();
                json!(parts)
            };

            formatted_messages.push(json!({
                "role": role_str,
                "content": content
            }));
        }

        let body = json!({
            "model": self.model_name,
            "messages": formatted_messages,
        });

        let res: Value = self.reqwest_client.post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        let text = res["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Failed to parse OpenAI response: {}", res))?
            .to_string();

        Ok(LlmResponse { text })
    }

    async fn generate_text_anthropic(&self, messages: &[Message]) -> Result<LlmResponse> {
        let api_key = env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        let url = "https://api.anthropic.com/v1/messages";

        let mut formatted_messages = Vec::new();
        let mut system = String::new();

        for msg in messages {
            if msg.role == Role::System {
                if let Some(ContentPart::Text(t)) = msg.content.first() {
                    system = t.clone();
                }
                continue;
            }

            let role_str = match msg.role {
                Role::User => "user",
                Role::Assistant => "assistant",
                _ => "user",
            };

            let content: Vec<Value> = msg.content.iter().map(|p| match p {
                ContentPart::Text(t) => json!({"type": "text", "text": t}),
                ContentPart::Image { data, mime_type } => {
                    let clean_data = data.split(',').next_back().unwrap_or(data);
                    json!({
                        "type": "image",
                        "source": {
                            "type": "base64",
                            "media_type": mime_type,
                            "data": clean_data
                        }
                    })
                }
            }).collect();

            formatted_messages.push(json!({
                "role": role_str,
                "content": content
            }));
        }

        let mut body = json!({
            "model": self.model_name,
            "max_tokens": 4096,
            "messages": formatted_messages,
        });

        if !system.is_empty() {
            body["system"] = json!(system);
        }

        let res: Value = self.reqwest_client.post(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        let text = res["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Failed to parse Anthropic response: {}", res))?
            .to_string();

        Ok(LlmResponse { text })
    }

    async fn generate_image_google(&self, prompt: &str) -> Result<ImageGenResponse> {
        let api_key = env::var("GOOGLE_API_KEY").unwrap_or_default();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/imagen-3.0-generate-001:predict?key={}",
            api_key
        );

        let body = json!({
            "instances": [
                { "prompt": prompt }
            ],
            "parameters": {
                "sampleCount": 1,
                "outputOptions": { "mimeType": "image/png" }
            }
        });

        let res: Value = self.reqwest_client.post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if let Some(predictions) = res.get("predictions").and_then(|p| p.as_array()) {
            let mut images = Vec::new();
            for pred in predictions {
                if let Some(b64) = pred.get("bytesBase64Encoded").and_then(|b| b.as_str()) {
                    images.push(b64.to_string());
                }
            }
            if !images.is_empty() {
                return Ok(ImageGenResponse { images_base64: images });
            }
        }

        Err(anyhow!("Failed to parse Google image generation response: {}", res))
    }

    async fn generate_image_openai(&self, prompt: &str) -> Result<ImageGenResponse> {
        let api_key = env::var("OPENAI_API_KEY").unwrap_or_default();
        let url = "https://api.openai.com/v1/images/generations";

        let body = json!({
            "model": self.model_name,
            "prompt": prompt,
            "n": 1,
            "response_format": "b64_json"
        });

        let res: Value = self.reqwest_client.post(url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if let Some(data) = res.get("data").and_then(|d| d.as_array()) {
            let mut images = Vec::new();
            for item in data {
                if let Some(b64) = item.get("b64_json").and_then(|b| b.as_str()) {
                    images.push(b64.to_string());
                }
            }
            if !images.is_empty() {
                return Ok(ImageGenResponse { images_base64: images });
            }
        }

        Err(anyhow!("Failed to parse OpenAI image generation response: {}", res))
    }
}
