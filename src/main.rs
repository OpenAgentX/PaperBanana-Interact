mod llm;
mod utils;
mod prompts;
mod agents;
mod refiners;
mod session;

use clap::{Parser, Subcommand};
use anyhow::Result;
use dotenvy::dotenv;

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
    },
    Resume {
        #[arg(short, long)]
        prompt: String,

        #[arg(short, long)]
        session: String,

        #[arg(short, long)]
        image: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok(); // load .env file if present

    let cli = Cli::parse();
    let refiner = PbInteractRefiner::new("gemini-3.1-pro-preview");

    match &cli.command {
        Commands::Start { prompt, output, image } => {
            println!("Starting new session with prompt: {}", prompt);

            let initial_b64 = if let Some(img_path) = image {
                read_image_to_base64(img_path)?
            } else {
                String::new()
            };

            let refined_image = refiner.refine(&initial_b64, prompt, "").await?;

            let session = Session {
                history: format!("User: {}\n", prompt),
                current_image_b64: refined_image,
            };

            session.save_to_file(output)?;
            println!("Session saved to {}", output);
        }
        Commands::Resume { prompt, session, image } => {
            println!("Resuming session from {}", session);

            let mut state = Session::load_from_file(session)?;

            let input_b64 = if let Some(img_path) = image {
                read_image_to_base64(img_path)?
            } else {
                state.current_image_b64.clone()
            };

            let refined_image = refiner.refine(&input_b64, prompt, &state.history).await?;

            state.history.push_str(&format!("User: {}\n", prompt));
            state.current_image_b64 = refined_image;

            state.save_to_file(session)?;
            println!("Session updated at {}", session);
        }
    }

    Ok(())
}
