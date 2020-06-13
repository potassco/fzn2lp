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
    let mut constraint_counter = 1;

    if let Some(file) = opt.file {
        let file = File::open(file)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            print_fz_stmt(&line?, &mut constraint_counter, &mut level)?;
        }
    } else {
        let mut buf = String::new();
        while 0 < io::stdin().read_line(&mut buf)? {
            print_fz_stmt(&buf, &mut constraint_counter, &mut level)?;
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
fn print_fz_stmt(
    input: &str,
    constraint_counter: &mut usize,
    level: &mut i32,
) -> Result<(), FlatZincError> {
    match fz_statement::<VerboseError<&str>>(&input) {
        Ok((_rest, stmt)) => {
            match stmt {
                FzStmt::Comment(s) => {
                    println!("%{}", s);
                }
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
                    print_constraint(&c, *constraint_counter);
                    *constraint_counter += 1;
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
            (PredParType::Basic(par_type), id) => {
                for element in basic_pred_par_type(&par_type) {
                    println!(
                        "predicate_parameter({},{},{},{}).",
                        identifier(&item.id),
                        pos,
                        element,
                        identifier(id)
                    )
                }
            }
            (PredParType::Array { ix, par_type }, id) => {
                for element in basic_pred_par_type(&par_type) {
                    println!(
                        "predicate_parameter({},{},array({},{}),{}).",
                        identifier(&item.id),
                        pos,
                        pred_index(&ix),
                        element,
                        identifier(id)
                    )
                }
            }
        }
    }
}
fn print_par_decl_item(item: &ParDeclItem) {
    match item {
        ParDeclItem::Bool { id, bool } => {
            println!("parameter(bool, {},{}).", identifier(id), bool.to_string());
        }
        ParDeclItem::Int { id, int } => {
            println!("parameter(int, {},{}).", identifier(id), int.to_string())
        }
        ParDeclItem::Float { id, float } => println!(
            "parameter(float, {},{}).",
            identifier(id),
            float.to_string()
        ),
        ParDeclItem::SetOfInt {
            id,
            set_literal: sl,
        } => {
            let set = set_literal(sl);
            for element in set {
                println!("parameter(set_of_int, {},{}).", identifier(id), element);
            }
        }
        ParDeclItem::ArrayOfBool { ix, id, expr } => {
            for (pos, e) in expr.iter().enumerate() {
                println!(
                    "parameter(array({},bool),{},array_of_bool({},{})).",
                    index(ix),
                    identifier(id),
                    pos,
                    bool_literal(*e)
                );
            }
        }
        ParDeclItem::ArrayOfInt { ix, id, expr } => {
            for (pos, e) in expr.iter().enumerate() {
                println!(
                    "parameter(array({},int),{},array_of_int({},{})).",
                    index(ix),
                    identifier(id),
                    pos,
                    int_literal(e)
                );
            }
        }
        ParDeclItem::ArrayOfFloat { ix, id, expr } => {
            for (pos, e) in expr.iter().enumerate() {
                println!(
                    "parameter(array({},float),{},array_of_float({},{})).",
                    index(ix),
                    identifier(id),
                    pos,
                    float_literal(*e)
                );
            }
        }
        ParDeclItem::ArrayOfSet { ix, id, expr } => {
            for (pos, e) in expr.iter().enumerate() {
                let set = set_literal(e);
                for element in set {
                    println!(
                        "parameter(array({},set),{},array_of_set({},{})).",
                        index(ix),
                        identifier(id),
                        pos,
                        element
                    );
                }
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
            for element in set {
                println!("variable(int_in_set({}),{}).", element, identifier(id));
            }
        }
        VarDeclItem::IntInSet {
            id,
            set,
            int: Some(e),
            annos,
        } => {
            for element in set {
                println!(
                    "variable(int_in_set({}),{},{}).",
                    element,
                    identifier(id),
                    int_expr(e)
                );
            }
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
            let set = set_expr(e);
            for element in set {
                println!("variable(set_of_int,{},{}).", identifier(id), element);
            }
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
            let set = set_expr(e);
            for element in set {
                println!(
                    "variable({},{},{}).",
                    set_of_int_in_range(lb, ub),
                    identifier(id),
                    element
                );
            }
        }
        VarDeclItem::SetOfIntInSet {
            id,
            set,
            expr: None,
            annos,
        } => {
            for type_element in set {
                println!(
                    "variable(set_of_int_in_set({}),{}).",
                    type_element,
                    identifier(id)
                );
            }
        }
        VarDeclItem::SetOfIntInSet {
            id,
            set,
            expr: Some(e),
            annos,
        } => {
            for type_element in set {
                let value_set = set_expr(e);
                for element in value_set {
                    println!(
                        "variable(set_of_int_in_set({}),{},{}).",
                        type_element,
                        identifier(id),
                        element
                    );
                }
            }
        }

        VarDeclItem::ArrayOfBool {
            id,
            ix,
            array_literal,
            annos,
        } => {
            for (pos, e) in array_literal.iter().enumerate() {
                println!(
                    "variable(array({},bool),{},array_of_bool({},{})).",
                    index(ix),
                    identifier(id),
                    pos,
                    bool_expr(e)
                );
            }
        }
        VarDeclItem::ArrayOfInt {
            id,
            ix,
            array_literal,
            annos,
        } => {
            for (pos, e) in array_literal.iter().enumerate() {
                println!(
                    "variable(array({},int),{},array_of_int({},{})).",
                    index(ix),
                    identifier(id),
                    pos,
                    int_expr(e)
                );
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
            for (pos, e) in array_literal.iter().enumerate() {
                println!(
                    "variable(array({},int),{},array_of_int_in_range({},{},{},{})).",
                    index(ix),
                    identifier(id),
                    lb,
                    ub,
                    pos,
                    int_expr(e)
                );
            }
        }
        VarDeclItem::ArrayOfIntInSet {
            id,
            ix,
            set,
            array_literal,
            annos,
        } => {
            for (pos, e) in array_literal.iter().enumerate() {
                for element in set {
                    println!(
                        "variable(array({},int_in_set({})),{},{},{}).",
                        index(ix),
                        element,
                        identifier(id),
                        pos,
                        int_expr(e)
                    );
                }
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
                let set = set_expr(e);
                for element in set {
                    println!("in_array({},{},{}).", identifier(id), pos, element);
                }
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
                let set = set_expr(e);
                for element in set {
                    println!("in_array({},{},{}).", identifier(id), pos, element);
                }
            }
        }
        VarDeclItem::ArrayOfSetOfIntInSet {
            id,
            ix,
            set,
            array_literal,
            annos,
        } => {
            for (pos, e) in array_literal.iter().enumerate() {
                for type_element in set {
                    let setl = set_expr(e);
                    for element in setl {
                        println!(
                            "variable(array({},set_of_int_in_set({})),{},{},{}).",
                            index(ix),
                            type_element,
                            identifier(id),
                            pos,
                            element
                        );
                    }
                }
            }
        }
    }
}
fn basic_var_type(t: &BasicVarType) -> Vec<String> {
    match t {
        BasicVarType::Bool => vec!["bool".to_string()],
        BasicVarType::Int => vec!["int".to_string()],
        BasicVarType::IntInRange(lb, ub) => vec![int_in_range(lb, ub)],
        BasicVarType::IntInSet(set) => int_in_set(set),
        BasicVarType::Float => vec!["float".to_string()],
        BasicVarType::FloatInRange(lb, ub) => vec![float_in_range(*lb, *ub)],
        BasicVarType::SetOfInt => vec!["set_of_int".to_string()],
        BasicVarType::SetOfIntInRange(lb, ub) => vec![set_of_int_in_range(lb, ub)],
        BasicVarType::SetOfIntInSet(set) => set_of_int_in_set(set),
    }
}
// TODO implement sets
fn int_in_range(lb: &i128, ub: &i128) -> String {
    format!("int_in_range({},{})", lb, ub)
}
fn int_in_set(set: &[i128]) -> Vec<String> {
    let mut ret = vec![];
    for integer in set {
        ret.push(format!("int_in_set({})", integer))
    }
    ret
}
fn float_in_range(lb: f64, ub: f64) -> String {
    format!("float_in_range(\"{}\",\"{}\")", lb, ub)
}
fn set_of_int_in_range(lb: &i128, ub: &i128) -> String {
    format!("set_of_int_in_range(\"{}\",\"{}\")", lb, ub)
}
fn set_of_int_in_set(set: &[i128]) -> Vec<String> {
    let mut ret = vec![];
    for integer in set {
        ret.push(format!("set_of_int_in_set({})", integer))
    }
    ret
}
fn print_constraint(c: &ConstraintItem, i: usize) {
    println!("constraint(c{},{}).", i, identifier(&c.id));
    for (cpos, ce) in c.exprs.iter().enumerate() {
        match ce {
            Expr::VarParIdentifier(id) => {
                println!("in_constraint(c{},{},{}).", i, cpos, identifier(id))
            }
            Expr::Bool(e) => println!("in_constraint(c{},{},{}).", i, cpos, bool_expr(&e)),
            Expr::Int(e) => println!("in_constraint(c{},{},{}).", i, cpos, int_expr(&e)),
            Expr::Float(e) => println!("in_constraint(c{},{},{}).", i, cpos, float_expr(&e)),
            Expr::Set(e) => {
                let set = set_expr(&e);
                for element in set {
                    println!("in_constraint(c{},{},{}).", i, cpos, element);
                }
            }
            Expr::ArrayOfBool(v) => {
                for (apos, ae) in v.iter().enumerate() {
                    println!(
                        "in_constraint(c{},{},array_of_bool({},{})).",
                        i,
                        cpos,
                        apos,
                        bool_expr(&ae)
                    );
                }
            }
            Expr::ArrayOfInt(v) => {
                for (apos, ae) in v.iter().enumerate() {
                    println!(
                        "in_constraint(c{},{},array_of_int({},{})).",
                        i,
                        cpos,
                        apos,
                        int_expr(&ae)
                    );
                }
            }
            Expr::ArrayOfFloat(v) => {
                for (apos, ae) in v.iter().enumerate() {
                    println!(
                        "in_constraint(c{},{},array_of_float({},{})).",
                        i,
                        cpos,
                        apos,
                        float_expr(&ae)
                    );
                }
            }
            Expr::ArrayOfSet(v) => {
                for (apos, ae) in v.iter().enumerate() {
                    let set = set_expr(ae);
                    for element in set {
                        println!(
                            "in_constraint(c{},{},array_of_set({},{})).",
                            i, cpos, apos, element
                        );
                    }
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
        Goal::OptimizeSet(ot, e) => {
            let set = set_expr(e);
            for element in set {
                println!("solve({},{}).", opt_type(ot), element);
            }
        }
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
fn basic_pred_par_type(t: &BasicPredParType) -> Vec<String> {
    match t {
        BasicPredParType::BasicParType(t) => vec![basic_par_type(t)],
        BasicPredParType::BasicVarType(t) => basic_var_type(t),
        BasicPredParType::FloatInRange(lb, ub) => vec![float_in_range(*lb, *ub)],
        BasicPredParType::IntInRange(lb, ub) => vec![int_in_range(lb, ub)],
        BasicPredParType::IntInSet(set) => int_in_set(set),
        BasicPredParType::SetOfIntInRange(lb, ub) => vec![set_of_int_in_range(lb, ub)],
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
fn set_expr(e: &SetExpr) -> Vec<String> {
    match e {
        SetExpr::Set(sl) => set_literal_expr(sl),
        SetExpr::VarParIdentifier(id) => vec![identifier(id)],
    }
}
fn set_literal_expr(l: &SetLiteralExpr) -> Vec<String> {
    let mut ret = Vec::new();
    match l {
        SetLiteralExpr::FloatRange(f1, f2) => ret.push(format!(
            "set_float_range({},{})",
            float_expr(f1),
            float_expr(f2)
        )),
        SetLiteralExpr::IntRange(i1, i2) => {
            ret.push(format!("set_int_range({},{})", int_expr(i1), int_expr(i2)))
        }
        SetLiteralExpr::SetFloats(v) => {
            for f in v {
                ret.push(format!("set_of_float(\"{}\")", float_expr(f)));
            }
        }
        SetLiteralExpr::SetInts(v) => {
            for i in v {
                ret.push(format!("set_of_int({})", int_expr(i)));
            }
        }
    }
    ret
}
fn set_literal(l: &SetLiteral) -> Vec<String> {
    let mut ret = Vec::new();
    match l {
        SetLiteral::FloatRange(f1, f2) => ret.push(format!(
            "set_float_range({},{})",
            float_literal(*f1),
            float_literal(*f2)
        )),
        SetLiteral::IntRange(i1, i2) => ret.push(format!("set_int_range({},{})", i1, i2)),
        SetLiteral::SetFloats(v) => {
            for f in v {
                ret.push(format!("set_of_float(\"{}\")", f));
            }
        }
        SetLiteral::SetInts(v) => {
            for f in v {
                ret.push(format!("set_of_int({})", f));
            }
        }
    }
    ret
}
