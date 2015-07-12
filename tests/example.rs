#[macro_use] extern crate guilt_by_association;

guilty! {
    /// A trait for things that do stuff
    pub trait Trait {
        const WithDefault: i32 = 0,
        const NoDefault: Self,

        type Type;

        fn with_impl(&self) -> &Self { self }
        fn no_impl(&self) -> &Self;
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Struct { i: i32 }

guilty! {
    impl Trait for Struct {
        const WithDefault: i32 = 42,
        const NoDefault: Self = Struct { i: 42 },
        
        type Type = bool;

        fn no_impl(&self) -> &Self { self }
    }
}

#[cfg(test)] use std::any::TypeId;

#[test]
fn test() {
    let s = Struct { i: 42 };
    println!("{}", s.i);
    println!("{} {:?}", guilty!(Struct::WithDefault), guilty!(Struct::NoDefault));

    assert_eq!(s.i,                                     42);
    assert_eq!(s.with_impl(),                           &s);
    assert_eq!(s.no_impl(),                             &s);
    assert_eq!(TypeId::of::<<Struct as Trait>::Type>(), TypeId::of::<bool>());
    assert_eq!(guilty!(Struct::WithDefault),            42);
    assert_eq!(guilty!(Struct::NoDefault),              Struct { i: 42 });
}

