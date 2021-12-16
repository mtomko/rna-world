use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;
mod dna;
mod str;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, rna-world!")
}

async fn rc(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(dna::reverse_complement(&req_body))
}

async fn edit_distance(path: web::Path<(String, String)>) -> impl Responder {
    let i = path.into_inner();
    let d = str::levenshtein(&i.0, &i.1);
    HttpResponse::Ok().body(d.to_string())
}

#[post("/restriction-enzymes")]
async fn add_restriction_enzyme(
    data: web::Data<RnaWorldState>,
    form: web::Form<dna::RestrictionEnzyme>,
) -> impl Responder {
    let mut enzymes = data.restriction_enzymes.lock().unwrap();
    // unsure how to take ownership of the enzyme from the form
    let new_enzyme = dna::RestrictionEnzyme {
        name: form.name.clone(),
        recognition_sequence: form.recognition_sequence.clone(),
    };
    enzymes.push(new_enzyme);
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(&form.name)
}

#[derive(Debug, Clone)]
struct CsvError;

fn enzymes_csv(enzymes: &[dna::RestrictionEnzyme]) -> Result<String, CsvError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(&["Name", "Recognition Sequence"])
        .map_err(|_| CsvError)
        .and_then(|_| {
            let r: Result<Vec<_>, _> = enzymes
                .iter()
                .map(|e| {
                    wtr.write_record(&[&e.name, &e.recognition_sequence])
                        .map_err(|_| CsvError)
                })
                .collect();
            r
        })
        .and_then(|_| wtr.into_inner().map_err(|_| CsvError))
        .and_then(|r| String::from_utf8(r).map_err(|_| CsvError))
}

#[get("/restriction-enzymes")]
async fn list_restriction_enzymes(data: web::Data<RnaWorldState>) -> impl Responder {
    let enzymes = data.restriction_enzymes.lock().unwrap();
    match enzymes_csv(&enzymes) {
        Ok(body) => HttpResponse::Ok().content_type("text/csv").body(body),
        Err(_) => HttpResponse::InternalServerError().body("Unable to construct response"),
    }
}

fn restriction_sites_csv(sites: &[(usize, &dna::RestrictionEnzyme)]) -> Result<String, CsvError> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(&["Index", "Name", "Recognition Sequence"])
        .map_err(|_| CsvError)
        .and_then(|_| {
            let r: Result<Vec<_>, _> = sites
                .iter()
                .map(|(i, e)| {
                    wtr.write_record(&[
                        i.to_string(),
                        e.name.clone(),
                        e.recognition_sequence.clone(),
                    ])
                    .map_err(|_| CsvError)
                })
                .collect();
            r
        })
        .and_then(|_| wtr.into_inner().map_err(|_| CsvError))
        .and_then(|r| String::from_utf8(r).map_err(|_| CsvError))
}

async fn find_restriction_sites(
    data: web::Data<RnaWorldState>,
    path: web::Path<String>,
) -> impl Responder {
    let enzymes = data.restriction_enzymes.lock().unwrap();
    let sites = dna::find_restriction_sites(&path, &enzymes);
    match restriction_sites_csv(&sites[..]) {
        Ok(body) => HttpResponse::Ok().content_type("text/csv").body(body),
        Err(_) => HttpResponse::InternalServerError().body("Unable to construct response"),
    }
}

struct RnaWorldState {
    restriction_enzymes: Mutex<Vec<dna::RestrictionEnzyme>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(RnaWorldState {
                restriction_enzymes: Mutex::new(vec![]),
            }))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .service(add_restriction_enzyme)
            .service(list_restriction_enzymes)
            .service(hello)
            .route(
                "/restriction-sites/{s1}",
                web::get().to(find_restriction_sites),
            )
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
