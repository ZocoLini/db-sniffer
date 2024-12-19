use crate::db_objects::{
    Column, ColumnId, ColumnType, Database, GenerationType, KeyType, Relation, RelationType, Table,
};
use crate::sniffers::SniffResults;
#[cfg(test)]
use crate::test_utils;
#[cfg(test)]
use crate::test_utils::mysql::trivial_sniff_results;
use std::cmp::PartialEq;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::{env, fs};

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

        #[cfg(debug_assertions)]
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

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE hibernate-configuration PUBLIC 
"-//Hibernate/Hibernate Configuration DTD 3.0//EN" 
"http://www.hibernate.org/dtd/hibernate-configuration-3.0.dtd">

<hibernate-configuration>
    <session-factory>
        <property name="hibernate.dialect">org.hibernate.dialect.MySQLDialect</property>
        <property name="hibernate.connection.driver_class">com.mysql.cj.jdbc.Driver</property>
        <property name="hibernate.connection.url">jdbc:mysql://{}:{}/{}</property>
        <property name="hibernate.connection.username">{}</property>
        <property name="hibernate.connection.password">{}</property>

        <property name="hibernate.show_sql">true</property>

        <property name="hibernate.format_sql">true</property>

        <property name="hibernate.hbm2ddl.auto">none</property>
   
        <!-- Mapping files -->
        {}
    </session-factory>

