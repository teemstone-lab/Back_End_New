use actix_web::{web};
use deadpool_postgres::{Client, Pool};
use std::path::Path;
use ini::Ini;
use crate::{errors::MyError};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

pub mod dbmodel;

lazy_static! {
  static ref CACHE_DATA: Mutex<HashMap<i32, String>>= {
  let mut m = HashMap::new();
  m.insert(0, "".to_string());
  Mutex::new(m)
};
}

static mut DBINFO:dbmodel::DBModel = dbmodel::DBModel{
  host: String::new(),
  port: String::new(),
  user: String::new(),
  password: String::new(),
  dbname: String::new(),
};

const INIT_HOST: &str = "127.0.0.1"; 
const INIT_PORT: &str = "5432";  
const INIT_USER: &str = "ontune";  
const INIT_PASSWORD: &str = "ontune";  
const INIT_DBNAME: &str = "webTest";    

pub fn load_db() -> &'static dbmodel::DBModel{
  if Path::new("setting.ini").exists() {   
    let get_conf = Ini::load_from_file("setting.ini").unwrap();
    let section = get_conf.section(Some("db")).unwrap();
    unsafe{
      DBINFO.host = section.get("host").unwrap().to_string();
      DBINFO.port = section.get("port").unwrap().to_string();
      DBINFO.user = section.get("user").unwrap().to_string();
      DBINFO.password = section.get("password").unwrap().to_string();
      DBINFO.dbname = section.get("dbname").unwrap().to_string();
    }
  } else {
    let mut conf = Ini::new();

    conf.with_section(None::<String>)
    .set("encoding", "utf-8");
    conf.with_section(Some("db".to_owned())).set("host", INIT_HOST).set("port", INIT_PORT).set("user", INIT_USER)
    .set("password", INIT_PASSWORD).set("dbname", INIT_DBNAME);
    conf.write_to_file("setting.ini").unwrap();

    unsafe{
      DBINFO.host = INIT_HOST.to_string();
      DBINFO.port = INIT_PORT.to_string();
      DBINFO.user = INIT_USER.to_string();
      DBINFO.password = INIT_PASSWORD.to_string();
      DBINFO.dbname = INIT_DBNAME.to_string();
    }
  }
  CACHE_DATA.lock().unwrap().clear();
  unsafe{
    &DBINFO
  }
}

