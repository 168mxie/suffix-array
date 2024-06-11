mod minimizers;
mod sa;
use crate::minimizers::*;
use crate::sa::*;
use std::char;
use std::collections::HashMap;
use std::fmt::format;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // let query = "ATTAGACATATTACAGA";
    // let input = "abracadabracada";
    // input file contains sequence: abracadabracada
    let input_file = "samonella_sub.fa";
    let query_file = "reads_sal_sub.fq";
    
    // SET MINIMIZER TYPE HERE
    let minimizer_type = "char";

    // let window_size = 5;
    // let minimizer_size = 5;

    let window_sizes = vec![4, 8];
    let minimizer_sizes: Vec<f64> = vec![(1.0/4.0), (1.0/2.0), (3.0/4.0)];
    let mut construction_times = Vec::new();

    for window_size in &window_sizes {
        for minimizer_size in &minimizer_sizes {
            let minimizer_size = (*minimizer_size * *window_size as f64) as usize;
            let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let sa = build(input_file, *window_size, minimizer_size, "output", &minimizer_type);
            let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let construction_time = end - start;
            println!("Construction Time: {} ms", construction_time.as_millis());
            construction_times.push((
                format!("ws: {}, ms: {}", window_size, minimizer_size),
                construction_time,
            ));

            let seqs = read_fasta(&query_file);
            let mut results: Vec<String> = Vec::new();
            let output_file_size = std::fs::metadata("output").unwrap().len();
            results.push(format!("Construction Time: {} ms", construction_time.as_millis()));
            results.push(format!("SA Size: {} bytes", output_file_size));
            results.push(format!("Query Time: {} ms", "TBD"));

            let sa2 = bincode::deserialize::<SuffixArray>(&std::fs::read("output").unwrap()).unwrap();
            let mut scheme_str: HashMap<String, u32> = HashMap::new();
            let mut scheme: HashMap<&str, u32> = HashMap::new();
            let mut char_scheme: HashMap<char, u32> = HashMap::new();
            if minimizer_type == "scheme" {
                scheme_str = bincode::deserialize::<Scheme>(&sa2.buffer).unwrap().scheme;
                scheme = scheme_str.iter().map(|(k, v)| (k.as_str(), *v)).collect();
            }
            if minimizer_type == "char" {
                char_scheme = bincode::deserialize::<CharScheme>(&sa2.buffer).unwrap().scheme;
            }
           

            let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            for seq in seqs {
                let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let queryf = &seq.sequence;
                let query = &queryf[0..queryf.len() - 1];
                // println!("Query: {}", &query);
                let mut query_min = 0;
                if minimizer_type == "scheme" {
                    query_min = scheme_minimizer(&query[0..*window_size], &scheme, *window_size, minimizer_size);
                } else if minimizer_type == "char"{
                    query_min = char_minimizer( &query[0..*window_size], &char_scheme, *window_size, minimizer_size);
                } else {
                    query_min = minimizer(minimizer_type, &query[0..*window_size], *window_size, minimizer_size);
                }
                // println!(
                //     "minimizer: {} {}",
                //     query_min,
                //     &query[query_min..query_min + *minimizer_size]
                // );
                let st = search(&sa, &query, "#", query_min);

                let end = search(&sa, &query, "}", query_min);
                // println!("st: {} end: {}", st, end);
                let verified = verify(&sa, query, st, end, query_min);
                // println!("ws: {}, ms: {}", window_size, minimizer_size);
                let mut output: Vec<String> = Vec::new();
                for i in verified {
                    // println!("{} {:?}", &seq.uid, &sa.sequence[sa.array[i] - query_min..]);
                    output.push(format!("{}", sa.array[i] - query_min));
                }
                let output_string = output.join("\t");
                let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let construction_time = end - start;
                results.push(format!("{} {} Query Time: {} micros", &seq.uid, output_string, construction_time.as_micros()));
               
            }
            let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            let construction_time = end - start;
            results[2] = format!("Query Time: {} ms", construction_time.as_millis());
            // println!("Construction Time: {} ms", construction_time.as_millis());
            let result_string = results.join("\n");
            std::fs::write(format!("{}/ws{}ms{}q{}", minimizer_type, window_size, minimizer_size, "samonella"), result_string).expect("Could not write output.");

           
            // println!("{}", &test[min..min + minimizer_size]);
        }
    }
}
