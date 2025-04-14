#  Rust Documentation Guide

This guide outlines comprehensive rules, templates, and best practices for writing high-quality documentation in Rust using `rustdoc`. It is intended for advanced users, teams, and tooling.

---

## Purpose

To standardize and optimize the use of Rust documentation comments across crates and modules. Designed for public API design, libraries, and developer tooling.

---

## Core Principles

1. **Documentation is part of the API contract.**
2. **Documentation must be precise, complete, and testable.**
3. **Code examples should compile and run.**
4. **Clarity over cleverness. Write for the reader.**

---

## Types of Rustdoc Comments

| Style | Usage | Scope |
|-------|-------|-------|
| `///` | Item-level docs | Functions, structs, enums, traits |
| `//!` | Inner docs | Module-level or crate-level overviews |

---

## Advanced Documentation Template

```rust
/// <Concise one-line summary>
///
/// <Expanded description detailing behavior, side effects, use cases, or rationale.>
///
/// # Arguments
///
/// * `arg_name` - Description, format, and usage
/// * `options` - Optional argument explanation, if applicable
///
/// # Returns
///
/// * Description of the return type and its significance
///
/// # Panics
///
/// * Conditions under which this function will panic
///
/// # Errors
///
/// * Description of error conditions and error types returned
///
/// # Safety
///
/// * Requirements the caller must uphold for `unsafe` code
///
/// # Examples
///
/// ```rust
/// let result = function_call(...);
/// assert_eq!(result, ...);
/// ```
```

---

## Best Practices

- Use **active voice** and keep the first sentence a concise summary.
- Describe **ownership**, **borrowing**, and **lifetimes** clearly.
- Document **trait contracts**: specify expectations for implementors.
- Always provide **examples** that compile and pass.
- Prefer **doctests** over inline comments—they act as both examples and tests.
- Use **Markdown** for formatting (`*italic*`, `**bold**`, `# Headings`).
- Include **cross-references** using `[TypeName]` or `[module::item]` syntax.

---

## Crate & Module Docs

Use `//!` to describe the purpose, scope, and usage of a module or crate:

```rust
//! # My Crate
//!
//! Utilities for working with binary trees.
//!
//! ## Features
//! - Tree traversal
//! - Balancing
//! - Serialization
```

---

## Sections Explained

| Section     | Purpose                                                                 |
|-------------|-------------------------------------------------------------------------|
| Summary     | A one-line overview using active voice.                                |
| Description | A deeper explanation of logic, intent, and context.                    |
| Arguments   | Each parameter's name, type, and purpose.                              |
| Returns     | Return type and structure of successful results.                       |
| Panics      | When and why the function panics (e.g., out of bounds, unwrap).        |
| Errors      | What causes failure and what error types are returned.                 |
| Safety      | Required for `unsafe fn`—caller obligations and memory guarantees.     |
| Examples    | Demonstrative usage with assertions and realistic context.             |

---

## Prompt Template for AI Assistants

> **"Write a Rustdoc comment for this item using advanced formatting, including `# Arguments`, `# Returns`, `# Panics`, `# Errors`, `# Safety`, and `# Examples` where relevant. Use clear markdown and valid doctests."**

---

## ️ Tools for Working with Rustdoc

- `cargo doc --open`: Generate and view documentation locally.
- `cargo test`: Verifies correctness of doctests.
- [docs.rs](https://docs.rs): Public documentation host for crates.
- `rust-analyzer`: IDE support for inline doc rendering.
- [`rustdoc-stripper`](https://github.com/gtk-rs/rustdoc-stripper): Tool for cleaning up and extracting doc comments.

---

## Summary

Well-documented code improves maintainability, usability, and collaboration. Following this advanced standard ensures your Rust APIs are predictable, testable, and easy to integrate with tools and users alike.

