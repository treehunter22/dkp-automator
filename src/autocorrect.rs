use std::collections::HashSet;

use textdistance::{self};

pub struct Autocorrecter {
    vocab: HashSet<String>,
}

impl Autocorrecter {
    pub fn new(words: Vec<String>) -> Self {
        let vocab = HashSet::from_iter(words.iter().cloned());

        Self { vocab }
    }

    pub fn add_word(&mut self, word: String) {
        let _ = self.vocab.insert(word);
    }

    pub fn correct(&self, input_word: &str) -> Vec<String> {
        let input_word = input_word.to_lowercase();

        let mut similarities: Vec<(String, f64)> = self
            .vocab
            .iter()
            .map(|v| (v.clone(), textdistance::str::cosine(v, &input_word)))
            .collect();

        similarities.sort_by(|a, b| a.1.total_cmp(&b.1));
        let similarities: Vec<String> = similarities.into_iter().map(|(a, _)| a).rev().collect();

        similarities[..5].to_vec()
    }
}
