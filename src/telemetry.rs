use std::io::ErrorKind;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::config::{
    ACCEL_OFFSET, CAR_CLASS_OFFSET, CAR_ORDINAL_OFFSET, CAR_PERFORMANCE_INDEX_OFFSET,
    CURRENT_ENGINE_RPM_OFFSET, CURRENT_GEAR_OFFSET, DEFAULT_LISTEN_HOST, DEFAULT_LISTEN_PORT,
    ENGINE_MAX_RPM_OFFSET, HANDBRAKE_OFFSET, IS_RACE_ON_OFFSET, NUM_CYLINDERS_OFFSET, PACKET_SIZE,
    POWER_OFFSET, TORQUE_OFFSET,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CarId {
    pub ordinal: i32,
    pub class: i32,
    pub pi: i32,
}

#[derive(Clone, Copy)]
pub(crate) struct TelemetrySample {
    pub rpm: f32,
    pub max_rpm: f32,
    pub power: f32,
    pub torque: f32,
    pub accel: u8,
    pub handbrake: u8,
    pub current_gear: u8,
    pub is_race_on: i32,
    pub car_ordinal: i32,
    pub car_class: i32,
    pub car_pi: i32,
    pub num_cylinders: i32,
}

impl TelemetrySample {
    pub(crate) fn car_id(self) -> Option<CarId> {
        if self.is_race_on == 0 || self.car_ordinal == 0 || self.max_rpm <= 1.0 {
            return None;
        }
        Some(CarId {
            ordinal: self.car_ordinal,
            class: self.car_class,
            pi: self.car_pi,
        })
    }
}

fn default_listen_addr() -> SocketAddr {
    SocketAddr::new(
        DEFAULT_LISTEN_HOST
            .parse()
            .expect("DEFAULT_LISTEN_HOST must be a valid IP address"),
        DEFAULT_LISTEN_PORT,
    )
}

static QUEUE: OnceLock<Arc<Mutex<Vec<TelemetrySample>>>> = OnceLock::new();
static STOP: AtomicBool = AtomicBool::new(false);
static GENERATION: AtomicU64 = AtomicU64::new(0);
static THREAD: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
static ADDR: Mutex<SocketAddr> = Mutex::new(SocketAddr::V4(SocketAddrV4::new(
    Ipv4Addr::LOCALHOST,
    DEFAULT_LISTEN_PORT,
)));
static BIND_ERROR: Mutex<Option<String>> = Mutex::new(None);

pub(crate) fn listen_addr() -> SocketAddr {
    *ADDR.lock().unwrap()
}

pub(crate) fn listen_host_port() -> (String, u16) {
    let addr = listen_addr();
    (addr.ip().to_string(), addr.port())
}

pub(crate) fn listen_addr_display() -> String {
    listen_addr().to_string()
}

pub(crate) fn listen_generation() -> u64 {
    GENERATION.load(Ordering::Relaxed)
}

pub(crate) fn listen_error() -> Option<String> {
    BIND_ERROR.lock().unwrap().clone()
}

pub(crate) fn parse_listen_addr(host: &str, port: &str) -> Result<SocketAddr, String> {
    let host = host.trim();
    let port = port.trim();
    if host.is_empty() {
        return Err("请输入监听地址。".into());
    }
    if port.is_empty() {
        return Err("请输入端口。".into());
    }
    let port: u16 = port
        .parse()
        .map_err(|_| "端口必须是 0 到 65535 的整数。".to_string())?;
    let ip = if host.eq_ignore_ascii_case("localhost") {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    } else {
        host.parse::<IpAddr>()
            .map_err(|_| "监听地址必须是有效的 IP。".to_string())?
    };
    Ok(SocketAddr::new(ip, port))
}

pub(crate) fn spawn_udp_listener() -> Arc<Mutex<Vec<TelemetrySample>>> {
    QUEUE
        .get_or_init(|| {
            let queue = Arc::new(Mutex::new(Vec::new()));
            bind_and_spawn(listen_addr(), queue.clone());
            queue
        })
        .clone()
}

pub(crate) fn apply_listen_addr(addr: SocketAddr) -> Result<(), String> {
    let queue = spawn_udp_listener();
    let current = listen_addr();
    let running = THREAD.lock().unwrap().is_some();
    if running && current == addr && listen_error().is_none() {
        return Ok(());
    }

    if running && current == addr {
        stop_thread();
        bind_and_spawn(addr, queue);
        return listen_error().map_or(Ok(()), Err);
    }

    let socket = open_socket(addr)?;
    stop_thread();
    STOP.store(false, Ordering::SeqCst);
    set_addr(addr);
    set_error(None);
    let handle = thread::spawn(move || recv_loop(socket, queue));
    *THREAD.lock().unwrap() = Some(handle);
    bump_generation();
    crate::user_config::persist();
    Ok(())
}

pub(crate) fn apply_default_listen_addr() -> Result<(), String> {
    apply_listen_addr(default_listen_addr())
}

pub(crate) fn configure_listen_addr(host: &str, port: u16) {
    match parse_listen_addr(host, &port.to_string()) {
        Ok(addr) => set_addr(addr),
        Err(_) => set_addr(default_listen_addr()),
    }
}

fn bind_and_spawn(addr: SocketAddr, queue: Arc<Mutex<Vec<TelemetrySample>>>) {
    match open_socket(addr) {
        Ok(socket) => {
            STOP.store(false, Ordering::SeqCst);
            set_addr(addr);
            set_error(None);
            let handle = thread::spawn(move || recv_loop(socket, queue));
            *THREAD.lock().unwrap() = Some(handle);
        }
        Err(err) => {
            set_addr(addr);
            set_error(Some(err));
        }
    }
    bump_generation();
}

fn stop_thread() {
    STOP.store(true, Ordering::SeqCst);
    if let Some(handle) = THREAD.lock().unwrap().take() {
        let _ = handle.join();
    }
}

fn open_socket(addr: SocketAddr) -> Result<UdpSocket, String> {
    let socket = UdpSocket::bind(addr).map_err(|err| format!("无法绑定 {addr}：{err}"))?;
    socket
        .set_read_timeout(Some(Duration::from_millis(200)))
        .map_err(|err| format!("设置监听超时失败：{err}"))?;
    Ok(socket)
}

fn recv_loop(socket: UdpSocket, queue: Arc<Mutex<Vec<TelemetrySample>>>) {
    let mut buf = [0u8; PACKET_SIZE];
    loop {
        if STOP.load(Ordering::SeqCst) {
            break;
        }
        match socket.recv(&mut buf) {
            Ok(n) => {
                if let Some(sample) = parse_telemetry(&buf[..n]) {
                    queue.lock().unwrap().push(sample);
                }
            }
            Err(err)
                if err.kind() == ErrorKind::TimedOut || err.kind() == ErrorKind::WouldBlock => {}
            Err(_) => {}
        }
    }
}

fn set_addr(addr: SocketAddr) {
    *ADDR.lock().unwrap() = addr;
}

fn set_error(error: Option<String>) {
    *BIND_ERROR.lock().unwrap() = error;
}

fn bump_generation() {
    GENERATION.fetch_add(1, Ordering::Relaxed);
}

fn parse_telemetry(buf: &[u8]) -> Option<TelemetrySample> {
    if buf.len() != PACKET_SIZE {
        return None;
    }
    Some(TelemetrySample {
        max_rpm: f32_le(buf, ENGINE_MAX_RPM_OFFSET)?,
        rpm: f32_le(buf, CURRENT_ENGINE_RPM_OFFSET)?,
        power: f32_le(buf, POWER_OFFSET)?,
        torque: f32_le(buf, TORQUE_OFFSET)?,
        accel: buf[ACCEL_OFFSET],
        handbrake: buf[HANDBRAKE_OFFSET],
        current_gear: buf[CURRENT_GEAR_OFFSET],
        is_race_on: i32_le(buf, IS_RACE_ON_OFFSET)?,
        car_ordinal: i32_le(buf, CAR_ORDINAL_OFFSET)?,
        car_class: i32_le(buf, CAR_CLASS_OFFSET)?,
        car_pi: i32_le(buf, CAR_PERFORMANCE_INDEX_OFFSET)?,
        num_cylinders: i32_le(buf, NUM_CYLINDERS_OFFSET)?,
    })
}

fn f32_le(buf: &[u8], offset: usize) -> Option<f32> {
    Some(f32::from_le_bytes(buf[offset..offset + 4].try_into().ok()?))
}

fn i32_le(buf: &[u8], offset: usize) -> Option<i32> {
    Some(i32::from_le_bytes(buf[offset..offset + 4].try_into().ok()?))
}
