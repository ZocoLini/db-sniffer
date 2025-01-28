pub mod hibernate;
mod java;

pub trait Generator {
    fn generate(&self) -> Result<(), crate::Error>;
}