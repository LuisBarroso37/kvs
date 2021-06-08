use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
/// Response to Get command
pub enum CommandResponse {
  Error(String),
  Value(String),
  Success,
  KeyNotFound
}