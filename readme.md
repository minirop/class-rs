# class-rs

Reads a .class file into an almost 1-to-1 matching struct or generates a .class file from said structure.\
⚠️ `Constant::Utf8` are using Rust's String type and not the JVM's modified UTF-8. If you have a string that makes that crate panic, open an issue.

## Example

```rust
let mut jvm = JVMClass::new();
let mut file = std::fs::File::open("Test.class").unwrap();
jvm.load(&mut file).unwrap();
let mut fs = std::fs::File::create("Test2.class").unwrap();
jvm.store(&mut fs).unwrap();
```
