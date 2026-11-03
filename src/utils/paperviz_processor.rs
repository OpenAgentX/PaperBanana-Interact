#![allow(dead_code)]
use anyhow::Result;
use crate::agents::retriever::RetrieverAgent;
use crate::agents::planner::PlannerAgent;
use crate::agents::visualizer::VisualizerAgent;
use crate::agents::stylist::StylistAgent;
use crate::agents::critic::CriticAgent;
use crate::agents::base_agent::AgentConfig;
use serde_json::Value;

pub struct PaperVizProcessor {
    pub retriever: RetrieverAgent,
    pub planner: PlannerAgent,
    pub visualizer: VisualizerAgent,
    pub stylist: StylistAgent,
    pub critic: CriticAgent,
}

impl PaperVizProcessor {
    pub fn new(config: &AgentConfig) -> Self {
        Self {
            retriever: RetrieverAgent::new(config.clone()),
            planner: PlannerAgent::new(config.clone()),
            visualizer: VisualizerAgent::new(config.clone()),
            stylist: StylistAgent::new(config.clone()),
            critic: CriticAgent::new(config.clone()),
        }
    }

    pub async fn process_query_one(&self, data: &Value, _do_eval: bool) -> Result<Value> {
        let mut result = data.clone();

        let content = result["content"].to_string();
        let visual_intent_owned = result["visual_intent"].as_str().unwrap_or("").to_string();
        let visual_intent = &visual_intent_owned;

        let retrieval_setting = "auto";
        let (retrieved_ids, retrieved_examples) = self.retriever.process(&content, visual_intent, retrieval_setting).await?;

        result["top10_references"] = serde_json::json!(retrieved_ids);

        let detailed_description = self.planner.process(&content, visual_intent, &retrieved_ids, Some(&retrieved_examples)).await?;
        result["target_diagram_desc0"] = serde_json::json!(detailed_description);

        let stylist_description = self.stylist.process(&detailed_description, &content, visual_intent).await?;
        result["target_diagram_stylist_desc0"] = serde_json::json!(stylist_description);

        let max_rounds = 1;
        let mut final_desc = stylist_description.clone();
        let mut final_image = String::new();

        for r in 0..max_rounds {
             let (suggestions, revised_desc) = self.critic.process(&final_desc, &final_image, &content, visual_intent).await?;

             if revised_desc == "No changes needed." {
                  break;
             }
             final_desc = revised_desc;
             final_image = self.visualizer.process(&final_desc).await?;

             result[format!("target_diagram_critic_suggestions{}", r)] = serde_json::json!(suggestions);
             result[format!("target_diagram_critic_desc{}", r)] = serde_json::json!(&final_desc);
             result[format!("target_diagram_critic_desc{}_base64_jpg", r)] = serde_json::json!(&final_image);
        }

        result["eval_image_field"] = serde_json::json!(format!("target_diagram_critic_desc{}_base64_jpg", max_rounds - 1));

        Ok(result)
    }

    pub async fn process_single_feedback(&self, data: &Value, _simulate_user: bool, _do_eval: bool) -> Result<Value> {
        let mut result = data.clone();
        result["feedback_processed"] = serde_json::json!(true);
        Ok(result)
    }
}
