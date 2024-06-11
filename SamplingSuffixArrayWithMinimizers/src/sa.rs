use crate::minimizers::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub struct Sequence {
    pub uid: String,
    pub sequence: String,
}

impl Sequence {
    fn new() -> Sequence {
        Sequence {
            uid: String::new(),
            sequence: String::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.uid.is_empty() && self.sequence.is_empty()
    }
}

pub fn read_fasta(filename: &str) -> Vec<Sequence> {
    let mut seqs = Vec::new();
    let mut seq = Sequence::new();
    let buffer = std::fs::read_to_string(filename).expect("Could not open file");

    for line in buffer.lines() {
        if line.starts_with('>') {
            if !seq.is_empty() {
                seq.sequence += "$";
                seqs.push(seq);
                seq = Sequence::new();
            }
            line.split(" ").for_each(|s| {
                if s.starts_with('>') {
                    seq.uid = s[1..].to_string();
                }
            });
        } else {
            seq.sequence += line;
        }
    }
    seq.sequence += "$";
    seqs.push(seq);
    seqs
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Scheme {
    pub scheme: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CharScheme {
    pub scheme: HashMap<char, u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SuffixArray {
    pub sequence: String,
    pub array: Vec<usize>,
    pub buffer: Vec<u8>
}

pub fn longest_common_prefix_length(s1: &str, s2: &str) -> usize {
    let mut lcp = 0;
    for (c1, c2) in s1.chars().zip(s2.chars()) {
        if c1 == c2 {
            lcp += 1;
        } else {
            break;
        }
    }
    lcp
}

pub fn build(
    filename: &str,
    window_size: usize,
    minimizer_size: usize,
    output_filename: &str,
    minimizer_type: &str,
) -> SuffixArray {
    let seqs = read_fasta(filename);

    let mut suffix_tuple: Vec<(usize, &str)> = Vec::new();
    let first_seq = &seqs[0].sequence;

    // let mut minimizers: HashSet<(usize, &str)> = HashSet::new();
    let mut minimizers: Vec<(usize, &str)> = Vec::new();

    let mut scheme: HashMap<&str, u32> = HashMap::new();
    let mut char_scheme: HashMap<char, u32> = HashMap::new();
    if minimizer_type == "scheme" {
        scheme = preprocess_minimizer_scheme(&first_seq[0..&first_seq.len()-1], minimizer_size);
    }
    if minimizer_type == "char" {
        char_scheme = preprocess_char_scheme(&first_seq[0..&first_seq.len()-1]);
    }
    println!("Scheme: {:#?}", char_scheme);

    println!("Seq len: {}", first_seq.len());
    for i in 0..(first_seq.len() - window_size) {
        if i % 1000 == 0 {
            println!("i: {}", i);
        }
        // println!("i: {}", i);
        let mut min = 0;
        if minimizer_type == "scheme" {
            min = scheme_minimizer(&first_seq[i..], &scheme, window_size, minimizer_size);
        } else if minimizer_type == "char"{
            min = char_minimizer( &first_seq[i..], &char_scheme, window_size, minimizer_size);
        } else {
            min = minimizer(minimizer_type, &first_seq[i..], window_size, minimizer_size);
        }
        // let min = 0;
        if minimizers.len() == 0 {
            minimizers.push((min + i, &first_seq[i + min..]));
        } else {
            if minimizers[minimizers.len()-1] != (min + i, &first_seq[i + min..]) {
                minimizers.push((min + i, &first_seq[i + min..]));
            }
        }
        // minimizers.push((min + i, &first_seq[i + min..]));
        // minimizers.insert((min + i, &first_seq[i + min..]));
    }

    println!("Length of minimizers vec: {}", minimizers.len());

    let mut minimizers_vec: Vec<(usize, &str)> =
        minimizers.into_iter().collect::<Vec<(usize, &str)>>();
    minimizers_vec.sort_by(|a, b| a.1.cmp(&b.1));
    // println!("{:#?}", minimizers_vec);

    let sa: Vec<usize> = minimizers_vec.iter().map(|x| x.0).collect();

    let mut buffer: Vec<u8> = Vec::new();
    if scheme.len() > 0 {
        let scheme: HashMap<String, u32> = scheme.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        buffer = bincode::serialize(&Scheme { scheme }).expect("Could not encode as binary.");
    }

    if char_scheme.len() > 0 {
        buffer = bincode::serialize(&CharScheme { scheme: char_scheme }).expect("Could not encode as binary.");
    }

    let output: SuffixArray = SuffixArray {
        sequence: first_seq.to_string(),
        array: sa,
        buffer,
    };

    let encoded = bincode::serialize(&output).expect("Could not encode as binary.");

    std::fs::write(output_filename, encoded).expect("Could not write binary output.");

    output
}

pub fn search(suffix_array: &SuffixArray, query: &str, modifier: &str, offset: usize) -> usize {
    let mut left = 0;
    let mut right = suffix_array.array.len();
    let mut answer = 0;

    let pattern = &query[offset..];

    while left < right {
        let mid = (left + right) / 2;
        let mid_index = suffix_array.array[mid];
        let mid_seq = &suffix_array.sequence[mid_index..];
        // let mid_seq_print = &suffix_array.sequence[mid_index..mid_index+query.len()];
        // println!("Left: {}, Right: {}, Middle: {}", left, right, mid);
        // println!("Query: \t{}", query.to_owned() + modifier);
        // println!("Mid: \t{}", mid_seq_print);
        let cmp = less_than(&(pattern.to_owned() + modifier), mid_seq);
        if cmp == 'l' {
            if mid == left + 1 {
                answer = mid;
                break;
            } else {
                right = mid;
            }
        } else if cmp == 'g' {
            if mid == right - 1 {
                answer = right;
                break;
            } else {
                left = mid;
            }
        }
    }
    // let mut left_index = suffix_array.array[left];
    // println!("Left index: {}", left_index);
    // println!("Answer: {}", answer);
    answer
}

pub fn verify(
    suffix_array: &SuffixArray,
    query: &str,
    start: usize,
    end: usize,
    offset: usize,
) -> Vec<usize> {
    let mut result: Vec<usize> = Vec::new();
    for i in start..end {
        // println!("{}", &query[0..]);
        // println!(
        //     "{}",
        //     &suffix_array.sequence[(&suffix_array.array[i] - offset)
        //         ..(&suffix_array.array[i] - offset + query.len())]
        // );
        if &query[0..offset]
            == &suffix_array.sequence[(&suffix_array.array[i] - offset)..(&suffix_array).array[i]]
        {
            result.push(i);
        } else {
        }
    }
    result
}

fn less_than(s1: &str, s2: &str) -> char {
    for (c1, c2) in s1.chars().zip(s2.chars()) {
        if c1 < c2 {
            return 'l';
        } else if c1 > c2 {
            return 'g';
        }
    }
    'e'
}

