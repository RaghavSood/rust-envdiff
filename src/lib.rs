//! Compare a `.env` file against `.env.example` and report missing or extra keys.

pub mod cli;
pub mod diff;
pub mod parse;
pub mod report;
