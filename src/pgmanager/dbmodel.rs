use serde::Deserialize;

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