use locktree::locktree;

locktree! {
    Main {
        rw_lock: async RwLock<()>
    }
}

fn main() {}
