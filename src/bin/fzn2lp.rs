use anyhow::Result;
use flatzinc::*;
use log::{error, warn};
use nom::{
    branch::alt,
    error::{convert_error, ParseError, VerboseError},
    Err, IResult,
};
use std::{
    fs::File,
    io::{self, prelude::*, BufReader},
    path::PathBuf,
};
use stderrlog;
use structopt::StructOpt;

/// Convert FlatZinc to AnsProlog facts
#[derive(StructOpt, Debug)]
#[structopt(name = "fzn2lp")]
struct Opt {
    /// Input file in flatzinc format
    #[structopt(name = "FILE", parse(from_os_str))]
    file: Option<PathBuf>,
}

fn main() {
    if let Err(err) = run() {
        error!("{:?}", err);
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    stderrlog::new()
        .module(module_path!())
        .verbosity(2)
        .init()
        .unwrap();

    let opt = Opt::from_args();
    let mut level = 1;
    let mut counter = 1;

    if let Some(file) = opt.file {
        let file = File::open(file)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            match_fz_stmt(&line?, &mut counter, &mut level)?;
        }
        Ok(())
    } else {
        let mut buf = String::new();
        while 0 < io::stdin().read_line(&mut buf)? {
            match_fz_stmt(&buf, &mut counter, &mut level)?;
            buf.clear();
        }
        Ok(())
    }
}
use thiserror::Error;
#[derive(Error, Debug)]
pub enum FlatZincError {
    #[error("More than one solve item")]
    MultipleSolveItems,
    #[error("ParseError: {msg}")]
    ParseError { msg: String },
}
fn match_fz_stmt(input: &str, counter: &mut usize, level: &mut i32) -> Result<(), FlatZincError> {
    match fz_statement::<VerboseError<&str>>(&input) {
        Ok((_rest, stmt)) => {
            match stmt {
                FzStmt::Predicate(pred) => {
                    if *level > 1 {
                        warn!("Statements in wrong order.");
                    }
                    print_predicate(&pred);
                }
                FzStmt::Parameter(p) => {
                    if *level > 2 {
                        warn!("Statements in wrong order.");
                    } else {
                        *level = 2;
                    }
                    print_par_decl_item(&p);
                }
                FzStmt::Variable(d) => {
                    if *level > 3 {
                        warn!("Statements in wrong order.");
                    } else {
                        *level = 3;
                    }
                    print_var_decl_item(&d);
                }
                FzStmt::Constraint(c) => {
                    if *level > 4 {
                        warn!("Statements in wrong order.");
                    } else {
                        *level = 4;
                    }
                    print_constraint(&c, *counter);
                    *counter += 1;
                }
                FzStmt::SolveItem(i) => {
                    if *level > 4 {
                        return Err(FlatZincError::MultipleSolveItems);
                    }
                    print_solve_item(&i);
                    *level = 5;
                }
            }
            Ok(())
        }
        Err(Err::Error(e)) | Err(Err::Failure(e)) => {
            let bla = convert_error(&input, e);
            Err(FlatZincError::ParseError { msg: bla })
        }
        Err(e) => Err(FlatZincError::ParseError {
            msg: format!("{}", e),
        }),
    }
}
#[derive(PartialEq, Clone, Debug)]
pub enum FzStmt {
    Predicate(PredicateItem),
    Parameter(ParDeclItem),
    Variable(VarDeclItem),
    Constraint(ConstraintItem),
    SolveItem(SolveItem),
}
pub fn fz_statement<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, FzStmt, E> {
    let (input, res) = alt((
        fz_predicate,
        fz_parameter,
        fz_variable,
        fz_constraint,
        fz_solve_item,
    ))(input)?;
    Ok((input, res))
}
fn fz_predicate<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, FzStmt, E> {
    let (input, item) = predicate_item(input)?;
    Ok((input, FzStmt::Predicate(item)))
}
fn fz_parameter<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, FzStmt, E> {
    let (input, item) = par_decl_item(input)?;
    Ok((input, FzStmt::Parameter(item)))
}
fn fz_variable<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, FzStmt, E> {
    let (input, item) = var_decl_item(input)?;
    Ok((input, FzStmt::Variable(item)))
}
fn fz_constraint<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, FzStmt, E> {
    let (input, item) = constraint_item(input)?;
    Ok((input, FzStmt::Constraint(item)))
}
fn fz_solve_item<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, FzStmt, E> {
    let (input, item) = solve_item(input)?;
    Ok((input, FzStmt::SolveItem(item)))
}
fn print_predicate(item: &PredicateItem) {
    println!("predicate({}).", identifier(&item.id));
    for (pos, p) in item.parameters.iter().enumerate() {
        match p {
            (PredParType::Basic(par_type), id) => println!(
                "predicate_parameter({},{},{},{}).",
                identifier(&item.id),
                pos,
                basic_pred_par_type(&par_type),
                identifier(id)
            ),
            (PredParType::Array { ix, par_type }, id) => println!(
                "predicate_parameter({},{},array({},{}),{}).",
                identifier(&item.id),
                pos,
                pred_index(&ix),
                basic_pred_par_type(&par_type),
                identifier(id)
            ),
        }
    }
}
fn print_par_decl_item(item: &ParDeclItem) {
    match item {
        ParDeclItem::Basic { par_type, id, expr } => print!(
            "parameter({}, {},{}).",
            basic_par_type(&par_type),
            identifier(id),
            par_expr_basic_literal_expr(expr)
        ),
        ParDeclItem::Array {
            ix,
            par_type,
            id,
            expr,
        } => {
            let array_elements = par_expr_array_expr(expr);
            println!(
                "parameter(array({},{}),{}).",
                index(ix),
                basic_par_type(&par_type),
                identifier(id)
            );
            for (pos, e) in array_elements.iter().enumerate() {
                println!(
                    "in_array({},{},{}).",
                    identifier(id),
                    pos,
                    basic_literal_expr(e)
                );
            }
        }
    }
}
fn print_var_decl_item(item: &VarDeclItem) {
    match item {
        VarDeclItem::Array {
            ix,
            var_type,
            id,
            annos,
            array_literal,
        } => {
            println!(
                "variable(array({},{}),{}).",
                index(ix),
                basic_var_type(var_type),
                identifier(id),
            );
            for (pos, e) in array_literal.iter().enumerate() {
                println!("in_array({},{},{}).", identifier(id), pos, basic_expr(e));
            }
        }
        VarDeclItem::Basic {
            var_type,
            id,
            annos,
            expr: None,
        } => {
            println!("variable({},{}).", basic_var_type(var_type), identifier(id));
        }
        VarDeclItem::Basic {
            var_type,
            id,
            annos,
            expr: Some(e),
        } => {
            println!(
                "variable({},{},{}).",
                basic_var_type(var_type),
                identifier(id),
                basic_expr(e)
            );
        }
    }
}
fn basic_var_type(t: &BasicVarType) -> String {
    match t {
        BasicVarType::Bool => "bool".to_string(),
        BasicVarType::Domain(d) => domain(d),
        BasicVarType::Float => "float".to_string(),
        BasicVarType::Int => "int".to_string(),
        BasicVarType::SetOfInt => "set_of_int".to_string(),
    }
}
// TODO implement sets
fn domain(d: &Domain) -> String {
    match d {
        Domain::FloatRange(f1, f2) => format!("range_f({},{})", f1, f2),
        Domain::IntRange(i1, i2) => format!("range_i({},{})", i1, i2),
        Domain::SetInt(v) => {
            error!("Not implemented: VAR TYPE NON_EMPTY SET INT {:#?}", v);
            panic!("Not implemented: VAR TYPE NON_EMPTY SET INT {:#?}", v);
        }
        Domain::SetIntNonEmpty(v) => {
            error!("Not implemented: VAR TYPE NON_EMPTY SET INT {:#?}", v);
            panic!("Not implemented: VAR TYPE NON_EMPTY SET INT {:#?}", v);
        }
        Domain::SetIntRange(i1, i2) => format!("set_int_range({},{})", i1, i2),
    }
}

