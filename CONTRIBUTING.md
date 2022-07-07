# Contributing Guidelines

> **Note:** Despite what this document says, The owner may not be able to take actions on these contributions at the moment.

---

Thank you for taking your time contributing. I'll address these contributions as soon as possible.

## Opening an issue

If you have an issue, question, or suggestion, you may open an issue.

## Using REPL

While the compiler is not yet fully implemented. We have repl for testing out the implemented features. As of writing, this includes parser and type inference.

Before running the repl, you'll need to [install Cargo] first. Additionally, this requires knowledge of butter syntax of course: [read the documents].

Unless it's an "not yet implemented" error, if you found any issue, please report it.

[install cargo]: https://www.rust-lang.org/tools/install
[read the documents]: doc/README.md

### Parser REPL

Parser REPL can be started with `cargo run -- parser-repl` command. This REPL only parses expressions, you can use [blocks] to parse statements. Additionally, you can use `:{` `:}` to input multiline block expression.

[blocks]: doc/language/block.md

```txt
> 10 + 20
[ast will be printed here]

> :{
... a = 20;
... a
... :}
[ast will be printed here]
```

This REPL will output verbose and sparsely formatted AST.

### Type inference REPL

Type inference REPL can be started with `cargo run -- type-repl` command. This REPL parses an expression and outputs the inferred type.

```txt
> @val 10
[inferred type will be printed here]
```

## Opening a pull request

If you know how to fix such issues, consider forking and opening a pull request. Any form of pull requests is welcome, this includes typographic fixes and code improvements.

If such is a huge change especially to the codebase, please let us know by opening an issue or commenting in already existing issue page before working on the fork.

<!--

When opening a pull request, please make sure it follows the following conventions:

- There's no spelling nor grammatical errors
- Rust codes are formatted with [Rustfmt], [Cargo] comes with it and you can use the command `cargo fmt`
- Markdown files are linted with [DavidAnson/markdownlint]

[Rustfmt]: https://github.com/rust-lang/rustfmt
[Cargo]: https://github.com/rust-lang/cargo
[DavidAnson/markdownlint]: https://github.com/DavidAnson/markdownlint

You may instead allow me to modify your code before merging it so it comply with the conventions.

-->

---

[License](LICENSE)
