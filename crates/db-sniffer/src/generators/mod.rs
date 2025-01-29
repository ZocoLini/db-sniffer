mod hibernate;
mod java;

pub use hibernate::XMLGenerator;
pub use hibernate::JPAGenerator;

pub trait Generator {
    fn generate(&self) -> Result<(), crate::Error>;
}