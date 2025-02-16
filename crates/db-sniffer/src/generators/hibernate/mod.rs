mod jpa;
mod xml;

use crate::db_objects::{Column, ColumnType, RelationType};
use crate::generators::java;
use crate::naming;
use dotjava::{Field, Type, Visibility};
pub use jpa::JPAGenerator;
use std::cmp::PartialEq;
use std::ops::Add;
use std::path::{Path, PathBuf};
pub use xml::XMLGenerator;

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
        current_file_name = current
            .file_name()
            .expect("Reached a folder withour parent folder before src or java")
            .to_str()
            .unwrap();
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
    let field_type = column.r#type().to_java();

    Field::new(field_name, field_type, Some(Visibility::Private), None)
}

fn gen_rel_field(
    rel_type: &RelationType,
    rel_owner: bool,
    field_name: String,
    field_type: Type,
) -> Field {
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
