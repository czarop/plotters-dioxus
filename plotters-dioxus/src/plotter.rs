#![allow(non_snake_case)]

use dioxus::prelude::*;

use plotters_bitmap::BitMapBackend;
use plotters::prelude::*;
use plotters::coord::Shift;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

use image::ImageEncoder;
use image::codecs::png::PngEncoder;

use std::io::Cursor;

pub type DioxusDrawingArea<'a> = DrawingArea<BitMapBackend<'a>, Shift>;

#[derive(Props)]
pub struct PlottersProps<'a, F>
where
    F: for<'b> Fn(DrawingArea<BitMapBackend<'b>, Shift>),
{
    pub size: (u32, u32),
    pub init: F,
    #[props(optional)]
    pub on_click: Option<EventHandler<MouseData>>,
    #[props(optional)]
    pub on_dblclick: Option<EventHandler<MouseData>>,
    #[props(optional)]
    pub on_mousemove: Option<EventHandler<MouseData>>,
    #[props(optional)]
    pub on_mouseout: Option<EventHandler<MouseData>>,
    #[props(optional)]
    pub on_mouseup: Option<EventHandler<MouseData>>,
    #[props(optional)]
    pub on_mousedown: Option<EventHandler<MouseData>>,
    #[props(optional)]
    pub on_mouseover: Option<EventHandler<MouseData>>,
    #[props(optional)]
    pub on_wheel: Option<EventHandler<WheelData>>,
    #[props(default = false)]
    pub draggable: bool,
    #[props(optional)]
    pub on_drag: Option<EventHandler<DragData>>,
    #[props(optional)]
    pub on_dragend: Option<EventHandler<DragData>>,
    #[props(optional)]
    pub on_dragenter: Option<EventHandler<DragData>>,
    #[props(optional)]
    pub on_dragleave: Option<EventHandler<DragData>>,
    #[props(optional)]
    pub on_dragover: Option<EventHandler<DragData>>,
    #[props(optional)]
    pub on_dragstart: Option<EventHandler<DragData>>,
    #[props(optional)]
    pub on_drop: Option<EventHandler<DragData>>,
    #[props(optional)]
    pub on_scroll: Option<EventHandler<ScrollData>>,
}


pub fn Plotters<'a, F: Fn(DioxusDrawingArea)>(cx: Scope<'a, PlottersProps<'a, F>>) -> Element<'a> {
    let buffer_size = ((cx.props.size.1 * cx.props.size.0) as usize) * 3usize;
    let mut buffer = vec![0u8; buffer_size];
    let drawing_area = BitMapBackend::with_buffer(buffer.as_mut_slice(), cx.props.size)
        .into_drawing_area();
    (cx.props.init)(drawing_area);

    let mut data = Vec::new();
    let cursor = Cursor::new(&mut data);
    let encoder = PngEncoder::new(cursor);
    let color = image::ColorType::Rgb8;

    encoder
        .write_image(buffer.as_slice(), cx.props.size.0, cx.props.size.1, color)
        .expect("Failed to write the image");

    let buffer_base64 = BASE64_STANDARD.encode(data);

    render!(rsx! {
        img {
            src: "data:image/png;base64,{buffer_base64}",
            draggable: "{cx.props.draggable}",
            onclick: move |evt| cx.props.on_click.as_ref().map(|cb| cb.call(evt)),
            ondblclick: move |evt| cx.props.on_dblclick.as_ref().map(|cb| cb.call(evt)),
            onmousemove: move |evt| cx.props.on_mousemove.as_ref().map(|cb| cb.call(evt)),
            onmouseout: move |evt| cx.props.on_mouseout.as_ref().map(|cb| cb.call(evt)),
            onmouseover: move |evt| cx.props.on_mouseover.as_ref().map(|cb| cb.call(evt)),
            onmousedown: move |evt| cx.props.on_mousedown.as_ref().map(|cb| cb.call(evt)),
            onmouseup: move |evt| cx.props.on_mouseup.as_ref().map(|cb| cb.call(evt)),
            onwheel: move |evt| cx.props.on_wheel.as_ref().map(|cb| cb.call(evt)),
            ondrag: move |evt| cx.props.on_drag.as_ref().map(|cb| cb.call(evt)),
            ondragend: move |evt| cx.props.on_dragend.as_ref().map(|cb| cb.call(evt)),
            ondragenter: move |evt| cx.props.on_dragenter.as_ref().map(|cb| cb.call(evt)),
            ondragleave: move |evt| cx.props.on_dragleave.as_ref().map(|cb| cb.call(evt)),
            ondragover: move |evt| cx.props.on_dragover.as_ref().map(|cb| cb.call(evt)),
            ondragstart: move |evt| cx.props.on_dragstart.as_ref().map(|cb| cb.call(evt)),
            ondrop: move |evt| cx.props.on_drop.as_ref().map(|cb| cb.call(evt)),
            onscroll: move |evt| cx.props.on_scroll.as_ref().map(|cb| cb.call(evt)),
        }
    })
}

