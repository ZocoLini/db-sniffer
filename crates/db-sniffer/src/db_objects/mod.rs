use getset::Getters;
use std::str::FromStr;

#[derive(PartialEq, Debug)]
pub enum ColumnType {
    Integer(i32),
    Text(i32),
    Char(i32),
    Varchar(i32),
    Float(i32),
    Double(i32),
    Date,
    Time,
    DateTime,
    Boolean,
    Blob(i32),
    Decimal(i32, i32),
    Numeric(i32),
}

impl FromStr for ColumnType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.replace(" ", "");
        let regex = regex::Regex::new(r"(?P<type_name>[a-z]+)(\((?P<values>[\d,]+)\))?$")
            .expect("invalid regex");

        let Some((type_name, values)) = regex.captures(&s).map(|captures| {
            let type_name = captures
                .name("type_name")
                .expect("type_name not found")
                .as_str();
            let values = captures
                .name("values")
                .map(|v| {
                    v.as_str()
                        .split(',')
                        .map(|s| {
                            s.parse::<i32>()
                                .expect("Has to be a number as the regex says")
                        })
                        .collect::<Vec<i32>>()
                })
                .unwrap_or_default();

            (type_name, values)
        }) else {
            return Err(());
        };

        let first_value = *values.first().unwrap_or(&0);
        let second_value = *values.get(1).unwrap_or(&0);
        
        match type_name {
            "int" | "integer" => Ok(ColumnType::Integer(0)),
            "text" => Ok(ColumnType::Text(0)),
            "char" => Ok(ColumnType::Char(first_value)),
            "varchar" => Ok(ColumnType::Varchar(first_value)),
            "float" => Ok(ColumnType::Float(0)),
            "double" => Ok(ColumnType::Double(0)),
            "date" => Ok(ColumnType::Date),
            "time" => Ok(ColumnType::Time),
            "datetime" | "timestamp" => Ok(ColumnType::DateTime),
            "boolean" | "bool" => Ok(ColumnType::Boolean),
            "blob" => Ok(ColumnType::Blob(0)),
            "decimal" => Ok(ColumnType::Decimal(
                first_value,
                second_value,
            )),
            "numeric" => Ok(ColumnType::Numeric(0)),
            _ => Err(()),
        }
    }
}

impl ColumnType {
    pub fn to_hibernate(&self) -> String {
        match self {
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
            ColumnType::Numeric(_) => "big_decimal".to_string(),
            ColumnType::Decimal(precision, scale) => "big_decimal".to_string(),
        }
    }

    pub fn to_java(&self) -> dotjava::Type {
        match self {
            ColumnType::Integer(_) => dotjava::Type::integer(),
            ColumnType::Text(_) | ColumnType::Varchar(_) => dotjava::Type::string(),
            ColumnType::Blob(_) => dotjava::Type::new("byte[]".to_string(), "".to_string()),
            ColumnType::Boolean => dotjava::Type::boolean(),
            ColumnType::Date | ColumnType::DateTime | ColumnType::Time => {
                dotjava::Type::new("Date".to_string(), "java.util".to_string())
            }
            ColumnType::Double(_) => dotjava::Type::double(),
            ColumnType::Float(_) => dotjava::Type::float(),
            ColumnType::Char(_) => dotjava::Type::character(),
            ColumnType::Numeric(_) => {
                dotjava::Type::new("BigDecimal".to_string(), "java.math".to_string())
            }
            ColumnType::Decimal(precision, scale) => {
                dotjava::Type::new("BigDecimal".to_string(), "java.math".to_string())
            }
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum GenerationType {
    None,
    AutoIncrement,
}

#[derive(PartialEq, Debug)]
pub enum KeyType {
    Primary(GenerationType),
    Unique,
    None,
}

pub struct TableId(Vec<Column>);

#[derive(Getters, PartialEq, Debug)]
pub struct Table {
    #[get = "pub"]
    name: String,
    #[get = "pub"]
    columns: Vec<Column>,
    #[get = "pub"]
    references: Vec<Relation>,
}

impl Table {
    pub fn new(name: &str) -> Self {
        Table {
            name: name.to_string(),
            columns: Vec::new(),
            references: Vec::new(),
        }
    }

    pub fn is_col_fk(&self, column: &str) -> bool {
        self.references
            .iter()
            .any(|r| r.from.iter().any(|c| c.name == column))
    }

    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    pub fn add_reference_to(&mut self, relation: Relation) {
        self.references.push(relation);
    }

    pub fn ids(&self) -> Vec<&Column> {
        self.columns
            .iter()
            .filter(|&c| matches!(c.key(), KeyType::Primary(_)))
            .collect()
    }

    pub fn column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|c| c.id.name == name)
    }
}

#[derive(PartialEq, Debug)]
pub enum RelationType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

impl RelationType {
    pub fn inverse(&self) -> RelationType {
        match self {
            RelationType::OneToOne => RelationType::OneToOne,
            RelationType::OneToMany => RelationType::ManyToOne,
            RelationType::ManyToOne => RelationType::OneToMany,
            RelationType::ManyToMany => RelationType::ManyToMany,
        }
    }
}

#[derive(Getters, PartialEq, Debug)]
pub struct Relation {
    #[get = "pub"]
    from: Vec<ColumnId>,
    #[get = "pub"]
    to: Vec<ColumnId>,
    #[get = "pub"]
    r#type: RelationType,
}

impl Relation {
    pub fn new(from: Vec<ColumnId>, to: Vec<ColumnId>, r#type: RelationType) -> Self {
        if from.len() != to.len() {
            panic!("Invalid relation. |From columns| != |To columns|")
        }

        Relation { from, to, r#type }
    }
}

#[derive(Getters, PartialEq, Clone, Debug)]
pub struct ColumnId {
    #[get = "pub"]
    table: String,
    #[get = "pub"]
    name: String,
}

impl ColumnId {
    pub fn new(table_name: &str, column_name: &str) -> Self {
        ColumnId {
            table: table_name.to_string(),
            name: column_name.to_string(),
        }
    }
}

#[derive(Getters, PartialEq, Debug)]
pub struct Column {
    id: ColumnId,
    #[get = "pub"]
    r#type: ColumnType,
    #[get = "pub"]
    nullable: bool,
    #[get = "pub"]
    key: KeyType,
}

impl Column {
    pub fn new(id: ColumnId, r#type: ColumnType, nullable: bool, key: KeyType) -> Self {
        Column {
            id,
            r#type,
            nullable,
            key,
        }
    }

