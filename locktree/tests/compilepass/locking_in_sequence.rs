use locktree::locktree;

locktree! {
    Main {
        m0: Mutex<()>,
        m1: Mutex<()>,
    }
}

fn main() {
    let mut locks = MainLockTree::new((), ());
    let (_a, mut forward) = locks.lock_m0();
    let _b = forward.lock_m1();
}
