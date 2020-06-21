use locktree_derive::locktree;

locktree! {
    Main {
        m0: StdMutex<()>,
        m1: StdMutex<()>,
    }
}

fn main() {
    let mut locks = MainLockTree::new((), ());
    let (_a, forward) = locks.lock_m1();
    let _b = forward.lock_m0();
}
