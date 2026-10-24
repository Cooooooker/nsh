//! `Sheet`: it comes in when opened and goes once its slide-out ends, also
//! on a manual clock that only the test moves (its end is the motion's,
//! not a timer of the app's clock).

use nsh_base::sheet;
use nsh_core::{
    Size,
    ui::{Root, prelude::*},
};
use nsh_engine::EngineConfig;
use nsh_testing::Headless;

fn start<C: Component<Props = ()>>() -> Headless {
    let mut app = Headless::start(EngineConfig {
        viewport: Size::new(px(390.0), px(844.0)),
        ..EngineConfig::new(Root::of::<C>())
    })
    .unwrap();
    app.first_render().unwrap();
    app
}

#[component]
fn SwipeScreen() -> impl IntoElement {
    let open = signal(true);
    let permitted = signal(true);
    let decline = signal(false);
    let requests = signal(0);
    view()
        .child(text(move || format!("requests:{}", requests.get())))
        .child(
            sheet()
                .open(move || Some(open.get()))
                .close_on_swipe(permitted)
                .on_open_change(move |_: &nsh_base::SheetEvent| {
                    requests.update(|n| *n += 1);
                    if !decline.get_untracked() {
                        open.set(false);
                    }
                })
                .child(
                    view()
                        .h(px(260.0))
                        .child(
                            view()
                                .test_id("handle")
                                .h(px(44.0))
                                .child(text("Sheet body")),
                        )
                        .child(
                            view()
                                .test_id("lock")
                                .h(px(44.0))
                                .on_press(move || permitted.set(false)),
                        )
                        .child(
                            view()
                                .test_id("decline")
                                .h(px(44.0))
                                .on_press(move || decline.set(true)),
                        )
                        .child(
                            view()
                                .test_id("close")
                                .h(px(44.0))
                                .on_press(move || open.set(false)),
                        ),
                ),
        )
}

#[test]
fn a_bottom_sheet_can_be_swiped_closed() {
    let mut app = start::<SwipeScreen>();
    app.wait(std::time::Duration::from_millis(400)).unwrap();
    app.drag("id:handle", point(px(0.0), px(100.0))).unwrap();
    app.wait_for_gone("Sheet body", std::time::Duration::from_secs(2))
        .unwrap();
    app.wait_for("requests:1").unwrap();
}

#[test]
fn a_short_drag_returns_without_a_close_request() {
    let mut app = start::<SwipeScreen>();
    app.wait(std::time::Duration::from_millis(400)).unwrap();
    app.drag("id:handle", point(px(0.0), px(20.0))).unwrap();
    app.wait(std::time::Duration::from_millis(400)).unwrap();
    app.wait_for("requests:0").unwrap();
    app.press("id:close").unwrap();
    app.wait_for_gone("Sheet body", std::time::Duration::from_secs(2))
        .unwrap();
}

#[test]
fn a_cancelled_drag_does_not_close_even_past_the_release_threshold() {
    use std::sync::{Arc, Mutex};

    use nsh_core::{EventKind, EventPayload, MountItem};

    let mut app = Headless::start(EngineConfig {
        viewport: Size::new(px(390.0), px(844.0)),
        ..EngineConfig::new(Root::of::<SwipeScreen>())
    })
    .unwrap();
    let panel = Arc::new(Mutex::new(None));
    let seen = panel.clone();
    app.observe(move |_, batch| {
        for item in &batch.items {
            if let MountItem::Events { id, events } = item
                && events.contains(EventKind::PanEnd)
            {
                *seen.lock().unwrap() = Some(*id);
            }
        }
    });
    app.first_render().unwrap();
    app.wait(std::time::Duration::from_millis(400)).unwrap();
    let id = panel
        .lock()
        .unwrap()
        .expect("the sheet listens for release");
    app.engine().control().dispatch_event(
        id,
        EventKind::PanEnd,
        EventPayload::PanCancelled {
            translation: point(px(0.0), px(150.0)),
            velocity_x: 0.0,
            velocity_y: 2000.0,
            value_x: None,
            value_y: Some(px(150.0)),
        },
    );
    app.wait(std::time::Duration::from_millis(400)).unwrap();
    app.wait_for("requests:0").unwrap();
    app.press("id:close").unwrap();
    app.wait_for_gone("Sheet body", std::time::Duration::from_secs(2))
        .unwrap();
}

#[test]
fn a_permission_change_keeps_the_existing_content_and_stops_swiping() {
    let mut app = start::<SwipeScreen>();
    app.wait(std::time::Duration::from_millis(400)).unwrap();
    app.press("id:lock").unwrap();
    app.drag("id:handle", point(px(0.0), px(100.0))).unwrap();
    app.wait_for("requests:0").unwrap();
    app.wait_for("Sheet body").unwrap();
}

#[test]
fn declining_a_controlled_request_returns_the_sheet() {
    let mut app = start::<SwipeScreen>();
    app.wait(std::time::Duration::from_millis(400)).unwrap();
    app.press("id:decline").unwrap();
    app.drag("id:handle", point(px(0.0), px(100.0))).unwrap();
    app.wait(std::time::Duration::from_millis(400)).unwrap();
    app.wait_for("requests:1").unwrap();
    app.press("id:close").unwrap();
    app.wait_for_gone("Sheet body", std::time::Duration::from_secs(2))
        .unwrap();
}

#[test]
fn swiping_is_opt_in() {
    let mut app = start::<Screen>();
    app.press("id:toggle").unwrap();
    app.wait(std::time::Duration::from_millis(400)).unwrap();
    app.drag("Sheet body", point(px(0.0), px(120.0))).unwrap();
    app.wait_for("Sheet body").unwrap();
    app.press("id:toggle").unwrap();
    app.wait_for_gone("Sheet body", std::time::Duration::from_secs(2))
        .unwrap();
}

/// A button that opens and closes a sheet holding `Sheet body`.
#[component]
fn Screen() -> impl IntoElement {
    let open = signal(false);
    view()
        .child(
            view()
                .test_id("toggle")
                .w(px(40.0))
                .h(px(40.0))
                .on_press(move || open.set(!open.get_untracked())),
        )
        .child(
            sheet()
                .open(move || Some(open.get()))
                .child(view().h(px(200.0)).child(text("Sheet body"))),
        )
}

#[test]
fn a_sheet_goes_once_its_motion_ends_on_a_manual_clock() {
    let mut app = Headless::start(EngineConfig {
        viewport: Size::new(px(390.0), px(844.0)),
        clock: nsh_engine::Clock::manual(std::time::SystemTime::now()),
        ..EngineConfig::new(Root::of::<Screen>())
    })
    .expect("the engine starts");
    app.first_render().unwrap();
    app.press("id:toggle").unwrap();
    app.wait_for("Sheet body").unwrap();
    app.press("id:toggle").unwrap();
    app.wait_for_gone("Sheet body", std::time::Duration::from_secs(5))
        .unwrap();
}
