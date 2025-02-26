# Backgammon CLI

A command-line interface implementation of the classic board game Backgammon, featuring a text-based UI with colored output.

## Features

- Text-based board visualization with colored pieces
- Standard backgammon rules implementation
- Support for dice rolling and doubles
- Move validation
- Bar and bearing off mechanics
- Backgammon notation support
- Player turn management

## Installation

To install the game, you'll need Rust and Cargo installed on your system. Then:

```bash
# Clone the repository
git clone https://github.com/grahampreston/backgammon-cli
cd backgammon-cli

# Build and install
cargo install --path .
```

## Usage

To start a new game:

```bash
backgammon-cli
```

### Game Controls

The game uses standard backgammon notation for moves. For example:

- `1/2` moves a piece from point 1 to point 2
- `bar/1` moves a piece from the bar to point 1
- `20/off` bears off a piece from point 20
- Multiple moves can be combined with spaces: `1/2 2/3`

## Development

To run tests:

```bash
cargo test
```

To check code style and run lints:

```bash
cargo clippy
```
