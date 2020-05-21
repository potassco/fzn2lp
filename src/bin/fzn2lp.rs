use anyhow::Result;
use flatzinc::*;
use log::{error, warn};
use std::{
    fs::File,
    io::{self, prelude::*, BufReader},
    path::PathBuf,
};
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
    stderrlog::new()
        .module(module_path!())
        .verbosity(2)
        .init()
        .unwrap();
    if let Err(err) = run() {
        error!("{:?}", err);
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let opt = Opt::from_args();
    let mut level = 1;
    let mut counter = 1;

    if let Some(file) = opt.file {
        let file = File::open(file)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            match_fz_stmt(&line?, &mut counter, &mut level)?;
        }
    } else {
        let mut buf = String::new();
        while 0 < io::stdin().read_line(&mut buf)? {
            match_fz_stmt(&buf, &mut counter, &mut level)?;
            buf.clear();
        }
    }
    Ok(())
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
        ParDeclItem::Bool { id, bool } => {
            print!("parameter(bool, {},{}).", identifier(id), bool.to_string())
        }
        ParDeclItem::Int { id, int } => {
            print!("parameter(int, {},{}).", identifier(id), int.to_string())
        }
        ParDeclItem::Float { id, float } => print!(
            "parameter(float, {},{}).",
            identifier(id),
            float.to_string()
        ),
        ParDeclItem::SetOfInt {
            id,
            set_literal: sl,
        } => print!(
            "parameter(set_of_int, {},{}).",
            identifier(id),
            set_literal(sl)
        ),
        ParDeclItem::Array {
            ix,
            par_type,
            id,
            expr,
        } => {
            let array_elements = match expr {
                    ParExpr::ParArrayLiteral(v) => v,
                    other => panic!(
                        "I think this should be an array, but its a basic-literal-expr: {:#?}! Maybe use par_expr?",other
                    ),
                };
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
        VarDeclItem::Bool {
            id,
            expr: None,
            annos,
        } => {
            println!("variable(bool,{}).", identifier(id));
        }
        VarDeclItem::Bool {
            id,
            expr: Some(e),
            annos,
        } => {
            println!("variable(bool,{},{}).", identifier(id), bool_expr(e));
        }
        VarDeclItem::Int {
            id,
            expr: None,
            annos,
        } => {
            println!("variable(int,{}).", identifier(id));
        }
        VarDeclItem::Int {
            id,
            expr: Some(e),
            annos,
        } => {
            println!("variable(int,{},{}).", identifier(id), int_expr(e));
        }
        VarDeclItem::IntInRange {
            id,
            lb,
            ub,
            int: None,
            annos,
        } => {
            println!("variable({},{}).", int_in_range(lb, ub), identifier(id));
        }
        VarDeclItem::IntInRange {
            id,
            lb,
            ub,
            int: Some(e),
            annos,
        } => {
            println!(
                "variable({},{},{}).",
                int_in_range(lb, ub),
                identifier(id),
                int_expr(e)
            );
        }
        VarDeclItem::IntInSet {
            id,
            set,
            int: None,
            annos,
        } => {
            println!("variable({},{}).", int_in_set(set), identifier(id),);
        }
        VarDeclItem::IntInSet {
            id,
            set,
            int: Some(e),
            annos,
        } => {
            println!(
                "variable({},{},{}).",
                int_in_set(set),
                identifier(id),
                int_expr(e)
            );
        }
        VarDeclItem::Float {
            id,
            annos,
            expr: None,
        } => {
            println!("variable(float,{}).", identifier(id));
        }
        VarDeclItem::Float {
            id,
            expr: Some(e),
            annos,
        } => {
            println!("variable(float,{},{}).", identifier(id), float_expr(e));
        }
        VarDeclItem::FloatInRange {
            id,
            lb,
            ub,
            float: None,
            annos,
        } => {
            println!("variable({},{}).", float_in_range(*lb, *ub), identifier(id));
        }
        VarDeclItem::FloatInRange {
            id,
            lb,
            ub,
            float: Some(e),
            annos,
        } => {
            println!(
                "variable({},{},{}).",
                float_in_range(*lb, *ub),
                identifier(id),
                float_expr(e)
            );
        }
        VarDeclItem::SetOfInt {
            id,
            expr: None,
            annos,
        } => {
            println!("variable(set_of_int,{}).", identifier(id));
        }
        VarDeclItem::SetOfInt {
            id,
            annos,
            expr: Some(e),
        } => {
            println!("variable(set_of_int,{},{}).", identifier(id), set_expr(e));
        }
        VarDeclItem::SetOfIntInRange {
            id,
            lb,
            ub,
            expr: None,
            annos,
        } => {
            println!(
                "variable({},{}).",
                set_of_int_in_range(lb, ub),
                identifier(id)
            );
        }
        VarDeclItem::SetOfIntInRange {
            id,
            lb,
            ub,
            expr: Some(e),
            annos,
        } => {
            println!(
                "variable({},{},{}).",
                set_of_int_in_range(lb, ub),
                identifier(id),
                set_expr(e)
            );
        }
        VarDeclItem::SetOfIntInSet {
            id,
            set,
            expr: None,
            annos,
        } => {
            println!("variable({},{}).", set_of_int_in_set(set), identifier(id));
        }
        VarDeclItem::SetOfIntInSet {
            id,
            set,
            expr: Some(e),
            annos,
        } => {
            println!(
                "variable({},{},{}).",
                set_of_int_in_set(set),
                identifier(id),
                set_expr(e)
            );
        }

        VarDeclItem::ArrayOfBool {
            id,
            ix,
            array_literal,
            annos,
        } => {
            println!("variable(array({},bool),{}).", index(ix), identifier(id),);
            for (pos, e) in array_literal.iter().enumerate() {
                println!("in_array({},{},{}).", identifier(id), pos, bool_expr(e));
            }
        }
        VarDeclItem::ArrayOfInt {
            id,
            ix,
            array_literal,
            annos,
        } => {
            println!("variable(array({},int),{}).", index(ix), identifier(id),);
            for (pos, e) in array_literal.iter().enumerate() {
                println!("in_array({},{},{}).", identifier(id), pos, int_expr(e));
            }
        }
        VarDeclItem::ArrayOfIntInRange {
            id,
            ix,
            lb,
            ub,
            array_literal,
            annos,
        } => {
            println!(
                "variable(array({},{}),{}).",
                index(ix),
                int_in_range(lb, ub),
                identifier(id),
            );
            for (pos, e) in array_literal.iter().enumerate() {
                println!("in_array({},{},{}).", identifier(id), pos, int_expr(e));
            }
        }
        VarDeclItem::ArrayOfIntInSet {
            id,
            ix,
            set,
            array_literal,
            annos,
        } => {
            println!(
                "variable(array({},{}),{}).",
                index(ix),
                int_in_set(set),
                identifier(id),
            );
            for (pos, e) in array_literal.iter().enumerate() {
                println!("in_array({},{},{}).", identifier(id), pos, int_expr(e));
            }
        }
        VarDeclItem::ArrayOfFloat {
            id,
            ix,
            annos,
            array_literal,
        } => {
            println!("variable(array({},float),{}).", index(ix), identifier(id),);
            for (pos, e) in array_literal.iter().enumerate() {
                println!("in_array({},{},{}).", identifier(id), pos, float_expr(e));
            }
        }
        VarDeclItem::ArrayOfFloatInRange {
            id,
            ix,
            lb,
            ub,
            array_literal,
            annos,
        } => {
            println!(
                "variable(array({},{}),{}).",
                index(ix),
                float_in_range(*lb, *ub),
                identifier(id),
            );
            for (pos, e) in array_literal.iter().enumerate() {
                println!("in_array({},{},{}).", identifier(id), pos, float_expr(e));
            }
        }
        VarDeclItem::ArrayOfSet {
            id,
            ix,
            array_literal,
            annos,
        } => {
            println!("variable(array({},set),{}).", index(ix), identifier(id),);
            for (pos, e) in array_literal.iter().enumerate() {
                println!("in_array({},{},{}).", identifier(id), pos, set_expr(e));
            }
        }
        VarDeclItem::ArrayOfSetOfIntInRange {
            id,
            ix,
            lb,
            ub,
            array_literal,
            annos,
        } => {
            println!(
                "variable(array({},{}),{}).",
                index(ix),
                set_of_int_in_range(lb, ub),
                identifier(id),
            );
            for (pos, e) in array_literal.iter().enumerate() {
                println!("in_array({},{},{}).", identifier(id), pos, set_expr(e));
            }
        }
        VarDeclItem::ArrayOfSetOfIntInSet {
            id,
            ix,
            set,
            array_literal,
            annos,
        } => {
            println!(
                "variable(array({},{}),{}).",
                index(ix),
                set_of_int_in_set(set),
                identifier(id),
            );
            for (pos, e) in array_literal.iter().enumerate() {
                println!("in_array({},{},{}).", identifier(id), pos, set_expr(e));
            }
        }
    }
}
fn basic_var_type(t: &BasicVarType) -> String {
    match t {
        BasicVarType::Bool => "bool".to_string(),
        // BasicVarType::Domain(d) => domain(d),
        BasicVarType::Int => "int".to_string(),
        BasicVarType::IntInRange(lb, ub) => int_in_range(lb, ub),
        BasicVarType::IntInSet(set) => int_in_set(set),
        BasicVarType::Float => "float".to_string(),
        BasicVarType::FloatInRange(lb, ub) => float_in_range(*lb, *ub),
        BasicVarType::SetOfInt => "set_of_int".to_string(),
        BasicVarType::SetOfIntInRange(lb, ub) => set_of_int_in_range(lb, ub),
        BasicVarType::SetOfIntInSet(set) => set_of_int_in_set(set),
    }
}
// TODO implement sets
fn int_in_range(lb: &i128, ub: &i128) -> String {
    format!("int_in_range({},{})", lb, ub)
}
fn int_in_set(_set: &[i128]) -> String {
    panic!("TODO: int_in_set ..")
}
fn float_in_range(lb: f64, ub: f64) -> String {
    format!("float_in_range(\"{}\",\"{}\")", lb, ub)
}
fn set_of_int_in_range(lb: &i128, ub: &i128) -> String {
    format!("set_of_int_in_range(\"{}\",\"{}\")", lb, ub)
}
fn set_of_int_in_set(_set: &[i128]) -> String {
    panic!("TODO: set_of_int_in_set ..")
}
fn print_constraint(c: &ConstraintItem, i: usize) {
    println!("constraint(c{},{}).", i, identifier(&c.id));
    for (cpos, ce) in c.exprs.iter().enumerate() {
        match ce {
            Expr::VarParIdentifier(id) => {
                println!("in_constraint(c{},{},{}).", i, cpos, identifier(id))
            }
            Expr::Bool(e) => println!("in_constraint(c{},{},{}).", i, cpos, bool_literal(*e)),
            Expr::Int(e) => println!("in_constraint(c{},{},{}).", i, cpos, int_literal(&e)),
            Expr::Float(e) => println!("in_constraint(c{},{},{}).", i, cpos, float_literal(*e)),
            Expr::Set(e) => println!("in_constraint(c{},{},{}).", i, cpos, set_literal(&e)),
            Expr::ArrayOfBool(v) => {
                println!("in_constraint(c{},{},array).", i, cpos);
                for (apos, ae) in v.iter().enumerate() {
                    println!(
                        "in_constraint(c{},{},{},{}).",
                        i,
                        cpos,
                        apos,
                        bool_expr(&ae)
                    );
                }
            }
            Expr::ArrayOfInt(v) => {
                println!("in_constraint(c{},{},array).", i, cpos);
                for (apos, ae) in v.iter().enumerate() {
                    println!("in_constraint(c{},{},{},{}).", i, cpos, apos, int_expr(&ae));
                }
            }
            Expr::ArrayOfFloat(v) => {
                println!("in_constraint(c{},{},array).", i, cpos);
                for (apos, ae) in v.iter().enumerate() {
                    println!(
                        "in_constraint(c{},{},{},{}).",
                        i,
                        cpos,
                        apos,
                        float_expr(&ae)
                    );
                }
            }
            Expr::ArrayOfSet(v) => {
                println!("in_constraint(c{},{},array).", i, cpos);
                for (apos, ae) in v.iter().enumerate() {
                    println!("in_constraint(c{},{},{},{}).", i, cpos, apos, set_expr(&ae));
                }
            }
        }
    }
}
fn print_solve_item(i: &SolveItem) {
    match &i.goal {
        Goal::Satisfy => println!("solve(satisfy)."),
        Goal::OptimizeBool(ot, e) => println!("solve({},{}).", opt_type(ot), bool_expr(&e)),
        Goal::OptimizeInt(ot, e) => println!("solve({},{}).", opt_type(ot), int_expr(&e)),
        Goal::OptimizeFloat(ot, e) => println!("solve({},{}).", opt_type(ot), float_expr(&e)),
        Goal::OptimizeSet(ot, e) => println!("solve({},{}).", opt_type(ot), set_expr(&e)),
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
        BasicPredParType::FloatInRange(lb, ub) => float_in_range(*lb, *ub),
        BasicPredParType::IntInRange(lb, ub) => int_in_range(lb, ub),
        BasicPredParType::IntInSet(set) => int_in_set(set),
        BasicPredParType::SetOfIntInRange(lb, ub) => set_of_int_in_range(lb, ub),
        BasicPredParType::SetOfIntInSet(set) => set_of_int_in_set(set),
    }
}
fn opt_type(opt_type: &OptimizationType) -> String {
    match opt_type {
        OptimizationType::Minimize => "minimize".to_string(),
        OptimizationType::Maximize => "maximize".to_string(),
    }
}
fn index(IndexSet(i): &IndexSet) -> String {
    i.to_string()
}
fn identifier(id: &str) -> String {
    format!("\"{}\"", id)
}
fn pred_index(is: &PredIndexSet) -> String {
    match is {
        PredIndexSet::IndexSet(i) => i.to_string(),
        PredIndexSet::Int => "int".to_string(),
    }
}
fn bool_expr(e: &BoolExpr) -> String {
    match e {
        BoolExpr::Bool(b) => bool_literal(*b),
        BoolExpr::VarParIdentifier(id) => identifier(id),
    }
}
fn bool_literal(b: bool) -> String {
    if b {
        "true".to_string()
    } else {
        "false".to_string()
    }
}
fn int_expr(e: &IntExpr) -> String {
    match e {
        IntExpr::Int(i) => int_literal(i),
        IntExpr::VarParIdentifier(id) => identifier(id),
    }
}
fn int_literal(i: &i128) -> String {
    i.to_string()
}
fn float_expr(e: &FloatExpr) -> String {
    match e {
        FloatExpr::Float(f) => float_literal(*f),
        FloatExpr::VarParIdentifier(id) => identifier(id),
    }
}
fn float_literal(f: f64) -> String {
    format!("\"{}\"", f)
}
fn set_expr(e: &SetExpr) -> String {
    match e {
        SetExpr::Set(sl) => set_literal(sl),
        SetExpr::VarParIdentifier(id) => identifier(id),
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
        SetLiteral::FloatRange(f1, f2) => format!("range_f(\"{}\",\"{}\")", f1, f2),
        SetLiteral::IntRange(i1, i2) => format!("range_i({},{})", i1, i2),
        SetLiteral::SetFloats(v) => panic!("TODO: set_floats(v)"),
        SetLiteral::SetInts(v) => panic!("TODO: set_ints(v)"),
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
