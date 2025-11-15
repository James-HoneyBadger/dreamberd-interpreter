# GOM - Gulf of Mexico Interpreter

This implementation is based on the conceptual design of **Gulf of Mexico**, the perfect programming language created by Lu Wilson (TodePond) (https://github.com/TodePond/GulfOfMexico).

## Installation

```bash
git clone https://github.com/James-HoneyBadger/GOM.git
cd GOM
pip install -e .
```

## Usage

```bash
# Run a program
python -m gulfofmexico script.gom

# Interactive REPL
python -m gulfofmexico

# Execute inline code
python -m gulfofmexico -c "print(42)!"

# Launch IDE
python -m gulfofmexico.ide
```

## Core Language Features

### Arrays Start at -1
```
const nums [10, 20, 30]!
print(nums[-1])!  // 10
print(nums[0])!   // 20
```

### Fractional Indexing
```
const list [1, 3]!
list[0.5] = 2!
// Result: [1, 2, 3]
```

### Probabilistic Variables
```
var x 10!    // Confidence 1
var x 20!!   // Confidence 2 (wins)
var x 30!!!  // Confidence 3 (highest)
```

### Variable Lifetimes
```
const temp <5.0> = 99!     // Expires in 5 seconds
const brief 100 = 42!      // Expires after 100 lines
```

### Three-Valued Logic
```
const yes true!
const no false!
const maybe maybe!
```

### Reactive Programming
```
var count 0!

when count > 5 {
   print("Count exceeded 5!")!
}

count = 10!  // Triggers when statement
```

### Async Functions
```
async function fetch() => {
   return 42!
}!

const result await(fetch())!
```

### String Interpolation
```
const name "Alice"!
print("Hello, ${name}!")!
```

## Syntax

### Variables
```
const x 10!          // Immutable
var y 20!            // Mutable  
const var z [1,2]!   // Mutable content, immutable reference
```

### Functions
```
function add(a, b) => a + b!

fn multiply(x, y) => x * y!  // Short form
```

### Classes
```
class Person {
   var name "Unknown"!
   var age 0!
   
   function greet() => {
      print("Hi, I'm ${name}")!
   }!
}!

const p new Person()!
p.name = "Bob"!
```

### Control Flow
```
if condition {
   // code
}

when condition {
   // reactive
}

after <2.0> {
   // delayed
}
```

## Operators

**Arithmetic:** `+` `-` `*` `/` `^`

**Comparison:** `<` `>` `<=` `>=`

**Equality:**
- `=` Approximate
- `==` Standard
- `===` Strict
- `====` Strictest

**Logical:** `&` `|` `;` (not)

## Built-in Functions

- `print()` - Output
- `Number()` `String()` `Boolean()` - Type conversion
- `read()` `write()` - File I/O
- `Map()` - Create dictionary
- `sleep()` `exit()` - Control
- `regex_match()` `regex_findall()` `regex_replace()` - Regex
- Math functions: `sin` `cos` `sqrt` `log` etc.
- Word numbers: `zero` through `ninteen`, `twenty()` `thirty()` etc.

## Debug Output

```
const x 10?      // Level 1
const y 20??     // Level 2
const z 30???    // Level 3
const w 40????   // Level 4
```

## Multi-File Programs

```
===== utils =====
function helper() => 42!
export helper to main!

===== main =====
import helper!
print(helper())!
```

## IDE

```bash
python -m gulfofmexico.ide       # Qt GUI with web fallback
python -m gulfofmexico.ide --web # Force web interface
```

Web IDE: `http://localhost:8080/ide`

## Requirements

- Python 3.10+
- requests (required)
- pynput (optional)
- pygithub (optional)
- PySide6/PyQt5 (optional, for Qt IDE)

## Author

James Temple  
Email: james@honey-badger.org  
GitHub: [James-HoneyBadger](https://github.com/James-HoneyBadger)

## License

This is free and unencumbered software released into the public domain.
