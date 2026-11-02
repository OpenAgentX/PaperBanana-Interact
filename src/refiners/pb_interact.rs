use anyhow::Result;
use crate::llm::client::LlmClient;
use crate::agents::{planner::PlannerAgent, critic::CriticAgent, visualizer::VisualizerAgent};

pub struct PbInteractRefiner {
    _llm: LlmClient,
    planner: PlannerAgent,
    _critic: CriticAgent,
    visualizer: VisualizerAgent,
}

impl PbInteractRefiner {
    pub fn new(model_name: &str) -> Self {
        Self {
            _llm: LlmClient::new(model_name),
            planner: PlannerAgent::new(LlmClient::new(model_name)),
            _critic: CriticAgent::new(LlmClient::new(model_name)),
            visualizer: VisualizerAgent::new(LlmClient::new(model_name)),
        }
    }

    pub async fn refine(&self, initial_image_b64: &str, user_request: &str, history: &str) -> Result<String> {
        let planner_res = self.planner.run(
            &format!("History:\n{}\nNew Request:\n{}", history, user_request),
            Some(initial_image_b64)
        ).await?;

        let visualizer_res = self.visualizer.run(&planner_res).await?;

        let best_image = visualizer_res.first().cloned()
            .unwrap_or_else(|| initial_image_b64.to_string());

        Ok(best_image)
    }
}
