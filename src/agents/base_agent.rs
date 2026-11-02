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
