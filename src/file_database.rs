use dashmap::DashMap;
use rocket::fairing::{AdHoc};
use rocket::serde::uuid::Uuid;
use rocket::State;
use rocket::http::ContentType;
use rocket::fs::TempFile;
use std::sync::{Arc};
use rocket::fs::NamedFile;
use std::io::{Error, ErrorKind};

use crate::utils::uuid4;

pub type FileID = Arc<Uuid>;

#[derive(Debug, Hash)]
pub struct StoredFile {
    pub id: FileID,
    pub path: String,
    pub content_type: Option<ContentType>,
}

impl Drop for StoredFile {
    fn drop(&mut self) {
        println!("StoredFile {} removed from memory", self.path);
        std::fs::remove_file(&self.path).ok();
    }
}

impl StoredFile {
    pub async fn get_named_file(&self) -> std::io::Result<NamedFile> {
        NamedFile::open(&self.path).await
    }
}

pub type SharedStoredFile = Arc<StoredFile>;

pub struct FileDatabase {
    pub files: DashMap<FileID, SharedStoredFile>,
}

pub type FileDatabaseRef = Arc<FileDatabase>;
pub type FileDatabaseState<'a> = &'a State<FileDatabaseRef>;

impl FileDatabase {
    // Creates new instance of FileDatabase
    fn new() -> FileDatabaseRef {
        Arc::new(FileDatabase {
            files: DashMap::new(),
        })
    }

    async fn create_stored_file(&self, id: Uuid, path: String, content_type: Option<ContentType>) -> Result<(), std::io::Error> {
        let stored_file = StoredFile { 
            id: Arc::new(id),
            path, 
            content_type 
        };

        println!("New file now storing at {}", stored_file.path.as_str());

        self.files.insert(stored_file.id.clone(), Arc::new( stored_file ));

        Ok(())
    }

    pub async fn add_file_by_id(&self, id: &Uuid, mut file: TempFile<'_>)  -> std::io::Result<()> {
        if self.file_exists(id) {
            self.remove_file(id);
        }

        let content_type = file.content_type().cloned();

        let mut path = file.path()
            .ok_or_else(|| Error::new(ErrorKind::Other, "Invalid temp file path"))?
            .to_path_buf();

        path.set_file_name(id.as_hyphenated().to_string());

        // Setup file extension
        match content_type.as_ref().and_then(ContentType::extension).map(|s| s.as_str()) {
            Some(ext) => {
                path.set_extension(ext);
            },
            None => (),
        }

        if content_type.is_some() {
            path.set_extension( content_type.as_ref().unwrap().extension().unwrap().to_string() );
        }

        file.persist_to(&path).await?;

        self.create_stored_file(
            *id, 
            path.to_string_lossy().to_string(),
            content_type
        ).await?;

        Ok(())
    }

    pub async fn add_file(&self, file: TempFile<'_>) -> std::io::Result<Uuid> {
        let id = uuid4();
        self.add_file_by_id(&id, file).await?;

        Ok(id)
    }

    pub fn get_file(&self, key: &Uuid) -> Option<SharedStoredFile> {
        self.files.get(key)
            .map(|r| r.value().clone())
    }

    pub fn remove_file(&self, key: &Uuid) -> Option<SharedStoredFile> {
        self.files.remove(key)
            .map(|r| r.1)
    }

    pub fn file_exists(&self, key: &Uuid) -> bool {
        self.files.contains_key(key)
    }

    pub fn stage() -> AdHoc {
        AdHoc::on_ignite("FileDatabase", |rocket| async {
            let file_db = Self::new();

            rocket.manage(file_db)
        })
    }
}

// #[rocket::async_trait]
// impl<'r> FromRequest<'r> for FileDatabaseRef {
//     type Error = ();

//     // Returns cloned arc
//     async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
//         req.guard::<&State<FileDatabaseRef>>().await
//             .map(|v| v.inner().me())
//     }
// }
