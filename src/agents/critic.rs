#![allow(dead_code)]
use anyhow::Result;
use crate::llm::{client::LlmClient, models::{Message, Role, ContentPart}};
use crate::prompts::agent_prompts::CRITIC_PROMPT;
use super::base_agent::Agent;

pub struct CriticAgent {
    llm: LlmClient,
}

impl CriticAgent {
    pub fn new(llm: LlmClient) -> Self { Self { llm } }

    pub async fn run(&self, criteria: &str, image_b64: &str) -> Result<String> {
        let messages = vec![
            Message { role: Role::System, content: vec![ContentPart::Text(CRITIC_PROMPT.to_string())] },
            Message { role: Role::User, content: vec![
                ContentPart::Text(criteria.to_string()),
                ContentPart::Image {
                    data: image_b64.to_string(),
                    mime_type: "image/png".to_string()
                }
            ] }
        ];
        let res = self.llm.generate_text(&messages).await?;
        Ok(res.text)
    }
}

impl Agent for CriticAgent {
    fn name(&self) -> &str { "Critic" }
}
