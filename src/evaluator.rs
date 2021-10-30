use crate::{lexer, token, value};

use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    io,
    iter::Peekable,
    slice::Iter,
};

#[derive(Debug)]
struct ForLoop {
    line_no: lexer::LineNumber, // Jump?
    pos: u32, // Token position of 'to'
    slide: bool, // Condition symbol (gt/lt)
    stes: bool // Try custom step?
}

#[derive(Debug)]
struct WhileLoop {
    line_no: lexer::LineNumber,
    pos: u32
}

#[derive(Debug)]
struct Context {
    variables: HashMap<String, value::Value>, // Variables
    floops: HashMap<String, ForLoop>, // For loops
    wloops: Vec<WhileLoop>,
}

impl Context {
    fn new() -> Context {
        Context {
            variables: HashMap::new(),
            floops: HashMap::new(),
            wloops: Vec::new(),
        }
    }
}

macro_rules! err {
    ($line:ident, $pos:expr, $fmt:expr $(, $p:expr ) *) => {
        return Err((**$line, $pos, format!($fmt, $($p),*)))
    }
}

macro_rules! get_variable {
    ($ctx:ident, $var:expr, $line:ident, $pos:expr) => {
        match $ctx.variables.get($var) {
            Some(value) => value,
            None => err!($line, $pos, "Invalid variable expression {}", $var),
        }
    }
}

pub fn evaluate(code_lines: Vec<lexer::LineOfCode>) -> Result<String, (lexer::LineNumber, u32, String)> {
    let mut context = Context::new();
    let mut lineno_to_code = BTreeMap::new();
    let mut line_map = BTreeMap::new();

    for (index, line) in code_lines.iter().enumerate() {
        line_map.insert(&line.line_number, index);
        lineno_to_code.insert(&line.line_number, &line.tokens);
    }

    let line_numbers: Vec<_> = line_map.keys().clone().collect();
    let num_lines = line_numbers.len();
    let mut line_index = 0;
    // TODO: Feels hacky
    let mut line_has_goto = false;

    loop {
        let line_number = line_numbers[line_index];
        let tokens = &lineno_to_code[line_number];
        let mut token_iter = tokens.iter().peekable();

        // println!("Looking at line: {:?}", line_number);
        if !tokens.is_empty() {
            let lexer::TokenAndPos(pos, ref token) = *token_iter.next().unwrap();
            // Set default value
            line_has_goto = false;

            match evaluate_com(&mut context,
                        &lineno_to_code,
                        &line_map,
                        &mut line_index,
                        &mut line_has_goto,
                        token_iter,
                        line_number,
                        pos,
                        token
            ) {
                Ok(_) => {},
                Err(e) => return Err(e),
            };
        }

        if !line_has_goto {
            line_index += 1;
            if line_index == num_lines {
                break;
            }
        }
    }

    Ok("\nExecuted successfully".to_string())
}

