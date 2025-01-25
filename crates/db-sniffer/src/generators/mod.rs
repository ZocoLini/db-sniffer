pub mod hibernate;

pub trait Generator {
    fn generate(&self) -> Result<(), crate::Error>;
}