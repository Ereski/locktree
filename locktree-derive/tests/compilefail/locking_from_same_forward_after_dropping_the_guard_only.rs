use locktree_derive::locktree;

locktree! {
    Main {
        mutex: StdMutex<()>
    }
}

fn main() {
    let mut locks = MainLockTree::new(());
    let _a = locks.lock_mutex().1;
    // Invalid
    45
    let _b = locks.lock_mutex();
}
