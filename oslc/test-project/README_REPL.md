# OSL REPL Demo

## Quick Start

```bash
# Run the REPL
oslc repl

# Or run this demo file
oslc run repl_demo.osl
```

## REPL Commands

| Command | Description |
|---------|-------------|
| `:help` | Show help |
| `:quit` | Exit REPL |
| `:clear` | Clear screen |

## Example Sessions

```
>> 1 + 2
  = 3

>> let x = 10
>> x * 2
  = 20

>> "Hello " + "World!"
  = Hello World!

>> fn double(n: int) -> int { n * 2 }
>> double(21)
  = 42
```

## Supported Syntax

- Arithmetic: `+`, `-`, `*`, `/`, `%`, `^`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Logical: `&&`, `||`, `!`
- Types: `int`, `float`, `bool`, `str`, `list<T>`, `map<K,V>`
- Control flow: `if/else`, `loop`, `while`, `for/in`
- Functions: `fn name(params) -> ret_type { body }`
