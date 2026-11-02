#![allow(dead_code)]
use anyhow::Result;
use crate::llm::{client::LlmClient, models::{Message, Role, ContentPart}};
use super::base_agent::Agent;

pub struct StylistAgent {
    llm: LlmClient,
}

impl StylistAgent {
    pub fn new(llm: LlmClient) -> Self { Self { llm } }

    pub async fn run(&self, plan: &str) -> Result<String> {
        let messages = vec![
            Message { role: Role::System, content: vec![ContentPart::Text("You are a stylist.".to_string())] },
            Message { role: Role::User, content: vec![ContentPart::Text(plan.to_string())] }
        ];
        let res = self.llm.generate_text(&messages).await?;
        Ok(res.text)
    }
}

impl Agent for StylistAgent {
    fn name(&self) -> &str { "Stylist" }
}
