use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use code::{CodeRequest, CodeResponse};
use lang::Lang;
use std::{env::temp_dir, fs::create_dir_all, path::PathBuf};

mod code;
mod lang;

lazy_static::lazy_static! {
    pub static ref TMPDIR: PathBuf = {
        let mut dir = temp_dir();
        dir.push("code_runner");
        dir
    };
}

#[post("/api")]
async fn run_code(code: web::Json<CodeRequest>) -> impl Responder {
    let lang = match code.language.as_str() {
        "Go" => Lang::Go,
        "Rust" => Lang::Rust,
        "C" => Lang::C,
        "C++" => Lang::Cpp,
        "Python2" => Lang::Python2,
        "Python3" => Lang::Python3,
        _ => return HttpResponse::BadRequest().body("Invaild language!"),
    };
    if code.time_limit == 0 || code.time_limit > 10 {
        return HttpResponse::BadRequest().body("Time limit must be between 1..10 (seconds)!");
    }
    let (status, message) = lang.execute_code(&code.source_code, &code.input, code.time_limit);
    HttpResponse::Ok().json(CodeResponse {
        status: format!("{:?}", status),
        message,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    create_dir_all(format!("{}", TMPDIR.to_string_lossy())).unwrap();
    HttpServer::new(|| App::new().service(run_code))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
