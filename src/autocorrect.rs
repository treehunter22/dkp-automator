use std::collections::{HashMap, HashSet};

use textdistance::{self};

pub struct Autocorrecter {
    vocab: HashSet<String>,
    word_freq_map: HashMap<String, i32>,
}

impl Autocorrecter {
    pub fn new(words: Vec<String>) -> Self {
        let vocab = HashSet::from_iter(words.iter().cloned());
        let word_freq_map = words.iter().cloned().fold(HashMap::new(), |mut map, val| {
            map.entry(val).and_modify(|frq| *frq += 1).or_insert(1);
            map
        });

        Self {
            vocab,
            word_freq_map,
        }
    }

    pub fn correct(&self, input_word: &str) -> Option<Vec<String>> {
        let input_word = input_word.to_lowercase();

        if self.vocab.contains(&input_word) {
            None
        } else {
            let mut similarities: Vec<(String, f64)> = self
                .word_freq_map
                .keys()
                .cloned()
                .map(|v| (v.clone(), 1.0 - textdistance::str::jaccard(&v, &input_word)))
                .collect();

            similarities.sort_by(|(_, a), (_, b)| a.total_cmp(b));
            let similarities: Vec<String> = similarities.into_iter().map(|(a, _)| a).collect();

            let mut result = similarities[..4].to_vec();

            let mut similarities: Vec<(String, f64)> = self
                .word_freq_map
                .keys()
                .cloned()
                .map(|v| (v.clone(), 1.0 - textdistance::str::jaccard(&v, &input_word)))
                .collect();

            similarities.sort_by(|(_, a), (_, b)| a.total_cmp(b));
            let similarities: Vec<String> = similarities.into_iter().map(|(a, _)| a).collect();

            for name in similarities {
                if !result.contains(&name) {
                    result.insert(1, name);
                    break;
                }
            }

            Some(result)
        }
    }

    // def correct(self, input_word):

    //     input_word = input_word.lower()

    //     if input_word in self.V:
    //         return None
    //     else:
    //         similarities = [(v, 1-(textdistance.Jaccard(qval=2).distance(v,input_word))) for v in self.word_freq_dict.keys()]
    //         similarities.sort(key=lambda x: x[1], reverse=True)
    //         similarities = list(map(lambda x: x[0], similarities))

    //         result = similarities[:4]

    //         similarities = [(v, 1-(textdistance.Jaccard(qval=1).distance(v,input_word))) for v in self.word_freq_dict.keys()]
    //         similarities.sort(key=lambda x: x[1], reverse=True)
    //         similarities = list(map(lambda x: x[0], similarities))

    //         for name in similarities:
    //             if name not in result:
    //                 result.insert(1, name)
    //                 break

    //         return result
}
