use anyhow::Result;
use clap::Parser;
use cli::Cli;
use metrics::MetricsController;

// 2 operazioni
// 1. Put
//    3 versioni: async (che aspetta solo che la tx abbia un valido formato), quella di default che aspetta
//    l'inclusione della tx nel blocco. La terza che attende la finalizzazione del blocco in qui la tx viene messa.
// Sia la put che la get farla in funzione dei byte letti e scritti

// 2. Get
// 2 versioni: get dell'ultimo blocco senza finalizzazione, e con finalizzazione
//
// 1 kb -> 1 MB x 10 campioni incremento di 100 k
// 1 MB -> 10 MB x 10 campioni incremento di 1 mb
// 10 MB -> 100 MB
// 100 MB -> 1 GB

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        cli::Commands::BytesRange { start, end, step } => {
            let metrics_controller = MetricsController::new(start, end, step).await?;

            metrics_controller.compute_metrics().await?;
        }
    }
    Ok(())
}

mod cli;
mod metrics;
pub mod types;
mod utils;
