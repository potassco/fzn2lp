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
    if level < 5 {
        Err(FlatZincError::NoSolveItem)?;
    }
    Ok(())
}
use thiserror::Error;
#[derive(Error, Debug)]
pub enum FlatZincError {
    #[error("More than one solve item")]
    MultipleSolveItems,
    #[error("No solve item")]
    NoSolveItem,
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

fn print_predicate(predicate: &PredicateItem) {
    println!("predicate({}).", identifier(&predicate.id));
    for (pos, p) in predicate.parameters.iter().enumerate() {
        match p {
            (PredParType::Basic(par_type), id) => {
                for element in basic_pred_par_type(&par_type) {
                    println!(
                        "predicate_parameter({},{},{},{}).",
                        identifier(&predicate.id),
                        pos,
                        identifier(id),
                        element
                    )
                }
            }
            (PredParType::Array { ix, par_type }, id) => {
                for element in basic_pred_par_type(&par_type) {
                    println!(
                        "predicate_parameter({},{},{},{}).",
                        identifier(&predicate.id),
                        pos,
                        identifier(id),
                        array_type(&pred_index(&ix), &element)
                    )
                }
            }
        }
    }
}
fn print_par_decl_item(item: &ParDeclItem) {
    match item {
        ParDeclItem::Bool { id, bool } => {
            println!("parameter_type({},bool).", identifier(id));
            println!(
                "parameter_value({},value,{}).",
                identifier(id),
                bool_literal(*bool)
            );
        }
        ParDeclItem::Int { id, int } => {
            println!("parameter_type({},int).", identifier(id));
            println!(
                "parameter_value({},value,{}).",
                identifier(id),
                int_literal(int)
            );
        }
        ParDeclItem::Float { id, float } => {
            println!("parameter_type({},float).", identifier(id));
            println!(
                "parameter_value({},value,{}).",
                identifier(id),
                float_literal(*float)
            );
        }
        ParDeclItem::SetOfInt {
            id,
            set_literal: sl,
        } => {
            println!("parameter_type({},set_of_int).", identifier(id));
            let set = dec_set_literal(sl);
            for element in set {
                println!("parameter_value({},{}).", identifier(id), element);
            }
        }
        ParDeclItem::ArrayOfBool { ix, id, v } => {
            println!(
                "parameter_type({},{}).",
                identifier(id),
                array_type(&index(ix), "bool")
            );
            for (pos, e) in v.iter().enumerate() {
                println!(
                    "parameter_value({},array,({},{})).",
                    identifier(id),
                    pos,
                    bool_literal(*e)
                );
            }
        }
        ParDeclItem::ArrayOfInt { ix, id, v } => {
            println!(
                "parameter_type({},{}).",
                identifier(id),
                array_type(&index(ix), "int")
            );
            for (pos, e) in v.iter().enumerate() {
                println!(
                    "parameter_value({},array,({},{})).",
                    identifier(id),
                    pos,
                    int_literal(e)
                );
            }
        }
        ParDeclItem::ArrayOfFloat { ix, id, v } => {
            println!(
                "parameter_type({},{}).",
                identifier(id),
                array_type(&index(ix), "float"),
            );
            for (pos, e) in v.iter().enumerate() {
                println!(
                    "parameter_value({},array,({},{})).",
                    identifier(id),
                    pos,
                    float_literal(*e)
                );
            }
        }
        ParDeclItem::ArrayOfSet { ix, id, v } => {
            println!(
                "parameter_type({},{}).",
                identifier(id),
                array_type(&index(ix), "set")
            );
            for (pos, e) in v.iter().enumerate() {
                let set = dec_set_literal(e);
                for element in set {
                    println!("parameter({},array,({},{})).", identifier(id), pos, element);
                }
            }
        }
    }
}
fn print_var_decl_item(item: &VarDeclItem) {
    match item {
        VarDeclItem::Bool { id, expr, annos } => {
            println!("variable_type({},bool).", identifier(id));
            if let Some(expr) = expr {
                println!(
                    "variable_value({},value,{}).",
                    identifier(id),
                    bool_expr(expr)
                );
            }
        }
        VarDeclItem::Int { id, expr, annos } => {
            println!("variable_type({},int).", identifier(id));
            if let Some(expr) = expr {
                println!(
                    "variable_value({},value,{}).",
                    identifier(id),
                    int_expr(expr)
                );
            }
        }
        VarDeclItem::IntInRange {
            id,
            lb,
            ub,
            expr,
            annos,
        } => {
            println!(
                "variable_type({},{}).",
                identifier(id),
                int_in_range(lb, ub)
            );
            if let Some(expr) = expr {
                println!(
                    "variable_value({},value,{}).",
                    identifier(id),
                    int_expr(expr)
                );
            }
        }
        VarDeclItem::IntInSet {
            id,
            set,
            expr,
            annos,
        } => {
            for element in set {
                println!("variable_type({},int_in_set({})).", identifier(id), element,);
            }
            if let Some(expr) = expr {
                println!(
                    "variable_value({},value,{}).",
                    identifier(id),
                    int_expr(expr)
                );
            }
        }
        VarDeclItem::Float { id, expr, annos } => {
            println!("variable_type({},float).", identifier(id));
            if let Some(expr) = expr {
                println!(
                    "variable_value({},value,{}).",
                    identifier(id),
                    float_expr(expr)
                );
            }
        }
        VarDeclItem::BoundedFloat {
            id,
            lb,
            ub,
            expr,
            annos,
        } => {
            println!(
                "variable_type({},{}).",
                identifier(id),
                bounded_float(*lb, *ub)
            );
            if let Some(expr) = expr {
                println!(
                    "variable_value({},value,{}).",
                    identifier(id),
                    float_expr(expr)
                );
            }
        }
        VarDeclItem::SetOfInt { id, annos, expr } => {
            println!("variable_type({},set_of_int).", identifier(id));
            if let Some(expr) = expr {
                let set = dec_set_expr(expr);
                for element in set {
                    println!("variable_value({},{}).", identifier(id), element);
                }
            }
        }
        VarDeclItem::SubSetOfIntRange {
            id,
            lb,
            ub,
            expr,
            annos,
        } => {
            println!(
                "variable_type({},{}).",
                identifier(id),
                subset_of_int_range(lb, ub),
            );
            if let Some(expr) = expr {
                let set = dec_set_expr(expr);
                for element in set {
                    println!("variable_value({},{}).", identifier(id), element);
                }
            }
        }
        VarDeclItem::SubSetOfIntSet {
            id,
            set,
            expr,
            annos,
        } => {
            for element in set {
                println!(
                    "variable_type({},subset_of_int_set({})).",
                    identifier(id),
                    element,
                );
            }
            if let Some(expr) = expr {
                let set = dec_set_expr(expr);
                for element in set {
                    println!("variable_value({},{}).", identifier(id), element);
                }
            }
        }

        VarDeclItem::ArrayOfBool {
            id,
            ix,
            array_expr,
            annos,
        } => {
            println!(
                "variable_type({},{}).",
                identifier(id),
                array_type(&index(ix), "bool")
            );
            match array_expr {
                Some(ArrayOfBoolExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        println!(
                            "variable_value({},array,({},{})).",
                            identifier(id),
                            pos,
                            bool_expr(e)
                        );
                    }
                }
                Some(ArrayOfBoolExpr::VarParIdentifier(id2)) => {
                    println!(
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    );
                }
                None => {}
            }
        }
        VarDeclItem::ArrayOfInt {
            id,
            ix,
            array_expr,
            annos,
        } => {
            println!(
                "variable_type({},{}).",
                identifier(id),
                array_type(&index(ix), "int"),
            );
            match array_expr {
                Some(ArrayOfIntExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        println!(
                            "variable_value({},array,({},{})).",
                            identifier(id),
                            pos,
                            int_expr(e)
                        );
                    }
                }
                Some(ArrayOfIntExpr::VarParIdentifier(id2)) => {
                    println!(
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    );
                }
                None => {}
            }
        }
        VarDeclItem::ArrayOfIntInRange {
            id,
            ix,
            lb,
            ub,
            array_expr,
            annos,
        } => {
            println!(
                "variable_type({},{}).",
                identifier(id),
                array_type(&index(ix), &int_in_range(lb, ub)),
            );
            match array_expr {
                Some(ArrayOfIntExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        println!(
                            "variable_value({},array,({},{})).",
                            identifier(id),
                            pos,
                            int_expr(e)
                        );
                    }
                }
                Some(ArrayOfIntExpr::VarParIdentifier(id2)) => {
                    println!(
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    );
                }
                None => {}
            }
        }
        VarDeclItem::ArrayOfIntInSet {
            id,
            ix,
            set,
            array_expr,
            annos,
        } => {
            for element in set {
                println!(
                    "variable_type({},{}).",
                    identifier(id),
                    array_type(&index(ix), &format!("int_in_set({})", element))
                );
            }
            match array_expr {
                Some(ArrayOfIntExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        println!(
                            "variable_value({},array,({},{})).",
                            identifier(id),
                            pos,
                            int_expr(e)
                        );
                    }
                }
                Some(ArrayOfIntExpr::VarParIdentifier(id2)) => {
                    println!(
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    );
                }
                None => {}
            }
        }
        VarDeclItem::ArrayOfFloat {
            id,
            ix,
            annos,
            array_expr,
        } => {
            println!(
                "variable_type({},{}).",
                identifier(id),
                array_type(&index(ix), "float"),
            );
            match array_expr {
                Some(ArrayOfFloatExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        println!(
                            "variable_value({},array,({},{})).",
                            identifier(id),
                            pos,
                            float_expr(e)
                        );
                    }
                }
                Some(ArrayOfFloatExpr::VarParIdentifier(id2)) => {
                    println!(
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    );
                }
                None => {}
            }
        }
        VarDeclItem::ArrayOfBoundedFloat {
            id,
            ix,
            lb,
            ub,
            array_expr,
            annos,
        } => {
            println!(
                "variable_type({},{}).",
                identifier(id),
                array_type(&index(ix), &bounded_float(*lb, *ub)),
            );
            match array_expr {
                Some(ArrayOfFloatExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        println!(
                            "variable_value({},array,({},{})).",
                            identifier(id),
                            pos,
                            float_expr(e)
                        );
                    }
                }
                Some(ArrayOfFloatExpr::VarParIdentifier(id2)) => {
                    println!(
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    );
                }
                None => {}
            }
        }
        VarDeclItem::ArrayOfSet {
            id,
            ix,
            array_expr,
            annos,
        } => {
            println!(
                "variable_type({},{}).",
                identifier(id),
                array_type(&index(ix), "set"),
            );
            match array_expr {
                Some(ArrayOfSetExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        let set = dec_set_expr(e);
                        for element in set {
                            println!(
                                "variable_value({},array,({},{})).",
                                identifier(id),
                                pos,
                                element
                            );
                        }
                    }
                }
                Some(ArrayOfSetExpr::VarParIdentifier(id2)) => {
                    println!(
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    );
                }
                None => {}
            }
        }
        VarDeclItem::ArrayOfSubSetOfIntRange {
            id,
            ix,
            lb,
            ub,
            array_expr,
            annos,
        } => {
            println!(
                "variable_type({},{}).",
                identifier(id),
                array_type(&index(ix), &subset_of_int_range(lb, ub))
            );
            match array_expr {
                Some(ArrayOfSetExpr::Array(v)) => {
                    for (pos, e) in v.iter().enumerate() {
                        let set = dec_set_expr(e);
                        for element in set {
                            println!(
                                "variable_value({}, array,({},{})).",
                                identifier(id),
                                pos,
                                element
                            );
                        }
                    }
                }
                Some(ArrayOfSetExpr::VarParIdentifier(id2)) => {
                    println!(
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    );
                }
                None => {}
            }
        }
        VarDeclItem::ArrayOfSubSetOfIntSet {
            id,
            ix,
            set,
            array_expr,
            annos,
        } => {
            for element in set {
                println!(
                    "variable_type({},{}).",
                    identifier(id),
                    array_type(&index(ix), &format!("subset_of_int_set({})", element)),
                );
            }
            match array_expr {
                Some(ArrayOfSetExpr::Array(v)) => {
                    for (pos, se) in v.iter().enumerate() {
                        for e in dec_set_expr(se) {
                            println!("variable_value({},array,({},{})).", identifier(id), pos, e);
                        }
                    }
                }
                Some(ArrayOfSetExpr::VarParIdentifier(id2)) => {
                    println!(
                        "variable_value({},value,{}).",
                        identifier(id),
                        identifier(id2)
                    );
                }
                None => {}
            }
        }
    }
}
fn basic_var_type(t: &BasicVarType) -> Vec<String> {
    match t {
        BasicVarType::BasicType(BasicType::Bool) => vec!["bool".to_string()],
        BasicVarType::BasicType(BasicType::Int) => vec!["int".to_string()],
        BasicVarType::IntInRange(lb, ub) => vec![int_in_range(lb, ub)],
        BasicVarType::IntInSet(set) => int_in_set(set),
        BasicVarType::BasicType(BasicType::Float) => vec!["float".to_string()],
        BasicVarType::BoundedFloat(lb, ub) => vec![bounded_float(*lb, *ub)],
        BasicVarType::SubSetOfIntRange(lb, ub) => vec![subset_of_int_range(lb, ub)],
        BasicVarType::SubSetOfIntSet(set) => subset_of_int_set(set),
    }
}
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
fn float_in_set(set: &[f64]) -> Vec<String> {
    let mut ret = vec![];
    for float in set {
        ret.push(format!("float_in_set({})", float))
    }
    ret
}
fn bounded_float(lb: f64, ub: f64) -> String {
    format!("bounded_float({},{})", float_literal(lb), float_literal(ub))
}
fn subset_of_int_range(lb: &i128, ub: &i128) -> String {
    format!(
        "subset_of_int_range({},{})",
        int_literal(lb),
        int_literal(ub)
    )
}
fn subset_of_int_set(set: &[i128]) -> Vec<String> {
    let mut ret = vec![];
    for i in set {
        ret.push(format!("subset_of_int_set({})", int_literal(i)))
    }
    ret
}
fn print_constraint(c: &ConstraintItem, i: usize) {
    println!("constraint(c{},{}).", i, identifier(&c.id));
    for (cpos, ce) in c.exprs.iter().enumerate() {
        match ce {
            Expr::VarParIdentifier(id) => {
                println!("constraint_type_at(c{},{},var_par).", i, cpos);
                println!(
                    "constraint_value_at(c{},{},value,{}).",
                    i,
                    cpos,
                    identifier(id)
                )
            }
            Expr::Bool(e) => {
                println!("constraint_type_at(c{},{},bool).", i, cpos);
                println!(
                    "constraint_value_at(c{},{},value,{}).",
                    i,
                    cpos,
                    bool_literal(*e)
                );
            }
            Expr::Int(e) => {
                println!("constraint_type_at(c{},{},int).", i, cpos);
                println!(
                    "constraint_value_at(c{},{},value,{}).",
                    i,
                    cpos,
                    int_literal(e)
                );
            }
            Expr::Float(e) => {
                println!("constraint_type_at(c{},{},float).", i, cpos);
                println!(
                    "constraint_value_at(c{},{},value,{}).",
                    i,
                    cpos,
                    float_literal(*e)
                );
            }
            Expr::Set(e) => {
                println!("constraint_type_at(c{},{},set).", i, cpos);
                let set = dec_set_literal_expr(e);
                for element in set {
                    println!("constraint_value_at(c{},{},{}).", i, cpos, element);
                }
            }
            Expr::ArrayOfBool(v) => {
                println!("constraint_type_at(c{},{},array).", i, cpos);
                for (apos, ae) in v.iter().enumerate() {
                    println!(
                        "constraint_value_at(c{},{},array,({},{})).",
                        i,
                        cpos,
                        apos,
                        bool_expr(&ae)
                    );
                }
            }
            Expr::ArrayOfInt(v) => {
                println!("constraint_type_at(c{},{},array).", i, cpos);
                for (apos, ae) in v.iter().enumerate() {
                    println!(
                        "constraint_value_at(c{},{},array,({},{})).",
                        i,
                        cpos,
                        apos,
                        int_expr(&ae)
                    );
                }
            }
            Expr::ArrayOfFloat(v) => {
                println!("constraint_type_at(c{},{},array).", i, cpos,);
                for (apos, ae) in v.iter().enumerate() {
                    println!(
                        "constraint_value_at(c{},{},array,({},{})).",
                        i,
                        cpos,
                        apos,
                        float_expr(&ae)
                    );
                }
            }
            Expr::ArrayOfSet(v) => {
                println!("constraint_type_at(c{},{},array_of_set).", i, cpos);
                for (apos, ae) in v.iter().enumerate() {
                    let set = dec_set_expr(ae);
                    for element in set {
                        println!(
                            "constraint_value_at(c{},{},array,({},{})).",
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
            let set = dec_set_expr(e);
            for element in set {
                println!("solve({},{}).", opt_type(ot), element);
            }
        }
    }
}
fn basic_par_type(t: &BasicParType) -> String {
    match t {
        BasicParType::BasicType(BasicType::Bool) => "bool".to_string(),
        BasicParType::BasicType(BasicType::Float) => "float".to_string(),
        BasicParType::BasicType(BasicType::Int) => "int".to_string(),
        BasicParType::SetOfInt => "set_of_int".to_string(),
    }
}
fn basic_pred_par_type(t: &BasicPredParType) -> Vec<String> {
    match t {
        BasicPredParType::BasicParType(t) => vec![basic_par_type(t)],
        BasicPredParType::BasicVarType(t) => basic_var_type(t),
        BasicPredParType::VarSetOfInt => vec!["set_of_int".to_string()],
        BasicPredParType::BoundedFloat(lb, ub) => vec![bounded_float(*lb, *ub)],
        BasicPredParType::IntInRange(lb, ub) => vec![int_in_range(lb, ub)],
        BasicPredParType::IntInSet(set) => int_in_set(set),
        BasicPredParType::FloatInSet(set) => float_in_set(set),
        BasicPredParType::SubSetOfIntRange(lb, ub) => vec![subset_of_int_range(lb, ub)],
        BasicPredParType::SubSetOfIntSet(set) => subset_of_int_set(set),
    }
}
fn array_type(idx: &str, element_type: &str) -> String {
    format!("array({},{})", idx, element_type)
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
fn dec_set_expr(e: &SetExpr) -> Vec<String> {
    match e {
        SetExpr::Set(sl) => dec_set_literal_expr(sl),
        SetExpr::VarParIdentifier(id) => vec![format!("value,{}", identifier(id))],
    }
}
fn dec_set_literal_expr(l: &SetLiteralExpr) -> Vec<String> {
    let mut ret = Vec::new();
    match l {
        SetLiteralExpr::BoundedFloat(f1, f2) => ret.push(format!(
            "float_bound,({},{})",
            float_expr(f1),
            float_expr(f2)
        )),
        SetLiteralExpr::IntInRange(i1, i2) => {
            ret.push(format!("int_range,({},{})", int_expr(i1), int_expr(i2)))
        }
        SetLiteralExpr::SetFloats(v) => {
            for f in v {
                ret.push(format!("value,{}", float_expr(f)));
            }
        }
        SetLiteralExpr::SetInts(v) => {
            for i in v {
                ret.push(format!("value,{}", int_expr(i)));
            }
        }
    }
    ret
}
fn dec_set_literal(l: &SetLiteral) -> Vec<String> {
    let mut ret = Vec::new();
    match l {
        SetLiteral::BoundedFloat(f1, f2) => ret.push(format!(
            "float_bound,({},{})",
            float_literal(*f1),
            float_literal(*f2)
        )),
        SetLiteral::IntRange(i1, i2) => ret.push(format!("int_range,({},{})", i1, i2)),
        SetLiteral::SetFloats(v) => {
            for f in v {
                ret.push(format!("value,{}", float_literal(*f)));
            }
        }
        SetLiteral::SetInts(v) => {
            for f in v {
                ret.push(format!("value,{}", f));
            }
        }
    }
    ret
}
