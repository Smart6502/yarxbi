use std::{
    ops::{Add, Div, Mul, Neg, Not, Sub},
    str::FromStr,
};

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
}

// -----------------------------------------------
// Implementations of unary operators
impl Neg for Value {
    type Output = Result<Value, String>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Number(ref number) => Ok(Value::Number(-*number)),
            _ => Err("Cannot negate non-numeric values!".to_string()),
        }
    }
}

impl Not for Value {
    type Output = Result<Value, String>;

    fn not(self) -> Self::Output {
        match self {
            Value::Bool(ref boolean) => Ok(Value::Bool(!boolean)),
            _ => Err("Cannot apply unary not to non-Boolean values.".to_string()),
        }
    }
}

// -----------------------------------------------
// Implementations of binary operators
impl Add for Value {
    type Output = Result<Value, String>;

    fn add(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Number(number1), Value::Number(number2)) => {
                Ok(Value::Number(number1 + number2))
            }
            (Value::String(string1), Value::String(string2)) => {
                Ok(Value::String(format!("{}{}", string1, string2)))
            }
            (Value::Number(number1), Value::String(string2)) => {
                let number2 = f64::from_str(string2.as_str());

                if let Result::Ok(number2_value) = number2 {
                    Ok(Value::Number(number1 + number2_value))
                } else {
                    Err(format!(
                        "Cannot add integer {} and string {}",
                        number1, string2
                    ))
                }
            }
            (Value::String(string1), Value::Number(number2)) => {
                let number1 = f64::from_str(string1.as_str());

                if let Result::Ok(number1_value) = number1 {
                    Ok(Value::Number(number1_value + number2))
                } else {
                    Err(format!(
                        "Cannot add string {} and integer {}",
                        string1, number2
                    ))
                }
            }
            _ => Err("Can only add integers or concatenate strings.".to_string()),
        }
    }
}

impl Div for Value {
    type Output = Result<Value, String>;

    fn div(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Number(number1), Value::Number(number2)) => {
                Ok(Value::Number(number1 / number2))
            }
            (Value::Number(number1), Value::String(string2)) => {
                let number2 = f64::from_str(string2.as_str());

                if let Result::Ok(number2_value) = number2 {
                    Ok(Value::Number(number1 / number2_value))
                } else {
                    Err(format!(
                        "Cannot divide integer {} and string {}",
                        number1, string2
                    ))
                }
            }
            (Value::String(string1), Value::Number(number2)) => {
                let number1 = f64::from_str(string1.as_str());

                if let Result::Ok(number1_value) = number1 {
                    Ok(Value::Number(number1_value / number2))
                } else {
                    Err(format!(
                        "Cannot divide string {} and integer {}",
                        string1, number2
                    ))
                }
            }
            _ => Err("Can only divide integers.".to_string()),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value, String>;

    fn mul(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Number(number1), Value::Number(number2)) => {
                Ok(Value::Number(number1 * number2))
            }
            (Value::Number(number1), Value::String(string2)) => {
                let number2 = f64::from_str(string2.as_str());

                if let Result::Ok(number2_value) = number2 {
                    Ok(Value::Number(number1 * number2_value))
                } else {
                    Err(format!(
                        "Cannot multiply integer {} and string {}",
                        number1, string2
                    ))
                }
            }
            (Value::String(string1), Value::Number(number2)) => {
                let number1 = f64::from_str(string1.as_str());

                if let Result::Ok(number1_value) = number1 {
                    Ok(Value::Number(number1_value * number2))
                } else {
                    Err(format!(
                        "Cannot multiply string {} and integer {}",
                        string1, number2
                    ))
                }
            }
            _ => Err("Can only multiply integers.".to_string()),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value, String>;

    fn sub(self, other: Value) -> Self::Output {
        match (self, other) {
            (Value::Number(number1), Value::Number(number2)) => {
                Ok(Value::Number(number1 - number2))
            }
            (Value::Number(number1), Value::String(string2)) => {
                let number2 = f64::from_str(string2.as_str());

                if let Result::Ok(number2_value) = number2 {
                    Ok(Value::Number(number1 - number2_value))
                } else {
                    Err(format!(
                        "Cannot subtract integer {} from string {}",
                        number1, string2
                    ))
                }
            }
            (Value::String(string1), Value::Number(number2)) => {
                let number1 = f64::from_str(string1.as_str());

                if let Result::Ok(number1_value) = number1 {
                    Ok(Value::Number(number1_value - number2))
                } else {
                    Err(format!(
                        "Cannot subtract string {} from integer {}",
                        string1, number2
                    ))
                }
            }
            _ => Err("Can only subtract integers.".to_string()),
        }
    }
}

