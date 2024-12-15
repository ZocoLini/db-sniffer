use getset::Getters;
use std::cmp::PartialEq;
use std::str::FromStr;

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
            "datetime" | "timestamp" => Ok(ColumnType::DateTime),
            "boolean" | "bool" => Ok(ColumnType::Boolean),
            "blob" => Ok(ColumnType::Blob),
            "decimal" => Ok(ColumnType::Decimal),
            _ => Err(()),
        }
    }
}

#[derive(PartialEq)]
pub enum GenerationType {
    None,
    AutoIncrement,
}

#[derive(PartialEq)]
pub enum KeyType {
    Primary(GenerationType),
    Foreign,
    Unique,
    None,
}

pub struct TableId(Vec<Column>);

#[derive(Getters)]
pub struct Table {
    #[get = "pub"]
    name: String,
    #[get = "pub"]
    columns: Vec<Column>,
}

impl Table {
    pub fn new(name: &str) -> Self {
        Table {
            name: name.to_string(),
            columns: Vec::new(),
        }
    }

    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    pub fn ids(&self) -> Vec<&Column> {
        self.columns
            .iter()
            .filter(|&c| {
                return if let KeyType::Primary(_) = c.key() {
                    true
                } else {
                    false
                };
            })
            .collect()
    }

    pub fn fks(&self) -> Vec<&Column> {
        self.columns
            .iter()
            .filter(|&c| {
                return if let Some(_) = c.reference {
                    true
                } else {
                    false
                };
            })
            .collect()
    }
    
    pub fn column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|c| c.name == name)
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
    /// (table, column)
    reference: Option<(String, String)>,
}

/// References are stored as (table, column)
impl Column {
    pub fn new(
        name: String,
        r#type: ColumnType,
        nullable: bool,
        key: KeyType,
        reference: Option<(String, String)>,
    ) -> Self {
        Column {
            name,
            r#type,
            nullable,
            key,
            reference,
        }
    }
    
    pub fn reference(&self) -> Option<(&String, &String)> {
        self.reference.as_ref().map(|(t, c)| (t, c))
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
    
    pub fn table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == name)
    }
}

pub struct Schema {}

pub struct Metadata {}
