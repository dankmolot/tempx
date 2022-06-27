use rocket::{State};
use rocket::fairing::{AdHoc};
use rocket::tokio::{task};
use rocket::serde::uuid::Uuid;
use std::sync::{Arc, Weak, Mutex, Condvar};
use std::time::{Duration, Instant};
use std::collections::HashMap;

use crate::file_database::{FileDatabaseRef, SharedStoredFile, FileID, StoredFile};

struct SchecludedDelete {
    created: Instant,
    dur: Duration,
    id: FileID,
}

impl SchecludedDelete {
    fn new(dur: Duration, id: Uuid) -> SchecludedDelete {
        SchecludedDelete {
            created: Instant::now(),
            dur,
            id: Arc::new(id),
        }
    }

    fn expired(&self) -> bool {
        self.created.elapsed() >= self.dur
    }
}

pub struct Deleter {
    me: Weak<Mutex<Deleter>>,
    worker: Option<task::JoinHandle<()>>,
    stop: bool,
    stop_cvar: Arc<Condvar>,
    file_db: Option<FileDatabaseRef>,
    schecluded_deletes: HashMap<FileID, SchecludedDelete>
}

pub type DeleterRef = Arc<Mutex<Deleter>>;
pub type DeleterState<'a> = &'a State<DeleterRef>;

impl Deleter {
    fn new() -> DeleterRef {
        Arc::new_cyclic(|me| {
            Mutex::new(Deleter {
                me: me.clone(),
                worker: None,
                stop: false,
                stop_cvar: Arc::new(Condvar::new()),
                file_db: None,
                schecluded_deletes: HashMap::new()
            })
        })
    }

    fn me(&self) -> DeleterRef {
        self.me.upgrade().unwrap()
    }

    fn process_scheclude(self: &mut Self) {
        self.schecluded_deletes.retain(|_, sd| {
            if sd.expired() {
                println!("File {} expired.", &sd.id);

                match self.file_db.as_ref() {
                    Some(file_db) => { file_db.remove_file(&sd.id); },
                    None => (),
                }

                return false;
            }
            
            true
        });
    }

    fn start_worker(self: &mut Self) {
        let me = self.me();

        self.worker = Some(task::spawn_blocking(move || {
            let mut lock = me.lock().unwrap();
            let cvar = lock.stop_cvar.clone();
            
            loop {
                let result = cvar.wait_timeout(lock, Duration::from_secs(60)).unwrap();
                lock = result.0;

                if lock.stop == true {
                    println!("Deleter stopped!");
                    break;
                }

                lock.process_scheclude();
            }
        }));
    }

    pub fn scheclude_delete(&mut self, id: Uuid, dur: Duration) {
        let sd = SchecludedDelete::new(dur, id);

        self.schecluded_deletes.insert(
            sd.id.clone(),
            sd,
        );
    }

    pub fn stage() -> AdHoc {
        AdHoc::on_ignite("Deleter Startup", |rocket| Box::pin(async {
            let deleter = Deleter::new();

            {
                let mut deleter = deleter.lock().unwrap();

                let file_db = rocket.state::<FileDatabaseRef>()
                    .expect("Can't get file database? A big bug here.");
                
                deleter.file_db = Some(file_db.clone());
                deleter.start_worker();
            }

            rocket
                .manage(deleter)
                .attach(AdHoc::on_shutdown("Deleter Shutdown", |rocket| Box::pin(async {
                    let state = rocket.state::<DeleterRef>().unwrap();
                    
                    println!("Shutting down Deleter...");

                    #[allow(unused_assignments)]
                    let mut worker = None;
                    {
                        let mut deleter = state.lock().unwrap();

                        // Notify worker thread about exiting
                        deleter.stop = true;
                        deleter.stop_cvar.notify_all();
    
                        // Take worker join handler, to join it. lol
                        worker = deleter.worker.take();
                    }

 
                    // If join handle still existing, let's join it!
                    match worker {
                        Some(worker) => {
                            worker.await.ok();
                        },
                        None => ()
                    }
                })))
        }))
    }
}
