mod llm;
mod utils;
mod prompts;
mod agents;
mod refiners;
mod session;

use clap::{Parser, Subcommand};
use anyhow::Result;
use dotenvy::dotenv;

use agents::base_agent::AgentConfig;
use refiners::pb_interact::PbInteractRefiner;
use session::Session;
use utils::image::read_image_to_base64;

#[derive(Parser)]
#[command(name = "PaperBanana-Interact")]
#[command(about = "Rust rewrite of the PaperBanana-Interact CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start {
        #[arg(short, long)]
        prompt: String,

        #[arg(short, long)]
        output: String,

        #[arg(short, long)]
        image: Option<String>,

        #[arg(long, default_value = "gemini-3.1-pro-preview")]
        model: String,
    },
    Resume {
        #[arg(short, long)]
        prompt: String,

        #[arg(short, long)]
        session: String,

        #[arg(short, long)]
        image: Option<String>,

        #[arg(long, default_value = "gemini-3.1-pro-preview")]
        model: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Start { prompt, output, image, model } => {
            println!("Starting new session with prompt: {}", prompt);

            let config = AgentConfig {
                main_model_name: model.clone(),
                image_gen_model_name: model.clone(),
                temperature: 0.7,
                dataset_dir: "data/MTPaperBananaBench".to_string(),
                work_dir: ".".to_string(),
            };
            let refiner = PbInteractRefiner::new(config);

            let initial_b64 = if let Some(img_path) = image {
                read_image_to_base64(img_path)?
            } else {
                String::new()
            };

            let method_section = "Initial methodological context.";
            let figure_caption = "Target figure logic.";

            let (refined_image, new_desc) = refiner.refine(&initial_b64, "", prompt, "", method_section, figure_caption).await?;

            let session = Session {
                history: format!("User: {}\n", prompt),
                current_image_b64: refined_image,
                detailed_description: new_desc,
            };

            session.save_to_file(output)?;
            println!("Session saved to {}", output);
        }
        Commands::Resume { prompt, session, image, model } => {
            println!("Resuming session from {}", session);

            let config = AgentConfig {
                main_model_name: model.clone(),
                image_gen_model_name: model.clone(),
                temperature: 0.7,
                dataset_dir: "data/MTPaperBananaBench".to_string(),
                work_dir: ".".to_string(),
            };
            let refiner = PbInteractRefiner::new(config);

            let mut state = Session::load_from_file(session)?;

            let input_b64 = if let Some(img_path) = image {
                read_image_to_base64(img_path)?
            } else {
                state.current_image_b64.clone()
            };

            let method_section = "Initial methodological context.";
            let figure_caption = "Target figure logic.";

            let (refined_image, new_desc) = refiner.refine(&input_b64, &state.detailed_description, prompt, &state.history, method_section, figure_caption).await?;

            state.history.push_str(&format!("User: {}\n", prompt));
            state.current_image_b64 = refined_image;
            state.detailed_description = new_desc;

            state.save_to_file(session)?;
            println!("Session updated at {}", session);
        }
    }

    Ok(())
}
