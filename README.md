
# fzn2lp [![Build Status](https://github.com/potassco/fzn2lp/workflows/CI%20Test/badge.svg)](https://github.com/potassco/fzn2lp)

A FlatZinc to AnsProlog converter.

## Usage

    fzn2lp <FILE>

## Download

Binaries for 64bit linux and macOS can be found on the [release page](https://github.com/potassco/fzn2lp/releases/latest).

## Compile yourself

Clone the git repository:

    git clone https://github.com/potassco/fzn2lp.git
    cargo build --release

The executables can be found under `./target/release/`

## AnsProlog output

### Predicate declarations

Predicate declarations are presented by facts of form:

    predicate(PredicateName).
    predicate_parameter(PredicateName, Pos, ParameterName, ParameterType).

For example:

    predicate median_of_3(var int: x, var int: y, var int: z, var int: m);

is represented as:

    predicate("median_of_3").
    predicate_parameter("median_of_3", 0, "x", int).
    predicate_parameter("median_of_3", 1, "y", int).
    predicate_parameter("median_of_3", 2, "z", int).
    predicate_parameter("median_of_3", 3, "m", int).

### Parameter declarations

Basic parameters are declared by facts of form:

    parameter(ParameterName, ParameterType, ParameterExpression).

For example:

    bool: X_21 = true;

is represented as:

    parameter("X_21", bool, true).

For other parameter types see  [*Parameter/variable types*](#parametervariable-types).

Parameters of type array are declared by facts of form

    parameter(ParameterName, array(Index,BasicParameterType) ).

For example:

    array [1..2] of int: X_22 = [1,-1,5];

is represented as:

    parameter("X_22", array(2,int)).

For the representation of the array see [*Representation of arrays*](#representation-of-arrays)

### Variable declarations

Variable declarations are presented by facts of form:

    variable_type(VariableName, Type).
    variable_value(VariableName, Value).

For example:

    var bool: X_35;

is represented as:

    variable_type("X_35", bool).

For other parameter types see [*Parameter/variable types*](#parametervariable-types).

### Representation of arrays

Parameters or variables of type *array* like:

    array [1..2] of int: X = [1,-1,5];
    array [1..2] of var int: Y  = [Y_0,Y_1,Y_2];

are represented by facts

    parameter("X", array(2,int),array_at(0,1)).
    parameter("X", array(2,int),array_at(1,-1)).
    parameter("X", array(2,int),array_at(3,5)).
    variable_type("Y", array(2,int)).
    variable_value("Y", array_at(0,"Y_0")).
    variable_value("Y", array_at(1,"Y_1")).
    variable_value("Y", array_at(2,"Y_2")).

### Constraints

Constraints are presented by facts of form:

    constraint(ConstraintId, ConstraintName).
    in_constraint(ConstraintId, Pos, Expr).

If the constraint parameter is of type array the following predicate is used to represent the elements of the array.

    in_constraint(ConstraintId, Pos, array).
    in_constraint(ConstraintId, Pos, ArrayPos, Expr).

For example:

    constraint array_bool_or([X_35,X_36],true);

is represented as:

    constraint(c1, "array_bool_or").
    in_constraint(c1, 0, array).
    in_constraint(c1, 0, 0, "X_35").
    in_constraint(c1, 0, 1, "X_36").
    in_constraint(c1, 1, true).

### Solve statement

The solve statement is represented by one fact of the following form:

    solve(satisfy).
    solve(maximize, Expr).
    solve(minimize, Expr).

For example:

    solve minimize X_24;

is represented as:

    solve(minimize, "X_24").

### Parameter/variable types

Basic types are:

- `bool`
- `int`
- `float`
- `int_in_range(lb,ub)` where `lb` and `ub` are integers,
- `bounded_float(lb,ub)` where `lb` and `ub` are quoted float literals,
- `set_of_int_in_range(lb,ub)`  where `lb` and `ub` are integers,
<!-- - `set_of_int_in_set(set_id)` where set_id is the id of a set -->

Further types are:

- `array(i,t)`  where `i` is an integer denoting the length of the array or `int` and `t` is a basic type.
