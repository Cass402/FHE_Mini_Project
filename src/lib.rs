// This is the library root.
// Declare the modules that make up your library.
// These files (e.g., data_generator.rs, encryption.rs)
// should be in the same directory as this lib.rs file (i.e., in src/).

pub mod computations;
pub mod data_generator;
pub mod encryption;
pub mod visualization;

// You can also re-export specific items if you want to make them easier to access, e.g.:
// pub use data_generator::BiosampleRecord;
