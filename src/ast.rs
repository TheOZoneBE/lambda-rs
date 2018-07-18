use parser::*;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ASTError {
    message: String,
}

impl ASTError {
    pub fn new(message: String) -> ASTError {
        ASTError { message }
    }
}

impl fmt::Display for ASTError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ASTError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[derive(Debug)]
pub enum Value {
    True,
    False,
    Zero,
}

#[derive(Debug)]
pub enum Operator {
    Succ,
    Pred,
}

#[derive(Debug)]
pub enum Type {
    Bool,
    Nat,
}

#[derive(Debug)]
pub enum ASTNode {
    AbstractionNode {
        ident: Box<ASTNode>,
        data_type: Type,
        body: Box<ASTNode>,
    },
    ApplicationNode {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    IdentifierNode {
        name: String,
    },
    ConditionNode {
        clause: Box<ASTNode>,
        then_arm: Box<ASTNode>,
        else_arm: Box<ASTNode>,
    },
    ArithmeticNode {
        op: Operator,
        expr: Box<ASTNode>,
    },
    IsZeroNode {
        expr: Box<ASTNode>,
    },
    ValueNode {
        value: Value,
    },
}

impl ASTNode {
    pub fn print(&self) {
        self.print_node(0)
    }

    fn print_node(&self, level: usize) {
        match self {
            ASTNode::AbstractionNode {
                ident,
                data_type,
                body,
            } => {
                println!(
                    "{}Abstraction with type {:?}",
                    "\t".repeat(level),
                    data_type
                );
                ident.print_node(level + 1);
                body.print_node(level + 1);
            }
            ASTNode::ApplicationNode { left, right } => {
                println!("{}Application", "\t".repeat(level));
                left.print_node(level + 1);
                right.print_node(level + 1);
            }
            ASTNode::IdentifierNode { name } => {
                println!("{}Identifier with name {}", "\t".repeat(level), name);
            }
            ASTNode::IsZeroNode { expr } => {
                println!("{}IsZero", "\t".repeat(level));
                expr.print_node(level + 1);
            }
            ASTNode::ValueNode { value } => {
                println!("{}Value {:?}", "\t".repeat(level), value);
            }
            ASTNode::ArithmeticNode { op, expr } => {
                println!("{}Arithmetic with operator {:?}", "\t".repeat(level), op);
                expr.print_node(level + 1);
            }
            ASTNode::ConditionNode {
                clause,
                then_arm,
                else_arm,
            } => {
                println!("{}Condition", "\t".repeat(level));
                clause.print_node(level + 1);
                then_arm.print_node(level + 1);
                else_arm.print_node(level + 1);
            }
        }
    }
}

pub fn build_ast(mut parsed: Pairs<Rule>) -> Result<ASTNode, Box<Error>> {
    let first = parsed
        .next()
        .ok_or_else(|| Box::new(ASTError::new(format!("Invalid program"))))?;
    build_node(first)
}

fn build_node(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let rule = pair.as_rule();
    match rule {
        Rule::program => build_program(pair),
        Rule::application => build_application(pair),
        Rule::abstraction => build_abstraction(pair),
        Rule::ident => build_ident(pair),
        Rule::arithmetic => build_arithmetic(pair),
        Rule::zero_check => build_zero_check(pair),
        Rule::if_then => build_if_then(pair),
        Rule::val_zero => Ok(ASTNode::ValueNode { value: Value::Zero }),
        Rule::val_true => Ok(ASTNode::ValueNode { value: Value::True }),
        Rule::val_false => Ok(ASTNode::ValueNode {
            value: Value::False,
        }),
        _ => Err(Box::new(ASTError::new(format!(
            "Not implemented: {:?}",
            rule
        )))),
    }
}

fn build_program(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let inner: Vec<Pair<'_, Rule>> = pair.into_inner().collect();

    match inner.get(0) {
        Some(x) => build_node(x.clone()),
        None => Err(Box::new(ASTError::new(format!(
            "No application body in program"
        )))),
    }
}

fn build_application(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let inner: Vec<Pair<'_, Rule>> = pair.into_inner().collect();

    if inner.len() == 1 {
        build_node(inner[0].clone())
    } else if inner.len() == 2 {
        Ok(ASTNode::ApplicationNode {
            left: Box::new(build_node(inner[0].clone())?),
            right: Box::new(build_node(inner[1].clone())?),
        })
    } else {
        Err(Box::new(ASTError::new(format!(
            "Found application with incorrect number of arguments"
        ))))
    }
}

fn build_abstraction(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let inner: Vec<Pair<'_, Rule>> = pair.into_inner().collect();

    if inner.len() == 3 {
        let data_type = match inner[1].as_rule() {
            Rule::type_nat => Ok(Type::Nat),
            Rule::type_bool => Ok(Type::Bool),
            _ => Err(Box::new(ASTError::new(format!("Incorrect type")))),
        };
        Ok(ASTNode::AbstractionNode {
            ident: Box::new(build_node(inner[0].clone())?),
            data_type: data_type?,
            body: Box::new(build_node(inner[2].clone())?),
        })
    } else {
        Err(Box::new(ASTError::new(format!(
            "Found abstraction with incorrect number of arguments"
        ))))
    }
}

fn build_ident(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let span = pair.into_span();
    Ok(ASTNode::IdentifierNode {
        name: span.as_str().to_string(),
    })
}

fn build_arithmetic(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let inner: Vec<Pair<'_, Rule>> = pair.into_inner().collect();

    if inner.len() == 2 {
        let op = match inner[0].as_rule() {
            Rule::op_succ => Ok(Operator::Succ),
            Rule::op_pred => Ok(Operator::Pred),
            _ => Err(Box::new(ASTError::new(format!("Incorrect operator")))),
        };
        Ok(ASTNode::ArithmeticNode {
            op: op?,
            expr: Box::new(build_node(inner[1].clone())?),
        })
    } else {
        Err(Box::new(ASTError::new(format!(
            "Found arithmetic with incorrect number of arguments"
        ))))
    }
}

fn build_zero_check(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let inner: Vec<Pair<'_, Rule>> = pair.into_inner().collect();
    if inner.len() == 1 {
        Ok(ASTNode::IsZeroNode {
            expr: Box::new(build_node(inner[0].clone())?),
        })
    } else {
        Err(Box::new(ASTError::new(format!(
            "Found zero check with incorrect number of arguments"
        ))))
    }
}

fn build_if_then(pair: Pair<'_, Rule>) -> Result<ASTNode, Box<Error>> {
    let inner: Vec<Pair<'_, Rule>> = pair.into_inner().collect();

    if inner.len() == 3 {
        Ok(ASTNode::ConditionNode {
            clause: Box::new(build_node(inner[0].clone())?),
            then_arm: Box::new(build_node(inner[1].clone())?),
            else_arm: Box::new(build_node(inner[2].clone())?),
        })
    } else {
        Err(Box::new(ASTError::new(format!(
            "Found ifthenelse with incorrect number of arguments"
        ))))
    }
}
