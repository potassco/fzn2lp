# Format proposal

- **PRE: parameters and variables share their namespace**

Parameters of any kind:

```asp
variable_value("a",value,1).                   % single integer
variable_value("b",value,"1.0").               % single float
variable_value("c",value,true).                % single bool
variable_value("d",array,(0,value,42)).        % array of int
variable_value("d",array,(1,value,23)).
variable_value("e",array,(0,value,"42.0")).    % array of float
variable_value("e",array,(1,value,"23.0")).
variable_value("f",set,(value,17)).            % set of int
variable_value("f",set,(range,(23,42))).       % !! INVALID FZN 
variable_value("g",set,(value,"23.0"))).       % set of float
variable_value("g",set,(value,"42.0"))).
variable_value("h",array,(0,set,(value,42))).  % array of set of int
variable_value("h",array,(0,set,(value,17))).
variable_value("h",array,(1,set,(range,(1,5)))).
variable_value("h",array,(1,set,(value,42))).
variable_value("h",array,(2,set,emptyset)).
```

Variable **assignments** of any kind.

```asp
% FZN
var int : a = 1;
% ASP
variable_type("a",int).
variable_value("a",value,1).

% FZN
var float : b = 1.0;
% ASP
variable_type("b",float).
variable_value("b",value,"1").

% FZN
var bool : c = true;
% ASP
variable_type("c",bool).
variable_value("c",value,true).

% FZN
array [1..2] of var int : d = [42,23];
% ASP
variable_type("d",array(2,int)).
variable_value("d",array,(0,value,42)).
variable_value("d",array,(1,value,23)).

% FZN
array [1..2] of var float : e = [42.1,23.1];
% ASP
variable_type("e",array(2,float)).
variable_value("e",array,(0,value,"42.1")).
variable_value("e",array,(1,value,"23.1")).

% FZN
var set of 17..42: f = {17,23};
% ASP
variable_type("f",subset_of_int_range(17,42)).
variable_value("f",set,(value,17)).
variable_value("f",set,(value,23)).

% TODO: Check if/how set of floats are allowed
% FZN
% var set of float: g = {23.1,42.1};
% ASP
% variable_value("g",set,(value,"23.1")).
% variable_value("g",set,(value,"42.1")).

% FZN
array [1..2] of var set of 17..42: h = [{42,17},23..X];  % TODO: check empty set
% ASP
variable_type("h",array(2,subset_of_int_range(17,42))).
variable_value("h",array,(0,set,(value,42))).
variable_value("h",array,(0,set,(value,17))).
variable_value("h",array,(1,range,(value,23,var,X))).

% TODO: check empty set
% ASP
variable_value("h",array,(2,set,emptyset)).
```

Variables:

```asp
variable_type("a", int).
variable_type("b", float).
variable_type("c", bool).
variable_type("d", int, range,(23,42)).
variable_type("e", int, set,(value,23)).
variable_type("e", int, set,(value,42)).
variable_type("e", int, set,(range,(100,200))).
variable_type("f", float, range,(23,42)).
variable_type("g", array(20,int, range,(17,23))).               % 20 elements of int in the range 17..23
variable_type("h", array(20,int)).                              % 20 elements of int
variable_type("i", array(20,int, set,(value,23))).              % 20 elements of int
variable_type("i", array(20,int, set,(value,25))).
variable_type("i", array(20,int, set,(range,(42,56)))).
variable_type("j", array(20,set_of_int, set,(value,23))).       % 20 elements of set of ints
variable_type("j", array(20,set_of_int, set,(value,2))).
variable_type("j", array(20,set_of_int, set,(range,(43,56)))).
variable_type("k", array(20,set_of_int)).                       % 20 elements of set of ints
```

Constraints:

```asp
constraint(c1,"constraint_name").
% constraint_value(ID, parameter position, value)
constraint_value(c1,0,value,42).                                % parameter 0 is integer 42
constraint_value(c1,1,value,"42.0").                            % parameter 1 is float 42.0
constraint_value(c1,2,value, true).                             % parameter 2 is bool true
constraint_value(c1,4,var,"a").                                 % parameter 4 is variable "a", what ever this is
constraint_value(c1,5,array,(0,value,42)).                      % parameter 5 is an array of integers and integer variables
constraint_value(c1,5,array,(1,value,17)).
constraint_value(c1,5,array,(2,var,"X")).
constraint_value(c1,6,set,(var,"X")).                           % parameter 6 is a set of var int
constraint_value(c1,6,set,(value,34)).
constraint_value(c1,6,set,(range,(37,48))).
constraint_value(c1,7,array,(0,set,(value,42))).                % parameter 7 is an array of sets of integers
constraint_value(c1,7,array,(0,set,(value,17))).
constraint_value(c1,7,array,(1,set,(range,(17,34)))).
constraint_value(c1,7,array,(2,set,(var,"X"))).
constraint_value(c1,7,array,(2,set,(var,"Y"))).
```
