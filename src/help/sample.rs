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