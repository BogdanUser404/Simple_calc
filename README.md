# How to use? 
Simple calc is an infix notation calculator. 
The install.py installation script copies the calculator to /usr/bin or ~/.local/bin. 
 
Operators: 
* `+` — Addition 
* `-` — Subtraction 
* `*` — Multiplication 
* `/` — Division 
* `**` — Exponentiation 
* `!` — Factorial 
* `!sub` — Subfactorial 
* `¬` — Bitwise NOT 
* `&` — Bitwise AND 
* `|` — Bitwise OR 
* `¬|` — Bitwise NOR 
* `¬&` — Bitwise NAND 
* `xor` — Bitwise XOR 
* `<<` — Shift bits left 
* `>>` — Shift bits right 
* `gcd` — Greatest Common Divisor (GCD)
* `lcm` — Least Common Multiple (LCM)
* `rand` — Random number 

Example: 
```bash
calc "2+2*2" #6 
calc "(2+2)*2" #8 
```

# How does it work? 
This math engine is based on a recursive stack. 

Each parenthesis on the main math stack opens a new stack, which is evaluated, and the result of the child stack's calculations is delivered to the call site. 

The parser simply assembles the math stack from strings.
