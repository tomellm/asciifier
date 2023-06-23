use cfg_if::cfg_if;

cfg_if! {
if #[cfg(feature = "csr")] {

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use wasm_bindgen::prelude::*;
use image:: {ImageBuffer, Luma};
use ab_glyph::FontRef;

use crate::asciifier::{Asciifier, CharDistributionType};
use crate::basic::{get_image, convert_to_luminance};




#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                </Routes>
            </main>
        </Router>
    }
}



/// Renders the home page of your application.
#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    // Creates a reactive value to update the button

/*
    let path: &str; 
    cfg_if! {
    if #[cfg(feature = "csr")] {
        path = "images/img.png";
    } else {
        path = "./assets/images/img.png";

    }
    };
    let image = get_img(path);
 */
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let val = document.create_element("p").expect("error 1");
    val.set_text_content(Some("Hello from Rust!"));

    body.append_child(&val).expect("error 2");
    view! { cx,
        <div>
            "hello my friends"
        </div>
        //<Canvas/>
    }
}


#[component]
fn Canvas(cx: Scope/*, buffer: ImageBuffer<Luma<u8>, Vec<u8>>*/ ) -> impl IntoView {

    view! { cx,
        //<canvas id="canvas"></canvas>
    } 
}


    

fn get_img(path: &str) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    //let chars: Vec<char> = vec!['a', '7', '#', 'Q', '?', '.', 'm', 'h', '@', '%', '+'];
    let chars = "^°<>|{}≠¿'][¢¶`.,:;-_#'+*?=)(/&%$§qwertzuiopasdfghjklyxcvbnmQWERTZUIOPASDFGHJKLYXCVBNM".chars().collect();
    //let chars = ".-_:,;?=#+*@".chars().collect();
    //let chars = ".,-*/+=%^1234567890".chars().collect();
    let f_height: usize = 128;

    let font = FontRef::try_from_slice(include_bytes!("../fonts/Hasklug-2.otf"))
        .expect("ERROR: The font parsing failed!");

    let asciifier = match Asciifier::new(chars, font, f_height, None) {
        Ok(a) => a,
        Err(err) => {
            println!("There seems to be an error with the asciifier creation: {:?}", err);
            panic!("{:?}", err);
        }
    };

    //let img = rotate90(&get_image("images/2.ektar 100.tif"));
    let img_r = get_image(path.clone()); 
    let img = match img_r {
        Ok(img) => img,
        Err(err) => panic!("There was an error {:?} with the path {:?}", err, path)
    }; 
    let luma_img = convert_to_luminance(&img);


    asciifier.convert(luma_img, CharDistributionType::ExactAdjustedBlacks)
}

}
}