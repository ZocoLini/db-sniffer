mod hibernate;
mod sql_ddl;

pub use hibernate::AnnotatedClassGenerator;
pub use hibernate::XMLGenerator;

pub use sql_ddl::SQLGenerator;

pub trait Generator
{
    
}