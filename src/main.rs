use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use tokio_postgres::NoTls;
mod config;
mod db;
mod dna;
mod errors;
mod seq;
mod service;

mod handler {
    use crate::db;
    use crate::dna;
    use crate::errors::RWError;
    use crate::seq;
    use crate::service;
    use actix_http::Error;
    use actix_web::{get, post, web, HttpResponse, Responder};
    use deadpool_postgres::{Client, Pool};

    #[get("/")]
    pub async fn hello() -> impl Responder {
        HttpResponse::Ok().body("Hello, rna-world!")
    }

    pub async fn rc(req_body: String) -> impl Responder {
        HttpResponse::Ok().body(dna::reverse_complement(&req_body))
    }

    pub async fn edit_distance(path: web::Path<(String, String)>) -> impl Responder {
        let i = path.into_inner();
        let d = seq::levenshtein(&i.0, &i.1);
        HttpResponse::Ok().body(d.to_string())
    }

    #[post("/restriction-enzymes")]
    pub async fn add_restriction_enzyme(
        form: web::Form<dna::RestrictionEnzyme>,
        db_pool: web::Data<Pool>,
    ) -> Result<HttpResponse, Error> {
        let client: Client = db_pool.get().await.map_err(RWError::PoolError)?;

        // unsure how to take ownership of the enzyme from the form
        let new_enzyme = dna::RestrictionEnzyme {
            name: form.name.clone(),
            recognition_sequence: form.recognition_sequence.clone(),
        };

        let _ = db::add_restriction_enzyme(&client, &new_enzyme).await?;

        Ok(HttpResponse::Ok()
            .content_type("text/plain")
            .body(&form.name))
    }

    async fn restriction_enzymes(
        db_pool: web::Data<Pool>,
    ) -> Result<Vec<dna::RestrictionEnzyme>, RWError> {
        let client: Client = db_pool.get().await.map_err(RWError::PoolError)?;
        db::restriction_enzymes(&client).await
    }

    #[get("/restriction-enzymes")]
    pub async fn list_restriction_enzymes(db_pool: web::Data<Pool>) -> impl Responder {
        let enzymes = restriction_enzymes(db_pool).await?;

        service::enzymes_csv(&enzymes)
            .map(|body| HttpResponse::Ok().content_type("text/csv").body(body))
    }

    pub async fn find_restriction_sites(
        db_pool: web::Data<Pool>,
        path: web::Path<String>,
    ) -> impl Responder {
        let enzymes = restriction_enzymes(db_pool).await?;
        let sites = dna::find_restriction_sites(&path, &enzymes);
        service::restriction_sites_csv(&sites[..])
            .map(|body| HttpResponse::Ok().content_type("text/csv").body(body))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    dotenv().ok();

    // load configuration from the environment
    let config = crate::config::Config::from_env().unwrap();

    HttpServer::new(move || {
        App::new()
            .data(config.pg.create_pool(NoTls).unwrap())
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(handler::add_restriction_enzyme)
            .service(handler::list_restriction_enzymes)
            .service(handler::hello)
            .route(
                "/restriction-sites/{s1}",
                web::get().to(handler::find_restriction_sites),
            )
            .route(
                "/edit-distance/{s1}/{s2}",
                web::get().to(handler::edit_distance),
            )
            .route("/rc", web::post().to(handler::rc))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_http::http::Method;
    use actix_web::body::{Body, ResponseBody};
    use actix_web::{test, web, App};

    // see for example:
    // https://stackoverflow.com/a/65867506
    // https://github.com/actix/examples/blob/master/forms/form/src/main.rs#L84
    trait BodyTest {
        fn as_str(&self) -> &str;
    }
    impl BodyTest for ResponseBody<Body> {
        fn as_str(&self) -> &str {
            match self {
                ResponseBody::Body(ref b) => match b {
                    Body::Bytes(ref by) => std::str::from_utf8(&by).unwrap(),
                    _ => panic!(),
                },
                ResponseBody::Other(ref b) => match b {
                    Body::Bytes(ref by) => std::str::from_utf8(&by).unwrap(),
                    _ => panic!(),
                },
            }
        }
    }

    #[actix_rt::test]
    async fn test_rc() {
        let mut app =
            test::init_service(App::new().route("/rc", web::post().to(handler::rc))).await;
        let req = test::TestRequest::with_header("content-type", "text/plain")
            .method(Method::POST)
            .uri("/rc")
            .set_payload("CTTCCTGGA")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
        assert_eq!(resp.response().body().as_str(), "TCCAGGAAG");
    }

    #[actix_rt::test]
    async fn test_edit_distance() {
        let mut app = test::init_service(App::new().route(
            "/edit-distance/{s1}/{s2}",
            web::get().to(handler::edit_distance),
        ))
        .await;
        let req = test::TestRequest::get()
            .uri("/edit-distance/GTGCCC/GTCGGG")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
        assert_eq!(resp.response().body().as_str(), "4");
    }
}
