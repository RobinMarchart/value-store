use std::collections::HashSet;

// generated by `sqlx migrate build-script`
fn main() {
    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=migrations");
    let env_vars: HashSet<String> = std::env::vars().map(|(name, _)| name).collect();
    if env_vars.contains("CARGO_FEATURE_DB_SQLITE") {
        println!("cargo:rustc-env=DATABASE_URL=sqlite:db.sqlite")
    } else {
        panic!("unknown db configuration")
    }
}
