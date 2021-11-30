use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
mod dna;
mod str;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn rc(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(dna::reverse_complement(req_body))
}

async fn edit_distance(path: web::Path<(String, String)>) -> impl Responder {
    let i = path.into_inner();
    let d = str::levenshtein(&i.0, &i.1);
    HttpResponse::Ok().body(d.to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(hello)
            .route("/edit-distance/{s1}/{s2}", web::get().to(edit_distance))
            .route("/rc", web::post().to(rc))
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
        let mut app = test::init_service(App::new().route("/rc", web::post().to(rc))).await;
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
        let mut app = test::init_service(
            App::new().route("/edit-distance/{s1}/{s2}", web::get().to(edit_distance)),
        )
        .await;
        let req = test::TestRequest::get()
            .uri("/edit-distance/GTGCCC/GTCGGG")
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
        assert_eq!(resp.response().body().as_str(), "4");
    }
}
