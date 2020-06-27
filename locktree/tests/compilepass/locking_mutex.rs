use locktree::locktree;

locktree! {
    Main {
        mutex: Mutex<()>,
    }
}

fn main() {
    let locks = MainLockTree::new(());
    let _ = locks.lock_mutex();
}
