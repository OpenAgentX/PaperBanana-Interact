#![allow(dead_code)]
use super::base_agent::{Agent, AgentConfig};
use crate::llm::client::LlmClient;
use crate::llm::models::{Message, Role, ContentPart};
use crate::prompts::agent_prompts::DIAGRAM_RETRIEVER_AGENT_SYSTEM_PROMPT;
use serde_json::Value;
use std::fs;
use anyhow::Result;

pub struct RetrieverAgent {
    llm: LlmClient,
    config: AgentConfig,
}

impl RetrieverAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            llm: LlmClient::new(&config.main_model_name),
            config,
        }
    }

    pub async fn process(&self, content: &str, visual_intent: &str) -> Result<Vec<String>> {
        let mut user_prompt = format!(
            "**Target Input**\n- Caption: {}\n- Methodology section: {}\n\n**Candidate Pool**\n",
            visual_intent, content
        );

        let ref_file = format!("{}/ref.json", self.config.dataset_dir);
        let candidate_pool_str = fs::read_to_string(&ref_file).unwrap_or_else(|_| "[]".to_string());
        let candidate_pool: Vec<Value> = serde_json::from_str(&candidate_pool_str)?;

        for (idx, item) in candidate_pool.iter().take(200).enumerate() {
            let id = item["id"].as_str().unwrap_or("");
            let caption = item["visual_intent"].as_str().unwrap_or("");
            let method = item["content"].to_string();

            user_prompt.push_str(&format!(
                "Candidate Diagram {}:\n- Diagram ID: {}\n- Caption: {}\n- Methodology section: {}\n\n",
                idx + 1, id, caption, method
            ));
        }

        user_prompt.push_str("Now, based on the Target Input and the Candidate Pool, select the Top 10 most relevant diagrams according to the instructions provided. Your output should be a strictly valid JSON object containing a single list of the exact ids of the top 10 selected diagrams.");

        let messages = vec![
            Message { role: Role::System, content: vec![ContentPart::Text(DIAGRAM_RETRIEVER_AGENT_SYSTEM_PROMPT.to_string())] },
            Message { role: Role::User, content: vec![ContentPart::Text(user_prompt)] }
        ];

        let res = self.llm.generate_text(&messages).await?;
        let cleaned = res.text.replace("```json", "").replace("```", "");
        let parsed: Value = serde_json::from_str(&cleaned).unwrap_or_else(|_| serde_json::json!({ "top10_diagrams": [] }));

        let mut ids = Vec::new();
        if let Some(arr) = parsed["top10_diagrams"].as_array() {
            for v in arr {
                if let Some(s) = v.as_str() {
                    ids.push(s.to_string());
                }
            }
        }
        Ok(ids)
    }
}

impl Agent for RetrieverAgent {
    fn name(&self) -> &str { "Retriever" }
}
