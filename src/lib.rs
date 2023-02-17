#![feature(drain_filter)]

use leptos::*;
use serde::{Serialize, Deserialize};
use web_sys::HtmlCanvasElement;

#[derive(Debug, Clone)]
pub struct PointXYData {
    x: RwSignal<f64>,
    y: RwSignal<f64>,
}

#[derive(Debug, Clone)]
pub enum PointData {
    Xy(PointXYData),
    Ref(RwSignal<Option<StepOption>>),
}

impl PointData {
    fn resolve(&self, drawables: &Vec<Drawable>) -> (f64, f64) {
        match self {
            PointData::Xy(PointXYData { x, y }) => (x(), y()),
            PointData::Ref(r) => {
                match r.get() {
                    None => (0.0, 0.0),
                    Some(r) => {
                        let found = drawables.get(r.drawable_id);
                        match found {
                            Some(drawable) => {
                                match drawable.drawable.snap_points().get(r.snap_point) {
                                    Some(point) => (point.x, point.y),
                                    _ => (0.0, 0.0)
                                }
                            },
                            _ => (0.0, 0.0),
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum StepData {
    DrawPoint(RwSignal<PointData>),
    DrawLine(RwSignal<PointData>, RwSignal<PointData>),
    // Translate(RwSignal<usize>),
}

impl StepData {
    pub fn describe(&self) -> String {
        match self {
            Self::DrawPoint(_) => String::from("DrawPoint"),
            Self::DrawLine(_, _) => String::from("DrawLine"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Step {
    id: usize,
    step: StepData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepOption {
    drawable_id: usize,
    snap_point: usize,
    name: String
}

impl IntoAttribute for StepOption {
    fn into_attribute(self, _cx: Scope) -> Attribute {
        Attribute::String(serde_json::to_string(&self).unwrap())
    }
}

pub fn options_for(cx: Scope, id: usize) -> Vec<StepOption> {
    let mut opts = Vec::new();

    let steps = use_context::<ReadSignal<Vec<Step>>>(cx).unwrap().get();
    let drawables = use_context::<Signal<Vec<Drawable>>>(cx).unwrap().get();

    let mut step_ids = Vec::new();
    for step in steps.iter() {
        if step.id == id {
            break;
        }
        else {
            step_ids.push(step.id);
        }
    }

    for (drawable_id, drawable) in drawables.iter().enumerate() {
        if step_ids.contains(&drawable.step_id) {
            for sp_id in 0..drawable.drawable.snap_points().len() {
                opts.push(StepOption {
                    drawable_id,
                    snap_point: sp_id,
                    name: format!("{} #{}, SP #{}", steps.iter().find(|s| s.id == drawable.step_id).unwrap().step.describe(), drawable.step_id, sp_id),
                });
            }
        }
    }

    opts
}

#[component]
pub fn PointC(cx: Scope, point: RwSignal<PointData>, step_id: usize) -> impl IntoView {
    move || match point.get() {
        PointData::Xy(xy) => {
            view!{ cx, 
                <div class="flex flex-row">
                    <p class="mr-1">"x:"</p>
                    <DraggableNumC d=xy.x/>
                </div>
                <div class="flex flex-row">
                    <p class="mr-1">"y:"</p>
                    <DraggableNumC d=xy.y/>
                </div>
                <button on:click=move |_| point.set(PointData::Ref(create_rw_signal(cx, None)))>"Switch it!"</button>
            }.into_view(cx)
        },
        PointData::Ref(r) => view! { cx,
            <p>"Ref"</p>
            // <select on:change=move |e| { 
            //     r.set(Some(serde_json::from_str::<StepOption>(&event_target_value(&e)).unwrap())) 
            // }>
            //     <For 
            //         each=move || options_for(cx, step_id) 
            //         key=|o| o.drawable_id
            //         view=move |o| {
            //             let o_name = o.name.clone();
            //             view!{cx, <option value={o}>{o_name}</option> }
            //         }
            //     />
            // </select>
            <button on:click=move |_| point.set(PointData::Xy(PointXYData { x: create_rw_signal(cx, 0.0), y: create_rw_signal(cx, 0.0) }))>"Switch it!"</button>
        }.into_view(cx)
    }
}

#[component]
pub fn InnerStepC(cx: Scope, step: Step) -> impl IntoView {
    match step.step {
        StepData::DrawPoint(point) => {
            view! { cx,
                <PointC point step_id={step.id}/>
            }.into_view(cx)
        },
        StepData::DrawLine(start, end) => {
            view! { cx,
                <p>"start:"</p>
                <PointC point=start step_id={step.id}/>

                <p>"end:"</p>
                <PointC point=end step_id={step.id}/>
            }.into_view(cx)
        }
    }
}

#[component]
pub fn StepC(cx: Scope, step: Step, set_steps: WriteSignal<Vec<Step>>) -> impl IntoView {
    view! { cx, 
        <div class="p-2 m-1 shadow bg-white w-[90%] min-h-[15rem] rounded-lg relative group">
            <p class="absolute left-[80%] opacity-0 group-hover:opacity-100 transition-all">
                {step.id}
            </p>
            <button 
                class="absolute left-[90%] opacity-0 group-hover:opacity-100 transition-all"
                on:click=move |_| set_steps.update(|s| { s.drain_filter(|s| s.id == step.id); })>
                "x"
            </button>
            <div class="w-full h-full flex flex-col">
                <p>"Step #" {step.id}</p>
                <InnerStepC step/>
            </div>
        </div>
    }
}

#[component]
pub fn DrawlingCanvasC(cx: Scope, drawables: Signal<Vec<Drawable>>) -> impl IntoView {
    let (mouse_pos, set_mouse_pos) = create_signal(cx, MousePos::default());
    let mousemove_callback = move |e: web_sys::MouseEvent| {
        let canvas_el = e.target().unwrap().dyn_ref::<HtmlCanvasElement>().unwrap().clone();
        let rect = canvas_el.get_bounding_client_rect();
        set_mouse_pos.set(MousePos { 
            x: (e.client_x() as f64 - rect.x() as f64) / rect.width(), 
            y: (e.client_y() as f64 - rect.y() as f64) / rect.height(),
        });
    };

    let canvas = view! { cx, <canvas class="border-2 border-gray-800 max-w-full max-h-screen aspect-[8/5]" on:mousemove=mousemove_callback/> };

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let scale_factor = 16.0f64;
    let current_aspect_ratio = canvas.width() as f64 / canvas.height() as f64;
    let desired_aspect_ratio = 8.0f64 / 5.0f64;
    let width_scale_factor = (desired_aspect_ratio / current_aspect_ratio).sqrt();
    let height_scale_factor = 1.0 / width_scale_factor;

    canvas.set_width((canvas.width() as f64 * scale_factor * width_scale_factor).ceil() as u32);
    canvas.set_height((canvas.height() as f64 * scale_factor * height_scale_factor).ceil() as u32);

    context.scale(scale_factor, scale_factor).unwrap();
    context.set_line_width(4.0 / scale_factor);

    let canvas_width = canvas.width();
    let canvas_height = canvas.height();

    create_effect(cx, move |_| {
        context.clear_rect(0.0, 0.0, canvas_width as f64, canvas_height as f64);
        for drawable in drawables().iter() {
            context.set_stroke_style(&wasm_bindgen::JsValue::from_str("black"));
            match drawable.drawable {
                DrawableData::Point(PointDrawable { x, y }) => {
                    context.begin_path();
                    context.arc(x, y, 0.8, 0.0, std::f64::consts::PI * 2.0).unwrap();
                    context.stroke();
                },
                DrawableData::Line(LineDrawable { 
                    start: PointDrawable { x: start_x, y :start_y }, 
                    end: PointDrawable { x: end_x, y :end_y }, 
                }) => {
                    context.begin_path();
                    context.move_to(start_x, start_y);
                    context.line_to(end_x, end_y);
                    context.stroke();
                }
            }

            context.set_stroke_style(&wasm_bindgen::JsValue::from_str("blue"));
            context.set_fill_style(&wasm_bindgen::JsValue::from_str("blue"));
            for sp in drawable.drawable.snap_points().iter() {
                context.begin_path();
                context.arc(sp.x, sp.y, 1.2, 0.0, std::f64::consts::PI * 2.0).unwrap();

                let mouse_pos = mouse_pos();

                // todo(chad): this is a hack, and doesn't prevent from multiple snap points being
                // selected. We need to calculate them once in the outer loop and then just pass
                // which one is selected here.
                let mx = mouse_pos.x * canvas_width as f64 / scale_factor;
                let my = mouse_pos.y * canvas_height as f64 / scale_factor;
                let dist = ((sp.x - mx) * (sp.x - mx) + (sp.y - my) * (sp.y - my)).sqrt();
                log::debug!("sp: {:?}, mouse_pos: {:?}", sp, (mx, my));
                if dist < 5.0 {
                    context.fill();
                }
                context.stroke();
            }
        }
    });

    view! { cx,
        <div class="block grow self-center"> 
            { canvas }
        </div>
    }
}

#[derive(Debug, Copy, Clone)]
struct Point {
    x: f64,
    y: f64,
}

type PointDrawable = Point;

#[derive(Debug, Copy, Clone)]
struct LineDrawable {
    start: PointDrawable,
    end: PointDrawable,
}

#[derive(Debug, Copy, Clone)]
enum DrawableData {
    Point(PointDrawable),
    Line(LineDrawable),
}

#[derive(Debug, Copy, Clone)]
pub struct Drawable {
    step_id: usize,
    drawable: DrawableData,
}

impl DrawableData {
    fn snap_points(&self) -> Vec<Point> {
        match self {
            DrawableData::Point(p) => vec![*p],
            DrawableData::Line(LineDrawable { start, end, }) => vec![
                *start,
                Point {
                    x: (start.x + end.x) / 2.0,
                    y: (start.y + end.y) / 2.0,
                },
                *end,
            ]
        }
    }
}

fn execute(steps: ReadSignal<Vec<Step>>) -> Vec<Drawable> {
    let mut drawables = Vec::new();

    for step in steps().iter() {
        match step.step {
            StepData::DrawPoint(xy) => {
                let (x, y) = xy().resolve(&drawables);
                drawables.push(Drawable { step_id: step.id, drawable: DrawableData::Point(PointDrawable { x, y }) });
            }
            StepData::DrawLine(start, end) => {
                let (start_x, start_y) = start().resolve(&drawables);
                let (end_x, end_y) = end().resolve(&drawables);
                drawables.push(
                    Drawable {
                        step_id: step.id,
                        drawable: DrawableData::Line(LineDrawable {
                            start: PointDrawable { x: start_x, y: start_y },
                            end: PointDrawable { x: end_x, y: end_y },
                        })
                    }
                    );
            }
        }
    }

    drawables
}

#[component]
pub fn DraggableNumC(cx: Scope, d: RwSignal<f64>) -> impl IntoView {
    let (d, set_d) = d.split();

    let set_drag_data = use_context::<WriteSignal<DragData>>(cx).unwrap();

    let mousedown_callback = move |e: web_sys::MouseEvent| { 
        set_drag_data.update(|dd| { 
            dd.prev = d();
            dd.start = e.y() as f64;
            dd.sig = Some(set_d); 
        }); 
    };

    view! { cx,
        <div 
            on:mousedown=mousedown_callback 
            style="user-select: none"
        >
            {d}
        </div>
    }
}

#[derive(Copy, Clone, Default)]
struct DragData {
    prev: f64,
    start: f64,
    sig: Option<WriteSignal<f64>>,
}

#[derive(Copy, Clone, Default, Debug)]
struct MousePos {
    x: f64,
    y: f64,
}

#[component]
pub fn DrawlingC(cx: Scope) -> impl IntoView {
    let (drag_data, set_drag_data) = create_signal(cx, DragData::default());
    provide_context(cx, set_drag_data);

    // let (id, set_id) = create_signal(cx, 1);
    let mut id = 1;

    let (steps, set_steps) = create_signal::<Vec<Step>>(cx, vec![
                                                        Step {
                                                            id: 0,
                                                            step: StepData::DrawPoint(create_rw_signal(cx, 
                                                                                                       PointData::Xy(
                                                                                                           PointXYData { 
                                                                                                               x: create_rw_signal(cx, 0.0), 
                                                                                                               y: create_rw_signal(cx, 0.0) 
                                                                                                           })))
                                                        }
    ]);
    provide_context(cx, steps);

    let drawables = Signal::derive(cx, move || execute(steps));
    provide_context(cx, drawables);

    let add_draw_point_step = move |_| { 
        let new_step = Step { id: id, step: StepData::DrawPoint(create_rw_signal(cx, 
                                                                                   PointData::Xy(
                                                                                       PointXYData { 
                                                                                           x: create_rw_signal(cx, 0.0), 
                                                                                           y: create_rw_signal(cx, 0.0) 
                                                                                       }))) 
        };

        set_steps.update(|s| {
            s.push(new_step);
            id += 1;
        });
    };

    let add_draw_line_step = move |_| { 
        let start = create_rw_signal(cx, 
                                     PointData::Xy(
                                         PointXYData { 
                                             x: create_rw_signal(cx, 0.0), 
                                             y: create_rw_signal(cx, 0.0) 
                                         }));

        let end = create_rw_signal(cx, 
                                     PointData::Xy(
                                         PointXYData { 
                                             x: create_rw_signal(cx, 0.0), 
                                             y: create_rw_signal(cx, 0.0) 
                                         }));

        let new_step = Step { id: id, step: StepData::DrawLine(start, end) };

        set_steps.update(|s| {
            s.push(new_step);
            id += 1;
        });
    };

    let classes = "mb-6 bg-blue-500 hover:bg-blue-700 py-4 px-2 text-white rounded w-[12rem] max-w-[85%] self-center";   

    let mousemove_callback = move |e: web_sys::MouseEvent| {
        if let DragData { prev, start, sig: Some(set_d) } = drag_data() {
            set_d(prev + start - e.y() as f64);
        }
    };
    
    let mouseup_callback = move |_: web_sys::MouseEvent| {
        set_drag_data.update(|dd| dd.sig = None);
    };

    view! { cx,
        <div class="flex flex-row h-screen w-screen" on:mousemove=mousemove_callback on:mouseup=mouseup_callback>
            <div class="flex flex-col basis-1/6 max-w-[20rem] min-w-[10rem] bg-slate-200">
                <h3 class="text-3xl text-center m-3">"Steps"</h3>
                <div class="flex flex-col items-center overflow-scroll">
                    <For
                        each=steps
                        key=|step| step.id
                        view=move |step: Step| {
                            view! { cx,
                                <StepC set_steps step/>
                            }
                        }
                    />
                </div>
                <div class="grow"/>
                <button class=classes on:click=add_draw_point_step>"Draw Point"</button>
                <button class=classes on:click=add_draw_line_step>"Draw Line"</button>
            </div>

            <DrawlingCanvasC drawables/>
        </div>
    }
}
