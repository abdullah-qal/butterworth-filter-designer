use std::io;

enum Structure {
    Series(f64),
    Shunt(f64),
    Load(f64),
    Line(f64),
}

fn main() {
    let mut input = String::new();
    clearscreen();
    println!(
        "Hello! This is the Butterworth Filter Designer. Please input your filter's order: (1-10)"
    );

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let number: usize = input
        .trim()
        .parse()
        .expect("Please enter a valid number (1-10)");

    if number < 1 || number > 10 {
        println!("Error: Please enter a number between 1 and 10.");
        return;
    }

    let table: [[f64; 11]; 10] = [
        [2.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 
        [1.4142, 1.4142, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 
        [1.0, 2.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 
        [0.7654, 1.8478, 1.8478, 0.7654, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], 
        [0.618, 1.618, 2.0, 1.618, 0.618, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], 
        [0.5176, 1.4142, 1.9318, 1.9318, 1.4142, 0.5176, 1.0, 0.0, 0.0, 0.0, 0.0],
        [0.445, 1.247, 1.8019, 2.0, 1.8019, 1.247, 0.445, 1.0, 0.0, 0.0, 0.0], 
        [0.39, 1.111, 1.6629, 1.9615, 1.9615, 1.6629, 1.111, 0.39, 1.0, 0.0, 0.0], 
        [0.3473, 1.0, 1.5321, 1.8794, 2.0, 1.8794, 1.5321, 1.0, 0.3473, 1.0, 0.0], 
        [0.3129, 0.908, 1.4142, 1.782, 1.9754, 1.782, 1.4142, 0.908, 0.3129, 1.0, 0.0], 
    ];

    let selected_row = &table[number - 1];

    let mut structures = Vec::new();
    for (i, &value) in selected_row.iter().enumerate() {
        if value == 0.0 {
            break;
        }
        if i % 2 == 0 && i != number  {
            structures.push(Structure::Series(value));
        } else if i % 2 != 0 && i != number  {
            structures.push(Structure::Shunt(value));
        }
        else {
            structures.push(Structure::Load(value));
        }
    }

    println!("Element values for filter:");
    for (i, structure) in structures.iter().enumerate() {
        match structure {
            Structure::Series(value) => println!("g{}: Series({})", i + 1, value),
            Structure::Shunt(value) => println!("g{}: Shunt({})", i + 1, value),
            Structure::Load(value) => println!("g{}: Load({})", i + 1, value),
            Structure::Line(_) => println!("Unexpected line"),
        }
    }
}

fn clearscreen() {
    print!("\x1B[H\x1B[2J\x1B[3J");
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
}
