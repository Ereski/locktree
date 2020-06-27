use locktree::locktree;
use tokio::sync::RwLock;

locktree! {
    Main {
        rw_lock: async RwLock(RwLock)<()>,
    }
}

#[tokio::main]
async fn main() {
    let locks = MainLockTree::new(());
    {
        let _ = locks.read_rw_lock().0.await;
    }
    let _ = locks.write_rw_lock().0.await;
}
