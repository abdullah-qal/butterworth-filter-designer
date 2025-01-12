#![feature(iter_map_windows)]
use std::io;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Structure {
    Series(f64),
    Shunt(f64),
    Load(f64),
    Line(f64),
}

fn input_generation() -> Vec<Structure> {
    let mut input = String::new();
    clearscreen();
    println!("Hello! This is the Butterworth Filter Designer. Please input your filter's order: (1-10)");

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read Shunt");

    let number: usize = input
        .trim()
        .parse()
        .expect("Please enter a valid number (1-10)");

    if number < 1 || number > 10 {
        println!("Error: Please enter a number between 1 and 10.");
        panic!();
    }

    const TABLE: &[&[f64]] = &[
        &[2.0000, 1.0000],
        &[1.4142, 1.4142, 1.0000],
        &[1.0000, 2.0000, 1.0000, 1.0000],
        &[0.7654, 1.8478, 1.8478, 0.7654, 1.0000],
        &[0.6180, 1.6180, 2.0000, 1.6180, 0.6180, 1.0000],
        &[0.5176, 1.4142, 1.9318, 1.9318, 1.4142, 0.5176, 1.0000],
        &[0.4450, 1.2470, 1.8019, 2.0000, 1.8019, 1.2470, 0.4450, 1.0000],
        &[0.3902, 1.1110, 1.6629, 1.9615, 1.9615, 1.6629, 1.1110, 0.3900, 1.0000],
        &[
            0.3473, 1.0000, 1.5321, 1.8794, 2.0000, 1.8794, 1.5321, 1.0000, 0.3473, 1.0000,
        ],
        &[
            0.3129, 0.9080, 1.4142, 1.7820, 1.9754, 1.9754, 1.7820, 1.4142, 0.9080, 0.3129, 1.0000,
        ],
    ];

    let selected_row = TABLE[number - 1];

    let mut structures = Vec::new();
    for (i, &value) in selected_row.iter().enumerate() {
        if i % 2 == 0 && i != number {
            structures.push(Structure::Series(value));
        } else if i % 2 != 0 && i != number {
            structures.push(Structure::Shunt(1.0 / value));
        } else {
            structures.push(Structure::Load(value));
        }
    }

    println!("Element values for filter:");
    for (i, structure) in structures.iter().enumerate() {
        match structure {
            Structure::Series(value) => println!("g{}: Series({:.4})", i + 1, value),
            Structure::Shunt(value) => println!("g{}: Shunt({:.4})", i + 1, value),
            Structure::Load(value) => println!("g{}: Load({:.4})", i + 1, value),
            Structure::Line(_) => unreachable!("Unexpected line"),
        }
    }
    structures
}
fn main() {
    let src_impedance: f64 = 50.0;

    let mut input = input_generation();
    input.pop();
    let mut transformed = transform_structure(&input, src_impedance).expect("F");
    println!("\n The normalised characteristic impedence of the lines are as follows:\n");
    println!("{:.4?}", transformed);
    println!("\n The actual characteristic impedence of the lines are as follows:\n");
    for structure in transformed.iter_mut() {
        match structure {
            Structure::Shunt(v) => *v *= 50.0,
            Structure::Line(v) => *v *= 50.0,
            _ => panic!("Unexpected value"),
        }
    }
    println!("{:.4?}", transformed);
}

fn process_pair(a: Structure, b: Structure) -> (Structure, Structure) {
    match (a, b) {
        (Structure::Line(a), Structure::Series(b)) => {
            let n: f64 = 1.0 + a / b;
            (Structure::Shunt(a * n), Structure::Line(b * n))
        }
        (Structure::Line(a), Structure::Shunt(b)) => {
            let n: f64 = 1.0 + b / a;
            (Structure::Series(a / n), Structure::Line(b / n))
        }
        (Structure::Shunt(a), Structure::Line(b)) => {
            let n: f64 = 1.0 + a / b;
            (Structure::Line(a / n), Structure::Series(b / n))
        }
        (Structure::Series(a), Structure::Line(b)) => {
            let n: f64 = 1.0 + b / a;
            (Structure::Line(n * a), Structure::Shunt(n * b))
        }
        _ => (a, b),
    }
}

#[derive(Debug)]
enum TransformError {
    InvalidInput,
}

fn transform_structure(input: &[Structure], _src_impedance: f64) -> Result<Vec<Structure>, TransformError> {
    if !all_pairs_lp(input.iter().copied()) {
        return Err(TransformError::InvalidInput);
    }

    if input.is_empty() {
        return Ok(Default::default());
    }

    let shunt_count = input
        .iter()
        .filter(|structure| matches!(structure, Structure::Shunt(_)))
        .count();
    let mut goodness = vec![0.0; shunt_count];

    if shunt_count == 0 {
        assert_eq!(input.len(), 1);
        let (shunt, line) = process_pair(Structure::Line(1.0), *input.first().unwrap());
        return Ok(vec![shunt, line]);
    }

    let first_shunt = match input.first().unwrap() {
        Structure::Shunt(_) => 0,
        Structure::Series(_) => 1,
        _ => unreachable!(),
    };
    let mut transformed = vec![Structure::Line(0.0); 2 * input.len() - 1];

    let loop_body = |i: usize, transformed: &mut [Structure]| {
        // initial setup
        for (ix_input, ix_transformed) in (0..transformed.len()).step_by(2).enumerate() {
            transformed[ix_transformed] = input[ix_input];
        }

        let cutoff_shunt = first_shunt + 2 * i;
        // left side
        {
            let mut finalised = 2 * cutoff_shunt;
            while finalised > 0 {
                let mut line_position = 1;
                (transformed[line_position - 1], transformed[line_position]) =
                    process_pair(Structure::Line(1.0), transformed[line_position - 1]);
                while line_position + 2 < finalised {
                    (transformed[line_position + 1], transformed[line_position + 2]) =
                        process_pair(transformed[line_position], transformed[line_position + 1]);
                    line_position += 2;
                }
                finalised -= 2;
            }
        }
        // right side
        {
            let mut finalised = 2 * cutoff_shunt;
            while finalised < transformed.len() - 1 {
                let mut line_position = transformed.len() - 2;
                (transformed[line_position], transformed[line_position + 1]) =
                    process_pair(transformed[line_position + 1], Structure::Line(1.0));
                while finalised < line_position - 2 {
                    (transformed[line_position - 2], transformed[line_position - 1]) =
                        process_pair(transformed[line_position - 1], transformed[line_position]);
                    line_position -= 2;
                }
                finalised += 2;
            }
        }
    };

    for i in 0..shunt_count {
        loop_body(i, &mut transformed);

        for &structure in &transformed {
            goodness[i] += match structure {
                Structure::Series(v) => v,
                Structure::Shunt(v) => v,
                Structure::Line(v) => v,
                _ => unreachable!(),
            }
            .powf(2.0);
        }
    }

    let best = goodness
        .iter()
        .enumerate()
        .min_by(|&(_, g1), &(_, g2)| f64::total_cmp(g1, g2))
        .map(|(i, _)| i)
        .unwrap();
    loop_body(best, &mut transformed);

    Ok(transformed)
}

fn all_pairs_lp(iter: impl Iterator<Item = Structure>) -> bool {
    iter.map_windows(|[a, b]| {
        matches!(
            (a, b),
            (Structure::Series(_), Structure::Shunt(_)) | (Structure::Shunt(_), Structure::Series(_))
        )
    })
    .all(std::convert::identity)
}

fn clearscreen() {
    print!("\x1B[H\x1B[2J\x1B[3J");
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
}
