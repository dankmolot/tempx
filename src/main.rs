
use std::path::PathBuf;
use file_database::{FileDatabase, FileDatabaseState};
use deleter::{Deleter, DeleterState};
use rocket::fs::TempFile;
use rocket::{get, post, put, routes, catch, Request, catchers, delete};
use rocket::serde::{Serialize};
use rocket::serde::json::{self, Json};
use rocket::serde::uuid::Uuid;
use rocket::http::{Status, ContentType};
use rocket::fs::NamedFile;
use utils::get_file_prefix;
use std::ffi::OsStr;
use app_config::AppConfig;
use ms::Ms;

mod utils;
mod ms;
mod deleter;
mod file_database;
mod app_config;

#[get("/")]
fn index() -> &'static str {
    "
    Usage
        POST /file?[expire=<ms>]
            Uploads file, and returns it id.

            Sample response:
                {
                    \"id\": \"2f327c1e-2764-43b7-8e32-c785072d1f3c\",
                    \"expire_secs\": 60
                }
            
        PUT /file/<id>?[expire=<ms>]
            Same as POST /file. Only difference, there you specifies <id>.
            Also, with this request, you can change existing file.
            Note: only valid hypernated uuid are accepted, any other are rejected.

        GET /file/<id>
            Downloads file. Returns 404 if file not found.
            Also, adds Content-Type same as were uploaded.

        GET /file/<id>.<ext>
            Same as request as ahead, but sets Content-Type same as extension specified in url.
            Example: if <ext> is .json, then Content-Type will be application/json

        DELETE /file/<id>
            Deletes file, if it is exists. Returns nothing.

    What is 'ms'?
        This is string time format, that converts to milliseconds.
        Here example: 5s (5 seconds = 5000 ms), 1m (1 minute = 60000 ms) and etc.
        
        POST /file?expire=5m - Uploads file, that expires after 5 minutes.
        I hope you get it.

        Hours: hours, hour, hrs, hr, h
        Minutes: minutes, minute, mins, min, m
        Seconds: seconds, second, secs, sec, s
        Milliseconds: milliseconds, millisecond, msecs, msec, ms and empty postfix

    "
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct UploadResponse {
    id: Uuid,
    expire_secs: u64,
}

#[post("/file?<expire>", data = "<file>")]
async fn upload(
    file: TempFile<'_>, 
    file_db: FileDatabaseState<'_>, 
    deleter: DeleterState<'_>,
    expire: Option<ms::Ms>,
    config: AppConfig
) -> std::io::Result<Json<UploadResponse>> {
    let id = file_db.add_file(file).await?;

    let expire = config.get_safe_duration(expire);
    deleter
        .lock().unwrap()
        .scheclude_delete(id, expire);

    Ok(Json(UploadResponse {
        id,
        expire_secs: expire.as_secs(),
    }))
}

#[put("/file/<id>?<expire>", data = "<file>")]
async fn upload_with_id(
    id: Uuid, 
    file: TempFile<'_>, 
    file_db: FileDatabaseState<'_>,
    deleter: DeleterState<'_>,
    expire: Option<ms::Ms>,
    config: AppConfig
) -> std::io::Result<Json<UploadResponse>> {
    file_db.add_file_by_id(&id, file).await?;

    let expire = config.get_safe_duration(expire);
    deleter
        .lock().unwrap()
        .scheclude_delete(id, expire);


    Ok(Json(UploadResponse {
        id,
        expire_secs: expire.as_secs(),
    }))
}

async fn get_file_by_id(id: &Uuid, file_db: FileDatabaseState<'_>) -> Option<NamedFile> {
    match file_db.get_file(&id) {
        Some(stored_file) => {
            let file = stored_file.get_named_file().await
                .map_err(|_| Status::NotFound);
   
            file.ok()
        },
        None => None,
    }
}

#[get("/file/<id>")]
async fn download(id: Uuid, file_db: FileDatabaseState<'_>) -> Option<rocket::fs::NamedFile> {
    get_file_by_id(&id, file_db).await
}

#[derive(rocket::Responder)]
enum DownloadResponder {
    OnlyFile(NamedFile),
    CustomType(NamedFile, ContentType),
}

#[get("/file/<path>", rank = 2)]
async fn download_by_path(path: PathBuf, file_db: FileDatabaseState<'_>) -> Option<DownloadResponder> {
    let id = path.file_name()
        .and_then(OsStr::to_str)
        .and_then(get_file_prefix)
        .and_then(|f| Uuid::try_parse(f).ok());

    if id.is_some() {
        let id = id.unwrap();
        let file = get_file_by_id(&id, file_db).await;

        if file.is_some() {
            let file = file.unwrap();
            let content_type = path.extension()
                .and_then(OsStr::to_str)
                .and_then(ContentType::from_extension);

            let responder = match content_type {
                Some(content_type) => 
                    DownloadResponder::CustomType(file, content_type),
                None =>
                    DownloadResponder::OnlyFile(file)
            };

            return Some(responder);
        }
    }
    
    None
}

#[delete("/file/<id>")]
fn delete(id: Uuid, file_db: FileDatabaseState<'_>) {
    file_db.remove_file(&id);
}

#[catch(default)]
fn default(status: rocket::http::Status, _: &Request) -> json::Value {
    let status_code = status.code;
    let reason = status.reason_lossy();

    json::json!({
        "statusCode": status_code,
        "error": reason,
    })
}

#[rocket::launch]
fn rocket() -> _ {
    use rocket::figment::{Figment, Profile, providers::{Format, Toml, Env}};

    let figment = Figment::from(rocket::Config::default())
        .merge(AppConfig::default())
        .merge(Toml::file("App.toml").nested())
        .merge(Env::prefixed("APP_").global())
        .select(Profile::from_env_or("APP_PROFILE", "default"));
 
    rocket::custom(figment) // Using our custom figment configuration.
        .attach(FileDatabase::stage()) // Adding FileDatabase hooks
        .attach(Deleter::stage()) // Adding Deleter hooks
        .mount("/", routes![index, upload, upload_with_id, download, download_by_path, delete]) // Adding routes
        .register("/", catchers![default]) // Registering status to json handler
        .attach(rocket::fairing::AdHoc::on_liftoff("Welcome Message", |rocket| Box::pin(async {
            let app_config: AppConfig = rocket.figment().extract().expect("invalid config");
            let config: rocket::Config = rocket.figment().extract().expect("Invalid conifg");

            let def_exp = Ms::dur_to_string(&app_config.default_expire).unwrap_or_else(|| "NaN".to_string());
            let max_exp = Ms::dur_to_string(&app_config.max_expire).unwrap_or_else(|| "NaN".to_string());

            println!("TempX running at port {}", config.port);
            println!("> Default expire {}, maximum {}", &def_exp, &max_exp);
            println!("> Maximum upload size is {}", &config.limits.get("file").unwrap())
        })))
}
