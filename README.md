# How to use? 
Simple calc is an infix notation fixed-point calculator (8 decimal places). 
The install.py installation script copies the calculator to /usr/bin or ~/.local/bin.

Features: arithmetic and bitwise operators, comparisons (with Russian word
aliases like БОЛЬШЕ/МЕНЬШЕ/РАВНО), real variables (`set X 5`, `X = {X} * 2`),
lazy conditional branching `IF[cond]:(then):(else)`, user-defined `.math` and
`.rhai` functions, a REPL and one-shot CLI mode.
See the full [Feature Documentation](./doc/features.md).
 
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
* `>` `<` `=` `==` `!=` `≠` `<=` `>=` — Comparisons (return 1 or 0; Russian aliases: БОЛЬШЕ, МЕНЬШЕ, РАВНО, НЕ_РАВНО, ...) 
* `IF[cond]:(then):(else)` — Lazy conditional branching 
* `set NAME VALUE`, `NAME = VALUE`, `{NAME}` or bare `NAME` — Variables 
* `FUNCNAME[args]` - Call the function 

Example: 
```bash
calc "2+2*2" #6 
calc "(2+2)*2" #8 
```

# How does it work? 
This math engine is based on a recursive stack. 

Each parenthesis on the main math stack opens a new stack, which is evaluated, and the result of the child stack's calculations is delivered to the call site. 
The parser simply assembles the math st
ack from strings.

[Function Doc](./doc/function.md)
[Feature Documentation](./doc/features.md)