// -----------------------------------------------
// Implementations of binary comparison operators
impl Value {
    pub fn eq(&self, other: &Value) -> Result<bool, String> {
        match (self, other) {
            (&Value::Number(number1), &Value::Number(number2)) => {
                Ok(number1 == number2)
            }
            (&Value::String(ref string1), &Value::String(ref string2)) => {
                Ok(string1 == string2)
            }
            (&Value::Bool(bool1), &Value::Bool(bool2)) => Ok(bool1 == bool2),
            (&Value::Number(number1), &Value::String(ref string2)) => {
                let number2 = f64::from_str(string2.as_str());

                if let Result::Ok(number2_value) = number2 {
                    Ok(number1 == number2_value)
                } else {
                    Err(format!(
                        "Cannot compare integer {} from string {}",
                        number1, string2
                    ))
                }
            }
            (&Value::String(ref string1), &Value::Number(number2)) => {
                let number1 = f64::from_str(string1.as_str());

                if let Result::Ok(number1_value) = number1 {
                    Ok(number1_value == number2)
                } else {
                    Err(format!(
                        "Cannot compare string {} and integer {}",
                        string1, number2
                    ))
                }
            }
            _ => Err(format!(
                "Cannot compare values of different types {:?} and {:?}",
                *self, *other
            )),
        }
    }

    pub fn neq(&self, other: &Value) -> Result<bool, String> {
        self.eq(other).map(|value| !value)
    }

    pub fn lt(&self, other: &Value) -> Result<bool, String> {
        match (self, other) {
            (&Value::Number(number1), &Value::Number(number2)) => Ok(number1 < number2),
            (&Value::String(ref string1), &Value::String(ref string2)) => {
                Ok(string1 < string2)
            }
            (&Value::Bool(bool1), &Value::Bool(bool2)) => Ok(bool1 == bool2),
            (&Value::Number(number1), &Value::String(ref string2)) => {
                let number2 = f64::from_str(string2.as_str());

                if let Result::Ok(number2_value) = number2 {
                    Ok(number1 < number2_value)
                } else {
                    Err(format!(
                        "Cannot compare integer {} from string {}",
                        number1, string2
                    ))
                }
            }
            (&Value::String(ref string1), &Value::Number(number2)) => {
                let number1 = f64::from_str(string1.as_str());

                if let Result::Ok(number1_value) = number1 {
                    Ok(number1_value < number2)
                } else {
                    Err(format!(
                        "Cannot compare string {} and integer {}",
                        string1, number2
                    ))
                }
            }
            _ => Err(format!(
                "Cannot compare values of different types {:?} and {:?}",
                *self, *other
            )),
        }
    }

    pub fn gt(&self, other: &Value) -> Result<bool, String> {
        match (self, other) {
            (&Value::Number(number1), &Value::Number(number2)) => Ok(number1 > number2),
            (&Value::String(ref string1), &Value::String(ref string2)) => {
                Ok(string1 > string2)
            }
            (&Value::Bool(bool1), &Value::Bool(bool2)) => Ok(bool1 && !bool2),
            (&Value::Number(number1), &Value::String(ref string2)) => {
                let number2 = f64::from_str(string2.as_str());

                if let Result::Ok(number2_value) = number2 {
                    Ok(number1 > number2_value)
                } else {
                    Err(format!(
                        "Cannot compare integer {} from string {}",
                        number1, string2
                    ))
                }
            }
            (&Value::String(ref string1), &Value::Number(number2)) => {
                let number1 = f64::from_str(string1.as_str());

                if let Result::Ok(number1_value) = number1 { 
                    Ok(number1_value > number2)
                } else {
                    Err(format!(
                        "Cannot compare string {} and integer {}",
                        string1, number2
                    ))
                }
            }
            _ => Err(format!(
                "Cannot compare values of different types {:?} and {:?}",
                *self, *other
            )),
        }
    }

    pub fn lteq(&self, other: &Value) -> Result<bool, String> {
        self.gt(other).map(|value| !value)
    }

    pub fn gteq(&self, other: &Value) -> Result<bool, String> {
        self.lt(other).map(|value| !value)
    }
}
