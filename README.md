[![Build Status](https://github.com/sthiele/fzn2lp/workflows/CI%20Test/badge.svg)](https://github.com/sthiele/fzn2lp)

# fzn2lp

## Predicate declarations

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
    predicate_parameter(id_median_of_3, 1, int, m )

## Parameter declarations

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

## Variable declarations

Variable declarations are presented by facts of form:

    variable(VariableType,Variablename).

For Example:

    var bool: X_35;

is represented as:

    variable(bool,id_X_35).


Other variable types are `int` and `float` ...

## Representation of arrays

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
