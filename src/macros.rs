#[macro_export] macro_rules! mask_8 {
    ($x:expr) => {
        2_u8.pow($x) - 1
    };
}

#[macro_export] macro_rules! mask_16 {
    ($x:expr) => {
        2_u16.pow($x) - 1
    };
}

#[macro_export] macro_rules! js_try {
    ($x:expr) => {
        ($x).map_err(|e| format_js_err!("{}", e))?
    };
}

#[macro_export] macro_rules! js_err {
    ($x:expr) => {
        Err(format_js_err!("{}", $x))
    };
}

#[macro_export] macro_rules! format_js_err {
    ($pattern:expr, $($x:expr),*) => {
        js_sys::Error::new(&format!($pattern, $($x,)*))
    };
}

#[macro_export] macro_rules! js_bail {
    ($x:expr) => {
        js_err!($x)?
    };
}