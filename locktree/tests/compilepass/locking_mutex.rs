use locktree::locktree;

locktree! {
    Main {
        mutex: Mutex<()>,
    }
}

fn main() {
    let mut locks = MainLockTree::new(());
    let _ = locks.lock_mutex();
}
