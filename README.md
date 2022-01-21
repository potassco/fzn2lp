
# fzn2lp [![Build Status](https://github.com/potassco/fzn2lp/workflows/CI%20Test/badge.svg)](https://github.com/potassco/fzn2lp)

A converter from FlatZinc into an ASP fact format.

## Usage

```text
fzn2lp <FILE>
```

## Download

Binaries for 64bit linux and macOS can be found on the [release page](https://github.com/potassco/fzn2lp/releases/latest).

## Compile yourself

Clone the git repository and build:

```bash
git clone https://github.com/potassco/fzn2lp.git
cd fzn2lp
cargo build --release
```

The executables can then be found under `./target/release/`

## Flatzinc to ASP translation

### Predicate declarations

Predicate declarations are represented by facts of form:

```prolog
predicate(PredicateName).
predicate_parameter(PredicateName, Pos, ParameterName, ParameterType).
```

In predicates the `ParameterType` can be either `bool`, `int`, `float`, `set_of_int`, `array(L,int)`,`array(L,set_of_int)`.
The types `int` and `set_of_int` can have specializations, either `range,(value,Int,value,Int)` or `set,(value,Int)`.

For example:

```flatzinc
predicate my_pred(int:a, {1,2,3}:a2, 1..11:a3, float:b, bool:c,
               set of int: d, set of {1,2,3}: e, set of 1..11: f,
               array [int] of int:g, array [int] of {1,2,3}:h, array [int] of 1..11:i,
               array [int] of set of int:j);
```

is represented as:

```prolog
predicate("my_pred").
predicate_parameter("my_pred",0,"a",int).
predicate_parameter("my_pred",1,"a2",int,set,(value,1)).
predicate_parameter("my_pred",1,"a2",int,set,(value,2)).
predicate_parameter("my_pred",1,"a2",int,set,(value,3)).
predicate_parameter("my_pred",2,"a3",int,range,(value,1,value,11)).
predicate_parameter("my_pred",3,"b",float).
predicate_parameter("my_pred",4,"c",bool).
predicate_parameter("my_pred",5,"d",set_of_int).
predicate_parameter("my_pred",6,"e",set_of_int,set,(value,1)).
predicate_parameter("my_pred",6,"e",set_of_int,set,(value,2)).
predicate_parameter("my_pred",6,"e",set_of_int,set,(value,3)).
predicate_parameter("my_pred",7,"f",set_of_int,range,(value,1,value,11)).
predicate_parameter("my_pred",8,"g",array(int,int)).
predicate_parameter("my_pred",9,"h",array(int,int,set,(value,1))).
predicate_parameter("my_pred",9,"h",array(int,int,set,(value,2))).
predicate_parameter("my_pred",9,"h",array(int,int,set,(value,3))).
predicate_parameter("my_pred",10,"i",array(int,int,range,(value,1,value,11))).
predicate_parameter("my_pred",11,"j",array(int,set_of_int)).
```

### Parameters

Basic parameters are declared by facts of form:

```prolog
parameter_value(ParameterName, ParameterType, ParameterValue).
```

Here the `ParameterType` can be either a basic `value`, or  a complex `set`, `range`, `array`.

Value of basic parameter type (`value`) are integers, floats, `true` and `false`.
Values of type `range` have the form `(value,Int,value,Int)`, values of type `array` have the form `(Pos,Type,Int)` and values of type `set` have the form `(value,BasicValue)`.

For example the parameters:

```flatzinc
int : a = 1;
float : b = 1.1;
bool : c = true;
array [1..2] of int : d = [42,23];
array [1..2] of float : e = [42.1,23.0];
set of int: f = 23..42;
array [1..3] of set of int : h = [{42,17},1..5,{}];
```

are represented as:

```prolog
parameter_value("a",value,1).
parameter_value("b",value,"1.1").
parameter_value("c",value,true).
parameter_value("d",array,(0,value,42)).
parameter_value("d",array,(1,value,23)).
parameter_value("e",array,(0,value,"42.1")).
parameter_value("e",array,(1,value,"23")).
parameter_value("f",range,(value,23,value,42)).
parameter_value("h",array,(0,set,(value,42))).
parameter_value("h",array,(0,set,(value,17))).
parameter_value("h",array,(1,range,(value,1,value,5))).
parameter_value("h",array,(2,empty_set)).
```

### Variable declarations

Variable declarations are presented by facts of form:

```prolog
variable_type(VariableName, VariableType).
variable_value(VariableName, Type, Value).
```

Variable types can be either `bool`, `int`, `float`, `set_of_int`, `array(L,int)`,`array(L,set_of_int)`.
The types `int` and `set_of_int` can have specializations, either `range,(value,Int,value,Int)` or `set,(value,Int)`.

For example the variable declarations:

```flatzinc
var int : a1 = 1;
var 1..3 : a2;
var {1,2,3} : a3;
var float : b1 = 1.0;
var 0.5..1.5: b2 = 1.0;
var bool : c = true;
array [1..2] of var int : d = [42,23];
array [1..2] of var float : e = [42.1,23.1];
var set of 17..42: f = {17,23};
var set of {17,23,100}: g = {17,23};
array [1..3] of var set of 17..42: h = [{42,17},23..X,{}];
```

are represented as:

```prolog
variable_type("a1",int).
variable_value("a1",value,1).

variable_type("a2",int,range,(value,1,value,3)).

variable_type("a3",int,set,(value,1)).
variable_type("a3",int,set,(value,2)).
variable_type("a3",int,set,(value,3)).

variable_type("b1",float).
variable_value("b1",value,"1").

variable_type("b2",float,(bounds,value,"0.5",value,"1.5")).
variable_value("b2",value,"1").

variable_type("c",bool).
variable_value("c",value,true).

variable_type("d",array(2,int)).
variable_value("d",array,(0,value,42)).
variable_value("d",array,(1,value,23)).

variable_type("e",array(2,float)).
variable_value("e",array,(0,value,"42.1")).
variable_value("e",array,(1,value,"23.1")).

variable_type("f",set_of_int,range,(value,17,value,42)).
variable_value("f",set,(value,17)).
variable_value("f",set,(value,23)).

variable_type("g",set_of_int,set,(value,17)).
variable_type("g",set_of_int,set,(value,23)).
variable_type("g",set_of_int,set,(value,100)).
variable_value("g",set,(value,17)).
variable_value("g",set,(value,23)).

variable_type("h",array(3,set_of_int,range,(value,17,value,42))).
variable_value("h",array,(0,set,(value,42))).
variable_value("h",array,(0,set,(value,17))).
variable_value("h",array,(1,range,(value,23,var,"X"))).
variable_value("h",array,(2,empty_set)).
```

### Constraints

Constraints are presented by facts of form:

```prolog
constraint(ConstraintId, ConstraintName).
constraint_value(ConstraintId, Pos, Type, Expr).
```

The expressions in constraints can contain variables `var` or values `value`. Complex expressions are `array`, `set` and `range`.

For example the constraint:

```flatzinc
constraint my_constraint(42, 42.1, true, a, [42,17,X], {X,34}, 37..48, [{42,17},17..34,{X,Y}]);
```

is represented as:

```prolog
constraint(c1,"my_constraint").
constraint_value(c1,0,value,42).
constraint_value(c1,1,value,"42.1").
constraint_value(c1,2,value,true).
constraint_value(c1,3,var,"a").
constraint_value(c1,4,array,(0,value,42)).
constraint_value(c1,4,array,(1,value,17)).
constraint_value(c1,4,array,(2,var,"X")).
constraint_value(c1,5,set,(var,"X")).
constraint_value(c1,5,set,(value,34)).
constraint_value(c1,6,range,(value,37,value,48)).
constraint_value(c1,7,array,(0,set,(value,42))).
constraint_value(c1,7,array,(0,set,(value,17))).
constraint_value(c1,7,array,(1,range,(value,17,value,34))).
constraint_value(c1,7,array,(2,set,(var,"X"))).
constraint_value(c1,7,array,(2,set,(var,"Y"))).
```

### Solve statement

The solve statement is represented by one fact of the following form:

```prolog
solve(satisfy).
solve(maximize, Type, Expr).
solve(minimize, Type, Expr).
```

For example:

```flatzinc
solve minimize X_24;
```

is represented as:

```prolog
solve(minimize,var,"X_24").
```
