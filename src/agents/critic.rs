#![allow(dead_code)]
use super::base_agent::{Agent, AgentConfig};
use crate::llm::client::LlmClient;
use crate::llm::models::{Message, Role, ContentPart};
use crate::prompts::agent_prompts::DIAGRAM_CRITIC_AGENT_SYSTEM_PROMPT;
use serde_json::Value;
use anyhow::Result;

pub struct CriticAgent {
    llm: LlmClient,
}

impl CriticAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            llm: LlmClient::new(&config.main_model_name),
        }
    }

    pub async fn process(&self, detailed_description: &str, image_b64: &str, content: &str, visual_intent: &str) -> Result<(String, String)> {
        let mut msg_content = vec![ContentPart::Text("Target Diagram for Critique:".to_string())];
        if !image_b64.is_empty() {
             msg_content.push(ContentPart::Image {
                 data: image_b64.to_string(),
                 mime_type: "image/jpeg".to_string()
             });
        } else {
             msg_content.push(ContentPart::Text("\n[SYSTEM NOTICE] The image could not be generated based on the current description. Please provide a revised version.".to_string()));
        }

        msg_content.push(ContentPart::Text(format!(
            "Detailed Description: {}\nMethodology Section: {}\nFigure Caption: {}\nYour Output:",
            detailed_description, content, visual_intent
        )));

        let messages = vec![
            Message { role: Role::System, content: vec![ContentPart::Text(DIAGRAM_CRITIC_AGENT_SYSTEM_PROMPT.to_string())] },
            Message { role: Role::User, content: msg_content }
        ];

        let res = self.llm.generate_text(&messages).await?;
        let cleaned = res.text.replace("```json", "").replace("```", "");
        let parsed: Value = serde_json::from_str(&cleaned).unwrap_or_else(|_| serde_json::json!({}));

        let suggestions = parsed["critic_suggestions"].as_str().unwrap_or("No changes needed.").to_string();
        let revised_desc = parsed["revised_description"].as_str().unwrap_or("No changes needed.").to_string();

        Ok((suggestions, revised_desc))
    }
}

impl Agent for CriticAgent {
    fn name(&self) -> &str { "Critic" }
}
