use std::time::Duration;
use serialport::SerialPort;
use std::io::{self};
use rand::Rng;
use std::f64::consts::PI;


const PORT_NAME: &str = "/dev/tty.usbserial-10"; // Adjust this to your serial port name
const BAUD_RATE: u32 = 115200;
const Z_LEVEL: u32 = 42;

const MIN_SPEED: u32 = 3000; // Minimum speed in mm/min
const MAX_SPEED: u32 = 6000; // Maximum speed in mm/min
const MAX_RADIUS: i32 = 75;


fn main() {
    let mut rng = rand::thread_rng();

    match serialport::new(PORT_NAME, BAUD_RATE)
        .timeout(Duration::from_millis(8000))
        .open() {
        Ok(mut port) => {
            println!("Connected to the printer!");
            initialize(&mut *port);
            for _ in 0..100 {
                for _ in 0..10 {
                    let speed = rng.gen_range(MIN_SPEED..MAX_SPEED);
                    draw_random_line_on_circle(&mut *port, MAX_RADIUS, speed);
                };
                draw_lines_across_circle(&mut *port, MAX_RADIUS);   
                
                for radius in (5..=MAX_RADIUS).step_by(5) {
    
                    draw_circle(&mut *port, radius, rand::thread_rng().gen_range(MIN_SPEED..=MAX_SPEED));
                }
                for radius in (5..=MAX_RADIUS).rev().step_by(5) {
            
                    draw_circle(&mut *port, radius, rand::thread_rng().gen_range(MIN_SPEED..=MAX_SPEED));
                }
            }
            stop_printer(&mut *port);
        },
        Err(e) => {
            eprintln!("Failed to open port: {}", e);
        }
    }
}


fn send_gcode(port: &mut dyn SerialPort, gcode: &str) {
    println!("Sending G-code: {}", gcode);
    port.write_all(gcode.as_bytes()).expect("Failed to write to port");
    port.write_all(b"\n").expect("Failed to write newline");

    // Wait for acknowledgment with timeout
    let mut buffer: Vec<u8> = vec![0; 64];
    let start_time = std::time::Instant::now();
    let timeout_duration = Duration::from_secs(10); // Set your desired timeout duration

    loop {
        if start_time.elapsed() > timeout_duration {
            // Timeout occurred, send commands to turn off fans and stop the printer
            eprintln!("Timeout occurred, stopping the printer.");
            stop_printer(port);
            break;
        }

        match port.read(&mut buffer) {
            Ok(bytes_read) => {
                let response = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("Received response: {}", response);
                if response.contains("ok") {
                    break;
                }
            },
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => continue,
            Err(e) => eprintln!("Failed to read from port: {}", e),
        }
    }
}

fn stop_printer(port: &mut dyn SerialPort) {
    // Send G-code commands to turn off fans and stop the printer
    let stop_commands = [
        "G28",
        "M107", // Turn off fan
        "M84"   // Disable motors
    ];

    for command in &stop_commands {
        send_gcode(&mut *port,command);
    }
}

fn draw_circle(port: &mut dyn SerialPort, radius: i32, speed: u32) {
    let move_start = format!("G1 X-{} Y0 Z{} F3000", radius, Z_LEVEL);
    send_gcode(port, &move_start);
    let gcode = format!("G2 I{} J0 Z{} F{}", radius, Z_LEVEL, speed);
    send_gcode(port, &gcode);
}

fn draw_random_line_on_circle(port: &mut dyn SerialPort, radius: i32, speed: u32) {
    let mut rng = rand::thread_rng();

    // Generate two random angles between 0 and 2π
    let angle1 = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
    let angle2 = rng.gen_range(0.0..2.0 * std::f64::consts::PI);

    // Calculate the coordinates of the points on the circle
    let x1 = radius as f64 * angle1.cos();
    let y1 = radius as f64 * angle1.sin();
    let x2 = radius as f64 * angle2.cos();
    let y2 = radius as f64 * angle2.sin();

    // Generate G-code to move to the first point
    let gcode1 = format!("G1 X{} Y{} F{}", x1, y1, speed);
    send_gcode(port, &gcode1);

    // Generate G-code to draw a line to the second point
    let gcode2 = format!("G1 X{} Y{} Z{} F{}", x2, y2, Z_LEVEL, speed);
    send_gcode(port, &gcode2);
}


fn draw_lines_across_circle(port: &mut dyn SerialPort, radius: i32) {
    let mut rng = rand::thread_rng();
    for angle in (0..180).step_by(20) {
        let mut speed = rng.gen_range(MIN_SPEED..=MAX_SPEED);
        let angle_rad = (angle as f64) * PI / 180.0;
        let opposite_angle_rad = angle_rad + PI;

        // Move to the first point
        let x1 = radius as f64 * angle_rad.cos();
        let y1 = radius as f64 * angle_rad.sin();

        let gcode1 = format!("G1 X{} Y{} Z{} F{}", x1, y1, Z_LEVEL, speed);
        send_gcode(port, &gcode1);

        // Draw a line to the opposite point
        let x2 = radius as f64 * opposite_angle_rad.cos();
        let y2 = radius as f64 * opposite_angle_rad.sin();
        speed = rng.gen_range(MIN_SPEED..=MAX_SPEED);
        let gcode2 = format!("G1 X{} Y{} F{}", x2, y2, speed);
        send_gcode(port, &gcode2);

        // Move to the opposite point to start the next line
        let next_angle_rad = ((angle + 5) as f64) * PI / 180.0;
        let next_x1 = radius as f64 * next_angle_rad.cos();
        let next_y1 = radius as f64 * next_angle_rad.sin();
        speed = rng.gen_range(MIN_SPEED..=MAX_SPEED);
        let gcode3 = format!("G1 X{} Y{} F{}", next_x1, next_y1, speed);
        send_gcode(port, &gcode3);

        // Move along circle
        let next_opposite_angle_rad = next_angle_rad + PI;
        let next_x2 = radius as f64 * next_opposite_angle_rad.cos();
        let next_y2 = radius as f64 * next_opposite_angle_rad.sin();
        speed = rng.gen_range(MIN_SPEED..=MAX_SPEED);
        let gcode4 = format!("G1 X{} Y{} F{}", next_x2, next_y2, speed);
        send_gcode(port, &gcode4);
    }
}

fn initialize(port: &mut dyn SerialPort) {
    let fan_off: String = format!("M107 S1");
    send_gcode(port, &fan_off);
    let gcode = format!("G28");
    send_gcode(port, &gcode);
    let mode_above_origo = format!("G1 X0 Y0 Z50 F8000");
    send_gcode(port, &mode_above_origo);
    // Add more G-code commands to draw patterns on sand
}
