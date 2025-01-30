use crate::Class;
use std::ops::Add;

#[derive(Copy, Clone)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

impl Into<String> for Visibility {
    fn into(self) -> String {
        match self {
            Visibility::Public => "public".to_string(),
            Visibility::Protected => "protected".to_string(),
            Visibility::Private => "private".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Type {
    name: String,
    package: String,
    generics: Vec<Type>,
}

impl Type {
    pub fn new(name: String, package: String) -> Self {
        Self {
            name,
            package,
            generics: vec![],
        }
    }

    pub fn new_primitive(name: String) -> Self {
        Self::new(name, "".to_string())
    }

    pub fn string() -> Self {
        Self::new("String".to_string(), "".to_string())
    }

    pub fn integer() -> Self {
        Self::new("Integer".to_string(), "".to_string())
    }

    pub fn boolean() -> Self {
        Self::new("Boolean".to_string(), "".to_string())
    }

    pub fn character() -> Self {
        Self::new("Character".to_string(), "".to_string())
    }

    pub fn byte() -> Self {
        Self::new("Byte".to_string(), "".to_string())
    }

    pub fn short() -> Self {
        Self::new("Short".to_string(), "".to_string())
    }

    pub fn long() -> Self {
        Self::new("Long".to_string(), "".to_string())
    }

    pub fn float() -> Self {
        Self::new("Float".to_string(), "".to_string())
    }

    pub fn double() -> Self {
        Self::new("Double".to_string(), "".to_string())
    }

    pub fn void() -> Self {
        Self::new("void".to_string(), "".to_string())
    }

    pub fn package_required(&self) -> String {
        if self.package.is_empty() {
            return "".to_string();
        }

        format!("{}.{}", self.package, self.name)
    }

    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn add_generic(&mut self, generic: Type) {
        self.generics.push(generic);
    }
}

impl Into<String> for Type {
    fn into(self) -> String {
        if self.generics.is_empty() {
            return self.name;
        }

        let generics_string = self
            .generics
            .into_iter()
            .map(|a| a.into())
            .collect::<Vec<String>>()
            .join(", ");

        format!("{}<{generics_string}>", self.name)
    }
}

#[derive(Clone)]
pub struct Field {
    name: String,
    r#type: Type,
    visibility: Option<Visibility>,
    value: Option<String>,
}

impl Field {
    pub fn new(
        name: String,
        r#type: Type,
        visibility: Option<Visibility>,
        value: Option<String>,
    ) -> Self {
        Self {
            name,
            r#type,
            visibility,
            value,
        }
    }

    pub fn package_required(&self) -> String {
        self.r#type.package_required()
    }

    pub fn getter(&self) -> Method {
        Method::getter(self)
    }

    pub fn setter(&self) -> Method {
        Method::setter(self)
    }

    pub fn getters_setters(&self) -> Vec<Method> {
        vec![self.getter(), self.setter()]
    }

    pub fn into_record_field_string(self) -> String {
        format!("{} {}", self.r#type.name, self.name)
    }
}

impl Into<String> for Field {
    fn into(self) -> String {
        let visibility_string = if let Some(a) = self.visibility {
            let a: String = a.into();
            format!("{a} ")
        } else {
            "".to_string()
        };

        let value_string = if let Some(a) = self.value {
            format!(" = {};", a)
        } else {
            ";".to_string()
        };

        let field_type: String = self.r#type.into();

        format!(
            "{visibility_string}{} {}{}",
            field_type, self.name, value_string
        )
    }
}

#[derive(Clone)]
pub struct Method {
    name: String,
    r#type: Type,
    visibility: Option<Visibility>,
    parameters: Vec<(Type, String)>,
    body: Option<String>,
}

impl Method {
    pub fn new(
        name: String,
        r#type: Type,
        visibility: Option<Visibility>,
        parameters: Vec<(Type, String)>,
        body: Option<String>,
    ) -> Self {
        Self {
            name,
            r#type,
            visibility,
            parameters,
            body,
        }
    }

    pub fn getter(field: &Field) -> Self {
        let mut field_name_upper_camel_case = field.name.clone();
        field_name_upper_camel_case.replace_range(0..1, &field.name[0..1].to_uppercase());

        let name = format!("get{}", &field_name_upper_camel_case);
        let body = Some(format!("return this.{};", &field.name));

        Self::new(
            name,
            field.clone().r#type,
            Some(Visibility::Public),
            vec![],
            body,
        )
    }

    pub fn setter(field: &Field) -> Self {
        let mut field_name_upper_camel_case = field.name.clone();
        field_name_upper_camel_case.replace_range(0..1, &field.name[0..1].to_uppercase());

        let name = format!("set{}", field_name_upper_camel_case);
        let body = Some(format!("this.{} = {};", field.name, field.name));

        let field_name = field.name.clone();

        Self::new(
            name,
            Type::void(),
            Some(Visibility::Public),
            vec![(field.r#type.clone(), field_name)],
            body,
        )
    }

    pub fn equals(class: &Class) -> Self {
        let mut body = String::new();

        body = body.add(&format!(
            r#"
        if (o == null || getClass() != o.getClass()) return false;

        {} that = ({}) o;
        "#,
            class.name(),
            class.name()
        ));

        let fields_comp = class
            .fields()
            .iter()
            .map(|field| {
                format!(
                    "Objects.equals(this.{}, that.{})",
                    field.name, field.name
                )
            })
            .collect::<Vec<String>>()
            .join(" && ");

        body = body.add(&format!("return {fields_comp};"));

        let param = Type::new("Object".to_string(), "java.lang".to_string());

        Method {
            name: "equals".to_string(),
            r#type: Type::new_primitive("boolean".to_string()),
            visibility: Some(Visibility::Public),
            parameters: vec![(param, "o".to_string())],
            body: Some(body),
        }
    }

    pub fn hash_code(class: &Class) -> Self {
        let mut body = String::new();

        body = body.add(&format!(
            "int result = Objects.hashCode({});\n\n",
            &class.fields()[0].name
        ));

        let fields_hash = class
            .fields()
            .iter()
            .skip(1)
            .map(|field| format!("result = 31 * result * Objects.hashCode({});", field.name))
            .collect::<Vec<String>>()
            .join(";\n");

        body = body.add(&format!("{fields_hash}\n\nreturn result;"));

        Method {
            name: "hashCode".to_string(),
            r#type: Type::new_primitive("int".to_string()),
            visibility: Some(Visibility::Public),
            parameters: vec![],
            body: Some(body),
        }
    }

    pub fn package_required(&self) -> String {
        self.r#type.package_required()
    }
}

impl Into<String> for Method {
    fn into(self) -> String {
        let visibility_string = if let Some(a) = self.visibility {
            let a: String = a.into();
            format!("{a} ")
        } else {
            "".to_string()
        };

        let parameters_string = self
            .parameters
            .into_iter()
            .map(|(r#type, name)| {
                let field_type: String = r#type.into();

                format!("{field_type} {name}")
            })
            .collect::<Vec<String>>()
            .join(", ");

        let body_string = if let Some(a) = self.body {
            format!(" {{\n{}\n}}", a)
        } else {
            ";".to_string()
        };

        let return_type: String = self.r#type.into();

        format!(
            "{visibility_string}{return_type} {}({parameters_string}){body_string}",
            self.name
        )
    }
}
