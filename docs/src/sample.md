## Contents of files used for examples.


# Sample Rust file: sample.rs
```

use log::debug;

trait Greeter {
    fn greet(&self, name: &str) -> String;
}

struct HelloApp;

impl Greeter for HelloApp {
    fn greet(&self, name: &str) -> String {
        format!("Hello, {name}!")
    }
}

fn make_message() -> String {
    let app = HelloApp;
    app.greet("world")
}

fn print_message(message: &str) {
    println!("{message}");
}

/// a doc 
fn fixture_signature_with_colon_colon_in_body() {
    // and a comment
    let marker = "::B5";
    debug!("yo!")
    println!("{marker}");
}

fn main() {
    let message = make_message();
    print_message(&message);
}

mod tests {
    fn always_pass_message(){
        println!("always passing");
    }
}


```
# Sample Python file: pysample.py
```

import os, sys
from pathlib import Path


def do_init(func):
    return func


class Calculator:
    "A simple calculator class to demonstrate Python basics."

    def __init__(self, name: str):
        # a comment
        self.name = name
        self.history = []

    def add(self, a: int, b: int) -> int:
        """Add two numbers  ."""
        result_add = a + b
        self.history.append(f"{a} + {b} = {result_add}")
        return result_add

    def multiply(self, a: int,
                b: int) -> int:
        """Multiply two numbers."""
        result_multiply = a * b
        self.history.append(f"{a} * {b} = {result_multiply}")
        return result_multiply

    @do_init
    def show_history(self) -> None:
        """Display calculation history
        ."""
        # commented out
        print(f"\n{self.name}'s History:")
        for entry in self.history:
            print(f"  {entry}")


# Usage example
if __name__ == "__main__":
    calc = Calculator("MyCalc")
    calc.add(5, 3)
    calc.multiply(4, 7)
    calc.show_history()
```

