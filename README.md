# hams
HeAlth Monitoring System

A library written in rust to implement kubernetes lifecycle interfaces. It is written as a shared object so that it can be utilised by many languages.

Typical usages are:
* Rust
* Python
* C/C++
* Java/Kotlin
* Node

# ToDo

List of topics that need work

* [x] Build core library as shared object
* [x] Logging over FFI
* [x] Validate CAPI calls to protect against stupid CAPI errors (null, etc)
* [x] Wrap CAPI interface with rust interface and use of Result<>
* [x] Rust cli program to demonstrate usage
* [ ] Helm sample chart using APIs
* [ ] Python bindings
* [x] NodeJS bindings
* [ ] Java/Kotlin bindings
  * [x] Consideration for future: https://openjdk.org/projects/panama/ and https://github.com/openjdk/jextract
* [x] C/C++ bindings
  * [x] Show usage of C logging from Rust SO
* [ ] Support for prometheus


# Useful Reference
List of useful sites to review
* https://rust-unofficial.github.io/patterns/intro.html
* https://medium.com/dwelo-r-d/wrapping-unsafe-c-libraries-in-rust-d75aeb283c65
*

# Check Link Dependencies
Check the link dependencies for a given binary ie dylib on osx or .so on linux

    otool -L <binary>

# Autotools

Build rust and other libraries and system install with autotools.
