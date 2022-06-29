# Troubleshooting

- Might need to point the linker to the right ruby manually - otherwise you get "Incompatible Library Version" errors. Typically happens when using an IDE with one set of env variables, and a console with a different set.
    ```
    PATH="~/.rvm/rubies/ruby-2.6.3/bin:$PATH" cargo build
    ```
