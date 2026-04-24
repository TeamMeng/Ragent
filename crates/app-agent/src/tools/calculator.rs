//! Calculator tool for basic arithmetic.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CalculatorTool;

#[derive(Debug, Deserialize, Serialize)]
pub struct CalculatorInput {
    pub expression: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CalculatorOutput {
    pub result: f64,
    pub expression: String,
}

impl CalculatorTool {
    pub fn new() -> Self {
        Self
    }

    pub fn name(&self) -> &str {
        "calculator"
    }

    pub fn description(&self) -> &str {
        "Evaluate a mathematical expression. Supports +, -, *, /, parentheses."
    }

    pub async fn call(
        &self,
        args: CalculatorInput,
    ) -> Result<CalculatorOutput, Box<dyn std::error::Error + Send + Sync>> {
        let result = eval_expr(&args.expression)?;
        Ok(CalculatorOutput {
            result,
            expression: args.expression,
        })
    }
}

/// Simple recursive descent parser for arithmetic expressions.
fn eval_expr(expr: &str) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let tokens = tokenize(expr)?;
    let (result, pos) = parse_add_sub(&tokens, 0)?;
    if pos != tokens.len() {
        return Err("Unexpected tokens after expression".into());
    }
    Ok(result)
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

fn tokenize(expr: &str) -> Result<Vec<Tok>, Box<dyn std::error::Error + Send + Sync>> {
    let mut out = Vec::new();
    let mut chars = expr.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\n' => {
                chars.next();
            }
            '+' => {
                out.push(Tok::Plus);
                chars.next();
            }
            '-' => {
                out.push(Tok::Minus);
                chars.next();
            }
            '*' => {
                out.push(Tok::Star);
                chars.next();
            }
            '/' => {
                out.push(Tok::Slash);
                chars.next();
            }
            '(' => {
                out.push(Tok::LParen);
                chars.next();
            }
            ')' => {
                out.push(Tok::RParen);
                chars.next();
            }
            c if c.is_ascii_digit() || c == '.' => {
                let mut s = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() || d == '.' {
                        s.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                out.push(Tok::Num(s.parse()?));
            }
            _ => return Err(format!("Unexpected char: {}", c).into()),
        }
    }
    Ok(out)
}

fn parse_add_sub(
    t: &[Tok],
    i: usize,
) -> Result<(f64, usize), Box<dyn std::error::Error + Send + Sync>> {
    let (mut left, mut i) = parse_mul_div(t, i)?;
    while i < t.len() {
        match t[i] {
            Tok::Plus => {
                i += 1;
                let (r, p) = parse_mul_div(t, i)?;
                left += r;
                i = p;
            }
            Tok::Minus => {
                i += 1;
                let (r, p) = parse_mul_div(t, i)?;
                left -= r;
                i = p;
            }
            _ => break,
        }
    }
    Ok((left, i))
}

fn parse_mul_div(
    t: &[Tok],
    i: usize,
) -> Result<(f64, usize), Box<dyn std::error::Error + Send + Sync>> {
    let (mut left, mut i) = parse_atom(t, i)?;
    while i < t.len() {
        match t[i] {
            Tok::Star => {
                i += 1;
                let (r, p) = parse_atom(t, i)?;
                left *= r;
                i = p;
            }
            Tok::Slash => {
                i += 1;
                let (r, p) = parse_atom(t, i)?;
                if r == 0.0 {
                    return Err("Division by zero".into());
                }
                left /= r;
                i = p;
            }
            _ => break,
        }
    }
    Ok((left, i))
}

fn parse_atom(
    t: &[Tok],
    i: usize,
) -> Result<(f64, usize), Box<dyn std::error::Error + Send + Sync>> {
    if i >= t.len() {
        return Err("Unexpected end".into());
    }
    match &t[i] {
        Tok::Num(n) => Ok((*n, i + 1)),
        Tok::LParen => {
            let (r, p) = parse_add_sub(t, i + 1)?;
            if p >= t.len() || t[p] != Tok::RParen {
                return Err("Missing )".into());
            }
            Ok((r, p + 1))
        }
        Tok::Minus => {
            let (r, p) = parse_atom(t, i + 1)?;
            Ok((-r, p))
        }
        _ => Err(format!("Unexpected token at {}", i).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic() {
        let tool = CalculatorTool::new();
        let out = tool
            .call(CalculatorInput {
                expression: "2 + 3 * 4".into(),
            })
            .await
            .unwrap();
        assert!((out.result - 14.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_parens() {
        let tool = CalculatorTool::new();
        let out = tool
            .call(CalculatorInput {
                expression: "(2 + 3) * 4".into(),
            })
            .await
            .unwrap();
        assert!((out.result - 20.0).abs() < f64::EPSILON);
    }
}
