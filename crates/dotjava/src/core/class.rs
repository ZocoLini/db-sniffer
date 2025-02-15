use crate::{Field, Method};
use std::collections::HashSet;
use crate::core::interface::Interface;

pub struct Class {
    name: String,
    package: String,
    fields: Vec<Field>,
    methods: Vec<Method>,
    imports: Vec<String>,
    interfaces: Vec<Interface>
}

impl Class {
    pub fn new(
        name: String,
        package: String,
        fields: Vec<Field>,
        methods: Vec<Method>
    ) -> Self {
        let mut imports = HashSet::new();

        for field in fields.iter() {
            let package = field.package_required();
            
            if !package.is_empty() {
                imports.insert(package);
            }
        }
        
        for method in methods.iter() {
            let package = method.package_required();
            
            if !package.is_empty() {
                imports.insert(package);
            }
        }
        
        Self {
            name,
            package,
            fields,
            methods,
            imports: imports.into_iter().collect(),
            interfaces: Vec::new()
        }
    }
    
    pub fn add_equals_method(&mut self) {
        self.imports.push("java.util.Objects".to_string());
        self.methods.push(Method::equals(self));
    }
    
    pub fn add_hash_code_method(&mut self) {
        self.methods.push(Method::hash_code(self));
    }
    
    pub fn fields(&self) -> &Vec<Field> {
        &self.fields
    }
    
    pub fn name(&self) -> &String {
        &self.name
    }
    
    pub fn add_interface(&mut self, interface: Interface) {
        self.imports.push(interface.package_required());
        self.interfaces.push(interface);
    }
}

impl Into<String> for Class {
    fn into(self) -> String {
        let mut imports = String::new();

        for import in self.imports {
            imports.push_str(&format!("import {};\n", import));
        }

        let mut fields = String::new();

        for field in self.fields {
            fields.push_str(&format!("    {}\n", <Field as Into<String>>::into(field)));
        }

        let mut methods = String::new();

        for method in self.methods {
            methods.push_str(&format!("    {}\n", <Method as Into<String>>::into(method)));
        }

        let implements = if self.interfaces.is_empty() {
            "".to_string()
        } else {
            let interfaces = self.interfaces
                .iter()
                .map(|interface| interface.name().clone())
                .collect::<Vec<String>>()
                .join(", ");
            
            format!(" implements {}", interfaces)
        };
        
        let package_string = if self.package.is_empty() {
            "".to_string()
        } else {
            format!("package {};", self.package)
        };
        
        format!(
            "{package_string}\n\n{imports}\n\npublic class {}{implements} {{\n{fields}\n{methods}\n}}",
            self.name
        )
    }
}