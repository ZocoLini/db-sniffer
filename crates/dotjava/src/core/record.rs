use crate::Field;
use std::collections::HashSet;

pub struct Record {
    package: String,
    name: String,
    fields: Vec<Field>,
    imports: Vec<String>,
}

impl Record {
    pub fn new(name: String, package: String, fields: Vec<Field>) -> Self {
        let mut imports = HashSet::new();

        for field in fields.iter() {
            let package = field.package_required();

            if !package.is_empty() {
                imports.insert(package);
            }
        }

        Self {
            name,
            package,
            fields,
            imports: imports.into_iter().collect(),
        }
    }
}

impl Into<String> for Record {
    fn into(self) -> String {
        let mut imports = String::new();

        for import in self.imports {
            imports.push_str(&format!("import {};\n", import));
        }

        let fields = self
            .fields
            .into_iter()
            .map(|f| f.into_record_field_string())
            .collect::<Vec<String>>()
            .join(", ");

        format!(
            "package {};\n\n{imports}\n\npublic record {}({fields}) {{}}",
            self.package, self.name
        )
    }
}
