# Contributing to `agy-racer`

## Stake Your Reputation

This project is built with heavy AI assistance (see [`DISCLOSURE.md`](DISCLOSURE.md)), and we welcome contributions written with the help of AI tools. However, we operate under one simple rule:

> **I am happy to receive your AI-generated code, on the condition that I will judge your capabilities by it as if you handwrote every line yourself—and if you send me unreviewed slop, I will think less of you and refuse your changes.**

Generating code with an LLM is cheap; reviewing, debugging, and maintaining it is expensive. Do not outsource the work of reading, understanding, and testing your code onto the reviewer:

1. **You Own What You Submit**: "The AI wrote that part" is never an excuse. When your name is on the commit or pull request, you are staking your reputation on every line of the diff.
2. **Read & Understand Your Diff**: Before opening a PR, read your own diff from top to bottom. Make sure it follows the codebase structure in [`ARCHITECTURE.md`](ARCHITECTURE.md)—no magic numbers, no monolithic functions, no inline math without named helpers, and no robotic AI comment clutter.
3. **Actually Run & Playtest It**:
   - Run `cargo test` and verify all tests pass with zero warnings.
   - Launch the game (`cargo run --release`) and verify your changes actually work in-game.

---

## License

By contributing to `agy-racer`, you agree that your contributions will be licensed under the [Apache License, Version 2.0](LICENSE.md).
