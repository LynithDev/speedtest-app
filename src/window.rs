use glib::Object;
use adw::{gio, glib, Application};

pub(crate) mod imp {

    use gio::glib::{MainContext, Priority, ControlFlow};
    use glib::subclass::InitializingObject;
    use adw::{subclass::prelude::*, glib, prelude::*, ActionRow, PasswordEntryRow, MessageDialog};
    use gtk::{CompositeTemplate, Button, traits::ButtonExt};

    use crate::{provider::{self, TestInfo, TestDownloadInfo, Provider, TestUploadInfo, TestResultInfo}, progress_bar::CircleProgressBar, providers::speedtest_net::{SpeedtestNetProvider, speedtest_cli_installed}};

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dev/lynith/speedtest/window.ui")]
    pub struct Window {
        #[template_child]
        pub start_test_button: TemplateChild<Button>,

        #[template_child]
        pub download_speed_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub upload_speed_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub isp_row: TemplateChild<adw::ActionRow>,

        #[template_child]
        pub server_row: TemplateChild<adw::ActionRow>,

        #[template_child]
        pub circle_progress_bar: TemplateChild<CircleProgressBar>,

        #[template_child]
        pub info_rows: TemplateChild<gtk::ListBox>,

        #[template_child]
        pub info_rows_revealer: TemplateChild<gtk::Revealer>,

        #[template_child]
        pub expander_row: TemplateChild<adw::ExpanderRow>
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "SpeedtestWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();

            let window = self.to_owned();

            let (
                sender, 
                receiver
            ) = MainContext::channel::<(
                Option<TestInfo>, 
                Option<TestDownloadInfo>, 
                Option<TestUploadInfo>,
                Option<TestResultInfo>
            )>(Priority::default());

            let mut provider = SpeedtestNetProvider::new();

            self.start_test_button.connect_clicked(move |_| {
                let sender = sender.clone();

                tokio::spawn(async move {
                    provider::start_speedtest(
                        &mut provider, 
                        |info| {
                            sender.send((Some(info), None, None, None)).unwrap();
                        }, 
                        |download| {
                            sender.send((None, Some(download), None, None)).unwrap();
                        },
                        |upload| {
                            sender.send((None, None, Some(upload), None)).unwrap();
                        },
                        |result| {
                            sender.send((None, None, None, Some(result))).unwrap();
                        }
                    ).await;
                });
            });

            fn reset(window: &Window, row: &ActionRow) {
                window.info_rows_revealer.set_reveal_child(false);
                window.start_test_button.set_sensitive(true);

                window.isp_row.set_subtitle("");
                window.server_row.set_subtitle("");

                window.download_speed_label.set_text("0.00");
                window.upload_speed_label.set_text("0.00");

                window.circle_progress_bar.set_percentage(0.0);
                window.circle_progress_bar.set_text_large("0.00");

                if !row.title().is_empty() {
                    window.expander_row.remove(row);
                }
            }

            let row = ActionRow::new();
            receiver.attach(
                None,
                move |msg| -> ControlFlow {
                    if msg.0.is_some() {
                        reset(&window, &row);
                        window.start_test_button.set_sensitive(false);

                        let info = msg.0.unwrap();
                        window.isp_row.set_subtitle(&info.isp);
                        window.server_row.set_subtitle(&info.name);

                        {
                            let row = ActionRow::new();
                            row.set_title("Country and location");
                            row.set_subtitle(format!("{}, {}", info.country, info.location).as_str());
                            row.set_subtitle_selectable(true);

                            window.expander_row.add_row(&row);
                        }

                        {
                            let row = PasswordEntryRow::new();
                            row.set_title("Public IP address");
                            row.set_text(&info.external_ip);
                            row.set_editable(false);
                            row.set_show_apply_button(false);

                            window.expander_row.add_row(&row);
                        }
                    }

                    if msg.1.is_some() {
                        let download = msg.1.unwrap();
                        let mbps = download.bandwidth as f32 / 125_000.0;

                        window.download_speed_label.set_text(format!("{:.2}", mbps).as_str());

                        let percent = mbps / 160.0;

                        window.circle_progress_bar.set_percentage(percent);
                        window.circle_progress_bar.set_text_large(format!("{:.0}", mbps).as_str());
                    }

                    if msg.2.is_some() {
                        let upload = msg.2.unwrap();
                        let mbps = upload.bandwidth as f32 / 125_000.0;

                        window.upload_speed_label.set_text(format!("{:.2}", mbps).as_str());

                        let percent = mbps / 160.0;

                        window.circle_progress_bar.set_percentage(percent);
                        window.circle_progress_bar.set_text_large(format!("{:.0}", mbps).as_str());
                    }

                    if msg.3.is_some() {
                        println!("Finished test");
                        window.start_test_button.set_sensitive(true);
                        let result = msg.3.unwrap();
                        let mbps_download = result.download_bandwidth as f32 / 125_000.0;
                        let mbps_upload = result.upload_bandwidth as f32 / 125_000.0;

                        window.download_speed_label.set_text(format!("{:.2}", mbps_download).as_str());
                        window.upload_speed_label.set_text(format!("{:.2}", mbps_upload).as_str());

                        window.circle_progress_bar.set_percentage(0.0);
                        window.circle_progress_bar.set_text_large("0");

                        window.info_rows_revealer.set_reveal_child(true);

                        if result.result_url.is_some() {
                            row.set_title("Result URL");
                            row.set_subtitle(result.result_url.unwrap().as_str());
                            row.set_subtitle_selectable(true);

                            window.expander_row.add_row(&row);
                        }
                    }

                    glib::ControlFlow::Continue
                }
            );
        }
    }

    impl WidgetImpl for Window {}
    impl WindowImpl for Window {}
    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        Object::builder().property("application", app).build()
    }
}