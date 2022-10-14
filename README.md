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

* [ ] Build core library as shared object
* [ ] Validate CAPI calls to protect against stupid CAPI errors (null, etc)
* [ ] Rust cli program to demonstrate usage
* [ ] Helm sample chart referring to APIs
* [ ] Python bindings
* [ ] NodeJS bindings
* [ ] Java/Kotlin bindings
* [ ] C/C++ bindings
* [ ] Optional support for prometheus