fn print_constraint(c: &ConstraintItem, i: usize) {
    println!("constraint(c{},{}).", i, identifier(&c.id));
    for (cpos, ce) in c.exprs.iter().enumerate() {
        match ce {
            Expr::BasicExpr(e) => println!(
                "in_constraint(c{},{},{},{}).",
                i,
                identifier(&c.id),
                cpos,
                basic_expr(&e)
            ),
            Expr::ArrayLiteral(v) => {
                println!("in_constraint(c{},{},array).", i, cpos);
                for (apos, ae) in v.iter().enumerate() {
                    println!(
                        "in_constraint(c{},{},{},{}).",
                        i,
                        cpos,
                        apos,
                        basic_expr(&ae)
                    );
                }
            }
        }
    }
}
fn print_solve_item(i: &SolveItem) {
    match &i.goal {
        Goal::Satisfy => println!("solve(satisfy)."),
        Goal::Maximize(e) => println!("solve(maximize,{}).", basic_expr(&e)),
        Goal::Minimize(e) => println!("solve(minimize,{}).", basic_expr(&e)),
    }
}
fn basic_par_type(t: &BasicParType) -> String {
    match t {
        BasicParType::Bool => "bool".to_string(),
        BasicParType::Float => "float".to_string(),
        BasicParType::Int => "int".to_string(),
        BasicParType::SetOfInt => "set_of_int".to_string(),
    }
}
fn basic_pred_par_type(t: &BasicPredParType) -> String {
    match t {
        BasicPredParType::BasicParType(t) => basic_par_type(t),
        BasicPredParType::BasicVarType(t) => basic_var_type(t),
        BasicPredParType::Domain(d) => domain(d),
    }
}
fn index(IndexSet(i): &IndexSet) -> String {
    i.to_string()
}
fn identifier(s: &str) -> String {
    format!("\"{}\"", s)
}
fn pred_index(is: &PredIndexSet) -> String {
    match is {
        PredIndexSet::IndexSet(i) => i.to_string(),
        PredIndexSet::Int => "int".to_string(),
    }
}
fn par_expr_array_expr(e: &ParExpr) -> &[BasicLiteralExpr] {
    match e {
        ParExpr::ParArrayLiteral(v) => v,
        ParExpr::BasicLiteralExpr(_l) => panic!(
            "I think this should be an array, but its a basic-literal-expr! Maybe use par_expr?"
        ),
    }
}
fn par_expr_basic_literal_expr(e: &ParExpr) -> String {
    match e {
        ParExpr::ParArrayLiteral(_v) => panic!(
            "I think this should be a basic-literal-expr, but its an array! Maybe use par_expr?"
        ),
        ParExpr::BasicLiteralExpr(l) => basic_literal_expr(l),
    }
}
// fn par_expr(e: &ParExpr) -> (String, String) {
//     match e {
//         ParExpr::ParArrayLiteral(v) => {
//             let mut x = String::new();
//             for e in v {
//                 if x.is_empty() {
//                     x = format!("in_array({},{},{})", basic_literal_expr(e));
//                 } else {
//                     x = format!("{},{}", x, basic_literal_expr(e));
//                 }
//             }
//             ("array".into(), x)
//         }
//         ParExpr::BasicLiteralExpr(l) => (basic_literal_expr(l), String::new()),
//     }
// }

