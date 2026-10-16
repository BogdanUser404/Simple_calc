# Feature Documentation

This document describes all features of **Simple Calc** (`scalc`), an infix-notation
fixed-point calculator written in Rust.

## Table of Contents
1. [Overview and Architecture](#1-overview-and-architecture)
2. [Fixed-Point Arithmetic](#2-fixed-point-arithmetic)
3. [Arithmetic Operators](#3-arithmetic-operators)
4. [Bitwise Operators](#4-bitwise-operators)
5. [Comparison Operators](#5-comparison-operators)
6. [Russian Operator Aliases](#6-russian-operator-aliases)
7. [Variables](#7-variables)
8. [Conditional Branching (IF Constructs)](#8-conditional-branching-if-constructs)
9. [User-Defined Functions](#9-user-defined-functions)
10. [REPL Mode](#10-repl-mode)
11. [Command-Line Mode](#11-command-line-mode)
12. [Random Numbers](#12-random-numbers)
13. [Error Handling](#13-error-handling)
14. [Installation](#14-installation)

---

## 1. Overview and Architecture

Simple Calc evaluates mathematical expressions written in **infix notation**
(e.g. `2 + 2 * 2`). The engine is built on a **recursive stack**:

* **Lexer** (`src/parser.rs`) — splits the input into tokens: numbers,
  operator words, comparison words, single/double-character operators.
  Identifiers may contain letters (including Cyrillic), digits and underscores.
* **ParsingEngine** (`src/parser.rs`) — converts the token stream into
  operations on the math stacks, resolving operator words (and their Russian
  aliases) into canonical internal tokens and looking up variables.
* **MathProcessor / MathStack** (`src/math_proccessor.rs`) — each parenthesis
  `(` opens a child stack; when the matching `)` is reached, the child stack is
  evaluated (comparisons first, then arithmetic by precedence) and its single
  result is pushed back onto the parent stack. Evaluation precedence inside a
  stack is:
  1. Comparisons (`>`, `<`, `=`, `!=`, `<=`, `>=`) → fixed-point booleans `1`/`0`
  2. `rand` token generators
  3. Unary operators (`not`, `¬`, `!`, `!sub`), right-to-left
  4. Exponentiation (`**`)
  5. Multiplicative level (`*`, `/`, `<<`, `>>`, `gcd`, `lcm`)
  6. Additive/bitwise level (`+`, `-`, `&`, `|`, `xor`, `¬|`, `¬&`)
* **Branching** (`src/branching.rs`) — parsing and lazy evaluation of
  `IF[...]:(...):(...)` conditional constructs.
* **Functions** (`src/main.rs`) — user-defined `.math` and `.rhai` functions
  loaded from disk.

All numeric values are represented internally as `i128` fixed-point integers.

## 2. Fixed-Point Arithmetic

* Every number is stored scaled by **10^8** (`SCALE = 100_000_000`), giving
  exactly **8 decimal places** of precision with no floating-point rounding
  errors for representable values.
* Literals accept an optional sign and fractional part: `5`, `-3.25`, `+0.5`,
  `.75`. Fractional digits beyond 8 are truncated.
* Output formatting strips trailing zeros: `2.50000000` prints as `2.5`,
  `4.00000000` prints as `4`.
* Overflow-checked arithmetic: multiplication, division, exponentiation and
  shifts report `Overflow` errors instead of wrapping.
* Bitwise operations and comparisons operate on the **integer part** of the
  fixed-point value (the raw value divided by the scale).

## 3. Arithmetic Operators

| Operator | Meaning | Example | Result |
|---|---|---|---|
| `+` | Addition | `1.5 + 2.25` | `3.75` |
| `-` | Subtraction (also unary minus when attached to a literal, e.g. `-5`) | `10 - 4` | `6` |
| `*` | Multiplication | `2.5 * 4` | `10` |
| `/` | Division (division by zero is an error) | `7 / 2` | `3.5` |
| `**` | Exponentiation (integer exponents, negative exponents supported) | `2 ** 10` | `1024` |
| `!` | Factorial (postfix, non-negative integers only) | `5!` | `120` |
| `!sub` | Subfactorial (derangement count, postfix) | `4!sub` | `9` |
| `( )` | Grouping; each group is evaluated on its own child stack | `(2+2)*2` | `8` |

Notes:
* Unary minus attaches to a directly-following number without whitespace
  (`-5`, `2.5-(-1)`); a standalone `-` between spaced operands remains binary
  subtraction.
* Factorial/subfactorial of a negative number produces an error.

## 4. Bitwise Operators

Bitwise operators work on the integer parts of their operands and return
integer results (scaled back to fixed-point).

| Operator | Meaning | Word alias |
|---|---|---|
| `¬x` or `not x` | Bitwise NOT | `не` via `not` |
| `a & b` | Bitwise AND | `и` |
| `a \| b` | Bitwise OR | `или` |
| `a ¬& b` or `nand(a,b)` | Bitwise NAND | `не_и` |
| `a ¬\| b` or `nor(a,b)` | Bitwise NOR | `не_или` |
| `a xor b` | Bitwise XOR | `исключительное_или` |
| `a << b` | Shift bits left | — |
| `a >> b` | Shift bits right | — |

Word-form operators (`not`, `xor`, `gcd`, `lcm`, `rand`) are reserved names and
can never be shadowed by variables.

## 5. Comparison Operators

Comparisons produce **fixed-point booleans**: `1` for true, `0` for false, so
they compose naturally with arithmetic (`(1 > 2) + 1` = `1`).

| Symbol | Meaning | Russian word | English aliases |
|---|---|---|---|
| `>` | Greater than | `БОЛЬШЕ` | `gt`, `greater` |
| `<` | Less than | `МЕНЬШЕ` | `lt`, `less` |
| `=` / `==` | Equal to | `РАВНО` | `eq`, `equal` |
| `!=` / `≠` | Not equal | `НЕ_РАВНО` | `ne_ravno`, `neravno` |
| `<=` | Not greater (≤) | `НЕ_БОЛЬШЕ` | `nb` |
| `>=` | Not less (≥) | `НЕ_МЕНЬШЕ` | `nm` |

Semantics:
* Comparisons are exact integer comparisons on the shared fixed-point scale
  (no epsilon needed).
* **Chained comparisons** behave like Rust: `a < b < c` means
  `(a < b) AND (b < c)` — the middle operand is reused and the boolean results
  are combined with a spliced bitwise AND.
* Single `<`/`>` are comparisons; double `<<`/`>>` are bit shifts; a triple
  (`<<<`, `>>>`) is rejected with a helpful error message.
* `!=` is disambiguated from the postfix factorial `!` by the lexer.

## 6. Russian Operator Aliases

The tokenizer recognizes case-insensitive Cyrillic operator words, e.g.
`БОЛЬШЕ`, `Больше`, `больше` all mean `>`. Reserved comparison words starting
with `б`, `м`, `р`, `н` are matched greedily before generic identifiers, so
`5 больше 3` tokenizes as `[5][больше][3]` and never as a variable name.
Available aliases: `БОЛЬШЕ`, `МЕНЬШЕ`, `РАВНО`, `НЕ_РАВНО`, `НЕ_БОЛЬШЕ`,
`НЕ_МЕНЬШЕ`, `И`, `ИЛИ`, `ИСКЛЮЧИТЕЛЬНОЕ_ИЛИ`, `НЕ_И`, `НЕ_ИЛИ` (see the table
in section 5 for the mapping). Cyrillic letters are also valid in variable
names (e.g. `ИКС`).

## 7. Variables

Real variables can be used in two interchangeable syntaxes:

* **Bare identifiers**: `x + 1`, `my_var2 * 3`, `ИКС - 1`
* **Legacy placeholders**: `{x} + 1` (names inside braces may have surrounding
  spaces, which are trimmed)

Behavior:
* A bare identifier is substituted only if it is not a reserved operator word
  or the `IF`/`if[...]` keyword; everything else passes through verbatim so
  branch constructs and function-call brackets are never damaged.
* Variable values are substituted as plain fixed-point literals before any
  other processing, so IF constructs, function arguments and nested calls all
  see resolved numbers.
* Referencing an undefined variable produces an error listing all currently
  defined variables (or a hint to use `set NAME VALUE` / `NAME=VALUE`).

Definition sources:
* REPL commands: `set NAME VALUE`, `NAME = VALUE`, `let NAME = VALUE`
* CLI arguments after the expression: `calc "x * 2" x=21`

Assignment RHS evaluation:
* Plain literals (`X = 5`, `X = -3.25`) are parsed directly.
* Anything else goes through the full engine, so expressions, comparisons and
  self-referential re-assignment work: `X = X * 2 + 1` uses the **old** value
  of `X` (dynamic re-assignment).
* The assignment splitter ignores `=` inside brackets and never mistakes `==`,
  `<=`, `>=`, `!=` for assignments.

Management commands (REPL): `unset NAME`, `list`, `clear`.

## 8. Conditional Branching (IF Constructs)

Syntax:

```
IF[CONDITION]:(THEN_EXPRESSION):(ELSE_EXPRESSION)
```

* The condition is **true** when it evaluates to a value that is **not zero**
  (any non-zero fixed-point value, including fractions like `0.5`).
* The construct evaluates to a numeric literal that participates in the
  surrounding expression normally: `IF[1]:(10):(20)` → `10`.
* **Lazy evaluation**: only the condition and the *taken* arm are ever parsed
  and evaluated. Errors inside the untaken arm (e.g. `1/0`) never surface:
  `IF[1-1]:(1/0):(7*7)` → `49`.
* **Nesting** is fully supported (up to 64 levels): inner constructs are
  resolved recursively, innermost-first, and bracket matching skips complete
  nested `IF[...]:(...):(...)` spans atomically. Function-call brackets such
  as `gcd[a,b]` inside any part are handled correctly.
* Both the uppercase `IF[...]` and lowercase `if[...]` spellings are accepted;
  these keywords are protected from variable substitution.
* Parse errors: unterminated condition (`]` missing), missing `:` separators,
  empty condition/THEN/ELSE parts.

## 9. User-Defined Functions

Custom functions live in `$HOME/.local/share/scalc/functions` and are invoked
with the syntax `FUNCNAME[arg1,arg2,...]` (comma-separated arguments; nested
parentheses inside arguments are respected). Arguments are arbitrary
sub-expressions evaluated by the engine before the call.

### 9.1 Mathematical expression functions (`.math`)

A `.math` file declares its arity and body:

```
ARGS: 2;
ARG_1 + ARG_2 * 2
```

* `ARGS: NUMBER;` — required declaration of the argument count.
* All remaining lines are joined into one expression.
* Arguments are exposed as `ARG_1`, `ARG_2`, ... (numbering starts at 1).
* The body sees both the caller's variables and the `ARG_X` bindings; calling
  with the wrong number of arguments is an error.

### 9.2 Rhai script functions (`.rhai`)

A `.rhai` file is executed by the embedded [Rhai](https://rhai.rs) scripting
engine (v1.26). The script must evaluate to an `i128` fixed-point value.

* **Variable**: `args` — array of arguments (each an `i128` fixed-point number).
* **Type**: `i128` is registered as `FixedPoint`.
* **Conversions**:
  * `to_fixed(i64) -> i128` / `to_fixed(f64) -> i128` — scale into fixed-point
  * `to_raw(i128) -> i64` — extract the integer part
  * `to_float(i128) -> f64` — convert to a floating-point number
* **Registered operators** (all fixed-point aware): `+`, `-`, `*`, `/`, `**`,
  `&`, `|`, `xor`, `<<`, `>>`
* **Registered functions**: `not()`, `nand()`, `nor()`, `gcd()`, `lcm()`,
  `fact()`, `subfact()`, `rand()`

Resolution order: if both `name.math` and `name.rhai` exist, the `.math`
version wins. A missing function produces `Function 'name' not found`.

## 10. REPL Mode

Started by running `scalc` with no arguments. Features:

* Prompt `calc>`, reads one line per iteration.
* `exit` / `quit` — leave the REPL.
* `set NAME VALUE` — define a variable (VALUE may be any expression).
* `NAME = VALUE` / `let NAME = VALUE` — alternative definition forms.
* `unset NAME` — remove a variable.
* `list` — print all defined variables (sorted).
* `clear` — remove all variables.
* Any other line is evaluated as an expression (with variables, IF constructs
  and function calls fully available).
* Assignments echo the new value: `X = 5` prints `X = 5`.

## 11. Command-Line Mode

```bash
calc "2+2*2"          # 6
calc "(2+2)*2"        # 8
calc "x * 2" x=21     # 42  (variable passed as NAME=VALUE argument)
```

* The first argument is the expression; subsequent `NAME=VALUE` arguments
  pre-define variables for it (VALUE itself may be an expression referencing
  already-defined variables).
* Arguments that don't parse as assignments are appended to the expression.
* Errors go to stderr; results go to stdout.

## 12. Random Numbers

* `rand` is a nullary token-generator usable anywhere a number is expected:
  `rand + 1`, `rand * 100`.
* Each occurrence generates an independent random value in `[0, 1)`
  (uniform over the fixed-point scale, using the `rand` crate).
* Inside `.rhai` scripts the same generator is available as `rand()`.

## 13. Error Handling

All failures are reported as human-readable messages (stderr in CLI mode,
inline in the REPL) rather than panics during normal evaluation:

* **Lexical**: unknown token characters, invalid `<<<`/`>>>` sequences,
  malformed numbers.
* **Parse**: unmatched brackets, missing operands, unknown variables (with a
  list of defined ones).
* **Evaluation**: division by zero, overflow, factorial/subfactorial of
  negative numbers, shift overflow, empty stack.
* **Branching**: malformed IF constructs, too-deep nesting (>64).
* **Functions**: missing files, bad `ARGS` declarations, wrong argument counts,
  Rhai execution errors.

## 14. Installation

`install.py` builds the project in release mode with Cargo, ensures the
functions directory `~/.local/share/scalc/functions` exists, and installs the
binary as `scalc`:

* to `/usr/bin/scalc` (via `sudo cp`) if you answer `y` to the prompt;
* otherwise to `~/.local/bin/scalc` (make sure it is in your `PATH`).

```bash
python3 install.py
```

Requires a Rust toolchain (edition 2024) and dependencies `rhai 1.26` and
`rand 0.8` (see `Cargo.toml`). Note: the script refuses to run on Windows
(joke aside, use Linux).

---

See also: [Function authoring guide](./function.md)
