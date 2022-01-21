use crate::engine::words::dictionary_words;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};

pub fn dictionary() -> HashSet<String> {
    let mut dict = HashSet::new();
    for w in dictionary_words() {
        dict.insert(w);
    }
    dict
}

pub fn get_random_word() -> String {
    let dict = dictionary();
    let list = Vec::from_iter(dict.iter());
    list.choose(&mut rand::thread_rng()).unwrap().to_string()
}

pub fn build_letter_counts(word: &str) -> HashMap<char, usize> {
    let mut counts = HashMap::new();
    for c in word.chars() {
        match counts.get_mut(&c) {
            Some(v) => *v += 1,
            None => {
                counts.insert(c, 1);
                ()
            }
        };
    }
    counts
}
