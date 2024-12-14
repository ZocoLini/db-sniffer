use crate::db_objects::{ColumnType, KeyType, Table};
use crate::sniffers::SniffResults;
#[cfg(test)]
use crate::test_utils;
#[cfg(test)]
use crate::test_utils::mysql::trivial_sniff_results;
use std::cmp::PartialEq;
use std::ops::Add;
use std::path::PathBuf;
use std::{env, fs};

pub struct AnnotatedClassGenerator;

impl AnnotatedClassGenerator {
    fn generate_table_files(&self, _sniff_results: SniffResults, _target_path: PathBuf) {
        todo!()
    }
}

pub struct XMLGenerator;

impl XMLGenerator {
    pub fn generate(&self, sniff_results: &SniffResults, target_path: &PathBuf) {
        if !target_path.exists() {
            if let Err(e) = fs::create_dir_all(target_path) {
                println!(
                    "Target path ({})  could not be created: {e}",
                    target_path.to_str().unwrap_or_default()
                );
                return;
            };
        }

        let src_root = get_java_src_root(target_path);
        let package = get_java_package_name(target_path);

        let src_root = if let Some(o) = src_root {
            o
        } else {
            println!("src dir not found as a parent od the output dir");
            return;
        };

        let package = if let Some(o) = package {
            o
        } else {
            println!("The package name couldn't be determined");
            return;
        };

        #[cfg(debug_assertions)]
        {
            println!("Detected package name: {package}");
            println!(
                "Found source root folder: {}",
                src_root.to_str().unwrap_or_default()
            )
        }

        self.generate_tables_files(sniff_results.database().tables(), target_path, &package);

        let conf_xml = self.generate_conf_xml(sniff_results);
        let conf_file_path = src_root.join("hibernate.cfg.xml");

        fs::File::create(&conf_file_path).unwrap();

        fs::write(conf_file_path, conf_xml).unwrap();
    }

    fn generate_conf_xml(&self, sniff_results: &SniffResults) -> String {
        let conn_params = sniff_results.conn_params();

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

    fn generate_tables_files(&self, tables: &Vec<Table>, target_path: &PathBuf, package: &str) {
        for table in tables {
            let table_xml = self.generate_table_xml(table, package);
            let table_file_path =
                target_path.join(format!("{}.hbm.xml", to_upper_camel_case(table.name())));

            fs::File::create(&table_file_path).unwrap();

            fs::write(table_file_path, table_xml).unwrap();
        }
    }

    fn generate_table_xml(&self, table: &Table, package: &str) -> String {
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
    </class>
</hibernate-mapping>
        "#,
            to_upper_camel_case(table.name()),
            table.name(),
            generate_id_xml(table, package),
            generate_properties_xml(table),
            generate_relations_xml(table),
        );

        return xml;

        fn generate_id_xml(table: &Table, package: &str) -> String {
            let id_columns = table.ids();
            let mut result = "<!-- Id -->\n".to_string();

            if id_columns.len() == 0 {
                return result;
            }

            if id_columns.len() == 1 {
                let id = id_columns[0];
                result = result.add(&format!(
                    r#"<id name="{}" column="{}" type="{}">{}"#,
                    to_lower_camel_case(id.name()),
                    id.name(),
                    column_type_to_hibernate_type(id.r#type()),
                    "\n"
                ));
                
                
                
                result = result.add("<generator class=\"native\"/>\n");
                result = result.add("</id>\n");
            } else {
                // TODO: Generate ID Class
                result = result.add(&format!(
                    r#"<composite-id name="{}" class="{}.{}">{}"#,
                    "id",
                    package,
                    to_upper_camel_case(&format!("{}Id", table.name())),
                    "\n"
                ));

                for id_column in id_columns {
                    result = result.add(&format!(
                        r#"<key-property name="{}" column="{}" type="{}"/>{}"#,
                        to_lower_camel_case(id_column.name()),
                        id_column.name(),
                        column_type_to_hibernate_type(id_column.r#type()),
                        "\n"
                    ))
                }

                result = result.add("</composite-id>\n");
            }

            result
        }

        fn generate_properties_xml(table: &Table) -> String {
            let mut result = "<!-- Properties -->\n".to_string();

            for column in table.columns() {
                if *column.key() == KeyType::Primary {
                    continue;
                }

                result = result.add(&format!(
                    r#"<property name="{}" column="{}" type="{}"/>"#,
                    column.name(),
                    column.name(),
                    column_type_to_hibernate_type(column.r#type())
                ));
            }

            result
        }

        fn generate_relations_xml(_table: &Table) -> String {
            let sql = r#"
SELECT REFERENCED_TABLE_NAME
FROM information_schema.REFERENTIAL_CONSTRAINTS
WHERE TABLE_NAME = 'Address'
  AND REFERENCED_TABLE_NAME IS NOT NULL;
            "#;

            let mut result = "<!-- Relations -->".to_string();

            result
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
            .nth(0)
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
            .nth(0)
            .unwrap()
            .to_lowercase()
            .to_string()
            .as_str(),
    );

    name
}

fn get_java_package_name(path: &PathBuf) -> Option<String> {
    let mut package = String::new();
    package = String::new();

    let mut current = path.as_path();

    while current.file_name().unwrap().to_str().unwrap() != "src" {
        package = current.file_name().unwrap().to_str().unwrap().to_string() + "." + &package;

        current = current.parent().unwrap();
    }

    package = package.trim_end_matches('.').to_string();

    Some(package)
}

fn get_java_src_root(path: &PathBuf) -> Option<PathBuf> {
    let mut current = path.clone();

    while current.parent().is_some() {
        if current.ends_with("src") {
            return Some(current);
        }

        current = current.parent().unwrap().to_path_buf();
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

        let generator = XMLGenerator;

        let generated = generator
            .generate_table_xml(&sniff_results.database().tables()[0], "com.example.model");
        let expected = r#"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE hibernate-mapping PUBLIC
    "-//Hibernate/Hibernate Mapping DTD 3.0//EN"
    "http://www.hibernate.org/dtd/hibernate-mapping-3.0.dtd">

<hibernate-mapping>
    <class name="com.example.model.Users" table="users">
        <!-- Id -->
        <id name="id" column="id" type="int">
            <generator class="native"/>
        </id>

        <!-- Properties -->
        <property name="name" column="name" type="string"/>
        
        <!-- Relations -->
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

        let generator = XMLGenerator;

        generator.generate(&sniff_results, &target_path);
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

        let generator = XMLGenerator;

        generator.generate(&sniff_results, &target_path);
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
