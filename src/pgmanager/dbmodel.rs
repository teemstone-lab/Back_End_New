use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct SetPaneJsonData {
    pub number: i32,
    pub data: String
}
#[derive(Clone)]
pub struct DBModel {
  pub host: String,
  pub port: String,
  pub user: String,
  pub password: String,
  pub dbname: String
}

#[derive(Serialize)]
pub struct tree_data{
  pub _id: i32,
  pub _status: i32,
  pub _name: String,
  pub _depth: String,
  pub _time: i32
}