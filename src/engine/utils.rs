use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn dictionary() -> HashSet<String> {
    let mut dict = HashSet::new();
    let filename = "./src/engine/dictionary.txt";
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        dict.insert(line.unwrap());
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
