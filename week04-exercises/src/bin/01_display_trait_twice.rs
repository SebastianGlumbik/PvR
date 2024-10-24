use std::fmt::{Display, Formatter};

struct Person {
    name: String,
    age: u8,
}

struct PersonJSON<'a>(&'a Person);

impl<'a> Display for PersonJSON<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{\n\tname: {},\n\tage: {}\n}}", self.0.name, self.0.age)
    }
}

struct PersonXML<'a>(&'a Person);

impl<'a> Display for PersonXML<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<body>\n\t<name>{}</name>\n\t<age>{}</age>\n</body>",
            self.0.name, self.0.age
        )
    }
}

// We want to have the ability to print the above struct both as JSON and XML.
// What are the different ways that we can use to achieve that?
// Is there a way to do it so that we can print persons as JSON or XML through the same trait
// (`Display`), so that we can do it through a unified interface?

fn print_something<T: Display>(t: &T) {
    println!("Printing\n{t}");
}

fn main() {
    let person = Person {
        name: "Kuba B.".to_string(),
        age: 30,
    };
    print_something(&PersonJSON(&person));
    print_something(&PersonXML(&person));
}
