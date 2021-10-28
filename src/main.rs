use std::{env, fs::File, io::Read, process::exit, time::Instant};

use yarxbi::{lexer, evaluator};

fn read_file(path: &str) -> Result<String, std::io::Error> {
    let mut f = File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;  // `s` contains the contents of "foo.txt"
    Ok(s)
}

fn main() {
    let mut argv = env::args();

    let dur = Instant::now();

    if env::args().len() > 1 {
        let program: String = argv.nth(1).unwrap();
        match read_file(program.as_str()) {
            Ok(s) => {
                let mut code_lines: Vec<lexer::LineOfCode> = Vec::new();

                for (lineno, line) in s.lines().enumerate() {
                    let result = lexer::tokenize_line(line);
                    match result {
                        Ok(x) => code_lines.push(x),
                        Err(e) => {
                            println!("Error at line {}: {}", lineno, e);
                            exit(1);
                        }
                    }
                }

                match evaluator::evaluate(code_lines) {
                    Ok(msg) => println!("{} in {:?}", msg, dur.elapsed()),
                    Err(err) => println!("Execution failed at {:?}:{} because: {}", err.0, err.1, err.2),
                }

            }
            Err(err) => println!("Getting file contents failed with error: {}", err),
        };
    }
}
