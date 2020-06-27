use locktree::locktree;

locktree! {
    Main {
        m0: Mutex<()>,
        m1: Mutex<()>,
    }
}

fn main() {
    let locks = MainLockTree::new((), ());
    let (_a, mut forward_a) = locks.lock_m0();
    let _b = forward_a.lock_m1();
    // Invalid
    let _ = forward_a.lock_m1();
}
