#![allow(dead_code)]
use super::base_agent::{Agent, AgentConfig};
use crate::llm::client::LlmClient;
use crate::llm::models::{Message, Role, ContentPart};
use crate::prompts::agent_prompts::DIAGRAM_RETRIEVER_AGENT_SYSTEM_PROMPT;
use serde_json::Value;
use std::fs;
use anyhow::Result;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct RetrieverAgent {
    _llm: LlmClient,
    _config: AgentConfig,
}

impl RetrieverAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            _llm: LlmClient::new(&config.main_model_name),
            _config: config,
        }
    }

    pub async fn process(&self, content: &str, visual_intent: &str, mut retrieval_setting: &str) -> Result<(Vec<String>, Vec<Value>)> {
        let ref_file = format!("{}/ref.json", self._config.dataset_dir);
        let manual_file = format!("{}/agent_selected_12.json", self._config.dataset_dir);

        if (retrieval_setting == "auto" || retrieval_setting == "random") && !std::path::Path::new(&ref_file).exists() {
            println!("Warning: Reference file not found at {}. Falling back to retrieval_setting='none'.", ref_file);
            retrieval_setting = "none";
        }

        if retrieval_setting == "manual" && !std::path::Path::new(&manual_file).exists() {
            println!("Warning: Manual reference file not found at {}. Falling back to retrieval_setting='none'.", manual_file);
            retrieval_setting = "none";
        }

        match retrieval_setting {
            "none" => Ok((vec![], vec![])),
            "manual" => self.load_manual_references(&manual_file),
            "random" => self.load_random_references(&ref_file),
            "auto" => self.retrieve_and_parse(content, visual_intent, &ref_file).await,
            _ => Err(anyhow::anyhow!("Unknown retrieval_setting: {}", retrieval_setting))
        }
    }

    fn load_manual_references(&self, manual_file: &str) -> Result<(Vec<String>, Vec<Value>)> {
        let content = fs::read_to_string(manual_file)?;
        let mut examples: Vec<Value> = serde_json::from_str(&content)?;
        examples.truncate(10);

        let ids = examples.iter()
            .filter_map(|item| item["id"].as_str().map(|s| s.to_string()))
            .collect();

        Ok((ids, examples))
    }

    fn load_random_references(&self, ref_file: &str) -> Result<(Vec<String>, Vec<Value>)> {
        let content = fs::read_to_string(ref_file)?;
        let candidate_pool: Vec<Value> = serde_json::from_str(&content)?;

        let mut ids: Vec<String> = candidate_pool.iter()
            .filter_map(|item| item["id"].as_str().map(|s| s.to_string()))
            .collect();

        let mut rng = thread_rng();
        ids.shuffle(&mut rng);
        ids.truncate(10);

        Ok((ids, vec![])) // Planner will load from ref.json
    }

    async fn retrieve_and_parse(&self, content: &str, visual_intent: &str, ref_file: &str) -> Result<(Vec<String>, Vec<Value>)> {
        let mut user_prompt = format!(
            "**Target Input**\n- Caption: {}\n- Methodology section: {}\n\n**Candidate Pool**\n",
            visual_intent, content
        );

        let candidate_pool_str = fs::read_to_string(ref_file)?;
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

        let res = self._llm.generate_text(&messages).await?;
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

        Ok((ids, vec![])) // Planner will load from ref.json
    }
}

impl Agent for RetrieverAgent {
    fn name(&self) -> &str { "Retriever" }
}
