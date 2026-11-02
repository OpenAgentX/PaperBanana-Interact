#![allow(dead_code)]
use super::base_agent::Agent;

pub struct RetrieverAgent {}

impl RetrieverAgent {
    pub fn new() -> Self { Self {} }
}

impl Agent for RetrieverAgent {
    fn name(&self) -> &str { "Retriever" }
}

impl Default for RetrieverAgent {
    fn default() -> Self {
        Self::new()
    }
}
