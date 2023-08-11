use gtk::{glib, prelude::*, subclass::prelude::*};

pub(crate) mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::{graphene::Rect, cairo};

    use super::*;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::CircleProgressBar)]
    pub struct CircleProgressBar {
        #[property(get, set = Self::set_percentage, default = 0.0, explicit_notify)]
        pub percentage: Cell<f32>,

        #[property(name = "text-large", get, set)]
        pub text_large: RefCell<String>,

        #[property(name = "text-small", get, set)]
        pub text_small: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CircleProgressBar {
        const NAME: &'static str = "CircleProgressBar";
        type Type = super::CircleProgressBar;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for CircleProgressBar {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_valign(gtk::Align::Center);
        }
    }

    impl WidgetImpl for CircleProgressBar {
        #[allow(deprecated)]
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();

            let width = widget.width() as f64;
            let height = widget.height() as f64;
            let line_width = 7.5;
            
            let center_x = width / 2.0;
            let center_y = height / 2.0;

            let radius = (height / 2.0) - line_width;

            let percentage = self.percentage.get().clamp(0.0, 1.0) as f64;
            let start_angle = 129.0 * std::f64::consts::PI / 180.0;
            let end_angle = 410.0 * std::f64::consts::PI / 180.0;

            let ctx = snapshot.append_cairo(&Rect::new(0.0, 0.0, widget.width() as f32, widget.height() as f32));

            let accent_color = widget.style_context().lookup_color("accent_bg_color").unwrap();
            let background_color = widget.style_context().lookup_color("shade_color").unwrap();
            let text_color = widget.style_context().lookup_color("card_fg_color").unwrap();

            // Unfilled
            ctx.set_source_rgba(
                background_color.red().into(),
                background_color.green().into(),
                background_color.blue().into(),
                background_color.alpha().into(),
            );

            ctx.set_antialias(cairo::Antialias::Best);
            ctx.arc(center_x, center_y, radius, start_angle, end_angle);
            ctx.set_line_cap(cairo::LineCap::Round);
            ctx.set_line_width(line_width);
            let _ = ctx.stroke();

            // Filled
            if percentage != 0.0 {
                ctx.set_source_rgba(
                    accent_color.red().into(),
                    accent_color.green().into(),
                    accent_color.blue().into(),
                    accent_color.alpha().into(),
                );
                
                ctx.set_antialias(cairo::Antialias::Best);
                ctx.arc(center_x, center_y, radius, start_angle, start_angle + (percentage * (end_angle - start_angle)));
                ctx.set_line_cap(cairo::LineCap::Round);
                ctx.set_line_width(line_width);
                let _ = ctx.stroke();
            }

            let large = widget.text_large();
            let large_font_size = (if large.len() <= 3 { 
                56.0 
            } else { 
                56.0 - (large.len() as f64 * 4.0) 
            }).max(16.0);

            ctx.set_source_rgba(
                text_color.red().into(),
                text_color.green().into(),
                text_color.blue().into(),
                text_color.alpha().into(),
            );
            ctx.set_font_size(large_font_size);
            ctx.select_font_face("Cantarell", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            ctx.move_to(center_x - (ctx.text_extents(&large).unwrap().width() / 2.0) - 2.0, center_y + 10.0);
            ctx.text_path(&large);
            let _ = ctx.fill();

            let small = widget.text_small();

            ctx.set_source_rgba(
                text_color.red().into(),
                text_color.green().into(),
                text_color.blue().into(),
                text_color.alpha().into(),
            );
            ctx.set_font_size(16.0);
            ctx.select_font_face("Cantarell", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
            ctx.move_to(center_x - (ctx.text_extents(&small).unwrap().width() / 2.0) - 2.0, center_y + 30.0);
            ctx.text_path(&small);
            let _ = ctx.fill();
        }
    }

    impl CircleProgressBar {
        fn set_percentage(&self, percentage: f32) {
            let obj = self.obj();
            if (percentage - obj.percentage()).abs() < f32::EPSILON {
                return;
            }
            let clamped = percentage.clamp(0.0, 1.0);
            self.percentage.replace(clamped);
            obj.queue_draw();
            obj.notify_percentage();
        }
    }
}

glib::wrapper! {
    pub struct CircleProgressBar(ObjectSubclass<imp::CircleProgressBar>)
        @extends gtk::Widget;
}

impl CircleProgressBar {}
