# Functions 

The functions are located in $HOME/.local/share/functions 
There are two types of functions: 
Rhai scripts 
Mathematical expressions 

## Macthematical expressions:
Files in scalc/functions with the .math extension. 
Structure: 
`ARGS: NUMBER;` - Number of arguments in a function. 
Other - Interpreted as an expression. 
Arguments are passed via ARG_X. 
X - Argument number starting from 1. 

## Rhai scripts 
Files on scalc/functions with the .rhai extension. 
A specific functions: 
to_fixed(i64) -> i128 - Return i128 number.  
to_raw(i128) -> i64 - Get the integer part of a number. 
to_float(i128) -> f64 - Get float type of number. 

### Opperationss: 
not() 
nand() 
nor() 
gcd() 
lcm() 
fact() 
subfact() 
rand() 

### Variable: 
args - Array of argument.  

### Types:
i128 | FixedPoint - I128 in Rhai. 

