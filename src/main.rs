mod multiturn;
mod llm;
mod utils;
mod prompts;
mod agents;
mod refiners;
mod session;

use clap::Parser;
use anyhow::Result;
use dotenvy::dotenv;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "PaperBanana-Interact")]
#[command(about = "Rust rewrite of the PaperBanana-Interact CLI (main_multiturn.py equivalent)")]
struct Cli {
    #[arg(long, default_value = "MTPaperBananaBench", help = "dataset to use")]
    dataset_name: String,

    #[arg(long, default_value = "test", help = "dataset split to use")]
    split_name: String,

    #[arg(long, default_value = "gemini-3.1-pro-preview", help = "model used for text generation and reasoning")]
    main_model_name: String,

    #[arg(long, default_value = "gemini-3-pro-image", help = "model used for image generation and generative refinement")]
    image_gen_model_name: String,

    #[arg(long, default_value = "dev_full", help = "PaperBanana experiment mode")]
    exp_mode: String,

    #[arg(long, default_value = "auto", help = "retrieval setting (auto, manual, random, none)")]
    retrieval_setting: String,

    #[arg(long, default_value_t = 3, help = "maximum number of critic rounds during single-turn generation")]
    max_critic_rounds: usize,

    #[arg(long, default_value_t = 64, help = "maximum number of concurrently processed examples and API calls")]
    concurrent_num: usize,

    #[arg(long, help = "path to JSON file containing single-turn results before any simulated feedback")]
    load_initial_result: Option<PathBuf>,

    #[arg(long, help = "path to existing exp directory containing numbered turn files to resume")]
    load_prior_results: Option<PathBuf>,

    #[arg(long, default_value_t = 5, help = "number of simulated feedback iterations after single-turn generation")]
    feedback_turns: usize,

    #[arg(long, default_value = "gemini-3.1-pro-preview", help = "model used to simulate user feedback")]
    user_model_name: String,

    #[arg(long, default_value_t = 1, help = "number of unsatisfied requirements surfaced per feedback turn")]
    n_surfaced_req_per_turn: usize,

    #[arg(long, default_value = "generative", help = "refiner used for simulated feedback turns (generative, pb_directrefine, pb_interact)")]
    refiner: String,

    #[arg(long, default_value_t = 1, help = "maximum internal refinement rounds per feedback turn for pb_directrefine and pb_interact")]
    refiner_internal_critic_rounds: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let cli = Cli::parse();

    if cli.load_initial_result.is_some() && cli.load_prior_results.is_some() {
        eprintln!("--load_initial_result and --load_prior_results are mutually exclusive");
        std::process::exit(1);
    }

    println!("Loaded configuration: {:?}", cli);

    // Placeholder for where multiturn driver logic will be integrated
    println!("Multiturn logic to be implemented here.");

    Ok(())
}
