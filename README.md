# fzn2lp

## Representation of arrays

Parameters or variables of type *array* like:

    array [1..2] of int: X = [1,-1,5];
    array [1..2] of var int: Y  = [Y_0_,Y_1_,Y_2_];

are declared by facts

    parameter(array(2,int), id_X, array).
    variable(array(2,int),id_Y).

The array itself represented by using the predicate `in_array/3`:

    in_array(Id, Pos, Element)

where

- Id: the name of the array
- Pos: the position in the array
- Element: the element at position Pos

For example:

    in_array(id_X, 0, 1).
    in_array(id_X, 1, -1).
    in_array(id_X, 2, 5).
    in_array(id_Y, 0, id_Y_0).
    in_array(id_Y, 1, id_Y_1).
    in_array(id_Y, 2, id_Y_2).
