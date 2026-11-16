# DSL Grammar (v0.1)

## Lexical rules
- Statements end at a newline. `#` starts a comment.
- Identifiers: `[A-Za-z_][A-Za-z0-9_]*`. Numbers: integers, negative allowed (`-5`).
- Reserved words (cannot be variable names):
  `let relation function on check closure truthtable classify equiv union intersect minus symdiff cross in subset and or not xor implies iff true false`
- Every operator has a symbol, word, and/or unicode form:

| Meaning | Forms |
|---|---|
| and | `&` `and` `∧` |
| or | `\|` `or` `∨` |
| not | `!` `not` `¬` |
| xor | `^` `xor` `⊕` |
| implies | `->` `implies` `→` |
| iff | `<->` `iff` `↔` |
| union / intersect / difference | `union` `∪` / `intersect` `∩` / `minus` `∖` |
| symmetric difference / Cartesian product | `symdiff` / `cross` `×` |
| element of / subset | `in` `∈` / `subset` `⊆` |
| equal / not equal | `==` / `!=` `≠` |

## Precedence (lowest to highest)
| Level | Operators | Assoc |
|---|---|---|
| 1 | `<->` | right |
| 2 | `->` | right |
| 3 | `\|` `^` | left |
| 4 | `&` | left |
| 5 | `in` `subset` `==` `!=` | none |
| 6 | `union` `minus` `symdiff` | left |
| 7 | `intersect` `cross` | left |
| 8 | prefix `!` | - |

## EBNF
```
program      = { [ statement ] NEWLINE } ;

statement    = let_stmt | relation_decl | function_decl
             | truthtable_cmd | classify_cmd | equiv_cmd
             | check_cmd | closure_cmd | expr ;

let_stmt       = "let" IDENT "=" expr ;
relation_decl  = "relation" IDENT "on" IDENT "=" "{" [ expr { "," expr } ] "}" ;
function_decl  = "function" IDENT ":" IDENT "->" IDENT "=" "{" [ mapping { "," mapping } ] "}" ;
mapping        = expr "->" expr ;

truthtable_cmd = "truthtable" expr ;
classify_cmd   = "classify" expr ;
equiv_cmd      = "equiv" expr "," expr ;
check_cmd      = "check" IDENT IDENT { "," IDENT } ;      (* check R reflexive, symmetric *)
closure_cmd    = "closure" IDENT IDENT ;                  (* closure R transitive *)

expr         = iff ;
iff          = implies [ "<->" iff ] ;
implies      = or_expr [ "->" implies ] ;
or_expr      = and_expr { ( "|" | "^" ) and_expr } ;
and_expr     = compare { "&" compare } ;
compare      = set_add [ ( "in" | "subset" | "==" | "!=" ) set_add ] ;
set_add      = set_mul { ( "union" | "minus" | "symdiff" ) set_mul } ;
set_mul      = unary { ( "intersect" | "cross" ) unary } ;
unary        = "!" unary | primary ;

primary      = NUMBER | "true" | "false"
             | IDENT [ "(" [ expr { "," expr } ] ")" ]    (* variable or call *)
             | set_literal
             | "(" expr ")"                               (* grouping *)
             | "(" expr "," expr { "," expr } ")" ;       (* tuple / ordered pair *)

set_literal  = "{" [ element { "," element } ] "}" ;
element      = expr [ ".." expr ] ;                       (* {1..5} range *)
```

## Notes
- Free variables in `truthtable` / `classify` / `equiv` (identifiers with no `let`) are treated as propositional variables.
- Property names (`reflexive`, `injective`, ...) are plain identifiers checked by the evaluator, not keywords.
- Domain/codomain in `function` are single identifiers so `->` isn't ambiguous with implication.
- Cardinality is `card(A)` (no `|A|`, since `|` is "or").
- Built-in calls planned: `powerset`, `card`, `compose`, `inverse`, `gcd`, `lcm`, `modpow`, `C`, `P`.

## Example
```
let A = {1, 2, 3, 4}
let B = {3, 4, 5}
A union B
A cross {a, b}
truthtable (p -> q) & !r
equiv (p -> q), (!p | q)
relation R on A = {(1,1), (2,2), (1,2)}
check R reflexive, transitive
```
