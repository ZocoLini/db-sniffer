pub struct Interface {
    name: String,
    package: String,
}

impl Interface {
    pub fn new(name: String, package: String) -> Self { Self { name, package }}
    
    pub fn package_required(&self) -> String {
        format!("{}.{}", self.package, self.name)
    }
    
    pub fn name(&self) -> &String {
        &self.name
    }
}

impl Into<String> for Interface {
    fn into(self) -> String {
        format!(r#"interface {}"#, self.name)
    }
}