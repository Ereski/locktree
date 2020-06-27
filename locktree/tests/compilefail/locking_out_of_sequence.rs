use locktree::locktree;

locktree! {
    Main {
        m0: Mutex<()>,
        m1: Mutex<()>,
        m2: Mutex<()>,
    }
}

fn main() {
    let locks = MainLockTree::new((), (), ());
    let (_a, mut forward_a) = locks.lock_m0();
    let (_b, mut forward_b) = forward_a.lock_m2();
    // Invalid
    let _ = forward_b.lock_m1();
}
