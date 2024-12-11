use lazy_static::lazy_static;
use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{
        watch::{self, Receiver},
        Notify,
    },
    task::JoinHandle,
};
use tracing::{error, info};

#[derive(Clone, Debug)]
struct Resource {
    value: i32,
}

struct UpdateManager {
    rx: Receiver<Option<Resource>>,
    stop_notify: Arc<Notify>,
    _task: JoinHandle<()>,
}

impl UpdateManager {
    fn new() -> UpdateManager {
        let (tx, rx) = watch::channel(None);
        let stop_notify = Arc::new(Notify::new());
        let stop_wait = stop_notify.clone();

        let _task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(2)) => {
                        match Self::update_resource().await {
                            Ok(new_resource) => {
                                if tx.send(Some(new_resource)).is_err() {
                                    break;
                                }
                            }
                            Err(e) => error!("Failed to update resource: {e}"),
                        }
                    },
                    _ = stop_wait.notified() => {
                        break;
                    }
                }
            }
            info!("UpdateManager: update task existed");
        });

        UpdateManager {
            rx,
            stop_notify,
            _task,
        }
    }

    async fn update_resource() -> anyhow::Result<Resource> {
        // update that takes 1 second to complete
        tokio::time::sleep(Duration::from_secs(1)).await;
        println!("Resource updated");
        Ok(Resource { value: 42 })
    }

    async fn get_resource(&self) -> anyhow::Result<Resource> {
        let mut rx = self.rx.clone();
        let resource_ref = rx.wait_for(|resource| resource.is_some()).await?;
        let res = resource_ref.as_ref().unwrap();
        Ok(res.clone())
    }
}

impl Drop for UpdateManager {
    fn drop(&mut self) {
        self.stop_notify.notify_waiters();
    }
}

#[tokio::main]
async fn main() {
    lazy_static! {
        static ref UPDATE_MANAGER: UpdateManager = UpdateManager::new();
    }

    println!("{:?}", UPDATE_MANAGER.get_resource().await.unwrap());
    println!("Sleeping 7 seconds...");
    tokio::time::sleep(Duration::from_secs(7)).await;
    println!("{:?}", UPDATE_MANAGER.get_resource().await.unwrap());
    println!("{:?}", UPDATE_MANAGER.get_resource().await.unwrap());
}
