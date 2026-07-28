/* performance_page/npu_details.rs
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

use glib::{ParamSpec, Properties, Value};
use gtk::prelude::WidgetExt;
use gtk::{glib, subclass::prelude::*};

mod imp {
    use super::*;

    #[derive(Properties)]
    #[properties(wrapper_type = super::NpuDetails)]
    #[derive(gtk::CompositeTemplate)]
    #[template(resource = "/io/missioncenter/MissionCenter/ui/performance_page/npu_details.ui")]
    pub struct NpuDetails {
        #[template_child]
        pub activity: TemplateChild<gtk::Label>,
        #[template_child]
        pub status_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub hwctx_count_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub context_count_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub partition_count_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub driver_version: TemplateChild<gtk::Label>,
        #[template_child]
        pub xrt_version: TemplateChild<gtk::Label>,
        #[template_child]
        pub pci_addr: TemplateChild<gtk::Label>,
    }

    impl Default for NpuDetails {
        fn default() -> Self {
            Self {
                activity: TemplateChild::default(),
                status_label: TemplateChild::default(),
                hwctx_count_label: TemplateChild::default(),
                context_count_label: TemplateChild::default(),
                partition_count_label: TemplateChild::default(),
                driver_version: TemplateChild::default(),
                xrt_version: TemplateChild::default(),
                pci_addr: TemplateChild::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NpuDetails {
        const NAME: &'static str = "NpuDetails";
        type Type = super::NpuDetails;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NpuDetails {
        fn properties() -> &'static [ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
            self.derived_property(id, pspec)
        }
    }

    impl WidgetImpl for NpuDetails {}

    impl BoxImpl for NpuDetails {}
}

glib::wrapper! {
    pub struct NpuDetails(ObjectSubclass<imp::NpuDetails>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::ConstraintTarget, gtk::Accessible, gtk::Buildable;
}

impl NpuDetails {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_collapsed(&self, collapsed: bool) {
        if collapsed {
            self.set_margin_top(10);
        } else {
            self.set_margin_top(65);
        }
    }

    pub fn activity(&self) -> &gtk::Label {
        &self.imp().activity
    }

    pub fn status_label(&self) -> &gtk::Label {
        &self.imp().status_label
    }

    pub fn hwctx_count_label(&self) -> &gtk::Label {
        &self.imp().hwctx_count_label
    }

    pub fn context_count_label(&self) -> &gtk::Label {
        &self.imp().context_count_label
    }

    pub fn partition_count_label(&self) -> &gtk::Label {
        &self.imp().partition_count_label
    }

    pub fn driver_version(&self) -> &gtk::Label {
        &self.imp().driver_version
    }

    pub fn xrt_version(&self) -> &gtk::Label {
        &self.imp().xrt_version
    }

    pub fn pci_addr(&self) -> &gtk::Label {
        &self.imp().pci_addr
    }
}
