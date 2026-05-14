use std::collections::HashMap;

use rand::Rng;

use crate::config::Config;

pub type SymbolWeights = HashMap<String, f64>;

pub async fn get_target_weights(config: &Config) -> anyhow::Result<SymbolWeights> {
    if config.symbols.is_empty() {
        anyhow::bail!("SYMBOLS must contain at least one symbol");
    }

    // Deduplicate symbols and generate weights in a single pass to ensure
    // the total matches what will be returned in the HashMap
    let mut raw_weights = HashMap::new();
    let mut rng = rand::thread_rng();
    
    for symbol in &config.symbols {
        // If symbol already exists, this will overwrite it, ensuring deduplication
        raw_weights.insert(symbol.clone(), rng.gen_range(0.0..1.0));
    }

    let total = raw_weights.values().sum::<f64>();

    if total <= 0.0 {
        anyhow::bail!("generated target weights have zero total");
    }

    Ok(raw_weights
        .into_iter()
        .map(|(symbol, weight)| (symbol, weight / total))
        .collect())
}
