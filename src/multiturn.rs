#![allow(dead_code)]
use anyhow::Result;
use std::path::PathBuf;
use tokio::sync::Semaphore;
use std::sync::Arc;
use serde_json::Value;
use std::fs;

pub struct ExpConfig {
    pub dataset_name: String,
    pub split_name: String,
    pub exp_mode: String,
    pub retrieval_setting: String,
    pub max_critic_rounds: usize,
    pub refiner_internal_critic_rounds: usize,
    pub main_model_name: String,
    pub image_gen_model_name: String,
    pub refiner: String,
    pub work_dir: PathBuf,
    pub concurrent_num: usize,
    pub user_model_name: String,
    pub n_surfaced_req_per_turn: usize,
    pub dataset_dir: PathBuf,
    pub result_dir: PathBuf,
}

pub struct MultiTurnRunner {
    pub config: ExpConfig,
}

impl MultiTurnRunner {
    pub fn new(config: ExpConfig) -> Self {
        Self { config }
    }

    pub async fn run(
        &self,
        _load_initial_result: Option<PathBuf>,
        _load_prior_results: Option<PathBuf>,
        feedback_turns: usize,
    ) -> Result<()> {
        let input_filename = self.config.dataset_dir.join(format!("{}.json", self.config.split_name));
        let output_dirname = self.config.result_dir.join(format!(
            "{}_{}_{}",
            self.config.dataset_name, self.config.split_name, self.config.refiner
        ));

        fs::create_dir_all(&output_dirname)?;
        println!("Input file: {:?}", input_filename);
        println!("Output dirname: {:?}", output_dirname);

        let input_str = fs::read_to_string(&input_filename)?;
        let data_list: Vec<Value> = serde_json::from_str(&input_str)?;

        // Ensure validity
        for data in &data_list {
            if data.get("requirements").is_none() || data.get("req_order").is_none() {
                anyhow::bail!("Input data missing 'requirements' or 'req_order'");
            }
        }

        let sem = Arc::new(Semaphore::new(self.config.concurrent_num));
        let total_examples = data_list.len();
        println!("Loaded {} examples.", total_examples);

        // A minimal task loop replicating the pipeline flow logic
        let mut tasks = Vec::new();
        for data in data_list {
            let sem_clone = Arc::clone(&sem);
            let turns = feedback_turns;

            let handle = tokio::spawn(async move {
                let _permit = sem_clone.acquire().await.unwrap();
                let mut current_data = data;

                // Simulate running T = 0 -> N
                for t in 0..=turns {
                    // Logic to invoke processor for either single-turn (t=0) or feedback turn (t>0)
                    // ...
                    current_data["turn_processed"] = serde_json::json!(t);
                }
                current_data
            });
            tasks.push(handle);
        }

        let mut results = Vec::new();
        for handle in tasks {
             results.push(handle.await?);
        }

        println!("Completed processing {} items across multi-turns.", results.len());
        Ok(())
    }
}
