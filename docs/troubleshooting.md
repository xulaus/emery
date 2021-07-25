# Troubleshooting

- Ruby will only look at certain file extensions for compiled extensions. This is limited to `.bundle` on mac, but rust will only compile to `.dylib`. To fix you might need to recompile ruby with
    ```
    DLEXT2='dylib' rvm reinstall ruby-2.6.3
    ```

- Might need to point the linker to the right ruby manually - otherwise you get "Incompatible Library Version" errors
    ```
    RUSTFLAGS="-C link-args=-L~/.rvm/rubies/ruby-2.6.3/lib" cargo build
    ```