fn evaluate_com(
    context: &mut Context,
    lineno_to_code: &BTreeMap<&lexer::LineNumber, &Vec<lexer::TokenAndPos>>,
    line_map: &BTreeMap<&lexer::LineNumber, usize>,
    line_index: &mut usize,
    line_has_goto: &mut bool,
    mut token_iter: Peekable<Iter<'_, lexer::TokenAndPos>>,
    line_number: &&lexer::LineNumber,
    pos: u32,
    token: &token::Token,
) -> Result<String, (lexer::LineNumber, u32, String)> {

    match *token {
        token::Token::Rem => {},

        token::Token::Goto => {
            *line_has_goto = true;
            match token_iter.next() {
                Some(&lexer::TokenAndPos(pos, token::Token::Number(number))) => {
                    let n = lexer::LineNumber(number as u32);
                    match line_map.get(&n) {
                        Some(index) => *line_index = *index,
                        _ => err!(line_number, pos, "Invalid target line for GOTO")
                    }
                }
                
                Some(&lexer::TokenAndPos(pos, _)) => err!(line_number, pos, "GOTO must be followed by a valid line number"),
                
                None => err!(line_number, pos + 4, "GOTO must be followed by a line number"),
            }
        }

        token::Token::Let => {
            // Expected Next:
            // Variable Equals EXPRESSION
            match (
                token_iter.next(),
                token_iter.next(),
                parse_and_eval_expression(&mut token_iter, &context),
            ) {
                (
                    Some(&lexer::TokenAndPos(_, token::Token::Variable(ref variable))),
                    Some(&lexer::TokenAndPos(_, token::Token::Equals)),
                    Ok(ref value),
                ) => {
                    context
                        .variables
                        .insert(variable.to_string(), value.clone());
                }

                (_, _, Err(e)) => err!(line_number, pos, "Error in LET expression: {}", e),

                _ => err!(line_number, pos, "Invalid syntax for LET"),
                
            }
        }

        token::Token::Print => {
            // Expected Next:
            // EXPRESSION
            match parse_and_eval_expression(&mut token_iter, &context) {
                Ok(value::Value::String(value)) => println!("{}", value),
                Ok(value::Value::Number(value)) => println!("{}", value),
                Ok(value::Value::Bool(value)) => println!("{}", value),
                Err(_) => err!(line_number, pos, "PRINT must be followed by valid expression"),
            }
        }

        token::Token::Input => {
            match token_iter.next() {
                Some(&lexer::TokenAndPos(_, token::Token::Variable(ref variable))) => {
                    let mut input = String::new();

                    io::stdin()
                        .read_line(&mut input)
                        .expect("failed to read line");
                    input = input.trim().to_string();
                    let value = value::Value::String(input);

                    // Store the string now, can coerce to number later if needed
                    // Can overwrite an existing value
                    context
                        .variables
                        .entry(variable.to_string())
                        .or_insert(value);
                }

                _ => err!(line_number, pos + 5, "INPUT must be followed by a variable name"),
            }
        }

        token::Token::If => {
            // Expected Next:
            // EXPRESSION Then Number
            // Where Number is a Line Number
            match (
                parse_and_eval_expression(&mut token_iter, &context),
                token_iter.next(),
                token_iter.next(),
            ) {
                (
                    Ok(value::Value::Bool(ref value)),
                    Some(&lexer::TokenAndPos(_, token::Token::Then)),
                    Some(&lexer::TokenAndPos(_, token::Token::Number(ref number))),
                ) => {
                        if *value {
                            *line_has_goto = true;
                            let n = lexer::LineNumber(*number as u32);
                            match line_map.get(&n) {
                                Some(index) => *line_index = *index,
                                _ => err!(line_number, pos, "Invalid target line for IF"),
                                }
                            }
                    }
                
                _ => err!(line_number, pos, "Invalid syntax for IF"),
            }
        }

        token::Token::For => {
            // Expected Next:
            // Variable equals EXPRESSION to Number step Number
            match (
                token_iter.next(),
                token_iter.next(),
                parse_and_eval_expression(&mut token_iter, &context),
            ) {
                (
                    Some(&lexer::TokenAndPos(_, token::Token::Variable(ref variable))),
                    Some(&lexer::TokenAndPos(_, token::Token::Equals)),
                    Ok(value::Value::Number(ref start)),
                ) => {
                    context
                        .variables
                        .insert(variable.to_string(), value::Value::Number(*start));

                    match (
                        token_iter.next(),
                        parse_and_eval_expression(&mut token_iter, &context),
                    ) {
                        (
                            Some(&lexer::TokenAndPos(epos, token::Token::To)),
                            Ok(value::Value::Number(ref end)),
                        ) => {
                            let stes = match token_iter.next() {
                                Some(&lexer::TokenAndPos(_, token::Token::Step)) => {
                                    match parse_and_eval_expression(&mut token_iter, &context) {
                                        Ok(value::Value::Number(ref _step)) => true,
                                        _ => err!(line_number, pos, "Cannot parse FOR step"),
                                    }
                                },
                                _ => false,
                            };

                            context
                                .floops
                                .insert(variable.to_string(), ForLoop {
                                    line_no: **line_number,
                                    pos: epos,
                                    slide: *start < *end,
                                    stes: stes});
                        },

                        _ => err!(line_number, pos, "Cannot parse secondary FOR expression"),
                    }
                }

                _ => err!(line_number, pos, "Cannot parse FOR initialisation expression"),
            }
        }

        token::Token::Next => {
            match token_iter.next() {
                Some(&lexer::TokenAndPos(_, token::Token::Variable(ref variable))) => {
                    let floop = match context
                        .floops
                        .get(variable) {
                            Some(floop) => floop,
                            None => err!(line_number, pos, "Cannot get FOR signature from hashmap"),
                    };
                    
                    let mut ftok_iter = &mut lineno_to_code[&floop.line_no]
                        .iter()
                        .peekable();

                    while ftok_iter.peek().unwrap().0 != floop.pos {
                        ftok_iter.next();
                    }

                    ftok_iter.next();
                    let end = match parse_and_eval_expression(&mut ftok_iter, &context) {
                        Ok(value::Value::Number(value)) => value,
                        _ => err!(line_number, pos, "Cannot parse end for FOR"),
                    };
                
                    let step = if floop.stes {
                        match parse_and_eval_expression(&mut token_iter, &context) {
                            Ok(value::Value::Number(value)) => value,
                            _ => err!(line_number, pos, "Cannot parse step for FOR"),
                        }
                    }
                    else {
                        if floop.slide { 1 as f64 } else { -1 as f64 }
                    };

                    let next = *match get_variable!(context, variable, line_number, pos) {
                        value::Value::Number(value) => value,
                        _ => err!(line_number, pos, "Cannot parse variable for jump"),
                    } + step;
                    
                    if if floop.slide { next < end } else { next > end } {
                        context
                            .variables
                            .insert(variable.to_string(), value::Value::Number(next));
                        
                        match line_map.get(&floop.line_no) {
                            Some(index) => *line_index = *index,
                            None => err!(line_number, pos, "Invalid target line for NEXT"),
                        }
                    }
                    else {
                        context
                            .floops
                            .remove(variable);
                    }
                }

                None => err!(line_number, pos, "No jumping iterator passed"),

                _ => err!(line_number, pos, "Invalid syntax for NEXT"),
            }
        }

        token::Token::While => {
            match parse_and_eval_expression(&mut token_iter, &context) {
                Ok(value::Value::Bool(_)) => context
                            .wloops
                            .push(WhileLoop { line_no: **line_number, pos: pos }),

                Err(_) => err!(line_number, pos, "Invalid boolean expression"),

                _ => err!(line_number, pos, "Invalid expression type (expected boolean)"),
            }
        }

        token::Token::Wend => {
            let wloops = &context.wloops;
            let wloop = match wloops.last() {
                Some(wl) => wl,
                None => err!(line_number, pos, "Cannot find last WHILE loop"),
            };

            let mut wtok_iter = &mut lineno_to_code[&wloop.line_no]
                .iter()
                .peekable();

            wtok_iter.next();

            match parse_and_eval_expression(&mut wtok_iter, &context) {
                Ok(value::Value::Bool(truth)) => {
                    if truth {
                        match line_map.get(&wloop.line_no) {
                            Some(index) => *line_index = *index,
                            None => err!(line_number, pos, "Invalid target line for WHILE"),
                        }
                    }
                    else {
                        context
                            .wloops
                            .pop();
                    }
                }

                Err(_) => err!(line_number, pos, "Invalid boolean expression"),

                _ => err!(line_number, pos, "Invalid expression type (expected boolean)"),
            }
        }

        _ => err!(line_number, pos, "Invalid syntax"),
    }
    
    return Ok(String::new());
}

