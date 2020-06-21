use locktree::locktree;

locktree! {
    Main {
        mutex: Mutex<()>
    }
}

fn main() {
    let mut locks = MainLockTree::new(());
    {
        let _a = locks.lock_mutex();
    }
    let _b = locks.lock_mutex();
}
