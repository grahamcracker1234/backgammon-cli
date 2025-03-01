# 🎲 Backgammon CLI

A command-line interface implementation of the classic board game [Backgammon](https://en.wikipedia.org/wiki/Backgammon), featuring a text-based UI with colored output.

![License](https://img.shields.io/badge/license-MIT-blue.svg)

## 📋 Table of Contents

- [Features](#-features)
- [Installation](#-installation)
- [Usage](#-usage)
  - [Game Controls](#-game-controls)
- [Development](#-development)
- [Contributing](#-contributing)
- [License](#-license)

## ✨ Features

- 🎮 Text-based board visualization with colored pieces
- 📏 Standard backgammon rules implementation
- 🎲 Support for dice rolling and doubles
- ✅ Move validation
- 🚫 Bar and bearing off mechanics
- 📝 Backgammon notation support
- 👥 Player turn management

## 🚀 Installation

To install the game, you'll need Rust and Cargo installed on your system.

```sh
# Clone the repository
git clone https://github.com/grahamcracker1234/backgammon-cli.git
cd backgammon-cli

# Build and install
cargo install --path .
```

## 🎮 Usage

To start a new local 2-player game:

```sh
backgammon-cli
```

> **Note:** There is not yet support for CPU opponents or online play.

### 🎯 Game Controls

The game uses [standard backgammon notation](https://en.wikipedia.org/wiki/Backgammon_notation) for moves:

| Notation | Description |
|----------|-------------|
| `1/2`    | Moves a piece from point 1 to point 2 |
| `bar/1`  | Moves a piece from the bar to point 1 |
| `20/off` | Bears off a piece from point 20 |
| `8/3/1`  | Multiple moves with the same piece (chained) |
| `1/2 5/9`| Multiple separate moves (combined with spaces) |

## 👨‍💻 Development

```sh
cargo run     # Run the game
cargo test    # Run the tests
cargo clippy  # Check code style and run lints
```

## 📚 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
