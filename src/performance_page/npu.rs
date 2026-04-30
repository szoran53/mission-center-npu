/* performance_page/npu.rs
 *
 * Copyright 2026 Mission Center Developers
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::cell::{Cell, RefCell};

use adw::{self, subclass::prelude::*};
use glib::{ParamSpec, Properties, Value};
use gtk::{gio, glib, prelude::*};

use magpie_types::npu::Npu;

use crate::{application::INTERVAL_STEP, settings, to_short_human_readable_time};

use crate::performance_page::widgets::{DatasetGroup, GraphWidget, ScalingSettings};

use super::PageExt;

const NPU_ACTIVE_THRESHOLD_HZ: f64 = 5.0;

mod imp {
    use super::*;

    #[derive(Properties)]
    #[properties(wrapper_type = super::PerformancePageNpu)]
    #[derive(gtk::CompositeTemplate)]
    #[template(resource = "/io/missioncenter/MissionCenter/ui/performance_page/npu.ui")]
    pub struct PerformancePageNpu {
        #[template_child]
        pub npu_id: TemplateChild<gtk::Label>,
        #[template_child]
        pub device_name: TemplateChild<gtk::Label>,
        #[template_child]
        pub status_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub graph_activity: TemplateChild<GraphWidget>,
        #[template_child]
        pub graph_max_duration: TemplateChild<gtk::Label>,
        #[template_child]
        pub hwctx_count_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub context_count_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub partition_count_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub context_menu: TemplateChild<gtk::Popover>,

        #[property(get = Self::name, set = Self::set_name, type = String)]
        name: RefCell<String>,
        #[property(get, set)]
        base_color: Cell<gtk::gdk::RGBA>,
        #[property(get, set)]
        summary_mode: Cell<bool>,

        // ring buffer for windowed-peak Y scaling
        peak_ring: RefCell<[f64; 60]>,
        peak_ring_pos: Cell<usize>,
    }

    impl Default for PerformancePageNpu {
        fn default() -> Self {
            Self {
                npu_id: Default::default(),
                device_name: Default::default(),
                status_label: Default::default(),
                graph_activity: Default::default(),
                graph_max_duration: Default::default(),
                hwctx_count_label: Default::default(),
                context_count_label: Default::default(),
                partition_count_label: Default::default(),
                context_menu: Default::default(),

                name: RefCell::new(String::new()),
                base_color: Cell::new(gtk::gdk::RGBA::new(0.0, 0.0, 0.0, 1.0)),
                summary_mode: Cell::new(false),

                peak_ring: RefCell::new([0.0_f64; 60]),
                peak_ring_pos: Cell::new(0),
            }
        }
    }

    impl PerformancePageNpu {
        fn name(&self) -> String {
            self.name.borrow().clone()
        }

        fn set_name(&self, name: String) {
            if name == *self.name.borrow() {
                return;
            }
            self.name.replace(name);
        }
    }

    impl PerformancePageNpu {
        fn configure_context_menu(this: &super::PerformancePageNpu) {
            let right_click_controller = gtk::GestureClick::new();
            right_click_controller.set_button(3); // Secondary click (right click)
            right_click_controller.connect_released({
                let this = this.downgrade();
                move |_click, _n_press, x, y| {
                    let this = match this.upgrade() {
                        Some(this) => this,
                        None => return,
                    };
                    let this = this.imp();

                    this.context_menu
                        .set_pointing_to(Some(&gtk::gdk::Rectangle::new(
                            x.round() as i32,
                            y.round() as i32,
                            1,
                            1,
                        )));
                    this.context_menu.popup();
                }
            });
            this.add_controller(right_click_controller);
        }

        fn configure_actions(this: &super::PerformancePageNpu) {
            let actions = gio::SimpleActionGroup::new();
            this.insert_action_group("graph", Some(&actions));

            let action = gio::SimpleAction::new("copy", None);
            action.connect_activate({
                let this = this.downgrade();
                move |_, _| {
                    if let Some(this) = this.upgrade() {
                        let clipboard = this.clipboard();
                        clipboard.set_text(this.imp().data_summary().as_str());
                    }
                }
            });
            actions.add_action(&action);
        }

        fn data_summary(&self) -> String {
            format!(
                r#"{}

    {}

    Status:          {}
    HW Contexts:     {}
    Contexts:        {}
    Partitions:      {}"#,
                self.npu_id.label(),
                self.device_name.label(),
                self.status_label.label(),
                self.hwctx_count_label.label(),
                self.context_count_label.label(),
                self.partition_count_label.label(),
            )
        }
    }

    impl PerformancePageNpu {
        pub fn set_static_information(
            this: &super::PerformancePageNpu,
            npu: &Npu,
        ) -> bool {
            let this = this.imp();

            this.npu_id.set_text("NPU");

            let device_id = npu
                .info
                .as_ref()
                .and_then(|i| i.device_id.clone())
                .unwrap_or_default();
            this.device_name.set_text(&device_id);

            true
        }

        pub fn update_readings(
            this: &super::PerformancePageNpu,
            npu: &Npu,
        ) -> bool {
            let this = this.imp();

            let irq_rate = npu
                .dynamic
                .as_ref()
                .and_then(|d| d.irq_rate_hz)
                .unwrap_or(0.0);
            let hwctx_count = npu
                .dynamic
                .as_ref()
                .and_then(|d| d.hwctx_count)
                .unwrap_or(0);
            let context_count = npu
                .dynamic
                .as_ref()
                .and_then(|d| d.context_count)
                .unwrap_or(0);
            let partition_count = npu
                .dynamic
                .as_ref()
                .and_then(|d| d.partition_count)
                .unwrap_or(0);

            // Update status label and CSS classes
            let status = if irq_rate > NPU_ACTIVE_THRESHOLD_HZ {
                "COMPUTING"
            } else if hwctx_count > 0 {
                "LOADED"
            } else {
                "IDLE"
            };
            this.status_label.set_text(status);
            this.status_label.remove_css_class("npu-computing");
            this.status_label.remove_css_class("npu-loaded");
            match status {
                "COMPUTING" => this.status_label.add_css_class("npu-computing"),
                "LOADED" => this.status_label.add_css_class("npu-loaded"),
                _ => {}
            }

            // Update info labels
            this.hwctx_count_label
                .set_text(&format!("{}", hwctx_count));
            this.context_count_label
                .set_text(&format!("{}", context_count));
            this.partition_count_label
                .set_text(&format!("{}", partition_count));

            // Update activity graph with windowed-peak Y scaling
            this.update_activity_graph(irq_rate);

            true
        }

        pub(crate) fn update_animations(
            this: &super::PerformancePageNpu,
            new_ticks: f32,
        ) -> bool {
            this.imp().graph_activity.update_animation(new_ticks);
            true
        }

        fn update_activity_graph(&self, irq_rate: f64) {
            // Update peak ring buffer
            let pos = self.peak_ring_pos.get();
            {
                let mut ring = self.peak_ring.borrow_mut();
                ring[pos] = irq_rate;
            }
            self.peak_ring_pos.set((pos + 1) % 60);

            // Windowed peak with 10% headroom, minimum 10.0
            let peak = self
                .peak_ring
                .borrow()
                .iter()
                .cloned()
                .fold(0.0_f64, f64::max);
            let y_max = (peak * 1.1).max(10.0);

            self.graph_activity
                .set_all_datasets_scaling(ScalingSettings::Fixed);
            self.graph_activity.set_dataset_max_scale(0, y_max as f32);
            self.graph_activity
                .add_data_point(vec![vec![irq_rate as f32]]);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PerformancePageNpu {
        const NAME: &'static str = "PerformancePageNpu";
        type Type = super::PerformancePageNpu;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PerformancePageNpu {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            let settings = settings!();

            let activity = DatasetGroup::new();
            self.graph_activity.add_dataset(activity);
            self.graph_activity.connect_to_settings(&settings);

            let this = self.obj();
            Self::configure_actions(&this);
            Self::configure_context_menu(&this);
        }
    }

    impl WidgetImpl for PerformancePageNpu {}

    impl BoxImpl for PerformancePageNpu {}
}

glib::wrapper! {
    pub struct PerformancePageNpu(ObjectSubclass<imp::PerformancePageNpu>)
        @extends gtk::Box, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::ConstraintTarget, gtk::Accessible, gtk::Buildable;
}

impl PageExt for PerformancePageNpu {
    fn infobar_collapsed(&self) {
        // No separate infobar for NPU v1 — all info is inline
    }

    fn infobar_uncollapsed(&self) {
        // No separate infobar for NPU v1 — all info is inline
    }
}

impl PerformancePageNpu {
    pub fn new(name: &str) -> Self {
        fn update_refresh_rate_sensitive_labels(
            this: &PerformancePageNpu,
            settings: &gio::Settings,
        ) {
            let this = this.imp();

            let data_points = settings.int("performance-page-data-points") as u32;
            let delay = settings.uint64("app-update-interval-u64");

            let graph_max_duration =
                (((delay as f64) * INTERVAL_STEP) * (data_points as f64)).round() as u32;

            this.graph_max_duration
                .set_text(&to_short_human_readable_time(graph_max_duration));
        }

        let this: Self = glib::Object::builder().property("name", name).build();
        let settings = settings!();
        update_refresh_rate_sensitive_labels(&this, &settings);

        settings.connect_changed(Some("performance-page-data-points"), {
            let this = this.downgrade();
            move |settings, _| {
                if let Some(this) = this.upgrade() {
                    update_refresh_rate_sensitive_labels(&this, settings);
                }
            }
        });

        settings.connect_changed(Some("app-update-interval"), {
            let this = this.downgrade();
            move |settings, _| {
                if let Some(this) = this.upgrade() {
                    update_refresh_rate_sensitive_labels(&this, settings);
                }
            }
        });

        this
    }

    pub fn set_static_information(&self, npu: &Npu) -> bool {
        imp::PerformancePageNpu::set_static_information(self, npu)
    }

    pub fn update_readings(&self, npu: &Npu) -> bool {
        imp::PerformancePageNpu::update_readings(self, npu)
    }

    pub fn update_animations(&self, new_ticks: f32) -> bool {
        imp::PerformancePageNpu::update_animations(self, new_ticks)
    }
}
