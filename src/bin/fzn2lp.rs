use anyhow::Result;
use flatzinc::*;
use log::error;
use nom::error::{convert_error, VerboseError};
use nom::Err;
use std::path::PathBuf;
use stderrlog;
use structopt::StructOpt;

/// Convert FlatZinc to AnsProlog facts
#[derive(StructOpt, Debug)]
#[structopt(name = "fzn2lp")]
struct Opt {
    /// Input in flatzinc format
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    file: PathBuf,
}

fn main() -> Result<()> {
    stderrlog::new()
        .module(module_path!())
        .verbosity(2)
        .init()
        .unwrap();

    let opt = Opt::from_args();
    let buf = std::fs::read_to_string(opt.file)?;
    match flatzinc::model::<VerboseError<&str>>(&buf) {
        Ok((_, result)) => fzn2lp(&result),

        Err(Err::Error(e)) | Err(Err::Failure(e)) => {
            error!("Failed to parse flatzinc!\n{}", convert_error(&buf, e))
        }
        Err(e) => error!("Failed to parse flatzinc: {:?}", e),
    }
    Ok(())
}

fn fzn2lp(model: &flatzinc::Model) {
    for i in &model.predicate_items {
        print_predicate(i);
    }
    for i in &model.par_decl_items {
        print_par_decl_item(i);
    }
    for i in &model.var_decl_items {
        print_var_decl_item(i);
    }
    for c in &model.constraint_items {
        print_constraint(c);
    }
    print_solve_item(&model.solve_item)
}
fn print_predicate(p: &PredicateItem) {
    println!("predicate({:?}", p.id);
    for p in &p.parameters {
        print!(",{:?}", p);
    }
    println!(").");
}
fn print_par_decl_item(p: &ParDeclItem) {
    match p {
        ParDeclItem::BasicParType { par_type, id, expr } => print!(
            "parameter({}, {},id_{}).",
            basic_par_type(&par_type),
            id,
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
                "parameter(array({},{}), id_{}).",
                index(ix),
                basic_par_type(&par_type),
                id
            );
            for (pos, e) in array_elements.iter().enumerate() {
                println!("in_array(id_{},{},{}).", id, pos, basic_literal_expr(e));
            }
        }
    }
}
fn print_var_decl_item(d: &flatzinc::VarDeclItem) {
    match d {
        VarDeclItem::Array(ix, t, id, annos, v) => {
            println!(
                "variable(array({},{}),id_{}).",
                index(ix),
                basic_var_type(t),
                id,
            );
            for (pos, e) in v.iter().enumerate() {
                println!("in_array(id_{},{},{}).", id, pos, basic_expr(e));
            }
        }
        VarDeclItem::Basic(t, id, annos, None) => {
            println!("variable({},id_{}).", basic_var_type(t), id);
        }
        VarDeclItem::Basic(t, id, annos, Some(e)) => {
            println!(
                "variable({},id_{},{}).",
                basic_var_type(t),
                id,
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
        BasicVarType::VarSetOFInt => "set_of_int".to_string(),
    }
}
fn domain(d: &Domain) -> String {
    match d {
        Domain::FloatRange(f1, f2) => format!("range_f({},{})", f1, f2),
        Domain::IntRange(i1, i2) => format!("range_i({},{})", i1, i2),
        Domain::SetInt(v) => format!("TODO NON_EMPTY SET INT"),
        Domain::SetIntNonEmpty(v) => format!("TODO NON_EMPTY SET INT"),
        Domain::SetIntRange(i1, i2) => format!("set_int_range({},{})", i1, i2),
    }
}

fn print_constraint(c: &ConstraintItem) {
    println!("constraint(id_{})", c.id);
    for (cpos, ce) in c.exprs.iter().enumerate() {
        match ce {
            Expr::BasicExpr(e) => {
                println!("in_constraint(id_{},{},{}).", c.id, cpos, basic_expr(&e))
            }
            Expr::ArrayLiteral(v) => {
                println!("in_constraint(id_{},{},array).", c.id, cpos);
                for (apos, ae) in v.iter().enumerate() {
                    println!(
                        "in_constraint({},{},{},{}).",
                        c.id,
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
    println!("solve{:?}", i);
}
fn basic_par_type(t: &BasicParType) -> String {
    match t {
        BasicParType::Bool => "bool".to_string(),
        BasicParType::Float => "float".to_string(),
        BasicParType::Int => "int".to_string(),
        BasicParType::SetOfInt => "set_of_int".to_string(),
    }
}
fn index(IndexSet(i): &IndexSet) -> String {
    format!("{}", i)
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
        BasicExpr::VarParIdentifier(s) => format!("id_{}", s),
    }
}
fn basic_literal_expr(e: &BasicLiteralExpr) -> String {
    match e {
        BasicLiteralExpr::BoolLiteral(b) => format!("{}", b),
        BasicLiteralExpr::FloatLiteral(f) => format!("{}", f),
        BasicLiteralExpr::IntLiteral(i) => format!("{}", i),
        BasicLiteralExpr::SetLiteral(s) => set_literal(s),
    }
}
fn set_literal(l: &SetLiteral) -> String {
    match l {
        SetLiteral::FloatRange(f1, f2) => format!("range_i({},{})", f1, f2),
        SetLiteral::IntRange(i1, i2) => format!("range_i({},{})", i1, i2),
        SetLiteral::SetFloats(v) => set_floats(v),
        SetLiteral::SetInts(v) => set_ints(v),
    }
}
fn set_floats(v: &[f64]) -> String {
    let mut x = String::new();
    for f in v {
        if x.is_empty() {
            x = format!("{}", f);
        } else {
            x = format!("{},{}", x, f);
        }
    }
    format!("set_floats({}).", x)
}
fn set_ints(v: &[i128]) -> String {
    let mut x = String::new();
    for f in v {
        if x.is_empty() {
            x = format!("{}", f);
        } else {
            x = format!("{},{}", x, f);
        }
    }
    format!("set_ints({}).", x)
}
