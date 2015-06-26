#[macro_use] extern crate guilt_by_association;

guilty!{
    trait Trait {
        const WithDefault: i32 = 0;
        const NoDefault: Self;

        fn with_impl(&self) -> &Self { self }
        fn no_impl(&self) -> &Self;
    }
}

#[derive(Debug)]
struct Struct { i: i32 }

guilty!{
    impl Trait for Struct {
        const WithDefault: i32 = 42;
        const NoDefault: Self = Struct { i: 42 };

        fn no_impl(&self) -> &Self { self }
    }
}

#[test]
fn test() {
    let s = Struct { i: 42 };
    println!("{}", s.i);
    println!("{} {:?}", guilty!(Struct::WithDefault), guilty!(Struct::NoDefault));
}

