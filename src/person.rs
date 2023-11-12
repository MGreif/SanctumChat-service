
#[derive(Debug)]
pub struct Person {
    pub name: String,
    pub age: u32
}

impl Person {
    pub fn new(name: &str, age: u32) -> Person {
        Person { name: String::from(name), age }
    }
    pub fn get_name(&self) -> &String  {
        &self.name
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = String::from(name);
    }
}
    

