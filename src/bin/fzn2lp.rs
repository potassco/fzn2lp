use anyhow::Result;
use flatzinc::*;
use nom::error::{convert_error, VerboseError};
use nom::Err;
use std::path::PathBuf;
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
    let opt = Opt::from_args();
    let buf = std::fs::read_to_string(opt.file)?;
    match flatzinc::model::<VerboseError<&str>>(&buf) {
        Ok((_, result)) => fzn2lp(&result),

        Err(Err::Error(e)) | Err(Err::Failure(e)) => {
            println!("Failed to parse flatzinc!\n{}", convert_error(&buf, e))
        }
        Err(e) => println!("Failed to parse flatzinc: {:?}", e),
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
            par_expr(expr)
        ),
        ParDeclItem::Array {
            ix,
            par_type,
            id,
            expr,
        } => print!(
            "parameter(array({},{}), id_{},{}).",
            index(ix),
            basic_par_type(&par_type),
            id,
            par_expr(expr)
        ),
    }
}
fn print_var_decl_item(d: &flatzinc::VarDeclItem) {
    match d {
        VarDeclItem::Array(ix, t, id, annos, v) => {
            let mut x = String::new();
            for e in v {
                if x.is_empty() {
                    x = format!("{}", basic_expr(e));
                } else {
                    x = format!("{},{}", x, basic_expr(e));
                }
            }
            // panic!(
            //     "# TODO ARRAY LITERAL in variable declaration
            //     \n- annos: {:#?}
            //     \n- expressions: {:#?}",
            //     annos, x
            // );
            println!(
                "variable(array({},{}),id_{}).
# TODO ARRAY LITERAL in variable declaration",
                index(ix),
                basic_var_type(t),
                id,
            );
        }
        VarDeclItem::Basic(t, id, annos, e) => {
            println!("variable({},id_{}).", basic_var_type(t), id);
        }
    }
}
fn basic_var_type(t: &BasicVarType) -> String {
    match t {
        BasicVarType::Bool => format!("bool"),
        BasicVarType::Domain(d) => format!("{}", domain(d)),
        BasicVarType::Float => format!("float"),
        BasicVarType::Int => format!("int"),
    }
}
fn domain(d: &Domain) -> String {
    match d {
        Domain::FloatRange(f1, f2) => format!("range_f({},{})", f1, f2),
        Domain::IntRange(i1, i2) => format!("range_i({},{})", i1, i2),
        Domain::SetInt(v) => format!("TODO SET INT"),
        Domain::SetIntNonEmpty(v) => format!("TODO NON_EMPTY SET INT"),
        Domain::SetIntRange(i1, i2) => format!("set_int_range({},{})", i1, i2),
    }
}

fn print_constraint(c: &ConstraintItem) {
    println!("constraint(id_{})", c.id);
    for e in &c.exprs {
        println!("in_constraint(id_{},{}).", c.id, expr(&e))
    }
}
fn print_solve_item(i: &SolveItem) {
    println!("solve{:?}", i);
}
fn basic_par_type(t: &BasicParType) -> String {
    match t {
        BasicParType::Bool => format!("bool"),
        BasicParType::Float => format!("float"),
        BasicParType::Int => format!("int"),
        BasicParType::SetOfInt => format!("set_of_int"),
    }
}
fn index(IndexSet(i): &IndexSet) -> String {
    format!("{}", i)
}
fn expr(e: &Expr) -> String {
    match e {
        Expr::ArrayLiteral(a) => format!(
            "array()
# TODO ARRAY LITERAL in constraint"
        ),
        Expr::BasicExpr(e) => basic_expr(&e),
    }
}
fn par_expr(e: &ParExpr) -> String {
    match e {
        ParExpr::ParArrayLiteral(v) => {
            let mut x = String::new();
            for e in v {
                if x.is_empty() {
                    x = format!("{}", basic_literal_expr(e));
                } else {
                    x = format!("{},{}", x, basic_literal_expr(e));
                }
            }
            format!(
                "array()
# TODO ARRAY LITERAL in parameter declaration
            \n- expresions: {}",
                x
            )
        }
        ParExpr::BasicLiteralExpr(l) => basic_literal_expr(l),
    }
}

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
        BasicLiteralExpr::SetLiteral(s) => format!("{}", set_literal(s)),
    }
}
fn set_literal(l: &SetLiteral) -> String {
    match l {
        SetLiteral::FloatRange(f1, f2) => format!("range_i({},{})", f1, f2),
        SetLiteral::IntRange(i1, i2) => format!("range_i({},{})", i1, i2),
        SetLiteral::SetFloats(v) => format!("{}", set_floats(v)),
        SetLiteral::SetInts(v) => format!("{}", set_ints(v)),
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
