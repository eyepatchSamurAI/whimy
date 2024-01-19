## What to do for a release

1. Create branch `git checkout -b release/0.x.0`
2. Make your changes 
3. Format all changed rust files `cargo fmt` and `cargo clippy`
4. Run rust tests `cargo tarpaulin --exclude-files "*\\mod.rs" --out Html -- --test-threads=1`
5. Run node tests `yarn test`
6. Change the version number of `package.json` and `Cargo.toml`
7. Make sure release CI/CD is passing
8. Check git ci/cd credentials, personal access token does expire
8. Make PR against main
