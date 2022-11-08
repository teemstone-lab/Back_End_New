mod errors;
mod pgmanager;

use actix_web::{App, HttpServer, web};
use actix_cors::Cors;
use actix_web::http::header;
use dotenv::dotenv;
use tokio_postgres::NoTls;


#[actix_web::main] 
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // let config_ = Config::builder()
    //     .add_source(::config::Environment::default())
    //     .build()
    //     .unwrap();
    let _dbinfo = pgmanager::load_db();
    
    
    let mut config_ = deadpool_postgres::Config::new();
    config_.host = Some(_dbinfo.host.clone());
    config_.port = Some(_dbinfo.port.parse::<u16>().unwrap());
    config_.user = Some(_dbinfo.user.clone());
    config_.password = Some(_dbinfo.password.clone());
    config_.dbname = Some(_dbinfo.dbname.clone());
    

    //let config: ServerConfig = config_.try_deserialize().unwrap();

    let pool = config_.create_pool(None, NoTls).unwrap();

    let server = HttpServer::new(move || {

        let cors = Cors::default()
            .allow_any_origin()
            //.allowed_origin("http://127.0.0.1:5173")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);
            

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .service(web::resource("/getPane/{page_id}").route(web::get().to(pgmanager::get_pane)))
            .service(web::resource("/setPane").route(web::post().to(pgmanager::set_pane_data)))
    })
        .bind("127.0.0.1:8000".clone())?
        .run();
    println!("Server running at http://{}/", "127.0.0.1:8000");

    server.await
}