pub async fn create_table(db_pool: Pool){
  let client: Client = db_pool.get().await.map_err(MyError::PoolError).unwrap();

  client.batch_execute("
        CREATE TABLE IF NOT EXISTS tbpane (
          _number          Integer PRIMARY KEY,
          _data            text
          )
    ").await.unwrap();
}

pub async fn set_pane_data(pane_data: web::Json<dbmodel::SetPaneJsonData>, db_pool: web::Data<Pool>) -> String {
  let str_panedata: String;
  let str_dbdata: String = get_pane_check(pane_data.number, db_pool.clone()).await;
  
  if str_dbdata.is_empty() {
      str_panedata = set_pane_insert(pane_data, db_pool.clone()).await;
      format!("Insert Data {:?}", str_panedata)
  } else {
      str_panedata = set_pane_update(pane_data, db_pool.clone()).await; 
      format!("Update Data {:?}", str_panedata)
  } 
}

pub async fn set_pane_insert(pane_data: web::Json<dbmodel::SetPaneJsonData>, db_pool: web::Data<Pool>) -> String{

  let _pane_data = pane_data.into_inner();
  println!("&pane_data.number == {}", &_pane_data.number);
  println!("&pane_data.data == {}", &_pane_data.data);
  let client: Client = db_pool.get().await.map_err(MyError::PoolError).unwrap();
  client.execute(
       "INSERT INTO tbpane (_number, _data) VALUES ($1, $2)",
       &[&_pane_data.number, &_pane_data.data],
  ).await.unwrap();
  format!("{:?}", _pane_data)
}

pub async fn set_pane_update(pane_data: web::Json<dbmodel::SetPaneJsonData>, db_pool: web::Data<Pool>) -> String{
  let _pane_data = pane_data.into_inner();
  //cache data update
  if CACHE_DATA.lock().unwrap().len() > 0 {
    if CACHE_DATA.lock().unwrap().contains_key(&_pane_data.number){
      CACHE_DATA.lock().unwrap().remove(&_pane_data.number);
      CACHE_DATA.lock().unwrap().insert(_pane_data.number, _pane_data.data.clone());
    }
  }

  println!("&pane_data.number == {}", &_pane_data.number);
  println!("&pane_data.data == {}", &_pane_data.data);
  let client: Client = db_pool.get().await.map_err(MyError::PoolError).unwrap();
  client.execute(
        "UPDATE tbpane SET _data = $2 WHERE _number = $1",
        &[&_pane_data.number, &_pane_data.data],
  ).await.unwrap();
  format!("{:?}", _pane_data)
}

pub async fn get_pane_check(_pane_id: i32, db_pool: web::Data<Pool>) -> String{
  let mut str_returndata: String;
  str_returndata = "".to_string();
  let client: Client = db_pool.get().await.map_err(MyError::PoolError).unwrap();
  for row in client.query("SELECT _data FROM tbpane where _number=$1", &[&_pane_id]).await.unwrap(){
    str_returndata = row.get("_data");
  }
  str_returndata
}

pub async fn get_pane(_page_id: web::Path<i32>, db_pool: web::Data<Pool>) -> String{
  let mut str_returndata: String;
  str_returndata = "".to_string();
  let _pane_id = _page_id.into_inner();

  //cache Check
  if CACHE_DATA.lock().unwrap().len() > 0 {
    if CACHE_DATA.lock().unwrap().contains_key(&_pane_id){
      let str_cachedata = CACHE_DATA.lock().unwrap().get(&_pane_id).unwrap().clone();
      if !str_cachedata.is_empty(){
        println!("cachedata == {}", str_cachedata);
        return str_cachedata;  
      }
    }
  }

  let client: Client = db_pool.get().await.map_err(MyError::PoolError).unwrap();
  for row in client.query("SELECT _data FROM tbpane where _number=$1", &[&_pane_id]).await.unwrap(){
    str_returndata = row.get("_data");

    if CACHE_DATA.lock().unwrap().len() > 0 {
      CACHE_DATA.lock().unwrap().clear();
      CACHE_DATA.lock().unwrap().insert(_pane_id, str_returndata.clone());
    } else {
      CACHE_DATA.lock().unwrap().insert(_pane_id, str_returndata.clone());
    }
  }
  str_returndata
}

pub async fn get_tree_all_data(db_pool: web::Data<Pool>) ->  String  {
  let mut vec:Vec<dbmodel::TreeData> = Vec::new();

  let client: Client = db_pool.get().await.map_err(MyError::PoolError).unwrap();
  for row in client.query("SELECT * FROM tbtree", &[]).await.unwrap(){
    let _id = row.get("_id");
    let _status = row.get("_status");
    let _name = row.get("_name");
    let _depth = row.get("_depth");
    let _time = row.get("_time");
    vec.push(dbmodel::TreeData{_id: _id, _name: _name, _status: _status, _depth: _depth, _time: _time});
  }

  let json = serde_json::to_string(&vec).unwrap();
  json
}


pub async fn get_tree_data(beforetime: web::Path<i32>,db_pool: web::Data<Pool>) ->  String  {
  let mut vec:Vec<dbmodel::TreeData> = Vec::new();
  let _oldtime = beforetime.into_inner();
  let client: Client = db_pool.get().await.map_err(MyError::PoolError).unwrap();
  for row in client.query("SELECT * FROM tbtree where _time > $1", &[&_oldtime]).await.unwrap(){
    let _id = row.get("_id");
    let _status = row.get("_status");
    let _name = row.get("_name");
    let _depth = row.get("_depth");
    let _time = row.get("_time");
    vec.push(dbmodel::TreeData{_id: _id, _name: _name, _status: _status, _depth: _depth, _time: _time});
  }

  let json = serde_json::to_string(&vec).unwrap();
  json
}