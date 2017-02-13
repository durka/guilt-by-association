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
//! - const declarations end with a comma, instead of a semicolon (this is due to a limitation of
//! the macro system -- a type followed by a semicolon is for some reason not parseable)
//!
//! See the tests for example usage.
//!
//! At the moment they are not consts at all -- they simply expand to static functions with the
//! same name as the declared const. You may therefore access the const by calling
//! `Trait::CONST()`, or (for future proofing, in case the macro implementation changes), call the
//! macro again to access the const, as `guilty!(Trait::CONST)`.

#![no_std]

/// Macro for declaring/implementing traits with fake associated consts
///
/// See the [crate-level documentation](index.html) for more.
#[macro_export]
macro_rules! guilty {
    // These are the user facing invocations:
    
    // FIXME what if the traits have docs/attributes?
    // 1. define a private trait
    ($(#[$attr:meta])* trait $traitname:ident $body:tt) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$(#[$attr])*] [trait] [$traitname], $body);
    };
    // 2. define a private trait with inheritance
    ($(#[$attr:meta])* trait $traitname:ident : $parent:ident $body:tt) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$(#[$attr])*] [pub trait] [$traitname : $parent], $body);
    };
    // 3. define a public trait
    ($(#[$attr:meta])* pub trait $traitname:ident $body:tt) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$(#[$attr])*] [pub trait] [$traitname], $body);
    };
    // 4. define a public trait with inheritance
    ($(#[$attr:meta])* pub trait $traitname:ident : $parent:ident $body:tt) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$(#[$attr])*] [pub trait] [$traitname : $parent], $body);
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
    // FIXME what if the const has docs/attributes?
    (INTERNAL: DEFINE TRAIT, [$(#[$attr:meta])*] [$($before:ident)+] [$($traitname:tt)*],
     {
         const $constname:ident : $consttype:ty = $constdefault:expr,
         $($body:tt)*
     }) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$(#[$attr])*] [$($before)+] [$($traitname)*],
                {
                    $($body)*
                    #[allow(non_snake_case)] fn $constname() -> $consttype { $constdefault }
                });
    };
    // parse-trait-nodefconst: parse a trait with a const (that has no default value) as the first declaration
    // this calls on to:
    //  - itself is there is another non-default-valued const
    //  - parse-trait-defconst if there is another default-valued const
    //  - def-trait-fn/def-trait-attr/def-trait-ty if there are no more consts
    (INTERNAL: DEFINE TRAIT, [$(#[$attr:meta])*] [$($before:ident)+] [$($traitname:tt)*],
     {
         const $constname:ident : $consttype:ty,
         $($body:tt)*
     }) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$(#[$attr])*] [$($before)+] [$($traitname)*],
                {
                    $($body)*
                    #[allow(non_snake_case)] fn $constname() -> $consttype;
                });
    };
    // def-trait-fn: output a trait that has no consts at the beginning (starts with an unadorned fn)
    // indirection through item-redir
    (INTERNAL: DEFINE TRAIT, [$(#[$attr:meta])*] [$($before:ident)+] [$($traitname:tt)*],
     {
         fn $($body:tt)*
     }) => {
        guilty!(INTERNAL: AS ITEM, $(#[$attr])* $($before)+ $($traitname)* { fn $($body)* });
    };
    // def-trait-attr: output a trait that has no consts at the beginning (starts with fn that has
    //    docs/attributes)
    // indirection through item-redir
    (INTERNAL: DEFINE TRAIT, [$(#[$attr:meta])*] [$($before:ident)+] [$($traitname:tt)*],
     {
         # $($body:tt)*
     }) => {
        guilty!(INTERNAL: AS ITEM, $(#[$attr])* $($before)+ $($traitname)* { # $($body)* });
    };
    // def-trait-ty: output a trait that has no consts at the beginning (starts with an associated type)
    // indirection through item-redir
    (INTERNAL: DEFINE TRAIT, [$(#[$attr:meta])*] [$($before:ident)+] [$($traitname:tt)*],
     {
         type $($body:tt)*
     }) => {
        guilty!(INTERNAL: AS ITEM, $(#[$attr])* $($before)+ $($traitname)* { type $($body)* });
    };

    // parse-impl-const: parse an impl with a const as the first declaration
    // calls on to:
    //  - itself if there is another const
    //  - def-impl-fn/def-impl-ty if there are no more consts
    (INTERNAL: DEFINE IMPL, $traitname:ty, $structname:ident,
     {
         const $constname:ident : $consttype:ty = $constvalue:expr,
         $($body:tt)*
     }) => {
        guilty!(INTERNAL: DEFINE IMPL, $traitname, $structname,
                {
                    $($body)*
                    fn $constname() -> $consttype { $constvalue }
                });
    };
    // def-impl-fn: output an impl that has no consts at the beginning (starts with fn)
    // indirection through item-redir
    (INTERNAL: DEFINE IMPL, $traitname:ty, $structname:ident,
     {
         fn $($body:tt)*
     }) => {
        guilty!(INTERNAL: AS ITEM, impl $traitname for $structname { fn $($body)* });
    };
    // def-impl-ty: output an impl that has no consts at the beginning (starts with type)
    // indirection through item-redir
    (INTERNAL: DEFINE IMPL, $traitname:ty, $structname:ident,
     {
         type $($body:tt)*
     }) => {
        guilty!(INTERNAL: AS ITEM, impl $traitname for $structname { type $($body)* });
    };
    // FIXME: need another DEFINE IMPL that's like def-trait-attr?

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

/*
 * BEFORE
 *
trait Trait {
    const WithDefault: i32 = 0,
    const NoDefault: Self,

    fn with_impl(&self) -> &Self { self }
    fn no_impl(&self) -> &Self;
}

struct Struct { i: i32 }

impl Trait for Struct {
    const WithDefault: i32 = 42,
    const NoDefault: Self = Self { i: 42 },

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

