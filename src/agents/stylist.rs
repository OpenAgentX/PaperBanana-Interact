#![allow(dead_code)]
use super::base_agent::{AgentConfig, TaskConfig};
use crate::llm::client::LlmClient;
use crate::llm::models::{Message, Role, ContentPart};
use crate::prompts::agent_prompts::DIAGRAM_STYLIST_AGENT_SYSTEM_PROMPT;
use std::fs;
use anyhow::Result;

pub struct StylistAgent {
    _llm: LlmClient,
    _config: AgentConfig,
    task_config: TaskConfig,
}

impl StylistAgent {
    pub fn new(config: AgentConfig) -> Self {
        let task_config = TaskConfig {
            task_name: "diagram".to_string(),
            context_labels: vec!["Methodology Section".to_string(), "Diagram Caption".to_string()],
            ..Default::default()
        };
        Self {
            _llm: LlmClient::new(&config.main_model_name),
            _config: config,
            task_config,
        }
    }

    pub async fn process(&self, detailed_description: &str, content: &str, visual_intent: &str) -> Result<String> {
        let style_file = format!("{}/style_guides/neurips2025_{}_style_guide.md", self._config.work_dir, self.task_config.task_name);
        let style_guide = fs::read_to_string(&style_file).unwrap_or_else(|_| "No style guide found.".to_string());

        let user_prompt = format!(
            "Detailed Description: {}\nStyle Guidelines: {}\n{}: {}\n{}: {}\nYour Output:",
            detailed_description, style_guide,
            self.task_config.context_labels[0], content,
            self.task_config.context_labels[1], visual_intent
        );

        let messages = vec![
            Message { role: Role::System, content: vec![ContentPart::Text(DIAGRAM_STYLIST_AGENT_SYSTEM_PROMPT.to_string())] },
            Message { role: Role::User, content: vec![ContentPart::Text(user_prompt)] }
        ];

        let res = self._llm.generate_text(&messages).await?;
        Ok(res.text)
    }
}
