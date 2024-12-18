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

// TODO: References should be stored at table level

#[derive(Getters)]
pub struct Table {
    #[get = "pub"]
    name: String,
    #[get = "pub"]
    columns: Vec<Column>,
    /// (this.column, other.column, relation_type)
    #[get = "pub"]
    references: Vec<(Column, ColumnId, Relation)>,
    /// (this.column, other.column, relation_type)
    #[get = "pub"]
    referenced_by: Vec<(Column, ColumnId, Relation)>,
}

impl Table {
    pub fn new(name: &str) -> Self {
        Table {
            name: name.to_string(),
            columns: Vec::new(),
            references: Vec::new(),
            referenced_by: Vec::new(),
        }
    }

    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    pub fn add_reference_to(&mut self, column: Column, other: ColumnId, relation: Relation) {
        self.references.push((column, other, relation));
    }
    
    pub fn add_referenced_by(&mut self, column: Column, other: ColumnId, relation: Relation) {
        self.referenced_by.push((column, other, relation));
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
    
    pub fn column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|c| c.id.name == name)
    }
}

pub enum Relation {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
    Unknown,
}

#[derive(Getters)]
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

#[derive(Getters)]
pub struct Column {
    id: ColumnId,
    #[get = "pub"]
    r#type: ColumnType,
    #[get = "pub"]
    nullable: bool,
    #[get = "pub"]
    key: KeyType,
}

/// References are stored as (table, column)
impl Column {
    pub fn new(
        id: ColumnId,
        r#type: ColumnType,
        nullable: bool,
        key: KeyType,
    ) -> Self {
        Column {
            id,
            r#type,
            nullable,
            key,
        }
    }
    
    pub fn name(&self) -> &str {
        &self.id.name
    }
    
    pub fn table(&self) -> &str {
        &self.id.table
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
    
    pub fn column(&self, column_id: &ColumnId) -> Option<&Column> {
        self.tables
            .iter()
            .find_map(|t| t.column(&column_id.name))
    }
}

pub struct Schema {}

pub struct Metadata {}