fn basic_expr(e: &BasicExpr) -> String {
    match e {
        BasicExpr::BasicLiteralExpr(e) => basic_literal_expr(e),
        BasicExpr::VarParIdentifier(s) => identifier(s),
    }
}
fn basic_literal_expr(e: &BasicLiteralExpr) -> String {
    match e {
        BasicLiteralExpr::Bool(b) => b.to_string(),
        BasicLiteralExpr::Float(f) => format!("\"{}\"", f),
        BasicLiteralExpr::Int(i) => i.to_string(),
        BasicLiteralExpr::Set(s) => set_literal(s),
    }
}
fn set_literal(l: &SetLiteral) -> String {
    match l {
        SetLiteral::FloatRange(f1, f2) => format!("range_i(\"{}\",\"{}\")", f1, f2),
        SetLiteral::IntRange(i1, i2) => format!("range_i({},{})", i1, i2),
        SetLiteral::SetFloats(v) => set_floats(v),
        SetLiteral::SetInts(v) => set_ints(v),
    }
}
fn set_floats(v: &[f64]) -> String {
    let mut x = String::new();
    for f in v {
        if x.is_empty() {
            x = format!("\"{}\"", f);
        } else {
            x = format!("{},\"{}\"", x, f);
        }
    }
    format!("set_floats({}).", x)
}
fn set_ints(v: &[i128]) -> String {
    let mut x = String::new();
    for i in v {
        if x.is_empty() {
            x = i.to_string();
        } else {
            x = format!("{},{}", x, i);
        }
    }
    format!("set_ints({}).", x)
}
