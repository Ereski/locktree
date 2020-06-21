use locktree_derive::locktree;

locktree! {
    Main {
        m0: StdMutex<()>,
        m1: StdMutex<()>,
    }
}

fn main() {
    let mut locks = MainLockTree::new((), ());
    let (_a, mut forward) = locks.lock_m0();
    let _b = forward.lock_m1();
}
