use std::process::exit;

use adw::{glib, Application, MessageDialog, traits::MessageDialogExt};
use gtk::prelude::*;
use speedtest::{window::Window, progress_bar::CircleProgressBar, providers::speedtest_net::speedtest_cli_installed};

pub const APP_ID: &str = "dev.lynith.Speedtest";

#[tokio::main]
async fn main() -> glib::ExitCode {
    CircleProgressBar::static_type();
    gio::resources_register_include!("speedtest.gresource")
        .expect("Failed to register resources.");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_shutdown(|_| {
        exit(0);
    });

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let window = Window::new(app);
    window.set_icon_name(Some("dev.lynith.Speedtest"));
    window.present();

    if !speedtest_cli_installed() {
        let dialog = MessageDialog::new(
            Some(&window), 
            Some("Proprietary software not installed"), 
            Some("In order to use this proprietary provider, you have to agree to Ookla's EULA and the installation of a proprietary executable from their servers.")
        );
        dialog.present();

        dialog.add_response("disagree", "Disagree");
        dialog.add_response("agree", "Agree");

        dialog.set_response_appearance("agree", adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some("disagree"));
        
        dialog.connect_response(Some("agree"), |dialog, _| {
            dialog.close();
        });

        dialog.connect_response(Some("disagree"), |dialog, _| {
            dialog.close();
            exit(0)
        });
    }
}