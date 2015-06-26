#[macro_export]
macro_rules! guilty {
    (trait $traitname:ident $body:tt) => {
        guilty!(INTERNAL: DEFINE TRAIT, [trait $traitname], $body);
    };
    (pub trait $traitname:ident $body:tt) => {
        guilty!(INTERNAL: DEFINE TRAIT, [pub trait $traitname], $body);
    };
    (impl $traitname:ident for $structname:ident $body:tt) => {
        guilty!(INTERNAL: DEFINE IMPL, $traitname, $structname, $body);
    };
    ($structname:ident :: $constname:ident) => {
        guilty!(INTERNAL: ACCESS CONST, $structname, $constname);
    };


    (INTERNAL: DEFINE TRAIT, [$($traitname:ident)+],
     {
         const $constname:ident : $consttype:ty = $constdefault:expr,
         $($body:tt)*
     }) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$($traitname)+],
                {
                    $($body)*
                    #[allow(non_snake_case)] fn $constname() -> $consttype { $constdefault }
                });
    };
    (INTERNAL: DEFINE TRAIT, [$($traitname:ident)+],
     {
         const $constname:ident : $consttype:ty,
         $($body:tt)*
     }) => {
        guilty!(INTERNAL: DEFINE TRAIT, [$($traitname)+],
                {
                    $($body)*
                    #[allow(non_snake_case)] fn $constname() -> $consttype;
                });
    };
    (INTERNAL: DEFINE TRAIT, [$($traitname:ident)+],
     {
         fn $($body:tt)*
     }) => {
        guilty!(INTERNAL: AS ITEM, $($traitname)+ { fn $($body)* });
    };
    (INTERNAL: DEFINE TRAIT, [$($traitname:ident)+],
     {
         # $($body:tt)*
     }) => {
        guilty!(INTERNAL: AS ITEM, $($traitname)+ { # $($body)* });
    };

    (INTERNAL: DEFINE IMPL, $traitname:ident, $structname:ident,
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
    (INTERNAL: DEFINE IMPL, $traitname:ident, $structname:ident,
     {
         fn $($body:tt)*
     }) => {
        guilty!(INTERNAL: AS ITEM, impl $traitname for $structname { fn $($body)* });
    };

    (INTERNAL: ACCESS CONST, $structname:ident, $constname:ident) => {{
        $structname :: $constname ()
    }};

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
    #[allow(non_snake_case)] fn WithDefault() -> i32 { 0 }
    #[allow(non_snake_case)] fn NoDefault() -> Self;

    fn with_impl(&self) -> &Self { self }
    fn no_impl(&self) -> &Self;
}

struct Struct { i: i32 }

impl Trait for Struct {
    fn WithDefault() -> i32 { 42 }
    fn NoDefault() -> Self { Struct { i: 42 } }

    fn no_impl(&self) -> &Self { self }
}
*/

