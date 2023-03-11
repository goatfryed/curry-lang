
use crate::*;

struct BarWrapper<'a> {
    bar: Bar<'a>,
    foo: Foo<'a>,
}

/*
fn  test(text: &str) -> BarWrapper {
    let foo = Foo { text };
    let bar = Bar { foo: &foo };
    return BarWrapper { bar, foo };
}
*/

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_it() {
        /*
          let mut wrapper = test("Test");
        println!("{:?}", wrapper.bar );
        */
    }
}