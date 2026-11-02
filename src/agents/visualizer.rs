use super::base_agent::{Agent, AgentConfig};
use crate::llm::client::LlmClient;
use anyhow::Result;

pub struct VisualizerAgent {
    llm: LlmClient,
}

impl VisualizerAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            llm: LlmClient::new(&config.image_gen_model_name),
        }
    }

    pub async fn process(&self, description: &str) -> Result<String> {
        let prompt_text = format!("Render an image based on the following detailed description: {}\n Note that do not include figure titles in the image. Diagram: ", description);
        // The visualizer just calls the image gen model, injecting the system prompt internally or as part of the config if supported.
        let res = self.llm.generate_image(&prompt_text).await?;
        Ok(res.images_base64.into_iter().next().unwrap_or_default())
    }
}

impl Agent for VisualizerAgent {
    fn name(&self) -> &str { "Visualizer" }
}
