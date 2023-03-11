mod wrapper;
mod provider;
mod unsafe_hack;

#[derive(Debug,Clone)]
struct Foo<'a> {
    text: &'a str
}

#[derive(Debug,Clone)]
struct Bar<'a> {
    foo: &'a Foo<'a>
}

impl Drop for Foo<'_> {
    fn drop(&mut self) {
        println!("drop Foo with {:?}", self.text)
    }
}

impl Drop for Bar<'_> {
    fn drop(&mut self) {
        println!("drop Bar with Foo's {:?}", self.foo.text)
    }
}
