# Gulf of Mexico Language Reference

Complete reference for the Gulf of Mexico programming language.

## Table of Contents

1. [Syntax Basics](#syntax-basics)
2. [Variables](#variables)
3. [Data Types](#data-types)
4. [Arrays](#arrays)
5. [Functions](#functions)
6. [Classes](#classes)
7. [Control Flow](#control-flow)
8. [Operators](#operators)
9. [Statements](#statements)
10. [Modules](#modules)

---

## Syntax Basics

### Statement Terminators

**All statements must end with `!`:**

```gom
print("Hello")!
const var x = 5!
```

**Debug statements end with `?`:**

```gom
print("Debug info")?
const var test = 10?
```

**Multiple exclamation marks indicate confidence:**

```gom
const var low_confidence = 1!
const var medium_confidence = 2!!
const var high_confidence = 3!!!
```

Later declarations with more `!` marks override earlier ones.

### Comments

```gom
// Single-line comments

/* Multi-line
   comments */
```

### Indentation

**Must use multiples of 3 spaces:**

```gom
function example() => {
   const var x = 5!           // 3 spaces
   if (x > 0) {
      print(x)!               // 6 spaces
   }!
}!
```

**Tabs count as 2 spaces** (not recommended, makes correct indentation difficult).

---

## Variables

### Declaration Types

Four combinations of mutability:

| Declaration | Reassign | Modify | Example |
|------------|----------|--------|---------|
| `const const` | ❌ | ❌ | `const const PI = 3.14!` |
| `const var` | ❌ | ✅ | `const var arr = [1,2,3]!` |
| `var const` | ✅ | ❌ | `var const x = 5!` |
| `var var` | ✅ | ✅ | `var var count = 0!` |

**Examples:**

```gom
// Cannot reassign or modify
const const name = "Alice"!
// name = "Bob"!  // Error
// name[0] = "X"!  // Error

// Can modify, cannot reassign
const var scores = [1, 2, 3]!
scores[0] = 10!  // OK
// scores = [4, 5, 6]!  // Error

// Can reassign, cannot modify
var const x = 5!
x = 10!  // OK
// x = x + 1!  // Error (modifying)

// Can reassign and modify
var var total = 0!
total = 10!  // OK
total = total + 5!  // OK
```

### Variable Naming

- Alphanumeric and underscore
- Unicode characters allowed
- Cannot start with a digit

```gom
const var valid_name = 1!
const var 👍 = true!
const var naïve = "yes"!
```

### Type Annotations

Optional and decorative (don't affect behavior):

```gom
const var name: String = "Alice"!
var var age: Number = 25!
```

---

## Data Types

### Numbers

```gom
const var integer = 42!
const var float = 3.14!
const var negative = -10!
const var exponent = 2^10!  // 1024
```

**Division by zero returns `undefined`:**

```gom
const var result = 10 / 0!
print(result)!  // undefined
```

### Strings

**Multiple quote styles:**

```gom
const var single = 'hello'!
const var double = "world"!
const var triple = '''multi
line'''!
```

**Zero quotes (bare words):**

```gom
const var word = hello!  // Equivalent to "hello"
```

**String interpolation:**

```gom
const var name = "Alice"!
print("Hello ${name}")!    // $-style
print("Hello £{name}")!    // £-style  
print("Hello €{name}")!    // €-style
print("Hello ¥{name}")!    // ¥-style
```

### Booleans

Three values:

```gom
const const is_true = true!
const const is_false = false!
const const is_maybe = maybe!
```

### Undefined

Returned by division by zero and missing values:

```gom
const var x = 10 / 0!  // undefined
const var arr = [1, 2, 3]!
const var y = arr[10]!  // undefined
```

### Arrays

See [Arrays](#arrays) section.

### Objects

Created via classes:

```gom
class Person {
   const var name = "Unknown"!
   const var age = 0!
}!

const var p = new Person()!
```

---

## Arrays

### Indexing Starts at -1

```gom
const const fruits = ["apple", "banana", "cherry"]!

print(fruits[-1])!  // "apple" (first element)
print(fruits[0])!   // "banana" (second element)  
print(fruits[1])!   // "cherry" (third element)
```

### Float Indexing

Insert at fractional positions:

```gom
var var numbers = [1, 2, 3]!
numbers[0.5] = 99!
print(numbers)!  // [1, 99, 2, 3]

numbers[-0.5] = 50!
print(numbers)!  // [50, 1, 99, 2, 3]
```

### Array Operations

```gom
const var arr = [1, 2, 3]!

// Modify elements
arr[0] = 10!

// Access length  
const var len = length(arr)!

// Concatenation
const var combined = arr + [4, 5]!
```

---

## Functions

### Declaration

Multiple keywords (all equivalent):

```gom
function add(a, b) => {
   return a + b!
}!

func multiply(x, y) => x * y!

fun divide(a, b) => {
   return a / b!
}!

fn subtract(x, y) => x - y!
```

### Single-Line Functions

```gom
func double(x) => x * 2!
fn square(x) => x ^ 2!
```

### Multi-Line Functions

```gom
function calculate(x, y) => {
   const var sum = x + y!
   const var product = x * y!
   return [sum, product]!
}!
```

### Return Statement

```gom
function get_value() => {
   return 42!
}!
```

### Calling Functions

```gom
const var result = add(3, 5)!
print(result)!  // 8
```

---

## Classes

### Declaration

```gom
class Player {
   const var health = 100!
   const var score = 0!
   const var name = "Player"!
}!
```

Alternative keyword:

```gom
className Enemy {
   const var damage = 10!
}!
```

### Instantiation

```gom
const var player = new Player()!
const var enemy = new Enemy()!
```

### Constructor Pattern

```gom
class PlayerMaker {
   function make(name) => {
      class Player {
         const var player_name = name!
      }!
      return new Player()!
   }!
}!

const var maker = new PlayerMaker()!
const var alice = maker.make("Alice")!
```

---

## Control Flow

### If Statements

```gom
const var x = 10!

if (x > 5) {
   print("Greater than 5")!
}!
```

### If-Else

```gom
if (x > 10) {
   print("Large")!
} else {
   print("Small")!
}!
```

### Nested Conditions

```gom
if (x > 0) {
   if (x < 10) {
      print("Between 0 and 10")!
   }!
}!
```

---

## Operators

### Arithmetic

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `5 + 3` → `8` |
| `-` | Subtraction | `5 - 3` → `2` |
| `*` | Multiplication | `5 * 3` → `15` |
| `/` | Division | `6 / 2` → `3` |
| `^` | Exponentiation | `2 ^ 3` → `8` |

### Comparison

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal | `5 == 5` → `true` |
| `!=` | Not equal | `5 != 3` → `true` |
| `>` | Greater than | `5 > 3` → `true` |
| `<` | Less than | `3 < 5` → `true` |
| `>=` | Greater or equal | `5 >= 5` → `true` |
| `<=` | Less or equal | `3 <= 5` → `true` |

### Logical

| Operator | Description | Example |
|----------|-------------|---------|
| `&` | AND | `true & false` → `false` |
| `\|` | OR | `true \| false` → `true` |
| `;` | NOT | `;false` → `true` |

---

## Statements

### Print

```gom
print("Hello")!
print("Value:", 42)!
print(x, y, z)!
```

### Delete

Remove values from memory:

```gom
delete 3!
delete "hello"!
delete true!
```

### Reverse

Undo previous assignments:

```gom
var var x = 5!
x = 10!
x = 20!
reverse!  // x is now 10
reverse!  // x is now 5
```

### Export

```gom
export const const PI = 3.14159!
export const var settings = {}!
export func add(a, b) => a + b!
```

### Import

```gom
import PI from "constants.gom"!
import add from "math.gom"!
```

---

## Modules

### Exporting

**File: `math.gom`**

```gom
export func add(a, b) => a + b!
export func multiply(a, b) => a * b!
export const const PI = 3.14159!
```

### Importing

**File: `main.gom`**

```gom
import add from "math.gom"!
import PI from "math.gom"!

const var result = add(3, 5)!
print("Result:", result)!
print("PI:", PI)!
```

---

## Temporal Keywords

Access variable history:

```gom
var var x = 10!
x = 20!
x = 30!

print(current(x))!   // 30
print(previous(x))!  // 20
```

---

## Built-in Functions

### Input/Output

- `print(...values)` - Print values to console
- `input(prompt)` - Read user input (if available)

### Type Checking

- `type(value)` - Get type of value

### Array Functions

- `length(array)` - Get array length
- `reverse(array)` - Reverse array order

### Math Functions

- Standard arithmetic operators
- Exponentiation via `^`

---

## Error Handling

Errors are reported with file location and context:

```
InterpretationError: web_ide, line 3

      return a + b!
  ^^^^
Invalid indenting detected (must be a multiple of 3).
```

---

## Best Practices

1. **Use consistent indentation** - Always use 3 spaces
2. **Choose appropriate mutability** - Use `const const` by default
3. **Remember -1 indexing** - Arrays start at -1, not 0
4. **End all statements with `!`** - Required by syntax
5. **Use meaningful names** - Even in an esoteric language

---

## Common Pitfalls

### Wrong Indentation

```gom
// WRONG - 4 spaces
function bad() => {
    return 5!
}!

// CORRECT - 3 spaces
function good() => {
   return 5!
}!
```

### Array Indexing

```gom
const var arr = [1, 2, 3]!

// WRONG
print(arr[0])!  // This is the SECOND element

// CORRECT for first element
print(arr[-1])!  // This is the FIRST element
```

### Missing Terminators

```gom
// WRONG
print("Hello")

// CORRECT
print("Hello")!
```

---

*For more examples, see the `examples/` directory in the project repository.*
