use crate::{Field, Method};
use std::collections::HashSet;

pub struct Class {
    name: String,
    package: String,
    fields: Vec<Field>,
    methods: Vec<Method>,
    imports: Vec<String>,
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
        }
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
            fields.push_str(&format!("    {}\n", <Field as Into<String>>::into(field.into())));
        }

        let mut methods = String::new();

        for method in self.methods {
            methods.push_str(&format!("    {}\n", <Method as Into<String>>::into(method.into())));
        }

        format!(
            "package {};\n\n{}\n\npublic class {} {{\n{}\n{}\n}}",
            self.package, imports, self.name, fields, methods
        )
    }
}