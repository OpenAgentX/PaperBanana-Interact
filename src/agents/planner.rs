use anyhow::Result;
use crate::llm::{client::LlmClient, models::{Message, Role, ContentPart}};
use crate::prompts::agent_prompts::PLANNER_PROMPT;
use super::base_agent::Agent;

pub struct PlannerAgent {
    llm: LlmClient,
}

impl PlannerAgent {
    pub fn new(llm: LlmClient) -> Self { Self { llm } }

    pub async fn run(&self, request: &str, image_b64: Option<&str>) -> Result<String> {
        let mut msg_content = vec![ContentPart::Text(request.to_string())];
        if let Some(b64) = image_b64 {
            msg_content.push(ContentPart::Image {
                data: b64.to_string(),
                mime_type: "image/png".to_string()
            });
        }

        let messages = vec![
            Message { role: Role::System, content: vec![ContentPart::Text(PLANNER_PROMPT.to_string())] },
            Message { role: Role::User, content: msg_content }
        ];

        let res = self.llm.generate_text(&messages).await?;
        Ok(res.text)
    }
}

impl Agent for PlannerAgent {
    fn name(&self) -> &str { "Planner" }
}
