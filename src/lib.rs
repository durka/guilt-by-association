//! Macro for declaring/implementing traits with fake associated consts (in stable Rust)
//!
//! Currently very fragile in terms of syntax: does not support traits/impls with _any_ kind of
//! generic parameters (either lifetimes or types).
//!
//! The same macro is used for declaring a trait with associated consts, implementing such a trait,
//! and accessing the consts.
//!
//! The syntax is the same as that proposed for associated consts, _except_ that:
//!
//! - all consts must be at the beginning of the trait/impl, before any functions
//!
//! See the tests for example usage.
//!
//! At the moment they are not consts at all -- they simply expand to static functions with the
//! same name as the declared const. You may therefore access the const by calling
//! `Trait::CONST()`, or (for future proofing, in case the macro implementation changes), call the
//! macro again to access the const, as `guilty!(Trait::CONST)`.

#![cfg_attr(not(test), no_std)]

/// Macro for declaring/implementing traits with fake associated consts
///
/// See the [crate-level documentation](index.html) for more.
#[macro_export]
macro_rules! guilty {
    // These are the user facing invocations:

    // 1. define a private trait
    ($(#[$attr:meta])* trait $traitname:ident $body:tt) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$(#[$attr])*] [trait] [$traitname], $body);
    };
    // 2. define a private trait with inheritance
    ($(#[$attr:meta])* trait $traitname:ident : $parent:ident $body:tt) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$(#[$attr])*] [pub trait] [$traitname : $parent], $body);
    };
    // 3a. define a public trait
    ($(#[$attr:meta])* pub trait $traitname:ident $body:tt) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$(#[$attr])*] [pub trait] [$traitname], $body);
    };
    // 3b. define a public restricted trait
    ($(#[$attr:meta])* pub $restr:tt trait $traitname:ident $body:tt) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$(#[$attr])*] [pub $restr trait] [$traitname], $body);
    };
    // 4a. define a public trait with inheritance
    ($(#[$attr:meta])* pub trait $traitname:ident : $parent:ident $body:tt) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$(#[$attr])*] [pub trait] [$traitname : $parent], $body);
    };
    // 4b. define a public restricted trait with inheritance
    ($(#[$attr:meta])* pub $restr:tt trait $traitname:ident : $parent:ident $body:tt) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$(#[$attr])*] [pub $restr trait] [$traitname : $parent], $body);
    };
    // 5. implement a trait (public or private)
    (impl $traitname:ident for $structname:ident $body:tt) => {
        guilty!(INTERNAL: DEFINE IMPL, $traitname, $structname, $body);
    };
    // 6a. access a const declared with this macro (mentioning trait)
    (<$structname:ident as $traitname:ident> :: $constname:ident) => {
        guilty!(INTERNAL: ACCESS CONST, (<$structname as $traitname>), $constname);
    };
    // 6b. access a const declared with this macro (w/o mentioning trait)
    ($structname:ident :: $constname:ident) => {
        guilty!(INTERNAL: ACCESS CONST, ($structname), $constname);
    };

    // Following are the internal macro calls
    // Since you can't export a macro which calls other unexported macros, guilty! calls itself
    // recursively in order to continue parsing. The invocation syntax for all these recursive
    // calls starts with the tokens `INTERNAL:`.
    //
    // The general strategy for parsing these declarations is we parse one const declaration from
    // the beginning of the trait/impl at a time, turning it into a static function which is
    // appended to the end of the trait/impl. When there are no more consts, the recursion stops
    // and the trait/impl is outputted (with an indirection through AS ITEM to appease the parser).


    // parse-trait-defconst: parse a trait with a const (that has a default value) as the first declaration
    // the square brackets contain [trait Trait] or [pub trait Trait]
    // this calls on to:
    //  - itself if there is another default-valued const
    //  - parse-trait-nodefconst if there is another const with no default value
    //  - def-trait-fn/def-trait-attr/def-trait-ty if there are no more consts
    (INTERNAL: DEFINE TRAIT, [$(#[$attr:meta])*] [$($before:tt)+] [$($traitname:tt)*],
     {
         $(#[$cattr:meta])* const $constname:ident : $consttype:ty = $constdefault:expr;
         $($body:tt)*
     }) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$(#[$attr])*] [$($before)+] [$($traitname)*],
                {
                    $($body)*
                    $(#[$cattr])* #[allow(non_snake_case)] fn $constname() -> $consttype { $constdefault }
                });
    };
    // parse-trait-nodefconst: parse a trait with a const (that has no default value) as the first declaration
    // this calls on to:
    //  - itself is there is another non-default-valued const
    //  - parse-trait-defconst if there is another default-valued const
    //  - def-trait-fn/def-trait-attr/def-trait-ty if there are no more consts
    (INTERNAL: DEFINE TRAIT, [$(#[$attr:meta])*] [$($before:tt)+] [$($traitname:tt)*],
     {
         $(#[$cattr:meta])* const $constname:ident : $consttype:ty;
         $($body:tt)*
     }) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$(#[$attr])*] [$($before)+] [$($traitname)*],
                {
                    $($body)*
                    $(#[$cattr])* #[allow(non_snake_case)] fn $constname() -> $consttype;
                });
    };
    // def-trait-fn: output a trait that has no consts at the beginning (starts with an unadorned fn)
    // indirection through item-redir
    (INTERNAL: DEFINE TRAIT, [$(#[$attr:meta])*] [$($before:tt)+] [$($traitname:tt)*],
     {
         $(#[$fattr:meta])* fn $($body:tt)*
     }) => {
        guilty!(INTERNAL: AS ITEM, $(#[$attr])* $($before)+ $($traitname)* { $(#[$fattr])* fn $($body)* });
    };
    // def-trait-attr: output a trait that has no consts at the beginning (starts with fn that has
    //    docs/attributes)
    // indirection through item-redir
    (INTERNAL: DEFINE TRAIT, [$(#[$attr:meta])*] [$($before:tt)+] [$($traitname:tt)*],
     {
         # $($body:tt)*
     }) => {
        guilty!(INTERNAL: AS ITEM, $(#[$attr])* $($before)+ $($traitname)* { # $($body)* });
    };
    // def-trait-ty: output a trait that has no consts at the beginning (starts with an associated type)
    // indirection through item-redir
    (INTERNAL: DEFINE TRAIT, [$(#[$attr:meta])*] [$($before:tt)+] [$($traitname:tt)*],
     {
         $(#[$tattr:meta])* type $($body:tt)*
     }) => {
        guilty!(INTERNAL: AS ITEM, $(#[$attr])* $($before)+ $($traitname)* { $(#[$tattr])* type $($body)* });
    };
    // def-trait-empty: output a trait that has no items
    (INTERNAL: DEFINE TRAIT, [$(#[$attr:meta])*] [$($before:tt)+] [$($traitname:tt)*],
     {
     }) => {
        guilty!(INTERNAL: AS ITEM, $(#[$attr])* $($before)+ $($traitname)* { });
    };

    // parse-impl-const: parse an impl with a const as the first declaration
    // calls on to:
    //  - itself if there is another const
    //  - def-impl-fn/def-impl-ty if there are no more consts
    (INTERNAL: DEFINE IMPL, $traitname:path, $structname:ident,
     {
         $(#[$cattr:meta])* const $constname:ident : $consttype:ty = $constvalue:expr;
         $($body:tt)*
     }) => {
        guilty!(INTERNAL: DEFINE IMPL, $traitname, $structname,
                {
                    $($body)*
                    $(#[$cattr])* #[allow(non_snake_case)] fn $constname() -> $consttype { $constvalue }
                });
    };
    // def-impl-fn: output an impl that has no consts at the beginning (starts with fn)
    // indirection through item-redir
    (INTERNAL: DEFINE IMPL, $traitname:path, $structname:ident,
     {
         $(#[$fattr:meta])* fn $($body:tt)*
     }) => {
        guilty!(INTERNAL: AS ITEM, impl $traitname for $structname { $(#[$fattr])* fn $($body)* });
    };
    // def-impl-ty: output an impl that has no consts at the beginning (starts with type)
    // indirection through item-redir
    (INTERNAL: DEFINE IMPL, $traitname:path, $structname:ident,
     {
         $(#[$tattr:meta])* type $($body:tt)*
     }) => {
        guilty!(INTERNAL: AS ITEM, impl $traitname for $structname { $(#[$tattr])* type $($body)* });
    };
    // def-impl-empty: output an impl that has no items in it
    (INTERNAL: DEFINE IMPL, $traitname:path, $structname:ident,
     {
     }) => {
        guilty!(INTERNAL: AS ITEM, impl $traitname for $structname { });
    };

    // access: access a const defined with this macro
    // For now, it just calls the function, since we turn consts into functions. In the future, it
    // might do something more clever if the implementation changes.
    (INTERNAL: ACCESS CONST, ($($structname:tt)*), $constname:ident) => {{
        $($structname)* :: $constname ()
    }};

    // item-redir: Item redirection.
    // For some reason the parser sometimes complains "expected item" when you are trying to output
    // a perfectly good item. The solution (sometimes) is to redirect through a macro like this.
    (INTERNAL: AS ITEM, $i:item) => ($i)
}

#[cfg(test)]
mod tests {
    // some small tests
    guilty! { trait Empty { } }
    guilty! { trait JustFn { fn foo(&self); } }
    guilty! { trait JustType { type Foo; } }
    guilty! { pub trait JustConst { const FOO: (); } }
    guilty! { trait DocFn { #[doc="bar"] fn foo(&self); } }
    guilty! { pub(crate) trait DocType { #[doc="bar"] type Foo; } }
    guilty! { trait DocConst { #[doc="bar"] const FOO: (); } }
    struct Foo;
    guilty! { impl Empty for Foo { } }
    guilty! { impl JustFn for Foo { fn foo(&self) {} } }
    guilty! { impl JustType for Foo { type Foo = (); } }
    guilty! { impl JustConst for Foo { const FOO: () = (); } }
    guilty! { impl DocFn for Foo { #[doc="bar"] fn foo(&self) {} } }
    guilty! { impl DocType for Foo { #[doc="bar"] type Foo = (); } }
    guilty! { impl DocConst for Foo { #[doc="bar"] const FOO: () = (); } }

    #[test]
    fn small() {
        assert_eq!(guilty!(<Foo as JustConst>::FOO), ());
        assert_eq!(guilty!(<Foo as DocConst>::FOO), ());
    }


    // bigger integration test

    guilty! {
        /// A trait for things that do stuff
        trait Trait {
            /// An associated const with a default
            const WithDefault: i32 = 0;
            /// An associated const without a default
            const NoDefault: Self;

            /// An associated type
            type Type;

            /// A method with a default impl
            fn with_impl(&self) -> &Self { self }
            /// A method without a default impl
            fn no_impl(&self) -> &Self;
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct Struct { i: i32 }

    guilty! {
        impl Trait for Struct {
            /// An associated const with a default
            const WithDefault: i32 = 42;
            /// An associated const without a default
            const NoDefault: Self = Struct { i: 42 };

            /// An associated type
            type Type = bool;

            /// A method without a default impl
            fn no_impl(&self) -> &Self { self }
        }
    }

    #[test]
    fn big() {
        use std::any::TypeId;

        let s = Struct { i: 42 };
        println!("{}", s.i);
        println!("{} {:?}", guilty!(Struct::WithDefault), guilty!(Struct::NoDefault));

        assert_eq!(s.i,                                     42);
        assert_eq!(s.with_impl(),                           &s);
        assert_eq!(s.no_impl(),                             &s);
        assert_eq!(TypeId::of::<<Struct as Trait>::Type>(), TypeId::of::<bool>());
        assert_eq!(guilty!(Struct::WithDefault),            42);
        assert_eq!(guilty!(Struct::NoDefault),              Struct { i: 42 });
        assert_eq!(guilty!(<Struct as Trait>::WithDefault), 42);
        assert_eq!(guilty!(<Struct as Trait>::NoDefault),   Struct { i: 42 });
    }

}


/*
 * BEFORE
 *
trait Trait {
    const WithDefault: i32 = 0;
    const NoDefault: Self;

    fn with_impl(&self) -> &Self { self }
    fn no_impl(&self) -> &Self;
}

struct Struct { i: i32 }

impl Trait for Struct {
    const WithDefault: i32 = 42;
    const NoDefault: Self = Self { i: 42 };

    fn no_impl(&self) -> &Self { self }
}
*/

/*
 * AFTER
 *
trait Trait {
    fn with_impl(&self) -> &Self { self }
    fn no_impl(&self) -> &Self;

    #[allow(non_snake_case)] fn WithDefault() -> i32 { 0 }
    #[allow(non_snake_case)] fn NoDefault() -> Self;
}

struct Struct { i: i32 }

impl Trait for Struct {
    fn no_impl(&self) -> &Self { self }

    fn WithDefault() -> i32 { 42 }
    fn NoDefault() -> Self { Struct { i: 42 } }
}
*/

