use crate::db_objects::{
    Column, ColumnType, Database, Dbms, GenerationType, KeyType, Relation, RelationType,
    Table,
};
use crate::naming;
use crate::sniffers::SniffResults;
use dotjava::{Class, Field, Interface, Type, Visibility};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fs;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::slice::Iter;

pub struct AnnotatedClassGenerator;

impl AnnotatedClassGenerator {
    fn generate_table_files(&self, _sniff_results: SniffResults, _target_path: PathBuf) {
        todo!()
    }
}

pub struct XMLGenerator<'a> {
    target_path: &'a PathBuf,
    sniff_results: &'a SniffResults,
    package: String,
    src_path: PathBuf,
}

impl<'a> XMLGenerator<'a> {
    pub fn new(sniff_results: &'a SniffResults, target_path: &'a PathBuf) -> Option<Self> {
        let src_path = get_java_src_root(target_path);
        let package = get_java_package_name(target_path);

        let src_path = if let Some(o) = src_path {
            o
        } else {
            println!("src dir not found as a parent od the output dir");
            return None;
        };

        let package = if let Some(o) = package {
            o
        } else {
            println!("The package name couldn't be determined");
            return None;
        };

        #[cfg(test)]
        {
            println!("Detected package name: {package}");
            println!(
                "Found source root folder: {}",
                src_path.to_str().unwrap_or_default()
            )
        }

        Some(XMLGenerator {
            target_path,
            sniff_results,
            package,
            src_path,
        })
    }

    pub fn generate(&self) {
        let target_path = self.target_path;
        let sniff_results = self.sniff_results;

        if !target_path.exists() {
            if let Err(e) = fs::create_dir_all(target_path) {
                println!(
                    "Target path ({})  could not be created: {e}",
                    target_path.to_str().unwrap_or_default()
                );
                return;
            };
        }

        self.generate_tables_files(sniff_results.database().tables());

        let conf_xml = self.generate_conf_xml();
        let conf_file_path = self.src_path.join("hibernate.cfg.xml");

        fs::File::create(&conf_file_path).unwrap();

        fs::write(conf_file_path, conf_xml).unwrap();
    }

