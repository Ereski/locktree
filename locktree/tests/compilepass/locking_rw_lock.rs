use locktree::locktree;

locktree! {
    Main {
        rw_lock: RwLock<()>,
    }
}

fn main() {
    let locks = MainLockTree::new(());
    let _ = locks.read_rw_lock();
    let _ = locks.write_rw_lock();
}
