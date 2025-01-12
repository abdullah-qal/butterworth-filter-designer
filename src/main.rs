#![feature(iter_map_windows)]
use std::collections::VecDeque;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Structure {
    Series(f64),
    Shunt(f64),
    Load(f64),
    Line(f64),
}
fn input_generation() -> VecDeque<Structure> {
    let mut input = String::new();
    clearscreen();
    println!(
        "Hello! This is the Butterworth Filter Designer. Please input your filter's order: (1-10)"
    );

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

    let table: [[f64; 11]; 10] = [
        [2.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 
        [1.4142, 1.4142, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 
        [1.0, 2.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 
        [0.7654, 1.8478, 1.8478, 0.7654, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 
        [0.618, 1.618, 2.0, 1.618, 0.618, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], 
        [0.5176, 1.4142, 1.9318, 1.9318, 1.4142, 0.5176, 1.0, 0.0, 0.0, 0.0, 0.0],
        [0.445, 1.247, 1.8019, 2.0, 1.8019, 1.247, 0.445, 1.0, 0.0, 0.0, 0.0], 
        [0.3902, 1.111, 1.6629, 1.9615, 1.9615, 1.6629, 1.111, 0.39, 1.0, 0.0, 0.0], 
        [0.3473, 1.0, 1.5321, 1.8794, 2.0, 1.8794, 1.5321, 1.0, 0.3473, 1.0, 0.0], 
        [0.3129, 0.908, 1.4142, 1.782, 1.9754, 1.9754, 1.7820, 1.4142, 0.9080, 0.3129, 0.0], 
    ];

    let selected_row = &table[number - 1];

    let mut structures = VecDeque::new();
    for (i, &value) in selected_row.iter().enumerate() {
        if value == 0.0 {
            break;
        }
        if i % 2 == 0 && i != number  {
            structures.push_back(Structure::Series(value));
        } else if i % 2 != 0 && i != number  {
            structures.push_back(Structure::Shunt(1.0/value));
        }
        else {
            structures.push_back(Structure::Load(value));
        }
    }

    println!("Element values for filter:");
    for (i, structure) in structures.iter().enumerate() {
        match structure {
            Structure::Series(value) => println!("g{}: Series({:.4})", i + 1, value),
            Structure::Shunt(value) => println!("g{}: Shunt({:.4})", i + 1, value),
            Structure::Load(value) => println!("g{}: Load({:.4})", i + 1, value),
            Structure::Line(_) => println!("Unexpected line"),
        }
    }
    structures
}
fn main() {
    let src_impedance: f64 = 50.0;

    let mut input = input_generation();
    input.pop_back();
    transform_structure(&mut input, src_impedance).expect("F");
    println!("\n The normalised characteristic impedence of the lines are as follows:\n");
    println!("{:.4?}", input);
    println!("\n The actual characteristic impedence of the lines are as follows:\n");
    let mut new_input = Vec::new();
    for i in input {
        match i {
            Structure::Shunt(v) => new_input.push(Structure::Shunt(v*50.0)),
            Structure::Line(v) => new_input.push(Structure::Line(v*50.0)),
            _ => println!("Unexpected value"),
        }
    }
    println!("{:.4?}", new_input)
}



fn process_pair(a: Structure, b: Structure) -> (Structure, Structure) {
    match (a, b) {
        (Structure::Line(a), Structure::Series(b)) => {
            let n : f64 = 1.0 + a/b;
            (Structure::Shunt(a*n), Structure::Line(b*n))
        },
        (Structure::Line(a), Structure::Shunt(b)) => {
            let n : f64 = 1.0 + b/a;
            (Structure::Series(a/n), Structure::Line(b/n))
        }
        (Structure::Shunt(a), Structure::Line(b)) => {
            let n : f64 = 1.0 + a/b; 
            (Structure::Line(a/n), Structure::Series(b/n))
        }
        (Structure::Series(a), Structure::Line(b)) => {
            let n : f64 = 1.0 + b/a; 
            (Structure::Line(n*a), Structure::Shunt(n*b))
        }
        _ => (a, b),
    }
}

#[derive(Debug)]
enum TransformError {
    InvalidInput,
}

fn transform_structure(input: &mut VecDeque<Structure>, _src_impedance: f64) -> Result<(), TransformError> {
    if !all_pairs_lp(input.iter().copied()) {
        return Err(TransformError::InvalidInput);
    }

    if input.is_empty() {
        return Ok(());
    }

    if matches!(input.front().unwrap(), Structure::Shunt(_)) {
        // Case 1:
        //   We have a pattern of `LPLPLP...`
        //   We will shift in Ds from the right one by one.

        // Invariant:
        //   With each iteration of the loop, we will ensure that `input[..finalised]`
        //   is in a finalised state:
        // ```
        // LDLDLPLP
        // ~~~~~^
        //      | finalised
        // ```
        let mut finalised = 1;
        while finalised < input.len() {
            // Here, we shift in a D from the right
            // ```
            // LDLDLPLPD
            //        ~^
            // LDLDLPLDL
            //        ^~
            // ```
            // etc.
            let mut line_position = input.len();
            input.push_back(Structure::Line(1.0));
            while line_position > finalised {
                (input[line_position - 1], input[line_position]) =
                    process_pair(input[line_position - 1], input[line_position]);
                line_position -= 1;
            }
            // We have now changed the state of the input from something like
            // ```
            // LDLDLPLPD
            //      ~~~^
            // ```
            // to something like
            // ```
            // LDLDLDLPL
            //      ^~~~
            // ```
            // hence, shifting the `finalised` up by 2
            finalised += 2;
        }
    } else if matches!(input.back().unwrap(), Structure::Shunt(_)) {
        // Case 2:
        //   We have a pattern of `...PLPLPL`
        //   We will shift in Ds from the left one by one.

        // Invariant:
        //   With each iteration of the loop, we will ensure that `&input[finalised..]`
        //   is in a finalised state:
        // ```
        // PLPLDLDL
        //    ^~~~~
        //    | finalised
        // ```
        let mut finalised = input.len() - 1;
        while finalised > 0 {
            // Here, we shift in a D from the left
            // ```
            // DPLPLDLDL
            // ^~
            // LDLPLDLDL
            // ~^
            // ```
            // etc.
            let mut line_position = 0;
            input.push_front(Structure::Line(1.0));
            // When we push a D on the left, we have to shift `finalised` up by 1
            // ```
            // PLPLDLDL
            //    ^~~~~
            //    | finalised
            // DPLPLDLDL
            // ^  >^~~~~
            //     | finalised
            // ```
            finalised += 1;
            while line_position + 1 < finalised {
                (input[line_position], input[line_position + 1]) =
                    process_pair(input[line_position], input[line_position + 1]);
                line_position += 1;
            }
            // We have now changed the state of the input from something like
            // ```
            // DPLPLDLDL
            // ^~~~
            // ```
            // to something like
            // ```
            // LPLDLDLDL
            // ~~~^
            // ```
            // hence, shifting the `finalised` down by 2
            finalised -= 2;
        }
    } else {
        // Case 3:
        //   We have a pattern of `PL...LP`
        //   We will pretend the first P isn't there,
        //   and apply the strategy of Case 1, shifting in Ds from the right one by one.
        //   We then deal with the first P last.

        // Invariant:
        //   With each iteration of the loop, we will ensure that `input[1..finalised]`
        //   is in a finalised state:
        // ```
        // PLDLDLPLP
        //  ^~~~~^
        //       | finalised
        // ```
        let mut finalised = 2;
        while finalised < input.len() {
            // Here, we shift in a D from the right
            // ```
            // PLDLDLPLPD
            //         ~^
            // PLDLDLPLDL
            //         ^~
            // ```
            // etc.
            let mut line_position = input.len();
            input.push_back(Structure::Line(1.0));
            while line_position > finalised {
                (input[line_position - 1], input[line_position]) =
                    process_pair(input[line_position - 1], input[line_position]);
                line_position -= 1;
            }
            // We have now changed the state of the input from something like
            // ```
            // PLDLDLPLPD
            //       ~~~^
            // ```
            // to something like
            // ```
            // PLDLDLDLPL
            //       ^~~~
            // ```
            // hence, shifting the `finalised` up by 2
            finalised += 2;
        }
        // Now, the state looks like something like
        // ```
        // PLDLDLDLDLDL
        // ```
        // We shift in a D on the left to handle the final P:
        // ```
        // DPLDLDLDLDLDL
        // ^~
        // LDLDLDLDLDLDL
        // ~^
        // ```
        input.push_front(Structure::Line(1.0));
        (input[0], input[1]) = process_pair(input[0], input[1]);
    }

    Ok(())
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