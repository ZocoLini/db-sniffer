pub struct Interface {
    name: String
}

impl Interface {
    pub fn new(name: String) -> Self { Self { name }}
}

impl Into<String> for Interface {
    fn into(self) -> String {
        self.name
    }
}