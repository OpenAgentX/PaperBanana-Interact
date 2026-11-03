#![allow(dead_code)]
use super::base_agent::{AgentConfig, TaskConfig};
use crate::llm::client::LlmClient;
use crate::llm::models::{Message, Role, ContentPart};
use crate::prompts::agent_prompts::DIAGRAM_PLANNER_AGENT_SYSTEM_PROMPT;
use crate::utils::image::read_image_to_base64;
use serde_json::Value;
use std::fs;
use anyhow::Result;

pub struct PlannerAgent {
    _llm: LlmClient,
    _config: AgentConfig,
    task_config: TaskConfig,
}

impl PlannerAgent {
    pub fn new(config: AgentConfig) -> Self {
        let task_config = TaskConfig {
            task_name: "diagram".to_string(),
            content_label: "Methodology Section".to_string(),
            visual_intent_label: "Diagram Caption".to_string(),
            ..Default::default()
        };

        Self {
            _llm: LlmClient::new(&config.main_model_name),
            _config: config,
            task_config,
        }
    }

    pub async fn process(&self, content: &str, visual_intent: &str, retrieved_ids: &[String], retrieved_examples: Option<&Vec<Value>>) -> Result<String> {
        let mut examples_pool = Vec::new();
        if let Some(exs) = retrieved_examples
             && !exs.is_empty() {
                  examples_pool = exs.clone();
             }

        if examples_pool.is_empty() {
             let ref_file = format!("{}/ref.json", self._config.dataset_dir);
             let candidate_pool_str = fs::read_to_string(&ref_file).unwrap_or_else(|_| "[]".to_string());
             let candidate_pool: Vec<Value> = serde_json::from_str(&candidate_pool_str)?;
             for item in candidate_pool {
                  if let Some(id) = item["id"].as_str()
                       && retrieved_ids.contains(&id.to_string()) {
                            examples_pool.push(item);
                       }
             }
        }

        let mut msg_content = Vec::new();
        let mut user_prompt = String::new();

        for (idx, item) in examples_pool.into_iter().enumerate() {
            let item_content = item["content"].to_string();
            let item_caption = item["visual_intent"].as_str().unwrap_or("");
            let img_path = item["path_to_gt_image"].as_str().unwrap_or("");

            user_prompt.push_str(&format!(
                "Example {}:\n{}: {}\n{}: {}\nReference {}: ",
                idx + 1,
                self.task_config.content_label, item_content,
                self.task_config.visual_intent_label, item_caption,
                self.task_config.task_name.to_uppercase()
            ));

            msg_content.push(ContentPart::Text(user_prompt.clone()));
            user_prompt.clear();

            let full_img_path = format!("{}/{}", self._config.dataset_dir, img_path);
            if let Ok(b64) = read_image_to_base64(&full_img_path) {
                 msg_content.push(ContentPart::Image {
                     data: b64,
                     mime_type: "image/png".to_string(),
                 });
            }
        }

        let task_cap = if self.task_config.task_name == "diagram" { " (do not include figure titles)" } else { "" };

        user_prompt.push_str(&format!(
            "Now, based on the following {} and {}, provide a detailed description for the figure to be generated.\n{}: {}\n{}: {}\nDetailed description of the target figure to be generated{}:",
            self.task_config.content_label.to_lowercase(),
            self.task_config.visual_intent_label.to_lowercase(),
            self.task_config.content_label, content,
            self.task_config.visual_intent_label, visual_intent,
            task_cap
        ));
        msg_content.push(ContentPart::Text(user_prompt));

        let messages = vec![
            Message { role: Role::System, content: vec![ContentPart::Text(DIAGRAM_PLANNER_AGENT_SYSTEM_PROMPT.to_string())] },
            Message { role: Role::User, content: msg_content }
        ];

        let res = self._llm.generate_text(&messages).await?;
        Ok(res.text.trim().to_string())
    }
}
