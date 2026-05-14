use std::collections::HashMap;

use rand::Rng;

use crate::config::Config;

pub type SymbolWeights = HashMap<String, f64>;

pub async fn get_target_weights(config: &Config) -> anyhow::Result<SymbolWeights> {
    if config.symbols.is_empty() {
        anyhow::bail!("SYMBOLS must contain at least one symbol");
    }

    let raw_weights = {
        let mut rng = rand::thread_rng();
        config
            .symbols
            .iter()
            .map(|symbol| (symbol.clone(), rng.gen_range(0.0..1.0)))
            .collect::<Vec<_>>()
    };

    let total = raw_weights.iter().map(|(_, weight)| weight).sum::<f64>();

    if total <= 0.0 {
        anyhow::bail!("generated target weights have zero total");
    }

    Ok(raw_weights
        .into_iter()
        .map(|(symbol, weight)| (symbol, weight / total))
        .collect())
}
