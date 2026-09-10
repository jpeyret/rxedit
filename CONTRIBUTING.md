# Contributing to rxedit

Thank you for your interest in contributing to rxedit!

## Ways to help

We welcome help with bug fixes, documentation improvements, and small feature work that matches the project’s scope.

- More idiomatic Rust.

- Architectural simplifications (and module rationalization).

- Suggestions for extra features.  Keep in mind the tension between that and presenting the user with an overcomplex API.

- Suggestions for making the DSL more intuitive.  The config system is intended to allow user experimentation.


If you are unsure whether a change is a good fit, please open an issue or ask before investing a lot of time.  For example:

- Significant architectural changes for the sake of performance.  Let's first make it right.

- Large AI-driven PRs.  We expect to know your PR, what it is doing and how  And keep change scope as narrow as possible.



## Getting Started

1. **Clone the repository**
   ```bash
   git clone https://github.com/jpeyret/rxedit.git
   cd rxedit
   ```

2. **Build and test**
   ```bash
   cargo build
   cargo test
   cargo clippy --all-targets --all-features
   ```

3. **Run the regression test suite**
   ```bash
   /path/to/shellscripts/runpylazy.sh
   ```

## Development Workflow

### Before committing

- Run `cargo fmt` to format code
- Run `cargo clippy --all-targets --all-features` to check for warnings
- Run `cargo test --release` to ensure tests pass
- Update `CHANGELOG.md` if adding features or fixing bugs

### Code Style

- Follow Rust naming conventions
- Prefer clarity over cleverness
- Add tests for new features
- Add doc comments for public APIs

## Bug Reports

Please include:
- A minimal reproducible example
- Expected vs. actual behavior
- Your environment (OS, Rust version, etc.)

## Feature Requests

Describe the use case and how it would benefit other users.

## Pull Request Process

1. Create a feature branch: `git checkout -b feature/your-feature`
2. Make focused commits with clear messages
3. Ensure all tests pass: `cargo test --release`
4. Push your branch and create a pull request
5. Link any related issues in the PR description

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
