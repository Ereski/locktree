use locktree::locktree;

locktree! {
    Main {
        mutex: async Mutex<()>
    }
}

fn main() {}