</hibernate-configuration>
            "#,
            conn_params.host().clone().unwrap(),
            conn_params.port().unwrap(),
            conn_params.dbname().clone().unwrap(),
            conn_params.user().clone().unwrap(),
            conn_params.password().clone().unwrap(),
            ""
        )
    }

    fn generate_tables_files(&self, tables: &Vec<Table>) {
        for table in tables {
            let table_xml = self.generate_table_xml(table);
            let table_file_path = self
                .target_path
                .join(format!("{}.hbm.xml", to_upper_camel_case(table.name())));

            fs::File::create(&table_file_path).unwrap();

            fs::write(table_file_path, table_xml).unwrap();
        }
    }

    fn generate_table_xml(&self, table: &Table) -> String {
        // TODO: Generate Table class here

        let package = &self.package;
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
            to_upper_camel_case(table.name()),
            table.name(),
            generate_id_xml(table, package),
            generate_properties_xml(table),
            generate_references_to_xml(table, package, self.sniff_results.database()),
            generate_referenced_by_xml(table, package, self.sniff_results.database())
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
                    to_lower_camel_case(id.name()),
                    column_type_to_hibernate_type(id.r#type()),
                    &generate_column_xml(id),
                    gen_class
                ));
            } else {
                // TODO: Generate ID Class here
                result = result.add(&format!(
                    r#"
    <composite-id name="{}" class="{}.{}">"#,
                    "id",
                    package,
                    to_upper_camel_case(&format!("{}Id", table.name())),
                ));

                for id_column in id_columns {
                    result = result.add(&format!(
                        r#"
      <key-property name="{}" type="{}">
        {}
      </key-property>
"#,
                        to_lower_camel_case(id_column.name()),
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

                result = result.add(&format!(
                    r#"
    <property name="{}" type="{}">
      {}
    </property>"#,
                    to_lower_camel_case(column.name()),
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
                column
                    .nullable()
                    .then(|| " not-null=\"true\"")
                    .unwrap_or(""),
                if let KeyType::Unique = column.key() {
                    " unique=\"true\""
                } else {
                    ""
                }
            )
        }

        fn generate_references_to_xml(table: &Table, package: &str, database: &Database) -> String {
            let mut result = "\n    <!-- References -->".to_string();

            let fks = table.references();

            #[cfg(debug_assertions)]
            {
                println!("Found {} columns referenced by {}", fks.len(), table.name());
            }

            for relation in fks {
                result.push_str(&generate_relation_xml(relation, package, database));
            }

            result
        }

        fn generate_referenced_by_xml(table: &Table, package: &str, database: &Database) -> String {
            let mut result = "\n    <!-- Referenced by -->".to_string();

            for relation in table.referenced_by() {
                result.push_str(&generate_relation_xml(relation, package, database));
            }

            result
        }

        fn generate_relation_xml(
            relation: &Relation,
            package: &str,
            database: &Database,
        ) -> String {
            let ref_table_name = relation.to()[0].table();
            let ref_col_name = relation.to()[0].name();

            let col = database.column(&relation.from()[0]).expect("Should exists");

            match relation.r#type() {
                RelationType::OneToOne => {
                    format!(
                        r#"    <one-to-one name="{}" class="{}.{}" lazy="true" />"#,
                        to_lower_camel_case(ref_col_name),
                        package,
                        to_upper_camel_case(ref_col_name)
                    )
                }
                RelationType::OneToMany => {
                    format!(
                        r#"
    <bag name="{}" table="{}" lazy="true" fetch="select">
      <key>
        {}
      </key>
      <one-to-many class="{}.{}" />
    </bag>"#,
                        to_lower_camel_case(ref_table_name),
                        ref_table_name,
                        generate_column_xml(col),
                        package,
                        to_upper_camel_case(ref_table_name)
                    )
                }
                RelationType::ManyToOne => {
                    format!(
                        r#"
    <many-to-one name="{}" class="{}.{}" fetch="select">
      {}
    </many-to-one>"#,
                        to_lower_camel_case(ref_table_name),
                        package,
                        to_upper_camel_case(ref_table_name),
                        generate_column_xml(col)
                    )
                }
                RelationType::ManyToMany | RelationType::Unknown => {
                    format!(
                        r#"
    <bag name="{}s" table="{}" lazy="true" fetch="select">
      <key>
        {}
      </key>
      <many-to-many class="{}.{}" />
    </bag>
    "#,
                        to_lower_camel_case(ref_table_name),
                        ref_table_name,
                        generate_column_xml(col),
                        package,
                        to_upper_camel_case(ref_table_name),
                    )
                }
            }
        }
    }
}

fn column_type_to_hibernate_type(column_type: &ColumnType) -> String {
    match column_type {
        ColumnType::Integer => "int".to_string(),
        ColumnType::Text => "string".to_string(),
        ColumnType::Blob => "binary".to_string(),
        ColumnType::Boolean => "boolean".to_string(),
        ColumnType::Date => "date".to_string(),
        ColumnType::DateTime => "timestamp".to_string(),
        ColumnType::Time => "time".to_string(),
        ColumnType::Float | ColumnType::Double | ColumnType::Decimal => "double".to_string(),
    }
}

fn to_upper_camel_case(s: &str) -> String {
    let mut name = to_lower_camel_case(s).to_string();

    name.replace_range(
        0..1,
        name.chars()
            .next()
            .unwrap()
            .to_uppercase()
            .to_string()
            .as_str(),
    );

    name
}

fn to_lower_camel_case(s: &str) -> String {
    let mut all_upper = true;

    for c in s.chars() {
        if c.is_lowercase() {
            all_upper = false;
            break;
        }
    }

    let mut name = if all_upper {
        s.to_lowercase()
    } else {
        s.to_string()
    };

    let mut i = 0;

    while i < name.len() - 1 {
        let c = name.chars().nth(i).unwrap();

        if c == '_' {
            name.replace_range(
                i..i + 2,
                name.chars()
                    .nth(i + 1)
                    .unwrap()
                    .to_uppercase()
                    .to_string()
                    .as_str(),
            );
        }

        i += 1;
    }

    name = name.replace("_", "");

    name.replace_range(
        0..1,
        name.chars()
            .next()
            .unwrap()
            .to_lowercase()
            .to_string()
            .as_str(),
    );

    name
}

fn get_java_package_name(path: &Path) -> Option<String> {
    let mut package = String::new();
    package = String::new();

    let mut current = path;

    while current.file_name().unwrap().to_str().unwrap() != "src" {
        package = current.file_name().unwrap().to_str().unwrap().to_string() + "." + &package;

        current = current.parent().unwrap();
    }

    package = package.trim_end_matches('.').to_string();

    Some(package)
}

fn get_java_src_root(path: &Path) -> Option<PathBuf> {
    let mut current = path;

    while current.parent().is_some() {
        if current.ends_with("src") {
            return Some(PathBuf::from(current));
        }

        current = current.parent().unwrap();
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::sniffers::DatabaseSniffer;
    use crate::test_utils::mysql::trivial_sniff_results;
    use crate::{sniffers, test_utils};
    use std::env;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_generate_table_xml_trivial() {
        dotenvy::dotenv().ok();

        let sniff_results = trivial_sniff_results();
        let target_path = PathBuf::from("src/com/example/model");

        let generator = XMLGenerator::new(&sniff_results, &target_path).unwrap();

        let generated = generator.generate_table_xml(&sniff_results.database().tables()[0]);
        let expected = r#"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE hibernate-mapping PUBLIC
    "-//Hibernate/Hibernate Mapping DTD 3.0//EN"
    "http://www.hibernate.org/dtd/hibernate-mapping-3.0.dtd">

<hibernate-mapping>
    <class name="com.example.model.Users" table="users">
        <!-- Id -->
        <id name="id" type="int">
            <column name="id" />
            <generator class="assigned"/>
        </id>

        <!-- Properties -->
        <property name="name" type="string">
            <column name="name" />
        </property>
        
        <!-- References -->
        <!-- Referenced by -->
    </class>
</hibernate-mapping>
        "#;

        assert!(test_utils::compare_xml(&generated, expected));
    }

    #[tokio::test]
    async fn test_trivial_generate() {
        dotenvy::dotenv().ok();

        let sniff_results = trivial_sniff_results();
        let target_path =
            PathBuf::from(env::var("TEST_DIR").unwrap()).join("src/com/example/model");

        let generator = XMLGenerator::new(&sniff_results, &target_path).unwrap();

        generator.generate();
    }

    #[tokio::test]
    async fn test_simple_generate() {
        dotenvy::dotenv().ok();

        let sniff_results =
            sniffers::mysql::MySQLSniffer::new(test_utils::mysql::simple_existing_db_conn_params())
                .await
                .unwrap()
                .sniff()
                .await;

        let target_path =
            PathBuf::from(env::var("TEST_DIR").unwrap()).join("src/com/example/model");

        let generator = XMLGenerator::new(&sniff_results, &target_path).unwrap();

        generator.generate();
    }

    #[tokio::test]
    async fn test_to_upper_camel_case() {
        assert_eq!(to_upper_camel_case("users"), "Users");
        assert_eq!(to_upper_camel_case("user_address"), "UserAddress");
        assert_eq!(to_upper_camel_case("USERS_ADDRESS"), "UsersAddress");
        assert_eq!(to_upper_camel_case("UserAddress"), "UserAddress");
        assert_eq!(to_upper_camel_case("UserAddress_"), "UserAddress");
        assert_eq!(to_upper_camel_case("_A"), "A");
        assert_eq!(to_upper_camel_case("_Abc_Def"), "AbcDef");
    }

    #[tokio::test]
    async fn test_to_lower_camel_case() {
        assert_eq!(to_lower_camel_case("users"), "users");
        assert_eq!(to_lower_camel_case("user_address"), "userAddress");
        assert_eq!(to_lower_camel_case("USERS_ADDRESS"), "usersAddress");
        assert_eq!(to_lower_camel_case("UserAddress"), "userAddress");
        assert_eq!(to_lower_camel_case("UserAddress_"), "userAddress");
        assert_eq!(to_lower_camel_case("_A"), "a");
        assert_eq!(to_lower_camel_case("_Abc_Def"), "abcDef");
    }

    #[tokio::test]
    async fn test_get_java_package_name() {
        let path = PathBuf::from("/home/user/projects/my_project/src/com/example/model");
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
    }
}
