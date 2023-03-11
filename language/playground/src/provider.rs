
use crate::*;

struct BarProvider<'a> {
    foo: Foo<'a>
}

impl <'a> BarProvider<'a> {
    fn create_bar(&'a self) -> Bar<'a> {
        return Bar { foo: &self.foo };
    }
}


fn  test(text: &str) -> BarProvider {
    let foo = Foo { text };
    return BarProvider {foo};
}

#[cfg(test)]
mod provider_tests {
    use super::*;

    #[test]
    fn test_it() {
        let bar_provider = test("Test");
        let bar = bar_provider.create_bar();
        let bar_ref: &Bar = &bar;
        println!("{:?}", bar_ref);
    }
}