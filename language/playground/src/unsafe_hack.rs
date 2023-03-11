use std::rc::Rc;
use crate::{Bar, Foo};

struct Wrapper<'ctx> {
    foo: Foo<'ctx>,
    bar: Option<Bar<'ctx>>
}

fn factory<'ctx>() -> Box<Wrapper<'ctx>> {
    let mut wrapper;
    let foo = Rc::new(Foo {text : "unsafe hackish test"});
    wrapper = Box::new(Wrapper { foo: Some(foo.clone()), bar: None});

    let bar = Rc::new(Bar { foo: wrapper.foo.as_ref().unwrap().as_ref()});
    wrapper.bar = Some(bar)
    return wrapper;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_hack_works() {
        let wrapper = factory();
        println!("got bar: {:?}", wrapper.bar)
    }
}
