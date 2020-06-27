use locktree::locktree;
use tokio::sync::Mutex;

locktree! {
    Main {
        mutex: async Mutex(Mutex)<()>,
    }
}

#[tokio::main]
async fn main() {
    let locks = MainLockTree::new(());
    let _ = locks.lock_mutex().0.await;
}
