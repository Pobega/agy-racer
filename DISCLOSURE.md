# AI Development Disclosure

`agy-racer` is developed with heavy AI involvement using **Google Antigravity** (powered by **Gemini** models), directed and reviewed by [@pobega](https://github.com/pobega).

Almost all code, procedural assets, and documentation in this repository are authored or refactored through human-AI pair programming. Rather than using AI to dump opaque boilerplate, development follows strict human-readability and maintainability standards (see [`ARCHITECTURE.md`](ARCHITECTURE.md)):

- **Human-Guided Design**: Game design, physics feel, architecture decisions, and feature priorities are directed and playtested by the maintainer.
- **Readable & Moddable Code**: Code is structured into small, single-purpose modules with named constants instead of magic numbers, hot-reloadable JSON/PNG configs (`F5`), and automated unit tests (`cargo test`) so that any human developer can easily read, debug, or modify the project by hand.
