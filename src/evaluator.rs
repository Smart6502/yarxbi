use crate::{lexer, token, value};

use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    io,
    iter::Peekable,
    slice::Iter,
};

#[derive(Debug)]
struct ForLoop {
    line_no: lexer::LineNumber,
    end: f64,
    step: f64,
    slide: bool
}

#[derive(Debug)]
struct Context {
    variables: HashMap<String, value::Value>,
    floops: HashMap<String, ForLoop>
}

impl Context {
    fn new() -> Context {
        Context {
            variables: HashMap::new(),
            floops: HashMap::new(),
        }
    }
}

macro_rules! err {
    ($line:ident, $pos:expr, $fmt:expr $(, $p:expr ) *) => {
        return Err((**$line, $pos, format!($fmt, $($p),*)))
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

            match *token {
                token::Token::Rem => {
                    // Skip the rest of the line so do nothing
                }

                token::Token::Goto => {
                    line_has_goto = true;
                    match token_iter.next() {
                        Some(&lexer::TokenAndPos(pos, token::Token::Number(number))) => {
                            let n = lexer::LineNumber(number as u32);
                            match line_map.get(&n) {
                                Some(index) => line_index = *index,
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
                            match context
                                .variables
                                .insert(variable.clone().to_string(), value.clone()) {
                                
                                _ => {},
                            }
                        }

                        (_, _, Err(e)) => err!(line_number, pos, "Error in LET expression: {}", e),

                        _ => err!(line_number, pos, "Invalid syntax for LET"),
                        
                    }
                }

                token::Token::Print => {
                    // Expected Next:
                    // EXPRESSION
                    match parse_and_eval_expression(&mut token_iter, &context) {
                        Ok(value::Value::String(value)) => print!("{}", value),
                        Ok(value::Value::Number(value)) => print!("{}", value),
                        Ok(value::Value::Bool(value)) => print!("{}", value),
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
                                .entry(variable.clone().to_string())
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
                                    line_has_goto = true;
                                    let n = lexer::LineNumber(*number as u32);
                                    match line_map.get(&n) {
                                        Some(index) => line_index = *index,
                                        _ => err!(line_number, pos, "Invalid target line for IF"),
                                        }
                                    }
                            }
                        
                        _ => err!(line_number, pos, "Invalid syntax for IF"),
                    }
                }

                token::Token::For => {
                    match (
                        token_iter.next(),
                        token_iter.next(),
                        parse_and_eval_expression(&mut token_iter, &context),
                        token_iter.next(),
                        parse_and_eval_expression(&mut token_iter, &context),
                        token_iter.next(),
                        parse_and_eval_expression(&mut token_iter, &context),
                    ) {
                        (
                            Some(&lexer::TokenAndPos(_, token::Token::Variable(ref variable))),
                            Some(&lexer::TokenAndPos(_, token::Token::Equals)),
                            Ok(value::Value::Number(ref start)),
                            Some(&lexer::TokenAndPos(_, token::Token::To)),
                            Ok(value::Value::Number(ref end)),
                            Some(&lexer::TokenAndPos(_, token::Token::Step)),
                            Ok(value::Value::Number(ref step))
                        ) => {
                            context.variables.insert(variable.clone(), value::Value::Number(*start));

                            context.floops.insert(variable.clone(), ForLoop {
                                line_no: *line_number.clone(),
                                end: end.clone(),
                                step: step.clone(),
                                slide: *start <= *end
                            });
                        }

                        _ => err!(line_number, pos, "Invalid syntax for FOR"),
                    }
                }

                token::Token::Next => {
                    match token_iter.next() {
                        Some(&lexer::TokenAndPos(_, token::Token::Variable(ref variable))) => {
                            let floop = match context.floops.get(variable) {
                                Some(f) => f,
                                None => err!(line_number, pos, "FOR loop is out of context"),
                            };

                            let raw = match context.variables.get(variable) {
                                Some(v) => v,
                                None => err!(line_number, pos, "Invalid variable expression {}", variable), 
                            }.clone();

                            let var = match raw {
                                value::Value::Number(num) => num,
                                _ => err!(line_number, pos, "Variable {} called by NEXT is not a number", variable),
                            };

                            let next = var + floop.step;
                            let loop_br = if floop.slide { next < floop.end } else { next > floop.end };

                            if loop_br {
                                context.variables.insert(variable.to_string(), value::Value::Number(next));
                                match line_map.get(&floop.line_no) {
                                    Some(index) => line_index = *index,
                                    None => err!(line_number, pos, "Could not get line map"),
                                }
                            }
                            else {
                                context.floops.remove(variable);
                            }
                        }
                        _ => err!(line_number, pos, "Invalid syntax for NEXT"),
                    }
                }
            
            _ => err!(line_number, pos, "Invalid syntax"),
            }
        }

        if !line_has_goto {
            line_index += 1;
            if line_index == num_lines {
                break;
            }
        }
    }

    Ok("\nCompleted Successfully".to_string())
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
            _ => unreachable!(),
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

            // println!("Evaluating queue: {:?}", output_queue);

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
            assert!(stack.len() == 1);
            // println!("Final expression result: {:?}", stack[0]);
            Ok(stack[0].clone())
        }

        _ => Err("Invalid expression!".to_string()),
    }
}
