# Grammar

Formal additions over the SQF grammar. Everything else is unchanged.

## Productions

```text
TypedPrivate   ::= "private" LocalVar ":" Type ( "=" Expr )? ";"
TypedParamEnt  ::= StringLit "?"? ":" Type ( "=" Expr )?          // inside params array literal
TypeAlias      ::= "type" Ident "=" Type ";"
Interface      ::= "interface" Ident "{" ( Ident "?"? ":" Type ";" )* "}"
DeclareVar     ::= "declare" Ident ":" Type ";"
DeclareFn      ::= "declare" "function" Ident "(" ParamList? ")" ":" Type ";"
CastExpr       ::= Expr "as" Type                                  // lowest precedence
Type           ::= UnionType
UnionType      ::= ArrayType ( "|" ArrayType )*
ArrayType      ::= AtomType ( "[" "]" )*
AtomType       ::= PrimitiveName | Ident | TupleType | "(" Type ")"
TupleType      ::= "[" Type "?"? ( "," Type "?"? )* "]"
```

## Disambiguation (superset preservation)

| Token | Keyword when… | Plain SQF when… |
|---|---|---|
| `type`, `declare`, `interface` | Statement-initial and followed by the grammar above | Followed by `=` (global assignment) or any other SQF continuation |
| `as` | Expression-postfix and followed by a type expression | Anywhere else (e.g. a variable named `as`) |
| `:` | After a `private` local or a string in a `params` array literal | `case` labels, `switch` `:` operator |
| `?` | In typed param entries and interface members | Nowhere in SQF today (invalid character), so no conflict |

All new constructs are invalid syntax in plain SQF, so no existing program parses differently.

## Preprocessing

SQFts checks run **after** the HEMTT preprocessor, so macros may expand to annotated code in principle; spans map back through the expansion.

**v1 implementation note:** erasure runs on the *unpreprocessed* source so identity / locality hold. Type annotations must appear **literally** in `.sqfts` source — annotations produced only by macro expansion are out of scope until a later revision that can erase through the preprocessor source map.

## Related handbook pages

- [Annotating Variables](Annotating-Variables)
- [Typed Params](Typed-Params)
- [Type Aliases](Type-Aliases)
- [Interfaces](Interfaces)
- [Casts](Casts)
- [Declaring Functions and Globals](Declaring-Functions-and-Globals)