    pub fn not_nullable(&self) -> bool {
        !self.nullable
    }

    pub fn name(&self) -> &str {
        &self.id.name
    }

    pub fn table(&self) -> &str {
        &self.id.table
    }
}

#[derive(Getters, PartialEq, Debug)]
pub struct Database {
    #[get = "pub"]
    name: String,
    #[get = "pub"]
    tables: Vec<Table>,
}

impl Database {
    pub fn new(db_name: &str) -> Self {
        Database {
            name: db_name.to_string(),
            tables: Vec::new(),
        }
    }
    
    pub fn add_table(&mut self, table: Table) {
        self.tables.push(table);
    }

    pub fn table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == name)
    }

    pub fn column(&self, column_id: &ColumnId) -> Option<&Column> {
        self.tables.iter().find_map(|t| t.column(&column_id.name))
    }

    pub fn table_referenced_by(&self, table_name: &str) -> Vec<&Relation> {
        self.tables
            .iter()
            .flat_map(|t| {
                t.references
                    .iter()
                    .filter(|r| {
                        r.to().first().expect("Relation can not be empty").table == table_name
                    })
                    .collect::<Vec<&Relation>>()
            })
            .collect()
    }

    pub fn table_references_to(&self, table_name: &str) -> Vec<&Relation> {
        self.tables
            .iter()
            .find(|t| t.name == table_name)
            .expect("Table not found. Should not happen")
            .references()
            .iter()
            .collect()
    }
}

#[derive(Getters)]
pub struct Metadata {
    #[getset(get = "pub", set = "pub")]
    dbms: Dbms,
}

impl Metadata {
    pub fn new(dbms: Dbms) -> Self {
        Metadata { dbms }
    }
}

pub enum Dbms {
    MySQL,
    Mssql,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_type_from_str() {
        assert_eq!("int".parse::<ColumnType>(), Ok(ColumnType::Integer(0)));
        assert_eq!("integer".parse::<ColumnType>(), Ok(ColumnType::Integer(0)));
        assert_eq!("text".parse::<ColumnType>(), Ok(ColumnType::Text(0)));
        assert_eq!("char".parse::<ColumnType>(), Ok(ColumnType::Char(0)));
        assert_eq!("varchar".parse::<ColumnType>(), Ok(ColumnType::Varchar(0)));
        assert_eq!("float".parse::<ColumnType>(), Ok(ColumnType::Float(0)));
        assert_eq!("double".parse::<ColumnType>(), Ok(ColumnType::Double(0)));
        assert_eq!("date".parse::<ColumnType>(), Ok(ColumnType::Date));
        assert_eq!("time".parse::<ColumnType>(), Ok(ColumnType::Time));
        assert_eq!("datetime".parse::<ColumnType>(), Ok(ColumnType::DateTime));
        assert_eq!("timestamp".parse::<ColumnType>(), Ok(ColumnType::DateTime));
        assert_eq!("boolean".parse::<ColumnType>(), Ok(ColumnType::Boolean));
        assert_eq!("bool".parse::<ColumnType>(), Ok(ColumnType::Boolean));
        assert_eq!("blob".parse::<ColumnType>(), Ok(ColumnType::Blob(0)));
        assert_eq!(
            "decimal".parse::<ColumnType>(),
            Ok(ColumnType::Decimal(0, 0))
        );
        assert_eq!("numeric".parse::<ColumnType>(), Ok(ColumnType::Numeric(0)));
        assert_eq!("invalid".parse::<ColumnType>(), Err(()));

        assert_eq!("char(3)".parse::<ColumnType>(), Ok(ColumnType::Char(3)));
        assert_eq!("varchar(3)".parse::<ColumnType>(), Ok(ColumnType::Varchar(3)));
        
        assert_eq!(
            "decimal(10)".parse::<ColumnType>(),
            Ok(ColumnType::Decimal(10, 0))
        );
        assert_eq!(
            "decimal(10, 2)".parse::<ColumnType>(),
            Ok(ColumnType::Decimal(10, 2))
        );
    }
}
