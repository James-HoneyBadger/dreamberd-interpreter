# Gulf of Mexico Quick Reference

Essential syntax and features at a glance.

## Statement Terminators

```gom
print("Hello")!           // Required exclamation mark
print("Debug")?           // Debug mode with question mark
const var x = 100!!       // High confidence (more ! = higher priority)
```

## Variables

```gom
const const x = 5!        // Cannot reassign or modify
const var arr = [1,2,3]!  // Can modify, cannot reassign
var const y = 10!         // Can reassign, cannot modify
var var z = 0!            // Can reassign and modify
```

## Arrays (Start at -1!)

```gom
const const items = ["a", "b", "c"]!

items[-1]    // "a" (first element)
items[0]     // "b" (second element)
items[1]     // "c" (third element)

// Float indexing
var var nums = [1, 2, 3]!
nums[0.5] = 99!  // [1, 99, 2, 3]
```

## Functions

```gom
function add(a, b) => {
   return a + b!
}!

func multiply(x, y) => x * y!
fun divide(a, b) => a / b!
fn subtract(x, y) => x - y!
```

## Strings

```gom
const var s1 = 'single'!
const var s2 = "double"!
const var s3 = hello!              // Zero quotes (bare word)
const var s4 = "Hello ${name}"!    // Interpolation
```

## Booleans

```gom
true!
false!
maybe!    // Third boolean value
```

## Classes

```gom
class Player {
   const var health = 100!
}!

const var p = new Player()!
```

## Control Flow

```gom
if (x > 5) {
   print("Greater")!
}!

if (x > 10) {
   print("Large")!
} else {
   print("Small")!
}!
```

## Operators

```gom
// Arithmetic
+ - * / ^

// Comparison
== != > < >= <=

// Logical
& | ;         // AND, OR, NOT

// Division by zero
10 / 0        // Returns undefined
```

## Import/Export

```gom
// Export (file: utils.gom)
export const const PI = 3.14!
export func add(a, b) => a + b!

// Import (file: main.gom)
import PI from "utils.gom"!
import add from "utils.gom"!
```

## Special Statements

```gom
// Reverse (undo assignments)
var var x = 5!
x = 10!
reverse!          // x is now 5

// Delete
delete 3!
delete "hello"!

// Temporal
var var y = 10!
y = 20!
y = 30!
current(y)!       // 30
previous(y)!      // 20
```

## Indentation Rules

**Must use multiples of 3 spaces:**

```gom
function example() => {
   const var x = 5!           // 3 spaces
   if (x > 0) {
      print("Positive")!      // 6 spaces
   }!
}!
```

## Running Programs

```bash
# Run a file
gulfofmexico program.gom

# Interactive REPL
gulfofmexico

# Web IDE
python -m gulfofmexico.ide --web
```

## Common Mistakes

❌ **Wrong indentation:**
```gom
function bad() => {
    return 5!    // 4 spaces - ERROR!
}!
```

✅ **Correct indentation:**
```gom
function good() => {
   return 5!     // 3 spaces - OK
}!
```

❌ **Wrong array index:**
```gom
const var arr = [1, 2, 3]!
arr[0]   // This is the SECOND element!
```

✅ **Correct array index:**
```gom
const var arr = [1, 2, 3]!
arr[-1]  // This is the FIRST element
```

❌ **Missing terminator:**
```gom
print("Hello")   // ERROR - missing !
```

✅ **With terminator:**
```gom
print("Hello")!  // OK
```

## Examples Directory

- `hello.gom` - Hello World
- `calculator.gom` - Basic calculator
- `array_testing.gom` - Array operations
- `mandelbrot.gom` - Mandelbrot set
- `sierpinski.gom` - Sierpinski triangle

## Help

- Full documentation: `LANGUAGE_REFERENCE.md`
- Project structure: `README.md`
- Examples: `examples/` directory