fn parse_expression(
    token_iter: &mut Peekable<Iter<'_, lexer::TokenAndPos>>,
) -> Result<VecDeque<token::Token>, String> {
    let mut output_queue: VecDeque<token::Token> = VecDeque::new();
    let mut operator_stack: Vec<token::Token> = Vec::new();

    loop {
        match token_iter.peek() {
            Some(&&lexer::TokenAndPos(_, token::Token::Then)) |
            Some(&&lexer::TokenAndPos(_, token::Token::To)) |
            Some(&&lexer::TokenAndPos(_, token::Token::Step)) |
            None => break,
            _ => {}
        }

        //println!("iter: {:?}", token_iter);

        match token_iter.next() {
            Some(&lexer::TokenAndPos(_, ref value_token)) if value_token.is_value() => {
                output_queue.push_back(value_token.clone())
            }
            Some(&lexer::TokenAndPos(_, ref op_token)) if op_token.is_operator() => {
                if !operator_stack.is_empty() {
                    let top_op = operator_stack.last().unwrap().clone();
                    if top_op.is_operator() {
                        let associativity = op_token.operator_associavity().unwrap();

                        if (associativity == token::Associativity::Left
                            && op_token.operator_precedence() <= top_op.operator_precedence())
                            || (associativity == token::Associativity::Right
                                && op_token.operator_precedence() < top_op.operator_precedence())
                        {
                            let top_op = operator_stack.pop().unwrap();
                            output_queue.push_back(top_op.clone());
                        }
                    }
                }

                operator_stack.push(op_token.clone());
            }
            Some(&lexer::TokenAndPos(_, token::Token::LParen)) => {
                operator_stack.push(token::Token::LParen);
            }
            Some(&lexer::TokenAndPos(_, token::Token::RParen)) => loop {
                match operator_stack.pop() {
                    Some(token::Token::LParen) => break,
                    Some(ref next_token) => output_queue.push_back(next_token.clone()),
                    None => return Err("Mismatched parenthesis in expression".to_string()),
                }
            },
            _ => {
                unreachable!();
            },
        }
    }

    while !operator_stack.is_empty() {
        match operator_stack.pop().unwrap() {
            token::Token::LParen | token::Token::RParen => {
                return Err("Mismatched parenthesis in expression.".to_string())
            }
            op_token => output_queue.push_back(op_token.clone()),
        }
    }

    Ok(output_queue)
}

