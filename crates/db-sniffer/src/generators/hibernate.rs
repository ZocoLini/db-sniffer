use crate::db_objects::{
    Column, ColumnType, Database, GenerationType, KeyType, Relation, RelationType, Table,
};
use crate::sniffers::SniffResults;
use dotjava::{Class, Field, Type, Visibility};
use std::cmp::PartialEq;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::{env, fs};
use crate::naming;

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

        <property name="hibernate.hbm2ddl.auto">validate</property>
   
        <!-- Mapping files -->
        {mapping_files}
    </session-factory>

</hibernate-configuration>
            "#,
            conn_params.host().as_ref().unwrap(),
            conn_params.port().unwrap(),
            conn_params.dbname().as_ref().unwrap(),
            conn_params.user().as_ref().unwrap(),
            conn_params.password().as_ref().unwrap(),
        )
    }

    fn generate_tables_files(&self, tables: &Vec<Table>) {
        for table in tables {
            let table_xml = self.generate_table_xml(table);
            let table_file_path = self
                .target_path
                .join(format!("{}.hbm.xml", naming::to_upper_camel_case(table.name())));

            fs::File::create(&table_file_path).unwrap();
            fs::write(table_file_path, table_xml).unwrap();

            //

            let table_java = self.generate_table_java(table);
            let table_java_file_path = self
                .target_path
                .join(format!("{}.java", naming::to_upper_camel_case(table.name())));

            fs::File::create(&table_java_file_path).unwrap();
            fs::write(table_java_file_path, table_java).unwrap();

            if table.ids().len() > 1 {
                let composite_id_java = self.generate_composite_id(table);
                let composite_id_java_file_path = self
                    .target_path
                    .join(format!("{}Id.java", naming::to_upper_camel_case(table.name())));

                fs::File::create(&composite_id_java_file_path).unwrap();
                fs::write(composite_id_java_file_path, composite_id_java).unwrap();
            }
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
            naming::to_upper_camel_case(table.name()),
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

                if KeyType::Foreign == *column.key() {
                    continue;
                }

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

            table
                .references()
                .iter()
                .filter(|&r| {
                    return if let KeyType::Primary(_) = table
                        .column(r.from()[0].name())
                        .expect("Should exists")
                        .key()
                    {
                        false
                    } else {
                        true
                    };
                })
                .for_each(|r| {
                    result.push_str(&generate_relation_xml(r, package, database, true));
                });

            result
        }

        fn generate_referenced_by_xml(table: &Table, package: &str, database: &Database) -> String {
            let mut result = "\n    <!-- Referenced by -->".to_string();

            table.referenced_by().iter().for_each(|r| {
                result.push_str(&generate_relation_xml(r, package, database, false));
            });

            result
        }

        fn generate_relation_xml(
            relation: &Relation,
            package: &str,
            database: &Database,
            rel_owner: bool,
        ) -> String {
            let (ref_table_name, col) = if rel_owner {
                (
                    relation.to()[0].table(),
                    database.column(&relation.from()[0]).expect("Should exists"),
                )
            } else {
                (
                    relation.from()[0].table(),
                    database.column(&relation.to()[0]).expect("Should exists"),
                )
            };

            match relation.r#type() {
                RelationType::OneToOne => {
                    format!(
                        r#"
    <one-to-one name="{}" class="{}.{}" lazy="proxy" />"#,
                        naming::to_lower_camel_case(ref_table_name),
                        package,
                        naming::to_upper_camel_case(ref_table_name)
                    )
                }
                RelationType::OneToMany => {
                    format!(
                        r#"
    <bag name="{}s" table="{}" lazy="true" fetch="select">
      <key>
        {}
      </key>
      <one-to-many class="{}.{}" />
    </bag>"#,
                        naming::to_lower_camel_case(ref_table_name),
                        ref_table_name,
                        generate_column_xml(col),
                        package,
                        naming::to_upper_camel_case(ref_table_name)
                    )
                }
                RelationType::ManyToOne => {
                    format!(
                        r#"
    <many-to-one name="{}" class="{}.{}" fetch="select">
      {}
    </many-to-one>"#,
                        naming::to_lower_camel_case(ref_table_name),
                        package,
                        naming::to_upper_camel_case(ref_table_name),
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
                        naming::to_lower_camel_case(ref_table_name),
                        ref_table_name,
                        generate_column_xml(col),
                        package,
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
                .filter(|c| *c.key() != KeyType::Foreign)
                .map(|c| generate_field(c))
                .collect()
        } else {
            let mut fields: Vec<Field> = table
                .columns()
                .iter()
                .filter(|c| !table_id.contains(c))
                .map(|c| generate_field(c))
                .collect();

            fields.push(Field::new(
                "id".to_string(),
                Type::new(format!("{}Id", class_name), "".to_string()),
                Some(Visibility::Private),
                None,
            ));

            fields
        };

        // Adding Fields based on relations
        table.references().iter().for_each(|r| {
            let ref_table_name = r.to()[0].table();

            let field_name = naming::to_lower_camel_case(ref_table_name);
            let field_type = Type::new(naming::to_upper_camel_case(ref_table_name), "".to_string());

            let field = gen_rel_field(r.r#type(), field_name, field_type);

            fields.push(field);
        });

        table.referenced_by().iter().for_each(|r| {
            let ref_table_name = r.from()[0].table();

            let field_name = naming::to_lower_camel_case(ref_table_name);
            let field_type = Type::new(naming::to_upper_camel_case(ref_table_name), package.clone());

            let field = gen_rel_field(r.r#type(), field_name, field_type);

            fields.push(field);
        });

        // Adding setters and getters

        let methods = fields
            .iter()
            .map(|f| f.getters_setters())
            .flatten()
            .collect();

        let java_class = Class::new(class_name.clone(), package.clone(), fields, methods);

        java_class.into()
    }

    fn generate_composite_id(&self, table: &Table) -> String {
        let package = &self.package;
        let class_name = naming::to_upper_camel_case(table.name());

        let fields: Vec<Field> = table.ids().iter().map(|c| generate_field(c)).collect();

        let methods = fields
            .iter()
            .map(|f| f.getters_setters())
            .flatten()
            .collect();

        let java_class = Class::new(
            format!("{}Id", class_name),
            package.clone(),
            fields,
            methods,
        );

        java_class.into()
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

fn column_type_to_java_type(column_type: &ColumnType) -> Type {
    match column_type {
        ColumnType::Integer => Type::integer(),
        ColumnType::Text => Type::string(),
        ColumnType::Blob => Type::new("byte[]".to_string(), "".to_string()),
        ColumnType::Boolean => Type::boolean(),
        ColumnType::Date | ColumnType::DateTime | ColumnType::Time => {
            Type::new("Date".to_string(), "".to_string())
        }
        ColumnType::Float | ColumnType::Double | ColumnType::Decimal => Type::double(),
    }
}

fn get_java_package_name(path: &Path) -> Option<String> {
    let mut package = String::new();
    package = String::new();

    let mut current = path;

    let mut current_file_name = current.file_name().unwrap().to_str().unwrap();
    while current_file_name != "src" && current_file_name != "java" {
        package = current_file_name.to_string() + "." + &package;

        // TODO: If file not found this will, eventually, throw an error
        current = current.parent().unwrap();
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
    let field = match rel_type {
        RelationType::OneToMany | RelationType::ManyToMany | RelationType::Unknown => {
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
    };
    field
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::sniffers::DatabaseSniffer;
    use crate::{sniffers, test_utils, ConnectionParams};
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::{env, path, process};

    #[tokio::test]
    async fn integration_test_simple_generate() {
        dotenvy::dotenv().ok();

        let test_dir = if let Ok(r) = env::var("TEST_DIR") {
            r
        } else {
            panic!("TEST_DIR env var not found")
        };

        if fs::exists(&test_dir).unwrap_or(false) {
            fs::remove_dir_all(&test_dir).expect("Should empty the test dir");
        }

        test_utils::mysql::start_container();

        let sniff_results = if let Ok(r) = sniffers::mysql::MySQLSniffer::new(
            if let Ok(r) =
                ConnectionParams::from_str("mysql://test_user:abc123.@localhost:3306/test_db")
            {
                r
            } else {
                test_utils::mysql::stop_container();
                panic!("Failed to create ConnectionParams")
            },
        )
        .await
        {
            r
        } else {
            test_utils::mysql::stop_container();
            panic!("Failed to create MySQL sniffer");
        }
        .sniff()
        .await;

        let target_path =
            PathBuf::from(format!("{test_dir}/{}", "src/main/java/com/example/model"));

        let generator = if let Some(r) = XMLGenerator::new(&sniff_results, &target_path) {
            r
        } else {
            test_utils::mysql::stop_container();
            panic!("Failed to create XMLGenerator")
        };

        generator.generate();

        // Creating a Maven archetype project
        fs::write(
            format!("{test_dir}/pom.xml"),
            r#"<?xml version="1.0" encoding="UTF-8" ?>

<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>

    <groupId>com.example</groupId>
    <artifactId>mysql-db-sniffer</artifactId>
    <version>0.0.0</version>

    <properties>
        <maven.compiler.source>11</maven.compiler.source>
        <maven.compiler.target>11</maven.compiler.target>
        <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>
    </properties>

    <dependencies>
        <dependency>
            <groupId>junit</groupId>
            <artifactId>junit</artifactId>
            <version>4.13.2</version>
        </dependency>
        <dependency>
            <groupId>mysql</groupId>
            <artifactId>mysql-connector-java</artifactId>
            <version>8.0.33</version>
        </dependency>
        <dependency>
            <groupId>org.hibernate</groupId>
            <artifactId>hibernate-core</artifactId>
            <version>4.3.11.Final</version>
        </dependency>
    </dependencies>

    <build>
        <plugins>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-compiler-plugin</artifactId>
                <version>3.11.0</version>
                <configuration>
                    <source>11</source>
                    <target>11</target>
                </configuration>
            </plugin>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-jar-plugin</artifactId>
                <version>3.2.0</version>
                <configuration>
                    <outputDirectory>${project.basedir}</outputDirectory>
                    <finalName>${project.artifactId}</finalName>
                    <archive>
                        <manifest>
                            <addClasspath>true</addClasspath>
                            <mainClass>com.example.Main</mainClass>
                        </manifest>
                    </archive>
                </configuration>
            </plugin>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-shade-plugin</artifactId>
                <version>3.2.4</version>
                <executions>
                    <execution>
                        <phase>package</phase>
                        <goals>
                            <goal>shade</goal>
                        </goals>
                        <configuration>
                            <createDependencyReducedPom>false</createDependencyReducedPom>
                            <filters>
                                <filter>
                                    <artifact>*:*</artifact>
                                    <excludes>
                                        <exclude>module-info.class</exclude>
                                        <exclude>META-INF/*.SF</exclude>
                                        <exclude>META-INF/*.DSA</exclude>
                                        <exclude>META-INF/*.RSA</exclude>
                                    </excludes>
                                </filter>
                            </filters>
                        </configuration>
                    </execution>
                </executions>
            </plugin>
            <plugin>
                <groupId>org.openjfx</groupId>
                <artifactId>javafx-maven-plugin</artifactId>
                <version>0.0.8</version>
                <configuration>
                    <mainClass>
                        com.example.Main
                    </mainClass>
                </configuration>
            </plugin>
        </plugins>
    </build>

</project>"#
        ).expect("Failed to write to pom.xml");

        fs::write(
            format!("{test_dir}/src/main/java/com/example/Main.java"),
            r#"package com.example;

import com.example.model.Person;
import com.example.model.PersonProject;
import com.example.model.PersonProjectId;
import org.hibernate.Session;
import org.hibernate.SessionFactory;
import org.hibernate.cfg.Configuration;

import static org.junit.Assert.assertEquals;

public class Main
{
    public static void main(String[] args)
    {
        Configuration configuration = new Configuration();
        configuration.configure();

        SessionFactory sessionFactory = configuration.buildSessionFactory();

        Session session = sessionFactory.openSession();

        try
        {
            Person person = (Person) session.get(Person.class, 1);

            assertEquals("John Smith", person.getName());
            assertEquals("Engineering", person.getDepartment().getName());

            System.out.println("Person found and asserted");

            final var id = new PersonProjectId();

            id.setPersonId(1);
            id.setProjectId(1);

            PersonProject personProject = (PersonProject) session.get(PersonProject.class, id);

            assertEquals("John Smith", personProject.getPerson().getName());
            assertEquals("Website Redesign", personProject.getProject().getName());

            System.out.println("PersonProject found and asserted");
        }
        catch (Exception exception)
        {
            exception.printStackTrace();
        } finally
        {
            session.close();
            sessionFactory.close();
        }
    }
}"#,
        )
        .expect("Failed to write to Main.java");

        // Creating and writing the META-INF folder
        fs::create_dir_all(format!("{test_dir}/src/main/resources/META-INF")).unwrap();

        fs::write(
            format!("{test_dir}/src/main/resources/META-INF/MANIFEST.MF"),
            "Manifest-Version: 1.0\nMain-Class: com.example.Main\n",
        )
        .expect("Failed to write to MANIFEST.MF");

        // Move the resources to the resources folder
        fs::create_dir_all(format!("{test_dir}/src/main/resources/com/example/model")).unwrap();
        target_path.read_dir().unwrap().for_each(|entry| {
            let entry = entry.unwrap();
            let file_name = entry.file_name();
            let file_name = file_name.to_str().unwrap();

            if !file_name.ends_with(".hbm.xml") {
                return;
            }

            let target = format!(
                "{test_dir}/src/main/resources/com/example/model/{}",
                file_name
            );

            fs::rename(entry.path(), target).unwrap();
        });

        path::PathBuf::from(format!("{test_dir}/{}", "src/main/java"))
            .read_dir()
            .unwrap()
            .for_each(|entry| {
                let entry = entry.unwrap();
                let file_name = entry.file_name();
                let file_name = file_name.to_str().unwrap();

                if !file_name.ends_with(".cfg.xml") {
                    return;
                }

                let target = format!("{test_dir}/src/main/resources/{}", file_name);

                fs::rename(entry.path(), target).unwrap();
            });

        // Todo: Validate using the generated files and mvn -> mvn package -> java -jar target/mysql-db-sniffer-0.0.0.jar ???
        // Using maven to validate

        let output = process::Command::new("mvn")
            .arg("clean")
            .arg("package")
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .current_dir(test_dir.clone())
            .output()
            .expect("Failed to run mvn package");

        if !output.status.success() {
            panic!("Failed to run mvn package")
        }

        let output = process::Command::new("java")
            .arg("-jar")
            .arg("mysql-db-sniffer.jar")
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .current_dir(test_dir.clone())
            .output()
            .expect("Failed to run java -jar");

        test_utils::mysql::stop_container();

        assert!(output.status.success());
    }

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
