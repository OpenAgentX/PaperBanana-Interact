#![allow(dead_code)]

pub trait Agent {
    fn name(&self) -> &str;
}

pub struct AgentConfig {
    pub main_model_name: String,
    pub image_gen_model_name: String,
    pub temperature: f32,
    pub dataset_dir: String,
    pub work_dir: String,
}

#[derive(Debug, Clone)]
pub struct TaskConfig {
    pub task_name: String,
    pub content_label: String,
    pub visual_intent_label: String,
    pub context_labels: Vec<String>,
    pub candidate_type: String,
    pub candidate_labels: Vec<String>,
    pub output_key: String,
    pub instruction_suffix: String,
    pub ref_limit: Option<usize>,
    pub prompt_template: String,
    pub max_output_tokens: usize,
    pub critique_target: String,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            task_name: "diagram".to_string(),
            content_label: "Methodology Section".to_string(),
            visual_intent_label: "Diagram Caption".to_string(),
            context_labels: vec![],
            candidate_type: "".to_string(),
            candidate_labels: vec![],
            output_key: "".to_string(),
            instruction_suffix: "".to_string(),
            ref_limit: None,
            prompt_template: "".to_string(),
            max_output_tokens: 50000,
            critique_target: "".to_string(),
        }
    }
}
