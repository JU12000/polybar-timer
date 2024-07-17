use clap::{value_parser, Arg, Command};
use rodio::{Decoder, OutputStream, Sink};
use std::{fs::{self, File}, io::{BufReader, ErrorKind}, ops::Add, process, str, thread, time::{Duration, SystemTime, UNIX_EPOCH}};

const TEMP_DIR: &str = "/tmp/polybar-timer";

fn main() {
	let args = Command::new("Polybar Timer")
		.about("Manage a timer with Rust. This should be called from a polybar config")
		.subcommand(Command::new("new")
			.about("Create a new timer if none exists.")
			.arg(
				Arg::new("minutes")
					.value_name("MINUTES")
					.value_parser(value_parser!(u64))
					.required(true)
					.help("The length in minutes to set the new timer to")
			)
		)
		.subcommand(Command::new("cancel")
			.about("Cancel the current timer if it exists.")
		)
		.subcommand(Command::new("increase")
			.about("Increase the current timer or create a new one.")
			.arg(
				Arg::new("seconds")
					.value_name("SECONDS")
					.value_parser(value_parser!(u64))
					.help("The length in seconds to increase the timer by")
			)
		)
		.subcommand(Command::new("toggle")
			.about("Play or pause the current timer if it exists.")
		)
		.subcommand(Command::new("tail")
			.about("Print the remaining time.")
			.arg(
				Arg::new("play_icon")
					.short('r')
					.long("play-icon")
					.value_name("PLAY_ICON")
					.value_parser(value_parser!(String))
					.default_value("⏵")
					.help("The icon to display when the timer is running")
			)
			.arg(
				Arg::new("pause_icon")
					.short('p')
					.long("pause-icon")
					.value_name("PAUSE_ICON")
					.value_parser(value_parser!(String))
					.default_value("⏸")
					.help("The icon to display when the timer is paused")
			)
		)
		.arg_required_else_help(true)
		.get_matches();

	match args.subcommand() {
		Some(("new", sub_args)) => {
			if !exists("expiry") {
				let minutes: u64 = *sub_args.get_one("minutes").unwrap();
				create_timer(Duration::from_secs(60 * minutes));
			}
		}
		Some(("cancel", _)) => {
			kill_timer_if_exists();
		}
		Some(("increase", sub_args)) => {
			let seconds: u64 = *sub_args.get_one("seconds").unwrap();
			
			if exists("expiry") {
				increase_timer(Duration::from_secs(seconds));
			}
			else {
				create_timer(Duration::from_secs(seconds));
			}
		}
		Some(("toggle", _)) => {
			if exists("expiry") {
				if exists("paused") {
					let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
					let paused = read_duration("paused");
					let delta = now.saturating_sub(paused);

					if delta != Duration::ZERO {
						increase_timer(delta);
					}

					fs::remove_file(format!("{TEMP_DIR}/paused")).unwrap();
				}
				else {
					let paused = SystemTime::now()
						.duration_since(UNIX_EPOCH)
						.unwrap()
						.as_secs()
						.to_string();
					
					fs::write(format!("{TEMP_DIR}/paused"), paused).ok();
				}
			}
		}
		Some(("tail", sub_args)) => {
			let play_icon: &str = sub_args.get_one::<String>("play_icon").unwrap().as_str();
			let pause_icon: &str = sub_args.get_one::<String>("pause_icon").unwrap().as_str();

			while exists("expiry") {
				let icon;

				let expiry = read_duration("expiry");
				let remaining;

				if exists("paused") {
					icon = pause_icon;
					let paused = read_duration("paused");
					remaining = expiry.saturating_sub(paused);
				}
				else {
					icon = play_icon;
					let now = SystemTime::now()
						.duration_since(UNIX_EPOCH)
						.unwrap();
					remaining = expiry.saturating_sub(now);
				}

				print_timer(icon, remaining);
				thread::sleep(Duration::from_millis(250));

				if remaining.as_secs() < 1 {
					play_notification();
					kill_timer_if_exists();
				}
			}

			println!("");
		}
		_ => {
			process::exit(1)
		}
	}
}

fn kill_timer_if_exists() {
	if fs::remove_file(format!("{TEMP_DIR}/expiry"))
		.is_err_and(|e| e.kind() == ErrorKind::PermissionDenied) ||
		fs::remove_file(format!("{TEMP_DIR}/paused"))
		.is_err_and(|e| e.kind() == ErrorKind::PermissionDenied) {
		panic!("Insufficient permissions! Try manually deleting /tmp/polybar-timer and creating a new timer");
	}
}

fn create_timer(duration: Duration) {
	let expiry = SystemTime::now()
		.add(duration)
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs()
		.to_string();
	
	if fs::create_dir_all(TEMP_DIR)
		.is_err_and(|e| e.kind() == ErrorKind::PermissionDenied) {
		panic!("Insufficient permissions! polybar-timer needs permission to write to /tmp.")
	}
	fs::write(format!("{TEMP_DIR}/expiry"), expiry).ok();
}

fn exists(file: &str) -> bool {
	fs::metadata(format!("{TEMP_DIR}/{file}")).is_ok_and(|m| m.is_file())
}

fn increase_timer(duration: Duration) {
	let expiry = read_duration("expiry")
		.saturating_add(duration)
		.as_secs();

	let now = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs();
	if expiry > now {
		fs::write(format!("{TEMP_DIR}/expiry"), expiry.to_string()).unwrap();
	}
	else {
		kill_timer_if_exists();
		create_timer(duration);
	}
}

fn read_duration(file: &str) -> Duration {
	let duration_vec = fs::read(format!("{TEMP_DIR}/{file}")).unwrap();
	let duration_string = str::from_utf8(&duration_vec).unwrap();

	Duration::from_secs(
		u64::from_str_radix(duration_string, 10).unwrap()
	)
}

fn print_timer(icon: &str, duration: Duration) {
	let seconds = duration.as_secs() % 60;
	let minutes = (duration.as_secs() - seconds) / 60;

	println!("{} {:02}:{:02}", icon, minutes, seconds);
}

fn play_notification() {
	let (_stream,stream_handle) = OutputStream::try_default()
		.unwrap();
	let sink = Sink::try_new(&stream_handle).unwrap();

	let notify = File::open("notify.ogg").unwrap();
	let source = Decoder::new(BufReader::new(notify))
		.unwrap();

	sink.append(source);

	sink.sleep_until_end();
}
