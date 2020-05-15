
# fzn2lp [![Build Status](https://github.com/sthiele/fzn2lp/workflows/CI%20Test/badge.svg)](https://github.com/sthiele/fzn2lp)

A FlatZinc to AnsProlog converter.

## Usage

    fzn2lp <FILE>

## Download

Binaries for 64bit linux and macOS can be found on the [release page](https://github.com/sthiele/fzn2lp/releases/latest).

## Compile yourself

Clone the git repository:

    git clone https://github.com/sthiele/fzn2lp.git
    cargo build --release

The executables can be found under `./target/release/`

## AnsProlog output

### Predicate declarations

Predicate declarations are presented by facts of form:

    predicate(PredicateName).
    predicate_parameter(PredicateName, Pos, ParameterType, ParameterName).

For example:

    predicate median_of_3(var int: x, var int: y, var int: z, var int: m);

is represented as:

    predicate("median_of_3").
    predicate_parameter("median_of_3", 0, int, "x").
    predicate_parameter("median_of_3", 1, int, "y").
    predicate_parameter("median_of_3", 2, int, "z").
    predicate_parameter("median_of_3", 3, int, "m").

### Parameter declarations

Basic parameters are declared by facts of form:

    parameter(ParameterType, ParameterName, ParameterExpression).

For Example:

    bool: X_21 = true;

is represented as:

    parameter(bool, "X_21", true).

Other parameter types are `int` and `float`.

Parameters of type array are declared by facts of form

    parameter(array(Index,BasicParameterType), ParameterName).

For Example:

    array [1..2] of int: X_22 = [1,-1,5];

is represented as:

    parameter(array(2,int), "X_22").

For the representation of the array see *Representation of arrays*

### Variable declarations

Variable declarations are presented by facts of form:

    variable(VariableType, VariableName).

For Example:

    var bool: X_35;

is represented as:

    variable(bool, "X_35").

Other variable types are `int` and `float` ...

### Representation of arrays

Parameters or variables of type *array* like:

    array [1..2] of int: X = [1,-1,5];
    array [1..2] of var int: Y  = [Y_0,Y_1,Y_2];

are represented by facts

    parameter(array(2,int), "X").
    variable(array(2,int), "Y").

The array itself represented by using the predicate `in_array/3`:

    in_array(Id, Pos, Element)

where

- `Id`: is the name of the array
- `Pos`: is the position in the array
- `Element`: is the element at position `Pos`

For example:

    in_array("X", 0, 1).
    in_array("X", 1, -1).
    in_array("X", 2, 5).
    in_array("Y", 0, "Y_0").
    in_array("Y", 1, "Y_1").
    in_array("Y", 2, "Y_2").

### Constraints

Constraints are presented by facts of form:

    constraint(ConstraintId, ConstraintName).
    in_constraint(ConstraintId, ConstraintName, Pos, Expr).

If the constraint parameter is of type array the following predicate is used to represent the elements of the array.

    in_constraint(ConstraintId, Pos, array).
    in_constraint(ConstraintId, Pos, ArrayPos, Expr).

For Example:

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

For Example:

    solve minimize X_24;

is represented as:

    solve(minimize, "X_24").
