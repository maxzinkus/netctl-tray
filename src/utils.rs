use qt_widgets::cpp_utils::CppBox;
use qt_widgets::qt_core::QByteArray;
use qt_gui::{QIcon, QPixmap, QImage};
use std::process::Command;
use regex::RegexBuilder;
use std::fs;
use std::thread;
use shells::sh;

// NoProfile - no profile is active
// Good/Medium/Bad/NoSignal - connection strength
// The bool - is there internet?
pub enum Status {
   NoProfile,
   Good,
   Medium,
   Bad,
   NoSignal,
}

pub struct Profile {
   pub name: String,
   pub is_active: bool
}

pub fn get_profiles() -> Vec<Profile> {
	let mut profiles = Vec::new();
   let cmd_target = if cfg!(feature = "auto") {
      "/usr/share/netctl-tray/netctl-auto-list"
   } else {
      "/usr/share/netctl-tray/netctl-list"
   };
   if cfg!(debug_assertions) {
      println!("Using {} for list", cmd_target);
   }
	// Get the list of all profiles
	let raw_profiles = Command::new("pkexec")
      .arg(cmd_target)
		.output()
		.expect("Failed to run netctl").stdout;
	// Iterate through each line
	for line in raw_profiles.split(|c| *c == '\n' as u8) {
		if line.len() == 0 { continue; }
		// If the line starts with an asterisk, then the profile is active
		let active = line[0] == ('*' as u8);
		let profile_name = std::str::from_utf8(&line[2..]).unwrap().to_string();
		profiles.push(Profile { name: profile_name.to_string(), is_active: active});
	}
	profiles
}

pub fn get_active_profile() -> Result<String, &'static str> {
   for profile in get_profiles().iter() {
      if profile.is_active {
         return Ok(profile.name.to_string());
      }
   }
   static NOPROFILEERROR: &str = "No active profile";
   return Err(NOPROFILEERROR);
}

pub fn get_status() -> Status {
	// Check if any profiles are active
   let active_profile = match get_active_profile() {
      Ok(name) => name,
      Err(_) => return Status::NoProfile
   };
   if cfg!(debug_assertions) {
      println!("Got active profile {}", active_profile);
   }
	// Finally return the status
	let conn_strength = conn_strength(&active_profile) as f32;
	match (conn_strength/24f32).ceil() as u8 {
		0u8 => Status::NoSignal,
		1u8 => Status::Bad,
		2u8 => Status::Medium,
		_   => Status::Good,
	}
}

pub fn set_profile(profile: String) {
   let cmd_target = if cfg!(feature = "auto") {
      "/usr/share/netctl-tray/netctl-auto-switch-to"
   } else {
      "/usr/share/netctl-tray/netctl-switch-to"
   };
   if cfg!(debug_assertions) {
      println!("Using {} for switch-to", cmd_target);
   }
	thread::spawn( move || {
		// Switch to the new profile
		Command::new("pkexec")
            .arg(cmd_target)
				.arg(profile)
				.output()
				.expect("Failed to run netctl");
	});
}

pub unsafe fn load_icon(path: &str) -> CppBox<QIcon> {
	QIcon::from_q_pixmap(
		QPixmap::from_image_1a(
			QImage::from_data_q_byte_array(
				QByteArray::from_slice(
					fs::read_to_string(path).unwrap().as_bytes()
				).as_ref()
			).as_ref()
		).as_ref()
	)
}

pub fn send_notification(message: &str) {
   sh!("notify-send 'netctl' '{}' -t 4000 -i network-wireless", message);
}

pub fn get_rtt_str() -> String {
   static RTTFAIL: &str = "✗";
   match get_rtt() {
      Ok(n) => n.ceil().to_string(),
      Err(_) => RTTFAIL.to_string()
   }
}

fn get_rtt() -> Result<f64, std::num::ParseFloatError> {
	let (_, ping, _) = sh!("ping -qc1 1.1.1.1 2>&1 | awk -F'/' 'END{{ print (/^rtt/? $5:\"\") }}'");
	ping.trim().to_string().parse::<f64>()
}

pub fn conn_strength(profile: &str) -> u8 {
   // TODO read the file, don't cat it
   // TODO fail gracefully if iwconfig is missing
	// Get the interface the active profile is using
	let used_interface = Command::new("cat")
			.arg("/etc/netctl/".to_owned()+profile)
			.output()
			.expect(&format!("failed to read /etc/netctl/{}", profile)).stdout;
	let used_interface = RegexBuilder::new(r"^Interface\s*=\s*(.+)$")
		.multi_line(true)
		.case_insensitive(true)
		.build().unwrap()
		.captures(std::str::from_utf8(&used_interface).unwrap())
		.expect(&format!("Couldn't read the interface from /etc/netctl/{}", profile));

	// Check the strength of the connection
	let conn_strength = Command::new("iwconfig")
			.output()
			.expect("Failed to run iwconfig").stdout;
	let conn_strength: u8 = match
		RegexBuilder::new(&((&used_interface[1]).to_string() + r"(.|\n)+?Link Quality=([0-9]+)/70"))
		.case_insensitive(true)
		.build().unwrap()
		.captures(std::str::from_utf8(&conn_strength).unwrap()) {
			Some(c) => (&c[2]).to_string().parse().unwrap(),
			None	=> 0,
		};
	conn_strength
}
