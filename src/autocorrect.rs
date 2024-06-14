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
            .map(|v| (v.clone(), 1.0 - textdistance::str::jaccard(v, &input_word)))
            .collect();

        similarities.sort_by(|(_, a), (_, b)| a.total_cmp(b));
        let similarities: Vec<String> = similarities.into_iter().map(|(a, _)| a).collect();

        let mut result = similarities[..4].to_vec();

        let mut similarities: Vec<(String, f64)> = self
            .vocab
            .iter()
            .map(|v| (v.clone(), 1.0 - textdistance::str::jaccard(v, &input_word)))
            .collect();

        similarities.sort_by(|(_, a), (_, b)| a.total_cmp(b));
        let similarities: Vec<String> = similarities.into_iter().map(|(a, _)| a).collect();

        for name in similarities {
            if !result.contains(&name) {
                result.insert(1, name);
                break;
            }
        }

        result
    }
}
