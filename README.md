[![Build Status](https://github.com/sthiele/fzn2lp/workflows/CI%20Test/badge.svg)](https://github.com/sthiele/fzn2lp)

# fzn2lp

A FlatZinc to AnsProlog converter.

## Usage

    fzn2lp --input <file>

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

    predicate(id_median_of_3).
    predicate_parameter(id_median_of_3, 1, int, x)
    predicate_parameter(id_median_of_3, 1, int, y)
    predicate_parameter(id_median_of_3, 1, int, z)
    predicate_parameter(id_median_of_3, 1, int, m)

### Parameter declarations

Basic parameters are declared by facts of form:

    parameter(ParameterType,ParameterName, ParameterExpression).

For Example:

    bool: X_21 = true;

is represented as:

    parameter(bool,id_X_21, true).

Other parameter types are `int` and `float`.

Parameters of type array are declared by facts of form

    parameter(array(Index,BasicParmeterType),ParameterName).

For Example:

    array [1..2] of int: X_22 = [1,-1,5];

is represented as:

    parameter(array(2,int),id_X_22, ParameterExpression).

For the representation of the array see *Representation of arrays*

### Variable declarations

Variable declarations are presented by facts of form:

    variable(VariableType,VariableName).

For Example:

    var bool: X_35;

is represented as:

    variable(bool,id_X_35).

Other variable types are `int` and `float` ...

### Representation of arrays

Parameters or variables of type *array* like:

    array [1..2] of int: X = [1,-1,5];
    array [1..2] of var int: Y  = [Y_0_,Y_1_,Y_2_];

are declared by facts

    parameter(array(2,int), id_X).
    variable(array(2,int),id_Y).

The array itself represented by using the predicate `in_array/3`:

    in_array(Id, Pos, Element)

where

- `Id`: is the name of the array
- `Pos`: is the position in the array
- `Element`: is the element at position `Pos`

For example:

    in_array(id_X, 0, 1).
    in_array(id_X, 1, -1).
    in_array(id_X, 2, 5).
    in_array(id_Y, 0, id_Y_0).
    in_array(id_Y, 1, id_Y_1).
    in_array(id_Y, 2, id_Y_2).

### Representation of constraints

Constraints are presented by facts of form:

    constraint(ConstraintId).
    in_constraint(ConstraintId,Pos,Expr).

If the constraint parmeter is of type array the following predicate is used to represent the elements of the array.

    in_constraint(ConstraintId,Pos,array).
    in_constraint(ConstraintId,Pos,ArrayPos,Expr).

For Example:

    constraint array_bool_or([X_35,X__36],true);

is represented as:

    constraint(id_array_bool_or)
    in_constraint(id_array_bool_or,0,array).
    in_constraint(id_array_bool_or,0,0,id_X_35).
    in_constraint(id_array_bool_or,0,1,id_X_36).
    in_constraint(id_array_bool_or,1,true).

### Solve statement

The solve statement is represented by one fact of the following form:

    solve(satisfy).
    solve(maximize,Expr).
    solve(minimize,Expr).

For Example:

    solve minimize X_24;

is represented as:

    solve(minimize,id_X_24).
