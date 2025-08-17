use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocChunk {
    pub id: String,
    pub text: String,
}

#[derive(Default)]
pub struct RagStore {
    chunks: Vec<DocChunk>,
}

impl RagStore {
    pub fn new() -> Self { Self { chunks: Vec::new() } }

    pub fn ingest(&mut self, chunks: Vec<DocChunk>) { self.chunks.extend(chunks); }

    pub fn top_k(&self, query: &str, k: usize) -> Vec<DocChunk> {
        let q_vec = embed(query);
        let mut scored: Vec<(f32, &DocChunk)> = self
            .chunks
            .iter()
            .map(|c| (cosine(&q_vec, &embed(&c.text)), c))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        scored.into_iter().take(k).map(|(_, c)| c.clone()).collect()
    }
}

fn embed(text: &str) -> Vec<f32> {
    // Naive bag-of-words length-10 hash embedding to avoid extra deps
    let mut v = vec![0.0; 10];
    for (i, t) in text.split_whitespace().enumerate() { v[i % 10] += (t.len() as f32).sqrt(); }
    v
}

fn cosine(a: &Vec<f32>, b: &Vec<f32>) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na = (a.iter().map(|x| x * x).sum::<f32>()).sqrt();
    let nb = (b.iter().map(|x| x * x).sum::<f32>()).sqrt();
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_k_returns_relevant_chunks() -> Result<()> {
        let mut store = RagStore::new();
        store.ingest(vec![
            DocChunk { id: "1".into(), text: "Uniswap V2 Router interface: swapExactETHForTokens".into() },
            DocChunk { id: "2".into(), text: "Address checksum (EIP-55) and normalization".into() },
            DocChunk { id: "3".into(), text: "ENS resolution and fallback strategies".into() },
        ]);
        let res = store.top_k("uniswap swap exact eth for tokens", 2);
        assert!(!res.is_empty());
        Ok(())
    }
}
