use crate::agents::base_agent::AgentConfig;
use crate::agents::visualizer::VisualizerAgent;
use crate::llm::client::LlmClient;
use crate::llm::models::{Message, Role, ContentPart};
use anyhow::Result;
use serde_json::Value;

const STEP1_CRITIQUE_PROMPT_CRITIQUE_ONLY: &str = r#"Please perform the first step: Critique. Walk target diagram against every constraint one by one and emit the per-aspect critique, noting for each issue what is wrong and how it should be fixed.

## OUTPUT
Provide your response strictly in the following JSON format.

```json
{
    "critique_latest_turn": "...",
    "critique_source_content": "...",
    "critique_presentation": "..."
}
```"#;

const STEP2_GATING_PROMPT: &str = r#"Please perform the second step: Decide. Based on your critique above, decide whether target diagram still needs any revision.
Answer with **a single word and nothing else**: "YES" if any revision is needed, "NO" if the diagram already satisfies every requirement and no changes are needed."#;

const STEP3_REWRITE_PROMPT: &str = r#"Please perform the third step: Refine. The Detailed Description that produced target diagram is now provided below; revise it based on your critique above, producing these two fields sequentially:
1. "critic_suggestions": synthesize all of the above into concrete, actionable suggestions, trying to incorporate and balance all of the requirements above rather than fixing one at the expense of another.
2. "revised_description": the fully revised detailed description incorporating every suggestion. Follow the "POSITIVE, SPECIFIC DESCRIPTIONS" rule below — state what the diagram SHOULD contain, affirmatively and specifically, not what it should avoid.

** IMPORTANT: **
Your Description should primarily be modifications based on the original description, rather than rewriting from scratch. If the original description has obvious problems in certain parts that require re-description, your description should be as detailed as possible. Semantically, clearly describe each element and their connections. Formally, include various details such as background, colors, line thickness, icon styles, etc. Remember: vague or unclear specifications will only make the generated figure worse, not better.

## OUTPUT
Provide your response strictly in the following JSON format.

```json
{
    "critic_suggestions": "...",
    "revised_description": "..."
}
```"#;

pub struct PbInteractRefiner {
    llm: LlmClient,
    visualizer: VisualizerAgent,
}

impl PbInteractRefiner {
    pub fn new(config: AgentConfig) -> Self {
        let visualizer = VisualizerAgent::new(AgentConfig {
            main_model_name: config.main_model_name.clone(),
            image_gen_model_name: config.image_gen_model_name.clone(),
            temperature: config.temperature,
            dataset_dir: config.dataset_dir.clone(),
            work_dir: config.work_dir.clone(),
        });
        Self {
            llm: LlmClient::new(&config.main_model_name),
            visualizer,
        }
    }

    pub async fn refine(&self, initial_image_b64: &str, detailed_description: &str, user_request: &str, history: &str, method_section: &str, figure_caption: &str) -> Result<(String, String)> {
        // Step 1: Critique
        let mut msg_content = vec![ContentPart::Text("Target Diagram:".to_string())];
        if !initial_image_b64.is_empty() {
            msg_content.push(ContentPart::Image {
                data: initial_image_b64.to_string(),
                mime_type: "image/jpeg".to_string(),
            });
        }
        msg_content.push(ContentPart::Text(format!(
            "Methodology Section: {}\nFigure Caption: {}\nChat History:\n{}\nLatest Request: {}\n\n{}",
            method_section, figure_caption, history, user_request, STEP1_CRITIQUE_PROMPT_CRITIQUE_ONLY
        )));

        let mut messages = vec![
            Message { role: Role::User, content: msg_content }
        ];

        let res1 = self.llm.generate_text(&messages).await?;
        messages.push(Message { role: Role::Assistant, content: vec![ContentPart::Text(res1.text.clone())] });

        // Step 2: Gating
        messages.push(Message { role: Role::User, content: vec![ContentPart::Text(STEP2_GATING_PROMPT.to_string())] });
        let res2 = self.llm.generate_text(&messages).await?;
        messages.push(Message { role: Role::Assistant, content: vec![ContentPart::Text(res2.text.clone())] });

        if res2.text.trim().to_uppercase().contains("NO") {
             // No changes needed
             return Ok((initial_image_b64.to_string(), detailed_description.to_string()));
        }

        // Step 3: Rewrite
        messages.push(Message { role: Role::User, content: vec![ContentPart::Text(format!(
            "Detailed Description: {}\n\n{}", detailed_description, STEP3_REWRITE_PROMPT
        ))] });

        let res3 = self.llm.generate_text(&messages).await?;
        let cleaned = res3.text.replace("```json", "").replace("```", "");
        let parsed: Value = serde_json::from_str(&cleaned).unwrap_or_else(|_| serde_json::json!({}));
        let revised_desc = parsed["revised_description"].as_str().unwrap_or(detailed_description).to_string();

        let new_image_b64 = self.visualizer.process(&revised_desc).await?;

        Ok((new_image_b64, revised_desc))
    }
}
