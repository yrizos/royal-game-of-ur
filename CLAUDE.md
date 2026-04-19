# Royal Game of Ur

Review `README.md` for the project overview, structure, rules, and available `just` commands. Refer to `SPEC.md` for the complete Finkel ruleset. This document defines additional guidance required for LLM interaction.

## Architecture Boundary

`ur-core` defines all game rules and is the authoritative source. `ur-cli` must not implement rule logic. It must not determine move legality, handle captures, or evaluate win conditions. All such decisions must be delegated to `ur-core`. Maintain strict separation: no game logic in `ur-cli`, no I/O in `ur-core`.

## Development Conventions

### Code Style

- Execute `just fmt` before committing.
- Execute `just check` and resolve all reported warnings.
- Provide doc comments (`///`) for all public types and functions within `ur-core`.
- Keep functions concise. If a function exceeds 40 lines, split it.

### Testing

- Follow test-driven development. Write tests before implementation.
- Ensure every edge case described in `SPEC.md` is covered by a test.
- Use descriptive test names, for example `test_capture_blocked_by_rosette`, not `test_move_3`.
- Run `just test` to confirm all tests pass.

### Commits

- Each commit must represent a single logical change.
- Use conventional commit prefixes: `feat:`, `fix:`, `test:`, `refactor:`, `docs:`.
- Do not commit code that fails `just test` or `just check`.

### Dependencies

- Keep `ur-core` dependencies minimal. `rand` is acceptable for dice logic. `serde` is acceptable when gated behind a feature flag.
- `ur-cli` depends on `ratatui`, `crossterm`, and `rand`.
- Introduce new dependencies only with clear justification.

## Future Plans (Not for Implementation)

- WebAssembly wrapper for browser usage
- C FFI interface for non-Rust integrations
- Additional rulesets such as Masters and Aseb
- Online multiplayer support
- Mobile clients

These items exist to justify the strict separation of I/O and game logic in `ur-core`.

## graphify — Required for Code Work

A complete knowledge graph of the codebase is available in `graphify-out/`. It captures modules, types, functions, specifications, design decisions, and their relationships.

### Usage Before Writing Code

1. When introducing a function or type, run `/graphify query "where does X fit"` to identify the correct module and existing related elements. Avoid duplication.
2. Before modifying a module, run `/graphify explain "ModuleName"` to understand dependencies, usage, and test coverage.
3. When deciding placement, follow graph communities as the true module boundaries.
4. For debugging, run `/graphify query "how does X connect to Y"` to trace relationships across modules.

### Required Reading

Review `graphify-out/GRAPH_REPORT.md` before:

- Planning refactors
- Investigating cross-module issues
- Determining usage relationships between components

### Maintaining Graph Accuracy

- Code changes are tracked automatically via the git post-commit hook.
- After modifying non-code assets such as documentation or specifications, run `/graphify . --update`.
- If `graphify-out/.needs_update` is present, update the graph before continuing.

### Git Hook Installation

If the hook is not installed:

```bash
graphify hook install
```