fn parse_and_eval_expression<'a>(
    token_iter: &mut Peekable<Iter<'a, lexer::TokenAndPos>>,
    context: &Context,
) -> Result<value::Value, String> {
    match parse_expression(token_iter) {
        Ok(mut output_queue) => {
            let mut stack: Vec<value::Value> = Vec::new();

            while !output_queue.is_empty() {
                match output_queue.pop_front() {
                    Some(token::Token::Number(ref number)) => {
                        stack.push(value::Value::Number(*number))
                    }
                    Some(token::Token::BString(ref bstring)) => {
                        stack.push(value::Value::String(bstring.clone()))
                    }
                    Some(token::Token::Variable(ref name)) => match context.variables.get(name) {
                        Some(value) => stack.push(value.clone()),
                        None => {
                            return Err(format!(
                                "Invalid variable reference {} in expression",
                                name
                            ))
                        }
                    },
                    Some(ref unary_token) if unary_token.is_unary_operator() => {
                        if !stack.is_empty() {
                            let value = stack.pop().unwrap();
                            let result = match *unary_token {
                                token::Token::UMinus => -value,
                                token::Token::Bang => !value,
                                // Pattern guard prevents any other match
                                _ => unreachable!(),
                            };
                            match result {
                                Ok(value) => stack.push(value),
                                Err(e) => return Err(e),
                            }
                        } else {
                            return Err(format!("Operator {:?} requires an operand!", unary_token));
                        }
                    }
                    Some(ref comparison_token) if comparison_token.is_comparison_operator() => {
                        if stack.len() >= 2 {
                            let operand2 = &stack.pop().unwrap();
                            let operand1 = &stack.pop().unwrap();

                            let result = match *comparison_token {
                                token::Token::Equals => operand1.eq(operand2),
                                token::Token::NotEqual => operand1.neq(operand2),
                                token::Token::LessThan => operand1.lt(operand2),
                                token::Token::GreaterThan => operand1.gt(operand2),
                                token::Token::LessThanEqual => operand1.lteq(operand2),
                                token::Token::GreaterThanEqual => operand1.gteq(operand2),
                                // Pattern guard prevents any other match
                                _ => unreachable!(),
                            };
                            match result {
                                Ok(value) => stack.push(value::Value::Bool(value)),
                                Err(e) => return Err(e),
                            }
                        } else {
                            return Err(format!(
                                "Comparison operator {:?} requires two operands",
                                comparison_token
                            ));
                        }
                    }
                    Some(ref binary_op_token) if binary_op_token.is_binary_operator() => {
                        if stack.len() >= 2 {
                            let operand2 = stack.pop().unwrap();
                            let operand1 = stack.pop().unwrap();

                            let result = match *binary_op_token {
                                token::Token::Plus => operand1 + operand2,
                                token::Token::Minus => operand1 - operand2,
                                token::Token::Multiply => operand1 * operand2,
                                token::Token::Divide => operand1 / operand2,
                                // Pattern guard prevents any other match
                                _ => unreachable!(),
                            };
                            match result {
                                Ok(value) => stack.push(value),
                                Err(e) => return Err(e),
                            }
                        }
                    }
                    None => unreachable!(),
                    _ => unreachable!(),
                }
            }

            // If expression is well formed, there will only be the result on the stack
            if stack.len() != 1 {
                return Err("Cannot parse expression".to_string());
            }
            
            Ok(stack[0].clone())
        }

        _ => Err("Invalid expression!".to_string()),
    }
}
