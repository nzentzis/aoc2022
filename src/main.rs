use anyhow::Result;
use std::sync::Arc;

mod grid;
mod util;

macro_rules! problem {
    ($load:path => $input:ty => ()) => {
        type Input = $input;

        pub(crate) const PROBLEM: crate::Problem  = crate::Problem {
            load_input: |d| $load(d).map(|x: $input| -> std::sync::Arc<dyn std::any::Any> {
                std::sync::Arc::new(x)
            }),
            solve1: None,
            solve2: None,
        };
    };
    ($load:path => $input:ty => ($solve1:ident)) => {
        type Input = $input;

        pub(crate) const PROBLEM: crate::Problem = crate::Problem {
            load_input: |d| $load(d).map(|x: Input| -> std::sync::Arc<dyn std::any::Any> {
                std::sync::Arc::new(x)
            }),
            solve1: Some(|input| {
                let input = input.downcast_ref::<Input>().expect("Inconsistent data types");
                ($solve1)(input).map(|x: _| -> Box<dyn std::fmt::Display + Send> {Box::new(x)})
            }),
            solve2: None,
        };
    };
    ($load:path => $input:ty => ($solve1:ident, $solve2:ident)) => {
        type Input = $input;

        pub(crate) const PROBLEM: crate::Problem  = crate::Problem {
            load_input: |d| $load(d).map(|x: Input| -> std::sync::Arc<dyn std::any::Any> {
                std::sync::Arc::new(x)
            }),
            solve1: Some(|input| {
                let input = input.downcast_ref::<Input>().expect("Inconsistent data types");
                ($solve1)(input).map(|x: _| -> Box<dyn std::fmt::Display + Send> {Box::new(x)})
            }),
            solve2: Some(|input| {
                let input = input.downcast_ref::<Input>().expect("Inconsistent data types");
                ($solve2)(input).map(|x: _| -> Box<dyn std::fmt::Display + Send> {Box::new(x)})
            }),
        };
    };
}

macro_rules! problems {
    {$($mod_ident:ident)*} => {
        $(
            mod $mod_ident ;
        )*
        const PROBLEMS: &[Problem] = &[$($mod_ident::PROBLEM),*];
    };
}

type Solver = fn(Arc<dyn std::any::Any>) -> Result<Box<dyn std::fmt::Display + Send>>;

struct Problem {
    load_input: fn(&mut dyn std::io::BufRead) -> Result<std::sync::Arc<dyn std::any::Any>>,
    solve1: Option<Solver>,
    solve2: Option<Solver>,
}

fn main() {
    let mut args = std::env::args().skip(1);
    if let Some(prob) = args.next() {
        // parse problem number
        let prob_number = match prob.parse::<usize>() {
            Ok(0) => {
                eprintln!("error: Problem numbers are 1-based. Use #1 for the first problem.");
                std::process::exit(1);
            }
            Ok(x) => x,
            Err(_) => {
                eprintln!("unable to parse problem number");
                std::process::exit(1);
            }
        };
        let prob_idx = prob_number - 1;

        let Some(problem) = PROBLEMS.get(prob_idx) else {
            eprintln!("invalid problem number");
            std::process::exit(1);
        };

        // open input
        let mut input: Box<dyn std::io::BufRead> = match args.next().as_deref() {
            None => {
                let input = std::path::Path::new("inputs").join(format!("{:02}", prob_number));
                let input = match std::fs::File::open(input) {
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("{:02}: Failed to open input: {}", prob_number, e);
                        std::process::exit(1);
                    }
                };

                Box::new(std::io::BufReader::new(input))
            },
            Some("-") => {
                Box::new(std::io::BufReader::new(std::io::stdin()))
            },
            Some(name) => {
                let input = match std::fs::File::open(name) {
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("{:02}: Failed to open input: {}", prob_number, e);
                        std::process::exit(1);
                    }
                };

                Box::new(std::io::BufReader::new(input))
            },
        };

        let input = match (problem.load_input)(&mut input) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{:02}: Failed to load input: {}", prob_number, e);
                std::process::exit(1);
            }
        };

        if let Some(p1) = problem.solve1 {
            match (p1)(Arc::clone(&input)) {
                Ok(x) => {
                    println!("{:02}p1: {}", prob_number, x);
                }
                Err(e) => {
                    eprintln!("{:02}: Part 1 failed: {}", prob_number, e);
                }
            }
        }
        if let Some(p2) = problem.solve2 {
            match (p2)(input) {
                Ok(x) => {
                    println!("{:02}p2: {}", prob_number, x);
                }
                Err(e) => {
                    eprintln!("{:02}: Part 2 failed: {}", prob_number, e);
                }
            }
        }
    } else {
        let begin = std::time::Instant::now();
        for (idx, prob) in PROBLEMS.iter().enumerate() {
            let p_num = idx + 1;

            let input = std::path::Path::new("inputs").join(format!("{:02}", p_num));
            let input = match std::fs::File::open(input) {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("{:02}: Failed to open input: {}", p_num, e);
                    continue;
                }
            };
            let mut input = std::io::BufReader::new(input);

            let input = match (prob.load_input)(&mut input) {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("{:02}: Failed to load input: {}", p_num, e);
                    continue;
                }
            };

            if let Some(p1) = prob.solve1 {
                if let Err(e) = (p1)(Arc::clone(&input)).map(std::hint::black_box) {
                    eprintln!("{:02}: Part 1 failed: {}", p_num, e);
                }
            }
            if let Some(p2) = prob.solve2 {
                if let Err(e) = (p2)(input).map(std::hint::black_box) {
                    eprintln!("{:02}: Part 2 failed: {}", p_num, e);
                }
            }
        }
        let end = std::time::Instant::now();
        let dur = end.duration_since(begin);
        println!("Solved {} problems in {} ms", PROBLEMS.len(), dur.as_millis());
    }
}

problems! {
    p01 p02 p03 p04 p05 p06 p07
    p08 p09 p10 p11 p12 p13 p14
    p15 p16 p17 p18 p19 p20 p21
    p22 p23 p24 p25
}
