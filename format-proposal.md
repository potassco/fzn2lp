# Format proposal

Predicates:

```asp
% FZN
predicate my_pred(int:a, {1,2,3}:a2, 1..11:a3, float:b, bool:c,
               set of int: d, set of {1,2,3}: e, set of 1..11: f,
               array [int] of int:g, array [int] of {1,2,3}:h, array [int] of 1..11:i,
               array [int] of set of int:j);
% ASP
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

Parameters:

```asp
% FZN
int : a = 1;
% ASP
parameter_value("a",value,1).

% FZN
float : b = 1.1;
% ASP
parameter_value("b",value,"1.1").

% FZN
bool : c = true;
% ASP
parameter_value("c",value,true).

% FZN
array [1..2] of int : d = [42,23];
% ASP
parameter_value("d",array,(0,value,42)).
parameter_value("d",array,(1,value,23)).

% FZN
array [1..2] of float : e = [42.1,23.0];
parameter_value("e",array,(0,value,"42.1")).
parameter_value("e",array,(1,value,"23")).

% FZN
set of int: f = 23..42;
% ASP
parameter_value("f",range,(value,23,value,42)).

% FZN
set of float : g = {42.1,23.0};
% ASP
parameter_value("g",set,(value,"23"))).
parameter_value("g",set,(value,"42.1"))).

% FZN
array [1..3] of set of int : h = [{42,17},1..5,{}];
% ASP
parameter_value("h",array,(0,set,(value,42))).
parameter_value("h",array,(0,set,(value,17))).
parameter_value("h",array,(1,range,(value,1,value,5))).
parameter_value("h",array,(2,empty_set)).
```

Variables:

```asp
% FZN
var int : a = 1;
% ASP
variable_type("a",int).
variable_value("a",value,1).

% FZN
var 1..3 : a;
% ASP
variable_type("a",int,range,(value,1,value,3)).

% FZN
var {1,2,3} : a;
% ASP
variable_type("a",int,set,(value,1)).
variable_type("a",int,set,(value,2)).
variable_type("a",int,set,(value,3)).

% FZN
var float : b = 1.0;
% ASP
variable_type("b",float).
variable_value("b",value,"1").

% FZN
var 0.5..1.5: b = 1.0;
% ASP
variable_type("b",float,(bounds,value,"0.5",value,"1.5")).
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
variable_type("f",set_of_int,range,(value,17,value,42)).
variable_value("f",set,(value,17)).
variable_value("f",set,(value,23)).

% FZN
var set of {17,23,100}: f = {17,23};
% ASP
variable_type("f",set_of_int,set,(value,17)).
variable_type("f",set_of_int,set,(value,23)).
variable_type("f",set_of_int,set,(value,100)).
variable_value("f",set,(value,17)).
variable_value("f",set,(value,23)).

%FZN
array [1..3] of var set of 17..42: h = [{42,17},23..X,{}];
% ASP
variable_type("h",array(3,set_of_int,range,(value,17,value,42))).
variable_value("h",array,(0,set,(value,42))).
variable_value("h",array,(0,set,(value,17))).
variable_value("h",array,(1,range,(value,23,var,"X"))).
variable_value("h",array,(2,empty_set)).

```

Constraints:

```asp
% FZN
constraint bla(42,42.1,true,a,[42,17,X],{X,34},37..48,[{42,17},17..34,{X,Y}]);
% ASP
constraint(c1,"bla").
constraint_value_at(c1,0,value,42).
constraint_value_at(c1,1,value,"42.1").
constraint_value_at(c1,2,value,true).
constraint_value_at(c1,3,var,"a").
constraint_value_at(c1,4,array,(0,value,42)).
constraint_value_at(c1,4,array,(1,value,17)).
constraint_value_at(c1,4,array,(2,var,"X")).
constraint_value_at(c1,5,set,(var,"X")).
constraint_value_at(c1,5,set,(value,34)).
constraint_value_at(c1,6,range,(value,37,value,48)).
constraint_value_at(c1,7,array,(0,set,(value,42))).
constraint_value_at(c1,7,array,(0,set,(value,17))).
constraint_value_at(c1,7,array,(1,range,(value,17,value,34))).
constraint_value_at(c1,7,array,(2,set,(var,"X"))).
constraint_value_at(c1,7,array,(2,set,(var,"Y"))).
```
