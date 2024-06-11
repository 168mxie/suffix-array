use wyhash::wyhash;
use std::collections::HashMap;

pub fn minimizer(
    minimizer_type: &str,
    reference: &str,
    window_size: usize,
    minimizer_size: usize,
) -> usize {
    match minimizer_type {
        "lexicographical" => lexicographical_minimizer(reference, window_size, minimizer_size),
        "hash" => hash_minimizer(reference, window_size, minimizer_size),
        _ => panic!("Invalid minimizer type"),
    }
}

// returns offset position of minimizer given window size, minimizer size
pub fn lexicographical_minimizer(
    reference: &str,
    window_size: usize,
    minimizer_size: usize,
) -> usize {
    let mut min = 0;

    for i in 0..=(window_size - minimizer_size) {
        if &reference[i..i + minimizer_size] < &reference[min..min + minimizer_size] {
            min = i;
        }
    }

    return min;
}

// requires the previous creation of a MPHF object using all possible minimizers as key set
pub fn hash_minimizer(
    reference: &str,
    window_size: usize,
    minimizer_size: usize,
) -> usize {
    let mut min = 0;

    for i in 0..=(window_size - minimizer_size) {
        // hardcoded wyhash seed of 0
        if wyhash(&reference[i..i + minimizer_size].as_bytes(), 0)
            < wyhash(&reference[min..min + minimizer_size].as_bytes(), 0)
        {
            min = i;
        }
    }

    return min;
}

// requires minimizer scheme hashmap that maps minimizer keys to some unique value
pub fn scheme_minimizer(
    reference: &str,
    scheme: &HashMap<&str, u32>,
    window_size: usize,
    minimizer_size: usize,
) -> usize {
    let mut min = 0;

    for i in 0..=(window_size - minimizer_size) {
        if scheme.get(&reference[i..i + minimizer_size])
            < scheme.get(&reference[min..min + minimizer_size])
        {
            min = i;
        }
    }

    return min;
}

// requires hashmap that maps characters to their inverse ranking based on frequency
pub fn char_minimizer(
    reference: &str,
    char_freq_map: &HashMap<char, u32>,
    window_size: usize,
    minimizer_size: usize,
) -> usize {
    let mut min: usize = 0;
    let mut min_val: u32 = (1 << 31) - 1;

    for i in 0..=(window_size - minimizer_size) {
        let test_val = str_to_bin(&reference[i..i + minimizer_size], char_freq_map);

        if test_val < min_val {
            min = i;
            min_val = test_val;
        }
    }

    return min;
}

// requires hashmap that maps characters to their inverse ranking (from 0, 1, 2, 3) based on frequency
pub fn str_to_bin(ascii_str: &str, char_freq_map: &HashMap<char, u32>) -> u32 {
    let mut val: u32 = 0b0;

    for char in ascii_str.chars() {
        let char_val = char_freq_map.get(&char);

        val = val << 2;

        match char_val.unwrap() {
            0 => val |= 0b00,
            1 => val |= 0b01,
            2 => val |= 0b10,
            3 => val |= 0b11,
            _ => panic!("Invalid character value"),
        }
    }

    return val;
}

// preprocessing for minimizer scheme based on (inverse) minimizer frequency
// uses entire reference as input, outputs hashmap of minimizer keys to their inverse ranking
pub fn preprocess_minimizer_scheme(reference: &str, minimizer_size: usize) -> HashMap<&str, u32> {
    let mut scheme: HashMap<&str, u32> = HashMap::new();
    let mut freq_map: HashMap<&str, u32> = HashMap::new();

    // create a frequency hashmap for minimizers
    for i in 0..=(reference.len() - minimizer_size) {
        let count = freq_map
            .entry(&reference[i..i + minimizer_size])
            .or_insert(0);
        *count += 1;
    }

    // sort (in increasing order) by frequency
    let mut tuples: Vec<(&&str, &u32)> = freq_map.iter().collect();
    tuples.sort_by_key(|&(_, v)| *v);

    // assign ranking in scheme to minimizers with lowest frequency
    for i in 0..tuples.len() {
        scheme.insert(tuples[i].0, i as u32);
    }

    return scheme;
}

// preprocessing for minimizer scheme based on (inverse) character frequency
// uses entire reference as input, outputs hashmap of character keys to their inverse ranking
pub fn preprocess_char_scheme(reference: &str) -> HashMap<char, u32> {
    let mut scheme: HashMap<char, u32> = HashMap::new();
    let mut freq_map: HashMap<char, u32> = HashMap::new();

    // create a frequency hashmap for minimizers
    for c in reference.chars() {
        let count = freq_map.entry(c).or_insert(0);
        *count += 1;
    }

    // sort (in increasing order) by frequency
    let mut tuples: Vec<(&char, &u32)> = freq_map.iter().collect();
    tuples.sort_by_key(|&(_, v)| *v);

    // assign ranking in scheme to chars with lowest frequency
    for i in 0..tuples.len() {
        scheme.insert(*tuples[i].0, i as u32);
    }

    return scheme;
}

fn generate_combinations(alphabet: &[char], k: usize) -> Vec<String> {
    let mut combinations = Vec::new();
    generate_combinations_recursive(alphabet, k, &mut combinations, String::new());
    combinations
}

fn generate_combinations_recursive(
    alphabet: &[char],
    k: usize,
    combinations: &mut Vec<String>,
    current: String,
) {
    if k == 0 {
        combinations.push(current);
    } else {
        for &letter in alphabet {
            let mut new_current = current.clone();
            new_current.push(letter);
            generate_combinations_recursive(alphabet, k - 1, combinations, new_current);
        }
    }
}
