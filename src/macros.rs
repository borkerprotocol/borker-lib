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