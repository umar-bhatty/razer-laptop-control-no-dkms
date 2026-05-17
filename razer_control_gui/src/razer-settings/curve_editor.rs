use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{Box, Button, Label, SpinButton};
use gtk::glib::clone;

use crate::comms;
use crate::set_fan_curve;

struct Row {
    id: u64,
    container: Box,
    temp: SpinButton,
    rpm: SpinButton,
}

pub struct CurveEditor {
    pub container: Box,
}

pub fn make_curve_editor(
    ac: bool,
    sensor: comms::Sensor,
    fan_range: (f64, f64),
    initial: Vec<comms::CurvePoint>,
) -> CurveEditor {
    let container = Box::new(gtk::Orientation::Vertical, 6);

    let header = Box::new(gtk::Orientation::Horizontal, 8);
    let label = Label::new(Some(match sensor {
        comms::Sensor::Cpu => "CPU curve",
        comms::Sensor::Gpu => "GPU curve",
    }));
    label.set_halign(gtk::Align::Start);
    header.pack_start(&label, true, true, 0);
    let add_btn = Button::with_label("+ Add point");
    header.pack_end(&add_btn, false, false, 0);
    container.add(&header);

    let columns = Box::new(gtk::Orientation::Horizontal, 0);
    let col_a = Label::new(Some("Temp °C"));
    col_a.set_xalign(0.0);
    let col_b = Label::new(Some("RPM"));
    col_b.set_xalign(0.0);
    columns.pack_start(&col_a, true, true, 0);
    columns.pack_start(&col_b, true, true, 0);
    container.add(&columns);

    let rows_box = Box::new(gtk::Orientation::Vertical, 3);
    container.add(&rows_box);

    let rows: Rc<RefCell<Vec<Row>>> = Rc::new(RefCell::new(Vec::new()));
    let next_id: Rc<Cell<u64>> = Rc::new(Cell::new(0));

    let push_curve: Rc<dyn Fn()> = {
        let rows = rows.clone();
        Rc::new(move || {
            let mut points: Vec<comms::CurvePoint> = rows.borrow().iter()
                .map(|r| comms::CurvePoint {
                    temp_c: r.temp.value().clamp(0.0, 255.0) as u8,
                    rpm: r.rpm.value().clamp(0.0, 65535.0) as u16,
                })
                .collect();
            points.sort_by_key(|p| p.temp_c);
            if points.is_empty() {
                return;
            }
            let _ = set_fan_curve(ac, sensor, points);
        })
    };

    let add_row: Rc<dyn Fn(comms::CurvePoint)> = {
        let rows = rows.clone();
        let rows_box = rows_box.clone();
        let next_id = next_id.clone();
        let push_curve = push_curve.clone();
        Rc::new(move |point: comms::CurvePoint| {
            let row_id = next_id.get();
            next_id.set(row_id + 1);

            let row_container = Box::new(gtk::Orientation::Horizontal, 6);

            let temp_adjust = gtk::Adjustment::new(point.temp_c as f64, 0.0, 110.0, 1.0, 5.0, 0.0);
            let temp = SpinButton::new(Some(&temp_adjust), 1.0, 0);
            temp.set_hexpand(true);

            let rpm_adjust = gtk::Adjustment::new(point.rpm as f64, fan_range.0, fan_range.1, 50.0, 100.0, 0.0);
            let rpm = SpinButton::new(Some(&rpm_adjust), 50.0, 0);
            rpm.set_hexpand(true);

            let remove_btn = Button::with_label("\u{2212}");

            row_container.pack_start(&temp, true, true, 0);
            row_container.pack_start(&rpm, true, true, 0);
            row_container.pack_end(&remove_btn, false, false, 0);

            rows_box.add(&row_container);
            row_container.show_all();

            rows.borrow_mut().push(Row {
                id: row_id,
                container: row_container,
                temp: temp.clone(),
                rpm: rpm.clone(),
            });

            temp.connect_value_changed(clone!(@strong push_curve => move |_| push_curve()));
            rpm.connect_value_changed(clone!(@strong push_curve => move |_| push_curve()));

            remove_btn.connect_clicked(clone!(@strong rows, @strong rows_box, @strong push_curve => move |_| {
                let removed = {
                    let mut rows_mut = rows.borrow_mut();
                    if rows_mut.len() <= 1 {
                        return;
                    }
                    let idx = rows_mut.iter().position(|r| r.id == row_id);
                    idx.map(|i| rows_mut.remove(i))
                };
                if let Some(r) = removed {
                    rows_box.remove(&r.container);
                    push_curve();
                }
            }));
        })
    };

    if initial.is_empty() {
        add_row(comms::CurvePoint { temp_c: 50, rpm: fan_range.0 as u16 });
    } else {
        for p in initial {
            add_row(p);
        }
    }

    add_btn.connect_clicked(clone!(@strong rows, @strong add_row, @strong push_curve => move |_| {
        let (next_temp, next_rpm) = {
            let r = rows.borrow();
            match r.last() {
                Some(last) => (
                    (last.temp.value() as u8).saturating_add(5).min(110),
                    last.rpm.value() as u16,
                ),
                None => (50u8, fan_range.0 as u16),
            }
        };
        add_row(comms::CurvePoint { temp_c: next_temp, rpm: next_rpm });
        push_curve();
    }));

    CurveEditor { container }
}
