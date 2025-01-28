use getset::Getters;
use std::cmp::PartialEq;
use std::str::FromStr;

#[derive(PartialEq)]
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

// TODO: Add support for scale, precision, lenght properties for types
impl FromStr for ColumnType {
    type Err = ();
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "int" | "integer" => Ok(ColumnType::Integer(0)),
            "text" => Ok(ColumnType::Text(0)),
            "char" => Ok(ColumnType::Char(0)),
            "varchar" => Ok(ColumnType::Varchar(0)),
            "float" => Ok(ColumnType::Float(0)),
            "double" => Ok(ColumnType::Double(0)),
            "date" => Ok(ColumnType::Date),
            "time" => Ok(ColumnType::Time),
            "datetime" | "timestamp" => Ok(ColumnType::DateTime),
            "boolean" | "bool" => Ok(ColumnType::Boolean),
            "blob" => Ok(ColumnType::Blob(0)),
            "decimal" => Ok(ColumnType::Decimal(0, 0)),
            "numeric" => Ok(ColumnType::Numeric(0)),
            _ => Err(()),
        }
    }
}

#[derive(PartialEq)]
pub enum GenerationType {
    None,
    AutoIncrement,
}

// TODO: Remove Foreign
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
    #[get = "pub"]
    references: Vec<Relation>,
    #[get = "pub"]
    referenced_by: Vec<Relation>,
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

    pub fn is_col_fk(&self, column: &str) -> bool {
        self.references.iter().any(|r| r.from.iter().any(|c| c.name == column))
    }
    
    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    pub fn add_reference_to(&mut self, relation: Relation) {
        self.references.push(relation);
    }

    pub fn add_referenced_by(&mut self, relation: Relation) {
        self.referenced_by.push(relation);
    }

    pub fn ids(&self) -> Vec<&Column> {
        self.columns
            .iter()
            .filter(|&c| {
                matches!(c.key(), KeyType::Primary(_))
            })
            .collect()
    }

    pub fn column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|c| c.id.name == name)
    }
}

pub enum RelationType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

#[derive(Getters)]
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

#[derive(Getters, PartialEq)]
#[derive(Clone)]
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

#[derive(Getters, PartialEq)]
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
        self.tables.iter().find_map(|t| t.column(&column_id.name))
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