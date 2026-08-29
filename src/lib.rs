pub mod ui;
pub mod views;
pub mod components;
pub mod layout;
pub mod styling;

pub use verdant::prelude::*;
pub use taffy::prelude::*;


#[macro_export]
macro_rules! enclose {
    ( [ $($var:ident),* ] move || $body:expr ) => {
        {
            $( let $var = $var.clone(); )*
            move || $body
        }
    };
}
