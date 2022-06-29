<img alt="expo sdk" height="128" align="right" src="./emery.svg">
<h1 align="center">Emery</h1>

## Purpose

Emery is a simple glue layer between Ruby and Rust. Its goal is to allow the creation of Rust based Ruby extensions and allow idomatic code on both sides of the interface.

## Use on Mac

Ruby will only look at certain file extensions for compiled extensions. This is limited to `.bundle` on Mac, but rust will only compile to `.dylib`. To fix you need to relink your created dylibs to create bundles, eg:
```
ld target/debug/examples/libyourlib.dylib -bundle -arch x86_64 -platform_version macos 12.0 12.0 -o yourlib.bundle
```
