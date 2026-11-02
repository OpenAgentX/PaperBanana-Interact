#![allow(dead_code)]
pub const PLANNER_PROMPT: &str = r#"You are a planner for scientific diagram generation.
Given the previous multi-turn conversation and image, draft a plan to incorporate user requests."#;

pub const CRITIC_PROMPT: &str = r#"You are a strict critic of scientific diagrams.
Analyze the image and verify if it satisfies the criteria."#;
