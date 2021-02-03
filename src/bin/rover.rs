use command::RoverStdout;
use robot_panic::setup_panic;
use rover::*;
use sputnik::Session;
use structopt::StructOpt;

use std::{process, thread};

fn main() {
    setup_panic!();
    if let Err(error) = run() {
        eprint!("{}", error);
        process::exit(1)
    } else {
        process::exit(0)
    }
}

fn run() -> Result<()> {
    let app = cli::Rover::from_args();
    timber::init(app.log_level);
    log::trace!("command structure:\n{:?}", &app);

    // attempt to create a new `Session` to capture anonymous usage data
    let output: RoverStdout = match Session::new(&app) {
        // if successful, report the usage data in the background
        Ok(session) => {
            // kicks off the reporting on a background thread
            let report_thread = thread::spawn(move || {
                // log + ignore errors because it is not in the critical path
                let _ = session.report().map_err(|telemetry_error| {
                    log::debug!("{:?}", &telemetry_error);
                    telemetry_error
                });
            });

            // kicks off the app on the main thread
            // don't return an error with ? quite yet
            // since we still want to report the usage data
            let app_result = app.run();

            // makes sure the reporting finishes in the background
            // before continuing.
            // ignore errors because it is not in the critical path
            let _ = report_thread.join();

            // return result of app execution
            // now that we have reported our usage data
            app_result
        }

        // otherwise just run the app without reporting
        Err(_) => app.run(),
    }?;

    output.print();
    Ok(())
}
