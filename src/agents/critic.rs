#![allow(dead_code)]
use super::base_agent::{AgentConfig, TaskConfig};
use crate::llm::client::LlmClient;
use crate::llm::models::{Message, Role, ContentPart};
use crate::prompts::agent_prompts::DIAGRAM_CRITIC_AGENT_SYSTEM_PROMPT;
use serde_json::Value;
use anyhow::Result;

pub struct CriticAgent {
    _llm: LlmClient,
    task_config: TaskConfig,
}

impl CriticAgent {
    pub fn new(config: AgentConfig) -> Self {
        let task_config = TaskConfig {
            task_name: "diagram".to_string(),
            critique_target: "Target Diagram for Critique:".to_string(),
            context_labels: vec!["Methodology Section".to_string(), "Figure Caption".to_string()],
            ..Default::default()
        };
        Self {
            _llm: LlmClient::new(&config.main_model_name),
            task_config,
        }
    }

    pub async fn process(&self, detailed_description: &str, image_b64: &str, content: &str, visual_intent: &str) -> Result<(String, String)> {
        let mut msg_content = vec![ContentPart::Text(self.task_config.critique_target.clone())];
        if !image_b64.is_empty() {
             msg_content.push(ContentPart::Image {
                 data: image_b64.to_string(),
                 mime_type: "image/jpeg".to_string()
             });
        } else {
             msg_content.push(ContentPart::Text("\n[SYSTEM NOTICE] The image could not be generated based on the current description. Please provide a revised version.".to_string()));
        }

        msg_content.push(ContentPart::Text(format!(
            "Detailed Description: {}\n{}: {}\n{}: {}\nYour Output:",
            detailed_description,
            self.task_config.context_labels[0], content,
            self.task_config.context_labels[1], visual_intent
        )));

        let messages = vec![
            Message { role: Role::System, content: vec![ContentPart::Text(DIAGRAM_CRITIC_AGENT_SYSTEM_PROMPT.to_string())] },
            Message { role: Role::User, content: msg_content }
        ];

        let res = self._llm.generate_text(&messages).await?;
        let cleaned = res.text.replace("```json", "").replace("```", "");
        let parsed: Value = serde_json::from_str(&cleaned).unwrap_or_else(|_| serde_json::json!({}));

        let suggestions = parsed["critic_suggestions"].as_str().unwrap_or("No changes needed.").to_string();
        let revised_desc = parsed["revised_description"].as_str().unwrap_or("No changes needed.").to_string();

        Ok((suggestions, revised_desc))
    }
}