    fn generate_conf_xml(&self) -> String {
        let conn_params = self.sniff_results.conn_params();

        let mapping_files = self
            .sniff_results
            .database()
            .tables()
            .iter()
            .map(|t| {
                format!(
                    r#"<mapping resource="{}/{}.hbm.xml" />"#,
                    self.package.replace(".", "/"),
                    naming::to_upper_camel_case(t.name())
                )
            })
            .collect::<Vec<String>>()
            .join("\n         ");

        let (dialect, driver, conn_str) = match self.sniff_results.metadata() {
            Some(metadata) => match metadata.dbms() {
                Dbms::Mssql => (
                    "org.hibernate.dialect.SQLServerDialect",
                    "com.microsoft.sqlserver.jdbc.SQLServerDriver",
                    format!(
                        "jdbc:sqlserver://{}:{};databaseName={};trustServerCertificate=true",
                        conn_params.host().as_ref().unwrap(),
                        conn_params.port().unwrap(),
                        conn_params.dbname().as_ref().unwrap()
                    ),
                ),
                Dbms::MySQL => (
                    "org.hibernate.dialect.MySQLDialect",
                    "com.mysql.cj.jdbc.Driver",
                    format!(
                        "jdbc:mysql://{}:{}/{}",
                        conn_params.host().as_ref().unwrap(),
                        conn_params.port().unwrap(),
                        conn_params.dbname().as_ref().unwrap()
                    ),
                ),
            },
            None => ("", "", "".to_string()),
        };

        let properties = format!(
            r#"
        <property name="hibernate.dialect">{dialect}</property>
        <property name="hibernate.connection.driver_class">{driver}</property>
        <property name="hibernate.connection.url">{conn_str}</property>
        <property name="hibernate.connection.username">{}</property>
        <property name="hibernate.connection.password">{}</property>"#,
            escape_xml_special_chars(conn_params.user().as_ref().unwrap()),
            escape_xml_special_chars(conn_params.password().as_ref().unwrap()),
        );

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE hibernate-configuration PUBLIC 
"-//Hibernate/Hibernate Configuration DTD 3.0//EN" 
"http://www.hibernate.org/dtd/hibernate-configuration-3.0.dtd">

<hibernate-configuration>
    <session-factory>
        {properties}

        <property name="hibernate.hbm2ddl.auto">validate</property>
   
        <!-- Mapping files -->
        {mapping_files}
    </session-factory>

</hibernate-configuration>
            "#
        )
    }

    fn generate_tables_files(&self, tables: &Vec<Table>) {
        for table in tables {
            let table_xml = self.generate_table_xml(table);

            let table_file_path = self.target_path.join(format!(
                "{}.hbm.xml",
                naming::to_upper_camel_case(table.name())
            ));

            fs::File::create(&table_file_path).unwrap();
            fs::write(table_file_path, table_xml).unwrap();
            //

            let table_java = self.generate_table_java(table);
            let table_java_file_path = self.target_path.join(format!(
                "{}.java",
                naming::to_upper_camel_case(table.name())
            ));

            fs::File::create(&table_java_file_path).unwrap();
            fs::write(table_java_file_path, table_java).unwrap();

            if table.ids().len() > 1 {
                let composite_id_java = self.generate_composite_id(table);
                let composite_id_java_file_path = self.target_path.join(format!(
                    "{}Id.java",
                    naming::to_upper_camel_case(table.name())
                ));

                fs::File::create(&composite_id_java_file_path).unwrap();
                fs::write(composite_id_java_file_path, composite_id_java).unwrap();
            }
        }
    }

    fn generate_table_xml(&self, table: &Table) -> String {
        let package = &self.package;

        let mut used_names: HashMap<String, i32> = HashMap::new();

        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE hibernate-mapping PUBLIC
    "-//Hibernate/Hibernate Mapping DTD 3.0//EN"
    "http://www.hibernate.org/dtd/hibernate-mapping-3.0.dtd">

<hibernate-mapping>
  <class name="{package}.{}" table="{}">
{}
{}
{}
{}
  </class>
</hibernate-mapping>
        "#,
            naming::to_upper_camel_case(table.name()),
            table.name(),
            generate_id_xml(table, package),
            generate_properties_xml(table),
            generate_references_to_xml(
                table,
                package,
                self.sniff_results.database(),
                &mut used_names
            ),
            generate_referenced_by_xml(
                table,
                package,
                self.sniff_results.database(),
                &mut used_names
            )
        );

        return xml;

        fn generate_id_xml(table: &Table, package: &str) -> String {
            let id_columns = table.ids();
            let mut result = "    <!-- Id -->".to_string();

            if id_columns.is_empty() {
                return result;
            }

            if id_columns.len() == 1 {
                let id = id_columns[0];
                let gen_class = match id.key() {
                    KeyType::Primary(a) => match a {
                        GenerationType::None => "assigned",
                        GenerationType::AutoIncrement => "identity",
                    },
                    _ => panic!("This section should not be reached"),
                };

                result = result.add(&format!(
                    r#"
    <id name="{}" type="{}">
      {}
      <generator class="{}"/>
    </id>"#,
                    naming::to_lower_camel_case(id.name()),
                    column_type_to_hibernate_type(id.r#type()),
                    &generate_column_xml(id),
                    gen_class
                ));
            } else {
                result = result.add(&format!(
                    r#"
    <composite-id name="{}" class="{}.{}">"#,
                    "id",
                    package,
                    naming::to_upper_camel_case(&format!("{}Id", table.name())),
                ));

                for id_column in id_columns {
                    result = result.add(&format!(
                        r#"
      <key-property name="{}" type="{}">
        {}
      </key-property>
"#,
                        naming::to_lower_camel_case(id_column.name()),
                        column_type_to_hibernate_type(id_column.r#type()),
                        &generate_column_xml(id_column)
                    ));
                }

                result = result.add("    </composite-id>");
            }

            result
        }

        fn generate_properties_xml(table: &Table) -> String {
            let mut result = "\n    <!-- Properties -->".to_string();

            for column in table.columns() {
                if let KeyType::Primary(_) = column.key() {
                    continue;
                }

                if table.is_col_fk(column.name()) {
                    continue;
                }

                // TODO: decimal type not working with big decimal as it is defined right now
                result = result.add(&format!(
                    r#"
    <property name="{}" type="{}">
      {}
    </property>"#,
                    naming::to_lower_camel_case(column.name()),
                    column_type_to_hibernate_type(column.r#type()),
                    &generate_column_xml(column)
                ));
            }

            result
        }

        fn generate_column_xml(column: &Column) -> String {
            format!(
                r#"<column name="{}"{}{}/>"#,
                column.name(),
                if column.not_nullable() {
                    " not-null=\"true\""
                } else {
                    ""
                },
                if let KeyType::Unique = column.key() {
                    " unique=\"true\""
                } else {
                    ""
                }
            )
        }

        fn generate_multi_column_xml(columns: &Vec<&Column>) -> String {
            let mut result = "".to_string();

            for column in columns {
                result = result.add(&generate_column_xml(column)).add("\n        ");
            }

            result
        }

        // TODO: This function and the bottom one are really similar. Maybe they can be merged
        // TODO: This many parameters makes this function ugly af
        fn generate_references_to_xml(
            table: &Table,
            package: &str,
            database: &Database,
            used_names: &mut HashMap<String, i32>,
        ) -> String {
            let mut result = "\n    <!-- References -->".to_string();

            table.references().iter().for_each(|r| {
                if let KeyType::Primary(_) = table
                    .column(r.from()[0].name())
                    .expect("Should exists")
                    .key()
                {
                    result.push_str(&generate_relation_xml(
                        r, package, database, true, false, false, used_names,
                    ));
                } else {
                    result.push_str(&generate_relation_xml(
                        r, package, database, true, true, true, used_names,
                    ));
                };
            });

            result
        }

        // TODO: This many parameters makes this function ugly af
        fn generate_referenced_by_xml(
            table: &Table,
            package: &str,
            database: &Database,
            ref_tables: &mut HashMap<String, i32>,
        ) -> String {
            let mut result = "\n    <!-- Referenced by -->".to_string();

            table.referenced_by().iter().for_each(|r| {
                result.push_str(&generate_relation_xml(
                    r, package, database, false, true, true, ref_tables,
                ));
            });

            result
        }

        // TODO: This many parameters makes this function ugly af
        fn generate_relation_xml(
            relation: &Relation,
            package: &str,
            database: &Database,
            rel_owner: bool,
            insert: bool,
            update: bool,
            used_names: &mut HashMap<String, i32>,
        ) -> String {
            let cols: Vec<&Column> = relation
                .from()
                .iter()
                .map(|c| database.column(c).unwrap())
                .collect();

            let ref_table_name = if rel_owner {
                relation.to()[0].table()
            } else {
                relation.from()[0].table()
            };

            let ref_table_name_count = if let Some(count) = used_names.get_mut(ref_table_name) {
                *count += 1;
                format!("{}{}", ref_table_name, count)
            } else {
                used_names.insert(ref_table_name.to_string(), 1);
                ref_table_name.to_string()
            };

            match relation.r#type() {
                RelationType::OneToOne => {
                    // TODO: Maybe the OneToOne should implement the multicolumn reference.
                    format!(
                        r#"
    <one-to-one name="{}" class="{package}.{}" lazy="proxy" />"#,
                        naming::to_lower_camel_case(&ref_table_name_count),
                        naming::to_upper_camel_case(ref_table_name)
                    )
                }
                RelationType::OneToMany => {
                    format!(
                        r#"
    <set name="{}s" table="{}" lazy="true" fetch="select">
      <key>
        {}
      </key>
      <one-to-many class="{package}.{}" />
    </set>"#,
                        naming::to_lower_camel_case(&ref_table_name_count),
                        ref_table_name_count,
                        generate_multi_column_xml(&cols),
                        naming::to_upper_camel_case(ref_table_name)
                    )
                }
                RelationType::ManyToOne => {
                    let insert_update_str = if !insert && !update {
                        " insert=\"false\" update=\"false\""
                    } else if !insert {
                        " insert=\"false\""
                    } else if !update {
                        " update=\"false\""
                    } else {
                        ""
                    };

                    format!(
                        r#"
    <many-to-one name="{}" class="{package}.{}" {insert_update_str} fetch="select">
      {}
    </many-to-one>"#,
                        naming::to_lower_camel_case(&ref_table_name_count),
                        naming::to_upper_camel_case(ref_table_name),
                        generate_multi_column_xml(&cols)
                    )
                }
                RelationType::ManyToMany => {
                    format!(
                        r#"
    <set name="{}s" table="{}" lazy="true" fetch="select">
      <key>
        {}
      </key>
      <many-to-many class="{package}.{}" />
    </set>
    "#,
                        naming::to_lower_camel_case(&ref_table_name_count),
                        ref_table_name_count,
                        generate_multi_column_xml(&cols),
                        naming::to_upper_camel_case(ref_table_name),
                    )
                }
            }
        }
    }

    fn generate_table_java(&self, table: &Table) -> String {
        let package = &self.package;
        let class_name = naming::to_upper_camel_case(table.name());

        let table_id = table.ids();

        // Generating basic fields based on columns

        let mut fields: Vec<Field> = if table_id.len() == 1 {
            table
                .columns()
                .iter()
                // TODO: If removed, the pk of Developer doesn't get generated
                .filter(|c| *c.key() != KeyType::Foreign)
                .map(generate_field)
                .collect()
        } else {
            let mut fields: Vec<Field> = table
                .columns()
                .iter()
                .filter(|c| !table_id.contains(c))
                .map(generate_field)
                .collect();

            fields.push(Field::new(
                "id".to_string(),
                Type::new(format!("{}Id", class_name), "".to_string()),
                Some(Visibility::Private),
                None,
            ));

            fields
        };

        let mut used_names: HashMap<&String, i32> = HashMap::new();

        gen_rel_fields(
            table.references().iter(),
            true,
            &mut fields,
            &mut used_names,
        );
        gen_rel_fields(
            table.referenced_by().iter(),
            false,
            &mut fields,
            &mut used_names,
        );

        let methods = fields.iter().flat_map(|f| f.getters_setters()).collect();

        let java_class = Class::new(class_name.clone(), package.clone(), fields, methods);

        return java_class.into();

        // TODO: Ugly function again
        fn gen_rel_fields<'a>(
            relations: Iter<'a, Relation>,
            rel_owner: bool,
            fields: &mut Vec<Field>,
            used_name: &mut HashMap<&'a String, i32>,
        ) {
            relations.for_each(|r| {
                let ref_table_name = if rel_owner {
                    r.to()[0].table()
                } else {
                    r.from()[0].table()
                };

                let field_name = if let Some(count) = used_name.get_mut(ref_table_name) {
                    *count += 1;
                    format!("{}{}", naming::to_lower_camel_case(ref_table_name), count)
                } else {
                    used_name.insert(ref_table_name, 1);
                    naming::to_lower_camel_case(ref_table_name).to_string()
                };

                let field_type =
                    Type::new(naming::to_upper_camel_case(ref_table_name), "".to_string());

                let field = gen_rel_field(r.r#type(), field_name, field_type);

                fields.push(field);
            });
        }
    }

    fn generate_composite_id(&self, table: &Table) -> String {
        let package = &self.package;
        let class_name = naming::to_upper_camel_case(table.name());

        let fields: Vec<Field> = table.ids().iter().map(|c| generate_field(c)).collect();

        let methods = fields.iter().flat_map(|f| f.getters_setters()).collect();

        let mut java_class = Class::new(
            format!("{}Id", class_name),
            package.clone(),
            fields,
            methods,
        );

        java_class.add_interface(Interface::new(
            "Serializable".to_string(),
            "java.io".to_string(),
        ));

        java_class.add_equals_method();
        java_class.add_hash_code_method();

        java_class.into()
    }
}
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

fn gen_rel_field(rel_type: &RelationType, field_name: String, field_type: Type) -> Field {
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
