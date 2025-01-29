use dotjava::Type;
use crate::db_objects::ColumnType;

pub fn column_type_to_java_type(column_type: &ColumnType) -> Type {
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