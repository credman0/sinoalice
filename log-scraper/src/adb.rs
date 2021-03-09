use std::process::Command;

pub fn cap_screen () -> Vec<u8> {
    let output = Command::new("adb")
            .arg("shell")
            .arg("screencap -p")
            .output()
            .expect("failed to execute capture");
    
    return output.stdout;
}

pub fn tap (x:u32, y:u32) {
    Command::new("adb")
            .arg("shell")
            .arg("input")
            .arg("tap")
            .arg(x.to_string())
            .arg(y.to_string())
            .output()
            .expect("failed to execute tap");
}

pub fn swipe (x:u32, y:u32, endx:u32, endy:u32, duration:Option<u32>) {
    let mut command = Command::new("adb");
        command.arg("shell")
            .arg("input")
            .arg("swipe")
            .arg(x.to_string())
            .arg(y.to_string())
            .arg(endx.to_string())
            .arg(endy.to_string());
    if duration.is_some() {
        command.arg(duration.unwrap().to_string());
    }
    command.output().expect("failed to execute swipe");
}