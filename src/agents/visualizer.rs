use anyhow::Result;
use crate::llm::client::LlmClient;
use super::base_agent::Agent;

pub struct VisualizerAgent {
    llm: LlmClient,
}

impl VisualizerAgent {
    pub fn new(llm: LlmClient) -> Self { Self { llm } }

    pub async fn run(&self, final_prompt: &str) -> Result<Vec<String>> {
        let res = self.llm.generate_image(final_prompt).await?;
        Ok(res.images_base64)
    }
}

impl Agent for VisualizerAgent {
    fn name(&self) -> &str { "Visualizer" }
}
