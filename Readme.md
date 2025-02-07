# Rolly Polly

## Introduction

Rolly Polly is a simple asteroid clone game written in Rust, motivated by a desire to learn Rust programming and create something fun with my daughters.

## Getting Started

### Prerequisites

-   Rust toolchain: Ensure you have Rust installed. You can download it from [https://www.rust-lang.org/](https://www.rust-lang.org/).

### Building and Running

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/yourusername/rolly_polly.git
    cd rolly_polly
    ```

2.  **Build the project:**

    ```bash
    cargo build
    ```

3.  **Build for WebAssembly:**

    ```bash
    rustup target install wasm32-unknown-unknown
    ```
    ```bash
    cargo run --target wasm32-unknown-unknown
    ```

4.  **Run the game locally:**

    ```bash
    cargo run
    ```

## Contributing

Feel free to contribute to the project by submitting pull requests. Please ensure your code adheres to the project's coding standards and includes appropriate tests.

### Reporting Issues

If you encounter any issues, please report them on the [issue tracker](https://github.com/yourusername/rolly_polly/issues).

### Feature Requests

We welcome feature requests! If you have an idea to improve the game, please open a new issue with the "feature request" label.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.