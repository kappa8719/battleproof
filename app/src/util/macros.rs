#[macro_export]
macro_rules! clone {
    () => {};
    ([$($tt:tt)*], $expr:expr) => {{
        clone!($($tt)*);

        $expr
    }};
    ($(,)? mut { $expr:expr } as $ident:ident $($tt:tt)*) => {
        let mut $ident = ::core::clone::Clone::clone(&$expr);
        clone!($($tt)*);
    };
    ($(,)? mut $ident:ident $($tt:tt)*) => {
        let mut $ident = ::core::clone::Clone::clone(&$ident);
        clone!($($tt)*);
    };
    ($(,)? { $expr:expr } as $ident:ident $($tt:tt)*) => {
        let $ident = ::core::clone::Clone::clone(&$expr);
        clone!($($tt)*);
    };
    ($(,)? $ident:ident $($tt:tt)*) => {
        let $ident = ::core::clone::Clone::clone(&$ident);
        clone!($($tt)*);
    };
    ($(,)?) => {};
}
