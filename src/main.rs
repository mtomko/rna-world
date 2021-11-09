use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

mod dna {

    fn complement_dna(dna: String) -> String {
        let mut comp = String::new();
        for b in dna.chars() {
            let bc = match b {
                'A' => 'T',
                'a' => 'T',
                'T' => 'A',
                't' => 'A',
                'C' => 'G',
                'c' => 'G',
                'G' => 'C',
                'g' => 'G',
                _ => 'N',
            };
            comp.push(bc);
        }
        comp
    }

    fn reverse(s: String) -> String {
        s.chars().rev().collect::<String>()
    }

    pub fn reverse_complement(dna: String) -> String {
        reverse(complement_dna(dna))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn reverse_complement_test() {
            assert_eq!(reverse_complement("CATAGGTTG".to_string()), "CAACCTATG");
        }
    }
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

async fn rc(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(dna::reverse_complement(req_body))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello).route("/rc", web::post().to(rc)))
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
}
