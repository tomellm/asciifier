use cfg_if::cfg_if;

/*
cfg_if! {
if #[cfg(feature = "csr")] {
 */
use leptos::*;
use leptos::html::{Canvas, Input};
use leptos_meta::*;
use leptos_router::*;

use wasm_bindgen::{prelude::*, Clamped};
use wasm_bindgen_futures::{JsFuture};
use image:: {ImageBuffer, Luma, GenericImageView};
use ab_glyph::FontRef;
use web_sys::{Event, ImageData};
use js_sys::{Array, Int32Array, Uint8Array, Uint8ClampedArray};
use web_sys::console::log;

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
        <Stylesheet id="leptos" href="/pkg/asciifier.css"/>
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
    view! { cx,
        <Asciifier />
    }
}


#[component]
fn Asciifier(cx: Scope) -> impl IntoView {
    let (image, set_image): (ReadSignal<Option<ImageData>>, WriteSignal<Option<ImageData>>) = create_signal(cx, None);
    let (image_status, set_image_status) = create_signal(cx, "Image asciification pending...".to_string());

    let file_input: NodeRef<Input> = create_node_ref(cx);

    let asciify_image = create_action(cx, move |_| async move {        
        let file = (*file_input.get().expect("Could not get file input")).to_owned()
            .dyn_into::<web_sys::HtmlInputElement>().expect("Error casting to input element")
            .files().expect("Could not get files List").get(0);

        match file {
            Some(file) => { 
                set_image_status.set("Found file extracting information now...".to_string());
                let array: Vec<u8> = Uint8Array::new(
                    &JsFuture::from(file.array_buffer()).await.expect("Could not read file!")
                ).to_vec().into_iter().map(|n| n as u8).collect();


                set_image_status.set("Starting the asciification process...".to_string());
                let image_to_write = get_img(array);
                set_image_status.set("Finished the asciification process...".to_string());
                let image_data_length = image_to_write.width() * image_to_write.height() * 4;
                let mut new_image_data = vec![0;image_data_length as usize];
                let act_width = image_to_write.width() * 4;
                new_image_data.iter_mut().enumerate().for_each(|(i, v)| {
                    match (i as u32 % act_width) % 4 {
                        0 | 1 | 2 => *v = image_to_write.get_pixel((i as u32 % act_width) / 4, i as u32 / act_width).0.get(0).expect("Could not get luma").clone(),
                        3 => *v = 255,
                        _ => panic!("there should be no more then 4 choices")
                    }
                });
                set_image_status.set("Finishing final data manipulation steps...".to_string());
                let img_data = ImageData::new_with_u8_clamped_array(Clamped(&new_image_data), image_to_write.width()).unwrap();
                set_image_status.set("Finished! Setting the new image for show...".to_string());
                set_image.set(Some(img_data));
                set_image_status.set("Image asciification pending...".to_string());
                log!("Done!");
            },
            _ => log!("There is no file so there is no action")
        }
    });
    view! { cx,
        <div class="w-screen h-screen flex flex-col">
            <div class="p-3">
                <label for="dropzone-file" class="flex flex-col items-center justify-center w-64 h-32 border-2 border-gray-300 border-dashed rounded-lg cursor-pointer bg-gray-50 dark:hover:bg-bray-800 dark:bg-gray-700 hover:bg-gray-100 dark:border-gray-600 dark:hover:border-gray-500 dark:hover:bg-gray-600">
                    <div class="flex flex-col items-center justify-center pt-5 pb-6">
                        <svg class="w-8 h-8 mb-4 text-gray-500 dark:text-gray-400" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 16">
                            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 13h3a3 3 0 0 0 0-6h-.025A5.56 5.56 0 0 0 16 6.5 5.5 5.5 0 0 0 5.207 5.021C5.137 5.017 5.071 5 5 5a4 4 0 0 0 0 8h2.167M10 15V6m0 0L8 8m2-2 2 2"/>
                        </svg>
                        <p class="mb-2 text-sm text-gray-500 dark:text-gray-400"><span class="font-semibold">"Click to upload"</span>" or drag and drop"</p>
                    </div>
                    <input id="dropzone-file" type="file" class="hidden" node_ref=file_input/>
                </label>
            </div>
            <AsciifiedImage pending=asciify_image.pending() image=image image_status=image_status/>
        </div>
        <AsciifyButton asciify_action=asciify_image />
    } 
}

#[component]
fn AsciifiedImage(
    cx: Scope,
    pending: ReadSignal<bool>,
    image: ReadSignal<Option<ImageData>>,
    image_status: ReadSignal<String>
) -> impl IntoView {
    view!{ cx,
        <div class="p-3 grow flex justify-center items-center">
            { move || match pending.get() {
                true => view!{cx, <div>{image_status}</div>},
                false => view!{cx,
                    <div>
                        {move || match image.get() {
                            None => view!{cx, <div>"There is currently no image."</div>},
                            Some(image) => {
                                view!{cx, <div><CanvasImage image=image/></div>}    
                            }
                        }}
                    </div>
                }
            }}
        </div>
    }
}

#[component]
fn CanvasImage(
    cx: Scope,
    image: ImageData
) -> impl IntoView {
    let image_canvas: NodeRef<Canvas> = create_node_ref(cx);
    image_canvas.on_load(cx, move |image_canvas| {
        let canvas = (*image_canvas).to_owned()
            .dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        let context = canvas.get_context("2d").unwrap().unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
        canvas.set_width(image.width());
        canvas.set_height(image.height());
        let _ = context.put_image_data(&image.clone(), 0f64, 0f64);
    });

    view!{cx,
        <div class="flex">
            <canvas 
                id="canvas"
                class="w-full h-full"
                node_ref=image_canvas
            ></canvas>
        </div>
    }
}

#[component]
fn AsciifyButton(
    cx: Scope,
    asciify_action: Action<(), ()>
) -> impl IntoView {
    view!{cx,
        <div class="absolute top-0 left-0 w-screen h-screen flex justify-center items-end pointer-events-none">
            <button 
                on:click=move |_| {asciify_action.dispatch(());}
                class="p-3 m-3 bg-blue-800 text-white rounded-md pointer-events-auto"
            >"asciify image"</button>
        </div>
    }
}

fn get_img(img: Vec<u8>) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    //let chars: Vec<char> = vec!['a', '7', '#', 'Q', '?', '.', 'm', 'h', '@', '%', '+'];
    let chars = "^°<>|{}≠¿'][¢¶`.,:;-_#'+*?=)(/&%$§qwertzuiopasdfghjklyxcvbnmQWERTZUIOPASDFGHJKLYXCVBNM".chars().collect();
    //let chars = ".-_:,;?=#+*@".chars().collect();
    //let chars = ".,-*/+=%^1234567890".chars().collect();
    let f_height: usize = 12;

    let font = FontRef::try_from_slice(include_bytes!("../fonts/Hasklug-2.otf"))
        .expect("ERROR: The font parsing failed!");
    let asciifier = match Asciifier::new(chars, font, f_height, None) {
        Ok(a) => a,
        Err(err) => {
            panic!("There seems to be an error with the asciifier creation: {:?}", err);
        }
    };
    //let img = rotate90(&get_image("images/2.ektar 100.tif"));
    let img_r = get_image(img.clone());
    let img = match img_r {
        Ok(img) => img,
        Err(err) => panic!("There was an error {:?} with the image {:?}", err, img)
    };
    let luma_img = convert_to_luminance(&img);
    asciifier.convert(luma_img, CharDistributionType::ExactAdjustedBlacks)
}

/*
}
} */
