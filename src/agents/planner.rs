#![allow(dead_code)]
use super::base_agent::{Agent, AgentConfig};
use crate::llm::client::LlmClient;
use crate::llm::models::{Message, Role, ContentPart};
use crate::prompts::agent_prompts::DIAGRAM_PLANNER_AGENT_SYSTEM_PROMPT;
use crate::utils::image::read_image_to_base64;
use serde_json::Value;
use std::fs;
use anyhow::Result;

pub struct PlannerAgent {
    llm: LlmClient,
    config: AgentConfig,
}

impl PlannerAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            llm: LlmClient::new(&config.main_model_name),
            config,
        }
    }

    pub async fn process(&self, content: &str, visual_intent: &str, retrieved_ids: &[String]) -> Result<String> {
        let ref_file = format!("{}/ref.json", self.config.dataset_dir);
        let candidate_pool_str = fs::read_to_string(&ref_file).unwrap_or_else(|_| "[]".to_string());
        let candidate_pool: Vec<Value> = serde_json::from_str(&candidate_pool_str)?;

        let mut msg_content = Vec::new();
        let mut user_prompt = String::new();

        let mut idx = 1;
        for item in candidate_pool {
            if let Some(id) = item["id"].as_str()
                && retrieved_ids.contains(&id.to_string()) {
                    let item_content = item["content"].to_string();
                    let item_caption = item["visual_intent"].as_str().unwrap_or("");
                    let img_path = item["path_to_gt_image"].as_str().unwrap_or("");

                    user_prompt.push_str(&format!("Example {}:\nMethodology Section: {}\nDiagram Caption: {}\nReference Diagram: ", idx, item_content, item_caption));
                    msg_content.push(ContentPart::Text(user_prompt.clone()));
                    user_prompt.clear();

                    let full_img_path = format!("{}/{}", self.config.dataset_dir, img_path);
                    if let Ok(b64) = read_image_to_base64(&full_img_path) {
                         msg_content.push(ContentPart::Image {
                             data: b64,
                             mime_type: "image/png".to_string(),
                         });
                    }

                    idx += 1;
                }
        }

        user_prompt.push_str(&format!(
            "Now, based on the following methodology section and diagram caption, provide a detailed description for the figure to be generated.\nMethodology Section: {}\nDiagram Caption: {}\nDetailed description of the target figure to be generated (do not include figure titles):",
            content, visual_intent
        ));
        msg_content.push(ContentPart::Text(user_prompt));

        let messages = vec![
            Message { role: Role::System, content: vec![ContentPart::Text(DIAGRAM_PLANNER_AGENT_SYSTEM_PROMPT.to_string())] },
            Message { role: Role::User, content: msg_content }
        ];

        let res = self.llm.generate_text(&messages).await?;
        Ok(res.text.trim().to_string())
    }
}

impl Agent for PlannerAgent {
    fn name(&self) -> &str { "Planner" }
}
