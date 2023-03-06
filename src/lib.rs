use leptos::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointSignal {
    pub x: RwSignal<ResolvableTo<NumberSignal>>,
    pub y: RwSignal<ResolvableTo<NumberSignal>>,
}

pub type NumberSignal = RwSignal<f64>;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

trait ResolveToNumber {
    fn resolve(&self, cx: Scope) -> f64;
}

trait ResolveToPoint {
    fn resolve(&self, cx: Scope) -> Point;
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResolvableTo<T>
where
    T: Clone + PartialEq,
{
    T(T),
    Ref(DataRef),
}

impl ResolveToPoint for ResolvableTo<PointSignal> {
    fn resolve(&self, cx: Scope) -> Point {
        match self {
            ResolvableTo::T(point) => Point {
                x: point.x.get().resolve(cx),
                y: point.y.get().resolve(cx),
            },
            ResolvableTo::Ref(r) => ResolveToPoint::resolve(r, cx),
        }
    }
}

impl ResolveToNumber for ResolvableTo<NumberSignal> {
    fn resolve(&self, cx: Scope) -> f64 {
        match self {
            ResolvableTo::T(n) => n.get(),
            ResolvableTo::Ref(r) => ResolveToNumber::resolve(r, cx),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StepData {
    DrawPoint(RwSignal<ResolvableTo<PointSignal>>),
    DrawLine {
        start: RwSignal<ResolvableTo<PointSignal>>,
        end: RwSignal<ResolvableTo<PointSignal>>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataRefPathEl {
    Step,
    Data,
    WithId(usize),
    PropName(&'static str),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataRef(Vec<DataRefPathEl>);

impl DataRef {
    pub fn desc(&self) -> String {
        self.0
            .iter()
            .map(|el| match el {
                DataRefPathEl::Step => "step".to_string(),
                DataRefPathEl::Data => "data".to_string(),
                DataRefPathEl::WithId(id) => format!("[{}]", id),
                DataRefPathEl::PropName(name) => format!(".{}", name),
            })
            .collect::<Vec<String>>()
            .join("")
    }
}

impl ResolveToNumber for DataRef {
    fn resolve(&self, cx: Scope) -> f64 {
        match self.0[0] {
            DataRefPathEl::Step => {
                let step_id = match self.0[1] {
                    DataRefPathEl::WithId(i) => i,
                    _ => todo!(),
                };
                let step = use_context::<RwSignal<Vec<Step>>>(cx)
                    .unwrap()
                    .with(|steps| {
                        steps
                            .iter()
                            .find(|d| d.id == step_id)
                            .cloned()
                            .expect("Invalid step id")
                    });
                match step.data {
                    StepData::DrawPoint(point) => match self.0[2] {
                        DataRefPathEl::PropName("x") => point.get().resolve(cx).x,
                        DataRefPathEl::PropName("y") => point.get().resolve(cx).y,
                        _ => todo!(),
                    },
                    StepData::DrawLine { start, end } => match self.0[2] {
                        DataRefPathEl::PropName("start") => match self.0[3] {
                            DataRefPathEl::PropName("x") => start.get().resolve(cx).x,
                            DataRefPathEl::PropName("y") => start.get().resolve(cx).y,
                            _ => todo!(),
                        },
                        DataRefPathEl::PropName("end") => match self.0[3] {
                            DataRefPathEl::PropName("x") => end.get().resolve(cx).x,
                            DataRefPathEl::PropName("y") => end.get().resolve(cx).y,
                            _ => todo!(),
                        },
                        _ => todo!(),
                    },
                }
            }
            DataRefPathEl::Data => todo!(),
            _ => todo!(),
        }
    }
}

impl ResolveToPoint for DataRef {
    fn resolve(&self, cx: Scope) -> Point {
        match self.0[0] {
            DataRefPathEl::Step => {
                let step_id = match self.0[1] {
                    DataRefPathEl::WithId(i) => i,
                    _ => todo!(),
                };
                let step = use_context::<RwSignal<Vec<Step>>>(cx)
                    .unwrap()
                    .with(|steps| {
                        steps
                            .iter()
                            .find(|d| d.id == step_id)
                            .cloned()
                            .expect("Invalid step id")
                    });
                let prop_name = match self.0[2] {
                    DataRefPathEl::PropName(s) => s,
                    _ => todo!(),
                };
                match step.data {
                    StepData::DrawPoint(point) => match prop_name {
                        "self" => point.get().resolve(cx),
                        _ => panic!(
                            "Invalid prop name '{}': expected one of [{:?}]",
                            prop_name, "self"
                        ),
                    },
                    StepData::DrawLine { start, end } => {
                        let start = start.get().resolve(cx);
                        let end = end.get().resolve(cx);

                        match prop_name {
                            "start" => start,
                            "mid" => Point {
                                x: (start.x + end.x) / 2.0,
                                y: (start.y + end.y) / 2.0,
                            },
                            "end" => end,
                            _ => panic!(
                                "Invalid prop name '{}': expected one of [{:?}]",
                                prop_name,
                                &["start", "mid", "end"]
                            ),
                        }
                    }
                }
            }
            DataRefPathEl::Data => todo!(),
            _ => todo!(),
        }
    }
}

impl Step {
    pub fn snap_points(&self) -> Vec<DataRef> {
        match self.data {
            StepData::DrawPoint(_) => vec![DataRef(vec![
                DataRefPathEl::Step,
                DataRefPathEl::WithId(self.id),
                DataRefPathEl::PropName("self"),
            ])],
            StepData::DrawLine { .. } => vec![
                DataRef(vec![
                    DataRefPathEl::Step,
                    DataRefPathEl::WithId(self.id),
                    DataRefPathEl::PropName("start"),
                ]),
                DataRef(vec![
                    DataRefPathEl::Step,
                    DataRefPathEl::WithId(self.id),
                    DataRefPathEl::PropName("mid"),
                ]),
                DataRef(vec![
                    DataRefPathEl::Step,
                    DataRefPathEl::WithId(self.id),
                    DataRefPathEl::PropName("end"),
                ]),
            ],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Step {
    pub id: usize,
    pub data: StepData,
}

#[derive(Clone, Copy, Debug)]
pub enum DataData {
    Number(RwSignal<f64>),
    Point(RwSignal<PointSignal>),
}

#[derive(Clone, Copy, Debug)]
pub struct Data {
    pub id: usize,
    pub data: DataData,
}

#[derive(Copy, Clone, Default)]
struct DragData {
    initial_value: f64,
    start: f64,
}

#[component]
pub fn DraggableNumView(cx: Scope, d: RwSignal<f64>) -> impl IntoView {
    let (d, set_d) = d.split();

    let (drag_data, set_drag_data) = create_signal(cx, DragData::default());

    let mousemove_callback = move |e: web_sys::MouseEvent| {
        let delta = drag_data().start - e.y() as f64;
        set_d(drag_data().initial_value + delta);
    };
    let mousemove_closure =
        wasm_bindgen::prelude::Closure::<dyn Fn(_)>::new(mousemove_callback).into_js_value();
    let mousemove_closure_clone = mousemove_closure.clone();

    let mouseup_callback = move |_e: web_sys::MouseEvent| {
        document()
            .remove_event_listener_with_callback(
                "mousemove",
                mousemove_closure.as_ref().unchecked_ref(),
            )
            .unwrap();
    };
    let mouseup_closure = wasm_bindgen::prelude::Closure::<dyn Fn(_)>::new(mouseup_callback);

    let mousedown_callback = move |e: web_sys::MouseEvent| {
        set_drag_data.update(|dd| {
            dd.initial_value = d();
            dd.start = e.y() as f64;
        });

        document()
            .add_event_listener_with_callback(
                "mousemove",
                mousemove_closure_clone.as_ref().unchecked_ref(),
            )
            .unwrap();

        document()
            .add_event_listener_with_callback("mouseup", mouseup_closure.as_ref().unchecked_ref())
            .unwrap();
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

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum InferTarget {
    Number(RwSignal<ResolvableTo<NumberSignal>>),
    Point(RwSignal<ResolvableTo<PointSignal>>),
}

#[component]
fn ResolvableToNumberView(
    cx: Scope,
    n: RwSignal<ResolvableTo<NumberSignal>>,
    data_ref_path: StoredValue<Vec<DataRefPathEl>>,
) -> impl IntoView {
    move || {
        let context_infer_target = use_context::<RwSignal<Option<InferTarget>>>(cx).unwrap();

        if let Some(InferTarget::Number(it)) = context_infer_target.get() {
            if n == it {
                return view! { cx,
                    <div class="flex flex-row">
                        <p>"..."</p>
                        <button class="border-2 border-gray-800 mt-4" on:click=move |_| {
                            context_infer_target.set(None);
                        }>
                            "C"
                        </button>
                    </div>
                }
                .into_view(cx);
            } else {
                return view! { cx,
                    <div class="flex flex-row">
                        <button class="border-2 border-gray-800 mt-4" on:click=move |_| {
                            it.set(ResolvableTo::Ref(DataRef(data_ref_path.get())));
                            context_infer_target.set(None);
                        }>
                            "O"
                        </button>
                    </div>
                }
                .into_view(cx);
            }
        }

        match n.get() {
            ResolvableTo::T(t) => view! { cx,
                <div class="flex flex-row">
                    <DraggableNumView d=t />
                    <button class="border-2 border-gray-800" on:click=move |_| {
                        context_infer_target.set(Some(InferTarget::Number(n)));
                    }>"I"</button>
                </div>
            }
            .into_view(cx),
            ResolvableTo::Ref(r) => view! { cx,
                <div class="flex flex-col">
                    <p>{r.desc()}</p>
                    // <button class="border-2 border-gray-800 mt-4" on:click=move |_| {
                    //     n.set(ResolvableTo::T(create_signal(cx, 0.0)));
                    // }>
                    //     "Cancel Infer"
                    // </button>
                </div>
            }
            .into_view(cx),
        }
    }
}

#[component]
fn InnerStepViewDrawPoint(
    cx: Scope,
    sig: RwSignal<ResolvableTo<PointSignal>>,
    point: PointSignal,
    data_ref_path: StoredValue<Vec<DataRefPathEl>>,
) -> impl IntoView {
    move || {
        let context_infer_target = use_context::<RwSignal<Option<InferTarget>>>(cx).unwrap();
        if let Some(InferTarget::Point(it)) = context_infer_target.get() {
            if sig == it {
                return view! { cx,
                    <div class="flex flex-col">
                        <p>"..."</p>
                        <button class="border-2 border-gray-800 mt-4" on:click=move |_| {
                            context_infer_target.set(None);
                        }>
                            "Cancel Infer"
                        </button>
                    </div>
                }
                .into_view(cx);
            }
        }

        let mut x_path = data_ref_path.get();
        x_path.push(DataRefPathEl::PropName("x"));
        let x_path = store_value(cx, x_path);

        let mut y_path = data_ref_path.get();
        y_path.push(DataRefPathEl::PropName("y"));
        let y_path = store_value(cx, y_path);

        view! { cx,
            <div class="flex flex-col">
                <p>"Draw Point"</p>
                <div class="flex flex-row">
                    <p>"x: "</p>
                    <ResolvableToNumberView n={point.x} data_ref_path=x_path />
                    <p class="ml-3">"y: "</p>
                    <ResolvableToNumberView n={point.y} data_ref_path=y_path />
                </div>
                <button class="border-2 border-gray-800 mt-4" on:click=move |_| {
                    context_infer_target.set(Some(InferTarget::Point(sig)));
                }>
                    "Infer"
                </button>
            </div>
        }
        .into_view(cx)
    }
}

#[component]
fn InnerStepViewResolveableToPoint(
    cx: Scope,
    point: RwSignal<ResolvableTo<PointSignal>>,
    data_ref_path: StoredValue<Vec<DataRefPathEl>>,
) -> impl IntoView {
    move || match point() {
        ResolvableTo::T(p) => view! { cx,
            <InnerStepViewDrawPoint sig=point point=p data_ref_path=data_ref_path />
        }
        .into_view(cx),
        ResolvableTo::Ref(dr) => {
            let context_infer_target = use_context::<RwSignal<Option<InferTarget>>>(cx).unwrap();
            view! { cx,
                <div>{dr.desc()}</div>
                <button class="border-2 border-gray-800 mt-4" on:click=move |_| {
                    context_infer_target.set(Some(InferTarget::Point(point)));
                }>
                    "Infer"
                </button>
            }
        }
        .into_view(cx),
    }
}

#[component]
fn InnerStepViewDrawLine(
    cx: Scope,
    start: RwSignal<ResolvableTo<PointSignal>>,
    end: RwSignal<ResolvableTo<PointSignal>>,
    data_ref_path: StoredValue<Vec<DataRefPathEl>>,
) -> impl IntoView {
    let mut start_path = data_ref_path.get();
    start_path.push(DataRefPathEl::PropName("start"));
    let start_path = store_value(cx, start_path);

    let mut end_path = data_ref_path.get();
    end_path.push(DataRefPathEl::PropName("start"));
    let end_path = store_value(cx, end_path);

    view! { cx,
        <div class="flex flex-col">
            <p>"Draw Line"</p>

            <p>"start: "</p>
            <InnerStepViewResolveableToPoint point={start} data_ref_path=start_path />

            <p>"end: "</p>
            <InnerStepViewResolveableToPoint point={end} data_ref_path=end_path />
        </div>
    }
}

#[component]
pub fn InnerStepView(cx: Scope, step: Step) -> impl IntoView {
    move || {
        let data_ref_path = store_value(
            cx,
            vec![DataRefPathEl::Step, DataRefPathEl::WithId(step.id)],
        );

        match step.data {
            StepData::DrawPoint(point) => match point.get() {
                ResolvableTo::T(p) => view! { cx,
                    <InnerStepViewDrawPoint sig=point point=p data_ref_path />
                }
                .into_view(cx),
                ResolvableTo::Ref(dr) => {
                    view! { cx,
                        <div>"TODO"</div>
                    <div>{dr.desc()}</div>
                    }
                }
                .into_view(cx),
            },
            StepData::DrawLine { start, end } => view! { cx,
                <InnerStepViewDrawLine start end data_ref_path />
            }
            .into_view(cx),
        }
    }
}

#[component]
pub fn StepView(cx: Scope, step: Step) -> impl IntoView {
    view! { cx,
        <div class="p-2 m-1 shadow bg-white w-[90%] rounded-lg relative group">
            <button
                class="absolute left-[90%] opacity-0 group-hover:opacity-100 transition-all"
                on:click=move |_| {
                    use_context::<RwSignal<Vec<Step>>>(cx).unwrap().update(|s| {
                        s.retain(|s| s.id != step.id);
                    });
                }>
                "x"
            </button>
            <div class="w-full h-full flex flex-col">
                <p>"Step #" {step.id}</p>
                <InnerStepView step/>
            </div>
        </div>
    }
}

#[component]
pub fn InnerDataViewPoint(
    cx: Scope,
    point: PointSignal,
    data_ref_path: StoredValue<Vec<DataRefPathEl>>,
) -> impl IntoView {
    let mut x_path = data_ref_path.get();
    x_path.push(DataRefPathEl::PropName("x"));
    let x_path = store_value(cx, x_path);

    let mut y_path = data_ref_path.get();
    y_path.push(DataRefPathEl::PropName("y"));
    let y_path = store_value(cx, y_path);

    move || {
        view! { cx,
            <div class="flex flex-row">
                <p>"x: "</p>
                <ResolvableToNumberView n={point.x} data_ref_path=x_path />
                <p class="ml-3">"y: "</p>
                <ResolvableToNumberView n={point.y} data_ref_path=y_path />
            </div>
        }
    }
}

#[component]
pub fn InnerDataView(cx: Scope, data: Data) -> impl IntoView {
    move || match data.data {
        DataData::Number(n) => view! { cx,
            <div>
                <p>"Number"</p>
                <DraggableNumView d={n} />
            </div>
        }
        .into_view(cx),
        DataData::Point(p) => view! { cx,
            <div>
                <p>"Point"</p>
                <InnerDataViewPoint point={p.get()} data_ref_path={store_value(cx, vec![
                    DataRefPathEl::Data,
                    DataRefPathEl::WithId(data.id),
                ])} />
            </div>
        }
        .into_view(cx),
    }
}

#[component]
pub fn DataView(cx: Scope, data: Data) -> impl IntoView {
    view! { cx,
        <div class="p-2 m-1 shadow bg-white w-[90%] rounded-lg relative group">
            <button
                class="absolute left-[90%] opacity-0 group-hover:opacity-100 transition-all"
                on:click=move |_| {
                    use_context::<RwSignal<Vec<Data>>>(cx).unwrap().update(|s| {
                        s.retain(|d| d.id != data.id);
                    });
                }>
                "x"
            </button>
            <div class="w-full h-full flex flex-col">
                <p>"Data #" {data.id}</p>
                <InnerDataView data/>
            </div>
        </div>
    }
}

#[component]
pub fn DrawlingCanvasView(cx: Scope, steps: RwSignal<Vec<Step>>) -> impl IntoView {
    let scale_factor = 16.0f64;

    let canvas = view! { cx,
        <canvas class="border-2 border-gray-800 max-w-screen max-h-screen" />
    };
    let canvas_clone_mousemove = canvas.clone();

    let (mouse_pos, set_mouse_pos) = create_signal(cx, Point::default());

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let current_aspect_ratio = canvas.width() as f64 / canvas.height() as f64;
    let desired_aspect_ratio = current_aspect_ratio;
    let width_scale_factor = (desired_aspect_ratio / current_aspect_ratio).sqrt();
    let height_scale_factor = 1.0 / width_scale_factor;

    canvas.set_width((canvas.width() as f64 * scale_factor * width_scale_factor).ceil() as u32);
    canvas.set_height((canvas.height() as f64 * scale_factor * height_scale_factor).ceil() as u32);

    context.scale(scale_factor, scale_factor).unwrap();
    context.set_line_width(4.0 / scale_factor);

    let canvas_width = canvas.width();
    let canvas_height = canvas.height();

    // todo(chad): make mouse_pos a PointSignal
    let hover_infer_target = create_rw_signal(
        cx,
        Some(ResolvableTo::T(PointSignal {
            x: create_rw_signal(cx, ResolvableTo::T(create_rw_signal(cx, mouse_pos().x))),
            y: create_rw_signal(cx, ResolvableTo::T(create_rw_signal(cx, mouse_pos().y))),
        })),
    );

    let mousemove_callback = move |e: web_sys::MouseEvent| {
        let rect = canvas_clone_mousemove.get_bounding_client_rect();
        set_mouse_pos.set(Point {
            x: (e.client_x() as f64 - rect.x() as f64) / rect.width() as f64 * canvas_width as f64
                / scale_factor,
            y: (e.client_y() as f64 - rect.y() as f64) / rect.height() as f64
                * canvas_height as f64
                / scale_factor,
        });
    };
    let mousemove_closure =
        wasm_bindgen::prelude::Closure::<dyn Fn(_)>::new(mousemove_callback).into_js_value();
    canvas
        .add_event_listener_with_callback("mousemove", mousemove_closure.as_ref().unchecked_ref())
        .unwrap();

    let mousedown_callback = move |_e: web_sys::MouseEvent| {
        let context_infer_target = use_context::<RwSignal<Option<InferTarget>>>(cx).unwrap();

        if let (Some(InferTarget::Point(it)), Some(hover_infer_target)) =
            (context_infer_target.get(), hover_infer_target.get())
        {
            it.set(hover_infer_target);
            context_infer_target.set(None);
        }
    };
    let mousedown_closure =
        wasm_bindgen::prelude::Closure::<dyn Fn(_)>::new(mousedown_callback).into_js_value();
    canvas
        .add_event_listener_with_callback("mousedown", mousedown_closure.as_ref().unchecked_ref())
        .unwrap();

    let snap_points: Memo<Vec<DataRef>> = create_memo(cx, move |_| {
        console_log("Memoizing snap points!");
        steps.with(|steps| steps.iter().map(|s| s.snap_points()).flatten().collect())
    });

    create_effect(cx, move |_| {
        // console_log("running the effect!");

        context.clear_rect(0.0, 0.0, canvas_width as f64, canvas_height as f64);

        steps.with(|steps| {
            for step in steps.iter() {
                match step.data {
                    StepData::DrawPoint(point) => match point() {
                        ResolvableTo::T(point) => {
                            context.begin_path();
                            context
                                .arc(
                                    point.x.get().resolve(cx),
                                    point.y.get().resolve(cx),
                                    1.0,
                                    0.0,
                                    std::f64::consts::PI * 2.0,
                                )
                                .unwrap();
                            context.stroke();
                        }
                        ResolvableTo::Ref { .. } => todo!(),
                    },
                    StepData::DrawLine { start, end } => {
                        let start: Point = start().resolve(cx);
                        let end: Point = end().resolve(cx);

                        context.begin_path();
                        context.move_to(start.x, start.y);
                        context.line_to(end.x, end.y);
                        context.stroke();
                    }
                }
            }
        });

        snap_points.with(|snap_points| {
            for sp in snap_points.iter() {
                context.set_stroke_style(&wasm_bindgen::JsValue::from_str("red"));

                let sp = ResolveToPoint::resolve(sp, cx);

                context.begin_path();
                context
                    .arc(sp.x, sp.y, 1.3, 0.0, std::f64::consts::PI * 2.0)
                    .unwrap();
                context.stroke();
            }
        });

        let context_infer_target = use_context::<RwSignal<Option<InferTarget>>>(cx).unwrap();
        if context_infer_target.get().is_some() {
            hover_infer_target.set(Some(ResolvableTo::T(PointSignal {
                x: create_rw_signal(
                    cx,
                    ResolvableTo::T(create_rw_signal(cx, mouse_pos().x.round())),
                ),
                y: create_rw_signal(
                    cx,
                    ResolvableTo::T(create_rw_signal(cx, mouse_pos().y.round())),
                ),
            })));

            // todo(chad): @Performance
            // This subscribes the effect to any mouse move changes, which is a lot of unnecessary runs.
            // We should only run this effect when the mouse movement causes a change to the currently selected snap point.
            snap_points.with(|snap_points| {
                for sp in snap_points.iter() {
                    let spr = ResolveToPoint::resolve(sp, cx);
                    let dist =
                        ((spr.x - mouse_pos().x).powi(2) + (spr.y - mouse_pos().y).powi(2)).sqrt();
                    if dist < 5.0 {
                        hover_infer_target.set(Some(ResolvableTo::Ref(sp.clone())));
                    }
                }
            });
        }

        if let Some(hit) = hover_infer_target.get() {
            let mut fill = false;
            if let ResolvableTo::Ref(_) = hit {
                context.set_fill_style(&wasm_bindgen::JsValue::from_str("green"));
                fill = true;
            }

            let hit = hit.resolve(cx);
            context.begin_path();
            context.set_stroke_style(&wasm_bindgen::JsValue::from_str("green"));
            context
                .arc(hit.x, hit.y, 1.0, 0.0, std::f64::consts::PI * 2.0)
                .unwrap();
            context.stroke();
            if fill {
                context.fill();
            }
        }
    });

    view! { cx,
        <div class="block grow self-center">
            { canvas }
        </div>
    }
}

#[component]
pub fn DrawlingView(cx: Scope) -> impl IntoView {
    let datas = create_rw_signal::<Vec<Data>>(cx, Vec::new());

    let steps = create_rw_signal::<Vec<Step>>(cx, Vec::new());
    provide_context(cx, steps);

    let infer_target: RwSignal<Option<InferTarget>> = create_rw_signal(cx, None);
    provide_context(cx, infer_target);

    console_log("DrawlingView Setup");

    let add_draw_line_step = move |_| {
        steps.update(|s| {
            s.push(Step {
                id: s.len(),
                data: StepData::DrawLine {
                    start: create_rw_signal(
                        cx,
                        ResolvableTo::T(PointSignal {
                            x: create_rw_signal(cx, ResolvableTo::T(create_rw_signal(cx, 0.0))),
                            y: create_rw_signal(cx, ResolvableTo::T(create_rw_signal(cx, 0.0))),
                        }),
                    ),
                    end: create_rw_signal(
                        cx,
                        ResolvableTo::T(PointSignal {
                            x: create_rw_signal(cx, ResolvableTo::T(create_rw_signal(cx, 0.0))),
                            y: create_rw_signal(cx, ResolvableTo::T(create_rw_signal(cx, 0.0))),
                        }),
                    ),
                },
            })
        });
    };
    let add_draw_point_step = move |_| {
        steps.update(|s| {
            s.push(Step {
                id: s.len(),
                data: StepData::DrawPoint(create_rw_signal(
                    cx,
                    ResolvableTo::T(PointSignal {
                        x: create_rw_signal(cx, ResolvableTo::T(create_rw_signal(cx, 0.0))),
                        y: create_rw_signal(cx, ResolvableTo::T(create_rw_signal(cx, 0.0))),
                    }),
                )),
            })
        });
    };

    let add_number_data = move |_| {
        datas.update(|d| {
            d.push(Data {
                id: d.len(),
                data: DataData::Number(create_rw_signal(cx, 0.0)),
            })
        });
    };
    let add_point_data = move |_| {
        datas.update(|d| {
            d.push(Data {
                id: d.len(),
                data: DataData::Point(create_rw_signal(
                    cx,
                    PointSignal {
                        x: create_rw_signal(cx, ResolvableTo::T(create_rw_signal(cx, 0.0))),
                        y: create_rw_signal(cx, ResolvableTo::T(create_rw_signal(cx, 0.0))),
                    },
                )),
            })
        });
    };

    view! { cx,
        <div class="flex flex-row h-screen w-screen">
            <div class="flex flex-col basis-1/6 max-w-[20rem] min-w-[13rem] bg-slate-200">
                <h3 class="text-3xl text-center m-3">"Data"</h3>
                <div class="flex flex-col justify-self-end self-center">
                    <button class= "mb-6 bg-blue-500 hover:bg-blue-700 py-2 px-1 text-white rounded w-[12rem] max-w-[85%] self-center" on:click=add_number_data>"+ Number"</button>
                    <button class="mb-6 bg-blue-500 hover:bg-blue-700 py-2 px-1 text-white rounded w-[12rem] max-w-[85%] self-center" on:click=add_point_data>"+ Point"</button>
                </div>
                <div class="flex flex-col items-center overflow-scroll">
                    <For
                        each=datas
                        key=|data| data.id
                        view=move |data: Data| {
                            view! { cx,
                                <DataView data />
                            }
                        }
                    />
                </div>

                <h3 class="text-3xl text-center m-3">"Steps"</h3>
                <div class="flex flex-col items-center overflow-scroll">
                    <For
                        each=steps
                        key=|step| step.id
                        view=move |step: Step| {
                            view! { cx,
                                <StepView step />
                            }
                        }
                    />
                </div>
                <div class="flex flex-col justify-self-end self-center">
                    <button class= "mb-6 bg-blue-500 hover:bg-blue-700 py-2 px-1 text-white rounded w-[12rem] max-w-[85%] self-center" on:click=add_draw_point_step>"Draw Point"</button>
                    <button class="mb-6 bg-blue-500 hover:bg-blue-700 py-2 px-1 text-white rounded w-[12rem] max-w-[85%] self-center" on:click=add_draw_line_step>"Draw Line"</button>
                </div>
            </div>

            <DrawlingCanvasView steps />
        </div>
    }
}
