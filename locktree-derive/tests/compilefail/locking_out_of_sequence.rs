use locktree_derive::locktree;

locktree! {
    Main {
        m0: Mutex<()>,
        m1: Mutex<()>,
    }
}

fn main() {
    let mut locks = MainLockTree::new((), ());
    let (_a, forward) = locks.lock_m1();
    let _b = forward.lock_m0();
}
