# Keyword Renaming / Aliasing Design (Gulf of Mexico Language)

## Goal

Support runtime introduction of keyword aliases (e.g. `alias fun = function!`) without destabilizing parser or requiring full re-tokenization of previously loaded code. Provide a reversible, introspectable mechanism that:

- Allows users to add alternative spellings for existing keyword semantics.
- Preserves original keyword forms internally (canonical tokens) to avoid proliferation.
- Applies only to future parses (not retroactively rewriting already tokenized code).
- Avoids ambiguity and infinite alias chains.

## Scope (Phase 1)

- Allow creation of simple one-level aliases mapping a new name -> canonical keyword.
- Restrict alias targets to existing reserved keywords in the core set: `function|func|fun|fn|if|when|after|class|return|delete|export|import|reverse|var|const`.
- Provide builtin function: `alias(original: string, newName: string)` returning boolean success.
- Maintain alias registry in `Interpreter`.
- Lexer consults registry: if token is `Name` and matches an alias key, it substitutes its value with the canonical keyword value prior to parser dispatch.
- Prevent overriding existing variables accidentally: alias names share namespace with keywords; if a variable of same name exists, alias cannot be added.

## Data Structures

```rust
struct KeywordAliases {
  map: HashMap<String, String> // new_name -> canonical_keyword
}
```

Added to `Interpreter` as `keyword_aliases: KeywordAliases`.

## Canonicalization Contract

1. Canonical validation: `new_name` must be composed of ASCII letters only, length 1..32.
2. Must not collide with any existing canonical keyword nor existing alias keys.
3. Must not collide with a currently defined variable/function/class name in global namespace (to preserve user code clarity).
4. No chaining: `map.get(new_name)` yields canonical; canonical must not itself be an alias key.

## Lexer Integration

During tokenization (in `processor/lexer.rs`), after forming a `Name` token:

```rust
if keyword_aliases.contains(token.value) {
   token.value = keyword_aliases.resolve(token.value); // canonical keyword string
}
```

Parser remains unchanged; it already matches against canonical forms.

## Builtin Function API

`alias(original, newName)` semantics:

- `original`: canonical keyword being aliased.
- `newName`: proposed alias.

Return values:

- `true` if alias inserted.
- `false` if validation fails.

Errors printed (not thrown) for failure causes.

## Reversibility

Optional Phase 1 addition: `unalias(name)` removing mapping. Fails silently if not present. Returns boolean.

## Introspection

Builtin `list_aliases()` returns a list/map serialization of current alias mappings.

## Edge Cases

- Attempt to alias to an existing alias name: reject.
- Attempt to alias a non-keyword name: reject.
- Use of alias in the same statement that defines it: NOT supported (alias only active for subsequent parses; design keeps tokenization single-pass).
- Import/export: Exported code uses canonical keywords only (no alias leakage), ensuring portability.

## Future Enhancements (Deferred)

- Persistent alias storage across sessions (serialize to `~/.dreamberd_runtime/aliases.json`).
- Scoped aliases (file-level or block-level) with push/pop semantics.
- Alias precedence rules when colliding with user variable names (opt-in override).
- IDE integration for hover showing canonical target.

## Implementation Steps

1. Add `KeywordAliases` struct and field to `Interpreter`.
2. Add builtin functions (`alias`, `unalias`, `list_aliases`).
3. Thread `&KeywordAliases` reference into tokenizer entry points.
4. Modify tokenizer to canonicalize using alias map.
5. Add tests:
   - Simple alias creation and subsequent use (`alias("function", "make")` then `make foo() {}` works).
   - Prevent variable name overshadow: declare `const foo = 1!` then `alias("function", "foo")` fails.
   - Export preserves canonical keywords.

## Risks & Mitigations

- Performance: HashMap lookup on every name token. Mitigation: canonical keyword set small; alias map expected tiny (<50). Negligible.
- Ambiguity: Disallow chains and collisions up front; parser remains deterministic.
- User confusion: Provide `list_aliases()` for clarity; canonical keywords always accepted.

## Minimal MVP Acceptance Criteria

- Aliasing works for at least `function` and `if`.
- Failure cases produce human-readable messages.
- Aliases usable in new code after their creation.

---

End of design document.
