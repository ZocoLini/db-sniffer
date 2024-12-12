use std::cmp::PartialEq;
use std::str::FromStr;
use getset::Getters;

pub enum ColumnType {
    Integer,
    Text,
    Float,
    Double,
    Date,
    Time,
    DateTime,
    Boolean,
    Blob,
    Decimal,
}

impl FromStr for ColumnType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "int" | "integer" => Ok(ColumnType::Integer),
            "text" | "char" | "varchar" => Ok(ColumnType::Text),
            "float" => Ok(ColumnType::Float),
            "double" => Ok(ColumnType::Double),
            "date" => Ok(ColumnType::Date),
            "time" => Ok(ColumnType::Time),
            "datetime" => Ok(ColumnType::DateTime),
            "boolean" | "bool" => Ok(ColumnType::Boolean),
            "blob" => Ok(ColumnType::Blob),
            "decimal" => Ok(ColumnType::Decimal),
            _ => Err(()),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum KeyType {
    Primary,
    Foreign,
    Unique,
    None,
}

#[derive(Getters)]
pub struct Table {
    #[get = "pub"]
    name: String,
    #[get = "pub"]
    columns: Vec<Column>,
}

impl Table {
    pub fn new(name: &str,) -> Self {
        Table {
            name: name.to_string(),
            columns: Vec::new(),
        }
    }

    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }
    
    pub fn ids(&self) -> Vec<&Column> {
        self.columns.iter().filter(|c| *c.key() == KeyType::Primary).collect()
    }
}

#[derive(Getters)]
pub struct Column {
    #[get = "pub"]
    name: String,
    #[get = "pub"]
    r#type: ColumnType,
    #[get = "pub"]
    nullable: bool,
    #[get = "pub"]
    key: KeyType,
}

impl Column {
    pub fn new(name: String, r#type: ColumnType, nullable: bool, key: KeyType) -> Self {
        Column {
            name,
            r#type,
            nullable,
            key,
        }
    }
}

#[derive(Getters)]
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
}

pub struct Schema {}

pub struct Metadata {}
