#![allow(dead_code)]
use super::base_agent::{AgentConfig, TaskConfig};
use crate::llm::client::LlmClient;
use anyhow::Result;

pub struct VisualizerAgent {
    _llm: LlmClient,
    task_config: TaskConfig,
}

impl VisualizerAgent {
    pub fn new(config: AgentConfig) -> Self {
        let task_config = TaskConfig {
            task_name: "diagram".to_string(),
            prompt_template: "Render an image based on the following detailed description: {desc}\n Note that do not include figure titles in the image. Diagram: ".to_string(),
            max_output_tokens: 50000,
            ..Default::default()
        };
        Self {
            _llm: LlmClient::new(&config.image_gen_model_name),
            task_config,
        }
    }

    pub async fn process(&self, description: &str) -> Result<String> {
        let prompt_text = self.task_config.prompt_template.replace("{desc}", description);
        let res = self._llm.generate_image(&prompt_text).await?;
        Ok(res.images_base64.into_iter().next().unwrap_or_default())
    }
}
