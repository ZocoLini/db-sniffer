mod xml;
mod jpa;

use crate::db_objects::{
    Column, ColumnType, RelationType
    ,
};
use crate::naming;
use dotjava::{Field, Type, Visibility};
use std::cmp::PartialEq;
use std::ops::Add;
use std::path::{Path, PathBuf};

pub use jpa::JPAGenerator;
pub use xml::XMLGenerator;

// TODO: Decimal not working. Ask other for the correct mapping type
fn column_type_to_hibernate_type(column_type: &ColumnType) -> String {
    match column_type {
        ColumnType::Integer(_) => "int".to_string(),
        ColumnType::Text(_) | ColumnType::Varchar(_) => "string".to_string(),
        ColumnType::Blob(_) => "binary".to_string(),
        ColumnType::Boolean => "boolean".to_string(),
        ColumnType::Date => "date".to_string(),
        ColumnType::DateTime => "timestamp".to_string(),
        ColumnType::Time => "time".to_string(),
        ColumnType::Double(_) => "double".to_string(),
        ColumnType::Float(_) => "float".to_string(),
        ColumnType::Char(_) => "char".to_string(),
        ColumnType::Decimal(_, _) | ColumnType::Numeric(_) => "big_decimal".to_string(),
    }
}

fn column_type_to_java_type(column_type: &ColumnType) -> Type {
    match column_type {
        ColumnType::Integer(_) => Type::integer(),
        ColumnType::Text(_) | ColumnType::Varchar(_) => Type::string(),
        ColumnType::Blob(_) => Type::new("byte[]".to_string(), "".to_string()),
        ColumnType::Boolean => Type::boolean(),
        ColumnType::Date | ColumnType::DateTime | ColumnType::Time => {
            Type::new("Date".to_string(), "java.util".to_string())
        }
        ColumnType::Double(_) => Type::double(),
        ColumnType::Float(_) => Type::float(),
        ColumnType::Char(_) => Type::character(),
        ColumnType::Decimal(_, _) | ColumnType::Numeric(_) => {
            Type::new("BigDecimal".to_string(), "java.math".to_string())
        }
    }
}

fn get_java_package_name(path: &Path) -> Option<String> {
    let mut package = String::new();
    package = String::new();

    let mut current = path;

    let mut current_file_name = current.file_name().unwrap().to_str().unwrap();
    while current_file_name != "src" && current_file_name != "java" {
        package = current_file_name.to_string() + "." + &package;

        current = current
            .parent()
            .expect("Reached a folder withour parent folder before src or java");
        current_file_name = current.file_name().unwrap().to_str().unwrap();
    }

    package = package.trim_end_matches('.').to_string();

    Some(package)
}

fn get_java_src_root(path: &Path) -> Option<PathBuf> {
    let mut current = path;

    while current.parent().is_some() {
        if current.ends_with("src") || current.ends_with("java") {
            return Some(PathBuf::from(current));
        }

        current = current.parent().unwrap();
    }

    None
}

fn generate_field(column: &Column) -> Field {
    let field_name = naming::to_lower_camel_case(column.name());
    let field_type = column_type_to_java_type(column.r#type());

    Field::new(field_name, field_type, Some(Visibility::Private), None)
}

fn gen_rel_field(rel_type: &RelationType, rel_owner: bool, field_name: String, field_type: Type) -> Field {
    
    let rel_type = if rel_owner {
        rel_type
    } else {
        &rel_type.inverse()
    };
    
    match rel_type {
        RelationType::OneToMany | RelationType::ManyToMany => {
            let mut rel_type = Type::new("Set".to_string(), "java.util".to_string());
            rel_type.add_generic(field_type);

            Field::new(
                format!("{}s", field_name),
                rel_type,
                Some(Visibility::Private),
                None,
            )
        }
        RelationType::OneToOne | RelationType::ManyToOne => {
            Field::new(field_name, field_type, Some(Visibility::Private), None)
        }
    }
}

fn escape_xml_special_chars(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_get_java_package_name() {
        let path = PathBuf::from("/home/user/projects/my_project/src/com/example/model");
        let package = get_java_package_name(&path);

        assert_eq!(package, Some("com.example.model".to_string()));

        let path = PathBuf::from("/home/user/projects/my_project/src/main/java/com/example/model");
        let package = get_java_package_name(&path);

        assert_eq!(package, Some("com.example.model".to_string()));
    }

    #[tokio::test]
    async fn test_get_java_src_root() {
        let path = PathBuf::from("/home/user/projects/my_project/src/com/example/model");
        let src = get_java_src_root(&path);

        assert_eq!(
            src,
            Some(PathBuf::from("/home/user/projects/my_project/src"))
        );

        let path = PathBuf::from("/home/user/projects/my_project/src/main/java/com/example/model");
        let src = get_java_src_root(&path);

        assert_eq!(
            src,
            Some(PathBuf::from(
                "/home/user/projects/my_project/src/main/java"
            ))
        );
    }
}
