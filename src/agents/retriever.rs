#![allow(dead_code)]
use super::base_agent::{AgentConfig, TaskConfig};
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
    task_config: TaskConfig,
}

impl RetrieverAgent {
    pub fn new(config: AgentConfig) -> Self {
        let task_config = TaskConfig {
            task_name: "diagram".to_string(),
            context_labels: vec!["Caption".to_string(), "Methodology section".to_string()],
            candidate_labels: vec!["Diagram ID".to_string(), "Caption".to_string(), "Methodology section".to_string()],
            candidate_type: "Diagram".to_string(),
            output_key: "top10_references".to_string(),
            instruction_suffix: "select the Top 10 most relevant diagrams according to the instructions provided. Your output should be a strictly valid JSON object containing a single list of the exact ids of the top 10 selected diagrams.".to_string(),
            ..Default::default()
        };

        Self {
            _llm: LlmClient::new(&config.main_model_name),
            _config: config,
            task_config,
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

        Ok((ids, vec![]))
    }

    async fn retrieve_and_parse(&self, content: &str, visual_intent: &str, ref_file: &str) -> Result<(Vec<String>, Vec<Value>)> {
        let mut user_prompt = format!(
            "**Target Input**\n- {}: {}\n- {}: {}\n\n**Candidate Pool**\n",
            self.task_config.context_labels[0], visual_intent,
            self.task_config.context_labels[1], content
        );

        let candidate_pool_str = fs::read_to_string(ref_file)?;
        let candidate_pool: Vec<Value> = serde_json::from_str(&candidate_pool_str)?;

        for (idx, item) in candidate_pool.iter().take(self.task_config.ref_limit.unwrap_or(200)).enumerate() {
            let id = item["id"].as_str().unwrap_or("");
            let caption = item["visual_intent"].as_str().unwrap_or("");
            let method = item["content"].to_string();

            user_prompt.push_str(&format!(
                "Candidate {} {}:\n- {}: {}\n- {}: {}\n- {}: {}\n\n",
                self.task_config.candidate_type, idx + 1,
                self.task_config.candidate_labels[0], id,
                self.task_config.candidate_labels[1], caption,
                self.task_config.candidate_labels[2], method
            ));
        }

        user_prompt.push_str(&format!("Now, based on the Target Input and the Candidate Pool, {}", self.task_config.instruction_suffix));

        let messages = vec![
            Message { role: Role::System, content: vec![ContentPart::Text(DIAGRAM_RETRIEVER_AGENT_SYSTEM_PROMPT.to_string())] },
            Message { role: Role::User, content: vec![ContentPart::Text(user_prompt)] }
        ];

        let res = self._llm.generate_text(&messages).await?;
        let cleaned = res.text.replace("```json", "").replace("```", "");
        let parsed: Value = serde_json::from_str(&cleaned).unwrap_or_else(|_| serde_json::json!({ "top10_diagrams": [] }));

        let mut ids = Vec::new();
        let target_key = if self.task_config.task_name == "diagram" { "top10_diagrams" } else { "top10_plots" };
        if let Some(arr) = parsed[target_key].as_array() {
            for v in arr {
                if let Some(s) = v.as_str() {
                    ids.push(s.to_string());
                }
            }
        }

        Ok((ids, vec![]))
    }
}
