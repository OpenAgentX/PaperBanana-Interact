#![allow(dead_code)]
use super::base_agent::{Agent, AgentConfig};
use crate::llm::client::LlmClient;
use crate::llm::models::{Message, Role, ContentPart};
use crate::prompts::agent_prompts::DIAGRAM_STYLIST_AGENT_SYSTEM_PROMPT;
use std::fs;
use anyhow::Result;

pub struct StylistAgent {
    llm: LlmClient,
    config: AgentConfig,
}

impl StylistAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            llm: LlmClient::new(&config.main_model_name),
            config,
        }
    }

    pub async fn process(&self, detailed_description: &str, content: &str, visual_intent: &str) -> Result<String> {
        let style_file = format!("{}/style_guides/neurips2025_diagram_style_guide.md", self.config.work_dir);
        let style_guide = fs::read_to_string(&style_file).unwrap_or_else(|_| "No style guide found.".to_string());

        let user_prompt = format!(
            "Detailed Description: {}\nStyle Guidelines: {}\nMethodology Section: {}\nDiagram Caption: {}\nYour Output:",
            detailed_description, style_guide, content, visual_intent
        );

        let messages = vec![
            Message { role: Role::System, content: vec![ContentPart::Text(DIAGRAM_STYLIST_AGENT_SYSTEM_PROMPT.to_string())] },
            Message { role: Role::User, content: vec![ContentPart::Text(user_prompt)] }
        ];

        let res = self.llm.generate_text(&messages).await?;
        Ok(res.text)
    }
}

impl Agent for StylistAgent {
    fn name(&self) -> &str { "Stylist" }
}
