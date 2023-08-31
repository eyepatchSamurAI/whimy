# Contributing to Whimy 🌟

Hello and welcome! We're thrilled that you're interested in contributing to [Your Project Name]. Your help is essential for keeping it great. 🙏

## Table of Contents 📚

1. [Code of Conduct](#code-of-conduct)
2. [Setting Up Development Environment](#setting-up-development-environment)
4. [How to Contribute](#how-to-contribute)
5. [Issue Guidelines](#issue-guidelines)
6. [Pull Request Process](#pull-request-process)
7. [Code Style](#code-style)
8. [Testing](#testing)
9. [Community](#community)

## Code of Conduct 🤝

Please read and follow our [Code of Conduct](./CODE_OF_CONDUCT.md).

## Setting Up Development Environment 🛠

1. Fork Whimy
2. Clone your fork
3. `yarn install`
4. `yarn build`
5. `cd .\test_signed_data\custom_signed`
6. `.\setting_up_cert_testing.ps1`

## How to Contribute 🤔

1. Find an issue or suggest a feature 
2. Fork the repository.
3. Create a new branch.
4. Make your changes.
5. Submit a pull request

## Issue Guidelines 📝

1. Check existing issues.
2. Use the issue template.
3. Be as descriptive as possible.

## Pull Request Process 📥

1. Fork and clone the repository.
2. Create a new branch based on `main`.
3. Make your changes.
4. Run tests.
5. Submit a Pull Request and reference any related issues.
6. Await review.

## Code Style 🎨

- Best Rust coding practices
- Make sure to run `cargo fmt` and `cargo clippy`

## Testing 🧪

- Run `cargo test -- --test-threads=1` to execute tests.
- Run `cargo tarpaulin --exclude-files "*\\mod.rs" --out Html -- --test-threads=1` to see test coverage

---

Thank you for investing your time in contributing to our project! 🎉
